use anyhow::Result;
use crabforge_core::{
    config::{self, Config, Endpoints, Provider},
    peer,
    service::Service,
};
use crabforge_test_support::server::{Body, Bytes, Incoming, Router, StatusCode, post};
use futures_util::{StreamExt, stream};
use nextjson::{Value, json};
use std::{collections::BTreeMap, convert::Infallible, net::SocketAddr, time::Duration};
use tempfile::tempdir;
use tokio::{net::TcpListener, sync::watch, time::timeout};

fn ephemeral() -> SocketAddr {
    ([127, 0, 0, 1], 0).into()
}

#[tokio::test]
async fn native_endpoints_follow_models_aliases_and_enabled_group_members() -> Result<()> {
    let root = tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0","providers":[
        {"id":"both","models":["gpt-5"],"endpoints":{"responses":"https://api.openai.com/v1","chat":"https://api.openai.com/v1"}},
        {"id":"response","models":["r"],"endpoints":{"responses":"http://127.0.0.1:1/v1"}},
        {"id":"text","models":["claude-a"],"endpoints":{"chat":"http://127.0.0.1:1/v1","anthropic":"http://127.0.0.1:1"}}],
        "groups":{"pool":["both/gpt-5","response/r"],"paused":["text/claude-a"],"outer":["group/pool","group/paused","text/claude-a"],"mixed":["group/pool","text/claude-a"]},
        "group_off":{"outer":["text/claude-a"]},"disabled_groups":["paused"],"former_groups":{"old":"outer"}});
    config::save(
        root.path(),
        &config::snapshot(root.path())?,
        &cfg.to_string(),
    )?;
    let svc = Service::start(root.path()).await?;
    let http = crabforge_test_support::Client::default();
    let key = config::token(root.path())?;
    let url = format!("http://{}/v1/models", svc.addr);
    let get = |url: String| http.get(url).bearer_auth(&key);
    let catalog: Value = nextjson::from_slice(
        &get(url.clone())
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?,
    )
    .map_err(anyhow::Error::msg)?;
    for (id, paths, preferred) in [
        (
            "gpt-5",
            json!(["/v1/chat/completions", "/v1/responses"]),
            json!("/v1/responses"),
        ),
        (
            "text/claude-a",
            json!(["/v1/chat/completions", "/v1/messages"]),
            json!("/v1/messages"),
        ),
        (
            "group/outer",
            json!(["/v1/responses"]),
            json!("/v1/responses"),
        ),
        (
            "group/old",
            json!(["/v1/responses"]),
            json!("/v1/responses"),
        ),
        ("group/mixed", json!([]), Value::Null),
    ] {
        let detail: Value = nextjson::from_slice(
            &get(format!("{url}/{id}"))
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?,
        )
        .map_err(anyhow::Error::msg)?;
        assert_eq!(detail["native_endpoints"], paths, "{id}");
        assert_eq!(
            detail.get("magpie_prefers"),
            (!preferred.is_null()).then_some(&preferred),
            "{id}"
        );
        let listed = catalog["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["id"] == json!(id));
        assert_eq!(listed, (id != "group/old").then_some(&detail));
    }
    svc.stop().await;
    Ok(())
}

