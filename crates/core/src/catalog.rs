use crate::config::{AuthHeader, Endpoints, api_base, api_key_headers};
use anyhow::{Context, Result, bail, ensure};
use courierust::{
    courierust_client::Client,
    courierust_http::{HeaderMap, HeaderName, HeaderValue, Method},
};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    time::Duration,
};
use url::Url;
mod builtin;
pub use builtin::fixed_models;

/// Include the local lists in the catalog shown by both the CLI and Flutter.
pub fn presets() -> Result<String> {
    use nextjson::{Value, json};
    let mut entries: Vec<Value> =
        nextjson::from_str(crate::config::PRESETS).map_err(anyhow::Error::msg)?;
    for entry in &mut entries {
        let entry = entry
            .as_object_mut()
            .context("expected a provider preset object")?;
        let bases = nextjson::from_value(
            entry
                .get("endpoints")
                .context("missing preset endpoints")?
                .clone(),
        )
        .map_err(anyhow::Error::msg)?;
        if entry.get("id").and_then(Value::as_str) == Some("fireworks-nexus") {
            entry.insert("models".into(), json!(["auto"]));
        } else if let Some(ids) = fixed_models(&bases) {
            entry.insert("models".into(), json!(ids));
        }
    }
    nextjson::to_string_pretty(&entries).map_err(anyhow::Error::msg)
}

#[derive(nextjson::NsonDeserialize)]
struct List {
    #[njson(default)]
    data: Vec<Entry>,
    #[njson(default)]
    models: Vec<Entry>,
    #[njson(rename = "nextPageToken")]
    next: Option<String>,
    has_more: Option<bool>,
    last_id: Option<String>,
}
#[derive(nextjson::NsonDeserialize)]
struct Entry {
    id: Option<String>,
    name: Option<String>,
    free: Option<nextjson::Value>,
    native_endpoints: Option<nextjson::Value>,
    magpie_prefers: Option<nextjson::Value>,
    reasoning_levels: Option<nextjson::Value>,
    #[njson(rename = "supportedGenerationMethods")]
    methods: Option<Vec<String>>,
}

string_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum RelayEndpoint {
        Chat => "/v1/chat/completions",
        Responses => "/v1/responses",
        Messages => "/v1/messages",
        Decisions => "/v1/decisions",
        SystemOne => "/v1/systemone",
    }
}

#[derive(Clone, Debug, Default, nextjson::NsonSerialize, nextjson::NsonDeserialize)]
#[njson(default, deny_unknown_fields)]
pub struct RelayProtocols {
    #[njson(skip_serializing_if = "Option::is_none")]
    pub native_endpoints: Option<BTreeSet<RelayEndpoint>>,
    #[njson(skip_serializing_if = "Option::is_none")]
    pub magpie_prefers: Option<RelayEndpoint>,
}

impl Entry {
    fn relay_protocols(&self) -> Option<RelayProtocols> {
        let known = |value: &nextjson::Value| RelayEndpoint::from_wire(value.as_str()?);
        let magpie_prefers = self.magpie_prefers.as_ref().and_then(known);
        let native_endpoints = self
            .native_endpoints
            .as_ref()
            .and_then(nextjson::Value::as_array)
            .map(|values| values.iter().filter_map(known).collect());
        (magpie_prefers.is_some() || native_endpoints.is_some()).then_some(RelayProtocols {
            native_endpoints,
            magpie_prefers,
        })
    }
}

struct Source {
    url: Url,
    api: &'static str,
    deployments: bool,
}

