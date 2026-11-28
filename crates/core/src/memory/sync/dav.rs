use crate::config::MAX_BACKUP;
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use http::{HeaderValue, Response, StatusCode, header};
use std::time::Duration;
use ureq::{
    Body, RequestBuilder,
    tls::{RootCerts, TlsConfig},
    typestate,
};
use url::{Host, Url};
use zeroize::Zeroizing;

pub(super) struct Dav {
    pub(super) url: Url,
    auth: Option<Zeroizing<String>>,
    client: ureq::Agent,
}

pub(super) struct Document {
    pub(super) etag: Option<String>,
    pub(super) content: Vec<u8>,
}

impl Dav {
    pub(super) fn new(url: &str, user: &str, password: &str) -> Result<Self> {
        let mut url = Url::parse(url.trim()).context("invalid WebDAV URL")?;
        let loopback = match url.host() {
            Some(Host::Ipv4(ip)) => ip.is_loopback(),
            Some(Host::Ipv6(ip)) => ip.is_loopback(),
            _ => false,
        };
        ensure!(
            url.has_host() && (url.scheme() == "https" || (url.scheme() == "http" && loopback)),
            "WebDAV uses HTTPS or a numeric loopback HTTP address"
        );
        ensure!(
            url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none(),
            "enter WebDAV credentials separately and use a URL without a query or fragment"
        );
        if !url.path().ends_with(".crabforge-backup") && !url.path().ends_with(".magpie-backup") {
            url.path_segments_mut()
                .map_err(|_| anyhow::anyhow!("invalid WebDAV collection URL"))?
                .pop_if_empty()
                .push("crabforge.crabforge-backup");
        }
        ensure!(
            !user.contains(':')
                && !user.chars().any(char::is_control)
                && (password.is_empty() || !user.is_empty()),
            "invalid WebDAV user name"
        );
        let auth = (!user.is_empty()).then(|| {
            let credentials = Zeroizing::new(format!("{user}:{password}"));
            Zeroizing::new(format!("Basic {}", STANDARD.encode(credentials.as_bytes())))
        });
        let mut client = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_global(Some(Duration::from_secs(45)))
            .tls_config(
                TlsConfig::builder()
                    .root_certs(RootCerts::PlatformVerifier)
                    .build(),
            )
            .user_agent("crabforge");
        if loopback {
            client = client.proxy(None);
        }
        let client = client.build().new_agent();
        Ok(Self { url, auth, client })
    }

    fn authenticate<T>(&self, request: RequestBuilder<T>) -> RequestBuilder<T> {
        if let Some(auth) = &self.auth {
            let mut value = HeaderValue::from_str(auth).expect("Basic credentials are ASCII");
            value.set_sensitive(true);
            request.header(header::AUTHORIZATION, value)
        } else {
            request
        }
    }

    pub(super) fn read(&self) -> Result<Option<Document>> {
        let response = self
            .authenticate(self.client.get(self.url.as_str()))
            .header(header::ACCEPT_ENCODING, "identity")
            .header(header::CACHE_CONTROL, "no-cache")
            .call()
            .context("could not read the WebDAV backup")?;
        match response.status() {
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::OK => Self::document(response).map(Some),
            status => anyhow::bail!(
                "reading the WebDAV backup returned HTTP {}",
                status.as_u16()
            ),
        }
    }

    fn document(mut response: Response<Body>) -> Result<Document> {
        let mut etags = response.headers().get_all(header::ETAG).iter();
        let etag = etags
            .next()
            .filter(|_| etags.next().is_none())
            .and_then(|value| value.to_str().ok())
            .filter(|tag| {
                tag.strip_prefix('"')
                    .and_then(|tag| tag.strip_suffix('"'))
                    .is_some_and(|tag| {
                        tag.bytes()
                            .all(|byte| byte == b'!' || (b'#'..=b'~').contains(&byte))
                    })
            })
            .map(str::to_owned);
        let content = response
            .body_mut()
            .with_config()
            .limit(MAX_BACKUP as u64)
            .read_to_vec()
            .context("could not download the WebDAV backup within its size limit")?;
        Ok(Document { etag, content })
    }

    pub(super) fn write(&self, previous: Option<&Document>, bytes: &[u8]) -> Result<()> {
        let request: RequestBuilder<typestate::WithBody> = self
            .authenticate(self.client.put(self.url.as_str()))
            .header(header::CONTENT_TYPE, "application/octet-stream");
        let request = match previous {
            Some(previous) => request.header(
                header::IF_MATCH,
                previous
                    .etag
                    .as_deref()
                    .context("the WebDAV backup requires a strong ETag before it can be updated")?,
            ),
            None => request.header(header::IF_NONE_MATCH, "*"),
        };
        let status = request
            .send(bytes)
            .context("could not write the WebDAV backup")?
            .status();
        ensure!(
            status != StatusCode::PRECONDITION_FAILED,
            "the WebDAV backup changed; preview the sync again"
        );
        ensure!(
            status.is_success(),
            "writing the WebDAV backup returned HTTP {}",
            status.as_u16()
        );
        let stored = self
            .read()?
            .context("the WebDAV server did not return the backup after writing it")?;
        ensure!(
            stored.content == bytes,
            "the WebDAV server returned different bytes after writing; preview the sync again"
        );
        Ok(())
    }
}
