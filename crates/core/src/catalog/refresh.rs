use super::{ModelCatalog, discover_catalog};
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
        nextjson::to_string(&endpoints).expect("endpoint serialization")
    };
    a.key_env == b.key_env && a.web_site.is_none() && b.web_site.is_none() && bare(a) == bare(b)
}

fn take_matching(
    provider: &mut Provider,
    ids: impl IntoIterator<Item = String>,
) -> Result<Vec<String>> {
    let rules = patterns(&provider.auto_include)?;
    let mut selected: BTreeSet<_> = provider.models.iter().cloned().collect();
    let taken: Vec<_> = ids
        .into_iter()
        .filter(|id| rules.iter().any(|rule| rule.is_match(id)) && selected.insert(id.clone()))
        .collect();
    provider.models.splice(..0, taken.iter().cloned());
    Ok(taken)
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
    let added = take_matching(
        provider,
        fetched
            .models
            .iter()
            .filter(|id| !before.contains(*id))
            .cloned(),
    )?;
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
            let ids = list.available.iter().cloned().collect::<Vec<_>>();
            take_matching(provider, ids)?;
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
    let mut endpoints = provider.endpoints.clone();
    if provider.remote_gateway && endpoints.models_url.is_empty() && !endpoints.chat.is_empty() {
        endpoints.models_url =
            super::list_url(config::api_base(&endpoints.chat, "chat")?, "chat")?.to_string();
    }
    let fetched = tokio::time::timeout(
        FETCH_TIMEOUT,
        discover_catalog(&endpoints, &provider.key_env),
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

pub(crate) async fn maintain(root: &Path, app: Arc<Gateway>, mut active: Config) {
    let mut attempted = BTreeMap::new();
    let mut ticks = tokio::time::interval_at(
        Instant::now() + Duration::from_secs(30),
        Duration::from_secs(60),
    );
    ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        ticks.tick().await;
        let Ok(cfg) = config::load(root) else {
            continue;
        };
        attempted.retain(|id, _| cfg.providers.iter().any(|p| &p.id == id));
        for p in &cfg.providers {
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
                    .is_some_and(|last: &Instant| last.elapsed() < every.min(RETRY_INTERVAL))
            {
                continue;
            }
            attempted.insert(p.id.clone(), Instant::now());
            if let Err(error) = refresh(root, p).await {
                eprintln!("Could not refresh models for {}: {error:#}", p.id);
            }
        }
        let Ok(disk) = config::load(root) else {
            continue;
        };
        let mut updated = active.clone();
        let mut changed = false;
        for p in &mut updated.providers {
            let Some(saved) = disk
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
            if let Err(error) = app.replace_catalog(updated.clone()) {
                eprintln!("Could not update the gateway model directory: {error:#}");
                continue;
            }
        }
        active = updated;
    }
}
