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
        transport::{DefaultConnector, NextTimeout},
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
        assert_eq!(uri.host(), Some("opencode.ai"));
        let mut addresses = self.empty();
        addresses.push(self.0);
        Ok(addresses)
    }
}

#[tokio::test]
async fn console_go_forwards_untranslatable_input_in_its_original_protocol() -> Result<()> {
    const BODY: &str = r#"{ "model": "go/minimax-m2.7", "messages": [{"role":"user","content":[{"type":"image_url","image_url":{"url":"data:image/png;base64,aGk="}}]}] }"#;
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
    let client = ForwardClient::with_factory(None, move || {
        Agent::with_parts(
            config.clone(),
            DefaultConnector::default(),
            FixtureAddress(address),
        )
    });
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
