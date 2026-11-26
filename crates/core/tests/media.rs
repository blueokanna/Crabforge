use anyhow::Result;
use courierust::courierust_client::{Client, multipart::Multipart};
use crabforge_core::{config, service::Service};
use crabforge_test_support::server::{Incoming, Router, post};
use futures_util::stream;
use nextjson::json;
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