#[tokio::test]
async fn rotating_groups_share_turns_only_among_compatible_members() -> Result<()> {
    let listener = TcpListener::bind(ephemeral()).await?;
    let upstream = listener.local_addr()?;
    let app = Router::new().route(
        "/{provider}/embeddings",
        post(|Incoming { param: id, .. }| async move {
            let status = if id == "busy" {
                StatusCode::TOO_MANY_REQUESTS
            } else {
                StatusCode::OK
            };
            (status, id)
        }),
    );
    let server =
        tokio::spawn(async move { crabforge_test_support::server::serve(listener, app).await });
    let root = tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0","providers":[
       {"id":"a","models":["m"],"endpoints":{"chat":format!("http://{upstream}/a")}},
       {"id":"b","models":["m"],"endpoints":{"anthropic":format!("http://{upstream}/b")}},
       {"id":"c","models":["m"],"endpoints":{"chat":format!("http://{upstream}/c")}},
       {"id":"d","models":["m"],"endpoints":{"chat":format!("http://{upstream}/d")}},
       {"id":"busy","models":["m"],"endpoints":{"chat":format!("http://{upstream}/busy")}}],
       "groups":{"pool":["a/m","b/m","c/m"],"weighted":["a/m","b/m","c/m"],
                 "fallback":["a/m","busy/m","c/m"], "outer":["group/first","group/second","busy/m","c/m"],
                 "first":["busy/m","a/m"],"second":["b/m","c/m"],
                 "nested":["group/inner","d/m"],"inner":["a/m","c/m"],
                 "nested-weight":["group/inner-weight","d/m"],"inner-weight":["a/m","c/m"]},
       "rotating_groups":["pool","outer","nested","inner"], "group_weights":{"weighted":{"a/m":3,"b/m":50},"fallback":{"busy/m":1000},"nested-weight":{"group/inner-weight":3},"inner-weight":{"c/m":2}}});
    config::save(
        root.path(),
        &config::snapshot(root.path())?,
        &cfg.to_string(),
    )?;
    let svc = Service::start(root.path()).await?;
    let http = crabforge_test_support::Client::default();
    let key = config::token(root.path())?;
    let request = |group: &str| {
        http.post(format!("http://{}/v1/embeddings", svc.addr))
            .bearer_auth(&key)
            .body(json!({"model":format!("group/{group}"), "input":"sample"}).to_string())
    };
    for (group, sequence) in [
        ("pool", ["a", "c", "a", "c"]),
        ("weighted", ["a", "a", "c", "a"]),
        ("outer", ["a", "c", "a", "c"]),
        ("nested", ["a", "d", "c", "d"]),
        ("nested-weight", ["c", "a", "d", "c"]),
    ] {
        for expected in sequence {
            let reply = request(group).send().await?.error_for_status()?;
            assert_eq!(reply.text().await?, expected);
        }
    }
    let replies = futures_util::future::try_join_all((0..16).map(|_| {
        let req = request("weighted");
        async move { req.send().await?.error_for_status()?.text().await }
    }))
    .await?;
    assert_eq!(replies.iter().filter(|id| *id == "a").count(), 12);
    assert_eq!(replies.iter().filter(|id| *id == "c").count(), 4);
    assert_eq!(
        request("fallback")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?,
        "a"
    );
    {
        let history = svc.history.lock().unwrap();
        let attempts: Vec<_> = history
            .iter()
            .filter(|a| a.model == "group/fallback")
            .map(|a| (a.provider.as_str(), a.status))
            .collect();
        assert_eq!(attempts, [("busy", 429), ("a", 200)]);
        let nested: Vec<_> = history
            .iter()
            .filter(|a| a.model == "group/outer")
            .map(|a| (a.provider.as_str(), a.via[0].as_str()))
            .collect();
        assert_eq!(
            nested,
            [
                ("busy", "group/first"),
                ("a", "group/first"),
                ("c", "group/second")
            ]
            .repeat(2)
        );
    }
    svc.stop().await;
    server.abort();
    Ok(())
}

#[test]
fn invalid_nested_group_edits_preserve_the_saved_graph() -> Result<()> {
    let dir = tempdir()?;
    let mut cfg = json!({"providers":[{"id":"p","models":["m"],"endpoints":{"chat":"http://localhost:8080"}}],"groups":{"g0":["p/m"]}});
    for i in 1..=8 {
        cfg.get_mut("groups")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(format!("g{i}"), json!([format!("group/g{}", i - 1)]));
    }
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let saved = config::snapshot(dir.path())?;
    for (id, member) in [
        ("g0", "group/g8"),
        ("g8", "group/g8"),
        ("g9", "group/g8"),
        ("g2", "group/missing"),
    ] {
        let mut invalid = cfg.clone();
        invalid
            .get_mut("groups")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(id.into(), json!([member]));
        assert!(config::save(dir.path(), &saved, &invalid.to_string()).is_err());
        assert_eq!(config::snapshot(dir.path())?, saved);
    }
    Ok(())
}

