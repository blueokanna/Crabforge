use super::{Upload, asset_url, image_reference, image_type};
use crate::{
    config::{Endpoints, endpoint},
    upstream::ForwardClient,
};
use anyhow::{Context, Result, bail, ensure};
use axum_core::{body::Body, response::Response};
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use http::{Request, StatusCode, header, response::Parts};
use http_body_util::{BodyExt, Limited};
use nextjson::{Map, Value, json};
use url::Url;

pub(crate) mod video;

const MAX_DRAWS: u64 = 4;
const MAX_IMAGES: usize = 15;
const MAX_REPLY_BYTES: usize = 24 * 1024 * 1024;

pub(crate) fn host(url: &Url) -> bool {
    url.host_str()
        .is_some_and(|host| host.starts_with("ark.") && host.ends_with(".volces.com"))
}

fn base(endpoints: &Endpoints) -> Option<Url> {
    [&endpoints.chat, &endpoints.responses]
        .into_iter()
        .filter_map(|base| endpoint(base).ok())
        .find(host)
}

pub(crate) fn destination(endpoints: &Endpoints) -> Option<Url> {
    let mut url = base(endpoints)?;
    if url.path() == "/" {
        url.set_path("/api/v3");
    }
    url.path_segments_mut()
        .ok()?
        .pop_if_empty()
        .extend(["images", "generations"]);
    Some(url)
}

pub(crate) fn image_model(endpoints: &Endpoints, model: &str) -> bool {
    let model = model.to_ascii_lowercase();
    (model.starts_with("doubao-seedream-") || model.starts_with("seedream-"))
        && base(endpoints).is_some()
}

