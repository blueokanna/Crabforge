use anyhow::{Context, Result, ensure};
use bytes::Bytes;
use futures_util::stream;
use http::header;
use std::convert::Infallible;

pub(crate) mod dashscope;
pub(crate) mod speech;

struct Field {
    name: String,
    file: bool,
    mime: Option<String>,
    head: Bytes,
    value: Bytes,
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
            let file = field.file_name().is_some();
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
                    model.is_none() && !file && value.len() <= 512,
                    "expected one text model field"
                );
                model = Some(std::str::from_utf8(&value)?.to_owned());
            }
            fields.push(Field {
                name,
                file,
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
