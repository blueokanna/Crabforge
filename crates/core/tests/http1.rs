use anyhow::Result;
use crabforge_core::{config, service::Service};
use std::time::Duration;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    time::timeout,
};

#[tokio::test]
async fn continue_chunked_upload_and_next_request_share_one_connection() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::write(
        dir.path().join("settings.json"),
        r#"{"listen":"127.0.0.1:0"}"#,
    )?;
    let key = config::token(dir.path())?;
    let gateway = Service::start(dir.path()).await?;
    timeout(Duration::from_secs(5), async {
        let mut socket = BufReader::new(TcpStream::connect(gateway.addr).await?);
        let headers = format!("POST /v1/chat/completions HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {key}\r\nExpect: 100-continue\r\nTransfer-Encoding: chunked\r\n\r\n");
        socket.get_mut().write_all(headers.as_bytes()).await?;
        let mut line = String::new();
        socket.read_line(&mut line).await?;
        assert_eq!(line, "HTTP/1.1 100 Continue\r\n");
        line.clear();
        socket.read_line(&mut line).await?;
        assert_eq!(line, "\r\n");
        let json = r#"{"model":"unknown"}"#;
        let next = format!("{:x}\r\n{json}\r\n0\r\nX-Note: done\r\n\r\nGET /v1/models HTTP/1.0\r\nAuthorization: Bearer {key}\r\n\r\n", json.len());
        socket.get_mut().write_all(next.as_bytes()).await?;
        let mut replies = String::new();
        socket.read_to_string(&mut replies).await?;
        assert!(replies.starts_with("HTTP/1.1 404"), "{replies}");
        let (upload, catalog) = replies.split_once("HTTP/1.0 200").expect("HTTP/1.0 catalog reply");
        assert!(upload.contains("unknown model"));
        assert!(catalog.ends_with(r#"{"object":"list","data":[]}"#));
        Ok::<_, anyhow::Error>(())
    }).await??;
    gateway.stop().await;
    Ok(())
}

#[tokio::test]
async fn pending_upload_checks_credentials_and_size_before_continue() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::write(
        dir.path().join("settings.json"),
        r#"{"listen":"127.0.0.1:0"}"#,
    )?;
    let key = config::token(dir.path())?;
    let gateway = Service::start(dir.path()).await?;
    for (credential, status) in [("other", 401), (key.as_str(), 413)] {
        let mut socket = TcpStream::connect(gateway.addr).await?;
        let headers = format!(
            "POST /v1/responses HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {credential}\r\nContent-Length: 8388609\r\nExpect: 100-continue\r\n\r\n"
        );
        socket.write_all(headers.as_bytes()).await?;
        let mut reply = String::new();
        timeout(Duration::from_secs(3), socket.read_to_string(&mut reply)).await??;
        assert!(reply.starts_with(&format!("HTTP/1.1 {status} ")), "{reply}");
        assert!(reply.contains("connection: close"));
    }
    gateway.stop().await;
    Ok(())
}
