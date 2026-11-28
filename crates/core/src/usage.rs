//! Read-only quota windows reported by configured API-key providers.

use crate::{config, opencode::Console, upstream::ForwardClient};
use anyhow::{Context, Result, ensure};
use http::{HeaderValue, Request, header};
use http_body_util::{BodyExt, Limited};
use nextjson::{NsonSerialize, Value};
use std::{path::Path, time::Duration};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use url::Url;
use zeroize::Zeroizing;

mod plans;

const MAX_REPLY: usize = 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(15);
const HOUR: u64 = 3600;
const WEEK: u64 = 7 * 24 * HOUR;
const MONTH: u64 = 30 * 24 * HOUR;

#[derive(NsonSerialize)]
pub struct Report {
    pub provider: String,
    pub plan: Option<String>,
    pub windows: Vec<Window>,
}

#[derive(NsonSerialize)]
pub struct Window {
    pub name: String,
    pub used_percent: Option<f64>,
    pub duration_seconds: Option<u64>,
    pub resets_at: Option<String>,
    pub used: Option<f64>,
    pub limit: Option<f64>,
    pub balance: Option<f64>,
    pub unit: Option<String>,
    /// A separate allowance or one whose exhaustion does not block requests.
    pub aside: bool,
}

impl Window {
    fn percent(
        name: impl Into<String>,
        percent: f64,
        span: u64,
        reset: Option<OffsetDateTime>,
    ) -> Self {
        Self {
            name: name.into(),
            used_percent: Some(percent.clamp(0.0, 100.0)),
            duration_seconds: (span > 0).then_some(span),
            resets_at: reset.and_then(|date| date.format(&Rfc3339).ok()),
            used: None,
            limit: None,
            balance: None,
            unit: None,
            aside: false,
        }
    }

