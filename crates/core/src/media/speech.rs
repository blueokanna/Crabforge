use anyhow::{Context, Result, bail, ensure};
use axum_core::response::{IntoResponse, Response};
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use futures_util::StreamExt;
use http::header;
use nextjson::{Value, json};

pub(crate) fn gemini_model(model: &str) -> bool {
    let model = model.strip_prefix("models/").unwrap_or(model);
    model.starts_with("gemini-") && model.split('-').any(|part| part == "tts")
}

#[derive(Clone, Copy)]
pub(crate) enum AudioFormat {
    Wav,
    Pcm,
}

pub(crate) struct Speech {
    pub body: Bytes,
    pub format: AudioFormat,
}

impl Speech {
    pub(crate) fn gemini(bytes: &[u8]) -> Result<Self> {
        let request: Value = nextjson::from_slice(bytes).map_err(anyhow::Error::msg)?;
        let request = request
            .as_object()
            .context("expected a speech request object")?;
        for field in request.keys() {
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
                "unsupported Gemini speech field {field}"
            );
        }
        let text = |name| {
            request
                .get(name)
                .filter(|value| !value.is_null())
                .map_or(Ok(""), |value| {
                    value
                        .as_str()
                        .with_context(|| format!("expected string {name}"))
                })
        };
        let input = text("input")?;
        ensure!(!input.trim().is_empty(), "speech input is empty");
        ensure!(
            matches!(text("stream_format")?, "" | "audio"),
            "Gemini speech requires a single audio response"
        );
        let format = match text("response_format")?.to_ascii_lowercase().as_str() {
            "pcm" => AudioFormat::Pcm,
            "" | "wav" | "mp3" | "opus" | "aac" | "flac" => AudioFormat::Wav,
            _ => bail!("Gemini speech outputs WAV or PCM"),
        };
        let instructions = text("instructions")?.trim().trim_end_matches([' ', ':']);
        let prompt = if instructions.is_empty() {
            input.to_owned()
        } else {
            format!("{instructions}: {input}")
        };
        let voice = voice(text("voice")?);
        let body = nextjson::to_vec(&json!({
            "contents":[{"role":"user","parts":[{"text":prompt}]}],
            "generationConfig":{
                "responseModalities":["AUDIO"],
                "speechConfig":{"voiceConfig":{"prebuiltVoiceConfig":{"voiceName":voice}}}
            }
        }))
        .map_err(anyhow::Error::msg)?;
        Ok(Self {
            body: body.into(),
            format,
        })
    }
}

fn voice(requested: &str) -> &'static str {
    const VOICES: &[&str] = &[
        "Zephyr",
        "Puck",
        "Charon",
        "Kore",
        "Fenrir",
        "Leda",
        "Orus",
        "Aoede",
        "Callirrhoe",
        "Autonoe",
        "Enceladus",
        "Iapetus",
        "Umbriel",
        "Algieba",
        "Despina",
        "Erinome",
        "Algenib",
        "Rasalgethi",
        "Laomedeia",
        "Achernar",
        "Alnilam",
        "Schedar",
        "Gacrux",
        "Pulcherrima",
        "Achird",
        "Zubenelgenubi",
        "Vindemiatrix",
        "Sadachbia",
        "Sadaltager",
        "Sulafat",
    ];
    match requested.trim().to_ascii_lowercase().as_str() {
        "alloy" => "Kore",
        "ash" => "Charon",
        "ballad" => "Algieba",
        "coral" => "Sulafat",
        "echo" => "Iapetus",
        "fable" => "Puck",
        "onyx" => "Algenib",
        "nova" => "Zephyr",
        "sage" => "Vindemiatrix",
        "shimmer" => "Achernar",
        "verse" => "Rasalgethi",
        "marin" => "Aoede",
        "cedar" => "Schedar",
        _ => VOICES
            .iter()
            .copied()
            .find(|voice| voice.eq_ignore_ascii_case(requested.trim()))
            .unwrap_or("Kore"),
    }
}

impl AudioFormat {
    pub(crate) async fn reply(self, upstream: Response) -> Result<Response> {
        const AUDIO_JSON_LIMIT: usize = 16 * 1024 * 1024;
        let (parts, body) = upstream.into_parts();
        let mut bytes = Vec::new();
        let mut chunks = body.into_data_stream();
        while let Some(chunk) = chunks.next().await {
            let chunk = chunk?;
            ensure!(
                bytes.len() + chunk.len() <= AUDIO_JSON_LIMIT,
                "Gemini speech reply exceeds 16 MiB"
            );
            bytes.extend_from_slice(&chunk);
        }
        let reply: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
        let (rate, audio) = read_pcm(&reply)?;
        let (mime, audio) = match self {
            Self::Wav => ("audio/wav", wav(rate, &audio)?),
            Self::Pcm => {
                ensure!(
                    rate == 24000,
                    "raw speech PCM requires a 24 kHz sample rate"
                );
                ("audio/pcm", audio)
            }
        };
        let mut response = ([(header::CONTENT_TYPE, mime)], audio).into_response();
        for name in ["x-request-id", "request-id"] {
            if let Some(value) = parts.headers.get(name) {
                response.headers_mut().insert(name, value.clone());
            }
        }
        Ok(response)
    }
}

