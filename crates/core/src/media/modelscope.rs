use super::{Upload, asset_url, dashscope, image_reference, image_type};
use crate::{config, gateway, upstream::ForwardClient};
use anyhow::{Context, Result, bail, ensure};
use axum_core::{body::Body, response::Response};
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use http::{Method, Request, StatusCode, header};
use http_body_util::{BodyExt, Limited};
use nextjson::{Map, Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, Weak},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex as AsyncMutex;
use url::Url;

const HOST: &str = "api-inference.modelscope.cn";
const MAX_IMAGES: usize = 10;
const MAX_ASSETS: usize = 16 * 1024 * 1024;
const MAX_REPLY: usize = 1024 * 1024;
const TASK_TYPE: &str = "x-modelscope-task-type";
const ASYNC_MODE: &str = "x-modelscope-async-mode";
const TASK_DEADLINE: Duration = Duration::from_secs(180);
const BUSY_DELAYS: [u64; 3] = [3, 8, 15];

pub(crate) fn host(url: &Url) -> bool {
    url.host_str() == Some(HOST)
}

pub(crate) fn destination(endpoints: &config::Endpoints) -> Option<Url> {
    let mut url = config::api_base(&endpoints.chat, "chat")
        .ok()
        .filter(host)?;
    if url.path() == "/" {
        url.set_path("/v1");
    }
    url.path_segments_mut()
        .ok()?
        .pop_if_empty()
        .extend(["images", "generations"]);
    Some(url)
}

type QueueKey = [u8; 32];

#[derive(Default)]
pub(crate) struct Queues(Mutex<BTreeMap<QueueKey, Weak<AsyncMutex<()>>>>);

impl Queues {
    fn for_request(&self, request: &Request<Vec<Bytes>>) -> Arc<AsyncMutex<()>> {
        use sha2::{Digest, Sha256};

        let credential = request.headers().get(header::AUTHORIZATION);
        let key = Sha256::digest(credential.map_or(&[][..], |value| value.as_bytes())).into();
        let mut queues = self.0.lock().unwrap_or_else(|error| error.into_inner());
        queues.retain(|_, queue| queue.strong_count() > 0);
        if let Some(queue) = queues.get(&key).and_then(Weak::upgrade) {
            return queue;
        }
        let queue = Arc::new(AsyncMutex::new(()));
        queues.insert(key, Arc::downgrade(&queue));
        queue
    }
}

pub(crate) struct Drawing {
    count: usize,
    inline: bool,
}

