use super::{Upload, asset_url, image_source, image_type};
use crate::{config::Provider, upstream::ForwardClient};
use anyhow::{Context, Result, bail, ensure};
use axum_core::response::Response;
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use http::{Request, header};
use http_body_util::{BodyExt, Limited};
use nextjson::{Value, json};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::{Host, Url};

pub(crate) mod audio;

pub(crate) const MULTIMODAL: &str = "api/v1/services/aigc/multimodal-generation/generation";
pub(crate) const SPEECH: &str = "api/v1/services/audio/tts/SpeechSynthesizer";
const MAX_IMAGES: usize = 4;
const MAX_ASSET_BYTES: usize = 16 * 1024 * 1024;

pub(crate) fn destination(
    provider: &Provider,
    model: &str,
    path: &str,
) -> Option<(Url, &'static str)> {
    let model = model.to_ascii_lowercase();
    let native = match path {
        "images/generations" | "images/edits"
            if model.starts_with("qwen-image") || model.starts_with("wan2.7-image") =>
        {
            MULTIMODAL
        }
        "audio/speech" if model.starts_with("qwen-audio") && model.contains("-tts") => SPEECH,
        "audio/transcriptions" | "audio/translations"
            if model.starts_with("qwen-audio") && model.contains("-asr") =>
        {
            MULTIMODAL
        }
        _ => return None,
    };
    let mut url = [&provider.endpoints.chat, &provider.endpoints.anthropic]
        .into_iter()
        .filter_map(|base| crate::config::endpoint(base).ok())
        .find(|url| {
            let host = url.host_str().unwrap_or_default();
            (host.starts_with("dashscope") && host.ends_with(".aliyuncs.com"))
                || host.ends_with(".maas.aliyuncs.com")
                || ["maas.qianwenaiapi.com", "maas.qwencloudapi.com"]
                    .iter()
                    .any(|domain| {
                        host == *domain
                            || host
                                .strip_suffix(domain)
                                .is_some_and(|label| label.ends_with('.'))
                    })
                || (loopback(url)
                    && matches!(
                        url.path().trim_end_matches('/'),
                        "/compatible-mode/v1" | "/apps/anthropic"
                    ))
        })?;
    url.set_path(native);
    Some((url, native))
}

pub(crate) struct ImageRequest {
    pub body: Bytes,
    pub inline: bool,
}

impl ImageRequest {
    pub(crate) fn read(
        model: &str,
        bytes: &[u8],
        upload: Option<&Upload>,
        edit: bool,
    ) -> Result<Self> {
        let mut images = Vec::new();
        let request = if let Some(form) = upload {
            let mut request = json!({});
            for field in &form.fields {
                let name = field.name.as_str();
                match name {
                    "image" | "image[]" | "images" => {
                        let part = if field.filename.is_some()
                            || field
                                .mime
                                .as_deref()
                                .is_some_and(|mime| mime.starts_with("image/"))
                        {
                            let mime =
                                image_type(&field.value).context("unrecognized image file")?;
                            format!("data:{mime};base64,{}", STANDARD.encode(&field.value))
                        } else {
                            image_source(std::str::from_utf8(&field.value)?.trim())?.to_owned()
                        };
                        images.push(json!({"image":part}));
                    }
                    "mask" | "markup" => bail!("DashScope image edits accept no {name}"),
                    _ => {
                        ensure!(field.filename.is_none(), "unexpected file field {name}");
                        let value = std::str::from_utf8(&field.value)?.trim();
                        let value = match name {
                            "n" => json!(value.parse::<u64>().context("invalid image count")?),
                            "watermark" | "stream" => json!(
                                value
                                    .parse::<bool>()
                                    .with_context(|| format!("expected a boolean {name}"))?
                            ),
                            _ => json!(value),
                        };
                        let fields = request.as_object_mut().expect("image request object");
                        ensure!(
                            fields.insert(name.into(), value).is_none(),
                            "duplicate image field {name}"
                        );
                    }
                }
            }
            request
        } else {
            nextjson::from_slice(bytes).map_err(anyhow::Error::msg)?
        };
        let fields = request
            .as_object()
            .context("expected an image request object")?;
        for (name, value) in fields.iter().filter(|(_, value)| !value.is_null()) {
            match name {
                "model" | "prompt" | "size" | "response_format" | "n" | "watermark" | "user" => {}
                "image" | "images" | "image[]" => {
                    let list = value
                        .as_array()
                        .map(Vec::as_slice)
                        .unwrap_or_else(|| std::slice::from_ref(value));
                    for image in list {
                        let link = image.get("image_url").unwrap_or(image);
                        let url = link
                            .as_str()
                            .or_else(|| link.get("url").and_then(Value::as_str));
                        images.push(json!({"image":image_source(url.context("expected an image URL or data URL")?)?}));
                    }
                }
                "background" | "quality" if value.as_str() == Some("auto") => {}
                "stream" if value.as_bool() == Some(false) => {}
                "mask" | "markup" => bail!("DashScope image edits accept no {name}"),
                _ => bail!("unsupported DashScope image field {name}"),
            }
        }
        ensure!(images.len() <= 16, "too many input images");
        ensure!(!edit || !images.is_empty(), "image edit requires an image");
        let text = |name| {
            fields
                .get(name)
                .filter(|value| !value.is_null())
                .map_or(Ok(""), |value| {
                    value
                        .as_str()
                        .with_context(|| format!("expected string {name}"))
                })
        };
        let prompt = text("prompt")?;
        ensure!(!prompt.trim().is_empty(), "image prompt is empty");
        images.push(json!({"text":prompt}));
        let inline = match text("response_format")?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "" | "url" => false,
            "b64_json" => true,
            _ => bail!("DashScope image response_format must be url or b64_json"),
        };
        let watermark = fields
            .get("watermark")
            .filter(|value| !value.is_null())
            .map_or(Ok(false), |value| {
                value.as_bool().context("expected a boolean watermark")
            })?;
        let count = fields
            .get("n")
            .filter(|value| !value.is_null())
            .map_or(Ok(1), |value| {
                value.as_u64().context("expected an integer image count")
            })?;
        ensure!(
            (1..=MAX_IMAGES as u64).contains(&count),
            "image count must be between 1 and {MAX_IMAGES}"
        );
        let mut parameters = json!({"watermark":watermark});
        let fields = parameters.as_object_mut().expect("image parameters object");
        if count > 1 {
            fields.insert("n".into(), json!(count));
        }
        let size = text("size")?.trim();
        if !size.is_empty() && !size.eq_ignore_ascii_case("auto") {
            let (width, height) = size
                .split_once(['x', 'X', '*'])
                .and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)))
                .filter(|(w, h)| *w > 0 && *h > 0)
                .context("DashScope image size must be WIDTHxHEIGHT or auto")?;
            fields.insert("size".into(), json!(format!("{width}*{height}")));
        }
        let body = nextjson::to_vec(&json!({
            "model":model,
            "input":{"messages":[{"role":"user","content":images}]},
            "parameters":parameters
        }))
        .map_err(anyhow::Error::msg)?;
        Ok(Self {
            body: body.into(),
            inline,
        })
    }
}

