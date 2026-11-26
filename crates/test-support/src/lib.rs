use anyhow::{Result, ensure};
use bytes::Bytes;
use http::{HeaderMap, Method, Request, StatusCode, request::Builder, response::Parts};
use std::{fmt::Display, io::Read, time::Duration};
use tokio::task::spawn_blocking;
use ureq::{Agent, RequestExt};

pub mod server;

/// Async-facing loopback client for exercising a gateway or installed reference binary.
#[derive(Clone, Copy)]
pub struct Client {
    timeout: Duration,
}

impl Default for Client {
    fn default() -> Self {
        Self::with_timeout(Duration::from_secs(30))
    }
}

impl Client {
    pub fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }

    pub fn get(self, url: impl AsRef<str>) -> TestRequest {
        self.request(Method::GET, url.as_ref())
    }

    pub fn post(self, url: impl AsRef<str>) -> TestRequest {
        self.request(Method::POST, url.as_ref())
    }

    fn request(self, method: Method, url: &str) -> TestRequest {
        TestRequest {
            builder: Request::builder().method(method).uri(url),
            body: String::new(),
            timeout: self.timeout,
        }
    }
}

pub struct TestRequest {
    builder: Builder,
    body: String,
    timeout: Duration,
}

impl TestRequest {
    pub fn header(mut self, name: &str, value: impl AsRef<str>) -> Self {
        self.builder = self.builder.header(name, value.as_ref());
        self
    }

    pub fn bearer_auth(self, token: impl Display) -> Self {
        self.header("authorization", format!("Bearer {token}"))
    }

    pub fn body(mut self, value: impl Into<String>) -> Self {
        self.body = value.into();
        self
    }

    pub async fn send(self) -> Result<Response> {
        let agent = Agent::config_builder()
            .proxy(None)
            .timeout_global(Some(self.timeout))
            .max_redirects(0)
            .http_status_as_error(false)
            .build()
            .new_agent();
        let request = self.builder.body(self.body)?;
        spawn_blocking(move || {
            let reply = request.with_agent(&agent).run()?;
            let (parts, body) = reply.into_parts();
            Ok(Response {
                parts,
                reader: Some(body.into_reader()),
                agent,
            })
        })
        .await?
    }
}

pub struct Response {
    parts: Parts,
    reader: Option<ureq::BodyReader<'static>>,
    agent: Agent,
}

impl Response {
    pub fn status(&self) -> StatusCode {
        self.parts.status
    }
    pub fn headers(&self) -> &HeaderMap {
        &self.parts.headers
    }

    pub fn error_for_status(self) -> Result<Self> {
        ensure!(
            !self.status().is_client_error() && !self.status().is_server_error(),
            "HTTP {}",
            self.status()
        );
        Ok(self)
    }

    pub async fn chunk(&mut self) -> Result<Option<Bytes>> {
        let mut reader = self.reader.take().expect("response reader");
        let (reader, data) = spawn_blocking(move || -> Result<_> {
            let mut buffer = vec![0; 16 * 1024];
            let count = reader.read(&mut buffer)?;
            buffer.truncate(count);
            Ok((reader, buffer))
        })
        .await??;
        self.reader = Some(reader);
        Ok((!data.is_empty()).then(|| data.into()))
    }

    pub async fn bytes(mut self) -> Result<Bytes> {
        let mut reader = self.reader.take().expect("response reader");
        spawn_blocking(move || {
            let mut data = Vec::new();
            reader.read_to_end(&mut data)?;
            Ok(data.into())
        })
        .await?
    }

    pub async fn text(self) -> Result<String> {
        Ok(String::from_utf8(self.bytes().await?.to_vec())?)
    }
}

impl Drop for Response {
    fn drop(&mut self) {
        self.agent.clone().shutdown();
    }
}
