use anyhow::Result;
use axum::{Router, body::Bytes, extract::OriginalUri, http::HeaderMap, routing::post};
use crabforge_core::{config, service::Service};
use futures_util::stream;
use reqwest::multipart::{Form, Part};
use serde_json::json;
use std::convert::Infallible;
use tokio::net::TcpListener;

#[tokio::test]
async fn media_uploads_rewrite_model_and_preserve_binary_parts() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    const BINARY: &[u8] = b"\0\xff\r\nlocal/m";
    let app = Router::new().fallback(post(
        |OriginalUri(uri): OriginalUri, headers: HeaderMap, body: Bytes| async move {
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
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let dir = tempfile::tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0","providers":[{"id":"local","models":["m"],"endpoints":{"chat":format!("http://{addr}/v1")}}]});
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let service = Service::start(dir.path()).await?;
    let key = config::token(dir.path())?;
    let http = reqwest::Client::builder().no_proxy().build()?;
    for path in ["images/edits", "audio/transcriptions", "audio/translations"] {
        let mut form = Form::new().text("model", "local/m");
        let file = || {
            Part::bytes(BINARY)
                .file_name("sample.bin")
                .mime_str("application/octet-stream")
        };
        if path == "images/edits" {
            form = form.part("image[]", file()?).part("image[]", file()?);
        } else {
            form = form.part("file", file()?);
        }
        let response = http
            .post(format!("http://{}/v1/{path}", service.addr))
            .bearer_auth(&key)
            .multipart(form)
            .send()
            .await?;
        assert_eq!(response.error_for_status()?.text().await?, "received");
    }
    service.stop().await;
    server.abort();
    let _ = server.await;
    Ok(())
}
