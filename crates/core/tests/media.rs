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