fn sources(bases: &Endpoints) -> Result<Vec<Source>> {
    let protocols = [
        (&bases.chat, "chat"),
        (&bases.responses, "chat"),
        (&bases.anthropic, "anthropic"),
        (&bases.gemini, "gemini"),
        (&bases.decisions, "decisions"),
        (&bases.systemone, "systemone"),
    ];
    if !bases.models_url.is_empty() {
        let api = if bases.vertex.is_some() {
            "gemini"
        } else {
            protocols
                .iter()
                .find(|(base, _)| !base.is_empty())
                .context("configure an API endpoint first")?
                .1
        };
        return Ok(vec![Source {
            url: crate::config::catalog_url(&bases.models_url)?,
            api,
            deployments: false,
        }]);
    }
    let ai_studio_pair = bases.chat.trim_end_matches('/')
        == "https://generativelanguage.googleapis.com/v1beta/openai"
        && bases.gemini.trim_end_matches('/') == "https://generativelanguage.googleapis.com/v1beta";
    let mut sources = Vec::new();
    for (base, api) in protocols {
        if base.is_empty()
            || (ai_studio_pair
                && base.trim_end_matches('/')
                    == "https://generativelanguage.googleapis.com/v1beta/openai")
        {
            continue;
        }
        let address = api_base(base, api)?;
        let (url, api) = match crate::opencode::Console::at(&address) {
            Some(console) => (console.models_url(&address), "chat"),
            None => (list_url(address, api)?, api),
        };
        if api == "anthropic"
            && url.host_str() == Some("openrouter.ai")
            && (!bases.chat.is_empty() || !bases.responses.is_empty())
        {
            // Its Messages directory repeats the Chat models with unusable prefixes.
            continue;
        }
        if sources
            .iter()
            .any(|s: &Source| s.url == url && s.api == api)
        {
            continue;
        }
        let deployments = crate::config::azure(&url) && api == "chat";
        sources.push(Source {
            url,
            api,
            deployments,
        });
    }
    ensure!(!sources.is_empty(), "configure an API endpoint first");
    Ok(sources)
}

fn list_url(mut url: Url, api: &str) -> Result<Url> {
    if crate::config::mantle(&url) && url.path().trim_end_matches('/') == "/openai/v1" {
        url.set_path("/v1");
    }
    let clean_path = url.path().trim_end_matches('/');
    let decision = matches!(api, "decisions" | "systemone");
    let append_v1 =
        api == "anthropic" && !clean_path.ends_with("/v1") || decision && clean_path.is_empty();
    let strip_method = decision && clean_path.rsplit('/').next() == Some(api);
    {
        let mut path = url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("invalid API base"))?;
        path.pop_if_empty();
        if strip_method {
            path.pop();
        }
        if append_v1 {
            path.push("v1");
        }
        path.push("models");
    }
    if api == "gemini" {
        url.query_pairs_mut().append_pair("pageSize", "1000");
    }
    Ok(url)
}

#[cfg(test)]
#[test]
fn console_catalogs_deduplicate_by_plan_and_allow_an_explicit_override() -> Result<()> {
    let mut bases: Endpoints = nextjson::from_value(nextjson::json!({
            "chat":"https://opencode.ai/zen/go/v1", "gemini":"https://opencode.ai/inference/go/google/v1beta",
            "anthropic":"https://opencode.ai/inference/anthropic"})).map_err(anyhow::Error::msg)?;
    let locations = sources(&bases)?;
    assert_eq!(
        locations
            .iter()
            .map(|s| (s.url.as_str(), s.api))
            .collect::<Vec<_>>(),
        [
            ("https://opencode.ai/inference/go/v1/models", "chat"),
            ("https://opencode.ai/inference/v1/models", "chat")
        ]
    );
    bases.models_url = "https://relay.example/account/models".into();
    let locations = sources(&bases)?;
    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0].url.as_str(), bases.models_url);
    Ok(())
}

/// Merge complete protocol catalogs in configuration order. A broken page
/// invalidates that source only. Discovery never changes the saved model IDs.
pub async fn discover(bases: &Endpoints, key_env: &str) -> Result<Vec<String>> {
    Ok(discover_catalog(bases, key_env).await?.models)
}

#[derive(Default, nextjson::NsonSerialize)]
pub struct ModelCatalog {
    pub models: Vec<String>,
    pub free_models: std::collections::BTreeSet<String>,
    pub model_protocols: BTreeMap<String, RelayProtocols>,
    pub model_reasoning_levels: BTreeMap<String, Vec<String>>,
    #[njson(skip)]
    pub learned_auth: Option<crate::config::AuthHeader>,
}

