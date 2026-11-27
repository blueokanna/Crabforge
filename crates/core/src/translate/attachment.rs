use super::{Format, field, keys, text};
use anyhow::{Result, bail, ensure};
use nextjson::{Value, json};

#[derive(Debug)]
pub(super) enum Attachment {
    Image {
        source: ImageSource,
        detail: Option<String>,
    },
    Audio {
        mime: String,
        data: String,
    },
}

#[derive(Debug)]
pub(super) enum ImageSource {
    Inline { mime: String, data: String },
    Url(String),
}

impl ImageSource {
    fn inline(mime: &str, data: &str) -> Result<Self> {
        ensure!(
            mime.starts_with("image/") && mime.len() > "image/".len() && !data.is_empty(),
            "expected a base64 image with an image MIME type"
        );
        Ok(Self::Inline {
            mime: mime.into(),
            data: data.into(),
        })
    }

    fn url(value: &str) -> Result<Self> {
        if let Some((mime, data)) = value
            .strip_prefix("data:")
            .and_then(|value| value.split_once(";base64,"))
        {
            return Self::inline(mime, data);
        }
        let url = url::Url::parse(value)?;
        ensure!(
            matches!(url.scheme(), "http" | "https") && url.has_host(),
            "expected an HTTP image URL or base64 image"
        );
        Ok(Self::Url(value.into()))
    }
}

impl Attachment {
    pub(super) fn read(format: Format, part: &Value) -> Result<Option<Self>> {
        let kind = field(part, "type").as_str();
        let (source, detail) = match (format, kind) {
            (Format::Chat, Some("image_url")) => {
                keys(part, &["type", "image_url"])?;
                let image = field(part, "image_url");
                keys(image, &["url", "detail"])?;
                (ImageSource::url(text(image, "url")?)?, image.get("detail"))
            }
            (Format::Responses, Some("input_image")) => {
                keys(part, &["type", "image_url", "detail", "file_id"])?;
                ensure!(
                    field(part, "file_id").is_null(),
                    "image file IDs require a native endpoint"
                );
                (
                    ImageSource::url(text(part, "image_url")?)?,
                    part.get("detail"),
                )
            }
            (Format::Messages, Some("image")) => {
                keys(part, &["type", "source"])?;
                let image = field(part, "source");
                let source = match text(image, "type")? {
                    "base64" => {
                        keys(image, &["type", "media_type", "data"])?;
                        ImageSource::inline(text(image, "media_type")?, text(image, "data")?)?
                    }
                    "url" => {
                        keys(image, &["type", "url"])?;
                        ImageSource::url(text(image, "url")?)?
                    }
                    _ => bail!("image file IDs require a native endpoint"),
                };
                (source, None)
            }
            (Format::Chat, Some("input_audio")) => {
                keys(part, &["type", "input_audio"])?;
                let audio = field(part, "input_audio");
                keys(audio, &["format", "data"])?;
                let format = text(audio, "format")?.to_ascii_lowercase();
                let mime = match format.as_str() {
                    "wav" | "mp3" | "aiff" | "aac" | "flac" | "ogg" => format!("audio/{format}"),
                    "mpeg" => "audio/mp3".into(),
                    _ => bail!("unsupported input audio format {format}"),
                };
                return Ok(Some(Self::audio(mime, text(audio, "data")?)?));
            }
            (Format::Gemini, _) => {
                let Some((name, inline)) = ["inlineData", "inline_data"]
                    .into_iter()
                    .find_map(|name| part.get(name).map(|value| (name, value)))
                else {
                    return Ok(None);
                };
                keys(part, &[name])?;
                keys(inline, &["mimeType", "mime_type", "data"])?;
                ensure!(
                    inline.get("mimeType").is_none() || inline.get("mime_type").is_none(),
                    "multiple MIME types on inline data"
                );
                let mime = text(
                    inline,
                    if inline.get("mimeType").is_some() {
                        "mimeType"
                    } else {
                        "mime_type"
                    },
                )?;
                let data = text(inline, "data")?;
                if mime.starts_with("audio/") {
                    return Ok(Some(Self::audio(mime.into(), data)?));
                }
                (ImageSource::inline(mime, data)?, None)
            }
            _ => return Ok(None),
        };
        let detail = match detail.filter(|value| !value.is_null()) {
            None => None,
            Some(value) => {
                let detail = value
                    .as_str()
                    .filter(|value| matches!(*value, "auto" | "low" | "high" | "original"));
                Some(
                    detail
                        .ok_or_else(|| anyhow::anyhow!("unsupported image detail"))?
                        .into(),
                )
            }
        };
        Ok(Some(Self::Image { source, detail }))
    }

    fn audio(mime: String, data: &str) -> Result<Self> {
        ensure!(!data.is_empty(), "expected base64 audio data");
        Ok(Self::Audio {
            mime,
            data: data.into(),
        })
    }

    pub(super) fn write(&self, format: Format) -> Result<Value> {
        let (source, detail) = match self {
            Self::Image { source, detail } => (source, detail.as_deref()),
            Self::Audio { mime, data } => {
                return Ok(match format {
                    Format::Gemini => json!({"inlineData":{"mimeType":mime,"data":data}}),
                    Format::Chat => {
                        let format = match mime.as_str() {
                            "audio/wav" => "wav",
                            "audio/mp3" | "audio/mpeg" => "mp3",
                            _ => bail!("Chat input audio requires WAV or MP3"),
                        };
                        json!({"type":"input_audio","input_audio":{"format":format,"data":data}})
                    }
                    _ => bail!("this endpoint cannot represent input audio"),
                });
            }
        };
        if matches!(format, Format::Chat | Format::Responses) {
            let url = match source {
                ImageSource::Inline { mime, data } => format!("data:{mime};base64,{data}"),
                ImageSource::Url(url) => url.clone(),
            };
            let detail = detail.unwrap_or("auto");
            return Ok(match format {
                Format::Chat => json!({"type":"image_url","image_url":{"url":url,"detail":detail}}),
                Format::Responses => json!({"type":"input_image","image_url":url,"detail":detail}),
                _ => unreachable!(),
            });
        }
        ensure!(
            detail.is_none_or(|detail| detail == "auto"),
            "this endpoint cannot preserve explicit image detail"
        );
        Ok(match (format, source) {
            (Format::Gemini, ImageSource::Inline { mime, data }) => {
                json!({"inlineData":{"mimeType":mime,"data":data}})
            }
            (Format::Gemini, ImageSource::Url(_)) => {
                bail!("Gemini translation requires base64 image data")
            }
            (Format::Messages, ImageSource::Inline { mime, data }) => {
                json!({"type":"image","source":{"type":"base64","media_type":mime,"data":data}})
            }
            (Format::Messages, ImageSource::Url(url)) => {
                json!({"type":"image","source":{"type":"url","url":url}})
            }
            _ => unreachable!(),
        })
    }
}
