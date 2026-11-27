use super::{Api, Format};
use crate::translate::sse;
use async_stream::stream;
use axum_core::{body::Body, response::Response};
use bytes::Bytes;
use futures_util::StreamExt;
use http::{StatusCode, header};
use http_body_util::{BodyExt, Limited};
use nextjson::{Value, json};
use std::{future::Future, time::Duration};
use tokio::time::{Instant, sleep_until, timeout};

const FIRST_COMMENT: Duration = Duration::from_secs(3);
const WAIT_INTERVAL: Duration = Duration::from_secs(5);
const IDLE_INTERVAL: Duration = Duration::from_secs(15);
const IDLE_LIMIT: Duration = Duration::from_secs(5 * 60);
const COMMENT: Bytes = Bytes::from_static(b": keepalive\n\n");

pub(super) fn protocol(api: Api<'_>) -> Option<(Api<'static>, Format)> {
    match api {
        Api::Chat("chat/completions") => Some((Api::Chat("chat/completions"), Format::Chat)),
        Api::Responses("responses") => Some((Api::Responses("responses"), Format::Responses)),
        Api::Messages("messages") => Some((Api::Messages("messages"), Format::Messages)),
        _ => None,
    }
}

fn eligible(response: &Response) -> bool {
    response.status() == StatusCode::OK
        && sse::is_event_stream(response.headers())
        && response
            .headers()
            .get_all(header::CONTENT_ENCODING)
            .iter()
            .all(|v| v.as_bytes().eq_ignore_ascii_case(b"identity"))
}

pub(super) async fn send(
    send: impl Future<Output = Response> + Send + 'static,
    format: Format,
    model: String,
) -> Response {
    let mut send = Box::pin(send);
    if let Ok(response) = timeout(FIRST_COMMENT, &mut send).await {
        return relay(response);
    }
    let body = stream! {
        yield Ok(COMMENT);
        let response = loop {
            if let Ok(response) = timeout(WAIT_INTERVAL, &mut send).await {
                break response;
            }
            yield Ok(COMMENT);
        };
        if eligible(&response) {
            let mut chunks = relay_body(response.into_body(), IDLE_INTERVAL, true).into_data_stream();
            while let Some(chunk) = chunks.next().await {
                let failed = chunk.is_err();
                yield chunk;
                if failed {
                    break;
                }
            }
        } else {
            yield Ok(error_event(response, format, &model).await);
        }
    };
    Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from_stream(body))
        .expect("valid SSE headers")
}

pub(super) fn relay(response: Response) -> Response {
    if !eligible(&response) {
        return response;
    }
    let (mut head, body) = response.into_parts();
    head.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(head, relay_body(body, IDLE_INTERVAL, false))
}

fn relay_body(body: Body, interval: Duration, comment_sent: bool) -> Body {
    let mut chunks = body.into_data_stream();
    Body::from_stream(stream! {
        // A UTF-8 BOM is allowed only at the start of an SSE stream.
        // When a gateway comment precedes the upstream stream, consume it here.
        const BOM: &[u8] = b"\xEF\xBB\xBF";
        let mut bom = comment_sent.then_some(0);
        let mut at_start = true;
        let mut last_data = Instant::now();
        let mut next_comment = last_data + interval;
        // Keep the suffix across HTTP chunks. A trailing CR may still be
        // followed by the LF of a CRLF pair, so wait for the LF in that case.
        let mut tail = *b"\n\n\n";
        loop {
            let can_comment = (tail.ends_with(b"\n\n") || matches!(&tail, b"\n\r\n" | b"\r\r\n"))
                && next_comment < last_data + IDLE_LIMIT;
            let chunk = if can_comment {
                tokio::select! {
                    biased;
                    chunk = chunks.next() => chunk,
                    () = sleep_until(next_comment) => {
                        if at_start && bom.is_none() {
                            bom = Some(0);
                        }
                        yield Ok(COMMENT);
                        next_comment = Instant::now() + interval;
                        continue;
                    }
                }
            } else {
                chunks.next().await
            };
            match chunk {
                Some(Ok(mut chunk)) if !chunk.is_empty() => {
                    if let Some(matched) = bom.take() {
                        let left = &BOM[matched..];
                        if chunk.starts_with(left) {
                            chunk = chunk.slice(left.len()..);
                        } else if left.starts_with(&chunk) {
                            bom = Some(matched + chunk.len());
                            continue;
                        } else if matched > 0 {
                            yield Ok(Bytes::from_static(&BOM[..matched]));
                        }
                        if chunk.is_empty() {
                            at_start = false;
                            continue;
                        }
                    }
                    at_start = false;
                    let suffix = &chunk[chunk.len().saturating_sub(tail.len())..];
                    tail.rotate_left(suffix.len());
                    tail[3 - suffix.len()..].copy_from_slice(suffix);
                    last_data = Instant::now();
                    next_comment = last_data + interval;
                    yield Ok(chunk);
                }
                Some(Ok(_)) => (),
                Some(Err(error)) => {
                    yield Err(error);
                    break;
                }
                None => {
                    if let Some(matched) = bom.filter(|matched| *matched > 0) {
                        yield Ok(Bytes::from_static(&BOM[..matched]));
                    }
                    break;
                }
            }
        }
    })
}