pub(crate) struct Drawing {
    count: u64,
    inline: bool,
    without_tier: Option<Bytes>,
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
                if matches!(name, "image" | "image[]" | "images") {
                    images.push(field.image_reference()?);
                    continue;
                }
                ensure!(
                    field.filename.is_none(),
                    "Ark image requests do not support the {name} file field"
                );
                let text = std::str::from_utf8(&field.value)?;
                let text = text.trim();
                let value = match name {
                    "n" => json!(text.parse::<u64>().context("expected an integer n")?),
                    "watermark" | "stream" | "layer_decomposition" => {
                        json!(
                            text.parse::<bool>()
                                .with_context(|| format!("expected a boolean {name}"))?
                        )
                    }
                    "seed"
                    | "sequential_image_generation_options"
                    | "optimize_prompt_options"
                    | "tools" => nextjson::from_str(text).map_err(anyhow::Error::msg)?,
                    _ => json!(text),
                };
                ensure!(
                    fields.insert(name.to_owned(), value).is_none(),
                    "duplicate image field {name}"
                );
            }
            fields
        } else {
            let value: Value = nextjson::from_slice(bytes).map_err(anyhow::Error::msg)?;
            match value {
                Value::Object(fields) => fields,
                _ => bail!("expected an image request object"),
            }
        };
        let mut body = Map::new();
        let mut count = 1;
        let mut inline = true;
        let mut size = None;
        let mut resolution = None;
        body.insert("watermark".into(), json!(false));
        for (name, value) in fields {
            if value.is_null() {
                continue;
            }
            match name.as_str() {
                "model" | "user" => {}
                "image" | "image[]" | "images" => {
                    let values = value
                        .as_array()
                        .map(Vec::as_slice)
                        .unwrap_or(std::slice::from_ref(&value));
                    for image in values {
                        images.push(image_reference(image)?.to_owned());
                    }
                }
                "n" => count = value.as_u64().context("expected an integer n")?,
                "response_format" => {
                    inline = match value.as_str() {
                        Some("url") => false,
                        Some("b64_json" | "") => true,
                        _ => bail!("Ark image response_format must be url or b64_json"),
                    };
                }
                "size" | "resolution" => match value {
                    Value::String(value) if name == "size" => size = Some(value),
                    Value::String(value) => resolution = Some(value),
                    _ => bail!("expected a string {name}"),
                },
                "stream" if value.as_bool() == Some(false) => {}
                "background" | "quality" if value.as_str() == Some("auto") => {}
                "watermark" if value.is_bool() => {
                    body.insert(name, value);
                }
                "output_format" if value.as_str() == Some("jpg") => {
                    body.insert(name, json!("jpeg"));
                }
                "prompt"
                | "background"
                | "output_format"
                | "seed"
                | "sequential_image_generation"
                | "sequential_image_generation_options"
                | "optimize_prompt_options"
                | "tools"
                | "layer_decomposition" => {
                    body.insert(name, value);
                }
                _ => bail!("unsupported Ark image field {name}"),
            }
        }
        ensure!(
            (1..=MAX_DRAWS).contains(&count),
            "Ark image count must be between 1 and {MAX_DRAWS}"
        );
        ensure!(images.len() <= MAX_IMAGES, "too many Ark reference images");
        ensure!(!edit || !images.is_empty(), "image edit requires an image");
        ensure!(
            body.get("prompt")
                .and_then(Value::as_str)
                .is_some_and(|prompt| !prompt.trim().is_empty()),
            "image prompt is empty"
        );
        match images.as_slice() {
            [] => {}
            [one] => {
                body.insert("image".into(), json!(one));
            }
            _ => {
                body.insert("image".into(), json!(images));
            }
        }
        body.insert("model".into(), json!(model));
        body.insert(
            "response_format".into(),
            json!(if inline { "b64_json" } else { "url" }),
        );
        let (plain, tiered) = sizes(model, size.as_deref(), resolution.as_deref())?;
        if let Some(size) = plain {
            body.insert("size".into(), json!(size));
        }
        let encode = |body| -> Result<Bytes> {
            Ok(nextjson::to_vec(&Value::Object(body))
                .map_err(anyhow::Error::msg)?
                .into())
        };
        let without_tier = if let Some(size) = tiered.filter(|tiered| Some(*tiered) != plain) {
            let plain = encode(body.clone())?;
            body.insert("size".into(), json!(size));
            Some(plain)
        } else {
            None
        };
        Ok((
            encode(body)?,
            Self {
                count,
                inline,
                without_tier,
            },
        ))
    }

    pub(crate) async fn send(
        &self,
        client: &ForwardClient,
        request: Request<Vec<Bytes>>,
    ) -> Result<Response> {
        let reply = self.send_batch(client, &request).await?;
        if let Some(body) = &self.without_tier
            && matches!(
                reply.status(),
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
            )
        {
            let mut plain = request;
            plain
                .headers_mut()
                .insert(header::CONTENT_LENGTH, body.len().into());
            *plain.body_mut() = vec![body.clone()];
            if let Ok(retry) = self.send_batch(client, &plain).await
                && retry.status().is_success()
            {
                return Ok(retry);
            }
        }
        Ok(reply)
    }

    async fn send_batch(
        &self,
        client: &ForwardClient,
        request: &Request<Vec<Bytes>>,
    ) -> Result<Response> {
        let mut remaining = MAX_REPLY_BYTES;
        let mut first_reply = None;
        let mut first_failure = None;
        let mut result = Vec::new();
        let mut usage = Map::new();
        for _ in 0..self.count {
            let upstream = match client.send(request.clone()).await {
                Ok(reply) => reply,
                Err(error) => {
                    first_failure.get_or_insert(Err(error));
                    continue;
                }
            };
            let (parts, body) = upstream.into_parts();
            if !parts.status.is_success() {
                if self.count == 1 {
                    return Ok(Response::from_parts(parts, body));
                }
                if first_failure.is_none() {
                    // Drain the failed exchange before starting the next one.
                    let body = Body::from(read_body(body, 64 * 1024).await.unwrap_or_default());
                    first_failure = Some(Ok(Response::from_parts(parts, body)));
                }
                continue;
            }
            let (fields, images, length) = match self.read_reply(body, remaining).await {
                Ok(reply) => reply,
                Err(error) => {
                    first_failure.get_or_insert(Err(error));
                    continue;
                }
            };
            remaining -= length;
            result.extend(images);
            if let Some(stats) = fields.get("usage").and_then(Value::as_object) {
                for (name, number) in stats
                    .iter()
                    .filter_map(|(name, value)| Some((name, value.as_u64()?)))
                {
                    let total = usage.get(name).and_then(Value::as_u64).unwrap_or_default();
                    usage.insert(name.to_owned(), json!(total.saturating_add(number)));
                }
            }
            first_reply.get_or_insert((parts, fields));
            if remaining == 0 {
                break;
            }
        }
        let Some((parts, mut fields)) = first_reply else {
            return first_failure.context("Ark images returned no response")?;
        };
        fields.insert("data".into(), json!(result));
        if !usage.is_empty() {
            fields.insert("usage".into(), Value::Object(usage));
        }
        Ok(response(parts, Value::Object(fields)))
    }

    async fn read_reply(&self, body: Body, limit: usize) -> Result<(Map, Vec<Value>, usize)> {
        let bytes = read_body(body, limit).await?;
        let Value::Object(mut fields) = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?
        else {
            bail!("expected an Ark image response object");
        };
        let Some(Value::Array(images)) = fields.remove("data") else {
            bail!("Ark response has no images");
        };
        ensure!(images.len() <= MAX_IMAGES, "Ark returned too many images");
        let field = if self.inline { "b64_json" } else { "url" };
        let mut valid = Vec::new();
        for image in images {
            let Some(raw) = image.get(field).and_then(Value::as_str) else {
                continue;
            };
            let valid_image = if self.inline {
                STANDARD
                    .decode(raw)
                    .ok()
                    .is_some_and(|bytes| image_type(&bytes).is_some())
            } else {
                asset_url(raw).is_some()
            };
            if valid_image {
                valid.push(image);
            }
        }
        ensure!(!valid.is_empty(), "Ark returned no usable images");
        Ok((fields, valid, bytes.len()))
    }
}

