use super::*;
use anyhow::Context;
use crabforge_test_support::{
    Client,
    server::{Incoming, Router, get, post, serve_io},
};
use tokio::net::TcpListener;
use tokio_rustls::{TlsAcceptor, rustls};
use ureq::{
    Agent,
    config::Config as HttpConfig,
    tls::{Certificate, RootCerts, TlsConfig},
    unversioned::{
        resolver::{ResolvedSocketAddrs, Resolver},
        transport::NextTimeout,
    },
};

#[derive(Debug)]
struct FixtureAddress(&'static str, std::net::SocketAddr);

impl Resolver for FixtureAddress {
    fn resolve(
        &self,
        uri: &http::Uri,
        _: &HttpConfig,
        _: NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let mut addresses = self.empty();
        addresses.push(match uri.host() {
            Some("127.0.0.1") => {
                ([127, 0, 0, 1], uri.port_u16().expect("fixture proxy port")).into()
            }
            Some(host) if host == self.0 => self.1,
            host => panic!("unexpected fixture host {host:?}"),
        });
        Ok(addresses)
    }
}

fn tls_fixture(
    host: &'static str,
    address: std::net::SocketAddr,
    bypass: Option<no_proxy::NoProxy>,
) -> Result<(ForwardClient, TlsAcceptor)> {
    let key = rcgen::KeyPair::generate()?;
    let certificate = rcgen::CertificateParams::new(vec![host.to_owned()])?.self_signed(&key)?;
    let cert = certificate.der();
    let tls = TlsConfig::builder()
        .root_certs(RootCerts::Specific(Arc::new(vec![
            Certificate::from_der(cert).to_owned(),
        ])))
        .build();
    let config = Agent::config_builder()
        .proxy(None)
        .http_status_as_error(false)
        .max_redirects(0)
        .tls_config(tls)
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    let client =
        ForwardClient::with_resolver(bypass, config, move || FixtureAddress(host, address));
    let server = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_no_client_auth()
    .with_single_cert(
        vec![cert.clone()],
        rustls_pki_types::PrivatePkcs8KeyDer::from(key.serialize_der()).into(),
    )?;
    Ok((client, TlsAcceptor::from(Arc::new(server))))
}

#[tokio::test]
async fn console_go_forwards_untranslatable_input_in_its_original_protocol() -> Result<()> {
    const BODY: &str = r#"{ "model": "go/minimax-m2.7", "messages": [{"role":"user","content":[{"type":"image_url","image_url":{"url":"data:image/png;base64,aGk=","detail":"high"}}]}] }"#;
    const REPLY: &str = r#"{"choices":[{"message":{"content":"received"}}]}"#;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let bypass =
        std::env::var_os("CRABFORGE_TEST_PROXY_ROUTE").map(|_| ForwardClient::environment_bypass());
    let (client, tls) = tls_fixture("opencode.ai", address, bypass)?;
    let upstream = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await?;
        let tls = tls.accept(tcp).await?;
        let upstream = Router::new().route(
            "/inference/go/openai/v1/chat/completions",
            post(|Incoming { headers, body, .. }| async move {
                assert_eq!(headers["x-opencode-session"], "test-turn");
                assert_eq!(
                    headers[header::USER_AGENT],
                    crate::opencode::USER_AGENT_VALUE
                );
                assert!(!headers.contains_key(header::AUTHORIZATION));
                assert_eq!(body, BODY.replace("go/minimax-m2.7", "minimax-m2.7"));
                ([("connection", "close")], REPLY)
            }),
        );
        serve_io(tls, Arc::new(upstream)).await;
        Ok::<_, anyhow::Error>(())
    });
    let cfg = nextjson::from_value(
        json!({"providers":[{"id":"go","models":["minimax-m2.7"],"endpoints":{"chat":format!("https://opencode.ai:{}/inference/go/openai/v1",address.port())}}]}),
    ).map_err(anyhow::Error::msg)?;
    let root = tempfile::tempdir()?;
    let router = router_with_client(
        cfg,
        "gateway-key".into(),
        Default::default(),
        root.path(),
        client,
    )?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let gateway = listener.local_addr()?;
    let task = tokio::spawn(crate::service::serve_local(listener, router));
    let reply = Client::with_timeout(Duration::from_secs(8))
        .post(format!("http://{gateway}/v1/chat/completions"))
        .bearer_auth("gateway-key")
        .header("x-opencode-session", "test-turn")
        .body(BODY)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    task.abort();
    upstream.await??;
    assert_eq!(reply, REPLY);
    Ok(())
}

