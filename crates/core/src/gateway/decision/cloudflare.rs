use crate::{config, upstream::ForwardClient};
use anyhow::{Context, Result, ensure};
use bytes::Bytes;
use http::{Request, header};
use http_body_util::{BodyExt, Limited};
use nextjson::{Value, json};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use url::Url;

pub(crate) const CLOUDFLARE_DECISION_MODELS: &[&str] = &[
    "typesafe/jev",
    "@cf/cloudflare/clef",
    "@cf/cloudflare/clef-flash",
];
const API: &str = "/client/v4";
const ACCOUNTS: &str = "/accounts";
const RUN: &str = "/ai/run";

pub(crate) fn cloudflare_decision_base(base: &str) -> Option<Url> {
    let mut url = config::endpoint(base).ok()?;
    let path = url.path().trim_end_matches('/');
    let (prefix, rest) = if path.is_empty() && url.host_str() == Some("api.cloudflare.com") {
        ("", "")
    } else {
        path.split_once(API)?
    };
    let account = match rest {
        "" | ACCOUNTS => None,
        _ => {
            let account_path = rest.strip_prefix("/accounts/")?;
            let (id, suffix) = account_path.split_once('/').unwrap_or((account_path, ""));
            if !matches!(
                suffix,
                "" | "ai/run" | "ai/run/@cf/cloudflare/clef" | "ai/run/@cf/cloudflare/clef-flash"
            ) {
                return None;
            }
            if valid_account(id) {
                Some(id)
            } else if id.starts_with(['$', ':', '{', '<'])
                || id.starts_with("%7B")
                || id.starts_with("%3C")
            {
                None
            } else {
                return None;
            }
        }
    };
    let path = match account {
        Some(id) => format!("{prefix}{API}{ACCOUNTS}/{id}{RUN}"),
        None => format!("{prefix}{API}{ACCOUNTS}"),
    };
    url.set_path(&path);
    Some(url)
}

pub(in crate::gateway) fn workers_ai(url: &Url) -> bool {
    let Some((_, path)) = url.path().split_once("/client/v4/accounts") else {
        return false;
    };
    path.is_empty()
        || path
            .strip_prefix('/')
            .and_then(|path| path.strip_suffix(RUN))
            .is_some_and(valid_account)
}

fn valid_account(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|char| char.is_ascii_alphanumeric() || matches!(char, b'-' | b'_'))
}

fn clef(model: &str) -> Option<&str> {
    CLOUDFLARE_DECISION_MODELS[1..]
        .contains(&model)
        .then(|| model.trim_start_matches("@cf/cloudflare/"))
}

struct FoundAccount {
    token_hash: [u8; 32],
    id: String,
    found_at: Instant,
}

#[derive(Default)]
pub(in crate::gateway) struct AccountCache(Mutex<Option<FoundAccount>>);

impl AccountCache {
    pub(in crate::gateway) async fn run_url(
        &self,
        base: &Url,
        model: &str,
        client: &ForwardClient,
        key: Option<&str>,
    ) -> Result<Url> {
        let mut url = base.clone();
        if url.path().ends_with(ACCOUNTS) {
            let key = key.context("Cloudflare Workers AI account discovery requires an API token or an account ID in systemone")?;
            let account =
                tokio::time::timeout(Duration::from_secs(20), self.account(client, base, key))
                    .await
                    .context("Workers AI account lookup timed out")??;
            url.path_segments_mut()
                .map_err(|_| anyhow::anyhow!("invalid Workers AI URL"))?
                .push(&account)
                .extend(["ai", "run"]);
        }
        if clef(model).is_some() {
            url.path_segments_mut()
                .map_err(|_| anyhow::anyhow!("invalid Workers AI URL"))?
                .extend(model.split('/'));
        }
        Ok(url)
    }

    async fn account(&self, client: &ForwardClient, base: &Url, key: &str) -> Result<String> {
        let token_hash = Sha256::digest(key).into();
        let mut slot = self.0.lock().await;
        if let Some(entry) = slot.as_ref()
            && entry.token_hash == token_hash
            && entry.found_at.elapsed() < Duration::from_secs(600)
        {
            return Ok(entry.id.clone());
        }
        let id = find_account(client, base, key).await?;
        *slot = Some(FoundAccount {
            token_hash,
            id: id.clone(),
            found_at: Instant::now(),
        });
        Ok(id)
    }
}

async fn find_account(client: &ForwardClient, base: &Url, key: &str) -> Result<String> {
    let mut url = base.clone();
    url.set_query(None);
    let request = Request::get(url.as_str())
        .header(header::AUTHORIZATION, format!("Bearer {key}"))
        .body(Vec::<Bytes>::new())?;
    let response = client
        .send(request)
        .await
        .context("Workers AI account lookup failed")?;
    ensure!(
        response.status().is_success(),
        "Workers AI account lookup returned HTTP {}; put the account ID in systemone",
        response.status().as_u16()
    );
    let bytes = Limited::new(response.into_body(), 1024 * 1024)
        .collect()
        .await
        .map_err(|_| anyhow::anyhow!("could not read Workers AI account lookup within 1 MiB"))?
        .to_bytes();
    let value: Value = nextjson::from_slice(&bytes)
        .map_err(|_| anyhow::anyhow!("invalid Workers AI account list"))?;
    value
        .pointer("/result/0/id")
        .and_then(Value::as_str)
        .filter(|id| valid_account(id))
        .map(str::to_owned)
        .context("Workers AI token lists no account; put the account ID in systemone")
}

pub(super) fn request(mut value: Value, model: &str) -> Result<Value> {
    let body = value
        .as_object_mut()
        .context("expected a System One request object")?;
    let count = body
        .get("questions")
        .and_then(Value::as_object)
        .context("questions must be an object keyed by name")?
        .len();
    super::check_count(count)?;
    if let Some(model) = clef(model) {
        body.insert("model".into(), json!(model));
        Ok(value)
    } else {
        body.remove("model");
        Ok(json!({"model":model,"input":value}))
    }
}

pub(super) fn reply(mut value: Value) -> Result<Value> {
    ensure!(
        value.get("success").and_then(Value::as_bool) != Some(false),
        "Workers AI returned an error envelope"
    );
    let result = value
        .as_object_mut()
        .and_then(|value| value.remove("result"))
        .context("missing Workers AI result")?;
    ensure!(
        result.get("answers").and_then(Value::as_object).is_some(),
        "expected Workers AI answers keyed by name"
    );
    Ok(result)
}
