use super::*;
use crabforge_test_support::{
    Client,
    server::{Incoming, Router, post, serve_io},
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
struct FixtureAddress(std::net::SocketAddr);

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
            Some("opencode.ai") => self.0,
            host => panic!("unexpected fixture host {host:?}"),
        });
        Ok(addresses)
    }
}

#[tokio::test]
async fn console_go_forwards_untranslatable_input_in_its_original_protocol() -> Result<()> {
    const BODY: &str = r#"{ "model": "go/minimax-m2.7", "messages": [{"role":"user","content":[{"type":"image_url","image_url":{"url":"data:image/png;base64,aGk=","detail":"high"}}]}] }"#;
    const REPLY: &str = r#"{"choices":[{"message":{"content":"received"}}]}"#;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let key = rcgen::KeyPair::generate()?;
    let certificate =
        rcgen::CertificateParams::new(vec!["opencode.ai".to_string()])?.self_signed(&key)?;
    let cert = certificate.der();
    let tls = TlsConfig::builder()
        .root_certs(RootCerts::Specific(Arc::new(vec![
            Certificate::from_der(cert).to_owned(),
        ])))
        .build();
    let config = Agent::config_builder()
        .proxy(None)
        .tls_config(tls)
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    let bypass =
        std::env::var_os("CRABFORGE_TEST_PROXY_ROUTE").map(|_| ForwardClient::environment_bypass());
    let client = ForwardClient::with_resolver(bypass, config, move || FixtureAddress(address));
    let server = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_no_client_auth()
    .with_single_cert(
        vec![cert.clone()],
        rustls_pki_types::PrivatePkcs8KeyDer::from(key.serialize_der()).into(),
    )?;
    let upstream = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await?;
        let tls = TlsAcceptor::from(Arc::new(server)).accept(tcp).await?;
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