#[tokio::test]
async fn minimax_gateway_retries_a_vendor_rate_limit_on_the_next_group_member() -> Result<()> {
    let upstream = TcpListener::bind("127.0.0.1:0").await?;
    let (client, tls) = tls_fixture("api.minimax.io", upstream.local_addr()?, None)?;
    let upstream = tokio::spawn(async move {
        for model in ["image-01-live", "image-01"] {
            let (tcp, _) = upstream.accept().await?;
            let socket = tls.accept(tcp).await?;
            let routes = Router::new().route(
                "/v1/image_generation",
                post(move |Incoming { headers, body, .. }| async move {
                    let request: nextjson::Value = nextjson::from_slice(&body).unwrap();
                    assert_eq!(request["model"], json!(model));
                    assert_eq!(request["prompt"], json!("A field of blue flowers"));
                    assert_eq!(request["aspect_ratio"], json!("16:9"));
                    assert_eq!(request["response_format"], json!("url"));
                    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
                    assert!(!headers.contains_key(header::AUTHORIZATION));
                    let reply = if model == "image-01-live" {
                        json!({"base_resp":{"status_code":1002}})
                    } else {
                        json!({"base_resp":{"status_code":0},"data":{"image_urls":["https://example.com/flowers.png"]}})
                    };
                    ([("connection", "close")], reply.to_string()).into_response()
                }),
            );
            serve_io(socket, Arc::new(routes)).await;
        }
        Ok::<_, anyhow::Error>(())
    });
    let config = nextjson::from_value(json!({
        "providers":[{"id":"images","models":["image-01-live","image-01"],
                      "endpoints":{"chat":"https://api.minimax.io/v1"}}],
        "groups":{"draw":["images/image-01-live","images/image-01"]}
    }))
    .map_err(anyhow::Error::msg)?;
    let root = tempfile::tempdir()?;
    let history = History::default();
    let router = router_with_client(
        config,
        "gateway-key".into(),
        history.clone(),
        root.path(),
        client,
    )?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let gateway = tokio::spawn(crate::service::serve_local(listener, router));
    let response = Client::with_timeout(Duration::from_secs(8))
        .post(format!("http://{addr}/v1/images/generations"))
        .bearer_auth("gateway-key")
        .body(
            json!({"model":"group/draw","size":"16:9","prompt":"A field of blue flowers"})
                .to_string(),
        )
        .send()
        .await;
    let body = response?.error_for_status()?.bytes().await?;
    gateway.abort();
    tokio::time::timeout(Duration::from_secs(8), upstream).await???;
    let response: nextjson::Value = nextjson::from_slice(&body).map_err(anyhow::Error::msg)?;
    assert_eq!(
        response.pointer("/data/0/url"),
        Some(&json!("https://example.com/flowers.png"))
    );
    assert_eq!(
        history
            .lock()
            .unwrap()
            .iter()
            .map(|attempt| attempt.status)
            .collect::<Vec<_>>(),
        [429, 200]
    );
    Ok(())
}

