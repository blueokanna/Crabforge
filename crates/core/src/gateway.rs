use std::{
    borrow::Cow,
    collections::{BTreeMap, VecDeque},
    ops::Range,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::upstream::ForwardClient;
use anyhow::Result;
use axum_core::{
    body::Body,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use futures_util::StreamExt;
use http::{HeaderMap, Method, Request, StatusCode, header};
use nextjson::{NsonSerialize, json};
use subtle::ConstantTimeEq;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use url::Url;

mod cors;
mod first_content;
mod native;
mod patience;
pub use native::{NativeProtocols, native_protocols};
mod ollama;
mod round;
mod routing;
mod session;

use crate::config::{Config, Provider};
use crate::translate::{self, Format};
use crate::webchat::{Site, WebPrompt};
use round::Round;
use routing::Route;
use session::Caller;
use std::path::{Path, PathBuf};

pub(crate) const BODY_LIMIT: usize = 8 * 1024 * 1024;
pub(crate) const MAX_IN_FLIGHT: usize = 16;

#[derive(Clone, NsonSerialize)]
pub struct Attempt {
    pub agent: Option<Cow<'static, str>>,
    #[njson(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[njson(skip_serializing_if = "Option::is_none")]
    pub native_session: Option<String>,
    pub at: u64,
    pub model: String,
    pub provider: String,
    #[njson(skip_serializing_if = "Vec::is_empty")]
    pub via: Vec<String>,
    pub status: u16,
}
pub type History = Arc<Mutex<VecDeque<Attempt>>>;

#[derive(Clone, Copy)]
struct Target {
    provider: usize,
    model: usize,
    protocols: NativeProtocols,
}

pub struct Gateway {
    root: PathBuf,
    config: Config,
    routes: BTreeMap<String, Route>,
    group_ids: Vec<String>,
    manual_members: BTreeMap<String, Vec<String>>,
    anthropic_auth: Vec<std::sync::OnceLock<crate::config::AuthHeader>>,
    // IDs already listed in group order, or hidden aliases of public IDs.
    skip_in_sorted_catalog: std::collections::BTreeSet<String>,
    ordered_group_refs: Vec<String>,
    rounds: BTreeMap<String, Round>,
    models: Bytes,
    client: ForwardClient,
    vertex_tokens: BTreeMap<String, crate::vertex::auth::TokenCache>,
    token: String,
    slots: Arc<Semaphore>,
    history: History,
}

#[derive(NsonSerialize)]
struct Model<'a> {
    id: &'a str,
    object: &'static str,
    created: u64,
    owned_by: &'static str,
    #[njson(skip_serializing_if = "std::ops::Not::not")]
    free: bool,
    native_endpoints: Vec<&'static str>,
    #[njson(skip_serializing_if = "Option::is_none")]
    magpie_prefers: Option<&'static str>,
}

fn listed_model<'a>(id: &'a str, route: Route, config: &Config) -> Model<'a> {
    let protocols = route.protocols().expect("listed route");
    let free = if let Route::Model(t) = route {
        let provider = &config.providers[t.provider];
        provider.free_models.contains(&provider.models[t.model])
    } else {
        false
    };
    Model {
        id,
        free,
        object: "model",
        created: 0,
        owned_by: "crabforge",
        native_endpoints: protocols.endpoints(),
        magpie_prefers: protocols.preferred_endpoint(),
    }
}

fn gemini_model(id: &str) -> nextjson::Value {
    json!({"name":format!("models/{id}"),"displayName":id,"supportedGenerationMethods":["generateContent","streamGenerateContent"]})
}

impl Gateway {
    fn listed_ids(&self) -> impl Iterator<Item = &String> {
        self.ordered_group_refs.iter().chain(
            self.routes
                .keys()
                .filter(|id| !self.skip_in_sorted_catalog.contains(*id)),
        )
    }
    fn resolve(&self, id: &str) -> Option<(&String, &Route)> {
        self.routes
            .get_key_value(id)
            .or_else(|| {
                self.routes
                    .get_key_value(&self.config.canonical_auto_ref(id)?)
            })
            .or_else(|| self.routes.get_key_value(&self.config.auto_stand_in(id)?))
            .or_else(|| self.routes.get_key_value(&self.config.renamed_model(id)?))
    }
}

pub fn router(
    config: Config,
    token: String,
    history: History,
    root: &Path,
) -> Result<Arc<Gateway>> {
    let client = ForwardClient::new();
    router_with_client(config, token, history, root, client)
}

