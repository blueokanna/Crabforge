use crate::{
    catalog::RelayEndpoint,
    config::{self, Provider},
};
use anyhow::{Context, Result, bail, ensure};
use axum_core::{body::Body, response::Response};
use bytes::Bytes;
use http::header;
use http_body_util::{BodyExt, Limited};
use nextjson::{Map, Value, json};
use std::collections::BTreeSet;
use url::Url;

pub(super) const DECISIONS: &str = "decisions";
pub(super) const SYSTEM_ONE: &str = "systemone";
pub(super) const LUNA: &str = "gpt-6-luna";
const MAX_QUESTIONS: usize = 400;
const MAX_REPLY: usize = 4 * 1024 * 1024;

pub(super) fn decisions_url(provider: &Provider, model: &str) -> Option<Url> {
    if !advertised(provider, model, RelayEndpoint::Decisions) {
        return None;
    }
    let endpoints = &provider.endpoints;
    if !endpoints.decisions.is_empty() {
        return url(&endpoints.decisions, DECISIONS);
    }
    [&endpoints.chat, &endpoints.responses]
        .into_iter()
        .filter_map(|base| url(base, DECISIONS))
        .find(|url| {
            config::azure(url)
                || (model == LUNA
                    && !provider.key_env.is_empty()
                    && url.host_str() == Some("api.openai.com"))
        })
}

pub(super) fn system_one_url(provider: &Provider, model: &str) -> Option<Url> {
    if !advertised(provider, model, RelayEndpoint::SystemOne) {
        return None;
    }
    let endpoints = &provider.endpoints;
    if !endpoints.systemone.is_empty() {
        return url(&endpoints.systemone, SYSTEM_ONE);
    }
    let base = match model {
        "decision-model-preview" => [&endpoints.chat, &endpoints.responses]
            .into_iter()
            .filter_map(|base| config::endpoint(base).ok())
            .find(bailian)?,
        "doubao-seed-2-1-lite-260915" => {
            let mut url = [&endpoints.chat, &endpoints.responses]
                .into_iter()
                .filter_map(|base| config::endpoint(base).ok())
                .find(|url| ark(url) && url.path().trim_end_matches('/') == "/api/v3")?;
            url.set_path("/api/v1");
            url
        }
        _ => return None,
    };
    url(base.as_str(), SYSTEM_ONE)
}

fn advertised(provider: &Provider, model: &str, endpoint: RelayEndpoint) -> bool {
    provider
        .model_protocols
        .get(model)
        .and_then(|hints| hints.native_endpoints.as_ref())
        .is_none_or(|endpoints| {
            endpoints.contains(&endpoint)
                || !endpoints.iter().any(|endpoint| {
                    matches!(
                        endpoint,
                        RelayEndpoint::Decisions | RelayEndpoint::SystemOne
                    )
                })
        })
}

pub(super) fn ark(url: &Url) -> bool {
    url.host_str()
        .is_some_and(|host| host.starts_with("ark.") && host.ends_with(".volces.com"))
}

fn bailian(url: &Url) -> bool {
    url.host_str().is_some_and(|host| {
        host.ends_with(".maas.aliyuncs.com")
            || host == "maas.qianwenaiapi.com"
            || host.ends_with(".maas.qianwenaiapi.com")
    })
}

fn url(base: &str, path: &str) -> Option<Url> {
    let mut url = if path == DECISIONS {
        config::api_base(base, "openai")
    } else {
        config::endpoint(base)
    }
    .ok()?;
    let root = url.path() == "/";
    let full = url.path().trim_end_matches('/').rsplit('/').next() == Some(path);
    let mut segments = url.path_segments_mut().ok()?;
    segments.pop_if_empty();
    if root {
        segments.push("v1");
    }
    if !full {
        segments.push(path);
    }
    drop(segments);
    Some(url)
}

#[derive(Clone, Copy)]
enum Kind {
    Predicate,
    Choice,
    Score,
}

pub(super) struct Question {
    key: String,
    name: Option<String>,
    kind: Kind,
    options: Vec<(String, Value)>,
    score_criteria: Vec<String>,
}

pub(super) enum Reply {
    Decisions(Vec<Question>),
    SystemOne(Vec<Question>),
}