#[tokio::test]
async fn ark_gateway_rewrites_multipart_edits_and_keeps_successful_draws() -> Result<()> {
    const HOST: &str = "ark.cn-beijing.volces.com";
    const MODEL: &str = "ep-image-fixture";
    const SEEDREAM: &str = "doubao-seedream-5-0-pro";
    const PNG: &str = "iVBORw0KGgpmaXh0dXJl";
    let upstream = TcpListener::bind("127.0.0.1:0").await?;
    let (client, tls) = tls_fixture(HOST, upstream.local_addr()?, None)?;
    let upstream = tokio::spawn(async move {
        for i in 0..5 {
            let (tcp, _) = upstream.accept().await?;
            let socket = tls.accept(tcp).await?;
            let routes = Router::new().route(
                "/api/v3/images/generations",
                post(move |Incoming { headers, body, .. }| async move {
                    let request: nextjson::Value = nextjson::from_slice(&body).unwrap();
                    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
                    assert!(request.get("resolution").is_none());
                    if i >= 3 {
                        assert_eq!(request["model"], json!(SEEDREAM));
                        assert_eq!(request["size"], json!(if i == 3 { "2816x1584" } else { "2848x1600" }));
                        assert_eq!(request["response_format"], json!("url"));
                        let (status, reply) = if i == 3 {
                            (StatusCode::UNPROCESSABLE_ENTITY, json!({"error":{"message":"unsupported size"}}))
                        } else {
                            (StatusCode::OK, json!({"data":[{"url":"https://example.com/drawn.png"}]}))
                        };
                        return (status, [("connection", "close"), ("x-request-id", "ark-tier-fixture")], reply.to_string()).into_response();
                    }
                    assert_eq!(request["model"], json!(MODEL));
                    assert_eq!(request["size"], json!("2848x1600"));
                    assert_eq!(request["watermark"], json!(true));
                    assert_eq!(request["response_format"], json!("b64_json"));
                    assert_eq!(request.pointer("/sequential_image_generation_options/max_images"), Some(&json!(3)));
                    assert_eq!(request["image"], json!([format!("data:image/png;base64,{PNG}"), "https://example.com/second.png"]));
                    assert!(request.get("n").is_none());
                    let (status, reply) = if i == 1 {
                        (StatusCode::TOO_MANY_REQUESTS, json!({"error":{"message":"busy"}}))
                    } else {
                        (StatusCode::OK, json!({"data":[{"b64_json":PNG,"sample":i}],"usage":{"generated_images":1,"total_tokens":4}}))
                    };
                    (status, [("connection", "close"), ("x-request-id", "ark-fixture")], reply.to_string()).into_response()
                }),
            );
            serve_io(socket, Arc::new(routes)).await;
        }
        Ok::<_, anyhow::Error>(())
    });
    let config = nextjson::from_value(json!({
        "providers":[{"id":"ark","models":[MODEL,SEEDREAM],"endpoints":{"responses":format!("https://{HOST}/api/v3")}}]
    })).map_err(anyhow::Error::msg)?;
    let root = tempfile::tempdir()?;
    let router = router_with_client(
        config,
        "gateway-key".into(),
        Default::default(),
        root.path(),
        client,
    )?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let gateway = tokio::spawn(crate::service::serve_local(listener, router));
    let body = concat!(
        "--fixture\r\nContent-Disposition: form-data; name=\"model\"\r\n\r\nark/ep-image-fixture\r\n",
        "--fixture\r\nContent-Disposition: form-data; name=\"prompt\"\r\n\r\nA portrait\r\n",
        "--fixture\r\nContent-Disposition: form-data; name=\"size\"\r\n\r\n16:9\r\n",
        "--fixture\r\nContent-Disposition: form-data; name=\"n\"\r\n\r\n3\r\n",
        "--fixture\r\nContent-Disposition: form-data; name=\"watermark\"\r\n\r\ntrue\r\n",
        "--fixture\r\nContent-Disposition: form-data; name=\"image[]\"\r\n\r\ndata:image/png;base64,iVBORw0KGgpmaXh0dXJl\r\n",
        "--fixture\r\nContent-Disposition: form-data; name=\"image[]\"\r\n\r\nhttps://example.com/second.png\r\n",
        "--fixture\r\nContent-Disposition: form-data; name=\"sequential_image_generation_options\"\r\n\r\n{\"max_images\":3}\r\n",
        "--fixture--\r\n",
    );
    let reply = Client::with_timeout(Duration::from_secs(8))
        .post(format!("http://{addr}/v1/images/edits"))
        .bearer_auth("gateway-key")
        .header(
            header::CONTENT_TYPE.as_str(),
            "multipart/form-data; boundary=fixture",
        )
        .body(body)
        .send()
        .await?
        .error_for_status()?;
    assert_eq!(reply.headers()["x-request-id"], "ark-fixture");
    let value: nextjson::Value =
        nextjson::from_slice(&reply.bytes().await?).map_err(anyhow::Error::msg)?;
    let tiered = Client::with_timeout(Duration::from_secs(8))
        .post(format!("http://{addr}/v1/images/generations"))
        .bearer_auth("gateway-key")
        .header("content-type", "application/json")
        .body(json!({"model":format!("ark/{SEEDREAM}"), "prompt":"A wide scene", "size":"16:9", "resolution":"4k", "response_format":"url"}).to_string())
        .send()
        .await?
        .error_for_status()?;
    assert_eq!(tiered.headers()["x-request-id"], "ark-tier-fixture");
    let tiered: nextjson::Value =
        nextjson::from_slice(&tiered.bytes().await?).map_err(anyhow::Error::msg)?;
    assert_eq!(
        tiered.pointer("/data/0/url"),
        Some(&json!("https://example.com/drawn.png"))
    );
    gateway.abort();
    tokio::time::timeout(Duration::from_secs(8), upstream).await???;
    assert_eq!(
        value["data"],
        json!([{"b64_json":PNG,"sample":0},{"b64_json":PNG,"sample":2}])
    );
    assert_eq!(value.pointer("/usage/generated_images"), Some(&json!(2)));
    assert_eq!(value.pointer("/usage/total_tokens"), Some(&json!(8)));
    Ok(())
}

