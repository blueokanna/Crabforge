use super::{Api, Format};
use crate::translate::sse;
use anyhow::{Result, ensure};
use axum_core::{body::Body, response::Response};
use bytes::BytesMut;
use futures_util::{StreamExt, stream};
use nextjson::{NsonDeserialize, Value};

pub(super) fn requested(api: Api<'_>, bytes: &[u8]) -> bool {
    match api {
        Api::Gemini(action) => action == "streamGenerateContent",
        _ => {
            #[derive(NsonDeserialize)]
            struct Request {
                #[njson(default)]
                stream: bool,
            }
            api.format().is_some() && nextjson::from_slice::<Request>(bytes).is_ok_and(|r| r.stream)
        }
    }
}

pub(super) async fn wait(response: Response, format: Format) -> Result<Response> {
    if !response.status().is_success() || !sse::is_event_stream(response.headers()) {
        return Ok(response);
    }
    let (head, body) = response.into_parts();
    let mut chunks = body.into_data_stream();
    let mut prefix = BytesMut::new();
    let mut parser = sse::Parser::default();
    'prefetch: while let Some(chunk) = chunks.next().await {
        let chunk = chunk?;
        ensure!(
            prefix.len() + chunk.len() <= 8 * 1024 * 1024,
            "SSE prelude exceeds 8 MiB"
        );
        prefix.extend_from_slice(&chunk);
        for &byte in &chunk {
            if let Some((name, data)) = parser.push(byte)?
                && can_replay(format, &name, &data)
            {
                break 'prefetch;
            }
        }
    }
    let body = stream::iter((!prefix.is_empty()).then_some(Ok(prefix.freeze()))).chain(chunks);
    Ok(Response::from_parts(head, Body::from_stream(body)))
}

fn fields<'a>(value: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

fn present(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::String(text)) => !text.is_empty(),
        Some(Value::Array(values)) => !values.is_empty(),
        Some(Value::Object(entries)) => !entries.is_empty(),
        Some(_) => true,
    }
}

fn can_replay(format: Format, name: &str, data: &str) -> bool {
    if format == Format::Chat && data.trim() == "[DONE]" {
        return true;
    }
    let Ok(event) = nextjson::from_str::<Value>(data) else {
        return false;
    };
    let kind = event
        .get("type")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or(name);
    let delta = event.get("delta");
    let has = |value: &Value, keys: &[&str]| keys.iter().any(|key| present(value.get(key)));
    let type_of = |field| {
        event
            .get(field)
            .and_then(|v| v.get("type"))
            .and_then(Value::as_str)
    };
    match format {
        Format::Chat => fields(&event, "choices")
            .filter_map(|c| c.get("delta"))
            .any(|d| {
                has(
                    d,
                    &[
                        "content",
                        "refusal",
                        "reasoning_content",
                        "reasoning",
                        "tool_calls",
                        "function_call",
                    ],
                )
            }),
        Format::Responses => match kind {
            "response.completed" | "response.incomplete" | "response.failed" => true,
            "response.output_text.delta"
            | "response.refusal.delta"
            | "response.reasoning_summary_text.delta"
            | "response.reasoning_text.delta"
            | "response.function_call_arguments.delta"
            | "response.custom_tool_call_input.delta" => present(delta),
            "response.output_item.added" => {
                matches!(type_of("item"), Some("function_call" | "custom_tool_call"))
            }
            _ => false,
        },
        Format::Messages => match kind {
            "message_stop" => true,
            "content_block_delta" => {
                delta.is_some_and(|d| has(d, &["text", "thinking", "partial_json"]))
            }
            "content_block_start" => matches!(
                type_of("content_block"),
                Some("tool_use" | "server_tool_use")
            ),
            _ => false,
        },
        Format::Gemini => fields(&event, "candidates")
            .filter_map(|c| c.get("content"))
            .flat_map(|c| fields(c, "parts"))
            .any(|part| has(part, &["text", "functionCall"])),
    }
}
