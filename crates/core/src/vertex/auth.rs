use super::{Vertex, account};
use anyhow::{Context, Result, bail, ensure};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use courierust::{
    courierust_client::{Client, ClientConfig, TlsSettings, no_proxy_from_env},
    courierust_http::Method,
};
use nextjson::{NsonDeserialize, Value, json};
use ring::{
    rand::SystemRandom,
    signature::{RSA_PKCS1_SHA256, RsaKeyPair},
};
use rustls_pki_types::{PrivateKeyDer, pem::PemObject};
use sha2::{Digest, Sha256};
use std::{
    env,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::Mutex;
use url::{Url, form_urlencoded};

const SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";
#[cfg(test)]
mod tests;
const OAUTH: &str = "https://oauth2.googleapis.com/token";
const IAM: &str = "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/";
const MAX_REPLY: usize = 64 * 1024;
const EXCHANGE_TIMEOUT: Duration = Duration::from_secs(30);

fn absolute(path: &Path) -> Result<PathBuf> {
    if let Ok(rest) = path.strip_prefix("~") {
        return Ok(env::home_dir()
            .context("cannot locate home for ADC")?
            .join(rest));
    }
    ensure!(
        path.is_absolute(),
        "Google credential path must be absolute"
    );
    Ok(path.to_owned())
}

fn credential_file(vertex: &Vertex) -> Result<PathBuf> {
    if !vertex.credentials.is_empty() {
        return absolute(Path::new(&vertex.credentials));
    }
    let var = |name| {
        env::var_os(name)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
    };
    if let Some(path) = var("GOOGLE_APPLICATION_CREDENTIALS") {
        return absolute(&path);
    }
    let dir = if let Some(dir) = var("CLOUDSDK_CONFIG") {
        absolute(&dir)?
    } else if cfg!(windows) {
        absolute(&var("APPDATA").context("cannot locate gcloud configuration")?)?.join("gcloud")
    } else {
        env::home_dir()
            .context("cannot locate gcloud configuration")?
            .join(".config/gcloud")
    };
    Ok(dir.join("application_default_credentials.json"))
}

struct Token {
    value: String,
    expiry: i64,
}

struct Cached {
    fingerprint: [u8; 32],
    result: Result<Token, (Instant, String)>,
}

#[derive(Default)]
pub(crate) struct TokenCache(Mutex<Option<Cached>>);

impl TokenCache {
    pub(crate) async fn token(&self, vertex: &Vertex) -> Result<String> {
        self.exchange(vertex, OAUTH, IAM).await
    }

    async fn exchange(&self, vertex: &Vertex, oauth: &str, iam: &str) -> Result<String> {
        let file = credential_file(vertex)?;
        let mut cached = self.0.lock().await;
        let bytes = tokio::task::spawn_blocking(move || crate::config::read_limited(&file, 1024*1024)).await?.context("cannot read Google credentials; check the ADC path or sign in with gcloud auth application-default login")?;
        let fingerprint: [u8; 32] = Sha256::digest(&bytes).into();
        if let Some(entry) = cached.as_ref().filter(|c| c.fingerprint == fingerprint) {
            match &entry.result {
                Ok(t)
                    if t.expiry.saturating_sub(300)
                        > OffsetDateTime::now_utc().unix_timestamp() =>
                {
                    return Ok(t.value.clone());
                }
                Err((at, msg)) if at.elapsed() < Duration::from_secs(1) => bail!("{msg}"),
                _ => (),
            }
        }
        let mint = async {
            let credentials: Credentials = nextjson::from_slice(&bytes)
                .map_err(anyhow::Error::msg)
                .context("invalid Google credentials JSON")?;
            credentials.check_universe()?;
            let mut token = if credentials.kind == "impersonated_service_account" {
                let url = Url::parse(&credentials.service_account_impersonation_url)?;
                let (_, name) = url
                    .path()
                    .rsplit_once("/serviceAccounts/")
                    .context("invalid impersonation URL")?;
                let name = percent_encoding::percent_decode_str(
                    name.strip_suffix(":generateAccessToken")
                        .context("invalid impersonation URL")?,
                )
                .decode_utf8()?;
                let source = credentials
                    .source_credentials
                    .as_deref()
                    .context("missing impersonation source")?;
                let source = source.grant(oauth).await?;
                impersonate(iam, source, &name, &credentials.delegates).await?
            } else {
                credentials.grant(oauth).await?
            };
            if !vertex.impersonate.is_empty() {
                token = impersonate(iam, token, &vertex.impersonate, &[]).await?;
            }
            Ok::<_, anyhow::Error>(token)
        };
        let result = tokio::time::timeout(EXCHANGE_TIMEOUT, mint)
            .await
            .context("Google token exchange timed out")
            .and_then(|r| r);
        let value = result
            .as_ref()
            .map(|t| t.value.clone())
            .map_err(|e| anyhow::anyhow!("{e:#}"));
        *cached = Some(Cached {
            fingerprint,
            result: result.map_err(|e| (Instant::now(), format!("{e:#}"))),
        });
        value
    }
}

#[derive(Default, NsonDeserialize)]
#[njson(default)]
struct Credentials {
    #[njson(rename = "type")]
    kind: String,
    universe_domain: String,
    client_id: String,
    client_secret: String,
    refresh_token: String,
    client_email: String,
    private_key: String,
    private_key_id: String,
    service_account_impersonation_url: String,
    delegates: Vec<String>,
    source_credentials: Option<Box<Credentials>>,
}

impl Credentials {
    fn check_universe(&self) -> Result<()> {
        ensure!(
            matches!(self.universe_domain.as_str(), "" | "googleapis.com"),
            "unsupported Google credential universe"
        );
        Ok(())
    }

    fn assertion(&self, audience: &str) -> Result<String> {
        ensure!(account(&self.client_email), "invalid service account email");
        let der = PrivateKeyDer::from_pem_slice(self.private_key.as_bytes())
            .context("invalid service account PEM")?;
        let key = match der {
            PrivateKeyDer::Pkcs8(key) => RsaKeyPair::from_pkcs8(key.secret_pkcs8_der()),
            PrivateKeyDer::Pkcs1(key) => RsaKeyPair::from_der(key.secret_pkcs1_der()),
            _ => bail!("service account requires an RSA key"),
        }
        .map_err(|_| anyhow::anyhow!("invalid service account RSA key"))?;
        let head = if self.private_key_id.is_empty() {
            json!({"alg":"RS256","typ":"JWT"})
        } else {
            json!({"alg":"RS256","typ":"JWT","kid":self.private_key_id})
        };
        let now = OffsetDateTime::now_utc().unix_timestamp() - 10;
        let claims =
            json!({"iss":self.client_email,"scope":SCOPE,"aud":audience,"iat":now,"exp":now+3600});
        let encode = |v: &Value| -> Result<String> {
            Ok(URL_SAFE_NO_PAD.encode(nextjson::to_vec(v).map_err(anyhow::Error::msg)?))
        };
        let unsigned = format!("{}.{}", encode(&head)?, encode(&claims)?);
        let mut signature = vec![0; key.public().modulus_len()];
        key.sign(
            &RSA_PKCS1_SHA256,
            &SystemRandom::new(),
            unsigned.as_bytes(),
            &mut signature,
        )
        .map_err(|_| anyhow::anyhow!("could not sign Google JWT"))?;
        Ok(format!("{unsigned}.{}", URL_SAFE_NO_PAD.encode(signature)))
    }

    fn grant_form(&self, oauth: &str) -> Result<Vec<u8>> {
        self.check_universe()?;
        let mut form = form_urlencoded::Serializer::new(String::new());
        match self.kind.as_str() {
            "authorized_user" => {
                ensure!(
                    !self.client_id.is_empty() && !self.refresh_token.is_empty(),
                    "Google ADC lacks client ID or refresh token"
                );
                form.append_pair("grant_type", "refresh_token")
                    .append_pair("refresh_token", &self.refresh_token)
                    .append_pair("client_id", &self.client_id)
                    .append_pair("client_secret", &self.client_secret);
            }
            "service_account" => {
                form.append_pair("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer")
                    .append_pair("assertion", &self.assertion(oauth)?);
            }
            _ => bail!("expected authorized_user or service_account Google credentials"),
        }
        Ok(form.finish().into_bytes())
    }

    async fn grant(&self, oauth: &str) -> Result<Token> {
        #[derive(NsonDeserialize)]
        struct Grant {
            access_token: String,
            expires_in: Option<i64>,
        }
        let grant: Grant = reply(
            Url::parse(oauth)?,
            "application/x-www-form-urlencoded",
            self.grant_form(oauth)?,
            None,
        )
        .await?;
        let ttl = grant.expires_in.unwrap_or(3600);
        ensure!(
            !grant.access_token.is_empty() && ttl > 0,
            "Google returned an empty or expired token"
        );
        Ok(Token {
            value: grant.access_token,
            expiry: OffsetDateTime::now_utc()
                .unix_timestamp()
                .saturating_add(ttl),
        })
    }
}

async fn impersonate(iam: &str, source: Token, name: &str, delegates: &[String]) -> Result<Token> {
    ensure!(account(name), "invalid Google impersonation account");
    let mut url = Url::parse(iam)?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("invalid IAM endpoint"))?
        .pop_if_empty()
        .push(&format!("{name}:generateAccessToken"));
    let body = if delegates.is_empty() {
        json!({"scope":[SCOPE],"lifetime":"3600s"})
    } else {
        let delegates: Vec<_> = delegates
            .iter()
            .map(|d| {
                if d.starts_with("projects/") {
                    d.clone()
                } else {
                    format!("projects/-/serviceAccounts/{d}")
                }
            })
            .collect();
        json!({"scope":[SCOPE],"lifetime":"3600s","delegates":delegates})
    };
    #[derive(NsonDeserialize)]
    #[njson(rename_all = "camelCase")]
    struct Grant {
        access_token: String,
        expire_time: String,
    }
    let grant: Grant = reply(
        url,
        "application/json",
        nextjson::to_vec(&body).map_err(anyhow::Error::msg)?,
        Some(source.value),
    )
    .await?;
    let expiry = OffsetDateTime::parse(&grant.expire_time, &Rfc3339)?.unix_timestamp();
    ensure!(
        !grant.access_token.is_empty() && expiry > OffsetDateTime::now_utc().unix_timestamp(),
        "IAM returned an empty or expired token"
    );
    Ok(Token {
        value: grant.access_token,
        expiry,
    })
}

