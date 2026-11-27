use anyhow::Result;
use crabforge_core::{
    config::{self, Config, Endpoints, Provider},
    service::Service,
};
use crabforge_test_support::server::{Body, IntoResponse, Response};
use crabforge_test_support::server::{Incoming, Json, Router, post};
use nextjson::{Value, json};
use std::{collections::BTreeMap, net::SocketAddr};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio::sync::Notify;

fn request(protocol: &str, model: &str, stream: bool) -> (String, Value) {
    match protocol {
        "chat" => (
            "/v1/chat/completions".into(),
            json!({"model":model,"messages":[{"role":"user","content":"city?"}],"stream":stream,"store":false}),
        ),
        "responses" => (
            "/v1/responses".into(),
            json!({"model":model,"input":"city?","stream":stream,"store":false}),
        ),
        "anthropic" => (
            "/v1/messages".into(),
            json!({"model":model,"messages":[{"role":"user","content":[{"type":"text","text":"city?"}]}],"max_tokens":37,"stream":stream}),
        ),
        "gemini" => (
            format!(
                "/v1beta/models/{model}:{}",
                if stream {
                    "streamGenerateContent"
                } else {
                    "generateContent"
                }
            ),
            json!({"contents":[{"role":"user","parts":[{"text":"city?"}]}]}),
        ),
        _ => unreachable!(),
    }
}

