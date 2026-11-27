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
async fn decision_apis_preserve_native_requests_and_translate_group_fallbacks() -> Result<()> {
    use crabforge_test_support::server::{IntoResponse, Json};
    let listener = TcpListener::bind(ephemeral()).await?;
    let addr = listener.local_addr()?;
    let (seen, mut requests) = tokio::sync::mpsc::unbounded_channel();
    let app = Router::new().observe(seen)
        .route("/native/decisions", post(|Incoming {body, ..}: Incoming| async move {
            let value: Value = nextjson::from_slice(&body).unwrap();
            match value["model"].as_str().unwrap() {
                "luna" if value.get("prompt_cache_key").is_some() => {
                    assert_eq!(&body[..], br#"{ "model" : "luna", "input": "direct", "questions": [{"type":"predicate"}], "prompt_cache_key": "keep" }"#);
                    (StatusCode::OK, "{\"native\":true}").into_response()
                }
                "luna" => {
                    assert_eq!(value["questions"][0]["name"], json!("z"));
                    assert_eq!(value["questions"][1]["name"], json!("a"));
                    assert_eq!(value["questions"][1]["choices"][0]["value"], json!("second"));
                    assert_eq!(value["questions"][2]["levels"][0]["label"], json!("small"));
                    assert_eq!(value.pointer("/input/0/content/0/data/document"), Some(&json!(123)));
                    assert_eq!(value.pointer("/input/0/content/1/image_url"), Some(&json!("https://example.org/receipt.png")));
                    ([("x-request-id", "fixture"), ("set-cookie", "private=1")], Json(json!({"model":"luna","answers":[
                        {"type":"choice","name":"a","choice":"first","confidence":0.8,"probabilities":[{"value":"first","probability":0.9}]},
                        {"type":"predicate","name":"z","probability":0.75},
                        {"type":"score","name":"work","score":0.9,"probabilities":[{"label":"small","probability":0.1},{"label":"large","probability":0.9}]}
                    ],"usage":{"input_tokens":30,"output_tokens":4}}))).into_response()
                }
                "busy" => (StatusCode::TOO_MANY_REQUESTS, [("retry-after", "7")], "busy").into_response(),
                model => panic!("unexpected model {model}"),
            }
        }))
        .route("/fallback/systemone", post(|Incoming {body, ..}: Incoming| async move {
            let value: Value = nextjson::from_slice(&body).unwrap();
            assert_eq!(value["model"], json!("jev"));
            if value["state"].as_str() == Some("direct") {
                assert_eq!(value.get("vendor_extension"), Some(&json!([1, 2])));
                return Json(json!({"native":true}));
            }
            assert_eq!(value.pointer("/state/messages/0/content"), Some(&json!("A receipt\n{\"total\":42}")));
            assert_eq!(value.pointer("/state/messages/1/refusal"), Some(&json!("Unable to read it")));
            assert_eq!(value["images"], json!(["data:image/png;base64,AAAA"]));
            assert_eq!(value.pointer("/questions/question_2/type"), Some(&json!("noul")));
            assert_eq!(value.pointer("/questions/choice/criteria/true"), Some(&json!("Approved")));
            assert_eq!(value.pointer("/questions/work/criteria/1"), Some(&json!("large: Extensive")));
            assert!(value.get("prompt_cache_key").is_none());
            Json(json!({"answers":{
                "question_2":{"noul":0.25},"choice":{"choice":"true","probabilities":{"true":0.9,"false":0.1}},
                "work":{"score":0.8,"probabilities":{"large: Extensive":0.8,"0":0.2}}
            },"usage":{"input_tokens":12,"output_tokens":3}}))
        }));
    let server = tokio::spawn(crabforge_test_support::server::serve(listener, app));
    let dir = tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0","providers":[
        {"id":"openai","models":["luna","busy"],"endpoints":{"decisions":format!("http://{addr}/native/decisions/")}},
        {"id":"jev","models":["jev"],"endpoints":{"systemone":format!("http://{addr}/fallback")}},
        {"id":"chat","models":["m"],"endpoints":{"chat":format!("http://{addr}/chat/v1")}}
    ],"groups":{"judges":["chat/m","openai/busy","jev/jev"]}});
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let gateway = Service::start(dir.path()).await?;
    let token = config::token(dir.path())?;
    let base = format!("http://{}/v1", gateway.addr);
    let client = crabforge_test_support::Client::default();
    let send = |path: &str, body: String| {
        client
            .post(format!("{base}/{path}"))
            .bearer_auth(&token)
            .body(body)
            .send()
    };

    let native = send("decisions", r#"{ "model" : "openai/luna", "input": "direct", "questions": [{"type":"predicate"}], "prompt_cache_key": "keep" }"#.into()).await?;
    assert_eq!(native.text().await?, "{\"native\":true}");
    assert_eq!(requests.try_recv()?, "/native/decisions");
    let native = send(
        "systemone",
        json!({"model":"jev/jev","state":"direct","questions":{},"vendor_extension":[1,2]})
            .to_string(),
    )
    .await?;
    assert_eq!(native.text().await?, "{\"native\":true}");
    assert_eq!(requests.try_recv()?, "/fallback/systemone");

    let system_one = json!({"model":"openai/luna","state":{"document":123},"images":["https://example.org/receipt.png"],"questions":{
        "z":{"type":"noul","instructions":"Readable?"},"a":{"type":"choice","criteria":{"second":"Corresponding page","first":null}},
        "work":{"type":"score","criteria":["small","large"]}
    }});
    let reply = send("systemone", system_one.to_string())
        .await?
        .error_for_status()?;
    assert_eq!(reply.headers()["x-request-id"], "fixture");
    assert!(!reply.headers().contains_key("set-cookie"));
    let reply: Value = nextjson::from_slice(&reply.bytes().await?).map_err(anyhow::Error::msg)?;
    assert_eq!(reply.pointer("/answers/z/noul"), Some(&json!(0.75)));
    assert_eq!(
        reply["answers"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ["z", "a", "work"]
    );
    assert_eq!(reply.pointer("/answers/a/choice"), Some(&json!("first")));
    assert_eq!(
        reply.pointer("/answers/work/probabilities/large"),
        Some(&json!(0.9))
    );
    assert_eq!(reply.pointer("/usage/output_tokens"), Some(&json!(4)));
    assert_eq!(requests.try_recv()?, "/native/decisions");

    let request = json!({"model":"group/judges","prompt_cache_key":"cache","input":[
        {"role":"user","content":[{"type":"input_text","text":"A receipt"},{"type":"input_json","data":{"total":42}},
            {"type":"input_image","image_url":"data:image/png;base64,AAAA"}]},
        {"role":"assistant","content":[{"type":"refusal","refusal":"Unable to read it"}]}
    ],"questions":[{"type":"predicate","instructions":"Valid?"},{"type":"choice","name":"choice","choices":[{"value":true,"description":"Approved"},{"value":false}]},
        {"type":"score","name":"work","levels":[{"label":"small"},{"label":"large","description":"Extensive"}]},{"type":"predicate","name":"question_1"}]});
    let reply = send("decisions", request.to_string())
        .await?
        .error_for_status()?;
    let reply: Value = nextjson::from_slice(&reply.bytes().await?).map_err(anyhow::Error::msg)?;
    assert_eq!(reply.pointer("/answers/0/name"), Some(&Value::Null));
    assert_eq!(reply.pointer("/answers/0/probability"), Some(&json!(0.25)));
    assert_eq!(reply.pointer("/answers/1/choice"), Some(&json!(true)));
    assert_eq!(
        reply.pointer("/answers/1/probabilities/1/value"),
        Some(&json!(false))
    );
    assert_eq!(
        reply.pointer("/answers/2/probabilities/1/label"),
        Some(&json!("large"))
    );
    assert_eq!(reply.pointer("/answers/3/type"), Some(&json!("refusal")));
    assert_eq!(reply.pointer("/usage/total_tokens"), Some(&json!(15)));
    assert_eq!(requests.try_recv()?, "/native/decisions");
    assert_eq!(requests.try_recv()?, "/fallback/systemone");

    let rejected = send("decisions", json!({"model":"jev/jev","input":[{"type":"function_call","name":"search"}],"questions":[{"type":"predicate"}]}).to_string()).await?;
    assert_eq!(rejected.status(), 400);
    assert!(rejected.text().await?.contains("input[0]"));
    let rejected = send("systemone", json!({"model":"openai/luna","state":"test","audio":["data:audio/wav;base64,AAAA"],"questions":{"x":{"type":"noul"}}}).to_string()).await?;
    assert_eq!(rejected.status(), 400);
    assert!(rejected.text().await?.contains("audio"));
    assert_eq!(
        send("decisions", json!({"model":"chat/m"}).to_string())
            .await?
            .status(),
        400
    );
    assert!(requests.try_recv().is_err());
    gateway.stop().await;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn decision_classifier_applies_native_confidence_and_keeps_automatic_effort() -> Result<()> {
    use crabforge_test_support::server::Json;
    let listener = TcpListener::bind(ephemeral()).await?;
    let addr = listener.local_addr()?;
    let (seen, mut requests) = tokio::sync::mpsc::unbounded_channel();
    let app = Router::new().observe(seen)
        .route("/judge/decisions", post(|Incoming { body, .. }: Incoming| async move {
            let value: Value = nextjson::from_slice(&body).unwrap();
            assert_eq!(value["model"], json!("gpt-6-luna"));
            let message = value.pointer("/input/0/content/0/data/message").unwrap().as_str().unwrap();
            assert_eq!(value.pointer("/questions/0/name"), Some(&json!("intent")));
            let intent = if message == "Declined" {
                json!({"type":"refusal","name":"intent"})
            } else {
                json!({"type":"choice","name":"intent","choice":"1","confidence":0.9,
                    "probabilities":[{"value":"1","probability":0.65},{"value":"0","probability":0.35}]})
            };
            let mut answers = vec![intent];
            if let Some(name) = value.pointer("/questions/1/name") {
                assert_eq!(name, &json!("effort"));
                answers.push(json!({"type":"score","name":"effort","score":2.0}));
            }
            Json(json!({"answers":answers}))
        }))
        .route("/chat/chat/completions", post(|Incoming { body, .. }: Incoming| async move {
            let value: Value = nextjson::from_slice(&body).unwrap();
            let model = value["model"].as_str().unwrap();
            if model == "backup" {
                return Json(json!({"choices":[{"message":{"content":"1"}}]}));
            }
            Json(json!({"selected":model,"effort":value.get("reasoning_effort")}))
        }));
    let server = tokio::spawn(crabforge_test_support::server::serve(listener, app));
    let dir = tempdir()?;
    let members = json!(["chat/usual", "chat/review"]);
    let rule = json!([{"intent":"Review a report","use":"chat/review"}]);
    let cfg = json!({"listen":"127.0.0.1:0","providers":[
        {"id":"judge","models":["gpt-6-luna"],"endpoints":{"decisions":format!("http://{addr}/judge")}},
        {"id":"chat","models":["usual","review","backup"],"endpoints":{"chat":format!("http://{addr}/chat")},
            "model_reasoning_levels":{"usual":["medium","high"],"review":["medium","high"]}}
    ],"groups":{"default":members,"zero":members,"recover":members},"group_rules":{"default":rule,"zero":rule,"recover":rule},
      "group_classifiers":{
        "default":{"classifier":"judge/gpt-6-luna","effort":"auto"},
        "zero":{"classifier":"judge/gpt-6-luna","intentSure":0},
        "recover":{"classifier":"judge/gpt-6-luna","classifierMore":["chat/backup"]}
    }});
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let gateway = Service::start(dir.path()).await?;
    let token = config::token(dir.path())?;
    let url = format!("http://{}/v1/chat/completions", gateway.addr);
    let client = crabforge_test_support::Client::default();
    for (group, message, model, effort) in [
        ("default", "Review", "usual", "high"),
        ("zero", "Review", "review", "medium"),
        ("recover", "Declined", "review", "medium"),
    ] {
        let body = json!({"model":format!("group/{group}"),"reasoning_effort":"medium","messages":[{"role":"user","content":message}]});
        let reply = client
            .post(&url)
            .bearer_auth(&token)
            .body(body.to_string())
            .send()
            .await?
            .error_for_status()?;
        let reply: Value =
            nextjson::from_slice(&reply.bytes().await?).map_err(anyhow::Error::msg)?;
        assert_eq!(reply["selected"], json!(model));
        assert_eq!(reply["effort"], json!(effort));
        assert_eq!(requests.try_recv()?, "/judge/decisions");
        if group == "recover" {
            assert_eq!(requests.try_recv()?, "/chat/chat/completions");
        }
        assert_eq!(requests.try_recv()?, "/chat/chat/completions");
    }
    assert!(requests.try_recv().is_err());
    gateway.stop().await;
    server.abort();
    Ok(())
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
        model_reasoning_levels: Default::default(),
        contexts: Default::default(),
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
async fn opted_in_sse_wait_keeps_first_content_fallback_and_late_errors() -> Result<()> {
    const COMMENT: &str = ": keepalive\n\n";
    const REPLY: &str =
        "data: {\"choices\":[{\"delta\":{\"content\":\"backup\"}}]}\n\ndata: [DONE]\n\n";
    let (release, held) = watch::channel(false);
    let mock = Router::new().route(
        "/{provider}/chat/completions",
        post(move |Incoming { param: id, .. }| {
            let mut held = held.clone();
            async move {
                if id == "ready" {
                    return (
                        StatusCode::OK,
                        [("content-type", "text/event-stream")],
                        REPLY,
                    );
                }
                if id != "busy" {
                    held.wait_for(|value| *value).await.unwrap();
                }
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("retry-after", "7")],
                    r#"{"error":{"message":"upstream busy"}}"#,
                )
            }
        }),
    );
    let listener = TcpListener::bind(ephemeral()).await?;
    let addr = listener.local_addr()?;
    let upstream = tokio::spawn(crabforge_test_support::server::serve(listener, mock));
    let dir = tempdir()?;
    let providers = ["slow", "ready", "busy"].map(
        |id| json!({"id":id,"models":["m"],"endpoints":{"chat":format!("http://{addr}/{id}")}}),
    );
    let cfg = json!({
        "listen":"127.0.0.1:0","providers":providers,
        "groups":{"pool":["slow/m","ready/m"]},"group_first_content":{"pool":4}
    });
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let token = config::token(dir.path())?;
    let client = crabforge_test_support::Client::with_timeout(Duration::from_secs(12));
    let url = format!("http://{}/v1/chat/completions", service.addr);
    let send = |model: &str, keepalive: bool| {
        client
            .post(&url)
            .bearer_auth(&token)
            .header("x-crabforge-keepalive", keepalive.to_string())
            .body(
                json!({"model":model,"messages":[{"role":"user","content":"hello"}],"stream":true})
                    .to_string(),
            )
            .send()
    };
    let fast_error = send("busy/m", true).await?;
    assert_eq!(fast_error.status(), 429);
    assert_eq!(fast_error.headers().get("retry-after").unwrap(), "7");
    let (fallback, late) = tokio::join!(send("group/pool", true), send("slow/m", true));
    let mut fallback = fallback?;
    let mut late = late?;
    for response in [&mut fallback, &mut late] {
        assert_eq!(response.status(), 200);
        assert_eq!(response.chunk().await?.unwrap(), COMMENT);
    }
    assert_eq!(fallback.text().await?, REPLY);
    let unchanged = send("slow/m", false);
    tokio::pin!(unchanged);
    assert!(
        timeout(Duration::from_millis(50), unchanged.as_mut())
            .await
            .is_err()
    );
    release.send(true)?;
    let unchanged = unchanged.await?;
    assert_eq!(unchanged.status(), 429);
    assert_eq!(unchanged.headers().get("retry-after").unwrap(), "7");
    let body = late.text().await?;
    let error: Value = nextjson::from_str(body.strip_prefix("data: ").unwrap().trim())
        .map_err(anyhow::Error::msg)?;
    assert_eq!(error["error"]["status"], json!(429));
    assert_eq!(error["error"]["retry_after"], json!("7"));
    assert!(
        service
            .history
            .lock()
            .unwrap()
            .iter()
            .any(|a| a.provider == "slow" && a.status == 504)
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
                let first_call = {
                    let mut seen = seen.lock().unwrap();
                    let first_call = !seen.iter().any(|call| call == model);
                    seen.push(model.to_owned());
                    first_call
                };
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
                    "changing" => {
                        let prompt: Value =
                            nextjson::from_str(body["messages"][1]["content"].as_str().unwrap())
                                .unwrap();
                        let kinds = prompt["kinds"].as_array().unwrap();
                        let label = match kinds.as_slice() {
                            [general] if general["intent"].as_str() == Some("General review") => {
                                "0"
                            }
                            [general, deep]
                                if general["intent"].as_str() == Some("General review")
                                    && deep["intent"].as_str() == Some("Deep review") =>
                            {
                                "2"
                            }
                            _ => panic!("unexpected classifier kinds: {kinds:?}"),
                        };
                        (
                            StatusCode::OK,
                            json!({"choices":[{"message":{"content":label}}]}).to_string(),
                        )
                    }
                    "transient" if first_call => (StatusCode::SERVICE_UNAVAILABLE, String::new()),
                    "transient" => (
                        StatusCode::OK,
                        json!({"choices":[{"message":{"content":"1"}}]}).to_string(),
                    ),
                    "confidence" | "confidence-backup" => {
                        assert!(
                            body["messages"][0]["content"]
                                .as_str()
                                .unwrap()
                                .contains("include a confidence number")
                        );
                        let prompt: Value =
                            nextjson::from_str(body["messages"][1]["content"].as_str().unwrap())
                                .unwrap();
                        let answer = match (model, prompt["message"].as_str().unwrap()) {
                            (_, "Thanks") => json!({"intent":0}),
                            (_, "Low confidence") => json!({"intent":1,"confidence":0.4}),
                            ("confidence", "Out of range") => json!({"intent":1,"confidence":1.1}),
                            _ => json!({"intent":1,"confidence":0.6}),
                        };
                        (
                            StatusCode::OK,
                            json!({"choices":[{"message":{"content":answer.to_string()}}]})
                                .to_string(),
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
        "providers":[{"id":"fixture","models":["slow","decide","changing","transient","usual","review","deep","confidence","confidence-backup"],
            "endpoints":{"chat":format!("http://{addr}/v1")}}],
        "groups":{
            "outer":["fixture/usual","group/intent"],
            "intent":["fixture/usual","fixture/review"],
            "classifier":["fixture/decide"],
            "changing":["fixture/usual","fixture/review","fixture/deep"],
            "transient":["fixture/usual","fixture/review"],
            "manual":["fixture/usual","fixture/review"],
            "threshold":["fixture/usual","fixture/review"]
        },
        "group_picks":{"manual":"fixture/usual"},
        "group_rules":{
            "intent":[{"use":"fixture/review","intent":"Review report"}],
            "changing":[{"use":"fixture/review","intent":"General review"},
                        {"use":"fixture/deep","intent":"Deep review","effort":"high"}],
            "transient":[{"use":"fixture/review","intent":"Review report"}],
            "manual":[{"use":"fixture/review","intent":"Review report"}],
            "threshold":[{"use":"fixture/review","intent":"Review report"}]
        },
        "group_classifiers":{
            "intent":{"classifier":"fixture/slow","classifierMore":["group/classifier"],"classifyPrompt":"Judge which task was requested."},
            "changing":{"classifier":"fixture/changing"},
            "transient":{"classifier":"fixture/transient"},
            "manual":{"classifier":"fixture/decide","intentSure":0},
            "threshold":{"classifier":"fixture/confidence","classifierMore":["fixture/confidence-backup"],"intentSure":0.6}
        }
    });
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let url = format!("http://{}/v1/chat/completions", service.addr);
    let token = config::token(dir.path())?;
    let client = crabforge_test_support::Client::with_timeout(Duration::from_secs(5));
    let send = |group: &str, message: &str, with_tools: bool, reasoning: &str| {
        let mut messages = vec![
            json!({"role":"system","content":"Internal system instructions"}),
            json!({"role":"user","content":"Open the report"}),
            json!({"role":"assistant","content":"It is open"}),
            json!({"role":"user","content":message}),
        ];
        if with_tools {
            messages.extend([
                json!({"role":"assistant","content":"Checking the report"}),
                json!({"role":"tool","content":"Tool output","tool_call_id":"report"}),
            ]);
        }
        let request = json!({"model":group,"messages":messages,"reasoning_effort":reasoning});
        client
            .post(&url)
            .bearer_auth(&token)
            .header("x-session-id", "intent-fixture")
            .body(request.to_string())
            .send()
    };
    assert_eq!(
        send("group/outer", "Review it", false, "none")
            .await?
            .text()
            .await?,
        "usual"
    );
    assert_eq!(
        send("group/manual", "Review it", false, "none")
            .await?
            .text()
            .await?,
        "usual"
    );
    let (first, concurrent) = tokio::join!(
        send("group/intent", "Review it", false, "none"),
        send("group/intent", "Review it", false, "none"),
    );
    assert_eq!(first?.text().await?, "review");
    assert_eq!(concurrent?.text().await?, "review");
    assert_eq!(
        send("group/intent", "Review it", true, "high")
            .await?
            .text()
            .await?,
        "review"
    );
    assert_eq!(
        send("group/intent", "Thanks", false, "none")
            .await?
            .text()
            .await?,
        "usual"
    );
    assert_eq!(
        *calls.lock().unwrap(),
        [
            "usual", "usual", "slow", "decide", "review", "review", "review", "slow", "decide",
            "usual"
        ]
    );

    calls.lock().unwrap().clear();
    for (group, with_tools, reasoning, expected) in [
        ("group/changing", false, "none", "usual"),
        ("group/changing", false, "none", "usual"),
        ("group/changing", true, "high", "usual"),
        ("group/changing", false, "high", "deep"),
        ("group/changing", false, "high", "deep"),
        ("group/changing", false, "none", "usual"),
        ("group/transient", false, "none", "usual"),
        ("group/transient", true, "high", "review"),
        ("group/transient", false, "none", "review"),
    ] {
        assert_eq!(
            send(group, "Review it", with_tools, reasoning)
                .await?
                .text()
                .await?,
            expected
        );
    }
    assert_eq!(
        *calls.lock().unwrap(),
        [
            "changing",
            "usual",
            "usual",
            "usual",
            "changing",
            "deep",
            "deep",
            "usual",
            "transient",
            "usual",
            "transient",
            "review",
            "review"
        ]
    );

    calls.lock().unwrap().clear();
    for (message, with_tools, expected) in [
        ("Low confidence", false, "usual"),
        ("Low confidence", true, "usual"),
        ("Review it", false, "review"),
        ("Out of range", false, "review"),
        ("Thanks", false, "usual"),
    ] {
        assert_eq!(
            send("group/threshold", message, with_tools, "none")
                .await?
                .text()
                .await?,
            expected
        );
    }
    assert_eq!(
        *calls.lock().unwrap(),
        [
            "confidence",
            "usual",
            "usual",
            "confidence",
            "review",
            "confidence",
            "confidence-backup",
            "review",
            "confidence",
            "usual",
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
    for (group, threshold) in [("threshold", 0.6), ("manual", 0.0)] {
        let (restored, report) = config.import_group(&config.share_group(group)?, false)?;
        assert_eq!(
            restored.group_classifiers[&report.id].intent_sure,
            Some(threshold)
        );
    }
    service.stop().await;
    upstream.abort();
    Ok(())
}

#[tokio::test]
async fn tool_continuation_holds_the_rule_until_paused_or_outgrown() -> Result<()> {
    let mock = Router::new().route(
        "/v1/chat/completions",
        post(|request: Incoming| async move {
            let body: Value = nextjson::from_slice(&request.body).unwrap();
            format!(
                "{}:{}",
                body["model"].as_str().unwrap(),
                body["reasoning_effort"].as_str().unwrap()
            )
        }),
    );
    let listener = TcpListener::bind(ephemeral()).await?;
    let addr = listener.local_addr()?;
    let upstream = tokio::spawn(crabforge_test_support::server::serve(listener, mock));
    let root = tempdir()?;
    let cfg = json!({
        "listen":"127.0.0.1:0",
        "providers":[{"id":"fixture","models":["small","large","backup"],
            "contexts":{"*":1000,"small":100},
            "model_reasoning_levels":{"small":["medium","high"],"large":["medium","high"]},
            "endpoints":{"chat":format!("http://{addr}/v1")}}],
        "groups":{"review":["fixture/backup","fixture/large","fixture/small","group/manual"],
            "manual":["fixture/large","fixture/backup"]},
        "group_picks":{"manual":"fixture/large"},
        "group_rules":{"review":[
            {"use":"fixture/small","pause":true,"folders":["C:/paused"]},
            {"use":"fixture/large","tokens":95,"useEffort":"high"},
            {"use":"fixture/small","effort":"low","where":[{"field":"effort","op":"is","value":"low"}],"useEffort":"medium"}
        ]}
    });
    config::save(
        root.path(),
        &config::snapshot(root.path())?,
        &cfg.to_string(),
    )?;
    let service = Service::start(root.path()).await?;
    let url = format!("http://{}/v1/chat/completions", service.addr);
    let token = config::token(root.path())?;
    let client = crabforge_test_support::Client::default();
    let send = |session: &str, effort: &str, result: Option<usize>, paused: bool, new: bool| {
        let mut messages = vec![json!({"role":"user","content":"Review the report"})];
        if paused {
            messages.insert(
                0,
                json!({"role":"system","content":"Working directory: C:/paused"}),
            );
        }
        if let Some(size) = result {
            messages.push(
                json!({"role":"assistant","tool_calls":[{"id":"report","type":"function",
                "function":{"name":"report","arguments":"{}"}}]}),
            );
            messages
                .push(json!({"role":"tool","tool_call_id":"report","content":"x".repeat(size)}));
        }
        if new {
            messages.push(json!({"role":"user","content":"Check it again"}));
        }
        client
            .post(&url)
            .bearer_auth(&token)
            .header("x-session-id", session)
            .body(
                json!({"model":"group/review","messages":messages,"reasoning_effort":effort})
                    .to_string(),
            )
            .send()
    };
    for (session, effort, result, paused, new, expected) in [
        ("original", "low", None, false, false, "small:medium"),
        ("original", "high", Some(8), false, false, "small:medium"),
        ("different", "high", Some(8), false, false, "backup:high"),
        ("original", "high", Some(8), true, false, "backup:high"),
        ("original", "high", Some(8), false, true, "backup:high"),
        ("original", "low", Some(400), false, false, "large:high"),
        ("original", "low", Some(8), false, false, "large:high"),
    ] {
        assert_eq!(
            send(session, effort, result, paused, new)
                .await?
                .text()
                .await?,
            expected
        );
    }
    service.stop().await;
    upstream.abort();
    Ok(())
}

#[tokio::test]
async fn gemini_effort_fits_the_native_model_on_each_attempt() -> Result<()> {
    use crabforge_test_support::server::IntoResponse;
    use std::sync::{Arc, Mutex};

    let calls = Arc::new(Mutex::new(Vec::<(String, Value)>::new()));
    let seen = calls.clone();
    let mock = Router::new()
        .route(
            "/v1/chat/completions",
            post(|_: Incoming| async {
                json!({"choices":[{"message":{"content":"{\"intent\":0,\"effort\":\"medium\"}"}}]})
                    .to_string()
            }),
        )
        .fallback(post(move |req: Incoming| {
            let seen = seen.clone();
            async move {
                let path = req.uri.path();
                let model = path
                    .split('/')
                    .next_back()
                    .unwrap()
                    .split(':')
                    .next()
                    .unwrap();
                let body: Value = nextjson::from_slice(&req.body).unwrap();
                assert!(body.get("model").is_none());
                seen.lock().unwrap().push((model.to_owned(), body));
                if model == "gemini-4-neon" {
                    (StatusCode::SERVICE_UNAVAILABLE, "busy").into_response()
                } else if path.ends_with(":streamGenerateContent") {
                    (
                        [(http::header::CONTENT_TYPE, "text/event-stream")],
                        "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"ready\"}]},\"finishReason\":\"STOP\"}]}\r\n\r\n",
                    )
                        .into_response()
                } else {
                    (StatusCode::OK, "ready").into_response()
                }
            }
        }));
    let listener = TcpListener::bind(ephemeral()).await?;
    let base = format!("http://{}", listener.local_addr()?);
    let upstream = tokio::spawn(crabforge_test_support::server::serve(listener, mock));
    let root = tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0","providers":[
        {"id":"google","models":["gemini-4-neon","gemini-2.5-flash","gemini-3.1-pro-preview","custom"],"endpoints":{"gemini":&base}},
        {"id":"classifier","models":["judge"],"endpoints":{"chat":format!("{base}/v1")}}
    ],"groups":{
        "fixed":["google/gemini-4-neon","google/gemini-2.5-flash"],
        "automatic":["google/gemini-3.1-pro-preview"],
        "unknown":["google/custom"]
    },"group_rules":{"fixed":[{"use":"google/gemini-4-neon","more":["google/gemini-2.5-flash"],"useEffort":"xhigh",
        "where":[{"field":"message","op":"starts","value":"review"}]}],
        "unknown":[{"use":"google/custom","useEffort":"high","where":[{"field":"message","op":"starts","value":"review"}]}]},
    "group_classifiers":{"automatic":{"classifier":"classifier/judge","effort":"auto"}}});
    config::save(
        root.path(),
        &config::snapshot(root.path())?,
        &cfg.to_string(),
    )?;
    let service = Service::start(root.path()).await?;
    let token = config::token(root.path())?;
    let client = crabforge_test_support::Client::default();
    let send = |group: &str, action: &str, options: Value| {
        let mut body = json!({"contents":[{"role":"user","parts":[{"text":"Review the report"}]}]});
        body.as_object_mut()
            .unwrap()
            .extend(options.as_object().unwrap().clone());
        client
            .post(format!(
                "http://{}/v1beta/models/group/{group}:{action}",
                service.addr
            ))
            .bearer_auth(&token)
            .body(body.to_string())
            .send()
    };
    let options = json!({"generationConfig":{"temperature":0.7,"thinkingConfig":{"thinkingLevel":"LOW","includeThoughts":true}}});
    let reply = send("fixed", "streamGenerateContent", options).await?;
    assert_eq!(reply.status(), 200);
    assert_eq!(
        reply.text().await?,
        "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"ready\"}]},\"finishReason\":\"STOP\"}]}\r\n\r\n"
    );
    let options = json!({"generation_config":{"thinking_config":{"thinking_budget":-1,"include_thoughts":false}}});
    assert_eq!(
        send("automatic", "generateContent", options)
            .await?
            .status(),
        200
    );
    let options = json!({"generationConfig":{"thinkingConfig":{"thinkingBudget":0}}});
    assert_eq!(
        send("fixed", "generateContent", options).await?.status(),
        200
    );
    let options = json!({"generationConfig":{"thinkingConfig":{"thinkingLevel":"LOW"}}});
    assert_eq!(
        send("unknown", "generateContent", options.clone())
            .await?
            .status(),
        200
    );
    let calls = calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 6);
    assert_eq!(calls[0].0, "gemini-4-neon");
    assert_eq!(calls[1].0, "gemini-2.5-flash");
    assert_eq!(calls[2].0, "gemini-3.1-pro-preview");
    assert_eq!(
        calls[0].1["generationConfig"],
        json!({"temperature":0.7,"thinkingConfig":{"includeThoughts":true,"thinkingLevel":"XHIGH"}})
    );
    assert_eq!(
        calls[1].1["generationConfig"],
        json!({"temperature":0.7,"thinkingConfig":{"includeThoughts":true,"thinkingBudget":24576}})
    );
    assert_eq!(
        calls[2].1["generation_config"]["thinking_config"],
        json!({"include_thoughts":false,"thinking_level":"MEDIUM"})
    );
    for (_, body) in &calls[3..5] {
        assert_eq!(
            body["generationConfig"]["thinkingConfig"],
            json!({"thinkingBudget":0})
        );
    }
    assert_eq!(calls[5].0, "custom");
    assert_eq!(calls[5].1["generationConfig"], options["generationConfig"]);
    service.stop().await;
    upstream.abort();
    Ok(())
}

