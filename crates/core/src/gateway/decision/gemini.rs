use super::{RelayEndpoint, advertised, check_count, reply_model};
use crate::config::{self, Endpoints, Provider};
use anyhow::{Context, Result, ensure};
use http::StatusCode;
use nextjson::{Value, json};
use url::Url;

pub(crate) const GEMINI_MODEL: &str = "gemini-decision-1";

pub(super) fn host(url: &Url) -> bool {
    url.host_str() == Some("generativelanguage.googleapis.com")
}

pub(in crate::gateway) fn gemini_url(provider: &Provider, model: &str) -> Option<Url> {
    (model == GEMINI_MODEL && advertised(provider, model, RelayEndpoint::SystemOne))
        .then(|| gemini_decision_base(&provider.endpoints))
        .flatten()
}

pub(crate) fn gemini_decision_base(bases: &Endpoints) -> Option<Url> {
    if bases.vertex.is_some() {
        return None;
    }
    let mut url = if !bases.systemone.is_empty() {
        config::endpoint(&bases.systemone).ok().filter(host)
    } else {
        [&bases.gemini, &bases.chat, &bases.responses]
            .into_iter()
            .filter_map(|base| config::endpoint(base).ok())
            .find(host)
    }?;
    url.set_path("/v1beta/decisions");
    Some(url)
}

pub(super) fn request(body: Value, model: &str) -> Result<Value> {
    let Value::Object(mut fields) = body else {
        anyhow::bail!("expected a System One request object");
    };
    let state = fields.remove("state").context("missing state")?;
    let state = state
        .as_str()
        .map_or_else(|| state.to_string(), str::to_owned);
    let mut blocks = Vec::new();
    for (field, kind) in [("images", "image"), ("audio", "audio")] {
        if let Some(items) = fields.remove(field).filter(|value| !value.is_null()) {
            let items = items
                .as_array()
                .with_context(|| format!("{field} must be a list of URLs"))?;
            for item in items {
                let url = item
                    .as_str()
                    .with_context(|| format!("{field} must be a list of URLs"))?;
                blocks.push(media(kind, url)?);
            }
        }
    }
    let input = if blocks.is_empty() {
        json!(state)
    } else {
        blocks.insert(0, json!({"type":"text","text":state}));
        json!(blocks)
    };
    ensure!(
        fields.get("input").is_none(),
        "expected state without input"
    );
    fields.insert("input".into(), input);
    fields.insert("model".into(), json!(model));
    let mut questions = fields.remove("questions").context("missing questions")?;
    let entries = questions
        .as_object_mut()
        .context("questions must be an object keyed by name")?;
    check_count(entries.len())?;
    for (_, question) in entries.iter_mut() {
        let kind = question.get("type").and_then(Value::as_str);
        if let Some(kind @ ("choice" | "noul")) = kind {
            let is_choice = kind == "choice";
            if let Some(criteria) = question.get_mut("criteria").and_then(Value::as_object_mut) {
                if is_choice {
                    for (_, description) in criteria.iter_mut().filter(|(_, value)| value.is_null())
                    {
                        *description = json!("");
                    }
                } else {
                    for (from, to) in [("true", "true_description"), ("false", "false_description")]
                    {
                        if let Some(value) = criteria.remove(from)
                            && criteria.get(to).is_none()
                        {
                            criteria.insert(to.into(), value);
                        }
                    }
                }
            }
        }
    }
    fields.insert("questions".into(), questions);
    Ok(Value::Object(fields))
}

fn media(kind: &str, value: &str) -> Result<Value> {
    if let Some(data) = value.strip_prefix("data:") {
        let (meta, data) = data
            .split_once(',')
            .context("invalid decision media data URL")?;
        let mime = meta
            .strip_suffix(";base64")
            .context("decision media requires Base64 data")?;
        ensure!(
            mime.split_once('/')
                .is_some_and(|(prefix, sub)| prefix == kind && !sub.is_empty()),
            "expected {kind} MIME type"
        );
        ensure!(!data.is_empty(), "decision media is empty");
        return Ok(json!({"type":kind,"mime_type":mime,"data":data}));
    }
    let url = Url::parse(value).context("invalid decision media URL")?;
    let scheme = url.scheme();
    ensure!(
        matches!(scheme, "https" | "http")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none(),
        "decision media requires an HTTP(S) URL without embedded credentials"
    );
    let mime = if kind == "audio" {
        match url
            .path()
            .rsplit_once('.')
            .map(|(_, ext)| ext.to_ascii_lowercase())
            .as_deref()
        {
            Some("wav") => Some("audio/wav"),
            Some("mp3") => Some("audio/mp3"),
            Some("aif" | "aiff") => Some("audio/aiff"),
            Some("aac") => Some("audio/aac"),
            Some("ogg" | "oga") => Some("audio/ogg"),
            Some("flac") => Some("audio/flac"),
            Some("m4a") => Some("audio/m4a"),
            Some("opus") => Some("audio/opus"),
            Some("webm") => Some("audio/webm"),
            _ => None,
        }
    } else {
        None
    };
    Ok(match mime {
        Some(mime) => json!({"type":kind,"uri":value,"mime_type":mime}),
        None => json!({"type":kind,"uri":value}),
    })
}

pub(super) fn reply(value: &Value, asked: &str, status: StatusCode) -> Result<(StatusCode, Value)> {
    let answers = value.get("answers").and_then(Value::as_object);
    if answers.is_none_or(nextjson::Map::is_empty)
        && let Some(reason) = value
            .pointer("/prompt_feedback/block_reason")
            .and_then(Value::as_str)
            .filter(|reason| !reason.is_empty())
    {
        let mut message = format!("Google blocked the input ({reason})");
        if let Some(detail) = value
            .pointer("/prompt_feedback/block_reason_message")
            .and_then(Value::as_str)
            .filter(|detail| !detail.is_empty())
        {
            message.push_str(": ");
            message.push_str(detail);
        }
        return Ok((
            StatusCode::BAD_REQUEST,
            json!({"error":{"code":reason,"message":message}}),
        ));
    }
    let answers = answers.context("expected Gemini Decisions answers keyed by name")?;
    let tokens = |path| {
        value
            .pointer(path)
            .and_then(Value::as_u64)
            .unwrap_or_default()
    };
    Ok((
        status,
        json!({"model":reply_model(value, asked),"answers":answers,"usage":{
            "input_tokens":tokens("/usage/total_input_tokens"),"output_tokens":tokens("/usage/total_output_tokens")
        }}),
    ))
}