async fn reply<T: for<'de> NsonDeserialize<'de>>(
    url: Url,
    content_type: &'static str,
    body: Vec<u8>,
    bearer: Option<String>,
) -> Result<T> {
    let bytes = tokio::task::spawn_blocking(move || -> Result<_> {
        let https = url.scheme() == "https";
        let tls = https
            .then(TlsSettings::with_system_roots)
            .transpose()
            .map_err(anyhow::Error::msg)?;
        let proxy = if https {
            ["HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy"]
                .iter()
                .find_map(|name| env::var(name).ok().filter(|v| !v.is_empty()))
        } else {
            None
        };
        let client = Client::with_config(ClientConfig {
            tls,
            http2: https,
            proxy,
            no_proxy: no_proxy_from_env(),
            max_redirects: 0,
            max_body: MAX_REPLY,
            accept_encoding: false,
            connect_timeout: Some(EXCHANGE_TIMEOUT),
            read_timeout: Some(EXCHANGE_TIMEOUT),
            ..Default::default()
        });
        let mut request = client
            .request(url.as_str(), Method::POST)
            .header("content-type", content_type)
            .body(body);
        if let Some(token) = bearer {
            request = request.bearer_auth(&token);
        }
        let response = request.send().context("Google token request failed")?;
        ensure!(
            response.status.is_success(),
            "Google token endpoint returned HTTP {}",
            response.status.as_u16()
        );
        Ok(response.body.collect_limited(MAX_REPLY)?)
    })
    .await??;
    nextjson::from_slice(&bytes)
        .map_err(anyhow::Error::msg)
        .context("invalid Google token response")
}