#[tokio::test]
async fn route_group_streams_first_chunk_before_upstream_finishes() -> Result<()> {
    let (release, gate) = watch::channel(false);
    let mock=Router::new().route("/busy/chat/completions",post(|_|async{StatusCode::TOO_MANY_REQUESTS}))
        .route("/ready/chat/completions",post(move |Incoming {headers, body, ..}|{
            let mut gate=gate.clone();async move {
                assert!(!headers.contains_key("authorization"));
                assert!(!headers.contains_key("cookie"));
                assert_eq!(nextjson::from_slice::<Value>(&body).unwrap(),json!({"model":"family/m","messages":[{"role":"user","content":"group/test"}],"stream":true}));
                let first=stream::once(async{Ok::<_,Infallible>(Bytes::from_static(b"data: first\n\n"))});
                let last=stream::once(async move {gate.wait_for(|v|*v).await.unwrap(); Ok(Bytes::from_static(b"data: [DONE]\n\n"))});
                ([("content-type","text/event-stream")],Body::from_stream(first.chain(last)))
            }
        }));
    let listener = TcpListener::bind(ephemeral()).await?;
    let upstream = listener.local_addr()?;
    let upstream_task = tokio::spawn(async move {
        crabforge_test_support::server::serve(listener, mock)
            .await
            .unwrap()
    });
    let dir = tempdir()?;
    let providers = ["busy", "ready"].map(|id| Provider {
        fallback: vec![],
        patience: -1,
        no_stand_in: false,
        model_protocols: Default::default(),
        free_models: Default::default(),
        catalog: String::new(),
        web_site: None,
        id: id.into(),
        endpoints: Endpoints {
            chat: format!("http://{upstream}/{id}"),
            ..Default::default()
        },
        key_env: String::new(),
        models: vec!["family/m".into()],
    });
    let cfg = Config {
        listen: ephemeral(),
        providers: providers.into(),
        groups: BTreeMap::from([(
            "test".into(),
            vec!["busy/family/m".into(), "ready/family/m".into()],
        )]),
        ..Default::default()
    };
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_json()?)?;
    let service = Service::start(dir.path()).await?;
    let client = crabforge_test_support::Client::default();
    let url = format!("http://{}/v1/chat/completions", service.addr);
    let key = config::token(dir.path())?;
    assert_eq!(client.post(&url).send().await?.status(), 401);
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(&key)
            .header("Origin", "null")
            .send()
            .await?
            .status(),
        401
    );
    let mut response=client.post(url).bearer_auth(key).header("cookie","private").body(r#"{"model":"group/test","messages":[{"role":"user","content":"group/test"}],"stream":true}"#).send().await?;
    assert_eq!(response.status(), 200);
    assert_eq!(
        timeout(Duration::from_secs(5), response.chunk())
            .await??
            .unwrap(),
        "data: first\n\n"
    );
    release.send(true)?;
    assert_eq!(response.bytes().await?, "data: [DONE]\n\n");
    assert_eq!(
        service
            .history
            .lock()
            .unwrap()
            .iter()
            .map(|a| a.status)
            .collect::<Vec<_>>(),
        [429, 200]
    );
    service.stop().await;
    upstream_task.abort();
    let _ = upstream_task.await;
    Ok(())
}

