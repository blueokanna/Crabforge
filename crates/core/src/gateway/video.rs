use super::{Gateway, failure, json_response, upstream_response};
use crate::config::{Provider, authenticate};
use crate::media::video::{Kind, route};
use anyhow::{Context, Result, ensure};
use axum_core::{body::Body, response::Response};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use bytes::Bytes;
use http::{HeaderMap, Request, StatusCode, header, request::Parts};
use http_body_util::{BodyExt, Limited};
use nextjson::{NsonDeserialize, NsonSerialize, Value, json};
use ring::hmac;
use std::{sync::Arc, time::Duration};
use tokio::sync::OwnedSemaphorePermit;
use url::Url;
use zeroize::Zeroizing;

pub(super) const CREATE: &str = "videos";
const MAX_METADATA: usize = 1024 * 1024;
const MAX_JOB_ID: usize = 2048;
const MAX_VIDEO: usize = 512 * 1024 * 1024;

#[derive(NsonSerialize, NsonDeserialize)]
struct Job {
    provider: String,
    model: String,
    upstream: String,
    started: i64,
}

pub(super) async fn signer(
    gateway: &Gateway,
    provider: &Provider,
    base: &Url,
    key: Option<&str>,
) -> Result<hmac::Key> {
    let master = gateway
        .video_key
        .get_or_try_init(|| async {
            let root = gateway.root.clone();
            tokio::task::spawn_blocking(move || {
                let secret = Zeroizing::new(crate::config::private_file(
                    &root,
                    "video-jobs.key",
                    || {
                        let mut secret = vec![0; 32];
                        getrandom::fill(&mut secret)
                            .map_err(|error| anyhow::anyhow!("OS randomness: {error}"))?;
                        Ok(secret)
                    },
                )?);
                ensure!(secret.len() == 32, "invalid video job signing key");
                Ok(hmac::Key::new(hmac::HMAC_SHA256, &secret))
            })
            .await?
        })
        .await?;
    let mut context = hmac::Context::with_key(master);
    context.update(b"crabforge video job v1\0");
    context.update(if provider.remote_gateway {
        b"relay\0"
    } else {
        b"local\0"
    });
    context.update(base.as_str().as_bytes());
    context.update(b"\0");
    context.update(key.unwrap_or_default().as_bytes());
    Ok(hmac::Key::new(hmac::HMAC_SHA256, context.sign().as_ref()))
}

fn upstream_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 768
        && !matches!(id, "." | "..")
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

async fn metadata_bytes(body: Body) -> Result<Bytes> {
    let bytes = tokio::time::timeout(
        Duration::from_secs(30),
        Limited::new(body, MAX_METADATA).collect(),
    )
    .await?
    .map_err(anyhow::Error::from_boxed)?
    .to_bytes();
    Ok(bytes)
}

fn metadata(bytes: &[u8]) -> Result<nextjson::Map> {
    match nextjson::from_slice(bytes).map_err(anyhow::Error::msg)? {
        Value::Object(object) => Ok(object),
        _ => anyhow::bail!("video metadata must be an object"),
    }
}

pub(super) async fn business_status(kind: Kind, response: Response) -> Result<Response> {
    if !matches!(kind, Kind::MiniMax | Kind::DashScope) || !response.status().is_success() {
        return Ok(response);
    }
    let (mut parts, body) = response.into_parts();
    let bytes = metadata_bytes(body).await?;
    if let Some(status) = kind.business_status(&metadata(&bytes)?) {
        parts.status = status;
    }
    Ok(Response::from_parts(parts, Body::from(bytes)))
}

