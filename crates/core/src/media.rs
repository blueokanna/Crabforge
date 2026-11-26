use anyhow::{Context, Result, ensure};
use axum::{body::Bytes, http::header};
use futures_util::stream;
use std::convert::Infallible;

pub(crate) struct Upload {
    pub model: String,
    pub content_type: String,
    delimiter: String,
    model_part: usize,
    parts: Vec<Bytes>,
}

impl Upload {
    pub async fn parse(content_type: &str, bytes: Bytes) -> Result<Self> {
        let boundary = multer::parse_boundary(content_type)?;
        let delimiter = format!("--{boundary}");
        let mut form = multer::Multipart::new(
            stream::once(async move { Ok::<_, Infallible>(bytes) }),
            boundary,
        );
        let mut parts = Vec::new();
        let mut model = None;
        while let Some(field) = form.next_field().await? {
            ensure!(parts.len() < 2 * 128, "too many multipart fields");
            let is_model = field.name().context("missing multipart field name")? == "model";
            let is_file = field.file_name().is_some();
            // Preserve the sender's quoted parameters and per-part MIME type, including absence.
            let mut head = Vec::new();
            if !parts.is_empty() {
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
            parts.push(head.into());
            let bytes = field.bytes().await?;
            if is_model {
                ensure!(
                    model.is_none() && !is_file && bytes.len() <= 512,
                    "expected one text model field"
                );
                model = Some((parts.len(), std::str::from_utf8(&bytes)?.to_owned()));
            }
            parts.push(bytes);
        }
        let (model_part, model) = model.context("missing multipart model field")?;
        parts.push(format!("\r\n{delimiter}--\r\n").into());
        Ok(Self {
            model,
            content_type: content_type.to_owned(),
            delimiter,
            model_part,
            parts,
        })
    }

    pub fn body(&self, model: &str) -> Result<Vec<Bytes>> {
        ensure!(
            !model.lines().any(|line| line.starts_with(&self.delimiter)),
            "multipart boundary in destination model"
        );
        let mut parts = self.parts.clone();
        parts[self.model_part] = Bytes::copy_from_slice(model.as_bytes());
        Ok(parts)
    }
}