async fn read_body(body: Body, limit: usize) -> Result<Bytes> {
    Ok(Limited::new(body, limit)
        .collect()
        .await
        .map_err(anyhow::Error::from_boxed)?
        .to_bytes())
}

fn response(parts: Parts, value: Value) -> Response {
    let mut response = crate::gateway::json_response(StatusCode::OK, &value);
    for name in ["x-request-id", "request-id"] {
        if let Some(value) = parts.headers.get(name) {
            response.headers_mut().insert(name, value.clone());
        }
    }
    response
}

fn sizes<'a>(
    model: &str,
    size: Option<&'a str>,
    resolution: Option<&str>,
) -> Result<(Option<&'a str>, Option<&'static str>)> {
    const RATIOS: [&str; 8] = ["1:1", "4:3", "3:4", "16:9", "9:16", "3:2", "2:3", "21:9"];
    const SEEDREAM_2K: [&str; 8] = [
        "2048x2048",
        "2304x1728",
        "1728x2304",
        "2848x1600",
        "1600x2848",
        "2496x1664",
        "1664x2496",
        "3136x1344",
    ];
    let size = size.unwrap_or_default().trim();
    let resolution = resolution.unwrap_or_default().trim();
    let size_tier = tier(size);
    let asked = match resolution {
        "" => size_tier,
        value if value.eq_ignore_ascii_case("auto") => None,
        value => Some(tier(value).context("Ark resolution must be 1K, 2K, 4K, or auto")?),
    };
    let ratio = if size.contains(':') {
        let ratio = size.replace(' ', "");
        Some(RATIOS.iter().position(|&value| value == ratio).context(
            "unsupported Ark aspect ratio; use WIDTHxHEIGHT or describe the ratio in the prompt",
        )?)
    } else {
        None
    };
    let plain = if let Some(ratio) = ratio {
        Some(SEEDREAM_2K[ratio])
    } else if size.is_empty() || size_tier.is_some() || size.eq_ignore_ascii_case("auto") {
        None
    } else {
        return Ok((Some(size), None));
    };
    let tiered = asked.map(|asked| {
        let model = model
            .rsplit('/')
            .next()
            .unwrap_or(model)
            .to_ascii_lowercase()
            .replace('.', "-");
        let newer = model.contains("seedream-5")
            && !model.contains("seedream-5-0-lite")
            && !model.contains("seedream-5-lite");
        let fitted = match asked {
            "1K" if model.contains("seedream-4-5") || (model.contains("seedream-5") && !newer) => {
                "2K"
            }
            "4K" if newer => "2K",
            _ => asked,
        };
        let pixels = match (fitted, newer) {
            ("1K", true) => [
                "1024x1024",
                "1152x864",
                "864x1152",
                "1424x800",
                "800x1424",
                "1248x832",
                "832x1248",
                "1568x672",
            ],
            ("1K", false) => [
                "1024x1024",
                "1152x864",
                "864x1152",
                "1280x720",
                "720x1280",
                "1248x832",
                "832x1248",
                "1512x648",
            ],
            ("2K", true) => [
                "2048x2048",
                "2368x1776",
                "1776x2368",
                "2816x1584",
                "1584x2816",
                "2496x1664",
                "1664x2496",
                "3136x1344",
            ],
            ("4K", _) => [
                "4096x4096",
                "4704x3520",
                "3520x4704",
                "5504x3040",
                "3040x5504",
                "4992x3328",
                "3328x4992",
                "6240x2656",
            ],
            _ => SEEDREAM_2K,
        };
        ratio.map_or(fitted, |ratio| pixels[ratio])
    });
    Ok((plain, tiered))
}

fn tier(value: &str) -> Option<&'static str> {
    ["1K", "2K", "4K"]
        .into_iter()
        .find(|tier| value.eq_ignore_ascii_case(tier))
}