impl Drawing {
    pub(crate) fn prepare(
        model: &str,
        bytes: &[u8],
        upload: Option<&Upload>,
        edit: bool,
    ) -> Result<(Bytes, Self)> {
        let mut images = Vec::new();
        let fields = if let Some(upload) = upload {
            let mut fields = Map::new();
            for field in &upload.fields {
                let name = field.name.as_str();
                if matches!(name, "image" | "image[]" | "images" | "image_url") {
                    images.push(field.image_reference()?);
                    continue;
                }
                ensure!(
                    field.filename.is_none(),
                    "unsupported image file field {name}"
                );
                let text = std::str::from_utf8(&field.value)?;
                let value = if name == "n" {
                    json!(
                        text.trim()
                            .parse::<u64>()
                            .context("expected an integer n")?
                    )
                } else {
                    json!(text)
                };
                ensure!(
                    fields.insert(name.to_owned(), value).is_none(),
                    "duplicate image field {name}"
                );
            }
            fields
        } else {
            let value: Value = nextjson::from_slice(bytes).map_err(anyhow::Error::msg)?;
            let Value::Object(fields) = value else {
                bail!("expected an image request object");
            };
            fields
        };
        let mut native = json!({"model":model});
        let object = native.as_object_mut().expect("image request object");
        let mut count = 1;
        let mut inline = false;
        for (name, value) in fields {
            if value.is_null() {
                continue;
            }
            match name.as_str() {
                "model" | "user" => {}
                "image" | "image[]" | "images" | "image_url" => {
                    for image in value
                        .as_array()
                        .map_or(std::slice::from_ref(&value), Vec::as_slice)
                    {
                        images.push(image_reference(image)?.to_owned());
                    }
                }
                "n" => {
                    let n = value.as_u64().context("expected an integer n")?;
                    ensure!(
                        (1..=MAX_IMAGES as u64).contains(&n),
                        "image count must be between 1 and {MAX_IMAGES}"
                    );
                    count = n as usize;
                }
                "response_format" => {
                    inline = match value.as_str() {
                        Some("" | "url") => false,
                        Some("b64_json") => true,
                        _ => bail!("ModelScope image response_format must be url or b64_json"),
                    };
                }
                "size" => {
                    let size = value.as_str().context("expected a string size")?.trim();
                    if !size.is_empty() && !size.eq_ignore_ascii_case("auto") {
                        object.insert(name, json!(size));
                    }
                }
                "prompt" => {
                    object.insert(name, value);
                }
                "background" | "quality" if value.as_str() == Some("auto") => {}
                "stream" if value.as_bool() == Some(false) => {}
                _ => bail!("unsupported ModelScope image field {name}"),
            }
        }
        ensure!(
            object
                .get("prompt")
                .and_then(Value::as_str)
                .is_some_and(|prompt| !prompt.trim().is_empty()),
            "image prompt is empty"
        );
        ensure!(
            images.len() <= MAX_IMAGES,
            "too many ModelScope reference images"
        );
        ensure!(!edit || !images.is_empty(), "image edit requires an image");
        if !images.is_empty() {
            object.insert("image_url".into(), json!(images));
        }
        let body = Bytes::from(nextjson::to_vec(&native).map_err(anyhow::Error::msg)?);
        Ok((body, Self { count, inline }))
    }

    pub(crate) async fn send(
        &self,
        client: &ForwardClient,
        mut request: Request<Vec<Bytes>>,
        queues: &Queues,
    ) -> Result<Response> {
        let source = Url::parse(&request.uri().to_string())?;
        request.headers_mut().insert(ASYNC_MODE, "true".parse()?);
        let queue = queues.for_request(&request);
        let deadline = tokio::time::Instant::now() + TASK_DEADLINE;
        let mut data = Vec::new();
        let mut remaining = MAX_ASSETS;
        for _ in 0..self.count {
            let draw = async {
                let _guard = queue.lock().await;
                let urls = match task(client, &request, &source).await? {
                    Ok(urls) => urls,
                    Err(response) => return Ok(Err(response)),
                };
                drop(_guard);
                let mut images = Vec::new();
                for url in urls {
                    if self.inline {
                        let bytes = download(client, &source, url, remaining).await?;
                        remaining -= bytes.len();
                        images.push(json!({"b64_json":STANDARD.encode(bytes)}));
                    } else {
                        images.push(json!({"url":url.as_str()}));
                    }
                }
                Ok::<_, anyhow::Error>(Ok(images))
            };
            match tokio::time::timeout_at(deadline, draw).await {
                Ok(Ok(Ok(images))) => data.extend(images),
                _ if !data.is_empty() => break,
                Ok(Ok(Err(response))) => return Ok(response),
                Ok(Err(error)) => return Err(error),
                Err(_) => {
                    return Ok(gateway::failure(
                        StatusCode::GATEWAY_TIMEOUT,
                        "ModelScope image task timed out",
                    ));
                }
            }
            if data.len() >= MAX_IMAGES || remaining == 0 {
                break;
            }
        }
        data.truncate(MAX_IMAGES);
        let created = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Ok(gateway::json_response(
            StatusCode::OK,
            &json!({"created":created,"data":data}),
        ))
    }
}

