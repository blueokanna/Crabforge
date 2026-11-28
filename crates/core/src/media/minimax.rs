use super::{Upload, asset_url, image_source, image_type};
use crate::config::{Endpoints, endpoint};
use anyhow::{Context, Result, bail, ensure};
use axum_core::response::Response;
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use http::StatusCode;
use http_body_util::{BodyExt, Limited};
use nextjson::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

pub(crate) const IMAGE: &str = "v1/image_generation";
const MODELS: [&str; 2] = ["image-01", "image-01-live"];
const MAX_IMAGES: usize = 9;
const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;
const MAX_REPLY_BYTES: usize = MAX_IMAGE_BYTES.div_ceil(3) * 4 + 64 * 1024;
const NATIVE_OPTIONS: [&str; 4] = ["seed", "prompt_optimizer", "aigc_watermark", "style"];

pub(crate) fn destination(bases: &Endpoints, model: &str) -> Option<Url> {
    if !MODELS.iter().any(|name| model.eq_ignore_ascii_case(name)) {
        return None;
    }
    let mut url = base(bases)?;
    url.set_path(IMAGE);
    Some(url)
}

fn base(bases: &Endpoints) -> Option<Url> {
    [&bases.chat, &bases.anthropic, &bases.responses]
        .into_iter()
        .filter_map(|base| endpoint(base).ok())
        .find(|url| matches!(url.host_str(), Some("api.minimax.io" | "api.minimaxi.com")))
}

pub(crate) fn catalog_models(bases: &Endpoints) -> Option<&'static [&'static str]> {
    (bases.models_url.is_empty() && base(bases).is_some()).then_some(&MODELS)
}

#[derive(Clone, Copy)]
pub(crate) enum ImageFormat {
    Url,
    Base64,
}

pub(crate) fn request(
    model: &str,
    bytes: &[u8],
    upload: Option<&Upload>,
    edit: bool,
) -> Result<(Bytes, ImageFormat)> {
    let mut images = Vec::new();
    let input = if let Some(upload) = upload {
        let mut object = nextjson::Map::new();
        for field in &upload.fields {
            let name = field.name.as_str();
            if matches!(name, "image" | "image[]" | "images") {
                let source = if let Some(mime) = image_type(&field.value) {
                    format!("data:{mime};base64,{}", STANDARD.encode(&field.value))
                } else {
                    ensure!(field.filename.is_none(), "unrecognized image file");
                    image_source(std::str::from_utf8(&field.value)?.trim())?.to_owned()
                };
                images.push(source);
                continue;
            }
            ensure!(field.filename.is_none(), "unexpected file field {name}");
            let text = std::str::from_utf8(&field.value)?;
            let value = match name {
                "n" | "seed" => json!(text.trim().parse::<i64>().context("expected an integer")?),
                "watermark" | "stream" | "prompt_optimizer" | "aigc_watermark" => {
                    json!(text.trim().parse::<bool>().context("expected a boolean")?)
                }
                "style" => nextjson::from_str(text).map_err(anyhow::Error::msg)?,
                _ => json!(text),
            };
            ensure!(
                object.insert(name.to_owned(), value).is_none(),
                "duplicate image field {name}"
            );
        }
        Value::Object(object)
    } else {
        nextjson::from_slice(bytes).map_err(anyhow::Error::msg)?
    };
    let fields = input
        .as_object()
        .context("expected an image request object")?;
    for (name, value) in fields.iter().filter(|(_, value)| !value.is_null()) {
        match name {
            "model" | "prompt" | "size" | "n" | "response_format" | "user" | "watermark" => {}
            "image" | "image[]" | "images" => {
                let list = value
                    .as_array()
                    .map(Vec::as_slice)
                    .unwrap_or_else(|| std::slice::from_ref(value));
                for image in list {
                    let link = image.get("image_url").unwrap_or(image);
                    let raw = link
                        .as_str()
                        .or_else(|| link.get("url").and_then(Value::as_str));
                    images.push(
                        image_source(raw.context("expected an image URL or data URL")?)?.to_owned(),
                    );
                }
            }
            "background" | "quality" if value.as_str() == Some("auto") => {}
            "stream" if value.as_bool() == Some(false) => {}
            other if NATIVE_OPTIONS.contains(&other) => {}
            _ => bail!("unsupported MiniMax image field {name}"),
        }
    }
    ensure!(images.len() <= 16, "too many reference images");
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
    let (format, wire_format) = match text("response_format")?
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "" | "url" => (ImageFormat::Url, "url"),
        "b64_json" => (ImageFormat::Base64, "base64"),
        _ => bail!("MiniMax image response_format must be url or b64_json"),
    };
    let mut body = json!({"model":model,"prompt":prompt,"n":count,"response_format":wire_format});
    let body_fields = body.as_object_mut().expect("image request object");
    set_size(body_fields, model, text("size")?)?;
    if !images.is_empty() {
        let references: Vec<_> = images
            .iter()
            .map(|image| json!({"type":"character","image_file":image}))
            .collect();
        body_fields.insert("subject_reference".into(), json!(references));
    }
    for name in NATIVE_OPTIONS {
        if let Some(value) = fields.get(name).filter(|value| !value.is_null()) {
            body_fields.insert(name.into(), value.clone());
        }
    }
    if let Some(value) = fields.get("watermark").filter(|value| !value.is_null()) {
        let watermark = value.as_bool().context("expected a boolean watermark")?;
        ensure!(
            body_fields
                .insert("aigc_watermark".into(), json!(watermark))
                .is_none(),
            "use one watermark option"
        );
    }
    Ok((
        nextjson::to_vec(&body).map_err(anyhow::Error::msg)?.into(),
        format,
    ))
}

