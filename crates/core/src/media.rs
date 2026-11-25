use anyhow::{Context, Result, ensure};
use axum::body::Bytes;
use futures_util::stream;
use reqwest::multipart::{Form, Part};
use std::convert::Infallible;

struct Field {
    name: String,
    filename: Option<String>,
    mime: Option<String>,
    bytes: Bytes,
}

pub(crate) struct Upload {
    pub model: String,
    fields: Vec<Field>,
}

impl Upload {
    pub async fn parse(content_type: &str, bytes: Bytes) -> Result<Self> {
        let boundary = multer::parse_boundary(content_type)?;
        let mut form = multer::Multipart::new(
            stream::once(async move { Ok::<_, Infallible>(bytes) }),
            boundary,
        );
        let mut fields = vec![];
        let mut model = None;
        while let Some(field) = form.next_field().await? {
            ensure!(fields.len() < 128, "too many multipart fields");
            let name = field
                .name()
                .context("missing multipart field name")?
                .to_owned();
            let filename = field.file_name().map(str::to_owned);
            let mime = field.content_type().map(ToString::to_string);
            let bytes = field.bytes().await?;
            if name == "model" {
                ensure!(
                    model.is_none() && filename.is_none() && bytes.len() <= 512,
                    "expected one text model field"
                );
                model = Some(std::str::from_utf8(&bytes)?.to_owned());
            }
            fields.push(Field {
                name,
                filename,
                mime,
                bytes,
            });
        }
        Ok(Self {
            model: model.context("missing multipart model field")?,
            fields,
        })
    }

    pub fn form(&self, model: &str) -> Result<Form> {
        let mut form = Form::new();
        for field in &self.fields {
            let bytes = if field.name == "model" {
                Bytes::copy_from_slice(model.as_bytes())
            } else {
                field.bytes.clone()
            };
            let length = bytes.len() as u64;
            let mut part = Part::stream_with_length(reqwest::Body::from(bytes), length);
            if let Some(file) = &field.filename {
                part = part.file_name(file.clone());
            }
            if let Some(mime) = &field.mime {
                part = part.mime_str(mime)?;
            }
            form = form.part(field.name.clone(), part);
        }
        Ok(form)
    }
}
