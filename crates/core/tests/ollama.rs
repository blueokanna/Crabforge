use anyhow::{Context, Result};
use crabforge_core::{config, service::Service};
use crabforge_test_support::server::{Body, Incoming, IntoResponse, Router, post};
use nextjson::{Value, json};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Notify};

#[tokio::test]
async fn ollama_routes_preserve_streamed_calls_usage_and_fallback() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let release = Arc::new(Notify::new());
    let signal = release.clone();
    let vendor = Router::new()
        .route("/broken/chat/completions", post(|_| async { ([("content-type","text/event-stream")], "data: {\"error\":\"early failure\"}\n\n") }))
        .route("/live/chat/completions", post(move |Incoming {headers, body: payload, ..}| { let req: Value = nextjson::from_slice(&payload).unwrap();
            let signal = signal.clone();
            async move {
                assert!(!headers.contains_key("authorization"));
                assert!(!headers.contains_key("x-crabforge-session") && !headers.contains_key("x-session-affinity"));
                assert_eq!(req["model"], json!("m"));
                assert_eq!(req["messages"][0]["tool_calls"][0]["function"]["arguments"], json!("{\"x\":1}"));
                assert_eq!(req["messages"][1]["tool_call_id"], req["messages"][0]["tool_calls"][0]["id"]);
                assert_eq!(req["response_format"]["type"],json!("json_object"));
                assert_eq!(req["max_tokens"],json!(12));
                assert!(req.get("options").is_none());
                let stream = async_stream::stream! {
                    let first = json!({"choices":[{"delta":{"content":"你好","reasoning_content":"hidden"}}]});
                    yield Ok::<_,Infallible>(format!("data: {first}\n\n"));
                    signal.notified().await;
                    for chunk in [
                        json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call1","function":{"name":"lookup","arguments":"{\"city\":"}}]}}]}),
                        json!({"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"上海\"}"}}]},"finish_reason":"tool_calls"}]}),
                        json!({"choices":[],"usage":{"prompt_tokens":8,"completion_tokens":3}}),
                    ] { yield Ok(format!("data: {chunk}\n\n")); }
                    yield Ok("data: [DONE]\n\n".to_owned());
                };
                ([("content-type","text/event-stream")],Body::from_stream(stream)).into_response()
            }
        }))
        .route("/cut/chat/completions", post(|_| async { ([("content-type","text/event-stream")],"data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n") }));
    let server =
        tokio::spawn(async move { crabforge_test_support::server::serve(listener, vendor).await });
    let root = tempfile::tempdir()?;
    let providers: Vec<_> = ["broken", "live", "cut"]
        .iter()
        .map(
            |id| json!({"id":id,"models":["m"],"endpoints":{"chat":format!("http://{addr}/{id}")}}),
        )
        .collect();
    let cfg = json!({"listen":"127.0.0.1:0","providers":providers,"groups":{"fallback":["broken/m","live/m"],"cut":["cut/m","live/m"]}});
    config::save(
        root.path(),
        &config::snapshot(root.path())?,
        &cfg.to_string(),
    )?;
    let service = Service::start(root.path()).await?;
    let key = config::token(root.path())?;
    let base = format!("http://{}/agent/vscode/api", service.addr);
    let http = crabforge_test_support::Client::with_timeout(Duration::from_secs(6));
    let post = |path: &str, v: Value| {
        http.post(format!("{base}/{path}"))
            .bearer_auth(&key)
            .body(v.to_string())
    };
    let unauthorized = http.get(format!("{base}/tags")).send().await?;
    assert_eq!(unauthorized.status(), 401);
    assert_eq!(
        decode(unauthorized).await?["error"],
        json!("gateway key required")
    );
    let tags = decode(
        http.get(format!("{base}/tags"))
            .bearer_auth(&key)
            .send()
            .await?,
    )
    .await?;
    assert!(
        tags["models"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["model"] == json!("group/fallback")
                && m["digest"].as_str().is_some_and(|s| s.len() == 64))
    );
    let show = decode(post("show", json!({"name":"m:latest"})).send().await?).await?;
    assert_eq!(show["model_info"]["general.basename"], json!("m"));
    assert_eq!(
        post("show", json!({"model":"missing"}))
            .send()
            .await?
            .status(),
        404
    );
    let load = decode(post("generate", json!({"model":"m:latest"})).send().await?).await?;
    assert_eq!(load["done_reason"], json!("load"));
    assert!(service.history.lock().unwrap().is_empty());
    let req = json!({"model":"group/fallback:latest","think":false,"format":"json","options":{"num_ctx":4096,"num_predict":12},"messages":[
        {"role":"assistant","tool_calls":[{"function":{"name":"sum","arguments":{"x":1}}}]},
        {"role":"tool","tool_name":"sum","content":"1"}]});
    let mut resp = post("chat", req)
        .header("x-crabforge-session", "gateway-session")
        .header("x-session-affinity", "native-session")
        .send()
        .await?
        .error_for_status()?;
    assert_eq!(resp.headers()["x-crabforge-session"], "gateway-session");
    assert_eq!(resp.headers()["content-type"], "application/x-ndjson");
    let mut first = Vec::new();
    while !first.ends_with(b"\n") {
        first.extend_from_slice(&resp.chunk().await?.context("missing first NDJSON line")?);
    }
    let row: Value = nextjson::from_slice(&first).map_err(anyhow::Error::msg)?;
    assert_eq!(row["message"], json!({"role":"assistant","content":"你好"}));
    release.notify_one();
    let tail: Vec<Value> = resp
        .text()
        .await?
        .lines()
        .map(nextjson::from_str)
        .collect::<Result<_, _>>()
        .map_err(anyhow::Error::msg)?;
    assert_eq!(tail.len(), 2);
    assert_eq!(
        tail[0]["message"]["tool_calls"][0]["function"]["arguments"],
        json!({"city":"上海"})
    );
    assert_eq!(tail[1]["done"], json!(true));
    {
        let history = service.history.lock().unwrap();
        assert_eq!(history.len(), 2);
        assert!(history.iter().all(|attempt| attempt.session.as_deref()
            == Some("gateway-session")
            && attempt.native_session.as_deref() == Some("native-session")));
    }
    assert_eq!(
        (
            tail[1]["prompt_eval_count"].as_u64(),
            tail[1]["eval_count"].as_u64()
        ),
        (Some(8), Some(3))
    );
    let cut = post("generate", json!({"model":"group/cut","prompt":"Hello"}))
        .send()
        .await?
        .text()
        .await?;
    let lines: Vec<Value> = cut
        .lines()
        .map(nextjson::from_str)
        .collect::<Result<_, _>>()
        .map_err(anyhow::Error::msg)?;
    assert_eq!(lines[0]["response"], json!("partial"));
    assert!(lines[1]["error"].as_str().is_some());
    let attempts: Vec<_> = service
        .history
        .lock()
        .unwrap()
        .iter()
        .map(|r| (r.provider.clone(), r.agent.clone(), r.status))
        .collect();
    assert_eq!(
        attempts,
        [
            ("broken".into(), Some("VS Code".into()), 502),
            ("live".into(), Some("VS Code".into()), 200),
            ("cut".into(), Some("VS Code".into()), 200)
        ]
    );
    service.stop().await;
    server.abort();
    Ok(())
}

async fn decode(response: crabforge_test_support::Response) -> Result<Value> {
    nextjson::from_slice(&response.bytes().await?).map_err(anyhow::Error::msg)
}
