use super::{ModelCatalog, discover_directory};
use crate::{
    config::{self, Config, Provider},
    gateway::Gateway,
};
use anyhow::{Context, Result, bail, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use regex::{Regex, RegexBuilder};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::Instant;

const MIN_INTERVAL: Duration = Duration::from_secs(600);
const RETRY_INTERVAL: Duration = Duration::from_secs(900);
const FETCH_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, PartialEq, Eq, NsonSerialize, NsonDeserialize)]
#[njson(deny_unknown_fields)]
pub struct ModelDirectory {
    fetched_at: u64,
    available: BTreeSet<String>,
    #[njson(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub(crate) delisted: BTreeSet<String>,
}

impl ModelDirectory {
    fn delisted(&self) -> &BTreeSet<String> {
        &self.delisted
    }
}

#[derive(NsonSerialize)]
pub struct RefreshedModels {
    pub models: Vec<String>,
    pub added: Vec<String>,
    pub delisted: Vec<String>,
}

fn interval(raw: &str, remote: bool) -> Result<Option<Duration>> {
    let raw = raw.trim().to_ascii_lowercase();
    match raw.as_str() {
        "" => return Ok(remote.then_some(MIN_INTERVAL)),
        "off" | "never" | "no" | "none" | "0" | "-" => return Ok(None),
        _ => {}
    }
    let read = || -> Option<Duration> {
        let mut text = raw.as_str();
        let mut seconds = 0.0;
        while !text.is_empty() {
            let unit = text.find(|c: char| !(c.is_ascii_digit() || c == '.'))?;
            let number: f64 = text[..unit].parse().ok()?;
            let (suffix, scale) = [
                ("ms", 0.001),
                ("us", 0.000_001),
                ("µs", 0.000_001),
                ("μs", 0.000_001),
                ("ns", 0.000_000_001),
                ("s", 1.0),
                ("m", 60.0),
                ("h", 3600.0),
                ("d", 86400.0),
            ]
            .into_iter()
            .find(|(suffix, _)| text[unit..].starts_with(suffix))?;
            seconds += number * scale;
            text = &text[unit + suffix.len()..];
        }
        Duration::try_from_secs_f64(seconds).ok()
    };
    let duration = read().context("use a refresh interval such as 10m, 6h, 1d or off")?;
    ensure!(
        duration >= MIN_INTERVAL,
        "a model directory can refresh at most every 10 minutes"
    );
    Ok(Some(duration))
}

fn patterns(rules: &[String]) -> Result<Vec<Regex>> {
    rules
        .iter()
        .map(|rule| {
            let (expression, insensitive) = if let Some(re) = rule.strip_prefix("re:") {
                ensure!(!re.is_empty(), "empty automatic model regular expression");
                (re.to_owned(), false)
            } else {
                ensure!(rule.contains('*'), "automatic model rule must use * or re:");
                (
                    rule.split('*')
                        .map(regex::escape)
                        .collect::<Vec<_>>()
                        .join(".*"),
                    true,
                )
            };
            RegexBuilder::new(&format!(r"\A(?:{expression})\z"))
                .case_insensitive(insensitive)
                .size_limit(256 * 1024)
                .build()
                .context("invalid automatic model rule")
        })
        .collect()
}

fn owns_directory(provider: &Provider) -> bool {
    provider.web_site.is_none()
        && super::fixed_models(&provider.endpoints).is_none()
        && (provider.endpoints.vertex.is_some()
            || [
                &provider.endpoints.chat,
                &provider.endpoints.responses,
                &provider.endpoints.anthropic,
                &provider.endpoints.gemini,
            ]
            .into_iter()
            .any(|endpoint| !endpoint.is_empty()))
}