#[tokio::test]
async fn group_first_content_deadline_holds_preludes_and_respects_nested_overrides() -> Result<()> {
    const SLOW_PRELUDE: &str = "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\n";
    const FAST_PRELUDE: &str =
        ": ready\r\n\r\ndata: {\"choices\":[{\"delta\":{\"content\":\"你好\"}}]}\r\n\r\n";
    const DONE: &str = "data: [DONE]\n\n";
    const JSON_REPLY: &str = r#"{"choices":[{"message":{"content":"complete"}}]}"#;
    let (release_slow, slow) = watch::channel(false);
    let (release_fast, fast) = watch::channel(false);
    let mock = Router::new()
        .route(
            "/slow/chat/completions",
            post(move |_| {
                let mut slow = slow.clone();
                async move {
                    let prelude = stream::once(async {
                        Ok::<_, Infallible>(Bytes::from_static(SLOW_PRELUDE.as_bytes()))
                    });
                    let done = stream::once(async move {
                        slow.wait_for(|v| *v).await.unwrap();
                        Ok(Bytes::from_static(DONE.as_bytes()))
                    });
                    (
                        [("content-type", "text/event-stream")],
                        Body::from_stream(prelude.chain(done)),
                    )
                }
            }),
        )
        .route(
            "/ready/chat/completions",
            post(move |_| {
                let mut fast = fast.clone();
                async move {
                    let bytes = FAST_PRELUDE.as_bytes();
                    let split = FAST_PRELUDE.find('你').unwrap() + 1;
                    let chunks = stream::iter([
                        Ok::<_, Infallible>(Bytes::from_static(&bytes[..split])),
                        Ok(Bytes::from_static(&bytes[split..])),
                    ]);
                    let done = stream::once(async move {
                        fast.wait_for(|v| *v).await.unwrap();
                        Ok(Bytes::from_static(DONE.as_bytes()))
                    });
                    (
                        [("content-type", "text/event-stream; charset=utf-8")],
                        Body::from_stream(chunks.chain(done)),
                    )
                }
            }),
        )
        .route(
            "/busy/chat/completions",
            post(|_| async { StatusCode::SERVICE_UNAVAILABLE }),
        )
        .route(
            "/json/chat/completions",
            post(|_| async {
                tokio::time::sleep(Duration::from_millis(1100)).await;
                ([("content-type", "application/json")], JSON_REPLY)
            }),
        );
    let listener = TcpListener::bind(ephemeral()).await?;
    let addr = listener.local_addr()?;
    let upstream = tokio::spawn(crabforge_test_support::server::serve(listener, mock));
    let dir = tempdir()?;
    let cfg = json!({
        "listen":"127.0.0.1:0",
        "providers":[
            {"id":"slow","models":["m"],"endpoints":{"chat":format!("http://{addr}/slow")}},
            {"id":"ready","models":["m"],"endpoints":{"chat":format!("http://{addr}/ready")}},
            {"id":"busy","models":["m"],"endpoints":{"chat":format!("http://{addr}/busy")}},
            {"id":"other","models":["m"],"endpoints":{"chat":format!("http://{addr}/ready")}},
            {"id":"json","models":["m"],"endpoints":{"chat":format!("http://{addr}/json")}}
        ],
        "groups":{
            "timed":["group/inherits","ready/m","group/rotating"],
            "inherits":["slow/m"],
            "wait":["group/untimed","ready/m"],
            "untimed":["slow/m"],
            "last":["busy/m","slow/m"],
            "rotating":["busy/m","other/m"],
            "json":["json/m","ready/m"]
        },
        "rotating_groups":["rotating"],
        "default_group_first_content":1,
        "group_first_content":{"untimed":-1}
    });
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let client = crabforge_test_support::Client::default();
    let url = format!("http://{}/v1/chat/completions", service.addr);
    let token = config::token(dir.path())?;
    let send = |group: &str, streaming: bool| {
        client
            .post(&url)
            .bearer_auth(&token)
            .body(
                json!({
                    "model":group,"messages":[{"role":"user","content":"hello"}],"stream":streaming
                })
                .to_string(),
            )
            .send()
    };
    let mut response = timeout(Duration::from_secs(5), send("group/timed", true)).await??;
    assert_eq!(response.status(), 200);
    assert_eq!(
        timeout(Duration::from_secs(3), response.chunk())
            .await??
            .unwrap(),
        FAST_PRELUDE
    );
    let json = timeout(Duration::from_secs(5), send("group/json", false)).await??;
    assert_eq!(json.status(), 200);
    assert_eq!(json.text().await?, JSON_REPLY);
    release_fast.send(true)?;
    assert_eq!(
        timeout(Duration::from_secs(3), response.bytes()).await??,
        DONE
    );

    // Inspecting the nested fallback must leave its first turn intact.
    let rotated = timeout(Duration::from_secs(5), send("group/rotating", true)).await??;
    assert_eq!(rotated.status(), 200);
    assert_eq!(rotated.text().await?, format!("{FAST_PRELUDE}{DONE}"));

    // Both requests expose their prelude while the slow upstream is still waiting.
    let mut waits = Vec::new();
    for group in ["group/wait", "group/last"] {
        let mut response = timeout(Duration::from_secs(5), send(group, true)).await??;
        assert_eq!(response.status(), 200);
        assert_eq!(
            timeout(Duration::from_secs(3), response.chunk())
                .await??
                .unwrap(),
            SLOW_PRELUDE
        );
        waits.push(response);
    }
    release_slow.send(true)?;
    for response in waits {
        assert_eq!(
            timeout(Duration::from_secs(3), response.bytes()).await??,
            DONE
        );
    }
    assert_eq!(
        service
            .history
            .lock()
            .unwrap()
            .iter()
            .map(|a| (a.provider.as_str(), a.status))
            .collect::<Vec<_>>(),
        [
            ("slow", 504),
            ("ready", 200),
            ("json", 200),
            ("busy", 503),
            ("other", 200),
            ("slow", 200),
            ("busy", 503),
            ("slow", 200)
        ]
    );
    service.stop().await;
    upstream.abort();
    Ok(())
}