#[tokio::test]
async fn route_rule_effort_fits_each_model_and_preserves_native_options() -> Result<()> {
    use std::sync::{Arc, Mutex};

    let calls = Arc::new(Mutex::new(Vec::<Value>::new()));
    let seen = calls.clone();
    let answer = move |request: Incoming| {
        let seen = seen.clone();
        async move {
            let body: Value = nextjson::from_slice(&request.body).unwrap();
            let status = if body["model"].as_str() == Some("primary") {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::OK
            };
            seen.lock().unwrap().push(body);
            (status, "reply")
        }
    };
    let mock = Router::new()
        .route("/v1/chat/completions", post(answer.clone()))
        .route("/v1/responses", post(answer.clone()))
        .route("/v1/messages", post(answer));
    let listener = TcpListener::bind(ephemeral()).await?;
    let addr = listener.local_addr()?;
    let upstream = tokio::spawn(crabforge_test_support::server::serve(listener, mock));
    let dir = tempdir()?;
    let base = format!("http://{addr}/v1");
    let cfg = json!({
        "listen":"127.0.0.1:0",
        "providers":[{"id":"fixture", "models":["primary","backup","unknown"],
            "model_reasoning_levels":{"primary":["low","medium"]},
            "endpoints":{"chat":base,"responses":base,"anthropic":base}}],
        "model_efforts":{"fixture/primary":["low"],"fixture/backup":["low","high","xhigh"]},
        "groups":{
            "outer":["fixture/primary","group/nested"],
            "nested":["fixture/backup"],
            "unknown":["fixture/unknown"]
        },
        "group_rules":{
            "outer":[{"use":"fixture/primary","effort":"on","useEffort":"high"}],
            "nested":[{"use":"fixture/backup","effort":"on","useEffort":"max"}],
            "unknown":[{"use":"fixture/unknown","effort":"on","useEffort":"high"}]
        }
    });
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let token = config::token(dir.path())?;
    let client = crabforge_test_support::Client::with_timeout(Duration::from_secs(5));
    let send = |path: &str, body: Value| {
        client
            .post(format!("http://{}/v1/{path}", service.addr))
            .bearer_auth(&token)
            .body(body.to_string())
            .send()
    };
    assert_eq!(
        send(
            "chat/completions",
            json!({"model":"group/outer","reasoning_effort":"low",
        "messages":[{"role":"user","content":"Review"}]})
        )
        .await?
        .status(),
        200
    );
    assert_eq!(
        send(
            "chat/completions",
            json!({"model":"group/unknown","reasoning_effort":"low",
        "messages":[{"role":"user","content":"Review"}]})
        )
        .await?
        .status(),
        200
    );
    assert_eq!(
        send(
            "responses",
            json!({"model":"group/nested","input":"Review",
        "reasoning":{"effort":"low","summary":"auto","context":"all_turns"}})
        )
        .await?
        .status(),
        200
    );
    assert_eq!(send("messages", json!({"model":"group/nested","max_tokens":5000,
        "thinking":{"type":"enabled","budget_tokens":1024},"output_config":{"effort":"low","format":{"type":"json_schema"}},
        "messages":[{"role":"user","content":"Review"}]})).await?.status(), 200);
    assert_eq!(
        send(
            "messages",
            json!({"model":"group/nested","max_tokens":5000,
        "thinking":{"type":"adaptive"},"messages":[{"role":"user","content":"Review"}]})
        )
        .await?
        .status(),
        200
    );
    assert_eq!(
        send(
            "responses",
            json!({"model":"group/nested","input":"Title",
        "reasoning":{"effort":"none"}})
        )
        .await?
        .status(),
        200
    );

    let requests = calls.lock().unwrap().clone();
    assert_eq!(requests.len(), 7);
    assert_eq!(requests[0]["model"].as_str(), Some("primary"));
    assert_eq!(requests[0]["reasoning_effort"].as_str(), Some("medium"));
    assert_eq!(requests[1]["model"].as_str(), Some("backup"));
    assert_eq!(requests[1]["reasoning_effort"].as_str(), Some("xhigh"));
    assert_eq!(requests[2]["reasoning_effort"].as_str(), Some("low"));
    assert_eq!(requests[3]["reasoning"]["effort"].as_str(), Some("xhigh"));
    assert_eq!(requests[3]["reasoning"]["summary"].as_str(), Some("auto"));
    assert_eq!(
        requests[3]["reasoning"]["context"].as_str(),
        Some("all_turns")
    );
    assert_eq!(
        requests[4]["thinking"]["budget_tokens"].as_u64(),
        Some(4999)
    );
    assert_eq!(
        requests[4]["output_config"]["effort"].as_str(),
        Some("xhigh")
    );
    assert_eq!(
        requests[4]["output_config"]["format"]["type"].as_str(),
        Some("json_schema")
    );
    assert_eq!(
        requests[5]["output_config"]["effort"].as_str(),
        Some("xhigh")
    );
    assert_eq!(requests[6]["reasoning"]["effort"].as_str(), Some("none"));
    service.stop().await;
    upstream.abort();
    Ok(())
}

