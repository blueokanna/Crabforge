use super::*;
use super::{sse, stream_input::Decoder, stream_output::Encoder};
use axum_core::{
    body::Body,
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use http::StatusCode;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::OwnedSemaphorePermit;

struct Reply {
    parts: Vec<Part>,
    limited: bool,
    filtered: bool,
    input: u64,
    output: u64,
    id: String,
    created: u64,
}

impl Reply {
    fn read(format: Format, v: &Value) -> Result<Self> {
        let (parts, end, input, output) = match format {
            Format::Chat => {
                let c = array(v, "choices")?
                    .first()
                    .context("empty upstream choices")?;
                let m = field(c, "message");
                let refusal = field(m, "refusal");
                ensure!(refusal.is_null(), "upstream refused the request: {refusal}");
                (
                    read_turn(format, m)?.parts,
                    text(c, "finish_reason")?,
                    v.pointer("/usage/prompt_tokens"),
                    v.pointer("/usage/completion_tokens"),
                )
            }
            Format::Messages => (
                read_parts(format, field(v, "content"))?,
                text(v, "stop_reason")?,
                v.pointer("/usage/input_tokens"),
                v.pointer("/usage/output_tokens"),
            ),
            Format::Gemini => {
                let c = array(v, "candidates")?
                    .first()
                    .context("empty Gemini candidates")?;
                (
                    read_parts(
                        format,
                        c.pointer("/content/parts")
                            .context("missing Gemini parts")?,
                    )?,
                    text(c, "finishReason")?,
                    v.pointer("/usageMetadata/promptTokenCount"),
                    v.pointer("/usageMetadata/candidatesTokenCount"),
                )
            }
            Format::Responses => {
                let mut parts = vec![];
                for item in array(v, "output")? {
                    parts.extend(read_turn(format, item)?.parts);
                }
                let end = match text(v, "status")? {
                    "completed" => "stop",
                    "incomplete" => text(field(v, "incomplete_details"), "reason")?,
                    s => bail!("upstream response status {s}"),
                };
                (
                    parts,
                    end,
                    v.pointer("/usage/input_tokens"),
                    v.pointer("/usage/output_tokens"),
                )
            }
        };
        let limited = matches!(
            end,
            "length" | "max_tokens" | "MAX_TOKENS" | "max_output_tokens"
        );
        let filtered = matches!(end, "content_filter" | "SAFETY" | "refusal");
        ensure!(
            limited
                || filtered
                || matches!(
                    end,
                    "stop" | "end_turn" | "STOP" | "tool_calls" | "tool_use" | "stop_sequence"
                ),
            "unsupported upstream finish reason {end}"
        );
        ensure!(
            !parts.is_empty() || limited || filtered,
            "upstream returned an empty reply"
        );
        ensure!(
            parts.iter().all(|p| !matches!(p, Part::Result { .. })),
            "unexpected tool result in reply"
        );
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        Ok(Self {
            parts,
            limited,
            filtered,
            input: input.and_then(crate::json::unsigned).unwrap_or(0),
            output: output.and_then(crate::json::unsigned).unwrap_or(0),
            id: format!("cf_{}", now.as_nanos()),
            created: now.as_secs(),
        })
    }

    fn encode(self, format: Format, model: &str) -> Result<Value> {
        let calls = self.parts.iter().any(|p| matches!(p, Part::Call { .. }));
        let mut items = write_turns(
            format,
            &[Turn {
                role: "assistant".into(),
                parts: self.parts,
            }],
        )?;
        let i = self.input;
        let o = self.output;
        let id = self.id;
        let total = i.saturating_add(o);
        Ok(match format {
            Format::Chat => {
                json!({"id":format!("chatcmpl-{id}"),"object":"chat.completion","created":self.created,"model":model,"choices":[{"index":0,"message":items.pop().unwrap_or_else(||json!({"role":"assistant","content":""})),"finish_reason":if self.limited{"length"}else if self.filtered{"content_filter"}else if calls{"tool_calls"}else{"stop"}}],"usage":{"prompt_tokens":i,"completion_tokens":o,"total_tokens":total}})
            }
            Format::Messages => {
                json!({"id":format!("msg_{id}"),"type":"message","role":"assistant","content":items.first().and_then(|m|m.get("content")).cloned().unwrap_or_else(||json!([])),"model":model,"stop_reason":if self.limited{"max_tokens"}else if self.filtered{"refusal"}else if calls{"tool_use"}else{"end_turn"},"stop_sequence":null,"usage":{"input_tokens":i,"output_tokens":o}})
            }
            Format::Gemini => {
                json!({"responseId":id,"modelVersion":model,"candidates":[{"index":0,"content":items.pop().unwrap_or_else(||json!({"role":"model","parts":[]})),"finishReason":if self.limited{"MAX_TOKENS"}else if self.filtered{"SAFETY"}else{"STOP"}}],"usageMetadata":{"promptTokenCount":i,"candidatesTokenCount":o,"totalTokenCount":total}})
            }
            Format::Responses => {
                let interrupted = self.limited || self.filtered;
                let state = if interrupted {
                    "incomplete"
                } else {
                    "completed"
                };
                for (n, item) in items.iter_mut().enumerate() {
                    let prefix = if text(item, "type")? == "message" {
                        "msg"
                    } else {
                        "fc"
                    };
                    let item = item.as_object_mut().context("expected output item")?;
                    item.insert("id".into(), format!("{prefix}_{id}_{n}").into());
                    item.insert("status".into(), state.into());
                    if let Some(content) = item.get_mut("content").and_then(Value::as_array_mut) {
                        for p in content {
                            p.as_object_mut()
                                .context("expected output part")?
                                .insert("annotations".into(), json!([]));
                        }
                    }
                }
                json!({"id":format!("resp_{id}"),"object":"response","created_at":self.created,"model":model,"status":state,"error":null,"incomplete_details":if interrupted{json!({"reason":if self.limited{"max_output_tokens"}else{"content_filter"}})}else{Value::Null},"output":items,"usage":{"input_tokens":i,"output_tokens":o,"total_tokens":total}})
            }
        })
    }
}

fn live(
    upstream: Response,
    from: Format,
    mut encoder: Encoder,
    permit: std::sync::Arc<OwnedSemaphorePermit>,
) -> impl futures_util::Stream<Item = Result<String>> {
    async_stream::try_stream! {
        let _permit = permit;
        let frames = sse::data(upstream.into_body().into_data_stream());
        futures_util::pin_mut!(frames);
        let mut decoder = Decoder::default();
        let mut started = false;
        let mut budget: usize = 16 * 1024 * 1024;
        while let Some(frame) = frames.next().await {
            let data = frame?;
            budget = budget.checked_sub(data.len()).context("translated stream exceeds 16 MiB")?;
            let parts = decoder.read(from,&data)?;
            if !started { encoder.start(decoder.input); started = true; }
            for part in parts { encoder.part(part)?; }
            let out = encoder.drain();
            if !out.is_empty() { yield out; }
            if decoder.done { break; }
        }
        encoder.finish(decoder.end(from)?,decoder.input,decoder.output)?;
        yield encoder.drain();
    }
}

pub async fn convert_reply(
    upstream: Response,
    from: Format,
    to: Format,
    model: &str,
    stream: bool,
    permit: std::sync::Arc<OwnedSemaphorePermit>,
) -> Result<Response> {
    const SSE_HEADERS: [(&str, &str); 1] = [("content-type", "text/event-stream")];
    if stream && super::sse::is_event_stream(upstream.headers()) {
        let mut frames = Box::pin(live(upstream, from, Encoder::new(to, model)?, permit));
        let first = frames
            .next()
            .await
            .context("empty upstream event stream")??;
        let body = futures_util::stream::once(async move { Ok(first) }).chain(frames);
        return Ok((SSE_HEADERS, Body::from_stream(body)).into_response());
    }
    let mut bytes = vec![];
    let mut chunks = upstream.into_body().into_data_stream();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk?;
        ensure!(
            bytes.len() + chunk.len() <= 2 * 1024 * 1024,
            "translated reply exceeds 2 MiB"
        );
        bytes.extend_from_slice(&chunk);
    }
    let reply = Reply::read(
        from,
        &nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?,
    )?;
    if !stream {
        return Ok(crate::gateway::json_response(
            StatusCode::OK,
            &reply.encode(to, model)?,
        ));
    }
    let mut encoder = Encoder::new(to, model)?;
    encoder.start(reply.input);
    for part in reply.parts {
        encoder.part(part)?;
    }
    let reason = if reply.limited {
        "length"
    } else if reply.filtered {
        "content_filter"
    } else {
        "stop"
    };
    encoder.finish(reason, reply.input, reply.output)?;
    Ok((SSE_HEADERS, encoder.drain()).into_response())
}