async fn error_event(response: Response, format: Format, model: &str) -> Bytes {
    let mut status = response.status();
    let retry_after = response.headers().get(header::RETRY_AFTER).cloned();
    let mut message = None;
    let mut code = None;
    if status.is_success() {
        status = StatusCode::BAD_GATEWAY;
        message = Some("upstream did not return an uncompressed SSE stream".to_owned());
    } else if let Ok(Ok(body)) = timeout(
        Duration::from_secs(2),
        Limited::new(response.into_body(), 64 * 1024).collect(),
    )
    .await
        && let Ok(value) = nextjson::from_slice::<Value>(&body.to_bytes())
    {
        message = value
            .pointer("/error/message")
            .and_then(Value::as_str)
            .map(str::to_owned);
        code = value
            .pointer("/error/code")
            .and_then(Value::as_str)
            .map(str::to_owned);
    }
    let reason = status.canonical_reason().unwrap_or("Upstream error");
    let message = message.map_or_else(
        || format!("HTTP {} {reason}", status.as_u16()),
        |message| format!("HTTP {}: {message}", status.as_u16()),
    );
    let code = code.as_deref().unwrap_or(match status.as_u16() {
        413 => "context_length_exceeded",
        429 => "rate_limit_exceeded",
        529 => "server_is_overloaded",
        _ => "server_error",
    });
    let mut error =
        json!({"message": message, "type": "api_error", "code": code, "status": status.as_u16()});
    if let Some(retry_after) = retry_after.as_ref().and_then(|v| v.to_str().ok()) {
        error
            .as_object_mut()
            .expect("error object")
            .insert("retry_after".into(), retry_after.into());
    }
    let (prefix, data) = match format {
        Format::Chat => ("", json!({"error": error})),
        Format::Messages => ("event: error\n", json!({"type": "error", "error": error})),
        Format::Responses => (
            "event: response.failed\n",
            json!({
                "type": "response.failed", "sequence_number": 0,
                "response": {"object": "response", "model": model, "status": "failed", "error": error, "output": []},
            }),
        ),
        Format::Gemini => unreachable!("Gemini does not use gateway keepalives"),
    };
    format!("{prefix}data: {data}\n\n").into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_core::response::IntoResponse;
    use futures_util::stream;
    use std::convert::Infallible;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn comments_wait_for_event_boundaries_across_chunks() {
        let (tx, rx) = mpsc::channel(1);
        let body = Body::from_stream(stream::unfold(rx, |mut rx| async {
            let bytes = rx.recv().await?;
            Some((Ok::<_, Infallible>(Bytes::from_static(bytes)), rx))
        }));
        let idle = Duration::from_millis(20);
        let mut chunks = relay_body(body, idle, false).into_data_stream();
        assert_eq!(chunks.next().await.unwrap().unwrap(), COMMENT);
        tx.send(b"\xEF").await.unwrap();
        // Drive the relay until it receives the first byte of the split BOM.
        assert_eq!(chunks.next().await.unwrap().unwrap(), COMMENT);
        tx.send(b"\xBB\xBFdata: one\r\n\r").await.unwrap();
        assert_eq!(chunks.next().await.unwrap().unwrap(), "data: one\r\n\r");
        assert!(timeout(idle * 3, chunks.next()).await.is_err());
        tx.send(b"\n").await.unwrap();
        assert_eq!(chunks.next().await.unwrap().unwrap(), "\n");
        assert_eq!(chunks.next().await.unwrap().unwrap(), COMMENT);
        drop(tx);
        assert!(chunks.next().await.is_none());
    }

    #[tokio::test]
    async fn late_errors_keep_the_protocol_shape_and_retry_information() {
        for (format, prefix, path) in [
            (Format::Chat, "data: ", "/error"),
            (Format::Messages, "event: error\ndata: ", "/error"),
            (
                Format::Responses,
                "event: response.failed\ndata: ",
                "/response/error",
            ),
        ] {
            let response = (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, "7")],
                r#"{"error":{"message":"upstream busy","code":"rate_limit_exceeded"}}"#,
            )
                .into_response();
            let bytes = error_event(response, format, "group/model").await;
            let text = std::str::from_utf8(&bytes).unwrap();
            let value: Value =
                nextjson::from_str(text.strip_prefix(prefix).unwrap().trim()).unwrap();
            let error = value.pointer(path).unwrap();
            assert_eq!(error["status"].as_u64(), Some(429));
            assert_eq!(error["type"].as_str(), Some("api_error"));
            assert_eq!(error["code"].as_str(), Some("rate_limit_exceeded"));
            assert_eq!(error["message"].as_str(), Some("HTTP 429: upstream busy"));
            assert_eq!(error["retry_after"].as_str(), Some("7"));
        }
    }
}