fn set_size(fields: &mut nextjson::Map, model: &str, size: &str) -> Result<()> {
    const SIZES: [(&str, &str, f64); 8] = [
        ("1:1", "1024x1024", 1.0),
        ("16:9", "1280x720", 16.0 / 9.0),
        ("4:3", "1152x864", 4.0 / 3.0),
        ("3:2", "1248x832", 3.0 / 2.0),
        ("2:3", "832x1248", 2.0 / 3.0),
        ("3:4", "864x1152", 3.0 / 4.0),
        ("9:16", "720x1280", 9.0 / 16.0),
        ("21:9", "1344x576", 21.0 / 9.0),
    ];
    let standard = model.eq_ignore_ascii_case(MODELS[0]);
    let size = size.trim().to_ascii_lowercase();
    if matches!(size.as_str(), "" | "auto") {
        return Ok(());
    }
    let sizes = if standard { &SIZES[..] } else { &SIZES[..7] };
    if let Some((ratio, _, _)) = sizes
        .iter()
        .find(|(ratio, pixels, _)| size == *ratio || size == *pixels)
    {
        fields.insert("aspect_ratio".into(), json!(ratio));
        return Ok(());
    }
    let (width, height) = size
        .split_once('x')
        .context("unsupported MiniMax image size")?;
    let width: u32 = width.trim().parse().context("invalid image width")?;
    let height: u32 = height.trim().parse().context("invalid image height")?;
    ensure!(width > 0 && height > 0, "image dimensions must be positive");
    if standard
        && [width, height]
            .into_iter()
            .all(|value| (512..=2048).contains(&value) && value.is_multiple_of(8))
    {
        fields.insert("width".into(), json!(width));
        fields.insert("height".into(), json!(height));
        return Ok(());
    }
    let requested = f64::from(width) / f64::from(height);
    let distance = |ratio: f64| (requested.ln() - ratio.ln()).abs();
    let (ratio, _, _) = sizes
        .iter()
        .min_by(|a, b| distance(a.2).total_cmp(&distance(b.2)))
        .expect("MiniMax has supported image sizes");
    fields.insert("aspect_ratio".into(), json!(ratio));
    Ok(())
}

