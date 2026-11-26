use anyhow::{Context, Result};
use axum_core::{body::Body, response::Response};
use bytes::Bytes;
use http::Request;
use std::{
    collections::VecDeque,
    io::{self, Read},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{Semaphore, mpsc, oneshot};
use ureq::{
    Agent, Proxy, RequestExt, SendBody,
    config::AutoHeaderValue,
    tls::{RootCerts, TlsConfig},
};
use url::Url;

/// An agent is used by one exchange at a time so dropping a response can interrupt
/// a blocked socket read without interrupting any other request to the same host.
pub(crate) struct ForwardClient {
    make_agent: Box<dyn Fn() -> Agent + Send + Sync>,
    idle: Arc<Mutex<Vec<Agent>>>,
    bypass: Option<no_proxy::NoProxy>,
    workers: Arc<Semaphore>,
}

impl ForwardClient {
    pub(crate) fn new() -> Self {
        let config = Agent::config_builder()
            .proxy(None)
            .http_status_as_error(false)
            .max_redirects(0)
            .max_idle_connections(1)
            .timeout_resolve(Some(Duration::from_secs(10)))
            .timeout_connect(Some(Duration::from_secs(10)))
            .timeout_per_read(Some(Duration::from_secs(120)))
            .timeout_per_write(Some(Duration::from_secs(120)))
            .user_agent(AutoHeaderValue::None)
            .accept_encoding(AutoHeaderValue::None)
            .tls_config(
                TlsConfig::builder()
                    .root_certs(RootCerts::PlatformVerifier)
                    .build(),
            )
            .build();
        let bypass = super::no_proxy_from_env()
            .into_iter()
            .map(|entry| {
                entry
                    .trim_start_matches("*.")
                    .trim_start_matches('.')
                    .to_ascii_lowercase()
            })
            .collect::<Vec<_>>()
            .join(",");
        Self::with_factory(Some(bypass.into()), move || config.new_agent())
    }

    pub(crate) fn with_factory(
        bypass: Option<no_proxy::NoProxy>,
        make_agent: impl Fn() -> Agent + Send + Sync + 'static,
    ) -> Self {
        Self {
            make_agent: Box::new(make_agent),
            idle: Default::default(),
            bypass,
            workers: Arc::new(Semaphore::new(crate::gateway::MAX_IN_FLIGHT)),
        }
    }

    pub(crate) async fn send(&self, request: Request<Vec<Bytes>>) -> Result<Response> {
        let url = Url::parse(&request.uri().to_string())?;
        let proxy = self
            .bypass
            .as_ref()
            .filter(|rules| !rules.matches(url.host_str().unwrap_or_default()))
            .and_then(|_| super::proxy_url(&url))
            .map(|address| Proxy::new(&address))
            .transpose()?;
        let worker_slot = self.workers.clone().acquire_owned().await?;
        let agent = self
            .idle
            .lock()
            .unwrap()
            .pop()
            .unwrap_or_else(|| (self.make_agent)());
        let lease = Lease {
            agent: Some(agent.clone()),
            idle: self.idle.clone(),
        };
        let (head_tx, head_rx) = oneshot::channel();
        let (body_tx, mut body_rx) = mpsc::channel(1);
        let worker = tokio::task::spawn_blocking(move || -> Result<()> {
            let _slot = worker_slot;
            let request = request.map(|parts| SendBody::from_owned_reader(Segments(parts.into())));
            let response = request.with_agent(&agent).configure().proxy(proxy).run()?;
            let (parts, body) = response.into_parts();
            if head_tx.send(parts).is_err() {
                return Ok(());
            }
            let mut reader = body.into_reader();
            let mut buffer = [0; 16 * 1024];
            loop {
                let count = reader.read(&mut buffer)?;
                if count == 0
                    || body_tx
                        .blocking_send(Bytes::copy_from_slice(&buffer[..count]))
                        .is_err()
                {
                    break;
                }
            }
            Ok(())
        });
        let parts = match head_rx.await {
            Ok(parts) => parts,
            Err(_) => {
                worker.await??;
                anyhow::bail!("upstream closed before sending headers");
            }
        };
        let stream = async_stream::stream! {
            while let Some(bytes) = body_rx.recv().await { yield Ok(bytes); }
            match worker.await.context("upstream reader stopped").and_then(|result| result) {
                Ok(()) => lease.recycle(),
                Err(error) => yield Err(error),
            }
        };
        Ok(Response::from_parts(parts, Body::from_stream(stream)))
    }
}

struct Lease {
    agent: Option<Agent>,
    idle: Arc<Mutex<Vec<Agent>>>,
}

impl Lease {
    fn recycle(mut self) {
        let mut idle = self.idle.lock().unwrap();
        if idle.len() < 2 {
            idle.push(self.agent.take().unwrap());
        }
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        if let Some(agent) = self.agent.take() {
            agent.shutdown();
        }
    }
}

struct Segments(VecDeque<Bytes>);

impl Read for Segments {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        while let Some(first) = self.0.front_mut() {
            if first.is_empty() {
                self.0.pop_front();
                continue;
            }
            let count = first.len().min(out.len());
            out[..count].copy_from_slice(&first.split_to(count));
            return Ok(count);
        }
        Ok(0)
    }
}
