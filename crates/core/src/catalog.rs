use crate::config::{Endpoints, api_base, authenticate};
use anyhow::{Context, Result, bail, ensure};
use futures_util::StreamExt;
use reqwest::{Client, Url};
use serde::Deserialize;
use std::{collections::HashSet, time::Duration};
mod builtin;
pub use builtin::fixed_models;

/// Include the local lists in the catalog shown by both the CLI and Flutter.
pub fn presets() -> Result<String> {
    let mut entries: Vec<serde_json::Value> = serde_json::from_str(crate::config::PRESETS)?;
    for entry in &mut entries {
        let bases = Endpoints::deserialize(&entry["endpoints"])?;
        if let Some(ids) = fixed_models(&bases) {
            entry["models"] = serde_json::json!(ids);
        }
        // Nexus and ordinary Fireworks share a base URL. Only the explicitly
        // selected Nexus plan should suggest its auto model.
        if entry["id"] == "fireworks-nexus" {
            entry["models"] = serde_json::json!(["auto"]);
        }
    }
    Ok(serde_json::to_string_pretty(&entries)?)
}

#[derive(Deserialize)]
struct List {
    #[serde(default)]
    data: Vec<Entry>,
    #[serde(default)]
    models: Vec<Entry>,
    #[serde(rename = "nextPageToken")]
    next: Option<String>,
    has_more: Option<bool>,
    last_id: Option<String>,
}
#[derive(Deserialize)]
struct Entry {
    id: Option<String>,
    name: Option<String>,
    #[serde(rename = "supportedGenerationMethods")]
    methods: Option<Vec<String>>,
}

struct Source {
    url: Url,
    api: &'static str,
    deployments: bool,
}

fn sources(bases: &Endpoints) -> Result<Vec<Source>> {
    let protocols = [
        (&bases.chat, "chat"),
        (&bases.responses, "chat"),
        (&bases.anthropic, "anthropic"),
        (&bases.gemini, "gemini"),
    ];
    if !bases.models_url.is_empty() {
        let api = if bases.vertex.is_some() {
            "gemini"
        } else {
            protocols
                .iter()
                .find(|(base, _)| !base.is_empty())
                .context("configure an API endpoint first")?
                .1
        };
        return Ok(vec![Source {
            url: crate::config::catalog_url(&bases.models_url)?,
            api,
            deployments: false,
        }]);
    }
    let ai_studio_pair = bases.chat.trim_end_matches('/')
        == "https://generativelanguage.googleapis.com/v1beta/openai"
        && bases.gemini.trim_end_matches('/') == "https://generativelanguage.googleapis.com/v1beta";
    let mut sources = Vec::new();
    for (base, api) in protocols {
        if base.is_empty()
            || (ai_studio_pair
                && base.trim_end_matches('/')
                    == "https://generativelanguage.googleapis.com/v1beta/openai")
        {
            continue;
        }
        let address = api_base(base, api)?;
        let (url, api) = match crate::opencode::Console::at(&address) {
            Some(console) => (console.models_url(&address), "chat"),
            None => (list_url(address, api)?, api),
        };
        if api == "anthropic"
            && url.host_str() == Some("openrouter.ai")
            && (!bases.chat.is_empty() || !bases.responses.is_empty())
        {
            // Its Messages directory repeats the Chat models with unusable prefixes.
            continue;
        }
        if sources
            .iter()
            .any(|s: &Source| s.url == url && s.api == api)
        {
            continue;
        }
        let deployments = crate::config::azure(&url) && api == "chat";
        sources.push(Source {
            url,
            api,
            deployments,
        });
    }
    ensure!(!sources.is_empty(), "configure an API endpoint first");
    Ok(sources)
}

fn list_url(mut url: Url, api: &str) -> Result<Url> {
    if crate::config::mantle(&url) && url.path().trim_end_matches('/') == "/openai/v1" {
        url.set_path("/v1");
    }
    let append_v1 = api == "anthropic" && !url.path().trim_end_matches('/').ends_with("/v1");
    {
        let mut path = url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("invalid API base"))?;
        path.pop_if_empty();
        if append_v1 {
            path.push("v1");
        }
        path.push("models");
    }
    if api == "gemini" {
        url.query_pairs_mut().append_pair("pageSize", "1000");
    }
    Ok(url)
}

#[cfg(test)]
#[test]
fn console_catalogs_deduplicate_by_plan_and_allow_an_explicit_override() -> Result<()> {
    let mut bases: Endpoints = serde_json::from_value(serde_json::json!({
            "chat":"https://opencode.ai/zen/go/v1", "gemini":"https://opencode.ai/inference/go/google/v1beta",
            "anthropic":"https://opencode.ai/inference/anthropic"}))?;
    let locations = sources(&bases)?;
    assert_eq!(
        locations
            .iter()
            .map(|s| (s.url.as_str(), s.api))
            .collect::<Vec<_>>(),
        [
            ("https://opencode.ai/inference/go/v1/models", "chat"),
            ("https://opencode.ai/inference/v1/models", "chat")
        ]
    );
    bases.models_url = "https://relay.example/account/models".into();
    let locations = sources(&bases)?;
    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0].url.as_str(), bases.models_url);
    Ok(())
}

/// Merge complete protocol catalogs in configuration order. A broken page
/// invalidates that source only. Discovery never changes the saved model IDs.
pub async fn discover(bases: &Endpoints, key_env: &str) -> Result<Vec<String>> {
    discover_with_auth(bases, key_env, &Default::default()).await
}

