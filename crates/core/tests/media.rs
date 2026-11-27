use anyhow::Result;
use courierust::courierust_client::{Client, multipart::Multipart};
use crabforge_core::{config, service::Service};
use crabforge_test_support::server::{Incoming, IntoResponse, Json, Router, post};
use futures_util::stream;
use nextjson::{Value, json};
use std::convert::Infallible;
use tokio::net::TcpListener;

#[tokio::test]
async fn media_uploads_rewrite_model_and_preserve_binary_parts() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    const BINARY: &[u8] = b"\0\xff\r\nlocal/m";
    let app = Router::new().fallback(post(
        |Incoming {
             uri, headers, body, ..
         }| async move {
            let boundary =
                multer::parse_boundary(headers["content-type"].to_str().unwrap()).unwrap();
            let mut multipart =
                multer::Multipart::new(stream::iter([Ok::<_, Infallible>(body)]), boundary);
            let mut model_count = 0;
            let mut files = 0;
            while let Some(field) = multipart.next_field().await.unwrap() {
                match field.name().unwrap() {
                    "model" => {
                        assert_eq!(field.text().await.unwrap(), "m");
                        model_count += 1;
                    }
                    "file" | "image[]" => {
                        assert_eq!(field.file_name(), Some("sample.bin"));
                        assert_eq!(
                            field.content_type().unwrap().as_ref(),
                            "application/octet-stream"
                        );
                        assert_eq!(field.bytes().await.unwrap().as_ref(), BINARY);
                        files += 1;
                    }
                    "mask" => {
                        assert!(field.file_name().is_none());
                        assert_eq!(field.content_type().unwrap().as_ref(), "image/png");
                        assert_eq!(field.bytes().await.unwrap().as_ref(), BINARY);
                    }
                    _ => panic!("unexpected field"),
                }
            }
            assert_eq!(model_count, 1);
            assert_eq!(
                files,
                if uri.path() == "/v1/images/edits" {
                    2
                } else {
                    1
                }
            );
            "received"
        },
    ));
    let server =
        tokio::spawn(async move { crabforge_test_support::server::serve(listener, app).await });
    let dir = tempfile::tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0","providers":[{"id":"local","models":["m"],"endpoints":{"chat":format!("http://{addr}/v1")}}]});
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let key = config::token(dir.path())?;
    for path in ["images/edits", "audio/transcriptions", "audio/translations"] {
        let url = format!("http://{}/v1/{path}", service.addr);
        let auth = format!("Bearer {key}");
        tokio::task::spawn_blocking(move || -> Result<()> {
            use courierust::{
                courierust_body::Body,
                courierust_client::multipart::Part,
                courierust_http::{header::HeaderValue, request::Request},
            };
            let mut form = Multipart::new()?.text("model", "local/m");
            let name = if path == "images/edits" {
                "image[]"
            } else {
                "file"
            };
            form = form.file(name, "sample.bin", None, BINARY);
            if path == "images/edits" {
                form = form
                    .file(name, "sample.bin", None, BINARY)
                    .part(Part::bytes("mask", "image/png", BINARY));
            }
            let mut request = Request::post("/").with_body(Body::from(form.encode()?));
            request.headers.insert(
                "content-type".into(),
                HeaderValue::from_bytes(form.content_type().as_bytes())?,
            );
            request.headers.insert(
                "authorization".into(),
                HeaderValue::from_bytes(auth.as_bytes())?,
            );
            let response = Client::new().execute(&url, request)?;
            assert_eq!(response.status.as_u16(), 200);
            assert_eq!(response.body.as_bytes(), Some(b"received".as_slice()));
            Ok(())
        })
        .await??;
    }
    service.stop().await;
    server.abort();
    let _ = server.await;
    Ok(())
}