#[tokio::test]
async fn patience_retries_busy_members_within_group_or_direct_provider_budgets() -> Result<()> {
    use std::sync::{Arc, Mutex};
    use tokio::time::Instant;

    let calls = Arc::new(Mutex::new(Vec::new()));
    let attempts = calls.clone();
    let mock = Router::new().route(
        "/v1/chat/completions",
        post(move |request: Incoming| {
            let attempts = attempts.clone();
            async move {
                let request: Value = nextjson::from_slice(&request.body).unwrap();
                let model = request["model"].as_str().unwrap();
                let mut calls = attempts.lock().unwrap();
                let count = calls.iter().filter(|(name, _)| name == model).count();
                calls.push((model.to_owned(), Instant::now()));
                let (status, delay, body) = match (model, count) {
                    ("first", _) => (StatusCode::TOO_MANY_REQUESTS, "2", "busy"),
                    ("second", 0 | 1) => (StatusCode::SERVICE_UNAVAILABLE, "0", "busy"),
                    ("second", _) => (StatusCode::OK, "0", "ready"),
                    ("direct", 0 | 1) | ("legacy", _) => {
                        (StatusCode::SERVICE_UNAVAILABLE, "0", "busy")
                    }
                    ("quota", _) => (StatusCode::TOO_MANY_REQUESTS, "0", "insufficient_quota"),
                    ("future", _) => (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Sat, 01 Jan 2050 00:00:00 GMT",
                        "future retry",
                    ),
                    _ => (StatusCode::OK, "0", "ready"),
                };
                (status, [("retry-after", delay)], body)
            }
        }),
    );
    let listener = TcpListener::bind(ephemeral()).await?;
    let addr = listener.local_addr()?;
    let upstream = tokio::spawn(crabforge_test_support::server::serve(listener, mock));
    let dir = tempdir()?;
    let cfg = json!({
        "listen":"127.0.0.1:0",
        "providers":[
            {"id":"fixture","models":["first","second","quota","future","direct"],
             "patience":3,"endpoints":{"chat":format!("http://{addr}/v1")}},
            {"id":"legacy","models":["legacy"],"endpoints":{"chat":format!("http://{addr}/v1")}}
        ],
        "groups":{
            "pool":["fixture/first","fixture/second"],
            "off":["group/no-retry"],
            "no-retry":["fixture/second"],
            "direct-off":["fixture/direct"],
            "quota":["fixture/quota"],
            "future":["fixture/future"]
        },
        "default_group_patience":3,
        "group_patience":{"no-retry":-1,"direct-off":-1}
    });
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let url = format!("http://{}/v1/chat/completions", service.addr);
    let token = config::token(dir.path())?;
    let client = crabforge_test_support::Client::with_timeout(Duration::from_secs(5));
    let send = |model: &str| {
        client
            .post(&url)
            .bearer_auth(&token)
            .body(json!({"model":model,"messages":[{"role":"user","content":"hello"}]}).to_string())
            .send()
    };

    let off = send("group/off").await?;
    assert_eq!(off.status(), 503);
    assert_eq!(off.text().await?, "busy");
    let exhausted = send("group/quota").await?;
    assert_eq!(exhausted.status(), 429);
    assert_eq!(exhausted.text().await?, "insufficient_quota");
    let future = send("group/future").await?;
    assert_eq!(future.status(), 503);
    assert_eq!(future.text().await?, "future retry");
    let retried = send("group/pool").await?;
    assert_eq!(retried.status(), 200);
    assert_eq!(retried.text().await?, "ready");
    assert_eq!(send("group/direct-off").await?.status(), 503);
    let direct = send("fixture/direct").await?;
    assert_eq!(direct.status(), 200);
    assert_eq!(direct.text().await?, "ready");
    assert_eq!(send("legacy/legacy").await?.status(), 503);
    let calls = calls.lock().unwrap().clone();
    assert_eq!(
        calls
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        [
            "second", "quota", "future", "first", "second", "first", "second", "direct", "direct",
            "direct", "legacy"
        ],
    );
    assert!(calls[5].1.duration_since(calls[4].1) >= Duration::from_secs(2));
    assert!(calls[9].1.duration_since(calls[8].1) >= Duration::from_secs(2));
    assert_eq!(
        service
            .history
            .lock()
            .unwrap()
            .iter()
            .map(|attempt| attempt.status)
            .collect::<Vec<_>>(),
        [503, 429, 503, 429, 503, 429, 200, 503, 503, 200, 503],
    );
    service.stop().await;
    upstream.abort();
    Ok(())
}