async fn vendor(
    Incoming {
        uri, headers, body, ..
    }: Incoming,
) -> Json {
    let req: Value = nextjson::from_slice(&body).unwrap();
    assert!(req.to_string().contains("city?"));
    assert!(!headers.contains_key("authorization"));
    let args = json!({"city":"上海"});
    let native = match uri.path() {
        "/chat/completions" => {
            json!({"choices":[{"message":{"role":"assistant","content":null,"tool_calls":[{"type":"function","id":"call_7","function":{"name":"weather","arguments":args.to_string()}}]},"finish_reason":"tool_calls"}],"usage":{"prompt_tokens":13,"completion_tokens":6}})
        }
        "/responses" => {
            json!({"status":"completed","output":[{"type":"function_call","call_id":"call_7","name":"weather","arguments":args.to_string()}],"usage":{"input_tokens":13,"output_tokens":6}})
        }
        "/v1/messages" => {
            assert_eq!(headers["anthropic-version"], "2023-06-01");
            assert!(req["max_tokens"].as_u64().unwrap() > 0);
            json!({"content":[{"type":"tool_use","id":"call_7","name":"weather","input":args}],"stop_reason":"tool_use","usage":{"input_tokens":13,"output_tokens":6}})
        }
        "/v1beta/models/m:generateContent" | "/v1beta/models/m:streamGenerateContent" => {
            assert!(req.get("model").is_none());
            json!({"candidates":[{"content":{"role":"model","parts":[{"functionCall":{"id":"call_7","name":"weather","args":args}}]},"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":13,"candidatesTokenCount":6}})
        }
        path => panic!("unexpected vendor path {path}"),
    };
    Json(native)
}

#[tokio::test]
async fn four_protocols_preserve_function_calls_and_usage_across_the_gateway() -> Result<()> {
    let bind: SocketAddr = "127.0.0.1:0".parse()?;
    let listener = TcpListener::bind(bind).await?;
    let base = format!("http://{}", listener.local_addr()?);
    let task = tokio::spawn(async move {
        crabforge_test_support::server::serve(listener, Router::new().fallback(post(vendor))).await
    });
    let root = tempfile::tempdir()?;
    let formats = ["chat", "responses", "anthropic", "gemini"];
    let cfg = Config {
        listen: bind,
        providers: formats
            .iter()
            .map(|name| Provider {
                patience: -1,
                fallback: vec![],
                no_stand_in: false,
                model_protocols: Default::default(),
                model_reasoning_levels: Default::default(),
                contexts: Default::default(),
                free_models: Default::default(),
                id: name.to_string(),
                catalog: String::new(),
                web_site: None,
                key_env: String::new(),
                models: vec!["m".into()],
                endpoints: nextjson::from_value::<Endpoints>(nextjson::json!({(*name): &base}))
                    .unwrap(),
            })
            .collect(),
        groups: BTreeMap::new(),
        ..Default::default()
    };
    config::save(
        root.path(),
        &config::snapshot(root.path())?,
        &cfg.to_json()?,
    )?;
    let service = Service::start(root.path()).await?;
    let key = config::token(root.path())?;
    let client = crabforge_test_support::Client::default();
    for from in formats {
        for to in formats.into_iter().filter(|to| *to != from) {
            for stream in [false, true] {
                let (path, body) = request(from, &format!("{to}/m"), stream);
                let r = client
                    .post(format!("http://{}{path}", service.addr))
                    .bearer_auth(&key)
                    .body(body.to_string())
                    .send()
                    .await?;
                let status = r.status();
                let body = r.text().await?;
                assert_eq!(status, 200, "{from} -> {to}: {body}");
                if stream {
                    assert!(body.contains("call_7") && body.contains("weather"));
                    assert!(body.contains(match from {
                        "chat" => "[DONE]",
                        "anthropic" => "event: message_stop",
                        "responses" => "event: response.completed",
                        _ => "\"finishReason\":\"STOP\"",
                    }));
                    continue;
                }
                let reply: Value = nextjson::from_str(&body).map_err(anyhow::Error::msg)?;
                let (name,input,count)=match from {
            "chat"=>(&reply["choices"][0]["message"]["tool_calls"][0]["function"]["name"],nextjson::from_str(reply["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"].as_str().unwrap()).map_err(anyhow::Error::msg)?,&reply["usage"]["completion_tokens"]),
            "anthropic"=>(&reply["content"][0]["name"],reply["content"][0]["input"].clone(),&reply["usage"]["output_tokens"]),
            "responses"=>(&reply["output"][0]["name"],nextjson::from_str(reply["output"][0]["arguments"].as_str().unwrap()).map_err(anyhow::Error::msg)?,&reply["usage"]["output_tokens"]),
            _=>{let f=&reply["candidates"][0]["content"]["parts"][0]["functionCall"];(&f["name"],f["args"].clone(),&reply["usageMetadata"]["candidatesTokenCount"])},
        };
                assert_eq!(name.as_str(), Some("weather"));
                assert_eq!(input, json!({"city":"上海"}));
                assert_eq!(count, &json!(6));
            }
        }
    }
    let (path, mut body) = request("chat", "gemini/m", false);
    body.as_object_mut()
        .unwrap()
        .insert("store".into(), true.into());
    let response = client
        .post(format!("http://{}{path}", service.addr))
        .bearer_auth(&key)
        .body(body.to_string())
        .send()
        .await?;
    assert_eq!(response.status(), 400);
    assert!(response.text().await?.contains("store"));
    service.stop().await;
    task.abort();
    let _ = task.await;
    Ok(())
}

#[tokio::test]
async fn translated_media_preserves_inline_images_audio_and_tool_order() -> Result<()> {
    use std::sync::Mutex;

    const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aCHwAAAAASUVORK5CYII=";
    const WAV: &str = "UklGRiQAAABXQVZFZm10IBAAAAABAAEAQB8AAIA+AAACABAAZGF0YQAAAAA=";
    let uri = format!("data:image/png;base64,{PNG}");
    let formats = ["chat", "responses", "anthropic", "gemini"];
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", listener.local_addr()?);
    let captures = Arc::new(Mutex::new(Vec::new()));
    let router = Router::new().fallback(post({
        let captures = captures.clone();
        move |request: Incoming| {
            let captures = captures.clone();
            async move {
                captures
                    .lock()
                    .unwrap()
                    .push(nextjson::from_slice::<Value>(&request.body).unwrap());
                vendor(request).await
            }
        }
    }));
    let server =
        tokio::spawn(async move { crabforge_test_support::server::serve(listener, router).await });
    let dir = tempfile::tempdir()?;
    let providers: Vec<_> = formats
        .iter()
        .map(|name| json!({"id":name,"models":["m"],"endpoints":{(*name):base}}))
        .collect();
    let cfg = json!({"listen":"127.0.0.1:0","providers":providers});
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let client = crabforge_test_support::Client::default();
    let key = config::token(dir.path())?;
    let send = |from: &str, to: &str, input: Value| {
        let (path, mut body) = request(from, &format!("{to}/m"), false);
        body.as_object_mut()
            .unwrap()
            .extend(input.as_object().unwrap().clone());
        client
            .post(format!("http://{}{path}", service.addr))
            .bearer_auth(&key)
            .body(body.to_string())
            .send()
    };

    let image = |format| match format {
        "chat" => (
            "/messages/0/content",
            json!({"type":"image_url","image_url":{"url":uri,"detail":"auto"}}),
            json!({"type":"text","text":"city?"}),
        ),
        "responses" => (
            "/input/0/content",
            json!({"type":"input_image","image_url":uri,"detail":"auto"}),
            json!({"type":"input_text","text":"city?"}),
        ),
        "anthropic" => (
            "/messages/0/content",
            json!({"type":"image","source":{"type":"base64","media_type":"image/png","data":PNG}}),
            json!({"type":"text","text":"city?"}),
        ),
        "gemini" => (
            "/contents/0/parts",
            json!({"inlineData":{"mimeType":"image/png","data":PNG}}),
            json!({"text":"city?"}),
        ),
        _ => unreachable!(),
    };
    for from in formats {
        let (_, image_part, prompt) = image(from);
        let parts = json!([image_part, prompt]);
        let input = match from {
            "chat" | "anthropic" => json!({"messages":[{"role":"user","content":parts}]}),
            "responses" => json!({"input":[{"role":"user","content":parts}]}),
            "gemini" => json!({"contents":[{"role":"user","parts":parts}]}),
            _ => unreachable!(),
        };
        for to in formats.into_iter().filter(|to| *to != from) {
            let res = send(from, to, input.clone()).await?;
            assert_eq!(res.status(), 200, "{from} -> {to}: {}", res.text().await?);
            let (pointer, image_part, prompt) = image(to);
            assert_eq!(
                captures.lock().unwrap().last().unwrap().pointer(pointer),
                Some(&json!([image_part, prompt]))
            );
        }
    }

    let chat_audio = json!({"type":"input_audio","input_audio":{"format":"wav","data":WAV}});
    let gemini_audio = json!({"inlineData":{"mimeType":"audio/wav","data":WAV}});
    let res = send("chat", "gemini", json!({"messages":[
        {"role":"user","content":[{"type":"text","text":"city?"},chat_audio]},
        {"role":"assistant","content":null,"tool_calls":[{"id":"call_1","type":"function","function":{"name":"record","arguments":"{}"}}]},
        {"role":"tool","tool_call_id":"call_1","content":[{"type":"text","text":"recorded"},chat_audio]}
    ]})).await?;
    assert_eq!(res.status(), 200, "{}", res.text().await?);
    {
        let bodies = captures.lock().unwrap();
        let sent = bodies.last().unwrap();
        assert_eq!(sent.pointer("/contents/0/parts/1"), Some(&gemini_audio));
        assert_eq!(
            sent.pointer("/contents/2/parts"),
            Some(&json!([
                {"functionResponse":{"id":"call_1","name":"record","response":{"output":"recorded"}}},
                gemini_audio
            ]))
        );
    }
    let res = send(
        "gemini",
        "chat",
        json!({"contents":[{"role":"user","parts":[
            {"text":"city?"},{"inline_data":{"mime_type":"audio/wav","data":WAV}}
        ]}]}),
    )
    .await?;
    assert_eq!(res.status(), 200, "{}", res.text().await?);
    assert_eq!(
        captures
            .lock()
            .unwrap()
            .last()
            .unwrap()
            .pointer("/messages/0/content/1"),
        Some(&chat_audio)
    );

    let remote = "https://example.invalid/city.png";
    let res = send(
        "responses",
        "anthropic",
        json!({"input":[{"role":"user","content":[
            {"type":"input_text","text":"city?"},{"type":"input_image","image_url":remote}
        ]}]}),
    )
    .await?;
    assert_eq!(res.status(), 200, "{}", res.text().await?);
    assert_eq!(
        captures
            .lock()
            .unwrap()
            .last()
            .unwrap()
            .pointer("/messages/0/content/1/source"),
        Some(&json!({"type":"url","url":remote}))
    );
    for (from, to, input, error) in [
        (
            "chat",
            "gemini",
            json!({"messages":[{"role":"user","content":[{"type":"image_url","image_url":{"url":remote}}]}]}),
            "requires base64 image data",
        ),
        (
            "responses",
            "chat",
            json!({"input":[{"role":"user","content":[{"type":"input_image","file_id":"file_123"}]}]}),
            "require a native endpoint",
        ),
        (
            "chat",
            "anthropic",
            json!({"messages":[{"role":"user","content":[{"type":"image_url","image_url":{"url":uri,"detail":"high"}}]}]}),
            "cannot preserve explicit image detail",
        ),
        (
            "chat",
            "responses",
            json!({"messages":[{"role":"user","content":[chat_audio]}]}),
            "cannot represent input audio",
        ),
    ] {
        let res = send(from, to, input).await?;
        assert_eq!(res.status(), 400);
        let body = res.text().await?;
        assert!(body.contains(error), "{body}");
    }
    assert_eq!(captures.lock().unwrap().len(), 15);
    service.stop().await;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn relay_uses_a_reachable_native_endpoint_for_the_actual_request() -> Result<()> {
    let upstream = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", upstream.local_addr()?);
    // The configured Chat URL would return a 404. Only Responses is native and reachable.
    let router = Router::new().route("/responses", post(vendor));
    let task =
        tokio::spawn(async move { crabforge_test_support::server::serve(upstream, router).await });
    let dir = tempfile::tempdir()?;
    let config = json!({"listen":"127.0.0.1:0", "providers":[{
        "id":"relay", "models":["m"], "endpoints":{"responses":base, "chat":base},
        "model_protocols":{"m":{"native_endpoints":["/v1/messages","/v1/responses"],"magpie_prefers":"/v1/messages"}}
    }]});
    config::save(
        dir.path(),
        &config::snapshot(dir.path())?,
        &config.to_string(),
    )?;
    let service = Service::start(dir.path()).await?;
    let (path, body) = request("chat", "relay/m", false);
    let r = crabforge_test_support::Client::default()
        .post(format!("http://{}{path}", service.addr))
        .bearer_auth(config::token(dir.path())?)
        .body(body.to_string())
        .send()
        .await?
        .error_for_status()?;
    let reply: Value = nextjson::from_slice(&r.bytes().await?).map_err(anyhow::Error::msg)?;
    assert_eq!(
        reply["choices"][0]["message"]["tool_calls"][0]["id"],
        json!("call_7")
    );
    service.stop().await;
    task.abort();
    Ok(())
}

async fn streaming_vendor(release: Arc<Notify>, Incoming { uri, body, .. }: Incoming) -> Response {
    let req: Value = nextjson::from_slice(&body).unwrap();
    let call = json!({"type":"function_call","call_id":"call_7","name":"weather","arguments":"{\"city\":\"上海\"}"});
    let (first, end) = match uri.path() {
        "/chat/completions" => (
            json!({"choices":[{"index":0,"delta":{"content":"先到"}}]}),
            vec![
                json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_7","function":{"name":"weather","arguments":"{\"city\":"}}]}}]}),
                json!({"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"上海\"}"}}]},"finish_reason":"tool_calls"}]}),
                json!({"choices":[],"usage":{"prompt_tokens":13,"completion_tokens":6}}),
            ],
        ),
        "/v1/messages" => (
            json!({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"先到"}}),
            vec![
                json!({"type":"content_block_stop","index":0}),
                json!({"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"call_7","name":"weather","input":{}}}),
                json!({"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"city\":\"上海\"}"}}),
                json!({"type":"content_block_stop","index":1}),
                json!({"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"input_tokens":13,"output_tokens":6}}),
                json!({"type":"message_stop"}),
            ],
        ),
        "/responses" => (
            json!({"type":"response.output_text.delta","output_index":0,"content_index":0,"delta":"先到"}),
            vec![
                json!({"type":"response.output_item.done","output_index":1,"item":call}),
                json!({"type":"response.completed","response":{"status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"先到"}]},call],"usage":{"input_tokens":13,"output_tokens":6}}}),
            ],
        ),
        "/relay/models/m:streamGenerateContent" => {
            assert_eq!(uri.query(), Some("alt=sse"));
            (
                json!({"candidates":[{"content":{"parts":[{"text":"先到"}]}}]}),
                vec![
                    json!({"candidates":[{"content":{"parts":[{"functionCall":{"id":"call_7","name":"weather","args":{"city":"上海"}}}]},"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":13,"candidatesTokenCount":6}}),
                ],
            )
        }
        path => panic!("unexpected streaming upstream {path}"),
    };
    if !uri.path().contains("streamGenerateContent") {
        assert_eq!(req["stream"], json!(true));
    }
    let chat = uri.path() == "/chat/completions";
    let thoughts = match uri.path() {
        "/chat/completions" => {
            vec![json!({"choices":[{"delta":{"reasoning_content":"private scratchpad"}}]})]
        }
        "/v1/messages" => vec![
            json!({"type":"content_block_start","index":3,"content_block":{"type":"thinking","thinking":""}}),
            json!({"type":"content_block_delta","index":3,"delta":{"type":"thinking_delta","thinking":"private scratchpad"}}),
            json!({"type":"content_block_delta","index":3,"delta":{"type":"signature_delta","signature":"private signature"}}),
            json!({"type":"content_block_stop","index":3}),
        ],
        "/responses" => vec![
            json!({"type":"response.reasoning_summary_text.delta","output_index":3,"delta":"private scratchpad"}),
        ],
        _ => vec![
            json!({"candidates":[{"content":{"parts":[{"thought":true,"text":"private scratchpad"}]}}]}),
        ],
    };
    let body = async_stream::stream! {
        for value in thoughts { yield Ok(format!("data: {value}\r\n\r\n").into_bytes()); }
        let first = format!(": heartbeat\r\ndata: {first}\r\n\r\n").into_bytes();
        // Separate the first Chinese code point across transport chunks.
        let at = first.iter().position(|&c| c >= 128).unwrap() + 1;
        yield Ok::<_,Infallible>(first[..at].to_vec());
        yield Ok(first[at..].to_vec());
        release.notified().await;
        for value in end { yield Ok(format!("data: {value}\n\n").into_bytes()); }
        if chat { yield Ok(b"data: [DONE]\n\n".to_vec()); }
    };
    (
        [("content-type", "text/event-stream; charset=utf-8")],
        Body::from_stream(body),
    )
        .into_response()
}

#[tokio::test]
async fn translated_text_precedes_upstream_completion_in_all_twelve_directions() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", listener.local_addr()?);
    let release = Arc::new(Notify::new());
    let signal = release.clone();
    let app = Router::new().fallback(post(move |request| {
        streaming_vendor(signal.clone(), request)
    }));
    let server =
        tokio::spawn(async move { crabforge_test_support::server::serve(listener, app).await });
    let dir = tempfile::tempdir()?;
    let formats = ["chat", "responses", "anthropic", "gemini"];
    let providers: Vec<_> = formats
        .iter()
        .map(|f| {
            // The relay's pasted SDK URL supplies its own path prefix.
            let url = if *f == "gemini" {
                format!("{base}/relay/models/m:generateContent")
            } else {
                base.clone()
            };
            json!({"id":f,"models":["m"],"endpoints":{*f:url}})
        })
        .collect();
    let cfg = json!({"listen":"127.0.0.1:0","providers":providers});
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let key = config::token(dir.path())?;
    let http = crabforge_test_support::Client::with_timeout(Duration::from_secs(6));
    for from in formats {
        for to in formats.into_iter().filter(|f| *f != from) {
            let (path, req) = request(from, &format!("{to}/m"), true);
            let mut resp = http
                .post(format!("http://{}{path}", service.addr))
                .bearer_auth(&key)
                .body(req.to_string())
                .send()
                .await?
                .error_for_status()?;
            let mut received = vec![];
            while !received
                .windows("先到".len())
                .any(|s| s == "先到".as_bytes())
            {
                received.extend_from_slice(&resp.chunk().await?.expect("missing first text delta"));
            }
            release.notify_one();
            received.extend_from_slice(&resp.bytes().await?);
            let wire = String::from_utf8(received)?;
            assert!(!wire.contains("private"), "{from} <- {to}: {wire}");
            for expected in ["call_7", "weather", "上海"] {
                assert!(wire.contains(expected), "{from} <- {to}: {wire}");
            }
            let frames: Vec<Value> = wire
                .lines()
                .filter_map(|s| s.strip_prefix("data: "))
                .filter(|s| *s != "[DONE]")
                .map(nextjson::from_str)
                .collect::<Result<_, _>>()
                .map_err(anyhow::Error::msg)?;
            let usage = match from {
                "chat" => &frames.last().unwrap()["usage"]["completion_tokens"],
                "anthropic" => &frames[frames.len() - 2]["usage"]["output_tokens"],
                "responses" => &frames.last().unwrap()["response"]["usage"]["output_tokens"],
                _ => &frames.last().unwrap()["usageMetadata"]["candidatesTokenCount"],
            };
            assert_eq!(usage, &json!(6), "{from} <- {to}");
        }
    }
    service.stop().await;
    server.abort();
    let _ = server.await;
    Ok(())
}

#[tokio::test]
async fn translated_reasoning_uses_the_target_wire_format_and_preserves_disabled_thinking()
-> Result<()> {
    use std::sync::Mutex;

    let captures = Arc::new(Mutex::new(Vec::new()));
    let seen = captures.clone();
    let router = Router::new().fallback(post(move |req: Incoming| {
        let seen = seen.clone();
        async move {
            let body: Value = nextjson::from_slice(&req.body).unwrap();
            seen.lock().unwrap().push(body);
            let answer = match req.uri.path() {
                "/chat/completions" => json!({"choices":[{"message":{"role":"assistant","content":"ready"},"finish_reason":"stop"}]}),
                "/responses" => json!({"status":"completed","output":[{"type":"reasoning","summary":[]},{"type":"message","role":"assistant","content":[{"type":"output_text","text":"ready"}]}]}),
                "/v1/messages" => json!({"content":[{"type":"thinking","thinking":"private","signature":"private"},{"type":"text","text":"ready"}],"stop_reason":"end_turn"}),
                "/v1beta/models/gemini-2.5-flash:generateContent" => json!({"candidates":[{"content":{"role":"model","parts":[{"thought":true,"text":"private"},{"text":"ready"}]},"finishReason":"STOP"}]}),
                other => panic!("unexpected upstream path {other}"),
            };
            Json(answer)
        }
    }));
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", listener.local_addr()?);
    let server = tokio::spawn(crabforge_test_support::server::serve(listener, router));
    let dir = tempfile::tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0","providers":[
        {"id":"chat","models":["m"],"endpoints":{"chat":&base},"model_reasoning_levels":{"m":["none","low","high"]}},
        {"id":"response","models":["m"],"endpoints":{"responses":&base},"model_reasoning_levels":{"m":["low","medium","high"]}},
        {"id":"anthropic","models":["claude-sonnet-4-6"],"endpoints":{"anthropic":&base},"model_reasoning_levels":{"claude-sonnet-4-6":["low","medium","high"]}},
        {"id":"google","models":["gemini-2.5-flash"],"endpoints":{"gemini":&base}},
    ],"groups":{"gemini":["google/gemini-2.5-flash"]},"group_rules":{"gemini":[{
        "use":"google/gemini-2.5-flash","useEffort":"xhigh","where":[{"field":"message","op":"starts","value":"city"}]
    }]}});
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let key = config::token(dir.path())?;
    let client = crabforge_test_support::Client::default();
    let send = |from: &str, model: &str, options: Value| {
        let (path, mut body) = request(from, model, false);
        body.as_object_mut()
            .unwrap()
            .extend(options.as_object().unwrap().clone());
        client
            .post(format!("http://{}{path}", service.addr))
            .bearer_auth(&key)
            .body(body.to_string())
            .send()
    };
    for (from, model, options, pointer, expected) in [
        (
            "chat",
            "group/gemini",
            json!({"reasoning_effort":"low"}),
            "/generationConfig/thinkingConfig/thinkingBudget",
            json!(24576),
        ),
        (
            "chat",
            "group/gemini",
            json!({"reasoning_effort":"none"}),
            "/generationConfig/thinkingConfig/thinkingBudget",
            json!(0),
        ),
        (
            "responses",
            "anthropic/claude-sonnet-4-6",
            json!({"reasoning":{"effort":"medium"}}),
            "/output_config/effort",
            json!("medium"),
        ),
        (
            "anthropic",
            "response/m",
            json!({"thinking":{"type":"enabled","budget_tokens":9000},"max_tokens":10000}),
            "/reasoning/effort",
            json!("medium"),
        ),
        (
            "gemini",
            "chat/m",
            json!({"generationConfig":{"thinkingConfig":{"thinkingLevel":"MEDIUM","includeThoughts":false}}}),
            "/reasoning_effort",
            json!("high"),
        ),
        (
            "gemini",
            "chat/m",
            json!({"generationConfig":{"thinkingConfig":{"thinkingBudget":0}}}),
            "/reasoning_effort",
            json!("none"),
        ),
    ] {
        let reply = send(from, model, options).await?;
        let status = reply.status();
        let body = reply.text().await?;
        assert_eq!(status, 200, "{from} -> {model}: {body}");
        assert!(
            body.contains("ready") && !body.contains("private"),
            "{body}"
        );
        assert_eq!(
            captures.lock().unwrap().last().unwrap().pointer(pointer),
            Some(&expected)
        );
    }
    let reject = send(
        "responses",
        "chat/m",
        json!({"reasoning":{"effort":"high","summary":"detailed"}}),
    )
    .await?;
    assert_eq!(reject.status(), 400);
    assert!(reject.text().await?.contains("reasoning summaries"));
    assert_eq!(captures.lock().unwrap().len(), 6);
    service.stop().await;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn translated_streams_recover_before_output_and_release_slots_after_disconnect() -> Result<()>
{
    use crabforge_test_support::server::StatusCode;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let release = Arc::new(Notify::new());
    let attempts = Arc::new(AtomicUsize::new(0));
    let header = [("content-type", "text/event-stream")];
    let good_attempts = attempts.clone();
    let upstream = Router::new()
        .route("/early/chat/completions", post(move |_| async move {
            (header, "data: {\"error\":{\"message\":\"invalid\"}}\n\n")
        }))
        .route("/partial/chat/completions", post({
            let release = release.clone();
            move |_| {
                let release = release.clone();
                async move {
                    let body = async_stream::stream! {
                        yield Ok::<_, Infallible>("data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n");
                        release.notified().await;
                        yield Ok("data: {\"choices\":");
                    };
                    (header, Body::from_stream(body))
                }
            }
        }))
        .route("/good/chat/completions", post(move |_| {
            good_attempts.fetch_add(1, Ordering::SeqCst);
            async move {(header, "data: {\"choices\":[{\"delta\":{\"content\":\"restored\"},\"finish_reason\":\"length\"}]}\n\ndata: [DONE]\n\n")}
        }));
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server =
        tokio::spawn(
            async move { crabforge_test_support::server::serve(listener, upstream).await },
        );
    let dir = tempfile::tempdir()?;
    let providers: Vec<_> = ["early", "partial", "good"]
        .iter()
        .map(
            |id| json!({"id":id,"models":["m"],"endpoints":{"chat":format!("http://{addr}/{id}")}}),
        )
        .collect();
    let cfg = json!({"listen":"127.0.0.1:0","providers":providers,"groups":{"recover":["early/m","good/m"],"cut":["partial/m","good/m"]}});
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let http = crabforge_test_support::Client::with_timeout(Duration::from_secs(10));
    let key = config::token(dir.path())?;
    let ask = |model| {
        let (path, body) = request("responses", model, true);
        http.post(format!("http://{}{path}", service.addr))
            .bearer_auth(&key)
            .body(body.to_string())
            .send()
    };
    let recovered = ask("group/recover")
        .await?
        .error_for_status()?
        .text()
        .await?;
    assert!(
        recovered.contains("restored")
            && recovered.contains("event: response.incomplete")
            && recovered.contains("max_output_tokens")
    );
    let mut partial = ask("group/cut").await?.error_for_status()?;
    let mut received = Vec::new();
    while !received.windows(7).any(|s| s == b"partial") {
        received.extend_from_slice(&partial.chunk().await?.expect("expected a text delta"));
    }
    release.notify_one();
    assert!(
        partial.bytes().await.is_err(),
        "truncated SSE must abort the downstream body"
    );
    assert_eq!(attempts.load(Ordering::SeqCst), 1);

    let mut held = Vec::new();
    for _ in 0..16 {
        held.push(ask("partial/m").await?.error_for_status()?);
    }
    assert_eq!(ask("good/m").await?.status(), StatusCode::TOO_MANY_REQUESTS);
    drop(held);
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let response = ask("good/m").await?;
            if response.status() != StatusCode::TOO_MANY_REQUESTS {
                let text = response.error_for_status()?.text().await?;
                assert!(text.contains("restored"));
                return Ok::<_, anyhow::Error>(());
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await??;
    service.stop().await;
    server.abort();
    Ok(())
}