pub(super) async fn created(
    response: Response,
    signer: &hmac::Key,
    provider: &Provider,
    model: &str,
    kind: Kind,
) -> Response {
    let status = response.status();
    let result = async {
        let object = metadata(&metadata_bytes(response.into_body()).await?)?;
        let upstream = kind
            .created_id(&object)
            .filter(|id| upstream_id(id))
            .context("upstream returned an invalid video ID")?
            .to_owned();
        let started = object
            .get("created_at")
            .and_then(Value::as_i64)
            .unwrap_or_else(|| time::OffsetDateTime::now_utc().unix_timestamp());
        let payload = URL_SAFE_NO_PAD.encode(
            nextjson::to_vec(&Job {
                provider: provider.id.clone(),
                model: model.into(),
                upstream: upstream.clone(),
                started,
            })
            .map_err(anyhow::Error::msg)?,
        );
        let signature = URL_SAFE_NO_PAD.encode(hmac::sign(signer, payload.as_bytes()).as_ref());
        let id = format!("video_{payload}.{signature}");
        ensure!(id.len() <= MAX_JOB_ID, "upstream video ID is too long");
        let mut object = kind
            .status(object, &upstream, started, true)
            .context("invalid video metadata")?;
        object.insert("id".into(), json!(id));
        object.insert("model".into(), json!(format!("{}/{model}", provider.id)));
        Ok::<_, anyhow::Error>(Value::Object(object))
    }
    .await;
    match result {
        Ok(object) => json_response(status, &object),
        Err(_) => failure(
            StatusCode::BAD_GATEWAY,
            "upstream returned invalid video metadata",
        ),
    }
}