#[tokio::test]
async fn group_rules_order_compatible_members_and_respect_pauses_and_manual_picks() -> Result<()> {
    let mock = Router::new().route(
        "/v1/chat/completions",
        post(|request: Incoming| async move {
            let body: Value = nextjson::from_slice(&request.body).unwrap();
            let model = body["model"].as_str().unwrap();
            let status = if model == "busy" {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::OK
            };
            (status, model.to_owned())
        }),
    );
    let listener = TcpListener::bind(ephemeral()).await?;
    let addr = listener.local_addr()?;
    let upstream = tokio::spawn(crabforge_test_support::server::serve(listener, mock));
    let dir = tempdir()?;
    let cfg = json!({
        "listen":"127.0.0.1:0",
        "providers":[{"id":"fixture","models":["usual","busy","second","paused"],
            "endpoints":{"chat":format!("http://{addr}/v1")}}],
        "groups":{
            "inner":["fixture/usual","fixture/paused","fixture/second","fixture/busy"],
            "outer":["group/inner"],
            "manual":["fixture/usual","fixture/paused"],
            "closed":["fixture/paused"]
        },
        "rotating_groups":["inner"],
        "group_picks":{"manual":"fixture/paused"},
        "group_rules":{
            "inner":[
                {"use":"fixture/paused","pause":true,"agents":["claude"]},
                {"use":"fixture/busy","more":["fixture/paused","fixture/second"],"folders":["C:/work/repo"],
                 "where":[{"field":"message","op":"matches","value":"/^review\\b/"},
                    {"field":"effort","op":"gte","value":"medium"},
                    {"field":"subagent","op":"not"}]},
                {"use":"fixture/second","images":true}
            ],
            "manual":[{"use":"fixture/paused","pause":true,"agents":["claude"]}],
            "closed":[{"use":"fixture/paused","pause":true,"agents":["claude"]}]
        }
    });
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let url = format!("http://{}/v1/chat/completions", service.addr);
    let token = config::token(dir.path())?;
    let client = crabforge_test_support::Client::default();
    let send = |group: &str, user: Value, subagent: bool| {
        let mut request = client
            .post(&url)
            .bearer_auth(&token)
            .header("user-agent", "claude-cli/2.1.332");
        if subagent {
            request = request.header("x-claude-code-agent-id", "reviewer");
        }
        request.body(json!({"model":group,"reasoning_effort":"high","messages":[
            {"role":"developer","content":"<env>\nWorking directory: /mnt/c/work/repo/src\n</env>"},
            {"role":"user","content":user}
        ]}).to_string()).send()
    };
    let response = send("group/outer", json!("Review the result"), false).await?;
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await?, "second");
    let response = send("group/outer", json!("Review the result"), true).await?;
    assert_eq!(response.text().await?, "usual");
    let image =
        json!([{"type":"image_url","image_url":{"url":"https://example.invalid/image.png"}}]);
    assert_eq!(
        send("group/outer", image, false).await?.text().await?,
        "second"
    );
    assert_eq!(
        send("group/manual", json!("hello"), false)
            .await?
            .text()
            .await?,
        "paused"
    );
    assert_eq!(
        send("group/closed", json!("hello"), false).await?.status(),
        503
    );
    assert_eq!(
        service
            .history
            .lock()
            .unwrap()
            .iter()
            .map(|attempt| attempt.status)
            .collect::<Vec<_>>(),
        [503, 200, 200, 200, 200]
    );
    service.stop().await;
    upstream.abort();
    Ok(())
}