pub(super) fn request(
    path: &str,
    model: &str,
    bytes: &[u8],
    destination: &Url,
) -> Result<(Bytes, Reply)> {
    let value: Value = nextjson::from_slice(bytes).map_err(anyhow::Error::msg)?;
    let (body, reply) = if path == SYSTEM_ONE {
        let (body, questions) = into_decisions(&value, model)?;
        (body, Reply::SystemOne(questions))
    } else {
        let (body, questions) = into_system_one(&value, model, destination)?;
        (body, Reply::Decisions(questions))
    };
    Ok((body.to_string().into(), reply))
}

fn check_fields(value: &Value, fields: &[&str], location: &str) -> Result<()> {
    let object = value
        .as_object()
        .with_context(|| format!("{location} must be an object"))?;
    if let Some(key) = object.keys().find(|key| !fields.contains(key)) {
        bail!("{location}.{key} has no counterpart in the target decision protocol");
    }
    Ok(())
}

fn text<'a>(value: &'a Value, field: &str) -> Result<Option<&'a str>> {
    value
        .get(field)
        .filter(|value| !value.is_null())
        .map(|value| {
            value
                .as_str()
                .with_context(|| format!("{field} must be text"))
        })
        .transpose()
}

fn check_count(count: usize) -> Result<()> {
    ensure!(
        (1..=MAX_QUESTIONS).contains(&count),
        "expected 1 to {MAX_QUESTIONS} questions"
    );
    Ok(())
}

fn into_decisions(value: &Value, model: &str) -> Result<(Value, Vec<Question>)> {
    check_fields(value, &["model", "state", "questions", "images"], "request")?;
    let state = value.get("state").context("missing state")?;
    let mut parts = vec![if state.is_object()
        || state
            .as_array()
            .is_some_and(|list| !list.is_empty() && list.iter().all(Value::is_object))
    {
        json!({"type":"input_json","data":state})
    } else {
        let state = state
            .as_str()
            .map_or_else(|| state.to_string(), str::to_owned);
        json!({"type":"input_text","text":state})
    }];
    if let Some(images) = value.get("images").filter(|value| !value.is_null()) {
        for image in images.as_array().context("images must be a list of URLs")? {
            let url = image.as_str().context("images must be a list of URLs")?;
            ensure!(!url.is_empty(), "image URL is empty");
            parts.push(json!({"type":"input_image","image_url":url}));
        }
    }
    let input = match state.as_str() {
        Some(state) if parts.len() == 1 && !state.is_empty() => json!(state),
        _ => json!([{"type":"message","role":"user","content":parts}]),
    };
    let questions = value
        .get("questions")
        .and_then(Value::as_object)
        .context("questions must be an object keyed by name")?;
    check_count(questions.len())?;
    let mut converted = Vec::with_capacity(questions.len());
    let mut order = Vec::with_capacity(questions.len());
    for (key, question) in questions.iter() {
        let location = format!("questions.{key}");
        check_fields(
            question,
            &["type", "instructions", "criteria", "choices"],
            &location,
        )?;
        let instructions = text(question, "instructions")?.unwrap_or_default();
        let (kind, wire, options) = match text(question, "type")?.unwrap_or_default() {
            "noul" | "boolean" => {
                ensure!(
                    ["criteria", "choices"]
                        .into_iter()
                        .all(|field| question.get(field).is_none_or(Value::is_null)),
                    "{location} predicate criteria have no counterpart in OpenAI Decisions"
                );
                (Kind::Predicate, json!({"type":"predicate"}), vec![])
            }
            kind @ ("choice" | "score") => {
                let criteria = question.get("criteria").filter(|value| !value.is_null());
                let choices = question.get("choices").filter(|value| !value.is_null());
                ensure!(
                    criteria.is_none() || choices.is_none(),
                    "{location} has both criteria and choices"
                );
                let options = criteria
                    .or(choices)
                    .with_context(|| format!("{location} requires criteria"))?;
                let options = system_one_options(options).with_context(|| location.clone())?;
                let field = if kind == "choice" { "value" } else { "label" };
                let entries = options
                    .iter()
                    .map(|(name, description)| {
                        let mut entry = Map::new();
                        entry.insert(field.into(), json!(name));
                        if let Some(description) = description {
                            entry.insert("description".into(), json!(description));
                        }
                        Value::Object(entry)
                    })
                    .collect::<Vec<_>>();
                let (kind, wire) = if kind == "choice" {
                    (Kind::Choice, json!({"type":"choice","choices":entries}))
                } else {
                    (Kind::Score, json!({"type":"score","levels":entries}))
                };
                let options = options
                    .into_iter()
                    .map(|(name, _)| (name.clone(), json!(name)))
                    .collect();
                (kind, wire, options)
            }
            kind => bail!("{location} has unsupported question type {kind:?}"),
        };
        let mut wire = wire;
        let fields = wire.as_object_mut().expect("question object");
        fields.insert("name".into(), json!(key));
        fields.insert("instructions".into(), json!(instructions));
        converted.push(wire);
        order.push(Question {
            key: key.to_owned(),
            name: Some(key.to_owned()),
            kind,
            options,
            score_criteria: Vec::new(),
        });
    }
    Ok((
        json!({"model":model,"input":input,"questions":converted}),
        order,
    ))
}

