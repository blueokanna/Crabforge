use super::{error, json_response};
use anyhow::{Context, Result, bail, ensure};
use axum_core::{
    body::Body,
    response::{IntoResponse, Response},
};
use futures_util::{Stream, StreamExt, stream};
use http::{StatusCode, header};
use http_body_util::{BodyExt, Limited};
use nextjson::{Map, Value, json};
use std::{collections::BTreeMap, convert::Infallible, time::Instant};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Clone)]
pub(crate) struct Output {
    pub model: String,
    pub generate: bool,
    pub stream: bool,
    pub thinking: bool,
    pub start: Instant,
}

#[derive(Default)]
struct Tool {
    id: String,
    name: String,
    arguments: String,
}

#[derive(Default)]
struct Completion {
    calls: BTreeMap<u64, Tool>,
    reason: Option<String>,
    input: u64,
    output: u64,
    first_token: Option<Instant>,
}

impl Completion {
    fn apply<'a>(&mut self, value: &'a Value, thinking: bool) -> Result<(&'a str, &'a str)> {
        if let Some(e) = value.get("error") {
            bail!(
                "{}",
                e.get("message")
                    .and_then(Value::as_str)
                    .or(e.as_str())
                    .unwrap_or("upstream stream error")
            );
        }
        if let Some(n) = value
            .pointer("/usage/prompt_tokens")
            .and_then(crate::json::unsigned)
        {
            self.input = n;
        }
        if let Some(n) = value
            .pointer("/usage/completion_tokens")
            .and_then(crate::json::unsigned)
        {
            self.output = n;
        }
        let choices = value
            .get("choices")
            .and_then(Value::as_array)
            .context("upstream supplied no choices")?;
        let Some(c) = choices.first() else {
            return Ok(("", ""));
        };
        let msg = c
            .get("delta")
            .or_else(|| c.get("message"))
            .context("missing upstream message")?;
        if let Some(s) = c.get("finish_reason").and_then(Value::as_str) {
            self.reason = Some(s.to_owned());
        }
        if let Some(calls) = msg.get("tool_calls").and_then(Value::as_array) {
            for (i, call) in calls.iter().enumerate() {
                let t = self
                    .calls
                    .entry(
                        call.get("index")
                            .and_then(crate::json::unsigned)
                            .unwrap_or(i as u64),
                    )
                    .or_default();
                if let Some(s) = call.get("id").and_then(Value::as_str) {
                    t.id.push_str(s);
                }
                if let Some(s) = call.pointer("/function/name").and_then(Value::as_str) {
                    t.name.push_str(s);
                }
                if let Some(s) = call.pointer("/function/arguments").and_then(Value::as_str) {
                    t.arguments.push_str(s);
                }
            }
        }
        let text = msg.get("content").and_then(Value::as_str).unwrap_or("");
        let thought = if thinking {
            msg.get("reasoning_content")
                .and_then(Value::as_str)
                .or_else(|| msg.get("reasoning").and_then(Value::as_str))
                .unwrap_or("")
        } else {
            ""
        };
        if !text.is_empty() || !thought.is_empty() {
            self.first_token.get_or_insert_with(Instant::now);
        }
        Ok((text, thought))
    }

    fn take_calls(&mut self) -> Result<Vec<Value>> {
        std::mem::take(&mut self.calls)
            .into_iter()
            .map(|(i, t)| {
                ensure!(!t.name.is_empty(), "upstream sent an incomplete tool call");
                let args = if t.arguments.is_empty() {
                    json!({})
                } else {
                    nextjson::from_str(&t.arguments)
                        .map_err(anyhow::Error::msg)
                        .context("invalid upstream tool arguments")?
                };
                let id = if t.id.is_empty() {
                    format!("call_{i}")
                } else {
                    t.id
                };
                Ok(json!({"id":id,"function":{"index":i,"name":t.name,"arguments":args}}))
            })
            .collect()
    }
}

const NDJSON: [(&str, &str); 1] = [("content-type", "application/x-ndjson")];

fn line(value: Value) -> String {
    let mut s = value.to_string();
    s.push('\n');
    s
}

impl Output {
    fn part(&self, text: &str, thought: &str, calls: Vec<Value>, done: bool) -> Map {
        let now = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .expect("valid current date");
        let mut out = Map::new();
        out.insert("model".into(), self.model.as_str().into());
        out.insert("created_at".into(), now.into());
        out.insert("done".into(), done.into());
        let mut message = Map::new();
        let msg = if self.generate {
            out.insert("response".into(), text.into());
            &mut out
        } else {
            message.insert("role".into(), "assistant".into());
            message.insert("content".into(), text.into());
            &mut message
        };
        if !thought.is_empty() {
            msg.insert("thinking".into(), thought.into());
        }
        if !calls.is_empty() {
            msg.insert("tool_calls".into(), Value::Array(calls));
        }
        if !self.generate {
            out.insert("message".into(), Value::Object(message));
        }
        out
    }

