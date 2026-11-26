use anyhow::{Context, Result};
use crabforge_core::config::{self, AuthHeader};
use crabforge_test_support::server::{Incoming, IntoResponse, Json, Router, StatusCode};
use nextjson::{Value, json};
use std::{
    fs,
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
    process::Command,
    time::timeout,
};

#[tokio::test]
async fn anthropic_relays_negotiate_replayable_requests_and_keep_explicit_choices() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let seen = Arc::new(Mutex::new(Vec::new()));
    let observed = seen.clone();
    let long_error = "invalid credentials ".repeat(4500);
    let diagnostic = long_error.clone();
    let app = Router::new().fallback(crabforge_test_support::server::any(
        move |Incoming {
                  uri,
                  headers: h,
                  body,
                  ..
              }| {
            let seen = observed.clone();
            let diagnostic = diagnostic.clone();
            async move {
                let api = h.get("x-api-key").is_some_and(|v| v == "provider-key");
                let bearer = h
                    .get("authorization")
                    .is_some_and(|v| v == "Bearer provider-key");
                let mode = match (api, bearer) {
                    (true, true) => "both",
                    (true, false) => "x-api-key",
                    (false, true) => "bearer",
                    _ => "missing",
                };
                seen.lock().unwrap().push((uri.path().to_string(), mode));
                if !body.is_empty() {
                    let sent: Value = nextjson::from_slice(&body).unwrap();
                    assert_eq!(sent["model"], json!("m"));
                    assert_eq!(sent["messages"][0]["content"], json!("hello"));
                }
                if uri.path().starts_with("/invalid/") {
                    return (StatusCode::UNAUTHORIZED, diagnostic).into_response();
                }
                if mode == "both" {
                    return (
                        StatusCode::UNAUTHORIZED,
                        "x-api-key 和 Authorization 不能同时提供",
                    )
                        .into_response();
                }
                if uri.path().starts_with("/single/") && mode != "x-api-key"
                    || uri.path().starts_with("/bearer/") && mode != "bearer"
                {
                    return StatusCode::FORBIDDEN.into_response();
                }
                Json(json!({"data":[{"id":"m"}],"has_more":false})).into_response()
            }
        },
    ));
    let upstream =
        tokio::spawn(async move { crabforge_test_support::server::serve(listener, app).await });
    let temp = tempfile::tempdir()?;
    let providers: Vec<_> = ["single", "bearer", "invalid", "explicit"]
        .into_iter()
        .map(|id| {
            let mut endpoints = json!({"anthropic":format!("http://{addr}/{id}")});
            if id == "explicit" {
                endpoints
                    .as_object_mut()
                    .unwrap()
                    .insert("auth_header".into(), json!("bearer"));
            }
            json!({"id":id,"endpoints":endpoints,"key_env":"CRABFORGE_AUTH_FIXTURE","models":["m"]})
        })
        .collect();
    fs::write(
        temp.path().join("settings.json"),
        json!({"listen":"127.0.0.1:0","providers":providers}).to_string(),
    )?;
    let cli = || {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_carbforge-cli"));
        cmd.arg("--data-dir")
            .arg(temp.path())
            .env("CRABFORGE_AUTH_FIXTURE", "provider-key")
            .env("NO_PROXY", "127.0.0.1");
        cmd
    };
    for options in [&["models", "single", "--sync"][..], &["models", "single"]] {
        let out = cli().args(options).output().await?;
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(String::from_utf8(out.stdout)?.trim(), "m");
    }
    let mut gateway = cli()
        .arg("serve")
        .kill_on_drop(true)
        .stderr(Stdio::piped())
        .spawn()?;
    let first = timeout(
        Duration::from_secs(15),
        BufReader::new(gateway.stderr.take().unwrap())
            .lines()
            .next_line(),
    )
    .await??
    .context("missing gateway address")?;
    let base = first
        .strip_prefix("HTTP ")
        .context("invalid gateway address")?;
    let token = config::token(temp.path())?;
    let client = crabforge_test_support::Client::default();
    for id in ["single", "bearer", "bearer", "explicit", "invalid"] {
        let body =
            json!({"model":format!("{id}/m"),"messages":[{"role":"user","content":"hello"}]})
                .to_string();
        let reply = client
            .post(format!("{base}/messages"))
            .bearer_auth(&token)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await?;
        if id == "invalid" {
            assert_eq!(reply.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(reply.text().await?, long_error);
        } else {
            reply.error_for_status()?;
        }
    }
    gateway.kill().await?;
    upstream.abort();
    let cfg = config::load(temp.path())?;
    assert_eq!(
        cfg.providers
            .iter()
            .map(|p| p.endpoints.auth_header)
            .collect::<Vec<_>>(),
        [
            AuthHeader::ApiKey,
            AuthHeader::Bearer,
            AuthHeader::Both,
            AuthHeader::Bearer
        ]
    );
    let seen = seen.lock().unwrap();
    let modes = |path: &str| {
        seen.iter()
            .filter_map(|(p, m)| (p == path).then_some(*m))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        modes("/single/v1/models"),
        ["both", "x-api-key", "x-api-key"]
    );
    assert_eq!(modes("/single/v1/messages"), ["x-api-key"]);
    assert_eq!(
        modes("/bearer/v1/messages"),
        ["both", "x-api-key", "bearer", "bearer"]
    );
    assert_eq!(modes("/explicit/v1/messages"), ["bearer"]);
    assert_eq!(modes("/invalid/v1/messages"), ["both"]);
    Ok(())
}