fn system_one_options(value: &Value) -> Result<Vec<(String, Option<String>)>> {
    let options = if let Some(object) = value.as_object() {
        object
            .iter()
            .map(|(name, value)| {
                let description = (!value.is_null()).then(|| {
                    value
                        .as_str()
                        .map_or_else(|| value.to_string(), str::to_owned)
                });
                (name.to_owned(), description)
            })
            .collect::<Vec<_>>()
    } else {
        value
            .as_array()
            .context("criteria must be a list of names or an object")?
            .iter()
            .map(|name| {
                Ok((
                    name.as_str()
                        .context("criterion name must be text")?
                        .to_owned(),
                    None,
                ))
            })
            .collect::<Result<Vec<_>>>()?
    };
    let mut seen = BTreeSet::new();
    ensure!(
        !options.is_empty() && options.iter().all(|(name, _)| seen.insert(name)),
        "criteria must have distinct names"
    );
    Ok(options)
}

fn into_system_one(
    value: &Value,
    model: &str,
    destination: &Url,
) -> Result<(Value, Vec<Question>)> {
    check_fields(
        value,
        &[
            "model",
            "input",
            "questions",
            "prompt_cache_key",
            "prompt_cache_options",
            "safety_identifier",
        ],
        "request",
    )?;
    let input = value.get("input").context("missing input")?;
    let mut images = Vec::new();
    let state = if input.is_string() {
        input.clone()
    } else {
        let mut messages = Vec::new();
        for (index, message) in input
            .as_array()
            .context("input must be text or a list of messages")?
            .iter()
            .enumerate()
        {
            let location = format!("input[{index}]");
            check_fields(message, &["type", "role", "content"], &location)?;
            ensure!(
                matches!(text(message, "type")?, None | Some("message")),
                "{location} must be a message"
            );
            let role = text(message, "role")?
                .filter(|role| !role.is_empty())
                .with_context(|| format!("{location} is missing role"))?;
            let (content, refusal) = decision_parts(message.get("content"), Some(&mut images))
                .with_context(|| location.clone())?;
            let wire = if refusal.is_empty() {
                json!({"role":role,"content":content})
            } else {
                ensure!(
                    role == "assistant",
                    "{location} has a refusal outside an assistant message"
                );
                json!({"role":role,"content":content,"refusal":refusal})
            };
            messages.push(wire);
        }
        json!({"messages":messages})
    };
    ensure!(
        images.is_empty() || !(ark(destination) || bailian(destination)),
        "the target System One model does not accept images"
    );
    let questions = value
        .get("questions")
        .and_then(Value::as_array)
        .context("questions must be a list")?;
    check_count(questions.len())?;
    let mut names = BTreeSet::new();
    for question in questions {
        if let Some(name) = text(question, "name")? {
            ensure!(
                names.insert(name.to_owned()),
                "duplicate question name {name:?}"
            );
        }
    }
    let mut mapped = Map::new();
    let mut order = Vec::with_capacity(questions.len());
    for (index, question) in questions.iter().enumerate() {
        let name = text(question, "name")?.map(str::to_owned);
        let key = name.clone().unwrap_or_else(|| {
            (index + 1..)
                .map(|number| format!("question_{number}"))
                .find(|key| names.insert(key.clone()))
                .expect("unused question name")
        });
        let location = format!("questions[{index}]");
        let kind = text(question, "type")?.unwrap_or_default();
        let (instructions, refusal) =
            decision_parts(question.get("instructions"), None).with_context(|| location.clone())?;
        ensure!(
            refusal.is_empty(),
            "{location} has a refusal in its instructions"
        );
        let mut score_criteria = Vec::new();
        let (kind, wire, options) = match kind {
            "predicate" => {
                check_fields(question, &["type", "name", "instructions"], &location)?;
                (
                    Kind::Predicate,
                    json!({"type":"noul","instructions":instructions}),
                    vec![],
                )
            }
            "choice" => {
                check_fields(
                    question,
                    &["type", "name", "instructions", "choices"],
                    &location,
                )?;
                let choices = question
                    .get("choices")
                    .and_then(Value::as_array)
                    .filter(|choices| !choices.is_empty())
                    .with_context(|| format!("{location} requires choices"))?;
                let mut criteria = Map::new();
                let mut options = Vec::with_capacity(choices.len());
                for choice in choices {
                    check_fields(choice, &["value", "description"], &location)?;
                    let value = choice.get("value").context("missing choice value")?;
                    let name =
                        option_name(value).context("choice value must be text or a boolean")?;
                    ensure!(
                        criteria.get(&name).is_none(),
                        "duplicate choice {name:?} in {location}"
                    );
                    criteria.insert(name.clone(), json!(text(choice, "description")?));
                    options.push((name, value.clone()));
                }
                (
                    Kind::Choice,
                    json!({"type":"choice","instructions":instructions,"criteria":criteria}),
                    options,
                )
            }
            "score" => {
                check_fields(
                    question,
                    &["type", "name", "instructions", "levels"],
                    &location,
                )?;
                let levels = question
                    .get("levels")
                    .and_then(Value::as_array)
                    .filter(|levels| !levels.is_empty())
                    .with_context(|| format!("{location} requires levels"))?;
                let mut seen = BTreeSet::new();
                let mut options = Vec::with_capacity(levels.len());
                for (index, level) in levels.iter().enumerate() {
                    check_fields(level, &["label", "description"], &location)?;
                    let label = text(level, "label")?.context("missing level label")?;
                    ensure!(
                        seen.insert(label),
                        "duplicate score level {label:?} in {location}"
                    );
                    let description = text(level, "description")?.filter(|text| !text.is_empty());
                    score_criteria.push(description.map_or_else(
                        || label.to_owned(),
                        |description| format!("{label}: {description}"),
                    ));
                    options.push((label.to_owned(), json!(index)));
                }
                (
                    Kind::Score,
                    json!({"type":"score","instructions":instructions,"criteria":score_criteria}),
                    options,
                )
            }
            _ => bail!("{location} has unsupported question type {kind:?}"),
        };
        mapped.insert(key.clone(), wire);
        order.push(Question {
            key,
            name,
            kind,
            options,
            score_criteria,
        });
    }
    let mut request = json!({"model":model,"state":state,"questions":mapped});
    if !images.is_empty() {
        request
            .as_object_mut()
            .expect("request object")
            .insert("images".into(), json!(images));
    }
    Ok((request, order))
}