pub async fn discover_provider(
    dir: &std::path::Path,
    provider: &crate::config::Provider,
) -> Result<Vec<String>> {
    let learned = std::sync::OnceLock::new();
    let models = discover_with_auth(&provider.endpoints, &provider.key_env, &learned).await?;
    if let Some(&mode) = learned.get() {
        let _ = crate::config::remember_auth(dir, provider, mode);
    }
    Ok(models)
}

async fn discover_with_auth(
    bases: &Endpoints,
    key_env: &str,
    learned: &std::sync::OnceLock<crate::config::AuthHeader>,
) -> Result<Vec<String>> {
    if let Some(v) = &bases.vertex {
        v.validate(!key_env.is_empty(), &bases.models_url)?;
    }
    if let Some(ids) = fixed_models(bases) {
        return Ok(ids.iter().map(|&id| id.to_owned()).collect());
    }
    let sources = sources(bases)?;
    let key = if key_env.is_empty() {
        None
    } else {
        Some(
            std::env::var(key_env)
                .ok()
                .filter(|k| !k.is_empty())
                .context("provider key environment variable is missing or empty")?,
        )
    };
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let mut errors = Vec::new();
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    let bearer = match (&bases.vertex, &key) {
        (Some(vertex), None) => Some(
            crate::vertex::auth::TokenCache::default()
                .token(&client, vertex)
                .await?,
        ),
        _ => None,
    };
    for source in sources {
        match fetch(
            &client,
            &source,
            key.as_deref(),
            key_env,
            bearer.as_deref(),
            bases.auth_header,
            learned,
        )
        .await
        {
            Ok(models) => ids.extend(models.into_iter().filter(|m| seen.insert(m.clone()))),
            Err(e) => {
                let mut location = source.url;
                location.set_query(None);
                errors.push(format!("{} {}: {e}", source.api, location));
            }
        }
    }
    ensure!(!ids.is_empty(), "{}", errors.join("; "));
    Ok(ids)
}

async fn fetch(
    client: &Client,
    source: &Source,
    key: Option<&str>,
    key_env: &str,
    bearer: Option<&str>,
    mode: crate::config::AuthHeader,
    learned: &std::sync::OnceLock<crate::config::AuthHeader>,
) -> Result<Vec<String>> {
    let Source {
        ref url,
        api,
        deployments,
    } = *source;
    let mut budget = 4 * 1024 * 1024;
    let mut cursors = HashSet::new();
    let mut ids = Vec::new();
    let mut page = url.clone();
    if deployments {
        page.set_path("/openai/deployments");
        page.set_query(Some("api-version=2022-12-01"));
    }
    for _ in 0..32 {
        let mut req = client.get(page.clone());
        if crate::opencode::Console::request(&page).is_some() {
            req = req.header(
                reqwest::header::USER_AGENT,
                crate::opencode::USER_AGENT_VALUE,
            );
        }
        if api == "anthropic" {
            req = req.header("anthropic-version", "2023-06-01");
        }
        if let Some(token) = bearer {
            req = req.bearer_auth(token);
        } else if let Some(k) = key {
            req = authenticate(req, url, api, k, mode);
        }
        let response = if api == "anthropic" && key.is_some() {
            crate::anthropic_auth::send(req, learned).await
        } else {
            crate::bedrock::send(req, key_env).await
        }
        .context("model catalog request failed")?;
        if deployments && page.path() == "/openai/deployments" && !response.status().is_success() {
            page = url.clone();
            continue;
        }
        ensure!(
            response.status().is_success(),
            "model catalog returned HTTP {}",
            response.status()
        );
        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("could not read the model catalog body")?;
            ensure!(chunk.len() <= budget, "model catalog exceeds 4 MiB");
            budget -= chunk.len();
            bytes.extend_from_slice(&chunk);
        }
        let entries: List = serde_json::from_slice(&bytes).context("invalid model catalog")?;
        for entry in entries.data.into_iter().chain(entries.models) {
            if api == "gemini"
                && entry.methods.as_ref().is_some_and(|methods| {
                    !methods.is_empty() && !methods.iter().any(|s| s == "generateContent")
                })
            {
                continue;
            }
            if let Some(id) = entry.id.or(entry.name) {
                let id = if api == "gemini" {
                    id.strip_prefix("models/").unwrap_or(&id)
                } else {
                    &id
                };
                if !id.is_empty() {
                    ids.push(id.to_owned());
                }
            }
        }
        let cursor = if api == "gemini" {
            entries
                .next
                .filter(|v| !v.is_empty())
                .map(|v| ("pageToken", v))
        } else if api == "anthropic" && entries.has_more == Some(true) {
            Some((
                "after_id",
                entries.last_id.context("missing catalog cursor")?,
            ))
        } else {
            None
        };
        let Some((key, value)) = cursor else {
            if ids.is_empty() && deployments && page.path() == "/openai/deployments" {
                page = url.clone();
                continue;
            }
            ensure!(!ids.is_empty(), "upstream returned no model IDs");
            return Ok(ids);
        };
        ensure!(
            !value.is_empty() && cursors.insert(value.clone()),
            "invalid or repeated model catalog cursor"
        );
        page = url.clone();
        if url.query_pairs().any(|(k, _)| k == key) {
            page.set_query(None);
            page.query_pairs_mut()
                .extend_pairs(url.query_pairs().filter(|(k, _)| k != key));
        }
        page.query_pairs_mut().append_pair(key, &value);
    }
    bail!("model catalog exceeds 32 pages")
}