fn router_with_client(
    mut config: Config,
    token: String,
    history: History,
    root: &Path,
    client: ForwardClient,
) -> Result<Arc<Gateway>> {
    config.validate()?;
    for p in &mut config.providers {
        crate::opencode::Console::fill_missing(&mut p.endpoints);
    }
    anyhow::ensure!(!token.is_empty(), "empty gateway token");
    let mut routes = BTreeMap::new();
    for (i, p) in config.providers.iter().enumerate() {
        for (j, m) in p.models.iter().enumerate() {
            routes.insert(
                format!("{}/{m}", p.id),
                Route::Model(Target {
                    provider: i,
                    model: j,
                    protocols: NativeProtocols::model(p, m),
                }),
            );
        }
    }
    let mut resolved = config.resolved_groups();
    let mut hidden_aliases = config.unlisted_model_ids(&resolved);
    let ordered_group_refs = if config.group_display_order.is_empty() {
        vec![]
    } else {
        resolved.listed_refs(&config)
    };
    let manual_members = config
        .group_picks
        .keys()
        .filter(|name| !config.disabled_groups.contains(*name))
        .map(|name| {
            (
                name.clone(),
                resolved.display_members(&config, name).clone(),
            )
        })
        .collect();
    for (group, weights) in &mut config.group_weights {
        let original = std::mem::take(weights);
        for (id, target) in resolved
            .expanded
            .get(group)
            .unwrap_or(&config.groups[group])
            .iter()
            .zip(&resolved.members[group])
        {
            if let Some(weight) = original.get(id) {
                weights.insert(target.clone(), *weight);
            }
        }
    }
    // Keep the full member list for manual groups, whose selected member always wins.
    for (name, off) in &config.group_off {
        if !config.group_picks.contains_key(name) {
            let saved = resolved.expanded.get(name).unwrap_or(&config.groups[name]);
            let canonical = resolved
                .members
                .get_mut(name)
                .expect("validated route group");
            *canonical = saved
                .iter()
                .zip(std::mem::take(canonical))
                .filter_map(|(id, target)| (!off.contains(id)).then_some(target))
                .collect();
        }
    }
    config.groups = resolved.members;
    let mut group_ids = Vec::with_capacity(config.groups.len());
    for group in Config::group_order(&config.groups)? {
        let id = format!("group/{group}");
        if config.disabled_groups.contains(group) {
            hidden_aliases.insert(id.clone());
            routes.insert(id, Route::Disabled);
            continue;
        }
        let tools = config.groups[group]
            .iter()
            .any(|m| routes[m].tools(&config));
        let protocols = NativeProtocols::group(
            config.groups[group]
                .iter()
                .filter_map(|m| routes[m].protocols()),
        );
        routes.insert(
            id.clone(),
            Route::Group {
                tools,
                protocols,
                index: group_ids.len(),
            },
        );
        group_ids.push(id);
    }
    hidden_aliases.extend(resolved.public_refs.keys().cloned());
    for (alias, target) in resolved.aliases {
        let route = routes[&target];
        if matches!(route, Route::Disabled)
            && !alias.starts_with("group/")
            && !alias.starts_with("auto/")
        {
            continue;
        }
        if let std::collections::btree_map::Entry::Vacant(entry) = routes.entry(alias) {
            if matches!(route, Route::Disabled)
                || entry.key().starts_with("group/")
                || (entry.key().starts_with("auto/")
                    && resolved.public_refs.get(&target) != Some(entry.key()))
            {
                hidden_aliases.insert(entry.key().clone());
            }
            entry.insert(route);
        }
    }
    for p in &config.providers {
        for m in &p.models {
            let route = routes[&format!("{}/{m}", p.id)];
            routes.entry(m.clone()).or_insert(route);
        }
    }
    #[derive(NsonSerialize)]
    struct ModelList<'a> {
        object: &'static str,
        data: Vec<Model<'a>>,
    }
    hidden_aliases.extend(ordered_group_refs.iter().cloned());
    let mut rounds: BTreeMap<_, _> = config
        .rotating_groups
        .iter()
        .map(|name| (format!("group/{name}"), Round::Rotate(Default::default())))
        .collect();
    for (name, weights) in &config.group_weights {
        let weights = config.groups[name]
            .iter()
            .map(|id| weights.get(id).copied().unwrap_or(1))
            .collect();
        rounds.insert(format!("group/{name}"), Round::weighted(weights));
    }
    let mut state = Gateway {
        manual_members,
        anthropic_auth: (0..config.providers.len())
            .map(|_| Default::default())
            .collect(),
        vertex_tokens: config
            .providers
            .iter()
            .filter(|p| p.endpoints.vertex.is_some() && p.key_env.is_empty())
            .map(|p| (p.id.clone(), Default::default()))
            .collect(),
        rounds,
        group_ids,
        skip_in_sorted_catalog: hidden_aliases,
        ordered_group_refs,
        root: root.to_owned(),
        config,
        routes,
        models: Bytes::new(),
        client,
        token,
        slots: Arc::new(Semaphore::new(MAX_IN_FLIGHT)),
        history,
    };
    state.models = Bytes::from(
        nextjson::to_vec(&ModelList {
            object: "list",
            data: state
                .listed_ids()
                .map(|id| listed_model(id, state.routes[id], &state.config))
                .collect(),
        })
        .map_err(anyhow::Error::msg)?,
    );
    Ok(Arc::new(state))
}