fn decision_parts(
    value: Option<&Value>,
    mut images: Option<&mut Vec<String>>,
) -> Result<(String, String)> {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Ok((String::new(), String::new()));
    };
    if let Some(text) = value.as_str() {
        return Ok((text.to_owned(), String::new()));
    }
    let parts = value
        .as_array()
        .context("expected text or a list of parts")?;
    let mut content = Vec::new();
    let mut refusals = Vec::new();
    for part in parts {
        let kind = text(part, "type")?.unwrap_or_default();
        let field = match kind {
            "input_text" | "output_text" => "text",
            "input_json" => "data",
            "input_image" => "image_url",
            "refusal" => "refusal",
            _ => bail!("unsupported Decisions content part {kind:?}"),
        };
        check_fields(part, &["type", field], "content part")?;
        match field {
            "data" => content.push(
                part.get(field)
                    .context("input_json lacks data")?
                    .to_string(),
            ),
            "image_url" => {
                let url = text(part, field)?
                    .filter(|url| url.starts_with("data:image/"))
                    .context("System One requires an inline image data URL")?;
                images
                    .as_deref_mut()
                    .context("System One instructions do not accept images")?
                    .push(url.to_owned());
            }
            "refusal" => refusals.push(text(part, field)?.context("missing refusal text")?),
            _ => content.push(
                text(part, field)?
                    .context("missing content text")?
                    .to_owned(),
            ),
        }
    }
    Ok((content.join("\n"), refusals.join("\n")))
}