impl ImageFormat {
    pub(super) async fn reply(self, upstream: Response) -> Result<Response> {
        let (parts, body) = upstream.into_parts();
        let bytes = Limited::new(body, MAX_REPLY_BYTES)
            .collect()
            .await
            .map_err(|_| anyhow::anyhow!("could not read the bounded MiniMax image response"))?
            .to_bytes();
        let reply: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
        let code = reply
            .pointer("/base_resp/status_code")
            .and_then(Value::as_i64)
            .context("MiniMax image response has no status code")?;
        let mut response = if code == 0 {
            let mut remaining = MAX_IMAGE_BYTES;
            let field = match self {
                Self::Url => "/data/image_urls",
                Self::Base64 => "/data/image_base64",
            };
            let images = reply
                .pointer(field)
                .and_then(Value::as_array)
                .filter(|images| (1..=MAX_IMAGES).contains(&images.len()))
                .context("MiniMax returned no images or too many images")?;
            let mut data = Vec::with_capacity(images.len());
            for image in images {
                let raw = image.as_str().context("invalid MiniMax image")?;
                data.push(match self {
                    Self::Url => {
                        asset_url(raw).context("MiniMax returned an invalid image URL")?;
                        json!({"url":raw})
                    }
                    Self::Base64 => {
                        let bytes = STANDARD
                            .decode(raw)
                            .context("invalid MiniMax image encoding")?;
                        ensure!(
                            bytes.len() <= remaining && image_type(&bytes).is_some(),
                            "invalid or oversized MiniMax image"
                        );
                        remaining -= bytes.len();
                        json!({"b64_json":raw})
                    }
                });
            }
            let created = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            crate::gateway::json_response(parts.status, &json!({"created":created,"data":data}))
        } else {
            let (status, message) = match code {
                1002 => (
                    StatusCode::TOO_MANY_REQUESTS,
                    "MiniMax image rate limit reached",
                ),
                1004 | 2049 => (StatusCode::UNAUTHORIZED, "MiniMax rejected the API key"),
                1008 => (
                    StatusCode::PAYMENT_REQUIRED,
                    "MiniMax account balance is insufficient",
                ),
                1026 | 2013 => (
                    StatusCode::BAD_REQUEST,
                    "MiniMax rejected the image request",
                ),
                _ => bail!("MiniMax image request failed with status {code}"),
            };
            crate::gateway::failure(status, message)
        };
        for name in ["x-request-id", "request-id", "retry-after"] {
            if let Some(value) = parts.headers.get(name) {
                response.headers_mut().insert(name, value.clone());
            }
        }
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_core::body::Body;
    use axum_core::response::IntoResponse;

    #[tokio::test]
    async fn translates_multipart_references_and_base64_reply() -> Result<()> {
        let png = b"\x89PNG\r\n\x1a\nfixture";
        let mut multipart = b"--test\r\nContent-Disposition: form-data; name=\"model\"\r\n\r\ngroup/draw\r\n--test\r\nContent-Disposition: form-data; name=\"prompt\"\r\n\r\nA portrait\r\n--test\r\nContent-Disposition: form-data; name=\"size\"\r\n\r\n1200x800\r\n--test\r\nContent-Disposition: form-data; name=\"response_format\"\r\n\r\nb64_json\r\n--test\r\nContent-Disposition: form-data; name=\"image[]\"; filename=\"source.png\"\r\nContent-Type: image/png\r\n\r\n".to_vec();
        multipart.extend_from_slice(png);
        multipart.extend_from_slice(b"\r\n--test\r\nContent-Disposition: form-data; name=\"image[]\"\r\n\r\nhttps://images.example/ref.png\r\n--test--\r\n");
        let upload = Upload::parse("multipart/form-data; boundary=test", multipart.into()).await?;
        let (body, format) = request(MODELS[0], &[], Some(&upload), true)?;
        let translated: Value = nextjson::from_slice(&body).map_err(anyhow::Error::msg)?;
        let encoded = STANDARD.encode(png);
        assert_eq!(translated["model"], json!(MODELS[0]));
        assert_eq!(translated["width"], json!(1200));
        assert_eq!(translated["height"], json!(800));
        assert_eq!(translated["response_format"], json!("base64"));
        assert_eq!(
            translated["subject_reference"],
            json!([
                {"type":"character","image_file":format!("data:image/png;base64,{encoded}")},
                {"type":"character","image_file":"https://images.example/ref.png"}
            ])
        );
        let upstream = (
            [("x-request-id", "fixture")],
            json!({"base_resp":{"status_code":0},"data":{"image_base64":[encoded]}}).to_string(),
        )
            .into_response();
        let reply = format.reply(upstream).await?;
        assert_eq!(reply.headers()["x-request-id"], "fixture");
        let bytes = reply.into_body().collect().await?.to_bytes();
        let body: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
        assert_eq!(body.pointer("/data/0/b64_json"), Some(&json!(encoded)));

        let malformed = Response::new(Body::from(
            json!({"base_resp":{"status_code":0},"data":{"image_base64":["bad"]}}).to_string(),
        ));
        assert!(format.reply(malformed).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn translates_url_reply_and_vendor_rate_limit() -> Result<()> {
        let (body, format) = request(MODELS[1], br#"{"model":"group/draw","prompt":"A portrait","size":"1600x1000","n":2,"style":{"style_type":"\u6f2b\u753b"}}"#, None, false)?;
        let translated: Value = nextjson::from_slice(&body).map_err(anyhow::Error::msg)?;
        assert_eq!(translated["aspect_ratio"], json!("3:2"));
        assert_eq!(
            translated.pointer("/style/style_type"),
            Some(&json!("漫画"))
        );
        assert_eq!(translated["n"], json!(2));
        assert_eq!(translated["response_format"], json!("url"));
        assert!(
            request(
                MODELS[1],
                br#"{"prompt":"x","mask":"unsupported"}"#,
                None,
                false
            )
            .is_err()
        );
        assert!(request(MODELS[1], br#"{"prompt":"x"}"#, None, true).is_err());

        let url = "https://images.example/result.png";
        let upstream = json!({"base_resp":{"status_code":0},"data":{"image_urls":[url]}})
            .to_string()
            .into_response();
        let result = format.reply(upstream).await?;
        let bytes = result.into_body().collect().await?.to_bytes();
        let result: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
        assert_eq!(result.pointer("/data/0/url"), Some(&json!(url)));
        let busy = (
            [("retry-after", "2")],
            "{\"base_resp\":{\"status_code\":1002}}",
        )
            .into_response();
        let busy = format.reply(busy).await?;
        assert_eq!(busy.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(busy.headers()["retry-after"], "2");
        Ok(())
    }
}