async fn task(
    client: &ForwardClient,
    request: &Request<Vec<Bytes>>,
    source: &Url,
) -> Result<Result<Vec<Url>, Response>> {
    let mut delays = BUSY_DELAYS.into_iter();
    let start = loop {
        let response = client.send(request.clone()).await?;
        let (mut parts, body) = response.into_parts();
        let bytes = read(body, MAX_REPLY).await?;
        let busy = parts.status == StatusCode::TOO_MANY_REQUESTS
            || (!parts.status.is_success()
                && String::from_utf8_lossy(&bytes).contains("queue is full"));
        if busy {
            if let Some(seconds) = delays.next() {
                tokio::time::sleep(Duration::from_secs(seconds)).await;
                continue;
            }
            parts.status = StatusCode::TOO_MANY_REQUESTS;
        }
        if !parts.status.is_success() {
            return Ok(Err(Response::from_parts(parts, Body::from(bytes))));
        }
        break nextjson::from_slice::<Value>(&bytes).map_err(anyhow::Error::msg)?;
    };
    let images = pictures(&start)?;
    if !images.is_empty() {
        return Ok(Ok(images));
    }
    let id = start
        .get("task_id")
        .and_then(Value::as_str)
        .filter(|id| {
            !id.is_empty()
                && id.len() <= 128
                && id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        })
        .context("ModelScope returned no valid image task ID")?;
    let mut url = source.clone();
    url.path_segments_mut()
        .expect("validated ModelScope URL")
        .pop()
        .pop()
        .extend(["tasks", id]);
    let mut poll = Request::builder()
        .method(Method::GET)
        .uri(url.as_str())
        .header(TASK_TYPE, "image_generation")
        .header(header::CONTENT_LENGTH, 0)
        .body(Vec::new())?;
    if let Some(auth) = request.headers().get(header::AUTHORIZATION) {
        poll.headers_mut()
            .insert(header::AUTHORIZATION, auth.clone());
    }
    loop {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let response = client.send(poll.clone()).await?;
        if !response.status().is_success() {
            return Ok(Err(response));
        }
        let bytes = read(response.into_body(), MAX_REPLY).await?;
        let result: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
        let status = result
            .get("task_status")
            .and_then(Value::as_str)
            .context("ModelScope returned no task status")?;
        match status.to_ascii_uppercase().as_str() {
            "SUCCEED" | "SUCCEEDED" | "SUCCESS" => {
                let images = pictures(&result)?;
                ensure!(
                    !images.is_empty(),
                    "ModelScope finished the task with no usable images"
                );
                return Ok(Ok(images));
            }
            "FAILED" | "FAILURE" | "CANCELED" | "CANCELLED" => {
                bail!("ModelScope image task failed")
            }
            _ => {}
        }
    }
}

fn pictures(result: &Value) -> Result<Vec<Url>> {
    let strings = |field| {
        result
            .get(field)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
    };
    let urls: Vec<_> = strings("output_images")
        .filter_map(Value::as_str)
        .chain(strings("images").filter_map(|image| image.get("url")?.as_str()))
        .filter_map(asset_url)
        .take(MAX_IMAGES + 1)
        .collect();
    ensure!(
        urls.len() <= MAX_IMAGES,
        "ModelScope returned too many images"
    );
    Ok(urls)
}

async fn read(body: Body, limit: usize) -> Result<Bytes> {
    Ok(Limited::new(body, limit)
        .collect()
        .await
        .map_err(anyhow::Error::from_boxed)?
        .to_bytes())
}

async fn download(
    client: &ForwardClient,
    source: &Url,
    mut url: Url,
    limit: usize,
) -> Result<Bytes> {
    if dashscope::public_oss(&url) {
        url.set_scheme("https")
            .map_err(|()| anyhow::anyhow!("invalid ModelScope asset URL"))?;
    } else {
        ensure!(
            url.origin() == source.origin(),
            "ModelScope download requires its API origin or a public Alibaba Cloud OSS URL"
        );
    }
    let response = client
        .send(
            Request::get(url.as_str())
                .header(header::CONTENT_LENGTH, 0)
                .body(Vec::new())?,
        )
        .await?;
    ensure!(
        response.status().is_success(),
        "ModelScope image download failed"
    );
    let bytes = read(response.into_body(), limit).await?;
    ensure!(
        image_type(&bytes).is_some(),
        "downloaded ModelScope result is not an image"
    );
    Ok(bytes)
}