fn option_name(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::to_owned)
        .or_else(|| value.as_bool().map(|value| value.to_string()))
}

pub(super) fn probability(value: &Value) -> Option<f64> {
    value.as_f64().filter(|number| (0.0..=1.0).contains(number))
}

pub(super) fn calibrated_confidence(answer: &Value) -> Option<f64> {
    let probabilities = answer.get("probabilities")?.as_object()?;
    let mut max = 0.0_f64;
    for (_, value) in probabilities.iter() {
        max = max.max(probability(value)?);
    }
    (probabilities.len() >= 2).then(|| {
        let even = 1.0 / probabilities.len() as f64;
        ((max - even) / (1.0 - even)).clamp(0.0, 1.0)
    })
}

impl Reply {
    pub(super) async fn convert(self, upstream: Response, asked: &str) -> Result<Response> {
        let (parts, body) = upstream.into_parts();
        let body = Limited::new(body, MAX_REPLY)
            .collect()
            .await
            .map_err(|_| anyhow::anyhow!("could not read decision response within 4 MiB"))?
            .to_bytes();
        let value: Value = nextjson::from_slice(&body)
            .map_err(|_| anyhow::anyhow!("invalid upstream decision JSON"))?;
        let converted = match self {
            Self::Decisions(questions) => decisions_reply(&value, &questions, asked)?,
            Self::SystemOne(questions) => system_one_reply(&value, &questions, asked)?,
        };
        let mut response = Response::builder()
            .status(parts.status)
            .header(header::CONTENT_TYPE, "application/json");
        for name in ["retry-after", "x-request-id", "request-id"] {
            if let Some(value) = parts.headers.get(name) {
                response = response.header(name, value);
            }
        }
        Ok(response.body(Body::from(converted.to_string()))?)
    }
}

fn decisions_reply(value: &Value, questions: &[Question], asked: &str) -> Result<Value> {
    let answers = value
        .get("answers")
        .and_then(Value::as_object)
        .context("expected System One answers by name")?;
    let mut translated = Vec::with_capacity(questions.len());
    for question in questions {
        let name = &question.name;
        let refusal = || json!({"type":"refusal","name":name});
        let Some(answer) = answers
            .get(&question.key)
            .filter(|answer| answer.is_object())
        else {
            translated.push(refusal());
            continue;
        };
        if answer.get("type").and_then(Value::as_str) == Some("refusal") {
            translated.push(refusal());
            continue;
        }
        let probabilities = answer.get("probabilities").and_then(Value::as_object);
        let confidence = answer
            .get("confidence")
            .and_then(probability)
            .or_else(|| calibrated_confidence(answer))
            .unwrap_or_default();
        let wire = match question.kind {
            Kind::Predicate => {
                let probability = answer.get("noul").and_then(probability)
                    .or_else(|| answer.get("probability").and_then(probability));
                probability.map(|p| json!({"type":"predicate","name":name,"probability":p}))
            }
            Kind::Choice => {
                let picked = answer.get("choice").and_then(option_name);
                if let Some(picked) = picked {
                    let (_, selected) = question.options.iter().find(|(key, _)| key == &picked)
                        .with_context(|| format!("System One answered {} with an unknown choice", question.key))?;
                    let probabilities = question.options.iter().filter_map(|(key, value)| {
                        let p = probability(probabilities?.get(key)?)?;
                        Some(json!({"value":value,"probability":p}))
                    }).collect::<Vec<_>>();
                    Some(json!({"type":"choice","name":name,"choice":selected,"confidence":confidence,"probabilities":probabilities}))
                } else {
                    None
                }
            }
            Kind::Score => answer.get("score").and_then(Value::as_f64).map(|score| {
                let probabilities = question.options.iter().zip(&question.score_criteria).filter_map(|((label, value), criterion)| {
                    let probabilities = probabilities?;
                    let p = probabilities.get(&value.to_string()).or_else(|| probabilities.get(label))
                        .or_else(|| probabilities.get(criterion)).and_then(probability)?;
                    Some(json!({"label":label,"value":value,"probability":p}))
                }).collect::<Vec<_>>();
                json!({"type":"score","name":name,"score":score,"confidence":confidence,"probabilities":probabilities})
            }),
        };
        translated.push(wire.unwrap_or_else(refusal));
    }
    let (input, output) = usage(value);
    Ok(
        json!({"model":reply_model(value, asked),"answers":translated,"usage":{
            "input_tokens":input,"input_tokens_details":{"cached_tokens":0,"cache_write_tokens":0},
            "output_tokens":output,"output_tokens_details":{"reasoning_tokens":0},"total_tokens":input.saturating_add(output)
        }}),
    )
}

