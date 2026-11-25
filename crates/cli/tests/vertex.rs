use anyhow::{Context, Result};
use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::OriginalUri,
    http::HeaderMap,
    response::IntoResponse,
};
use crabforge_core::config;
use serde_json::{Value, json};
use std::{convert::Infallible, fs, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
    process::Command,
    time::timeout,
};

#[tokio::test]
async fn project_gateway_selects_google_anthropic_and_partner_paths() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let upstream = format!("http://{}/v1/", listener.local_addr()?);
    let resume = Arc::new(tokio::sync::Notify::new());
    let gate = resume.clone();
    let app = Router::new().fallback(
        move |OriginalUri(uri): OriginalUri, headers: HeaderMap, Json(body): Json<Value>| {
            let gate = gate.clone();
            async move {
                assert_eq!(headers["x-goog-api-key"], "vertex-test");
                assert!(!headers.contains_key("authorization"));
                let observed = json!({"uri":uri.to_string(),"body":body});
                if uri.path().ends_with(":streamGenerateContent")
                    || uri.path().ends_with(":streamRawPredict")
                {
                    use futures_util::{StreamExt, stream};
                    let head = stream::once(async move {
                        Ok::<_, Infallible>(Bytes::from(format!("data: {observed}\n\n")))
                    });
                    let tail = stream::once(async move {
                        gate.notified().await;
                        Ok(Bytes::from_static(b"data: {}\n\n"))
                    });
                    return (
                        [("content-type", "text/event-stream")],
                        Body::from_stream(head.chain(tail)),
                    )
                        .into_response();
                }
                Json(observed).into_response()
            }
        },
    );
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let root = tempfile::tempdir()?;
    let missing = root.path().join("absent-adc.json");
    let vertex = json!({"project":"example-project","location":"eu","base_url":upstream,"credentials":missing});
    let cfg = json!({"listen":"127.0.0.1:0","providers":[{"id":"cloud","key_env":"CRABFORGE_VERTEX_TEST_KEY","endpoints":{"vertex":vertex},"models":["gemini-2.5-pro","claude-test","org/partner"]},
        {"id":"adc","endpoints":{"vertex":{"project":"example-project","credentials":missing}},"models":["org/partner"]}],"groups":{"retry":["adc/org/partner","cloud/org/partner"]}});
    let key = config::token(root.path())?;
    fs::write(root.path().join("settings.json"), cfg.to_string())?;
    let mut gateway = Command::new(env!("CARGO_BIN_EXE_carbforge-cli"))
        .arg("--data-dir")
        .arg(root.path())
        .arg("serve")
        .env("CRABFORGE_VERTEX_TEST_KEY", "vertex-test")
        .env("NO_PROXY", "127.0.0.1")
        .kill_on_drop(true)
        .stderr(Stdio::piped())
        .spawn()?;
    let mut lines = BufReader::new(gateway.stderr.take().unwrap()).lines();
    let line = timeout(Duration::from_secs(15), lines.next_line())
        .await??
        .context("missing gateway address")?;
    let base = line
        .strip_prefix("HTTP ")
        .context("missing HTTP address")?
        .trim_end_matches("/v1");
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .build()?;
    for (endpoint, body, path) in [
        (
            "/v1beta/models/cloud/gemini-2.5-pro:streamGenerateContent",
            json!({"contents":[]}),
            "publishers/google/models/gemini-2.5-pro:streamGenerateContent?alt=sse",
        ),
        (
            "/v1/chat/completions",
            json!({"model":"group/retry","messages":[]}),
            "endpoints/openapi/chat/completions",
        ),
        (
            "/v1/messages",
            json!({"model":"cloud/claude-test","stream":true}),
            "publishers/anthropic/models/claude-test:streamRawPredict",
        ),
        (
            "/v1/messages",
            json!({"model":"cloud/claude-test","stream":false}),
            "publishers/anthropic/models/claude-test:rawPredict",
        ),
        (
            "/v1/messages/count_tokens",
            json!({"model":"cloud/claude-test","anthropic_version":"fixture"}),
            "publishers/anthropic/models/count-tokens:rawPredict",
        ),
    ] {
        let mut response = client
            .post(format!("{base}{endpoint}"))
            .bearer_auth(&key)
            .body(body.to_string())
            .send()
            .await?
            .error_for_status()?;
        let observed: Value = if path.contains(":stream") {
            assert_eq!(response.headers()["content-type"], "text/event-stream");
            let mut first = Vec::new();
            while !first.ends_with(b"\n\n") {
                first.extend_from_slice(&response.chunk().await?.context("missing SSE frame")?);
            }
            resume.notify_one();
            assert_eq!(response.bytes().await?.as_ref(), b"data: {}\n\n");
            serde_json::from_slice(first.strip_prefix(b"data: ").context("invalid SSE frame")?)?
        } else {
            serde_json::from_slice(&response.bytes().await?)?
        };
        assert_eq!(
            observed["uri"],
            format!("/v1/projects/example-project/locations/eu/{path}")
        );
        match endpoint {
            "/v1/messages" => {
                assert!(observed["body"].get("model").is_none());
                assert_eq!(observed["body"]["anthropic_version"], "vertex-2023-10-16");
            }
            "/v1/messages/count_tokens" => {
                assert_eq!(observed["body"]["model"], "claude-test");
                assert_eq!(observed["body"]["anthropic_version"], "fixture");
            }
            "/v1/chat/completions" => assert_eq!(observed["body"]["model"], "org/partner"),
            _ => assert_eq!(observed["body"], body),
        }
    }
    gateway.kill().await?;
    server.abort();
    let mut invalid = cfg;
    invalid["providers"][0]["key_env"] = json!("");
    assert!(config::Config::parse(&invalid.to_string()).is_err());
    Ok(())
}