#[tokio::test]
async fn intent_classifier_is_called_only_on_reached_groups_and_uses_backups() -> Result<()> {
    let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen = calls.clone();
    let mock = Router::new().route(
        "/v1/chat/completions",
        post(move |request: Incoming| {
            let seen = seen.clone();
            async move {
                let body: Value = nextjson::from_slice(&request.body).unwrap();
                let model = body["model"].as_str().unwrap();
                seen.lock().unwrap().push(model.to_owned());
                match model {
                    "slow" => {
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        (StatusCode::SERVICE_UNAVAILABLE, String::new())
                    }
                    "decide" => {
                        assert_eq!(body["stream"].as_bool(), Some(false));
                        let messages = body["messages"].as_array().unwrap();
                        assert_eq!(messages.len(), 2);
                        let system = messages[0]["content"].as_str().unwrap();
                        assert!(system.starts_with("Judge which task was requested."));
                        let prompt: Value =
                            nextjson::from_str(messages[1]["content"].as_str().unwrap()).unwrap();
                        assert_eq!(
                            prompt["previous_user_message"].as_str(),
                            Some("Open the report")
                        );
                        assert_eq!(
                            prompt["previous_assistant_reply"].as_str(),
                            Some("It is open")
                        );
                        assert_eq!(
                            prompt["kinds"],
                            json!([{"number":1,"intent":"Review report"}])
                        );
                        let label = if prompt["message"].as_str() == Some("Review it") {
                            "1"
                        } else {
                            "0"
                        };
                        (
                            StatusCode::OK,
                            json!({"choices":[{"message":{"content":label}}]}).to_string(),
                        )
                    }
                    _ => (StatusCode::OK, model.to_owned()),
                }
            }
        }),
    );
    let listener = TcpListener::bind(ephemeral()).await?;
    let addr = listener.local_addr()?;
    let upstream = tokio::spawn(crabforge_test_support::server::serve(listener, mock));
    let dir = tempdir()?;
    let cfg = json!({
        "listen":"127.0.0.1:0",
        "classify_timeout":2,
        "providers":[{"id":"fixture","models":["slow","decide","usual","review"],
            "endpoints":{"chat":format!("http://{addr}/v1")}}],
        "groups":{
            "outer":["fixture/usual","group/intent"],
            "intent":["fixture/usual","fixture/review"],
            "classifier":["fixture/decide"],
            "manual":["fixture/usual","fixture/review"]
        },
        "group_picks":{"manual":"fixture/usual"},
        "group_rules":{
            "intent":[{"use":"fixture/review","intent":"Review report"}],
            "manual":[{"use":"fixture/review","intent":"Review report"}]
        },
        "group_classifiers":{
            "intent":{"classifier":"fixture/slow","classifierMore":["group/classifier"],"classifyPrompt":"Judge which task was requested."},
            "manual":{"classifier":"fixture/decide"}
        }
    });
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let url = format!("http://{}/v1/chat/completions", service.addr);
    let token = config::token(dir.path())?;
    let client = crabforge_test_support::Client::with_timeout(Duration::from_secs(5));
    let send = |group: &str, message: &str| {
        client
            .post(&url)
            .bearer_auth(&token)
            .body(
                json!({
                    "model":group,"messages":[
                        {"role":"system","content":"Internal system instructions"},
                        {"role":"user","content":"Open the report"},
                        {"role":"assistant","content":"It is open"},
                        {"role":"user","content":message}
                    ]
                })
                .to_string(),
            )
            .send()
    };
    assert_eq!(
        send("group/outer", "Review it").await?.text().await?,
        "usual"
    );
    assert_eq!(
        send("group/manual", "Review it").await?.text().await?,
        "usual"
    );
    assert_eq!(
        send("group/intent", "Review it").await?.text().await?,
        "review"
    );
    assert_eq!(send("group/intent", "Thanks").await?.text().await?, "usual");
    assert_eq!(
        *calls.lock().unwrap(),
        [
            "usual", "usual", "slow", "decide", "review", "slow", "decide", "usual"
        ]
    );

    let config = config::load(dir.path())?;
    let shared = config.share_group("intent")?;
    let (restored, report) = config.import_group(&shared, false)?;
    assert_eq!(
        restored.group_classifiers[&report.id].classifier_more,
        ["group/classifier"]
    );
    assert_eq!(restored.group_rules[&report.id][0].intent, "Review report");
    service.stop().await;
    upstream.abort();
    Ok(())
}