fn read_pcm(reply: &Value) -> Result<(u32, Vec<u8>)> {
    let candidate = reply
        .pointer("/candidates/0")
        .context("Gemini returned no speech candidate")?;
    if let Some(reason) = candidate.get("finishReason").and_then(Value::as_str) {
        ensure!(reason == "STOP", "Gemini did not finish generating speech");
    }
    let parts = candidate
        .pointer("/content/parts")
        .and_then(Value::as_array)
        .context("Gemini returned no speech parts")?;
    let mut rate = None;
    let mut audio = Vec::new();
    for part in parts {
        let Some(inline) = part.get("inlineData").or_else(|| part.get("inline_data")) else {
            continue;
        };
        let mime = inline
            .get("mimeType")
            .or_else(|| inline.get("mime_type"))
            .and_then(Value::as_str)
            .context("Gemini speech has no MIME type")?;
        let data = inline
            .get("data")
            .and_then(Value::as_str)
            .context("Gemini speech has no audio data")?;
        let decoded = STANDARD
            .decode(data)
            .context("Gemini speech returned invalid base64")?;
        let mut mime = mime.split(';').map(str::trim);
        let (sample_rate, pcm) = match mime
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "audio/wav" => wav_pcm(&decoded)?,
            "audio/l16" => (pcm_sample_rate(mime)?, decoded.as_slice()),
            _ => bail!("Gemini speech did not return 16-bit PCM or WAV"),
        };
        ensure!(
            rate.is_none_or(|rate| sample_rate == rate),
            "Gemini speech parts have different sample rates"
        );
        rate = Some(sample_rate);
        ensure!(
            pcm.len().is_multiple_of(2),
            "Gemini returned an incomplete PCM sample"
        );
        audio.extend_from_slice(pcm);
    }
    ensure!(!audio.is_empty(), "Gemini returned no audio");
    Ok((rate.context("Gemini speech has no sample rate")?, audio))
}

fn pcm_sample_rate<'a>(fields: impl Iterator<Item = &'a str>) -> Result<u32> {
    let mut rate = 24000;
    for field in fields {
        let (name, value) = field
            .split_once('=')
            .context("invalid Gemini audio MIME parameter")?;
        let value = value.trim().trim_matches('"');
        match name.trim().to_ascii_lowercase().as_str() {
            "rate" => {
                rate = value.parse().context("invalid Gemini audio sample rate")?;
                ensure!(
                    (8000..=192000).contains(&rate),
                    "invalid Gemini audio sample rate"
                );
            }
            "channels" => ensure!(value == "1", "Gemini speech must be mono"),
            "codec" => ensure!(
                value.eq_ignore_ascii_case("pcm"),
                "Gemini speech did not return PCM"
            ),
            _ => (),
        }
    }
    Ok(rate)
}

fn wav_pcm(bytes: &[u8]) -> Result<(u32, &[u8])> {
    ensure!(
        bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WAVE",
        "Gemini returned an invalid WAV header"
    );
    let riff_size = u32::from_le_bytes(bytes[4..8].try_into()?) as usize;
    ensure!(
        riff_size == bytes.len() - 8,
        "Gemini WAV length does not match its header"
    );
    let mut chunks = &bytes[12..];
    let mut rate = None;
    let mut pcm = None;
    while !chunks.is_empty() {
        let header = chunks
            .get(..8)
            .context("incomplete Gemini WAV chunk header")?;
        let size = u32::from_le_bytes(header[4..8].try_into()?) as usize;
        ensure!(size <= chunks.len() - 8, "incomplete Gemini WAV chunk");
        let data = &chunks[8..8 + size];
        match &header[..4] {
            b"fmt " => {
                ensure!(
                    rate.is_none() && data.len() >= 16,
                    "invalid Gemini WAV format chunk"
                );
                ensure!(
                    data[..4] == [1, 0, 1, 0] && data[12..16] == [2, 0, 16, 0],
                    "Gemini WAV must contain 16-bit mono PCM"
                );
                let sample_rate = u32::from_le_bytes(data[4..8].try_into()?);
                ensure!(
                    (8000..=192000).contains(&sample_rate),
                    "invalid Gemini WAV sample rate"
                );
                ensure!(
                    data[8..12] == (sample_rate * 2).to_le_bytes(),
                    "invalid Gemini WAV byte rate"
                );
                rate = Some(sample_rate);
            }
            b"data" => {
                ensure!(pcm.is_none(), "multiple Gemini WAV data chunks");
                pcm = Some(data);
            }
            _ => (),
        }
        chunks = chunks
            .get(8 + size + size % 2..)
            .context("missing Gemini WAV chunk padding")?;
    }
    Ok((
        rate.context("missing Gemini WAV format chunk")?,
        pcm.context("missing Gemini WAV data chunk")?,
    ))
}

fn wav(rate: u32, audio: &[u8]) -> Result<Vec<u8>> {
    let len = u32::try_from(audio.len())?;
    let mut wav = Vec::with_capacity(44 + audio.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1_u16.to_le_bytes()); // Mono
    wav.extend_from_slice(&rate.to_le_bytes());
    wav.extend_from_slice(&(rate * 2).to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes()); // One 16-bit sample per frame
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&len.to_le_bytes());
    // Gemini's generateContent audio uses little-endian samples.
    wav.extend_from_slice(audio);
    Ok(wav)
}