pub(crate) fn validate_refresh(provider: &Provider) -> Result<()> {
    if provider.refresh_every.is_empty()
        && provider.auto_include.is_empty()
        && !provider.remote_gateway
        && provider.model_directory.is_none()
    {
        return Ok(());
    }
    ensure!(
        owns_directory(provider),
        "this provider does not manage a model directory"
    );
    interval(&provider.refresh_every, provider.remote_gateway)?;
    patterns(&provider.auto_include)?;
    if let Some(list) = &provider.model_directory {
        ensure!(
            !list.available.is_empty()
                && list.available.iter().all(|id| config::valid_model_id(id)),
            "invalid cached model directory"
        );
        ensure!(
            list.delisted.is_disjoint(&list.available)
                && list.delisted.iter().all(|id| config::valid_model_id(id)),
            "invalid delisted models"
        );
    }
    Ok(())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn same_source(a: &Provider, b: &Provider) -> bool {
    let bare = |p: &Provider| {
        let mut endpoints = p.endpoints.clone();
        endpoints.auth_header = Default::default();
        endpoints
    };
    a.key_env == b.key_env
        && a.remote_gateway == b.remote_gateway
        && a.web_site.is_none()
        && b.web_site.is_none()
        && bare(a) == bare(b)
}

fn take_matching(
    provider: &mut Provider,
    rules: &[Regex],
    ids: impl IntoIterator<Item = String>,
) -> Vec<String> {
    let mut selected: BTreeSet<_> = provider.models.iter().cloned().collect();
    let taken: Vec<_> = ids
        .into_iter()
        .filter(|id| rules.iter().any(|rule| rule.is_match(id)) && selected.insert(id.clone()))
        .collect();
    provider.models.splice(..0, taken.iter().cloned());
    taken
}

pub(super) fn apply_catalog(
    provider: &mut Provider,
    mut fetched: ModelCatalog,
) -> Result<RefreshedModels> {
    ensure!(
        !fetched.partial,
        "part of the upstream model directory was unavailable"
    );
    fetched.models.retain(|id| config::valid_model_id(id));
    let available: BTreeSet<_> = fetched.models.iter().cloned().collect();
    ensure!(
        !available.is_empty(),
        "upstream returned no valid model IDs"
    );
    let (before, mut delisted) = provider
        .model_directory
        .take()
        .map(|list| (list.available, list.delisted))
        .unwrap_or_default();
    let rules = patterns(&provider.auto_include)?;
    let added = take_matching(
        provider,
        &rules,
        fetched
            .models
            .iter()
            .filter(|id| !before.contains(*id))
            .cloned(),
    );
    delisted.retain(|id| !available.contains(id) && provider.models.contains(id));
    let removed: Vec<String> = provider
        .models
        .iter()
        .filter(|id| {
            before.contains(*id) && !available.contains(*id) && delisted.insert((*id).clone())
        })
        .cloned()
        .collect();
    provider.free_models.retain(|id| !before.contains(id));
    provider.free_models.extend(
        fetched
            .free_models
            .into_iter()
            .filter(|id| available.contains(id)),
    );
    provider
        .model_protocols
        .retain(|id, _| !before.contains(id));
    provider.model_protocols.extend(
        fetched
            .model_protocols
            .into_iter()
            .filter(|(id, _)| available.contains(id)),
    );
    provider
        .model_reasoning_levels
        .retain(|id, _| !before.contains(id));
    provider.model_reasoning_levels.extend(
        fetched
            .model_reasoning_levels
            .into_iter()
            .filter(|(id, _)| available.contains(id)),
    );
    if let Some(mode) = fetched.learned_auth {
        provider.endpoints.auth_header = mode;
    }
    provider.model_directory = Some(ModelDirectory {
        fetched_at: now(),
        available,
        delisted,
    });
    Ok(RefreshedModels {
        models: fetched.models,
        added,
        delisted: removed,
    })
}

pub(crate) fn on_save(previous: &Config, updated: &mut Config) -> Result<()> {
    for provider in &mut updated.providers {
        let Some(old) = previous.providers.iter().find(|p| p.id == provider.id) else {
            continue;
        };
        if (old.model_directory.is_some() || provider.model_directory.is_some())
            && !same_source(old, provider)
        {
            provider.model_directory = None;
            continue;
        }
        if provider.auto_include != old.auto_include
            && let Some(list) = &provider.model_directory
        {
            let rules = provider
                .auto_include
                .iter()
                .filter(|rule| !old.auto_include.contains(rule))
                .cloned()
                .collect::<Vec<_>>();
            let ids = list.available.iter().cloned().collect::<Vec<_>>();
            take_matching(provider, &patterns(&rules)?, ids);
        }
    }
    Ok(())
}

pub async fn refresh_provider(root: &Path, id: &str) -> Result<RefreshedModels> {
    let config = config::load(root)?;
    let provider = config
        .providers
        .iter()
        .find(|p| p.id == id)
        .context("provider not found")?;
    refresh(root, provider).await
}

async fn refresh(root: &Path, provider: &Provider) -> Result<RefreshedModels> {
    ensure!(
        owns_directory(provider),
        "this provider does not manage a model directory"
    );
    let fetched = tokio::time::timeout(
        FETCH_TIMEOUT,
        discover_directory(
            &provider.endpoints,
            &provider.key_env,
            provider.remote_gateway,
        ),
    )
    .await
    .context("upstream model directory timed out")??;
    let _lock = config::settings_lock(root)?;
    let mut config = config::load(root)?;
    let p = config
        .providers
        .iter_mut()
        .find(|p| p.id == provider.id)
        .context("provider removed during catalog refresh")?;
    if nextjson::to_string(p).map_err(anyhow::Error::msg)?
        != nextjson::to_string(provider).map_err(anyhow::Error::msg)?
    {
        bail!("provider changed during catalog refresh");
    }
    let report = apply_catalog(p, fetched)?;
    config.validate()?;
    config::write_settings(root, &config)?;
    Ok(report)
}

async fn refresh_due(root: &Path, active: &Config, attempted: &mut BTreeMap<String, Instant>) {
    let Ok(cfg) = config::load(root) else {
        return;
    };
    for p in &cfg.providers {
        if !active
            .providers
            .iter()
            .any(|old| old.id == p.id && same_source(old, p))
        {
            continue;
        }
        let Ok(Some(every)) = interval(&p.refresh_every, p.remote_gateway) else {
            continue;
        };
        let elapsed = p
            .model_directory
            .as_ref()
            .map(|list| now().saturating_sub(list.fetched_at));
        if elapsed.is_some_and(|seconds| Duration::from_secs(seconds) < every)
            || attempted
                .get(&p.id)
                .is_some_and(|last| last.elapsed() < every.min(RETRY_INTERVAL))
        {
            continue;
        }
        attempted.insert(p.id.clone(), Instant::now());
        if let Err(error) = refresh(root, p).await {
            eprintln!("Could not refresh models for {}: {error:#}", p.id);
        }
    }
}

fn publish_catalog(root: &Path, app: &Gateway, active: &mut Config) -> Result<()> {
    let saved = config::load(root)?;
    let mut updated = active.clone();
    let mut changed = false;
    for p in &mut updated.providers {
        let Some(saved) = saved
            .providers
            .iter()
            .find(|q| q.id == p.id && same_source(p, q))
        else {
            continue;
        };
        if saved.model_directory.is_some() || p.model_directory.is_some() {
            changed |= p.model_directory.as_ref().map(ModelDirectory::delisted)
                != saved.model_directory.as_ref().map(ModelDirectory::delisted)
                || p.models != saved.models
                || p.free_models != saved.free_models
                || p.model_protocols != saved.model_protocols
                || p.model_reasoning_levels != saved.model_reasoning_levels
                || p.endpoints.auth_header != saved.endpoints.auth_header;
            p.models.clone_from(&saved.models);
            p.model_directory.clone_from(&saved.model_directory);
            p.free_models.clone_from(&saved.free_models);
            p.model_protocols.clone_from(&saved.model_protocols);
            p.model_reasoning_levels
                .clone_from(&saved.model_reasoning_levels);
            p.endpoints.auth_header = saved.endpoints.auth_header;
        }
    }
    if changed {
        app.replace_catalog(updated.clone())?;
    }
    *active = updated;
    Ok(())
}

pub(crate) async fn maintain(root: &Path, app: Arc<Gateway>, mut active: Config) {
    let mut attempted = BTreeMap::new();
    let mut ticks = tokio::time::interval_at(
        Instant::now() + Duration::from_secs(30),
        Duration::from_secs(60),
    );
    ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        ticks.tick().await;
        refresh_due(root, &active, &mut attempted).await;
        if let Err(error) = publish_catalog(root, &app, &mut active) {
            eprintln!("Could not update the gateway model directory: {error:#}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{gateway, service};
    use crabforge_test_support::{
        Client,
        server::{Incoming, Router, get, post, serve},
    };
    use http::StatusCode;
    use nextjson::{Value, json};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::{net::TcpListener, sync::Notify};

    #[tokio::test]
    async fn scheduled_refresh_publishes_new_models_and_keeps_deselections() -> Result<()> {
        let calls = Arc::new(AtomicUsize::new(0));
        let state = calls.clone();
        let upstream = TcpListener::bind("127.0.0.1:0").await?;
        let origin = format!("http://{}/v1", upstream.local_addr()?);
        let upstream = tokio::spawn(serve(upstream, Router::new().route(
            "/v1/models", get(move |Incoming { headers, .. }| {
                let call = state.fetch_add(1, Ordering::Relaxed);
                async move {
                    assert_eq!(headers["x-magpie-drawers"], "1");
                    assert_eq!(headers["x-magpie-deciders"], "1");
                    assert_eq!(headers["x-magpie-videomakers"], "1");
                    let payload = match call {
                        0 => json!({"data":[{"id":"keep"},{"id":"steady"},{"id":"chat-old"},{"id":"chat-muted"}]}),
                        1 => json!({"data":[{"id":"keep"},{"id":"steady"},{"id":"chat-muted"},{"id":"CHAT-new","free":true,"native_endpoints":["/v1/chat/completions"],"reasoning_levels":["low","high"]},{"id":"embed-new"}]}),
                        3 => json!({"data":[{"id":"keep"},{"id":"steady"},{"id":"chat-old"},{"id":"chat-muted"},{"id":"CHAT-new"},{"id":"embed-new"}]}),
                        _ => return (StatusCode::SERVICE_UNAVAILABLE, String::new()),
                    };
                    (StatusCode::OK, payload.to_string())
                }
            })
        ).route("/v1/chat/completions", post(|Incoming { body, .. }| async move {
            let request: Value = nextjson::from_slice(&body).expect("chat request");
            json!({"model":request["model"],"choices":[]}).to_string()
        }))));
        let root = tempfile::tempdir()?;
        let dir = root.path();
        let settings = json!({"providers":[{
            "id":"relay","endpoints":{"chat":origin},"models":["keep","steady","chat-old","chat-muted"],
            "remote_gateway":true,"auto_include":["chat-*"]
        }],"groups":{"backup":["relay/chat-old","relay/keep"],"stable":["relay/keep","relay/steady"]},"rotating_groups":["stable"]});
        config::save(dir, &config::snapshot(dir)?, &settings.to_string())?;
        let mut active = config::load(dir)?;
        let gateway = gateway::router(
            active.clone(),
            "fixture-key".into(),
            Default::default(),
            dir,
        )?;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let base = format!("http://{}", listener.local_addr()?);
        let server = tokio::spawn(service::serve_local(listener, gateway.clone()));
        let client = Client::default();
        let next_member = || async {
            let response = client
                .post(format!("{base}/v1/chat/completions"))
                .bearer_auth("fixture-key")
                .header("content-type", "application/json")
                .body(
                    json!({"model":"group/stable","messages":[{"role":"user","content":"hello"}]})
                        .to_string(),
                )
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?;
            let response: Value = nextjson::from_slice(&response).map_err(anyhow::Error::msg)?;
            Ok::<_, anyhow::Error>(response["model"].clone())
        };
        assert_eq!(next_member().await?, json!("keep"));
        let mut attempted = BTreeMap::new();
        refresh_due(dir, &active, &mut attempted).await;
        let previous = config::snapshot(dir)?;
        let mut disk = config::load(dir)?;
        disk.providers[0].models.retain(|id| id != "chat-muted");
        config::save(dir, &previous, &disk.to_json()?)?;
        refresh_due(dir, &active, &mut attempted).await;
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        let result = refresh_provider(dir, "relay").await?;
        assert_eq!(result.added, ["CHAT-new"]);
        assert_eq!(result.delisted, ["chat-old"]);
        publish_catalog(dir, &gateway, &mut active)?;
        assert_eq!(next_member().await?, json!("steady"));
        let catalog = client
            .get(format!("{base}/v1/models"))
            .bearer_auth("fixture-key")
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        let catalog: Value = nextjson::from_slice(&catalog).map_err(anyhow::Error::msg)?;
        let ids: BTreeSet<_> = catalog["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id["id"].as_str().unwrap())
            .collect();
        assert_eq!(
            ids,
            BTreeSet::from([
                "CHAT-new",
                "group/backup",
                "group/stable",
                "keep",
                "relay/CHAT-new",
                "relay/keep",
                "relay/steady",
                "steady"
            ])
        );
        let dropped = client
            .get(format!("{base}/v1/models/relay/chat-old"))
            .bearer_auth("fixture-key")
            .send()
            .await?;
        assert_eq!(dropped.status(), StatusCode::NOT_FOUND);
        let saved = config::snapshot(dir)?;
        assert!(refresh_provider(dir, "relay").await.is_err());
        assert_eq!(config::snapshot(dir)?, saved);
        let mut reopened = config::load(dir)?;
        assert!(reopened.providers[0].free_models.contains("CHAT-new"));
        assert_eq!(
            reopened.providers[0].model_reasoning_levels["CHAT-new"],
            ["low", "high"]
        );
        reopened.providers[0]
            .auto_include
            .push("re:embed-.*".into());
        config::save(dir, &saved, &reopened.to_json()?)?;
        let provider = &config::load(dir)?.providers[0];
        assert!(provider.models.iter().any(|model| model == "embed-new"));
        assert!(!provider.models.iter().any(|model| model == "chat-muted"));
        assert!(refresh_provider(dir, "relay").await?.added.is_empty());
        publish_catalog(dir, &gateway, &mut active)?;
        let restored = client
            .get(format!("{base}/v1/models/relay/chat-old"))
            .bearer_auth("fixture-key")
            .send()
            .await?;
        assert_eq!(restored.status(), StatusCode::OK);
        server.abort();
        upstream.abort();
        Ok(())
    }

    #[tokio::test]
    async fn refresh_rejects_partial_sources_and_concurrent_provider_edits() -> Result<()> {
        let failures = Arc::new(AtomicUsize::new(0));
        let fixture_failures = failures.clone();
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let fixture_entered = entered.clone();
        let fixture_release = release.clone();
        let upstream = TcpListener::bind("127.0.0.1:0").await?;
        let origin = format!("http://{}", upstream.local_addr()?);
        let server = tokio::spawn(serve(
            upstream,
            Router::new()
                .route(
                    "/v1/models",
                    get(|_| async { json!({"data":[{"id":"listed"}]}).to_string() }),
                )
                .route(
                    "/missing/models",
                    get(move |_| {
                        fixture_failures.fetch_add(1, Ordering::Relaxed);
                        async { StatusCode::SERVICE_UNAVAILABLE }
                    }),
                )
                .route(
                    "/held/models",
                    get(move |_| {
                        let entered = fixture_entered.clone();
                        let release = fixture_release.clone();
                        async move {
                            entered.notify_one();
                            release.notified().await;
                            json!({"data":[{"id":"late"}]}).to_string()
                        }
                    }),
                ),
        ));
        let root = tempfile::tempdir()?;
        let dir = root.path();
        let settings = json!({"providers":[{"id":"test","models":["listed"],"auto_include":["*"],"refresh_every":"6h",
            "endpoints":{"chat":format!("{origin}/v1"),"responses":format!("{origin}/missing")}}]});
        config::save(dir, &config::snapshot(dir)?, &settings.to_string())?;
        let before = config::snapshot(dir)?;
        let error = refresh_provider(dir, "test")
            .await
            .err()
            .context("expected partial refresh to fail")?;
        assert!(
            error
                .to_string()
                .contains("part of the upstream model directory")
        );
        assert_eq!(config::snapshot(dir)?, before);

        let active = config::load(dir)?;
        let mut attempted = BTreeMap::new();
        refresh_due(dir, &active, &mut attempted).await;
        assert_eq!(failures.load(Ordering::Relaxed), 2);
        attempted.insert("test".into(), Instant::now() - Duration::from_secs(14 * 60));
        refresh_due(dir, &active, &mut attempted).await;
        assert_eq!(failures.load(Ordering::Relaxed), 2);
        attempted.insert("test".into(), Instant::now() - Duration::from_secs(15 * 60));
        refresh_due(dir, &active, &mut attempted).await;
        assert_eq!(failures.load(Ordering::Relaxed), 3);
        assert_eq!(config::snapshot(dir)?, before);

        let mut edited = active;
        edited.providers[0].endpoints.models_url = format!("{origin}/held/models");
        config::save(dir, &before, &edited.to_json()?)?;
        let data_dir = dir.to_owned();
        let attempt = tokio::spawn(async move { refresh_provider(&data_dir, "test").await });
        tokio::time::timeout(Duration::from_secs(10), entered.notified()).await?;
        edited.providers[0].models = vec!["saved-while-fetching".into()];
        config::save(dir, &config::snapshot(dir)?, &edited.to_json()?)?;
        let newer = config::snapshot(dir)?;
        release.notify_one();
        let error = attempt
            .await?
            .err()
            .context("expected the concurrent edit to reject refresh")?;
        assert!(
            error
                .to_string()
                .contains("provider changed during catalog refresh")
        );
        assert_eq!(config::snapshot(dir)?, newer);
        server.abort();
        Ok(())
    }
}
