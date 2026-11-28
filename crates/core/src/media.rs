use anyhow::{Context, Result, ensure};
use axum_core::response::Response;
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use futures_util::stream;
use http::header;
use std::convert::Infallible;
use url::Url;

pub(crate) mod ark;
pub(crate) mod dashscope;
pub(crate) mod minimax;
pub(crate) mod modelscope;
pub(crate) mod speech;

struct Field {
    name: String,
    filename: Option<String>,
    mime: Option<String>,
    head: Bytes,
    value: Bytes,
}

impl Field {
    fn image_reference(&self) -> Result<String> {
        if let Some(mime) = image_type(&self.value) {
            return Ok(format!(
                "data:{mime};base64,{}",
                STANDARD.encode(&self.value)
            ));
        }
        ensure!(self.filename.is_none(), "unrecognized image file");
        Ok(image_source(std::str::from_utf8(&self.value)?.trim())?.to_owned())
    }
}

fn image_reference(value: &nextjson::Value) -> Result<&str> {
    let link = value.get("image_url").unwrap_or(value);
    let raw = link
        .as_str()
        .or_else(|| link.get("url").and_then(nextjson::Value::as_str));
    image_source(raw.context("expected an image URL or data URL")?)
}

pub(crate) enum Reply {
    GeminiSpeech(speech::AudioFormat),
    DashScopeImage { inline: bool },
    DashScopeAudio(dashscope::audio::Reply),
    MiniMaxImage(minimax::ImageFormat),
}

impl Reply {
    pub(crate) async fn convert(
        self,
        upstream: Response,
        source: &Url,
        client: &crate::upstream::ForwardClient,
    ) -> Result<Response> {
        match self {
            Self::GeminiSpeech(format) => format.reply(upstream).await,
            Self::DashScopeImage { inline } => {
                dashscope::image_reply(upstream, inline, source, client).await
            }
            Self::DashScopeAudio(format) => format.convert(upstream, source, client).await,
            Self::MiniMaxImage(format) => format.reply(upstream).await,
        }
    }
}

fn image_source(raw: &str) -> Result<&str> {
    if let Some((mime, content)) = raw
        .strip_prefix("data:")
        .and_then(|raw| raw.split_once(";base64,"))
    {
        let bytes = STANDARD.decode(content).context("invalid base64 image")?;
        ensure!(
            image_type(&bytes) == Some(mime),
            "invalid data URL image type"
        );
    } else {
        asset_url(raw).context("invalid image URL")?;
    }
    Ok(raw)
}

fn asset_url(raw: &str) -> Option<Url> {
    let url = Url::parse(raw).ok()?;
    (matches!(url.scheme(), "http" | "https")
        && url.has_host()
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none())
    .then_some(url)
}

fn image_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        Some("image/jpeg")
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        Some("image/webp")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.starts_with(b"BM") {
        Some("image/bmp")
    } else {
        None
    }
}

pub(crate) struct Upload {
    pub model: String,
    pub content_type: String,
    delimiter: String,
    fields: Vec<Field>,
}

impl Upload {
    pub async fn parse(content_type: &str, bytes: Bytes) -> Result<Self> {
        let boundary = multer::parse_boundary(content_type)?;
        let delimiter = format!("--{boundary}");
        let mut form = multer::Multipart::new(
            stream::once(async move { Ok::<_, Infallible>(bytes) }),
            boundary,
        );
        let mut fields = Vec::new();
        let mut model = None;
        while let Some(field) = form.next_field().await? {
            ensure!(fields.len() < 128, "too many multipart fields");
            let name = field
                .name()
                .context("missing multipart field name")?
                .to_owned();
            let filename = field.file_name().map(str::to_owned);
            let mime = field.content_type().map(ToString::to_string);
            // Preserve the sender's quoted parameters and per-part MIME type, including absence.
            let mut head = Vec::new();
            if !fields.is_empty() {
                head.extend_from_slice(b"\r\n");
            }
            head.extend_from_slice(delimiter.as_bytes());
            head.extend_from_slice(b"\r\nContent-Disposition: ");
            head.extend_from_slice(field.headers()[header::CONTENT_DISPOSITION].as_bytes());
            if let Some(mime) = field.headers().get(header::CONTENT_TYPE) {
                head.extend_from_slice(b"\r\nContent-Type: ");
                head.extend_from_slice(mime.as_bytes());
            }
            head.extend_from_slice(b"\r\n\r\n");
            let value = field.bytes().await?;
            if name == "model" {
                ensure!(
                    model.is_none() && filename.is_none() && value.len() <= 512,
                    "expected one text model field"
                );
                model = Some(std::str::from_utf8(&value)?.to_owned());
            }
            fields.push(Field {
                name,
                filename,
                mime,
                head: head.into(),
                value,
            });
        }
        Ok(Self {
            model: model.context("missing multipart model field")?,
            content_type: content_type.to_owned(),
            delimiter,
            fields,
        })
    }

    pub fn body(&self, model: &str) -> Result<Vec<Bytes>> {
        ensure!(
            !model.lines().any(|line| line.starts_with(&self.delimiter)),
            "multipart boundary in destination model"
        );
        let mut parts = Vec::with_capacity(2 * self.fields.len() + 1);
        for field in &self.fields {
            parts.push(field.head.clone());
            parts.push(if field.name == "model" {
                Bytes::copy_from_slice(model.as_bytes())
            } else {
                field.value.clone()
            });
        }
        parts.push(format!("\r\n{}--\r\n", self.delimiter).into());
        Ok(parts)
    }
}