#[derive(Clone, Copy)]
enum Api<'a> {
    Chat(&'a str),
    Responses(&'a str),
    Messages(&'a str),
    Gemini(&'a str),
}

#[derive(Clone)]
enum Destination<'a> {
    Upstream(Url, Api<'a>, Option<(Format, Format)>),
    Browser(Site),
}

impl<'a> Api<'a> {
    fn format(self) -> Option<Format> {
        match self {
            Self::Chat("chat/completions") => Some(Format::Chat),
            Self::Messages("messages") => Some(Format::Messages),
            Self::Responses("responses") => Some(Format::Responses),
            Self::Gemini("generateContent" | "streamGenerateContent") => Some(Format::Gemini),
            _ => None,
        }
    }

    fn destination(self, p: &Provider, model: &str) -> Option<Destination<'a>> {
        if let Some(site) = p.web_site {
            return matches!(self, Self::Chat("chat/completions"))
                .then_some(Destination::Browser(site));
        }
        // A relay can expose every HTTP API while a particular upstream model
        // accepts only one. Translate once at this gateway when the directory
        // identifies a reachable native endpoint. Other actions, including
        // token counting and embeddings, have their own forwarding rules.
        if let Some(from) = self.format()
            && let Some(hints) = p.model_protocols.get(model)
            && let Some(endpoints) = &hints.native_endpoints
        {
            let mut translated = None;
            for endpoint in hints.magpie_prefers.iter().chain(endpoints) {
                use crate::catalog::RelayEndpoint;
                if !endpoints.contains(endpoint) {
                    continue;
                }
                let (api, to) = match endpoint {
                    RelayEndpoint::Chat => (Api::Chat("chat/completions"), Format::Chat),
                    RelayEndpoint::Messages => (Api::Messages("messages"), Format::Messages),
                    RelayEndpoint::Responses => (Api::Responses("responses"), Format::Responses),
                };
                if let Some(url) = api.url(p, model) {
                    if from == to {
                        return Some(Destination::Upstream(url, self, None));
                    }
                    translated.get_or_insert(Destination::Upstream(url, api, Some((from, to))));
                }
            }
            if translated.is_some() {
                return translated;
            }
        }
        if let Some(url) = self.url(p, model) {
            return Some(Destination::Upstream(url, self, None));
        }
        let from = self.format()?;
        [
            Api::Chat("chat/completions"),
            Api::Messages("messages"),
            Api::Responses("responses"),
            Api::Gemini("generateContent"),
        ]
        .into_iter()
        .find_map(|api| {
            Some(Destination::Upstream(
                api.url(p, model)?,
                api,
                Some((from, api.format()?)),
            ))
        })
    }
    fn url(self, p: &Provider, model: &str) -> Option<Url> {
        self.provider_url(p, model, true)
    }

    fn provider_url(self, p: &Provider, model: &str, prefer_native: bool) -> Option<Url> {
        if let Some(v) = &p.endpoints.vertex {
            let mut url = v.base().ok()?;
            let partner = model.contains('/');
            let claude = !partner && model.to_ascii_lowercase().starts_with("claude-");
            let mut seg = url.path_segments_mut().ok()?;
            match self {
                Self::Gemini(action) if !partner && !claude => {
                    seg.extend([
                        "publishers",
                        "google",
                        "models",
                        &format!("{model}:{action}"),
                    ]);
                }
                Self::Chat("chat/completions") if partner && !v.project.is_empty() => {
                    seg.extend(["endpoints", "openapi", "chat", "completions"]);
                }
                Self::Messages(path) if claude && !v.project.is_empty() => {
                    let name = if path == "messages/count_tokens" {
                        "count-tokens"
                    } else {
                        model
                    };
                    seg.extend([
                        "publishers",
                        "anthropic",
                        "models",
                        &format!("{name}:rawPredict"),
                    ]);
                }
                _ => return None,
            }
            drop(seg);
            if matches!(self, Self::Gemini("streamGenerateContent")) {
                url.set_query(Some("alt=sse"));
            }
            return Some(url);
        }
        let (base, path) = match self {
            Self::Chat(path) => (&p.endpoints.chat, path),
            Self::Responses(path) => (&p.endpoints.responses, path),
            Self::Messages(path) => (&p.endpoints.anthropic, path),
            Self::Gemini(action) => (&p.endpoints.gemini, action),
        };
        let protocol = match self {
            Self::Messages(_) => "anthropic",
            Self::Gemini(_) => "gemini",
            _ => "openai",
        };
        let mut url = crate::config::api_base(base, protocol).ok()?;
        if prefer_native && let Some(console) = crate::opencode::Console::at(&url) {
            let native = console.format(model);
            let family = match self {
                Self::Messages(_) => Some(Format::Messages),
                Self::Gemini(_) => Some(Format::Gemini),
                Self::Responses(_) => Some(Format::Responses),
                _ => self.format(),
            };
            if family == Some(Format::Gemini) && native.is_none()
                || family.zip(native).is_some_and(|(f, n)| f != n)
            {
                return None;
            }
        }
        if let Some((_, service)) = crate::bedrock::scope(&url) {
            let model = model.to_ascii_lowercase();
            let closed_gpt = model.contains("openai.gpt-") && !model.contains("gpt-oss");
            let claude = model.contains("anthropic.claude");
            if matches!(self, Self::Messages(_)) != claude
                || (matches!(self, Self::Responses(_)) && !closed_gpt)
            {
                return None;
            }
            if service == "bedrock-mantle"
                && matches!(url.path().trim_end_matches('/'), "/openai/v1" | "/v1")
                && matches!(self, Self::Chat(_) | Self::Responses(_))
            {
                url.set_path(if closed_gpt || model.starts_with("xai.") {
                    "/openai/v1"
                } else {
                    "/v1"
                });
            }
        }
        let needs_v1 =
            matches!(self, Self::Messages(_)) && !url.path().trim_end_matches('/').ends_with("/v1");
        {
            let mut seg = url.path_segments_mut().ok()?;
            seg.pop_if_empty();
            if needs_v1 {
                seg.push("v1");
            }
            if matches!(self, Self::Gemini(_)) {
                seg.push("models");
                let model = model.strip_prefix("models/").unwrap_or(model);
                let last = format!("{model}:{path}");
                seg.extend(last.split('/'));
            } else {
                seg.extend(path.split('/'));
            }
        }
        if matches!(self, Self::Gemini("streamGenerateContent")) {
            url.query_pairs_mut().append_pair("alt", "sse");
        }
        Some(url)
    }
}

fn failure(code: StatusCode, message: &str) -> Response {
    json_response(
        code,
        &json!({"error":{"message":message,"type":"gateway_error"}}),
    )
}

pub(crate) fn json_response(status: StatusCode, value: &impl NsonSerialize) -> Response {
    match nextjson::to_vec(value) {
        Ok(bytes) => (status, [(header::CONTENT_TYPE, "application/json")], bytes).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

fn authorized(headers: &HeaderMap, token: &str) -> bool {
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split_once(' '))
        .filter(|(s, _)| s.eq_ignore_ascii_case("bearer"))
        .map(|(_, v)| v.as_bytes());
    [
        bearer,
        headers.get("x-api-key").map(|v| v.as_bytes()),
        headers.get("x-goog-api-key").map(|v| v.as_bytes()),
    ]
    .into_iter()
    .flatten()
    .any(|v| v.ct_eq(token.as_bytes()).into())
}

fn selection(bytes: &[u8]) -> nextjson::Result<(String, Range<usize>)> {
    let mut decoder = nextjson::Decoder::new(bytes);
    decoder.begin_object()?;
    let mut model = None;
    while let Some(key) = decoder.object_key()? {
        if key == "model" {
            if model.is_some() {
                return Err(nextjson::Error::custom("duplicate model field"));
            }
            let start = decoder.save().pos();
            let id = decoder.string()?.into_owned();
            let end = decoder.save().pos();
            let literal = bytes[start..end].trim_ascii_start();
            model = Some((id, end - literal.len()..end));
        } else {
            decoder.skip_value()?;
        }
        if !decoder.object_entry_sep()? {
            break;
        }
    }
    decoder.end_object()?;
    decoder.end()?;
    model.ok_or_else(|| nextjson::Error::custom("missing model field"))
}

pub(crate) async fn dispatch(
    gw: &Gateway,
    req: Request<impl Future<Output = Result<Bytes>>>,
) -> Response {
    let (parts, body) = req.into_parts();
    let mut origins = parts.headers.get_all(header::ORIGIN).iter();
    let origin = origins.next();
    let allowed = origin.filter(|v| {
        v.to_str()
            .is_ok_and(|s| s == "app://obsidian.md" || gw.config.cors_origins.contains(s))
            && origins.next().is_none()
    });
    let preflight = allowed.is_some()
        && parts.method == Method::OPTIONS
        && parts
            .headers
            .contains_key(header::ACCESS_CONTROL_REQUEST_METHOD);
    let path = parts.uri.path().trim_start_matches('/');
    let (path, agent) = crate::agents::by_path(path);
    let ollama = ollama::Endpoint::parse(path);
    let mut response = if preflight {
        cors::preflight(&parts.headers)
    } else if (origin.is_some() && allowed.is_none()) || !authorized(&parts.headers, &gw.token) {
        if ollama.is_some() {
            ollama::error(StatusCode::UNAUTHORIZED, "gateway key required")
        } else {
            failure(StatusCode::UNAUTHORIZED, "gateway key required")
        }
    } else {
        let caller = Caller::new(agent, &parts.headers);
        let mut response = if let Some(endpoint) = ollama {
            endpoint.serve(gw, &caller, &parts, body).await
        } else {
            serve_http(gw, &caller, path, &parts, body).await
        };
        caller.set_response_headers(response.headers_mut());
        response
    };
    let headers = response.headers_mut();
    headers.append(header::VARY, http::HeaderValue::from_static("Origin"));
    if let Some(origin) = allowed {
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());
        headers.insert(
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            http::HeaderValue::from_static("X-Crabforge-Session, X-Magpie-Session"),
        );
    }
    response
}

async fn serve_http(
    gw: &Gateway,
    caller: &Caller<'_>,
    path: &str,
    parts: &http::request::Parts,
    body: impl Future<Output = Result<Bytes>>,
) -> Response {
    let path = path.strip_prefix("v1/").unwrap_or(path);
    if parts.method == Method::GET {
        if path == "models" {
            return (
                [(header::CONTENT_TYPE, "application/json")],
                gw.models.clone(),
            )
                .into_response();
        }
        if path == "v1beta/models" {
            return json_response(
                StatusCode::OK,
                &json!({"models":gw.listed_ids().map(|id|gemini_model(id)).collect::<Vec<_>>()}),
            );
        }
        if let Some((is_gemini, id)) = path
            .strip_prefix("models/")
            .map(|id| (false, id))
            .or_else(|| path.strip_prefix("v1beta/models/").map(|id| (true, id)))
        {
            let Ok(decoded) = percent_encoding::percent_decode_str(id).decode_utf8() else {
                return failure(StatusCode::BAD_REQUEST, "invalid model ID encoding");
            };
            let id = decoded.as_ref();
            let route = match gw.resolve(id) {
                Some((_, Route::Disabled)) => {
                    return failure(StatusCode::NOT_FOUND, "route group is switched off");
                }
                None => return failure(StatusCode::NOT_FOUND, "unknown model"),
                Some((_, route)) => route,
            };
            return if is_gemini {
                json_response(StatusCode::OK, &gemini_model(id))
            } else {
                json_response(StatusCode::OK, &listed_model(id, *route, &gw.config))
            };
        }
    }
    let mut path_model = None;
    let api = match path {
        "chat/completions"
        | "embeddings"
        | "rerank"
        | "images/generations"
        | "images/edits"
        | "audio/speech"
        | "audio/transcriptions"
        | "audio/translations" => Api::Chat(path),
        "responses" | "responses/compact" => Api::Responses(path),
        "messages" | "messages/count_tokens" => Api::Messages(path),
        _ => match path
            .strip_prefix("v1beta/models/")
            .and_then(|p| p.rsplit_once(':'))
        {
            Some((m, action @ ("generateContent" | "streamGenerateContent" | "countTokens"))) => {
                let Ok(decoded) = percent_encoding::percent_decode_str(m).decode_utf8() else {
                    return failure(StatusCode::BAD_REQUEST, "invalid model ID encoding");
                };
                path_model = Some(decoded);
                Api::Gemini(action)
            }
            _ => return failure(StatusCode::NOT_FOUND, "unknown gateway endpoint"),
        },
    };
    if parts.method != Method::POST {
        return failure(StatusCode::METHOD_NOT_ALLOWED, "POST required");
    }
    let (bytes, permit) = match gw.read_body(body).await {
        Ok(input) => input,
        Err((status, msg)) => return failure(status, msg),
    };
    let input = Inbound {
        api,
        path_model: path_model.as_deref(),
        headers: &parts.headers,
        bytes,
        ollama: None,
    };
    forward(gw, caller, input, permit).await
}

struct Inbound<'a> {
    api: Api<'a>,
    path_model: Option<&'a str>,
    headers: &'a HeaderMap,
    bytes: Bytes,
    ollama: Option<ollama::Output>,
}

async fn forward(
    gw: &Gateway,
    caller: &Caller<'_>,
    input: Inbound<'_>,
    permit: Arc<OwnedSemaphorePermit>,
) -> Response {
    let Inbound {
        api,
        path_model,
        headers,
        bytes,
        ollama,
    } = input;
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let upload = if ollama.is_none()
        && content_type
            .split(';')
            .next()
            .is_some_and(|s| s.trim().eq_ignore_ascii_case("multipart/form-data"))
    {
        if !matches!(
            api,
            Api::Chat("audio/transcriptions" | "audio/translations" | "images/edits")
        ) {
            return failure(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "this endpoint expects JSON",
            );
        }
        match crate::media::Upload::parse(content_type, bytes.clone()).await {
            Ok(upload) => Some(upload),
            Err(e) => return failure(StatusCode::BAD_REQUEST, &e.to_string()),
        }
    } else {
        None
    };
    let (asked, range) = match (path_model, &upload) {
        (_, Some(form)) => (form.model.clone(), None),
        (Some(m), _) => (m.to_owned(), None),
        _ => match selection(&bytes) {
            Ok((m, r)) => (m, Some(r)),
            Err(_) => {
                return failure(
                    StatusCode::BAD_REQUEST,
                    "expected a JSON object with one string model field",
                );
            }
        },
    };
    let Some((_, route)) = gw.resolve(&asked) else {
        return failure(StatusCode::NOT_FOUND, "unknown model");
    };
    if matches!(route, Route::Disabled) {
        return failure(StatusCode::NOT_FOUND, "route group is switched off");
    }
    let picks = if matches!(route, Route::Group { .. }) || !route.fallbacks(&gw.config).is_empty() {
        match gw.current_picks().await {
            Ok(picks) => picks,
            Err(e) => return failure(StatusCode::SERVICE_UNAVAILABLE, &e.to_string()),
        }
    } else {
        BTreeMap::new()
    };
    let mut candidates = patience::Attempts::new(gw.candidates(*route, api, &picks));
    if !candidates.has_more() {
        return failure(
            StatusCode::BAD_REQUEST,
            "model has no endpoint for this protocol",
        );
    }
    let mut requested_stream = None;
    while let Some(candidate) = candidates.next() {
        let p = &gw.config.providers[candidate.target.provider];
        let model = &p.models[candidate.target.model];
        let record = |status| gw.record(caller, &asked, &p.id, &candidate.via, status);
        let retry_candidate = (api.format().is_some()
            && !candidate.patience.is_zero()
            && matches!(candidate.destination, Destination::Upstream(..)))
        .then(|| candidate.clone());
        let destination = candidate.destination;
        let (mut url, mut native, mut translation) = match destination {
            Destination::Upstream(url, native, translation) => (url, native, translation),
            Destination::Browser(site) => {
                let prompt = match WebPrompt::parse(&bytes) {
                    Ok(prompt) => prompt,
                    Err(e) => return failure(StatusCode::BAD_REQUEST, &e.to_string()),
                };
                let response = match prompt.answer(&gw.root, site).await {
                    Ok(resp) => match ollama {
                        Some(out) => out
                            .convert(resp)
                            .await
                            .unwrap_or_else(|e| failure(StatusCode::BAD_GATEWAY, &e.to_string())),
                        None => resp,
                    },
                    Err(e) => failure(StatusCode::BAD_GATEWAY, &format!("web chat: {e:#}")),
                };
                record(response.status().as_u16());
                // The browser may already have submitted the turn when its page failed.
                return response;
            }
        };
        let converted = if let Some((from, to)) = translation {
            match translate::convert_request(
                from,
                to,
                &bytes,
                model,
                matches!(api, Api::Gemini("streamGenerateContent")),
            ) {
                Ok(body) => {
                    if body.stream && to == Format::Gemini {
                        url = Api::Gemini("streamGenerateContent")
                            .url(p, model)
                            .expect("validated Gemini endpoint");
                    }
                    Some(body)
                }
                Err(e) => {
                    // Go can translate on its side when the incoming protocol has
                    // features our text/tool converter cannot represent.
                    if crate::opencode::Console::request(&url) == Some(crate::opencode::Console::Go)
                        && let Some(direct) = api.provider_url(p, model, false)
                        && crate::opencode::Console::request(&direct)
                            == Some(crate::opencode::Console::Go)
                    {
                        url = direct;
                        native = api;
                        translation = None;
                        None
                    } else if candidates.advance().await {
                        continue;
                    } else {
                        return failure(StatusCode::BAD_REQUEST, &e.to_string());
                    }
                }
            }
        } else {
            None
        };
        let vertex_body = if p.endpoints.vertex.is_some()
            && let Api::Messages(path) = native
        {
            let data = converted
                .as_ref()
                .map_or(bytes.as_ref(), |body| body.body.as_slice());
            match crate::vertex::claude_body(&mut url, model, path == "messages/count_tokens", data)
            {
                Ok(body) => Some(body),
                Err(e) => return failure(StatusCode::BAD_REQUEST, &e.to_string()),
            }
        } else {
            None
        };
        let mut request = Request::post(url.as_str());
        if crate::opencode::Console::request(&url).is_some() && native.format().is_some() {
            let seed = headers
                .get("x-opencode-session")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .or(caller.session);
            request = crate::opencode::conversation(request, seed, &bytes, &gw.token);
        }
        if upload.is_none() {
            request = request.header(header::CONTENT_TYPE, "application/json");
        }
        for h in [
            "accept",
            "anthropic-version",
            "anthropic-beta",
            "openai-beta",
        ] {
            if translation.is_none()
                && ollama.is_none()
                && let Some(value) = headers.get(h)
            {
                request = request.header(h, value);
            }
        }
        if matches!(native, Api::Messages(_))
            && (translation.is_some() || !headers.contains_key("anthropic-version"))
        {
            request = request.header("anthropic-version", "2023-06-01");
        }
        if !p.key_env.is_empty() {
            let Some(key) = std::env::var(&p.key_env).ok().filter(|s| !s.is_empty()) else {
                record(503);
                if candidates.advance().await {
                    continue;
                }
                return failure(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "provider key environment variable is missing or empty",
                );
            };
            let protocol = match native {
                Api::Gemini(_) => "gemini",
                Api::Messages(_) => "anthropic",
                _ => "openai",
            };
            request = if p.endpoints.vertex.is_some() {
                request.header("x-goog-api-key", key)
            } else {
                crate::config::authenticate(request, &url, protocol, &key, p.endpoints.auth_header)
            };
        } else if let Some(v) = &p.endpoints.vertex {
            match gw.vertex_tokens[&p.id].token(v).await {
                Ok(token) => {
                    request = request.header(header::AUTHORIZATION, format!("Bearer {token}"))
                }
                Err(e) => {
                    record(503);
                    if candidates.advance().await {
                        continue;
                    }
                    return failure(StatusCode::SERVICE_UNAVAILABLE, &e.to_string());
                }
            }
        }
        let stream = converted.as_ref().is_some_and(|body| body.stream);
        let parts = if let Some(body) = vertex_body {
            vec![body.into()]
        } else if let Some(upload) = &upload {
            request = request.header(header::CONTENT_TYPE, &upload.content_type);
            match upload.body(model) {
                Ok(parts) => parts,
                Err(e) => return failure(StatusCode::BAD_REQUEST, &e.to_string()),
            }
        } else if let Some(converted) = converted {
            vec![converted.body.into()]
        } else {
            match &range {
                Some(r) if &asked != model => {
                    let wire = Bytes::from(nextjson::to_vec(model).expect("string serialization"));
                    vec![bytes.slice(..r.start), wire, bytes.slice(r.end..)]
                }
                _ => vec![bytes.clone()],
            }
        };
        let length: usize = parts.iter().map(Bytes::len).sum();
        let first_content = candidate.first_content.zip(native.format()).filter(|_| {
            *requested_stream.get_or_insert_with(|| first_content::requested(api, &bytes))
        });
        let send = async {
            let mut request = request.header(header::CONTENT_LENGTH, length).body(parts)?;
            let upstream = if matches!(native, Api::Messages(_))
                && !p.key_env.is_empty()
                && p.endpoints.vertex.is_none()
            {
                let learned = &gw.anthropic_auth[candidate.target.provider];
                let unknown = learned.get().is_none();
                let result = crate::anthropic_auth::send(&gw.client, request, learned).await;
                if unknown && let Some(&mode) = learned.get() {
                    let _ = crate::config::remember_auth(&gw.root, p, mode);
                }
                result
            } else {
                crate::bedrock::authenticate(&mut request, &url, &p.key_env)?;
                gw.client.send(request).await
            }?;
            match first_content {
                Some((_, format)) => first_content::wait(upstream, format).await,
                None => Ok(upstream),
            }
        };
        let result = if let Some((wait, _)) = first_content {
            match tokio::time::timeout(wait, send).await {
                Ok(result) => result,
                Err(_) => {
                    record(StatusCode::GATEWAY_TIMEOUT.as_u16());
                    continue;
                }
            }
        } else {
            send.await
        };
        let upstream = match result {
            Ok(resp) => resp,
            Err(_) => {
                record(502);
                if candidates.advance().await {
                    continue;
                }
                return failure(StatusCode::BAD_GATEWAY, "upstream transport failed");
            }
        };
        let status = upstream.status();
        let mut upstream = upstream;
        if status == StatusCode::TOO_MANY_REQUESTS
            || status.is_server_error()
            || status == StatusCode::REQUEST_TIMEOUT && retry_candidate.is_some()
        {
            if patience::retryable(status) {
                upstream = candidates.observe(retry_candidate, upstream).await;
            }
            if candidates.has_more() {
                record(status.as_u16());
                continue;
            }
            if let Some(retry_at) = candidates.replan() {
                record(status.as_u16());
                drop(upstream);
                tokio::time::sleep_until(retry_at).await;
                continue;
            }
        }
        let response = if let Some((to, from)) = translation
            && status.is_success()
        {
            translate::convert_reply(upstream, from, to, &asked, stream, permit.clone()).await
        } else {
            Ok(upstream_response(upstream, permit.clone()))
        };
        let response = match (response, &ollama) {
            (Ok(resp), Some(out)) if status.is_success() => out.clone().convert(resp).await,
            (result, _) => result,
        };
        let response = match response {
            Ok(r) => r,
            Err(_) if candidates.advance().await => {
                record(502);
                continue;
            }
            Err(e) => failure(StatusCode::BAD_GATEWAY, &e.to_string()),
        };
        record(response.status().as_u16());
        return response;
    }
    failure(StatusCode::BAD_GATEWAY, "no upstream available")
}

fn upstream_response(upstream: Response, permit: Arc<OwnedSemaphorePermit>) -> Response {
    let mut response = Response::builder().status(upstream.status());
    for h in [
        "content-type",
        "content-encoding",
        "retry-after",
        "x-request-id",
        "request-id",
    ] {
        if let Some(v) = upstream.headers().get(h) {
            response = response.header(h, v);
        }
    }
    let body = upstream.into_body().into_data_stream().map(move |chunk| {
        let _keep = &permit;
        chunk
    });
    response
        .body(Body::from_stream(body))
        .expect("validated upstream headers")
}

impl Gateway {
    async fn read_body(
        &self,
        body: impl Future<Output = Result<Bytes>>,
    ) -> std::result::Result<(Bytes, Arc<OwnedSemaphorePermit>), (StatusCode, &'static str)> {
        let permit = self
            .slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| (StatusCode::TOO_MANY_REQUESTS, "gateway is busy"))?;
        let bytes = tokio::time::timeout(Duration::from_secs(30), body)
            .await
            .map_err(|_| (StatusCode::REQUEST_TIMEOUT, "request body timed out"))?
            .map_err(|_| {
                (
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "request exceeds 8 MiB or could not be read",
                )
            })?;
        Ok((bytes, Arc::new(permit)))
    }

    fn record(&self, caller: &Caller<'_>, model: &str, provider: &str, via: &[&str], status: u16) {
        let at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut log = self.history.lock().unwrap_or_else(|p| p.into_inner());
        if log.len() == 128 {
            log.pop_front();
        }
        log.push_back(Attempt {
            agent: caller.agent.clone(),
            session: caller.session.map(str::to_owned),
            native_session: caller.native_session.map(str::to_owned),
            at,
            model: model.into(),
            provider: provider.into(),
            via: via.iter().map(|id| (*id).to_owned()).collect(),
            status,
        });
    }
}