#[tokio::test]
async fn speech_routes_native_gemini_audio_and_keeps_openai_relays_native() -> Result<()> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    const MODEL: &str = "gemini-3.1-flash-tts-preview";
    const WAV_MODEL: &str = "gemini-3.8-flash-tts";
    const PCM: &[u8] = b"\x34\x12\x78\x56\x02\x01";
    const WAV: &[u8] = b"RIFF\x34\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0\xc0\x5d\0\0\x80\xbb\0\0\x02\0\x10\0JUNK\x01\0\0\0\0\0data\x06\0\0\0\x34\x12\x78\x56\x02\x01";
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let attempts = Arc::new(AtomicUsize::new(0));
    let app = Router::new().fallback(post({
        let attempts = attempts.clone();
        move |Incoming { uri, headers, body, .. }: Incoming| {
            let attempts = attempts.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                let body: Value = nextjson::from_slice(&body).unwrap();
                match uri.path() {
                    "/bad/v1beta/models/gemini-3.1-flash-tts-preview:generateContent" => {
                        Json(json!({"candidates":[{"content":{"parts":[{
                            "inlineData":{"mimeType":"audio/L16;codec=pcm;rate=24000","data":"AQ=="}
                        }]}}]})).into_response()
                    }
                    "/v1beta/models/gemini-3.1-flash-tts-preview:generateContent" => {
                        assert!(!headers.contains_key("openai-beta"));
                        assert!(body.get("model").is_none());
                        assert_eq!(body.pointer("/contents/0/parts/0/text"), Some(&json!("Speak calmly: Test sentence.")));
                        assert_eq!(body.pointer("/generationConfig/responseModalities"), Some(&json!(["AUDIO"])));
                        assert_eq!(body.pointer("/generationConfig/speechConfig/voiceConfig/prebuiltVoiceConfig/voiceName"), Some(&json!("Zephyr")));
                        ([ ("x-request-id", "speech-fixture") ], Json(json!({
                            "candidates":[{"finishReason":"STOP","content":{"parts":[
                                {"inlineData":{"mimeType":"audio/L16; codec=pcm; rate=24000","data":"NBJ4Vg=="}},
                                {"inline_data":{"mime_type":"audio/l16;channels=1","data":"AgE="}}
                            ]}}]
                        }))).into_response()
                    }
                    "/v1beta/models/gemini-3.8-flash-tts:generateContent" => Json(json!({
                        "candidates":[{"finishReason":"STOP","content":{"parts":[
                            {"inlineData":{"mimeType":"audio/wav","data":STANDARD.encode(WAV)}}
                        ]}}]
                    })).into_response(),
                    "/v1/audio/speech" => {
                        assert_eq!(body["model"], json!(MODEL));
                        assert_eq!(body["response_format"], json!("mp3"));
                        assert_eq!(headers["openai-beta"], "speech-fixture");
                        ([("content-type", "audio/mpeg")], "relay-audio").into_response()
                    }
                    path => panic!("unexpected speech path {path}"),
                }
            }
        }
    }));
    let server = tokio::spawn(crabforge_test_support::server::serve(listener, app));
    let dir = tempfile::tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0","providers":[
        {"id":"gemini","models":[MODEL,WAV_MODEL],"endpoints":{"gemini":format!("http://{addr}/v1beta")}},
        {"id":"bad","models":[MODEL],"endpoints":{"gemini":format!("http://{addr}/bad/v1beta")}},
        {"id":"relay","models":[MODEL],"endpoints":{"chat":format!("http://{addr}/v1"),"gemini":format!("http://{addr}/v1beta")}}
    ],"groups":{"voice":[format!("bad/{MODEL}"),format!("gemini/{MODEL}")]}});
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let client = crabforge_test_support::Client::default();
    let key = config::token(dir.path())?;
    let send = |model: &str, format, stream_format| {
        let body = json!({"model":model,"input":"Test sentence.","instructions":"Speak calmly:","voice":"nova","response_format":format,"stream_format":stream_format,"speed":1.1});
        client
            .post(format!("http://{}/v1/audio/speech", service.addr))
            .bearer_auth(&key)
            .header("openai-beta", "speech-fixture")
            .body(body.to_string())
            .send()
    };
    let wav = send("group/voice", "mp3", "audio")
        .await?
        .error_for_status()?;
    assert_eq!(wav.headers()["content-type"], "audio/wav");
    assert_eq!(wav.headers()["x-request-id"], "speech-fixture");
    let wav = wav.bytes().await?;
    assert_eq!(&wav[..12], b"RIFF\x2a\0\0\0WAVE");
    assert_eq!(&wav[22..24], &1_u16.to_le_bytes());
    assert_eq!(&wav[24..28], &24000_u32.to_le_bytes());
    assert_eq!(&wav[34..36], &16_u16.to_le_bytes());
    assert_eq!(&wav[36..44], b"data\x06\0\0\0");
    assert_eq!(&wav[44..], PCM);
    assert_eq!(attempts.load(Ordering::SeqCst), 2);

    let model = format!("gemini/{MODEL}");
    let pcm = send(&model, "pcm", "audio").await?.error_for_status()?;
    assert_eq!(pcm.headers()["content-type"], "audio/pcm");
    assert_eq!(pcm.bytes().await?.as_ref(), PCM);
    let rejected = send(&model, "wav", "sse").await?;
    assert_eq!(rejected.status(), 400);
    assert!(rejected.text().await?.contains("single audio response"));
    assert_eq!(attempts.load(Ordering::SeqCst), 3);

    let pcm_from_wav = send(&format!("gemini/{WAV_MODEL}"), "pcm", "audio")
        .await?
        .error_for_status()?;
    assert_eq!(pcm_from_wav.headers()["content-type"], "audio/pcm");
    assert_eq!(pcm_from_wav.bytes().await?.as_ref(), PCM);
    let relay = send(&format!("relay/{MODEL}"), "mp3", "audio")
        .await?
        .error_for_status()?;
    assert_eq!(relay.headers()["content-type"], "audio/mpeg");
    assert_eq!(relay.text().await?, "relay-audio");
    service.stop().await;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn dashscope_images_use_native_generation_and_edits_with_group_fallback() -> Result<()> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use crabforge_test_support::server::{StatusCode, get};
    const MODEL: &str = "qwen-image-3.0-pro";
    const IMAGE: &[u8] = b"\x89PNG\r\n\x1a\nfixture";

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let image_url = format!("http://{addr}/image.png?a=signed");
    let url = image_url.clone();
    let (paths, mut calls) = tokio::sync::mpsc::unbounded_channel();
    let app = Router::new()
        .observe(paths)
        .route(
            "/api/v1/services/aigc/multimodal-generation/generation",
            post(move |Incoming { headers, body, .. }: Incoming| {
                let url = url.clone();
                async move {
                    assert_eq!(headers["content-type"], "application/json");
                    assert!(!headers.contains_key("openai-beta"));
                    let value: Value = nextjson::from_slice(&body).unwrap();
                    let content = value.pointer("/input/messages/0/content").and_then(Value::as_array).unwrap();
                    let prompt = content.last().unwrap()["text"].as_str().unwrap();
                    assert_eq!(value["model"], json!(MODEL));
                    let images = match prompt {
                        "draw" => {
                            let parameters = value["parameters"].as_object().unwrap();
                            assert_eq!(parameters.len(), 3);
                            assert_eq!(parameters.get("n"), Some(&json!(2)));
                            assert_eq!(parameters.get("size"), Some(&json!("1024*768")));
                            assert_eq!(parameters.get("watermark"), Some(&json!(false)));
                            assert_eq!(content.len(), 1);
                            vec![json!({"image":url}), json!({"image":"https://dashscope-result-intl.oss-cn-shanghai.aliyuncs.com/b.png"})]
                        }
                        "edit" => {
                            assert_eq!(value["parameters"], json!({"watermark":true}));
                            assert_eq!(content[0]["image"], json!("https://example.com/original.png"));
                            assert_eq!(content[1]["image"], json!(format!("data:image/png;base64,{}", STANDARD.encode(IMAGE))));
                            vec![json!({"image":url})]
                        }
                        "other origin" => vec![json!({"image":"http://192.0.2.1/result.png"})],
                        "redirect" => vec![json!({"image":url.replace("image.png", "redirect.png")})],
                        "group" => vec![json!({"text":"no image generated"})],
                        _ => panic!("unexpected prompt: {prompt}"),
                    };
                    ([ ("x-request-id", "dashscope-fixture") ], Json(json!({"output":{"choices":[{"message":{"content":images}}]}})))
                }
            }),
        )
        .route(
            "/image.png",
            get(|Incoming { uri, headers, .. }: Incoming| async move {
                assert_eq!(uri.query(), Some("a=signed"));
                assert!(!headers.contains_key("authorization"));
                ([ ("content-type", "application/octet-stream") ], IMAGE)
            }),
        )
        .route(
            "/redirect.png",
            get(|_: Incoming| async { (StatusCode::FOUND, [("location", "/image.png?a=signed")]) }),
        )
        .route(
            "/v1/images/generations",
            post(|Incoming { headers, body, .. }: Incoming| async move {
                let value: Value = nextjson::from_slice(&body).unwrap();
                assert_eq!(value["model"], json!(MODEL));
                assert_eq!(headers["openai-beta"], "image-fixture");
                Json(json!({"relay":true}))
            }),
        );
    let server = tokio::spawn(crabforge_test_support::server::serve(listener, app));
    let dir = tempfile::tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0","providers":[
        {"id":"qwen","models":[MODEL],"endpoints":{"chat":format!("http://{addr}/compatible-mode/v1")}},
        {"id":"relay","models":[MODEL],"endpoints":{"chat":format!("http://{addr}/v1")}}
    ],"groups":{"draw":[format!("qwen/{MODEL}"),format!("relay/{MODEL}")]}});
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let client = crabforge_test_support::Client::default();
    let key = config::token(dir.path())?;
    let send = |path: &str, body: Value| {
        client
            .post(format!("http://{}/v1/images/{path}", service.addr))
            .bearer_auth(&key)
            .header("openai-beta", "image-fixture")
            .body(body.to_string())
            .send()
    };
    let model = format!("qwen/{MODEL}");
    let response = send(
        "generations",
        json!({"model":model,"prompt":"draw","size":"1024x768","n":2}),
    )
    .await?
    .error_for_status()?;
    assert_eq!(response.headers()["x-request-id"], "dashscope-fixture");
    let result: Value = nextjson::from_slice(&response.bytes().await?).unwrap();
    assert_eq!(result["data"][0]["url"], json!(image_url));
    assert!(
        result["data"][1]["url"]
            .as_str()
            .unwrap()
            .ends_with("/b.png")
    );
    assert_eq!(
        calls.try_recv()?,
        "/api/v1/services/aigc/multimodal-generation/generation"
    );
    assert!(calls.try_recv().is_err());

    let target = format!("http://{}/v1/images/edits", service.addr);
    let token = key.clone();
    let model_id = model.clone();
    let encoded = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
        use courierust::{
            courierust_body::Body,
            courierust_http::{header::HeaderValue, request::Request},
        };
        let form = Multipart::new()?
            .text("model", &model_id)
            .text("prompt", "edit")
            .text("image[]", "https://example.com/original.png")
            .file("image[]", "picture.png", None, IMAGE)
            .text("watermark", "true")
            .text("response_format", "b64_json");
        let mut request = Request::post("/").with_body(Body::from(form.encode()?));
        request.headers.insert(
            "content-type".into(),
            HeaderValue::from_bytes(form.content_type().as_bytes())?,
        );
        request.headers.insert(
            "authorization".into(),
            HeaderValue::from_bytes(format!("Bearer {token}").as_bytes())?,
        );
        let response = Client::new().execute(&target, request)?;
        assert_eq!(response.status.as_u16(), 200);
        Ok(response.body.as_bytes().unwrap().to_vec())
    })
    .await??;
    let result: Value = nextjson::from_slice(&encoded).unwrap();
    assert_eq!(result["data"][0]["b64_json"], json!(STANDARD.encode(IMAGE)));
    assert_eq!(
        calls.try_recv()?,
        "/api/v1/services/aigc/multimodal-generation/generation"
    );
    assert_eq!(calls.try_recv()?, "/image.png");

    let rejected = send("edits", json!({"model":model,"prompt":"edit","image":"https://example.com/original.png","mask":"https://example.com/mask.png"})).await?;
    assert_eq!(rejected.status(), 400);
    assert!(rejected.text().await?.contains("no mask"));
    assert!(calls.try_recv().is_err());
    for (prompt, detail) in [
        ("other origin", "public Alibaba Cloud OSS"),
        ("redirect", "HTTP 302"),
    ] {
        let rejected = send(
            "generations",
            json!({"model":model,"prompt":prompt,"response_format":"b64_json"}),
        )
        .await?;
        assert_eq!(rejected.status(), 502);
        assert!(rejected.text().await?.contains(detail));
        assert_eq!(
            calls.try_recv()?,
            "/api/v1/services/aigc/multimodal-generation/generation"
        );
        if prompt == "redirect" {
            assert_eq!(calls.try_recv()?, "/redirect.png");
        }
        assert!(calls.try_recv().is_err());
    }
    let fallback = send(
        "generations",
        json!({"model":"group/draw","prompt":"group"}),
    )
    .await?
    .error_for_status()?;
    let body: Value = nextjson::from_slice(&fallback.bytes().await?).unwrap();
    assert_eq!(body, json!({"relay":true}));
    assert_eq!(
        calls.try_recv()?,
        "/api/v1/services/aigc/multimodal-generation/generation"
    );
    assert_eq!(calls.try_recv()?, "/v1/images/generations");
    service.stop().await;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn dashscope_audio_synthesizes_and_transcribes_through_native_endpoints() -> Result<()> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use crabforge_test_support::server::get;
    const TTS: &str = "qwen-audio-3.0-tts-plus";
    const ASR: &str = "qwen-audio-3.0-asr-flash";
    const MP3: &[u8] = b"ID3\x04\0\0\0\0\0\0";
    const PCM: &[u8] = b"\x34\x12\x78\x56";

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let (paths, mut calls) = tokio::sync::mpsc::unbounded_channel();
    let app = Router::new()
        .observe(paths)
        .route(
            "/api/v1/services/audio/tts/SpeechSynthesizer",
            post(move |Incoming { body, headers, .. }: Incoming| async move {
                assert_eq!(headers["content-type"], "application/json");
                let value: Value = nextjson::from_slice(&body).unwrap();
                assert_eq!(value["model"], json!(TTS));
                let input = &value["input"];
                assert_eq!(input["text"], json!("示例语音"));
                let audio = match input["format"].as_str().unwrap() {
                    "mp3" => {
                        assert_eq!(input["voice"], json!("longanlingxin"));
                        assert_eq!(input["instruction"], json!("轻声"));
                        assert_eq!(input["rate"].as_f64(), Some(2.0));
                        json!({"url":format!("http://{addr}/sample.mp3?signature=fixture"),"data":""})
                    }
                    "pcm" => {
                        assert_eq!(input["voice"], json!("longanlufeng"));
                        json!({"data":STANDARD.encode(PCM)})
                    }
                    "wav" => json!({"data":"invalid!"}),
                    other => panic!("unexpected format {other}"),
                };
                ([("request-id", "dashscope-audio")], Json(json!({"output":{"audio":audio}})))
            }),
        )
        .route("/sample.mp3", get(|Incoming { headers, uri, .. }: Incoming| async move {
            assert!(!headers.contains_key("authorization"));
            assert_eq!(uri.query(), Some("signature=fixture"));
            MP3
        }))
        .route("/v1/audio/speech", post(|Incoming { body, .. }: Incoming| async move {
            let value: Value = nextjson::from_slice(&body).unwrap();
            assert_eq!(value["model"], json!(TTS));
            ([("content-type", "audio/wav")], "relay fallback")
        }))
        .route(
            "/api/v1/services/aigc/multimodal-generation/generation",
            post(|Incoming { body, headers, .. }: Incoming| async move {
                assert_eq!(headers["content-type"], "application/json");
                let value: Value = nextjson::from_slice(&body).unwrap();
                assert_eq!(value["model"], json!(ASR));
                assert_eq!(value.pointer("/parameters/format"), Some(&json!("mp3")));
                assert_eq!(value.pointer("/parameters/language_hints"), Some(&json!(["zh"])));
                assert_eq!(value.pointer("/input/messages/0/content/0/input_audio/data"),
                    Some(&json!(format!("data:audio/mpeg;base64,{}", STANDARD.encode(MP3)))));
                Json(json!({"output":{"choices":[{"message":{"content":[{"text":"识别"},{"text":"成功"}]}}]},"usage":{"duration":1.25}}))
            }),
        );
    let server = tokio::spawn(crabforge_test_support::server::serve(listener, app));
    let dir = tempfile::tempdir()?;
    let config = json!({"listen":"127.0.0.1:0","providers":[
        {"id":"qwen","models":[TTS,ASR],"endpoints":{"chat":format!("http://{addr}/compatible-mode/v1")}},
        {"id":"relay","models":[TTS],"endpoints":{"chat":format!("http://{addr}/v1")}}
    ],"groups":{"speech":[format!("qwen/{TTS}"),format!("relay/{TTS}")]}});
    config::save(
        dir.path(),
        &config::snapshot(dir.path())?,
        &config.to_string(),
    )?;
    let service = Service::start(dir.path()).await?;
    let key = config::token(dir.path())?;
    let root = format!("http://{}/v1/audio", service.addr);
    let client = crabforge_test_support::Client::default();
    let send = |model: &str, format: &str, voice: &str, stream: &str| {
        client.post(format!("{root}/speech")).bearer_auth(&key).body(json!({
            "model":model,"input":"示例语音","voice":voice,"instructions":"轻声","response_format":format,"speed":3,"stream_format":stream
        }).to_string()).send()
    };
    let model = format!("qwen/{TTS}");
    let mp3 = send(&model, "", "alloy", "audio")
        .await?
        .error_for_status()?;
    assert_eq!(mp3.headers()["content-type"], "audio/mpeg");
    assert_eq!(mp3.headers()["request-id"], "dashscope-audio");
    assert_eq!(mp3.bytes().await?.as_ref(), MP3);
    assert_eq!(
        calls.try_recv()?,
        "/api/v1/services/audio/tts/SpeechSynthesizer"
    );
    assert_eq!(calls.try_recv()?, "/sample.mp3");
    let pcm = send(&model, "pcm", "longanlufeng", "")
        .await?
        .error_for_status()?;
    assert_eq!(pcm.headers()["content-type"], "audio/pcm");
    assert_eq!(pcm.bytes().await?.as_ref(), PCM);
    assert_eq!(
        calls.try_recv()?,
        "/api/v1/services/audio/tts/SpeechSynthesizer"
    );
    let fallback = send("group/speech", "wav", "", "")
        .await?
        .error_for_status()?;
    assert_eq!(fallback.text().await?, "relay fallback");
    assert_eq!(
        calls.try_recv()?,
        "/api/v1/services/audio/tts/SpeechSynthesizer"
    );
    assert_eq!(calls.try_recv()?, "/v1/audio/speech");
    for (format, stream) in [("flac", ""), ("mp3", "sse")] {
        assert_eq!(send(&model, format, "", stream).await?.status(), 400);
    }
    for (path, format) in [
        ("transcriptions", "json"),
        ("transcriptions", "text"),
        ("transcriptions", "verbose_json"),
        ("transcriptions", "srt"),
        ("translations", "json"),
    ] {
        let url = format!("{root}/{path}");
        let auth = format!("Bearer {key}");
        tokio::task::spawn_blocking(move || -> Result<()> {
            use courierust::{
                courierust_body::Body,
                courierust_http::{header::HeaderValue, request::Request},
            };
            let form = Multipart::new()?
                .text("model", format!("qwen/{ASR}"))
                .text("language", "zh")
                .text("response_format", format)
                .text("prompt", "术语提示")
                .file("file", "speech.MP3", None, MP3);
            let mut request = Request::post("/").with_body(Body::from(form.encode()?));
            request.headers.insert(
                "content-type".into(),
                HeaderValue::from_bytes(form.content_type().as_bytes())?,
            );
            request.headers.insert(
                "authorization".into(),
                HeaderValue::from_bytes(auth.as_bytes())?,
            );
            let response = Client::new().execute(&url, request)?;
            let rejected = path == "translations" || format == "srt";
            assert_eq!(response.status.as_u16(), if rejected { 400 } else { 200 });
            if !rejected {
                let body = response.body.as_bytes().unwrap();
                match format {
                    "text" => assert_eq!(body, "识别成功".as_bytes()),
                    "verbose_json" => {
                        let value: Value = nextjson::from_slice(body).unwrap();
                        assert_eq!(value["text"], json!("识别成功"));
                        assert_eq!(value["task"], json!("transcribe"));
                        assert_eq!(value["language"], json!("zh"));
                        assert_eq!(value["duration"], json!(1.25));
                    }
                    _ => assert_eq!(
                        nextjson::from_slice::<Value>(body).unwrap(),
                        json!({"text":"识别成功"})
                    ),
                }
            }
            Ok(())
        })
        .await??;
        if path == "transcriptions" && format != "srt" {
            assert_eq!(
                calls.try_recv()?,
                "/api/v1/services/aigc/multimodal-generation/generation"
            );
        }
    }
    assert!(calls.try_recv().is_err());
    service.stop().await;
    server.abort();
    Ok(())
}