pub async fn discover_catalog(bases: &Endpoints, key_env: &str) -> Result<ModelCatalog> {
    if let Some(v) = &bases.vertex {
        v.validate(!key_env.is_empty(), &bases.models_url)?;
    }
    if let Some(ids) = fixed_models(bases) {
        return Ok(ModelCatalog {
            models: ids.iter().map(|&id| id.to_owned()).collect(),
            ..Default::default()
        });
    }
    let sources = sources(bases)?;
    let key = if key_env.is_empty() {
        None
    } else {
        Some(
            std::env::var(key_env)
                .ok()
                .filter(|k| !k.is_empty())
                .context("provider key environment variable is missing or empty")?,
        )
    };
    let mut errors = Vec::new();
    let mut catalog = ModelCatalog::default();
    let mut seen = HashSet::new();
    let bearer = match (&bases.vertex, &key) {
        (Some(vertex), None) => Some(
            crate::vertex::auth::TokenCache::default()
                .token(vertex)
                .await?,
        ),
        _ => None,
    };
    let mut auth = CatalogAuth {
        key: key.as_deref(),
        bearer: bearer.as_deref(),
        mode: bases.auth_header,
        learned: None,
    };
    for source in sources {
        match fetch(&source, &mut auth).await {
            Ok(page) => {
                catalog
                    .models
                    .extend(page.models.into_iter().filter(|m| seen.insert(m.clone())));
                catalog.free_models.extend(page.free_models);
                for (id, protocols) in page.model_protocols {
                    catalog.model_protocols.entry(id).or_insert(protocols);
                }
                for (id, levels) in page.model_reasoning_levels {
                    catalog.model_reasoning_levels.entry(id).or_insert(levels);
                }
            }
            Err(e) => {
                let mut location = source.url;
                location.set_query(None);
                errors.push(format!("{} {}: {e}", source.api, location));
            }
        }
    }
    ensure!(!catalog.models.is_empty(), "{}", errors.join("; "));
    catalog.learned_auth = auth.learned;
    Ok(catalog)
}

pub async fn discover_provider(
    dir: &std::path::Path,
    provider: &crate::config::Provider,
) -> Result<Vec<String>> {
    let catalog = discover_catalog(&provider.endpoints, &provider.key_env).await?;
    if let Some(mode) = catalog.learned_auth {
        let _ = crate::config::remember_auth(dir, provider, mode);
    }
    Ok(catalog.models)
}

const MAX_CATALOG: usize = 4 * 1024 * 1024;
const PAGE_TIMEOUT: Duration = Duration::from_secs(20);

struct CatalogAuth<'a> {
    key: Option<&'a str>,
    bearer: Option<&'a str>,
    mode: AuthHeader,
    learned: Option<AuthHeader>,
}

impl CatalogAuth<'_> {
    async fn page(
        &mut self,
        client: &Client,
        url: &Url,
        api: &str,
    ) -> Result<(u16, courierust::Bytes)> {
        let mut headers = HeaderMap::new();
        if crate::opencode::Console::request(url).is_some() {
            headers.insert(
                "user-agent".into(),
                crate::opencode::USER_AGENT_VALUE.into(),
            );
        }
        if api == "anthropic" {
            headers.insert("anthropic-version".into(), "2023-06-01".into());
        }
        if matches!(api, "decisions" | "systemone") {
            headers.insert("x-magpie-deciders".into(), "true".into());
        }
        if let Some(token) = self.bearer {
            headers.insert(
                "authorization".into(),
                HeaderValue::from_bytes(format!("Bearer {token}").as_bytes())?,
            );
        } else if let Some(key) = self.key {
            for &name in api_key_headers(url, api, self.learned.unwrap_or(self.mode)) {
                let value = if name == "authorization" {
                    HeaderValue::from_bytes(format!("Bearer {key}").as_bytes())?
                } else {
                    HeaderValue::from_bytes(key.as_bytes())?
                };
                headers.insert(name.into(), value);
            }
        } else if crate::bedrock::scope(url).is_some() {
            for (name, value) in &crate::bedrock::catalog_headers(url)? {
                headers.insert(
                    HeaderName::from_bytes(name.as_str().as_bytes())?,
                    HeaderValue::from_bytes(value.as_bytes())?,
                );
            }
        }
        let negotiate = api == "anthropic"
            && headers.contains_key("authorization")
            && headers.contains_key("x-api-key");
        let mut reply = get(client, url, headers.clone()).await?;
        if negotiate
            && reply.0 == 401
            && crate::anthropic_auth::conflict(&reply.1[..reply.1.len().min(64 * 1024)])
        {
            for (mode, remove) in [
                (AuthHeader::ApiKey, "authorization"),
                (AuthHeader::Bearer, "x-api-key"),
            ] {
                let mut single = headers.clone();
                single.remove(remove);
                reply = get(client, url, single).await?;
                if (200..300).contains(&reply.0) {
                    self.learned = Some(mode);
                }
                if !matches!(reply.0, 401 | 403) {
                    break;
                }
            }
        }
        Ok(reply)
    }
}