    pub fn loaded(&self) -> Response {
        let mut v = self.part("", "", vec![], true);
        v.insert("done_reason".into(), "load".into());
        if self.stream {
            (NDJSON, line(Value::Object(v))).into_response()
        } else {
            json_response(StatusCode::OK, &v)
        }
    }

    fn frames(self, body: Body, sse: bool) -> impl Stream<Item = Result<Value>> {
        async_stream::try_stream! {
            let mut completion = Completion::default();
            let mut text = String::new();
            let mut thinking = String::new();
            if sse {
                let frames = crate::translate::sse::data(body.into_data_stream());
                futures_util::pin_mut!(frames);
                let mut budget: usize = 16 * 1024 * 1024;
                while let Some(frame) = frames.next().await {
                    let frame = frame?;
                    if frame == "[DONE]" { break; }
                    budget = budget.checked_sub(frame.len()).context("translated stream exceeds 16 MiB")?;
                    let chunk: Value = nextjson::from_str(&frame).map_err(anyhow::Error::msg)?;
                    let (add, thought) = completion.apply(&chunk, self.thinking)?;
                    if add.is_empty() && thought.is_empty() { continue; }
                    if self.stream { yield Value::Object(self.part(add,thought,vec![],false)); } else { text.push_str(add); thinking.push_str(thought); }
                }
            } else {
                let bytes = reply_bytes(body).await?;
                let v: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
                let (add, thought) = completion.apply(&v,self.thinking)?;
                text.push_str(add); thinking.push_str(thought);
            }
            let reason = completion.reason.as_deref().context("upstream reply has no finish reason")?;
            let reason = if reason == "length" { "length" } else { "stop" };
            let mut calls = completion.take_calls()?;
            if self.stream && (!calls.is_empty() || !text.is_empty() || !thinking.is_empty()) {
                yield Value::Object(self.part(&text,&thinking,std::mem::take(&mut calls),false));
                text.clear(); thinking.clear();
            }
            let mut last = self.part(&text,&thinking,calls,true);
            let now = Instant::now();
            let first = completion.first_token.unwrap_or(now);
            last.insert("done_reason".into(), reason.into());
            last.insert("prompt_eval_count".into(), completion.input.into());
            last.insert("eval_count".into(), completion.output.into());
            last.insert("total_duration".into(), ((now-self.start).as_nanos() as u64).into());
            last.insert("prompt_eval_duration".into(), ((first-self.start).as_nanos() as u64).into());
            last.insert("eval_duration".into(), ((now-first).as_nanos() as u64).into());
            yield Value::Object(last);
        }
    }

    pub async fn convert(self, response: Response) -> Result<Response> {
        let (parts, body) = response.into_parts();
        let sse = parts
            .headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next())
            .is_some_and(|s| s.trim().eq_ignore_ascii_case("text/event-stream"));
        let stream = self.stream;
        let mut frames = Box::pin(self.frames(body, sse));
        let first = frames.next().await.context("empty upstream reply")??;
        if !stream {
            return Ok(json_response(StatusCode::OK, &first));
        }
        let lines = stream::iter([Ok(first)]).chain(frames).map(|frame| {
            let v = frame.unwrap_or_else(|e| json!({"error":e.to_string()}));
            Ok::<_, Infallible>(line(v))
        });
        Ok((NDJSON, Body::from_stream(lines)).into_response())
    }
}

async fn reply_bytes(body: Body) -> Result<bytes::Bytes> {
    Ok(Limited::new(body, 2 * 1024 * 1024)
        .collect()
        .await
        .map_err(anyhow::Error::from_boxed)?
        .to_bytes())
}

pub(super) async fn convert_error(response: Response) -> Response {
    let (parts, body) = response.into_parts();
    let bytes = reply_bytes(body).await.unwrap_or_default();
    let v = nextjson::from_slice::<Value>(&bytes).unwrap_or(Value::Null);
    let message = v
        .pointer("/error/message")
        .and_then(Value::as_str)
        .or_else(|| v.get("error").and_then(Value::as_str))
        .or_else(|| std::str::from_utf8(&bytes).ok().filter(|s| !s.is_empty()))
        .unwrap_or_else(|| parts.status.canonical_reason().unwrap_or("upstream error"));
    let mut response = error(parts.status, message);
    for h in ["retry-after", "x-request-id", "request-id"] {
        if let Some(value) = parts.headers.get(h) {
            response.headers_mut().insert(h, value.clone());
        }
    }
    response
}