pub fn status_text(status: u16) -> &'static str {
    courierust::courierust_http::StatusCode::from_u16(status)
        .canonical_reason()
        .unwrap_or("Unknown status")
}

#[cfg(test)]
mod console_test;

#[cfg(test)]
mod tests {
    use super::*;
    use nextjson::json;

    #[test]
    fn selection_locates_only_the_top_level_model_literal() {
        let bytes = br#"{ "extra": {"model": "nested"}, "mo\u0064el":  "x\u002fy" , "tail": [1] }"#;
        let (name, range) = selection(bytes).unwrap();
        assert_eq!(name, "x/y");
        assert_eq!(&bytes[range], br#""x\u002fy""#);
        for invalid in [
            r#"{"model":"x","model":"y"}"#,
            r#"{"model":1}"#,
            r#"{"model":"x",}"#,
            r#"{"model":"x"} []"#,
            r#"{"fields":{"model":"x"}}"#,
        ] {
            assert!(selection(invalid.as_bytes()).is_err(), "{invalid}");
        }
    }

    #[test]
    fn relay_routes_use_the_incoming_protocol_or_the_advertised_native_api() -> Result<()> {
        let p: Provider = nextjson::from_value(json!({"id":"relay","models":[],
        "endpoints":{"chat":"http://localhost/v1","responses":"http://localhost/v1","anthropic":"http://localhost"},
        "model_protocols":{
            "mixed":{"native_endpoints":[]},
            "picked":{"native_endpoints":["/v1/messages","/v1/responses"],"magpie_prefers":"/v1/messages"},
            "text":{"native_endpoints":["/v1/chat/completions"]}
        }})).map_err(anyhow::Error::msg)?;
        for (model, api, expected, conversion) in [
            (
                "legacy",
                Api::Chat("chat/completions"),
                "/v1/chat/completions",
                None,
            ),
            ("mixed", Api::Responses("responses"), "/v1/responses", None),
            ("picked", Api::Responses("responses"), "/v1/responses", None),
            (
                "picked",
                Api::Chat("chat/completions"),
                "/v1/messages",
                Some((Format::Chat, Format::Messages)),
            ),
            (
                "text",
                Api::Messages("messages"),
                "/v1/chat/completions",
                Some((Format::Messages, Format::Chat)),
            ),
            ("picked", Api::Chat("embeddings"), "/v1/embeddings", None),
        ] {
            let Some(Destination::Upstream(url, _, converted)) = api.destination(&p, model) else {
                panic!("missing relay route for {model}")
            };
            assert_eq!(url.path(), expected);
            assert_eq!(converted, conversion);
        }
        Ok(())
    }

    #[test]
    fn console_routes_by_plan_and_preserves_custom_endpoints() -> Result<()> {
        for (base, mini, gemini) in [
            (
                "https://opencode.ai/zen/v1",
                "/zen/v1/chat/completions",
                "gemini-3-flash",
            ),
            (
                "https://opencode.ai/inference/go/openai/v1",
                "/inference/go/anthropic/v1/messages",
                "gemini-4-argon",
            ),
        ] {
            let mut p: Provider = nextjson::from_value(
                json!({"id":"my-console","models":[],"endpoints":{"chat":base}}),
            )
            .map_err(anyhow::Error::msg)?;
            crate::opencode::Console::fill_missing(&mut p.endpoints);
            let api = Api::Chat("chat/completions");
            let endpoint = |model| -> (Url, Option<(Format, Format)>) {
                let Some(Destination::Upstream(url, _, translation)) = api.destination(&p, model)
                else {
                    panic!("missing endpoint for {model}")
                };
                (url, translation)
            };
            let (url, translation) = endpoint("minimax-m2.7");
            assert_eq!(url.path(), mini);
            assert_eq!(translation.is_some(), base.contains("/go/"));
            let (url, translation) = endpoint("gpt-6-luna");
            assert!(url.path().ends_with("/openai/v1/responses"));
            assert_eq!(translation, Some((Format::Chat, Format::Responses)));
            let (url, translation) = endpoint(gemini);
            assert!(
                url.path()
                    .ends_with(&format!("/google/v1beta/models/{gemini}:generateContent"))
            );
            assert_eq!(translation, Some((Format::Chat, Format::Gemini)));
            assert!(Api::Gemini("countTokens").url(&p, "glm-5.3").is_none());
            assert!(
                Api::Gemini("generateContent")
                    .url(&p, "new-model")
                    .is_none()
            );
            assert_eq!(endpoint("new-model").1, None);
            p.endpoints.anthropic = "https://relay.example/own".into();
            crate::opencode::Console::fill_missing(&mut p.endpoints);
            assert_eq!(
                Api::Messages("messages")
                    .url(&p, "minimax-m2.7")
                    .unwrap()
                    .as_str(),
                "https://relay.example/own/v1/messages"
            );
        }
        Ok(())
    }

    #[test]
    fn bedrock_chooses_the_models_native_api_before_translation() -> Result<()> {
        let preset = nextjson::from_str::<Vec<nextjson::Value>>(crate::config::PRESETS)
            .map_err(anyhow::Error::msg)?
            .into_iter()
            .find(|p| p.get("id").and_then(nextjson::Value::as_str) == Some("bedrock"))
            .unwrap();
        let p = Provider {
            fallback: vec![],
            no_stand_in: false,
            model_protocols: Default::default(),
            free_models: Default::default(),
            id: "aws".into(),
            catalog: String::new(),
            web_site: None,
            endpoints: nextjson::from_value(preset["endpoints"].clone())
                .map_err(anyhow::Error::msg)?,
            key_env: String::new(),
            models: vec![],
        };
        for (api, model, expected_path, translated) in [
            (
                Api::Chat("chat/completions"),
                "global.anthropic.claude-sonnet-5",
                "/anthropic/v1/messages",
                true,
            ),
            (
                Api::Messages("messages"),
                "openai.gpt-oss-120b-1:0",
                "/openai/v1/chat/completions",
                true,
            ),
            (
                Api::Responses("responses"),
                "global.openai.gpt-6-sol",
                "/openai/v1/responses",
                false,
            ),
            (
                Api::Responses("responses"),
                "deepseek.v3.2",
                "/openai/v1/chat/completions",
                true,
            ),
        ] {
            let Some(Destination::Upstream(url, _, conversion)) = api.destination(&p, model) else {
                panic!("missing endpoint for {model}");
            };
            assert_eq!(url.path(), expected_path);
            assert_eq!(conversion.is_some(), translated);
        }
        Ok(())
    }
}