    fn balance(name: &str, balance: f64, unit: &str) -> Self {
        Self {
            used_percent: None,
            balance: Some(balance.max(0.0)),
            unit: Some(unit.to_owned()),
            aside: true,
            ..Self::percent(name, 0.0, 0, None)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Plan {
    OpenCodeGo,
    Zhipu,
    Kimi,
    CommandCode,
    MiniMax,
    Ollama,
    Mistral,
}

fn source(p: &config::Provider) -> Option<(Plan, Url)> {
    if p.remote_gateway || p.web_site.is_some() || p.endpoints.vertex.is_some() {
        return None;
    }
    [
        &p.endpoints.chat,
        &p.endpoints.anthropic,
        &p.endpoints.responses,
    ]
    .into_iter()
    .filter_map(|base| config::endpoint(base).ok())
    .filter(|base| base.scheme() == "https" && base.port().is_none())
    .find_map(|mut url| {
        let host = url.host_str()?;
        let (plan, path) = match host {
            "opencode.ai" if Console::at(&url) == Some(Console::Go) => {
                (Plan::OpenCodeGo, "/zen/go/v1/usage")
            }
            "open.bigmodel.cn" | "api.z.ai" => (Plan::Zhipu, "/api/monitor/usage/quota/limit"),
            "api.kimi.com" | "api.kimi.ai"
                if matches!(url.path().trim_end_matches('/'), "/coding" | "/coding/v1") =>
            {
                (Plan::Kimi, "/coding/v1/usages")
            }
            "api.commandcode.ai" => (Plan::CommandCode, "/alpha/billing/credits"),
            "api.minimax.io" | "api.minimax.cn" | "api.minimaxi.com" => {
                (Plan::MiniMax, "/v1/token_plan/remains")
            }
            "ollama.com" => (Plan::Ollama, "/api/balance"),
            "api.mistral.ai" => (Plan::Mistral, "/v1/billing/budget"),
            _ => return None,
        };
        url.set_path(path);
        Some((plan, url))
    })
}

pub async fn provider(dir: &Path, id: &str) -> Result<Report> {
    let cfg = config::load(dir)?;
    let provider = cfg
        .providers
        .iter()
        .find(|p| p.id == id)
        .context("provider not found")?;
    source(provider).context("provider has no supported subscription usage endpoint")?;
    ensure!(
        !provider.key_env.is_empty(),
        "provider key environment variable is missing or empty"
    );
    let key = Zeroizing::new(
        std::env::var(&provider.key_env)
            .ok()
            .filter(|key| !key.is_empty())
            .context("provider key environment variable is missing or empty")?,
    );
    report(provider, &ForwardClient::new(), &key).await
}

pub(crate) async fn report(
    provider: &config::Provider,
    client: &ForwardClient,
    key: &str,
) -> Result<Report> {
    let (plan, url) =
        source(provider).context("provider has no supported subscription usage endpoint")?;
    let (plan, windows) = if plan == Plan::Mistral {
        let who = fetch(
            client,
            &Url::parse("https://console.mistral.ai/api/vibe/whoami")?,
            key,
            true,
        )
        .await?;
        let kind = who
            .get("plan_type")
            .and_then(Value::as_str)
            .context("Mistral did not identify the key type")?
            .trim()
            .to_ascii_lowercase();
        if matches!(kind.as_str(), "chat" | "api") {
            let mut subscription = url.clone();
            subscription.set_path("/v1/billing/subscription");
            let (budget, subscription) = tokio::join!(
                fetch(client, &url, key, true),
                fetch(client, &subscription, key, true),
            );
            plans::mistral(&budget?, &subscription.unwrap_or(Value::Null), &kind)
        } else {
            (None, Vec::new())
        }
    } else {
        let value = fetch(client, &url, key, plan != Plan::Zhipu).await?;
        plans::parse(plan, &value)?
    };
    Ok(Report {
        provider: provider.id.clone(),
        plan,
        windows,
    })
}

pub fn available(cfg: &config::Config) -> Vec<&str> {
    cfg.providers
        .iter()
        .filter(|provider| !provider.key_env.is_empty() && source(provider).is_some())
        .map(|provider| provider.id.as_str())
        .collect()
}

async fn fetch(client: &ForwardClient, url: &Url, key: &str, bearer: bool) -> Result<Value> {
    let auth = Zeroizing::new(if bearer {
        format!("Bearer {key}")
    } else {
        key.to_owned()
    });
    let mut value =
        HeaderValue::from_str(&auth).context("invalid subscription authorization header")?;
    value.set_sensitive(true);
    let mut request = Request::get(url.as_str())
        .header(header::AUTHORIZATION, value)
        .header(header::ACCEPT, "application/json")
        .header(header::ACCEPT_LANGUAGE, "en-US,en");
    if url.host_str() == Some("opencode.ai") && url.path() == "/zen/go/v1/usage" {
        request = crate::opencode::conversation(request, None, b"subscription usage", key);
    }
    let request = request.body(Vec::new())?;
    tokio::time::timeout(TIMEOUT, async {
        let response = client
            .send(request)
            .await
            .context("subscription request failed")?;
        let status = response.status();
        ensure!(
            status.is_success(),
            "subscription endpoint returned HTTP {}",
            status.as_u16()
        );
        let bytes = Limited::new(response.into_body(), MAX_REPLY)
            .collect()
            .await
            .map_err(anyhow::Error::from_boxed)?
            .to_bytes();
        let value: Value = nextjson::from_slice(&bytes)
            .map_err(|_| anyhow::anyhow!("invalid subscription response JSON"))?;
        ensure!(value.is_object(), "subscription response must be an object");
        Ok(value)
    })
    .await
    .context("subscription request timed out")?
}

fn number(value: Option<&Value>) -> Option<f64> {
    let value = value?;
    value
        .as_f64()
        .or_else(|| value.as_str()?.trim().parse().ok())
        .filter(|n| n.is_finite())
}

fn reset(value: Option<&Value>) -> Option<OffsetDateTime> {
    let value = value?;
    if let Some(date) = value
        .as_str()
        .and_then(|date| OffsetDateTime::parse(date, &Rfc3339).ok())
    {
        return Some(date);
    }
    let n = number(Some(value)).filter(|n| *n > 0.0)?;
    let ms = if n >= 1e11 { n } else { n * 1000.0 };
    OffsetDateTime::from_unix_timestamp_nanos((ms as i128).checked_mul(1_000_000)?).ok()
}

fn duration_name(span: u64) -> String {
    match span {
        0 => "Allowance".into(),
        MONTH => "Month".into(),
        span if span >= 24 * HOUR && span.is_multiple_of(24 * HOUR) => {
            format!("{} days", span / (24 * HOUR))
        }
        span if span >= HOUR && span.is_multiple_of(HOUR) => format!("{} hours", span / HOUR),
        span => format!("{} minutes", span / 60),
    }
}