fn loopback(url: &Url) -> bool {
    match url.host() {
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        Some(Host::Domain("localhost")) => true,
        _ => false,
    }
}

pub(super) fn public_oss(url: &Url) -> bool {
    // Signed DashScope image URLs use public Alibaba Cloud OSS bucket hosts.
    url.port().is_none()
        && url
            .host_str()
            .and_then(|host| host.strip_suffix(".aliyuncs.com"))
            .and_then(|host| host.rsplit_once('.'))
            .is_some_and(|(bucket, region)| {
                !bucket.is_empty() && region.starts_with("oss-") && !region.ends_with("-internal")
            })
}

pub(crate) async fn image_reply(
    upstream: Response,
    inline: bool,
    source: &Url,
    client: &ForwardClient,
) -> Result<Response> {
    let (parts, body) = upstream.into_parts();
    let bytes = Limited::new(body, 1024 * 1024)
        .collect()
        .await
        .map_err(|_| anyhow::anyhow!("could not read DashScope image result within 1 MiB"))?
        .to_bytes();
    let reply: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
    let choices = reply
        .pointer("/output/choices")
        .and_then(Value::as_array)
        .context("DashScope returned no image choices")?;
    let mut links = Vec::new();
    for choice in choices {
        if let Some(content) = choice.pointer("/message/content").and_then(Value::as_array) {
            for part in content {
                if let Some(raw) = part.get("image").and_then(Value::as_str) {
                    let url = asset_url(raw).context("DashScope returned an invalid image URL")?;
                    ensure!(
                        links.len() < MAX_IMAGES,
                        "DashScope returned too many images"
                    );
                    links.push((raw, url));
                }
            }
        }
    }
    ensure!(!links.is_empty(), "DashScope returned no images");
    let mut remaining = MAX_ASSET_BYTES;
    let mut data = Vec::with_capacity(links.len());
    for (raw, url) in links {
        let image = if inline {
            let image = download(client, source, &url, remaining).await?;
            ensure!(
                image_type(&image).is_some(),
                "downloaded DashScope result is not an image"
            );
            remaining -= image.len();
            json!({"b64_json":STANDARD.encode(image)})
        } else {
            json!({"url":raw})
        };
        data.push(image);
    }
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut response =
        crate::gateway::json_response(parts.status, &json!({"created":created,"data":data}));
    for name in ["x-request-id", "request-id"] {
        if let Some(value) = parts.headers.get(name) {
            response.headers_mut().insert(name, value.clone());
        }
    }
    Ok(response)
}

async fn download(client: &ForwardClient, source: &Url, url: &Url, limit: usize) -> Result<Bytes> {
    let mut url = url.clone();
    if public_oss(&url) {
        // DashScope sometimes returns an HTTP signed URL. OSS also serves it over TLS.
        url.set_scheme("https")
            .map_err(|()| anyhow::anyhow!("invalid DashScope OSS URL"))?;
    } else {
        ensure!(
            loopback(source) && url.origin() == source.origin(),
            "DashScope result download requires a public Alibaba Cloud OSS URL"
        );
    }
    let download = async {
        // Use a new request without credentials. The forwarding client disables redirects.
        let response = client
            .send(
                Request::get(url.as_str())
                    .header(header::CONTENT_LENGTH, 0)
                    .body(Vec::new())?,
            )
            .await
            .map_err(|_| anyhow::anyhow!("DashScope result download failed"))?;
        ensure!(
            response.status().is_success(),
            "DashScope result download returned HTTP {}",
            response.status().as_u16()
        );
        Ok(Limited::new(response.into_body(), limit)
            .collect()
            .await
            .map_err(|_| {
                anyhow::anyhow!("could not read DashScope result within the 16 MiB response limit")
            })?
            .to_bytes())
    };
    tokio::time::timeout(Duration::from_secs(60), download)
        .await
        .context("DashScope result download timed out")?
}