pub(super) async fn retrieve(
    gateway: &Gateway,
    path: &str,
    parts: &Parts,
    request_body: impl Future<Output = Result<Bytes>>,
) -> Response {
    let (id, content) = path
        .strip_suffix("/content")
        .map_or((path, false), |id| (id, true));
    let Some((payload, signature, job)) = parse(id) else {
        return failure(StatusCode::NOT_FOUND, "unknown video ID");
    };
    let provider = gateway
        .config
        .providers
        .iter()
        .find(|p| p.id == job.provider)
        .or_else(|| {
            let current = gateway.config.former_providers.get(&job.provider)?;
            gateway.config.providers.iter().find(|p| &p.id == current)
        });
    let Some((provider, (kind, url))) = provider.and_then(|p| Some((p, route(p, &job.model)?)))
    else {
        return failure(StatusCode::NOT_FOUND, "unknown video ID");
    };
    let key = if provider.key_env.is_empty() {
        None
    } else {
        let Some(key) = std::env::var(&provider.key_env)
            .ok()
            .filter(|key| !key.is_empty())
        else {
            return failure(
                StatusCode::SERVICE_UNAVAILABLE,
                "provider key environment variable is missing or empty",
            );
        };
        Some(Zeroizing::new(key))
    };
    let key = key.as_ref().map(|key| key.as_str());
    let Ok(signer) = signer(gateway, provider, &url, key).await else {
        return failure(
            StatusCode::INTERNAL_SERVER_ERROR,
            "could not load the video job signing key",
        );
    };
    if hmac::verify(&signer, payload.as_bytes(), &signature).is_err() {
        return failure(StatusCode::NOT_FOUND, "unknown video ID");
    }
    let mut url = kind.task_url(url, &job.upstream, content);
    if let Some(query) = parts.uri.query() {
        let mut pairs = url::form_urlencoded::parse(query.as_bytes());
        let variant = pairs.next().filter(|(name, value)| {
            content
                && name == "variant"
                && matches!(value.as_ref(), "video" | "thumbnail" | "spritesheet")
        });
        match variant {
            Some((_, value))
                if pairs.next().is_none() && (kind == Kind::OpenAi || value == "video") =>
            {
                if kind == Kind::OpenAi {
                    url.query_pairs_mut().append_pair("variant", &value);
                }
            }
            _ => return failure(StatusCode::BAD_REQUEST, "invalid video query"),
        }
    }
    let (body, permit) = match gateway.read_body(request_body).await {
        Ok(request) => request,
        Err((status, message)) => return failure(status, message),
    };
    if !body.is_empty() {
        return failure(
            StatusCode::BAD_REQUEST,
            "video retrieval expects an empty body",
        );
    }
    let mut request = Request::get(url.as_str());
    if content && kind == Kind::OpenAi {
        request = content_headers(request, &parts.headers);
    }
    if let Some(key) = key {
        request = if provider.remote_gateway {
            request.header("x-api-key", key)
        } else {
            authenticate(request, &url, "openai", key, provider.endpoints.auth_header)
        };
    }
    let response = match request.body(Vec::new()) {
        Ok(request) => gateway.client.send(request).await,
        Err(error) => return failure(StatusCode::BAD_REQUEST, &error.to_string()),
    };
    let Ok(response) = response else {
        return failure(StatusCode::BAD_GATEWAY, "upstream video transport failed");
    };
    if response.status().is_redirection() {
        return failure(StatusCode::BAD_GATEWAY, "upstream video redirected");
    }
    if !response.status().is_success() {
        return upstream_response(response, permit);
    }
    if content && kind == Kind::OpenAi {
        return video_response(response, permit);
    }
    let status = response.status();
    let Ok(object) = metadata_bytes(response.into_body())
        .await
        .and_then(|bytes| metadata(&bytes))
    else {
        return failure(
            StatusCode::BAD_GATEWAY,
            "upstream returned invalid video metadata",
        );
    };
    if let Some(status) = kind.business_status(&object) {
        return failure(status, "upstream rejected the video status request");
    }
    let Some(mut object) = kind.status(object, &job.upstream, job.started, false) else {
        return failure(
            StatusCode::BAD_GATEWAY,
            "upstream returned invalid video metadata",
        );
    };
    if content {
        if object.get("status").and_then(Value::as_str) != Some("completed") {
            return failure(StatusCode::CONFLICT, "video is not ready");
        }
        let Some(asset) = kind.content_url(&url, object) else {
            return failure(
                StatusCode::BAD_GATEWAY,
                "upstream returned no allowed video URL",
            );
        };
        let request = content_headers(Request::get(asset.as_str()), &parts.headers)
            .body(Vec::new())
            .expect("validated video asset URL and HTTP headers");
        return match gateway.client.send(request).await {
            Ok(response)
                if matches!(
                    response.status(),
                    StatusCode::OK
                        | StatusCode::PARTIAL_CONTENT
                        | StatusCode::RANGE_NOT_SATISFIABLE
                ) =>
            {
                video_response(response, permit)
            }
            _ => failure(
                StatusCode::BAD_GATEWAY,
                "could not download the upstream video",
            ),
        };
    }
    object.insert("id".into(), json!(id));
    object.insert(
        "model".into(),
        json!(format!("{}/{}", provider.id, job.model)),
    );
    json_response(status, &Value::Object(object))
}

fn content_headers(
    mut request: http::request::Builder,
    headers: &HeaderMap,
) -> http::request::Builder {
    for name in [header::RANGE, header::ACCEPT] {
        if let Some(value) = headers.get(&name) {
            request = request.header(name, value);
        }
    }
    request
}

fn video_response(response: Response, permit: Arc<OwnedSemaphorePermit>) -> Response {
    if response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|length| length.parse::<usize>().ok())
        .is_some_and(|length| length > MAX_VIDEO)
    {
        return failure(StatusCode::BAD_GATEWAY, "upstream video exceeds 512 MiB");
    }
    let (parts, body) = response.into_parts();
    upstream_response(
        Response::from_parts(parts, Body::new(Limited::new(body, MAX_VIDEO))),
        permit,
    )
}

fn parse(id: &str) -> Option<(&str, Vec<u8>, Job)> {
    if id.len() > MAX_JOB_ID {
        return None;
    }
    let (payload, signature) = id.strip_prefix("video_")?.split_once('.')?;
    let signature = URL_SAFE_NO_PAD.decode(signature).ok()?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let job: Job = nextjson::from_slice(&bytes).ok()?;
    upstream_id(&job.upstream).then_some((payload, signature, job))
}