fn system_one_reply(value: &Value, questions: &[Question], asked: &str) -> Result<Value> {
    let answers = value
        .get("answers")
        .and_then(Value::as_array)
        .context("expected a list of Decisions answers")?;
    let mut translated = Map::new();
    for (index, answer) in answers.iter().enumerate() {
        let question = match text(answer, "name")? {
            Some(name) => questions.iter().find(|question| question.key == name),
            None => questions.get(index),
        }
        .context("Decisions returned an answer for an unknown question")?;
        ensure!(
            translated.get(&question.key).is_none(),
            "Decisions returned duplicate answers"
        );
        let kind = text(answer, "type")?.context("Decisions answer lacks type")?;
        let wire = match (kind, question.kind) {
            ("predicate", Kind::Predicate) => {
                let probability = answer
                    .get("probability")
                    .and_then(probability)
                    .context("invalid predicate probability")?;
                json!({"type":"noul","noul":probability})
            }
            ("choice", Kind::Choice) | ("score", Kind::Score) => {
                let mut probabilities = Map::new();
                let options = answer
                    .get("probabilities")
                    .and_then(Value::as_array)
                    .map_or(&[][..], |values| values.as_slice());
                for option in options {
                    let name = if kind == "choice" {
                        option.get("value").and_then(option_name)
                    } else {
                        text(option, "label")?.map(str::to_owned)
                    }
                    .context("missing decision probability label")?;
                    let p = option
                        .get("probability")
                        .and_then(probability)
                        .context("invalid decision probability")?;
                    ensure!(
                        probabilities.get(&name).is_none(),
                        "Decisions returned duplicate probabilities"
                    );
                    probabilities.insert(name, json!(p));
                }
                let mut wire = Map::new();
                wire.insert("type".into(), json!(kind));
                wire.insert("probabilities".into(), Value::Object(probabilities));
                if let Some(confidence) = answer
                    .get("confidence")
                    .filter(|confidence| !confidence.is_null())
                {
                    wire.insert(
                        "confidence".into(),
                        json!(probability(confidence).context("invalid decision confidence")?),
                    );
                }
                if kind == "choice" {
                    let choice = answer
                        .get("choice")
                        .and_then(option_name)
                        .context("invalid decision choice")?;
                    ensure!(
                        question.options.iter().any(|(key, _)| key == &choice),
                        "Decisions returned an unknown choice"
                    );
                    wire.insert("choice".into(), json!(choice));
                } else {
                    wire.insert(
                        "score".into(),
                        json!(
                            answer
                                .get("score")
                                .and_then(Value::as_f64)
                                .context("missing decision score")?
                        ),
                    );
                }
                Value::Object(wire)
            }
            ("refusal", _) => json!({"type":"refusal"}),
            _ => bail!("Decisions returned an unexpected answer type"),
        };
        translated.insert(question.key.clone(), wire);
    }
    let ordered = Map::from_iter(questions.iter().map(|question| {
        let answer = translated
            .remove(&question.key)
            .unwrap_or_else(|| json!({"type":"refusal"}));
        (question.key.clone(), answer)
    }));
    let (input, output) = usage(value);
    Ok(
        json!({"model":reply_model(value, asked),"answers":ordered,"usage":{"input_tokens":input,"output_tokens":output}}),
    )
}

fn usage(value: &Value) -> (u64, u64) {
    let tokens = |path| {
        value
            .pointer(path)
            .and_then(Value::as_u64)
            .unwrap_or_default()
    };
    (
        tokens("/usage/input_tokens"),
        tokens("/usage/output_tokens"),
    )
}

fn reply_model<'a>(value: &'a Value, asked: &'a str) -> &'a str {
    value
        .get("model")
        .and_then(Value::as_str)
        .filter(|model| !model.is_empty())
        .unwrap_or(asked)
}
