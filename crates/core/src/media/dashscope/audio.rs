use super::{MAX_ASSET_BYTES, asset_url, download};
use crate::{media::Upload, upstream::ForwardClient};
use anyhow::{Context, Result, bail, ensure};
use axum_core::{body::Body, response::Response};
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use http::header;
use http_body_util::{BodyExt, Limited};
use nextjson::{Value, json};
use std::collections::BTreeMap;
use url::Url;

#[derive(Clone, Copy)]
pub(crate) enum SpeechFormat {
    Mp3,
    Wav,
    Pcm,
    Opus,
}

impl SpeechFormat {
    fn wire(self) -> (&'static str, &'static str) {
        match self {
            Self::Mp3 => ("mp3", "audio/mpeg"),
            Self::Wav => ("wav", "audio/wav"),
            Self::Pcm => ("pcm", "audio/pcm"),
            Self::Opus => ("opus", "audio/ogg"),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum TranscriptFormat {
    Json,
    Text,
    Verbose,
}

pub(crate) enum Reply {
    Speech(SpeechFormat),
    Transcript {
        format: TranscriptFormat,
        language: String,
    },
}

pub(crate) fn speech(model: &str, bytes: &[u8]) -> Result<(Bytes, Reply)> {
    let request: Value = nextjson::from_slice(bytes).map_err(anyhow::Error::msg)?;
    let fields = request
        .as_object()
        .context("expected a speech request object")?;
    for (field, value) in fields.iter().filter(|(_, value)| !value.is_null()) {
        ensure!(
            matches!(
                field,
                "model"
                    | "input"
                    | "voice"
                    | "instructions"
                    | "response_format"
                    | "stream_format"
                    | "speed"
            ),
            "unsupported DashScope speech field {field}"
        );
        if field == "speed" {
            ensure!(value.as_f64().is_some(), "expected a numeric speed");
        }
    }
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
    ensure!(
        matches!(text("stream_format")?, "" | "audio"),
        "DashScope speech requires a single audio response"
    );
    let format = match text("response_format")?
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "" | "mp3" => SpeechFormat::Mp3,
        "wav" => SpeechFormat::Wav,
        "pcm" => SpeechFormat::Pcm,
        "opus" => SpeechFormat::Opus,
        _ => bail!("DashScope speech outputs MP3, WAV, PCM or Opus"),
    };
    let input = text("input")?;
    ensure!(!input.trim().is_empty(), "speech input is empty");
    let voice = text("voice")?.trim();
    let voice = if voice.is_empty()
        || crate::media::speech::openai_voice(&voice.to_ascii_lowercase()).is_some()
    {
        if model.to_ascii_lowercase().contains("flash") {
            "longanhuan_v3.6"
        } else {
            "longanlingxin"
        }
    } else {
        voice
    };
    let mut native_input = json!({"text":input,"voice":voice,"format":format.wire().0});
    let settings = native_input
        .as_object_mut()
        .expect("native speech input object");
    let instructions = text("instructions")?.trim();
    if !instructions.is_empty() {
        settings.insert("instruction".into(), json!(instructions));
    }
    if let Some(speed) = fields
        .get("speed")
        .and_then(Value::as_f64)
        .filter(|speed| *speed > 0.0)
    {
        settings.insert("rate".into(), json!(speed.clamp(0.5, 2.0)));
    }
    let body = nextjson::to_vec(&json!({"model":model,"input":native_input}))
        .map_err(anyhow::Error::msg)?;
    Ok((body.into(), Reply::Speech(format)))
}

pub(crate) fn transcription(
    model: &str,
    upload: Option<&Upload>,
    translate: bool,
) -> Result<(Bytes, Reply)> {
    ensure!(
        !translate,
        "DashScope ASR supports /v1/audio/transcriptions"
    );
    let upload = upload.context("DashScope transcription requires a multipart audio file")?;
    let mut file = None;
    let mut fields = BTreeMap::new();
    for field in &upload.fields {
        match field.name.as_str() {
            "file" => {
                ensure!(
                    file.is_none() && !field.value.is_empty(),
                    "expected one nonempty audio file"
                );
                file = Some(field);
            }
            "model" | "response_format" | "language" | "prompt" | "stream" => {
                ensure!(
                    field.filename.is_none(),
                    "expected a text {} field",
                    field.name
                );
                let value = std::str::from_utf8(&field.value)?.trim();
                ensure!(
                    fields.insert(field.name.as_str(), value).is_none(),
                    "duplicate audio field {}",
                    field.name
                );
            }
            name => bail!("unsupported DashScope transcription field {name}"),
        }
    }
    let file = file.context("missing audio file")?;
    ensure!(
        matches!(fields.get("stream"), None | Some(&"false")),
        "DashScope transcription requires a single response"
    );
    let format = match fields
        .get("response_format")
        .copied()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "" | "json" => TranscriptFormat::Json,
        "text" => TranscriptFormat::Text,
        "verbose_json" => TranscriptFormat::Verbose,
        _ => bail!("DashScope transcription outputs json, text or verbose_json"),
    };
    let language = fields.get("language").copied().unwrap_or_default();
    let declared = file
        .mime
        .as_deref()
        .and_then(|mime| mime.split(';').next())
        .and_then(|mime| mime.strip_prefix("audio/"));
    let kind = file
        .filename
        .as_deref()
        .and_then(|name| name.rsplit_once('.'))
        .map(|(_, extension)| extension)
        .filter(|extension| !extension.is_empty())
        .or(declared)
        .unwrap_or("wav")
        .to_ascii_lowercase();
    let kind = match kind.trim_start_matches("x-") {
        "wav" | "wave" => "wav",
        "mp3" | "mpeg" | "mpga" => "mp3",
        "ogg" | "oga" | "opus" => "opus",
        "mp4" | "m4a" => "m4a",
        kind => kind,
    };
    ensure!(
        !kind.is_empty()
            && kind.len() <= 16
            && kind.bytes().all(|byte| byte.is_ascii_alphanumeric()),
        "invalid audio format"
    );
    let mime = match kind {
        "mp3" => "audio/mpeg".into(),
        "opus" => "audio/ogg".into(),
        "m4a" => "audio/mp4".into(),
        kind => format!("audio/{kind}"),
    };
    let mut parameters = json!({"format":kind});
    if !language.is_empty() {
        parameters
            .as_object_mut()
            .expect("native transcription parameters object")
            .insert("language_hints".into(), json!([language]));
    }
    let data = format!("data:{mime};base64,{}", STANDARD.encode(&file.value));
    let body = nextjson::to_vec(&json!({
        "model":model,
        "input":{"messages":[{"role":"user","content":[{"type":"input_audio","input_audio":{"data":data}}]}]},
        "parameters":parameters
    })).map_err(anyhow::Error::msg)?;
    Ok((
        body.into(),
        Reply::Transcript {
            format,
            language: language.into(),
        },
    ))
}

impl Reply {
    pub(crate) async fn convert(
        self,
        upstream: Response,
        source: &Url,
        client: &ForwardClient,
    ) -> Result<Response> {
        let (parts, body) = upstream.into_parts();
        let limit = match self {
            Self::Speech(_) => MAX_ASSET_BYTES.div_ceil(3) * 4 + 1024 * 1024,
            Self::Transcript { .. } => 1024 * 1024,
        };
        let bytes = Limited::new(body, limit)
            .collect()
            .await
            .map_err(|_| {
                anyhow::anyhow!("could not read DashScope audio result within its size limit")
            })?
            .to_bytes();
        let reply: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
        let (mime, body) = match self {
            Self::Speech(format) => {
                let data = reply
                    .pointer("/output/audio/data")
                    .and_then(Value::as_str)
                    .filter(|data| !data.is_empty());
                let audio = if let Some(data) = data {
                    Bytes::from(
                        STANDARD
                            .decode(data)
                            .context("invalid DashScope base64 audio")?,
                    )
                } else {
                    let url = reply
                        .pointer("/output/audio/url")
                        .and_then(Value::as_str)
                        .and_then(asset_url)
                        .context("DashScope returned no valid speech URL or audio")?;
                    download(client, source, &url, MAX_ASSET_BYTES).await?
                };
                ensure!(
                    !audio.is_empty() && audio.len() <= MAX_ASSET_BYTES,
                    "DashScope speech requires 1 byte to 16 MiB of audio"
                );
                (format.wire().1, Body::from(audio))
            }
            Self::Transcript { format, language } => transcript(&reply, format, &language)?,
        };
        let mut response = Response::builder()
            .status(parts.status)
            .header(header::CONTENT_TYPE, mime);
        for name in ["x-request-id", "request-id"] {
            if let Some(value) = parts.headers.get(name) {
                response = response.header(name, value);
            }
        }
        Ok(response.body(body)?)
    }
}

fn transcript(
    reply: &Value,
    format: TranscriptFormat,
    language: &str,
) -> Result<(&'static str, Body)> {
    let choice = reply
        .pointer("/output/choices/0/message/content")
        .and_then(|content| {
            content.as_str().map(str::to_owned).or_else(|| {
                let mut parts = content
                    .as_array()?
                    .iter()
                    .filter_map(|part| part.get("text")?.as_str())
                    .peekable();
                parts.peek()?;
                Some(parts.collect::<String>())
            })
        });
    let candidates = [
        reply.pointer("/output/text").and_then(Value::as_str),
        reply
            .pointer("/output/sentence/text")
            .and_then(Value::as_str),
        choice.as_deref(),
    ];
    let text = candidates
        .into_iter()
        .flatten()
        .find(|text| !text.is_empty())
        .or_else(|| candidates.into_iter().flatten().next())
        .context("DashScope returned no transcript")?;
    let json = match format {
        TranscriptFormat::Text => {
            return Ok(("text/plain; charset=utf-8", Body::from(text.to_owned())));
        }
        TranscriptFormat::Json => json!({"text":text}),
        TranscriptFormat::Verbose => {
            let duration = reply
                .pointer("/usage/duration")
                .and_then(Value::as_f64)
                .unwrap_or_default();
            json!({"task":"transcribe","language":language,"duration":duration,"text":text,"segments":[]})
        }
    };
    Ok(("application/json", Body::from(json.to_string())))
}