#[tokio::test]
async fn automatic_effort_shares_the_intent_verdict_and_flows_into_subgroups() -> Result<()> {
    use std::sync::{Arc, Mutex};

    let calls = Arc::new(Mutex::new(Vec::new()));
    let seen = calls.clone();
    let mock = Router::new().route(
        "/v1/chat/completions",
        post(move |request: Incoming| {
            let seen = seen.clone();
            async move {
                let body: Value = nextjson::from_slice(&request.body).unwrap();
                let model = body["model"].as_str().unwrap();
                let effort = body
                    .get("reasoning_effort")
                    .and_then(Value::as_str)
                    .map(str::to_owned);
                let reply = match model {
                    "primary" => "invalid JSON",
                    "backup" => {
                        let question: Value =
                            nextjson::from_str(body["messages"][1]["content"].as_str().unwrap())
                                .unwrap();
                        if question["kinds"].as_array().unwrap().is_empty() {
                            "{\"effort\":\"high\"}"
                        } else {
                            "{\"intent\":1,\"effort\":\"high\",\"confidence\":0.6}"
                        }
                    }
                    _ => "done",
                };
                seen.lock().unwrap().push((model.to_owned(), effort));
                json!({"choices":[{"message":{"role":"assistant","content":reply}}]}).to_string()
            }
        }),
    );
    let listener = TcpListener::bind(ephemeral()).await?;
    let addr = listener.local_addr()?;
    let upstream = tokio::spawn(crabforge_test_support::server::serve(listener, mock));
    let dir = tempdir()?;
    let cfg = json!({
        "listen":"127.0.0.1:0",
        "providers":[{"id":"fixture", "models":["ordinary","reasoner","primary","backup"],
            "endpoints":{"chat":format!("http://{addr}/v1")}}],
        "model_efforts":{"fixture/reasoner":["low","xhigh"]},
        "groups":{"outer":["fixture/ordinary","group/nested"],"nested":["fixture/reasoner"]},
        "group_rules":{"outer":[{"use":"group/nested","intent":"Review","effort":"on"}]},
        "group_classifiers":{
            "outer":{"effort":"auto","classifier":"fixture/primary","classifierMore":["fixture/backup"],"intentSure":0.6},
            "nested":{"effort":"auto","classifier":"fixture/backup"}
        }
    });
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let config = config::load(dir.path())?;
    let share = config.share_group("outer")?;
    let (copied, report) = config.import_group(&share, false)?;
    assert_eq!(copied.group_classifiers[&report.id].effort, "auto");
    let service = Service::start(dir.path()).await?;
    let token = config::token(dir.path())?;
    let client = crabforge_test_support::Client::with_timeout(Duration::from_secs(5));
    let send = |group: &str, effort: &str, continuation: bool| {
        let mut messages = vec![json!({"role":"user","content":"Review the report"})];
        if continuation {
            messages.extend([
                json!({"role":"assistant","tool_calls":[{"id":"lookup","type":"function",
                    "function":{"name":"read_report","arguments":"{}"}}]}),
                json!({"role":"tool","tool_call_id":"lookup","content":"The report is ready"}),
            ]);
        }
        let body =
            json!({"model":format!("group/{group}"),"reasoning_effort":effort,"messages":messages});
        client
            .post(format!("http://{}/v1/chat/completions", service.addr))
            .bearer_auth(&token)
            .header("X-Session-ID", "auto-effort")
            .body(body.to_string())
            .send()
    };
    for (group, effort, continuation) in [
        ("outer", "low", false),
        ("outer", "medium", true),
        ("outer", "none", false),
        ("nested", "low", false),
    ] {
        assert_eq!(send(group, effort, continuation).await?.status(), 200);
    }
    let calls = calls.lock().unwrap().clone();
    assert_eq!(
        calls
            .iter()
            .map(|(model, effort)| (model.as_str(), effort.as_deref()))
            .collect::<Vec<_>>(),
        [
            ("primary", None),
            ("backup", None),
            ("reasoner", Some("xhigh")),
            ("reasoner", Some("xhigh")),
            ("ordinary", Some("none")),
            ("backup", None),
            ("reasoner", Some("xhigh")),
        ]
    );
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
