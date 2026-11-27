use super::{calibrated_confidence, reply_model};
use crate::config;
use anyhow::{Context, Result};
use nextjson::{Value, json};
use url::Url;

pub(crate) const VERCEL_EVALUATION_MODEL: &str = "typesafe-ai/jev";
const BASE: &str = "/v4/ai";
const METHOD: &str = "/evaluation-model";

pub(crate) fn vercel_evaluation_base(base: &str) -> Option<Url> {
    let mut url = config::endpoint(base).ok()?;
    let path = url.path().trim_end_matches('/');
    let root = path.strip_suffix(METHOD).unwrap_or(path);
    if !root.ends_with(BASE) {
        return None;
    }
    url.set_path(&format!("{root}{METHOD}"));
    Some(url)
}

pub(in crate::gateway) fn evaluation(url: &Url) -> bool {
    url.path()
        .strip_suffix(METHOD)
        .is_some_and(|path| path.ends_with(BASE))
}

pub(super) fn request(mut value: Value) -> Result<Value> {
    let body = value
        .as_object_mut()
        .context("expected a System One request object")?;
    body.remove("model");
    let questions = body
        .get_mut("questions")
        .and_then(Value::as_object_mut)
        .context("questions must be an object keyed by name")?;
    super::check_count(questions.len())?;
    for (_, question) in questions.iter_mut() {
        if let Some(question) = question.as_object_mut()
            && question.get("type").and_then(Value::as_str) == Some("noul")
        {
            question.insert("type".into(), json!("boolean"));
        }
    }
    Ok(value)
}

pub(super) fn reply(mut value: Value, asked: &str) -> Result<Value> {
    let model = reply_model(&value, asked).to_owned();
    let input = value
        .pointer("/usage/inputTokens")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let output = value
        .pointer("/usage/outputTokens")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let answers = value
        .get_mut("answers")
        .and_then(Value::as_object_mut)
        .context("expected Vercel Evaluation answers keyed by name")?;
    for (_, answer) in answers.iter_mut() {
        let confidence = calibrated_confidence(answer);
        let Some(answer) = answer.as_object_mut() else {
            continue;
        };
        if answer.get("type").and_then(Value::as_str) == Some("boolean") {
            answer.insert("type".into(), json!("noul"));
            if let Some(value) = answer.remove("probability") {
                answer.insert("noul".into(), value);
            }
        } else if answer.get("confidence").is_none()
            && let Some(confidence) = confidence
        {
            answer.insert("confidence".into(), json!(confidence));
        }
    }
    Ok(
        json!({"model":model,"answers":answers,"usage":{"input_tokens":input,"output_tokens":output}}),
    )
}