#[tokio::test]
async fn quic_peer_identity_and_gateway_key_are_both_required() -> Result<()> {
    let server_dir = tempdir()?;
    let client_dir = tempdir()?;
    let client_key = peer::identity(client_dir.path())?;
    let allowed = client_key.public().to_peer_id();
    assert_eq!(
        peer::identity(client_dir.path())?.public().to_peer_id(),
        allowed
    );
    let cfg = Config {
        listen: ephemeral(),
        peer_listen: "/ip4/127.0.0.1/udp/0/quic-v1".into(),
        allowed_peers: vec![allowed.to_string()],
        ..Default::default()
    };
    config::save(
        server_dir.path(),
        &config::snapshot(server_dir.path())?,
        &cfg.to_json()?,
    )?;
    let service = Service::start(server_dir.path()).await?;
    let address = service.peer.as_ref().unwrap().address.to_string();
    let (local, task) = peer::tunnel(client_key, &address, ephemeral()).await?;
    let url = format!("http://{local}/v1/models");
    let http = crabforge_test_support::Client::with_timeout(Duration::from_secs(5));
    assert_eq!(http.get(&url).send().await?.status(), 401);
    let response = http
        .get(&url)
        .bearer_auth(config::token(server_dir.path())?)
        .send()
        .await?;
    assert_eq!(response.status(), 200);
    assert_eq!(
        nextjson::from_slice::<Value>(&response.bytes().await?).map_err(anyhow::Error::msg)?,
        json!({"object":"list","data":[]})
    );
    let stranger = libp2p::identity::Keypair::generate_ed25519();
    if let Ok((untrusted, denied)) = peer::tunnel(stranger, &address, ephemeral()).await {
        // The dialer can complete TLS just before the listener rejects its identity.
        let response = http
            .get(format!("http://{untrusted}/v1/models"))
            .bearer_auth(config::token(server_dir.path())?)
            .send()
            .await;
        assert!(response.is_err());
        denied.abort();
        let _ = denied.await;
    }
    task.abort();
    let _ = task.await;
    service.stop().await;
    Ok(())
}

#[test]
fn concurrent_edit_and_corrupt_file_preserve_disk_state() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path();
    let old = config::snapshot(path)?;
    let changed = Config {
        listen: ephemeral(),
        ..Default::default()
    }
    .to_json()?;
    config::save(path, &old, &changed)?;
    assert!(config::save(path, &old, &old).is_err());
    assert_eq!(config::load(path)?.listen, ephemeral());
    std::fs::write(path.join("settings.json"), b"{")?;
    assert!(config::save(path, &old, &old).is_err());
    assert_eq!(std::fs::read(path.join("settings.json"))?, b"{");
    Ok(())
}
