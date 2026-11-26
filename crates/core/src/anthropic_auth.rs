use anyhow::Result;
use axum::{
    body::Bytes,
    http::{Response as HttpResponse, StatusCode, header::AUTHORIZATION},
};
use futures_util::{StreamExt, stream};
use http_body_util::BodyExt;
use reqwest::{Body, RequestBuilder, Response};
use std::sync::OnceLock;

string_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub enum AuthHeader {
        #[default] Both => "" | "both",
        ApiKey => "x-api-key",
        Bearer => "bearer",
    }
}

impl AuthHeader {
    pub(crate) fn is_default(&self) -> bool {
        *self == Self::Both
    }
}

fn conflict(body: &[u8]) -> bool {
    let text = String::from_utf8_lossy(body).to_ascii_lowercase();
    text.contains("x-api-key")
        && text.contains("authorization")
        && [
            "同时",
            "并存",
            "共存",
            "只能使用一个",
            "只能选择一个",
            "只能有一个",
            "cannot both",
            "can't both",
            "can not both",
            "cannot be both",
            "not both",
            "together",
            "at the same time",
            "simultaneous",
            "mutually exclusive",
            "only one of",
            "only one auth",
            "only one header",
            "both present",
            "both be present",
            "both provided",
            "both be provided",
        ]
        .iter()
        .any(|phrase| text.contains(phrase))
}

/// Inspect a bounded prefix while retaining the original response for unrelated 401s.
async fn inspect(response: Response) -> Result<(bool, Response)> {
    let (parts, body) = HttpResponse::<Body>::from(response).into_parts();
    let mut data = body.into_data_stream();
    let mut prefix = Vec::new();
    let mut tail = Bytes::new();
    const LIMIT: usize = 64 * 1024;
    while prefix.len() < LIMIT
        && let Some(next) = data.next().await
    {
        let mut chunk = next?;
        let size = chunk.len().min(LIMIT - prefix.len());
        prefix.extend_from_slice(&chunk.split_to(size));
        tail = chunk;
    }
    let refused = conflict(&prefix);
    let body = Body::wrap_stream(stream::iter([Ok(Bytes::from(prefix)), Ok(tail)]).chain(data));
    Ok((refused, HttpResponse::from_parts(parts, body).into()))
}

/// Only a third-party Messages request initially carrying both provider credentials
/// may negotiate. Other protocols retain their original transport and signing path.
pub(crate) async fn send(
    builder: RequestBuilder,
    learned: &OnceLock<AuthHeader>,
) -> Result<Response> {
    let (client, request) = builder.build_split();
    let mut request = request?;
    if !request.headers().contains_key(AUTHORIZATION)
        || !request.headers().contains_key("x-api-key")
    {
        return Ok(client.execute(request).await?);
    }
    let single = |r: &mut reqwest::Request, mode: AuthHeader| match mode {
        AuthHeader::ApiKey => {
            r.headers_mut().remove(AUTHORIZATION);
        }
        AuthHeader::Bearer => {
            r.headers_mut().remove("x-api-key");
        }
        AuthHeader::Both => {}
    };
    if let Some(&mode) = learned.get() {
        single(&mut request, mode);
        return Ok(client.execute(request).await?);
    }
    if request.body().is_some_and(|b| b.as_bytes().is_none()) {
        let body = request
            .body_mut()
            .take()
            .unwrap()
            .collect()
            .await?
            .to_bytes();
        *request.body_mut() = Some(body.into());
    }
    let template = request.try_clone().expect("replayable Messages body");
    let response = client.execute(request).await?;
    if response.status() != StatusCode::UNAUTHORIZED {
        return Ok(response);
    }
    let (refused, mut response) = inspect(response).await?;
    if refused {
        for mode in [AuthHeader::ApiKey, AuthHeader::Bearer] {
            let mut request = template.try_clone().expect("replayable Messages body");
            single(&mut request, mode);
            response = client.execute(request).await?;
            if !matches!(
                response.status(),
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
            ) {
                if response.status().is_success() {
                    let _ = learned.set(mode);
                }
                break;
            }
        }
    }
    Ok(response)
}
