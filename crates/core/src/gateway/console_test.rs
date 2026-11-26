use super::*;
use http_body_util::BodyExt;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use reqwest::Client;
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
        uri: &axum::http::Uri,
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
        http1::Builder::new()
            .keep_alive(false)
            .serve_connection(
                TokioIo::new(tls),
                service_fn(
                    |request: hyper::Request<hyper::body::Incoming>| async move {
                        let (parts, body) = request.into_parts();
                        assert_eq!(parts.uri.path(), "/inference/go/openai/v1/chat/completions");
                        assert_eq!(parts.headers["x-opencode-session"], "test-turn");
                        assert_eq!(
                            parts.headers[header::USER_AGENT],
                            crate::opencode::USER_AGENT_VALUE
                        );
                        assert!(!parts.headers.contains_key(header::AUTHORIZATION));
                        assert_eq!(
                            body.collect().await?.to_bytes(),
                            BODY.replace("go/minimax-m2.7", "minimax-m2.7")
                        );
                        Ok::<_, hyper::Error>(hyper::Response::new(Body::from(REPLY)))
                    },
                ),
            )
            .await?;
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
    let reply = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(8))
        .build()?
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