#[tokio::test]
async fn ark_video_maps_seedance_tasks_and_streams_the_signed_result() -> Result<()> {
    use crabforge_test_support::server::Json;
    const HOST: &str = "ark.cn-beijing.volces.com";
    const MODEL: &str = "doubao-seedance-2-0-260128";
    let upstream = TcpListener::bind("127.0.0.1:0").await?;
    let address = upstream.local_addr()?;
    let (client, tls) = tls_fixture(HOST, address, None)?;
    let origin = format!("https://{HOST}:{}", address.port());
    let asset = format!("{origin}/film.mp4?signature=fixture");
    let server = tokio::spawn(async move {
        for step in 0..5 {
            let (tcp, _) = upstream.accept().await?;
            let socket = tls.accept(tcp).await?;
            let asset = asset.clone();
            let routes = Router::new()
                .route("/api/v3/contents/generations/tasks", post(move |Incoming { body, .. }| async move {
                    assert_eq!(step, 0);
                    let request: nextjson::Value = nextjson::from_slice(&body).unwrap();
                    assert_eq!(request["model"], json!(MODEL));
                    assert_eq!(request["duration"], json!(-1));
                    assert_eq!(request["resolution"], json!("4k"));
                    assert_eq!(request["ratio"], json!("16:9"));
                    assert_eq!(request.pointer("/content/0/text"), Some(&json!("A cloud crosses the sky")));
                    assert_eq!(request.pointer("/content/1/role"), Some(&json!("first_frame")));
                    assert_eq!(request.pointer("/content/2/role"), Some(&json!("last_frame")));
                    assert_eq!(request.pointer("/content/3/image_url/url"), Some(&json!("https://example.test/reference.png")));
                    assert_eq!(request["return_last_frame"], json!(true));
                    ([("connection", "close")], Json(json!({"id":"seedance-task-1"})))
                }))
                .route("/api/v3/contents/generations/tasks/seedance-task-1", get(move |Incoming { headers, .. }| {
                    let asset = asset.clone();
                    async move {
                        assert!(!headers.contains_key("range"));
                        let reply = match step {
                            1 => json!({"id":"seedance-task-1","status":"running","progress":20}),
                            2 => json!({"id":"seedance-task-1","status":"succeeded","content":{"video_url":"http://127.0.0.1/unrelated"}}),
                            3 => json!({"id":"seedance-task-1","status":"succeeded","duration":4,"content":{"video_url":asset}}),
                            _ => panic!("unexpected video status request {step}"),
                        };
                        ([("connection", "close")], Json(reply))
                    }
                }))
                .route("/film.mp4", get(move |Incoming { uri, headers, .. }| async move {
                    assert_eq!(step, 4);
                    assert_eq!(uri.query(), Some("signature=fixture"));
                    assert_eq!(headers["range"], "bytes=2-4");
                    assert!(!headers.contains_key("authorization"));
                    (StatusCode::PARTIAL_CONTENT, [("connection","close"), ("content-type","video/mp4"), ("content-range","bytes 2-4/5")], Bytes::from_static(b"\0\x01\xff"))
                }));
            serve_io(socket, Arc::new(routes)).await;
        }
        Ok::<_, anyhow::Error>(())
    });
    let config = nextjson::from_value(json!({"providers":[{
        "id":"ark", "models":[MODEL], "endpoints":{"chat":origin}
    }]}))
    .map_err(anyhow::Error::msg)?;
    let root = tempfile::tempdir()?;
    let router = router_with_client(
        config,
        "gateway-key".into(),
        Default::default(),
        root.path(),
        client,
    )?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let gateway = tokio::spawn(crate::service::serve_local(listener, router));
    let client = Client::with_timeout(Duration::from_secs(8));
    let path = format!("http://{addr}/v1/videos");
    let create = client.post(&path).bearer_auth("gateway-key").body(json!({
        "model":format!("ark/{MODEL}"), "prompt":"A cloud crosses the sky", "size":"3840x2160", "seconds":"-1",
        "input_reference":"https://example.test/start.png", "last_frame":{"url":"https://example.test/end.png"},
        "reference_images":["https://example.test/reference.png"], "return_last_frame":true
    }).to_string()).send().await?.error_for_status()?;
    let created: nextjson::Value =
        nextjson::from_slice(&create.bytes().await?).map_err(anyhow::Error::msg)?;
    assert_eq!(created["status"], json!("queued"));
    let id = created["id"].as_str().unwrap();
    let content = format!("{path}/{id}/content");
    assert_eq!(
        client
            .get(&content)
            .bearer_auth("gateway-key")
            .send()
            .await?
            .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        client
            .get(&content)
            .bearer_auth("gateway-key")
            .send()
            .await?
            .status(),
        StatusCode::BAD_GATEWAY
    );
    let result = client
        .get(&content)
        .header("range", "bytes=2-4")
        .bearer_auth("gateway-key")
        .send()
        .await?;
    assert_eq!(result.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(result.headers()["content-range"], "bytes 2-4/5");
    assert_eq!(result.bytes().await?, Bytes::from_static(b"\0\x01\xff"));
    gateway.abort();
    tokio::time::timeout(Duration::from_secs(8), server).await???;
    Ok(())
}

#[tokio::test]
async fn vendor_video_tasks_use_native_paths_and_stream_content() -> Result<()> {
    use crabforge_test_support::server::Json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    for (host, model, create, poll) in [
        (
            "api.x.ai",
            "grok-imagine-video",
            "/v1/videos/generations",
            "/v1/videos/fixture-1",
        ),
        (
            "api.minimax.cn",
            "MiniMax-H3-Max",
            "/v2/video_generation",
            "/v2/query/video_generation/fixture-1",
        ),
        (
            "dashscope.aliyuncs.com",
            "wan3.0-video",
            "/api/v1/services/aigc/video-generation/video-synthesis",
            "/api/v1/tasks/fixture-1",
        ),
    ] {
        let vendor = TcpListener::bind("127.0.0.1:0").await?;
        let addr = vendor.local_addr()?;
        let (forward, tls) = tls_fixture(host, addr, None)?;
        let origin = format!("https://{host}:{}", addr.port());
        let asset = format!("{origin}/film.mp4?signature=fixture");
        let minimax = host == "api.minimax.cn";
        let server = tokio::spawn(async move {
            let polls = AtomicUsize::new(0);
            let routes = Router::new()
                .route(create, post(move |Incoming { headers, body, .. }| async move {
                    let fields: nextjson::Value = nextjson::from_slice(&body).unwrap();
                    let fallback = minimax && fields["model"] == json!("MiniMax-H3-Lite");
                    if !fallback {
                        assert_eq!(fields["model"], json!(model));
                    }
                    let reply = match host {
                        "api.x.ai" => {
                            assert_eq!(fields["duration"], json!(4));
                            assert_eq!(fields["resolution"], json!("720p"));
                            assert_eq!(fields["aspect_ratio"], json!("9:16"));
                            assert_eq!(fields.pointer("/image/url"), Some(&json!("https://example.test/start.png")));
                            json!({"request_id":"fixture-1"})
                        }
                        "api.minimax.cn" => {
                            assert_eq!(fields["duration"], json!(if fallback { 4 } else { 5 }));
                            assert_eq!(fields["resolution"], json!("768P"));
                            assert_eq!(fields.pointer("/content/1/role"), Some(&json!("first_frame")));
                            assert!(fields.get("ratio").is_none());
                            if fallback || fields.pointer("/content/0/text") == Some(&json!("Busy")) {
                                json!({"base_resp":{"status_code":1002}})
                            } else {
                                json!({"task_id":"fixture-1","base_resp":{"status_code":0}})
                            }
                        }
                        _ => {
                            assert_eq!(headers["x-dashscope-async"], "enable");
                            assert_eq!(fields.pointer("/input/media/0/type"), Some(&json!("first_frame")));
                            assert_eq!(fields.pointer("/parameters/resolution"), Some(&json!("1080P")));
                            json!({"output":{"task_id":"fixture-1","task_status":"PENDING"}})
                        }
                    };
                    ([("connection", "close")], Json(reply))
                }))
                .route(poll, get(move |Incoming { headers, .. }| {
                    let asset = asset.clone();
                    let id = if polls.fetch_add(1, Ordering::Relaxed) == 0 { "different-job" } else { "fixture-1" };
                    async move {
                        assert!(!headers.contains_key("range"));
                        let reply = match host {
                            "api.x.ai" if id == "fixture-1" => json!({"status":"done","video":{"duration":5,"url":asset}}),
                            "api.x.ai" => json!({"request_id":id,"status":"done","video":{"duration":5,"url":asset}}),
                            "api.minimax.cn" => json!({"task":{"id":id,"status":"succeeded","duration":5,"content":{"url":asset}}}),
                            _ => json!({"output":{"task_id":id,"task_status":"SUCCEEDED","video_url":asset},"usage":{"duration":5}}),
                        };
                        ([("connection", "close")], Json(reply))
                    }
                }))
                .route("/film.mp4", get(|Incoming { headers, uri, .. }| async move {
                    assert_eq!(uri.query(), Some("signature=fixture"));
                    assert_eq!(headers["range"], "bytes=1-2");
                    assert!(!headers.contains_key("authorization"));
                    assert!(!headers.contains_key("x-api-key"));
                    (StatusCode::PARTIAL_CONTENT, [("connection","close"), ("content-range","bytes 1-2/3"), ("content-type","video/mp4")], Bytes::from_static(b"\x01\xff"))
                }));
            let routes = Arc::new(routes);
            for _ in 0..(5 + 2 * usize::from(minimax)) {
                let (tcp, _) = vendor.accept().await?;
                serve_io(tls.accept(tcp).await?, routes.clone()).await;
            }
            Ok::<_, anyhow::Error>(())
        });
        let mut models = vec!["text", model];
        let mut members = vec!["vendor/text".to_owned()];
        if minimax {
            models.push("MiniMax-H3-Lite");
            members.push("vendor/MiniMax-H3-Lite".into());
        }
        let direct = format!("vendor/{model}");
        members.push(direct.clone());
        let config = nextjson::from_value(json!({
            "providers":[{"id":"vendor","models":models,"patience":-1,"endpoints":{"chat":origin}}],
            "groups":{"films":members},
            "group_patience":{"films":-1}
        }))
        .map_err(anyhow::Error::msg)?;
        let root = tempfile::tempdir()?;
        let router = router_with_client(
            config,
            "gateway-key".into(),
            Default::default(),
            root.path(),
            forward,
        )?;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let base = format!("http://{}/v1", listener.local_addr()?);
        let gateway = tokio::spawn(crate::service::serve_local(listener, router));
        let client = Client::with_timeout(Duration::from_secs(8));
        let catalog = client
            .get(format!("{base}/models"))
            .bearer_auth("gateway-key")
            .send()
            .await?
            .error_for_status()?;
        let catalog: nextjson::Value =
            nextjson::from_slice(&catalog.bytes().await?).map_err(anyhow::Error::msg)?;
        let catalog = catalog["data"].as_array().unwrap();
        for id in [format!("vendor/{model}"), "group/films".to_owned()] {
            let row = catalog
                .iter()
                .find(|row| row["id"] == json!(id))
                .context("missing video model or group")?;
            assert_eq!(row["native_endpoints"], json!(["/v1/videos"]));
        }
        let create = |route: &str, prompt| {
            client
                .post(format!("{base}/videos"))
                .bearer_auth("gateway-key")
                .body(
                    json!({
                        "model":route, "prompt":prompt, "seconds":4, "size":"1080x1920",
                        "input_reference":"https://example.test/start.png"
                    })
                    .to_string(),
                )
        };
        if minimax {
            assert_eq!(
                create(&direct, "Busy").send().await?.status(),
                StatusCode::TOO_MANY_REQUESTS
            );
        }
        let created = create("group/films", "Clouds over a lake")
            .send()
            .await?
            .error_for_status()?;
        let created: nextjson::Value =
            nextjson::from_slice(&created.bytes().await?).map_err(anyhow::Error::msg)?;
        assert_eq!(created["status"], json!("queued"));
        assert_eq!(created["model"], json!(direct));
        let job = format!("{base}/videos/{}", created["id"].as_str().unwrap());
        assert_eq!(
            client
                .get(&job)
                .bearer_auth("gateway-key")
                .send()
                .await?
                .status(),
            StatusCode::BAD_GATEWAY
        );
        let status = client
            .get(&job)
            .bearer_auth("gateway-key")
            .send()
            .await?
            .error_for_status()?;
        let status: nextjson::Value =
            nextjson::from_slice(&status.bytes().await?).map_err(anyhow::Error::msg)?;
        assert_eq!(status["status"], json!("completed"));
        assert_eq!(status["seconds"], json!("5"));
        let video = client
            .get(format!("{job}/content"))
            .bearer_auth("gateway-key")
            .header("range", "bytes=1-2")
            .send()
            .await?;
        assert_eq!(video.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(video.headers()["content-range"], "bytes 1-2/3");
        assert_eq!(video.bytes().await?, Bytes::from_static(b"\x01\xff"));
        gateway.abort();
        tokio::time::timeout(Duration::from_secs(8), server).await???;
    }
    Ok(())
}

#[tokio::test]
async fn subscription_usage_keeps_the_plan_key_on_the_vendor_and_rejects_redirects() -> Result<()> {
    use crabforge_test_support::server::Json;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let (client, tls) = tls_fixture("opencode.ai", listener.local_addr()?, None)?;
    let server = tokio::spawn(async move {
        for step in 0..2 {
            let (tcp, _) = listener.accept().await?;
            let socket = tls.accept(tcp).await?;
            let app = Router::new().route("/zen/go/v1/usage", get(move |Incoming { headers, body, .. }| async move {
                assert_eq!(headers["authorization"], "Bearer fixture-only");
                assert_eq!(headers["accept-language"], "en-US,en");
                assert!(!headers["x-opencode-session"].is_empty());
                assert!(body.is_empty());
                let (status, reply) = if step == 0 {
                    (StatusCode::OK, json!({"usage":{"rolling":{"percent":42,"resetsAt":"2026-11-29T12:00:00Z"}}}))
                } else {
                    (StatusCode::FOUND, json!({}))
                };
                (status, [("connection", "close"), ("location", "/never")], Json(reply))
            }));
            serve_io(socket, Arc::new(app)).await;
        }
        Ok::<_, anyhow::Error>(())
    });
    let config = Config::parse(
        r#"{"providers":[{"id":"go","models":["fixture"],"key_env":"GO_FIXTURE","endpoints":{"chat":"https://opencode.ai/inference/go/openai/v1"}}]}"#,
    )?;
    assert_eq!(crate::usage::available(&config), ["go"]);
    let provider = &config.providers[0];
    let report = crate::usage::report(provider, &client, "fixture-only").await?;
    assert_eq!(report.windows[0].used_percent, Some(42.0));
    let error = crate::usage::report(provider, &client, "fixture-only")
        .await
        .err()
        .context("redirect was accepted")?;
    assert!(error.to_string().contains("HTTP 302"));
    tokio::time::timeout(Duration::from_secs(8), server).await???;
    Ok(())
}

#[tokio::test]
async fn modelscope_gateway_polls_images_and_keeps_partial_results() -> Result<()> {
    const HOST: &str = "api-inference.modelscope.cn";
    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfixture";
    const KEY_ENV: &str = "CRABFORGE_TEST_MODELSCOPE_BEARER";
    const KEY: &str = "modelscope-fixture-key";
    if std::env::var(KEY_ENV).as_deref() != Ok(KEY) {
        let output = tokio::process::Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "gateway::console_test::modelscope_gateway_polls_images_and_keeps_partial_results",
            ])
            .env(KEY_ENV, KEY)
            .output()
            .await?;
        anyhow::ensure!(
            output.status.success(),
            "ModelScope credential fixture failed: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return Ok(());
    }
    let upstream = TcpListener::bind("127.0.0.1:0").await?;
    let address = upstream.local_addr()?;
    let (client, tls) = tls_fixture(HOST, address, None)?;
    let origin = format!("https://{HOST}:{}", address.port());
    let asset = format!("{origin}/result.png");
    let upstream = tokio::spawn(async move {
        for index in 0..6 {
            let (tcp, _) = upstream.accept().await?;
            let socket = tls.accept(tcp).await?;
            let asset = asset.clone();
            let routes = Router::new()
                .route("/v1/images/generations", post(move |Incoming { headers, body, .. }| async move {
                    assert_eq!(headers["x-modelscope-async-mode"], "true");
                    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
                    assert_eq!(headers[header::AUTHORIZATION], concat!("Bearer ", "modelscope-fixture-key"));
                    let request: nextjson::Value = nextjson::from_slice(&body).unwrap();
                    assert_eq!(request, json!({"model":"Qwen/Image-Edit","prompt":"Draw a crab","size":"1024x1024","image_url":["data:image/png;base64,iVBORw0KGgpmaXh0dXJl","https://example.com/second.png"]}));
                    let (status, reply) = match index {
                        0 => (StatusCode::TOO_MANY_REQUESTS, json!({"message":"user queue is full"})),
                        1 => (StatusCode::OK, json!({"task_id":"drawing-1"})),
                        5 => (StatusCode::SERVICE_UNAVAILABLE, json!({"message":"service unavailable"})),
                        _ => panic!("unexpected image start at step {index}"),
                    };
                    (status, [("connection", "close")], reply.to_string())
                }))
                .route("/v1/tasks/drawing-1", get(move |Incoming { headers, body, .. }| {
                    let asset = asset.clone();
                    async move {
                        assert_eq!(headers["x-modelscope-task-type"], "image_generation");
                        assert!(!headers.contains_key("x-modelscope-async-mode"));
                        assert_eq!(headers[header::AUTHORIZATION], concat!("Bearer ", "modelscope-fixture-key"));
                        assert!(body.is_empty());
                        let reply = match index {
                            2 => json!({"task_status":"RUNNING"}),
                            3 => json!({"task_status":"SUCCEED","output_images":[asset]}),
                            _ => panic!("unexpected image poll at step {index}"),
                        };
                        ([("connection", "close")], reply.to_string())
                    }
                }))
                .route("/result.png", get(move |Incoming { headers, .. }| async move {
                    assert_eq!(index, 4);
                    assert!(!headers.contains_key(header::AUTHORIZATION));
                    ([("connection", "close")], Bytes::from_static(PNG))
                }));
            serve_io(socket, Arc::new(routes)).await;
        }
        Ok::<_, anyhow::Error>(())
    });
    let config = nextjson::from_value(json!({"providers":[{
        "id":"modelscope","key_env":KEY_ENV,"models":["Qwen/Image-Edit"],"endpoints":{"chat":format!("{origin}/v1")}
    }]}))
    .map_err(anyhow::Error::msg)?;
    let root = tempfile::tempdir()?;
    let router = router_with_client(
        config,
        "gateway-key".into(),
        Default::default(),
        root.path(),
        client,
    )?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let gateway = tokio::spawn(crate::service::serve_local(listener, router));
    let body = [
        b"--fixture\r\nContent-Disposition: form-data; name=\"model\"\r\n\r\nmodelscope/Qwen/Image-Edit\r\n\
          --fixture\r\nContent-Disposition: form-data; name=\"prompt\"\r\n\r\nDraw a crab\r\n\
          --fixture\r\nContent-Disposition: form-data; name=\"size\"\r\n\r\n1024x1024\r\n\
          --fixture\r\nContent-Disposition: form-data; name=\"n\"\r\n\r\n2\r\n\
          --fixture\r\nContent-Disposition: form-data; name=\"response_format\"\r\n\r\nb64_json\r\n\
          --fixture\r\nContent-Disposition: form-data; name=\"image[]\"; filename=\"first.png\"\r\nContent-Type: image/png\r\n\r\n".as_slice(),
        PNG,
        b"\r\n--fixture\r\nContent-Disposition: form-data; name=\"image[]\"\r\n\r\nhttps://example.com/second.png\r\n--fixture--\r\n",
    ].concat();
    let reply = Client::with_timeout(Duration::from_secs(15))
        .post(format!("http://{addr}/v1/images/edits"))
        .bearer_auth("gateway-key")
        .header("content-type", "multipart/form-data; boundary=fixture")
        .body_bytes(body)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    gateway.abort();
    tokio::time::timeout(Duration::from_secs(5), upstream).await???;
    let value: nextjson::Value = nextjson::from_slice(&reply).map_err(anyhow::Error::msg)?;
    assert_eq!(
        value.get("data"),
        Some(&json!([{"b64_json":"iVBORw0KGgpmaXh0dXJl"}]))
    );
    Ok(())
}

#[tokio::test]
async fn https_response_crosses_connect_and_no_proxy_bypasses_it() -> Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, copy_bidirectional};
    let proxy = TcpListener::bind("127.0.0.1:0").await?;
    let address = proxy.local_addr()?;
    let relay = tokio::spawn(async move {
        let (socket, _) = proxy.accept().await?;
        let mut reader = BufReader::new(socket);
        let mut request = String::new();
        reader.read_line(&mut request).await?;
        let port: u16 = request
            .trim_end()
            .strip_prefix("CONNECT opencode.ai:")
            .and_then(|line| line.strip_suffix(" HTTP/1.1"))
            .expect("CONNECT authority")
            .parse()?;
        loop {
            let mut line = String::new();
            anyhow::ensure!(reader.read_line(&mut line).await? > 0, "incomplete CONNECT");
            if line == "\r\n" {
                break;
            }
        }
        let mut upstream =
            tokio::net::TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, port)).await?;
        let mut socket = reader.into_inner();
        socket
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await?;
        copy_bidirectional(&mut socket, &mut upstream).await?;
        Ok::<_, anyhow::Error>(proxy)
    });
    // Keep the test runner's and other requests' proxy environment intact.
    let exchange = |bypass: &'static str| async move {
        let output = tokio::process::Command::new(std::env::current_exe()?)
            .args(["--exact", "gateway::console_test::console_go_forwards_untranslatable_input_in_its_original_protocol"])
            .env_remove("https_proxy").env_remove("no_proxy")
            .env("HTTPS_PROXY", format!("http://{address}"))
            .env("NO_PROXY", bypass).env("CRABFORGE_TEST_PROXY_ROUTE", "1")
            .kill_on_drop(true).output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::ensure!(
            output.status.success() && stdout.contains("1 passed"),
            "{stdout}"
        );
        Ok::<_, anyhow::Error>(())
    };
    exchange("").await?;
    let proxy = tokio::time::timeout(Duration::from_secs(8), relay).await???;
    exchange("opencode.ai").await?;
    // The listener stayed live for the second request. A queued connection here
    // would indicate an unexpected proxy dial even if the request also went direct.
    assert!(
        tokio::time::timeout(Duration::from_millis(50), proxy.accept())
            .await
            .is_err()
    );
    Ok(())
}