async fn get(client: &Client, url: &Url, headers: HeaderMap) -> Result<(u16, courierust::Bytes)> {
    let client = client.clone();
    let url = url.to_string();
    tokio::time::timeout(
        PAGE_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            let mut request = client.request(&url, Method::GET);
            for (name, value) in headers.iter() {
                request = request.header(name.clone(), value.clone());
            }
            let response = request.send().context("model catalog request failed")?;
            Ok((
                response.status.as_u16(),
                response.body.collect_limited(MAX_CATALOG)?,
            ))
        }),
    )
    .await
    .context("model catalog request timed out")??
}

async fn fetch(source: &Source, auth: &mut CatalogAuth<'_>) -> Result<ModelCatalog> {
    let Source {
        ref url,
        api,
        deployments,
    } = *source;
    let endpoint = url.clone();
    let client = tokio::task::spawn_blocking(move || {
        crate::upstream::client(&endpoint, MAX_CATALOG, PAGE_TIMEOUT)
    })
    .await??;
    let mut budget = MAX_CATALOG;
    let mut cursors = HashSet::new();
    let mut catalog = ModelCatalog::default();
    let mut page = url.clone();
    if deployments {
        page.set_path("/openai/deployments");
        page.set_query(Some("api-version=2022-12-01"));
    }
    for _ in 0..32 {
        let (status, bytes) = auth.page(&client, &page, api).await?;
        if deployments && page.path() == "/openai/deployments" && !(200..300).contains(&status) {
            page = url.clone();
            continue;
        }
        ensure!(
            (200..300).contains(&status),
            "model catalog returned HTTP {status}"
        );
        ensure!(bytes.len() <= budget, "model catalog exceeds 4 MiB");
        budget -= bytes.len();
        let entries: List = nextjson::from_slice(&bytes)
            .map_err(|e| anyhow::anyhow!("invalid model catalog: {e}"))?;
        for entry in entries.data.into_iter().chain(entries.models) {
            if api == "gemini"
                && entry.methods.as_ref().is_some_and(|methods| {
                    !methods.is_empty() && !methods.iter().any(|s| s == "generateContent")
                })
            {
                continue;
            }
            if let Some(id) = entry.id.as_deref().or(entry.name.as_deref()) {
                let id = if api == "gemini" {
                    id.strip_prefix("models/").unwrap_or(id)
                } else {
                    id
                };
                if !id.is_empty() {
                    if entry.free.as_ref().and_then(nextjson::Value::as_bool) == Some(true) {
                        catalog.free_models.insert(id.to_owned());
                    }
                    if let Some(hints) = entry.relay_protocols() {
                        catalog
                            .model_protocols
                            .entry(id.to_owned())
                            .or_insert(hints);
                    }
                    if let Some(values) = entry
                        .reasoning_levels
                        .as_ref()
                        .and_then(nextjson::Value::as_array)
                    {
                        let mut seen = HashSet::new();
                        let levels = values
                            .iter()
                            .filter_map(nextjson::Value::as_str)
                            .filter(|level| {
                                crate::config::effort::rank(level).is_some() && seen.insert(*level)
                            })
                            .map(str::to_owned)
                            .collect::<Vec<_>>();
                        if !levels.is_empty() {
                            catalog
                                .model_reasoning_levels
                                .entry(id.to_owned())
                                .or_insert(levels);
                        }
                    }
                    catalog.models.push(id.to_owned());
                }
            }
        }
        let cursor = if api == "gemini" {
            entries
                .next
                .filter(|v| !v.is_empty())
                .map(|v| ("pageToken", v))
        } else if api == "anthropic" && entries.has_more == Some(true) {
            Some((
                "after_id",
                entries.last_id.context("missing catalog cursor")?,
            ))
        } else {
            None
        };
        let Some((key, value)) = cursor else {
            if catalog.models.is_empty() && deployments && page.path() == "/openai/deployments" {
                page = url.clone();
                continue;
            }
            ensure!(!catalog.models.is_empty(), "upstream returned no model IDs");
            return Ok(catalog);
        };
        ensure!(
            !value.is_empty() && cursors.insert(value.clone()),
            "invalid or repeated model catalog cursor"
        );
        page = url.clone();
        if url.query_pairs().any(|(k, _)| k == key) {
            page.set_query(None);
            page.query_pairs_mut()
                .extend_pairs(url.query_pairs().filter(|(k, _)| k != key));
        }
        page.query_pairs_mut().append_pair(key, &value);
    }
    bail!("model catalog exceeds 32 pages")
}
