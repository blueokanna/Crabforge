use anyhow::Result;
use courierust::courierust_client::{Client, ClientConfig, TlsSettings, no_proxy_from_env};
use std::{env, time::Duration};
use url::{Host, Url};

mod forward;
pub(crate) use forward::ForwardClient;

/// Bounded, non-redirecting requests for upstream directories and token exchanges.
/// Call from a blocking worker since loading platform trust roots may perform I/O.
pub(crate) fn client(url: &Url, max_body: usize, timeout: Duration) -> Result<Client> {
    let https = url.scheme() == "https";
    let tls = https
        .then(TlsSettings::with_system_roots)
        .transpose()
        .map_err(anyhow::Error::msg)?;
    Ok(Client::with_config(ClientConfig {
        tls,
        http2: https,
        proxy: proxy_url(url),
        no_proxy: no_proxy_from_env(),
        max_redirects: 0,
        max_body,
        accept_encoding: false,
        connect_timeout: Some(timeout),
        read_timeout: Some(timeout),
        ..Default::default()
    }))
}

fn local(url: &Url) -> bool {
    match url.host() {
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        None => false,
    }
}

fn proxy_url(url: &Url) -> Option<String> {
    let names: &[&str] = if local(url) {
        &[]
    } else if url.scheme() == "https" {
        &["https_proxy", "HTTPS_PROXY", "all_proxy", "ALL_PROXY"]
    } else if env::var_os("REQUEST_METHOD").is_some() {
        &["all_proxy", "ALL_PROXY"]
    } else {
        &["http_proxy", "HTTP_PROXY", "all_proxy", "ALL_PROXY"]
    };
    names
        .iter()
        .find_map(|name| env::var(name).ok().filter(|v| !v.is_empty()))
}
