use super::{EMPTY, ENABLED, MARKETPLACES, enabled, encode, patch_setting, read_object, valid_id};
use crate::{
    agents::NativeAgent,
    config::{open_magpie_backup, read_limited},
    native_settings,
    project_mcp::jsonc,
};
use anyhow::{Context, Result, ensure};
use nextjson::{Map, NsonDeserialize, NsonSerialize, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use url::Url;

const OFFICIAL: &str = "claude-plugins-official";
const LIMIT: usize = 1024 * 1024;

#[derive(Default, NsonDeserialize, NsonSerialize)]
struct Settings {
    #[nextjson(default)]
    marketplaces: BTreeMap<String, Value>,
    #[nextjson(default)]
    enabled: BTreeSet<String>,
}

#[derive(Default, NsonSerialize)]
pub struct ImportReport {
    marketplaces: BTreeMap<String, &'static str>,
    plugins: BTreeMap<String, &'static str>,
    applied: bool,
}

fn text(value: &Value, key: &str) -> Option<String> {
    let value = value.get(key)?.as_str()?;
    (!value.is_empty() && value.len() <= 2048 && !value.chars().any(char::is_control))
        .then(|| value.to_owned())
}

fn portable(entry: &Value) -> Option<Value> {
    let source = entry.get("source")?;
    let kind = source.get("source")?.as_str()?;
    let (field, locator) = match kind {
        "github" => {
            let repo = text(source, "repo")?;
            let (owner, name) = repo.split_once('/')?;
            if [owner, name].into_iter().any(|part| {
                part.is_empty()
                    || matches!(part, "." | "..")
                    || !part
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
            }) {
                return None;
            }
            ("repo", repo)
        }
        "git" | "url" => {
            let raw = text(source, "url")?;
            let url = Url::parse(&raw).ok()?;
            if url.scheme() != "https"
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || url.query().is_some()
                || url.fragment().is_some()
            {
                return None;
            }
            ("url", url.to_string())
        }
        _ => return None,
    };
    let mut fields = Map::from_iter([("source", Value::from(kind)), (field, Value::from(locator))]);
    // An unrecognized field may contain credentials or change how Claude resolves the source.
    for key in source.as_object()?.keys() {
        match key {
            "source" => {}
            key if key == field => {}
            "ref" if kind != "url" => {
                fields.insert(key.to_owned(), Value::from(text(source, key)?));
            }
            _ => return None,
        }
    }
    let mut result = Map::from_iter([("source", Value::Object(fields))]);
    if let Some(value) = entry.get("autoUpdate").and_then(Value::as_bool) {
        result.insert("autoUpdate".into(), Value::from(value));
    }
    Some(Value::Object(result))
}

fn declared(source: &str) -> Result<BTreeMap<String, Option<Value>>> {
    Ok(jsonc::parse(source, &[MARKETPLACES])?
        .into_iter()
        .flat_map(|object| object.entries(source))
        .map(|(name, entry)| {
            // Keep declarations as occupied even if their source is not portable.
            (name, portable_declaration(entry))
        })
        .collect())
}

fn portable_declaration(entry: &str) -> Option<Value> {
    let source = jsonc::parse(entry, &["source"]).ok()??;
    let fields = source
        .entries(entry)
        .map(|(name, raw)| Some((name, nextjson::from_str::<Value>(raw).ok()?)))
        .collect::<Option<Map>>()?;
    let mut declaration = Map::from_iter([("source", Value::Object(fields))]);
    if let Some((name, raw)) = jsonc::parse(entry, &[])
        .ok()??
        .entries(entry)
        .find(|(name, _)| name == "autoUpdate")
        && let Ok(value) = nextjson::from_str::<bool>(raw)
    {
        declaration.insert(name, Value::from(value));
    }
    portable(&Value::Object(declaration))
}

fn marketplaces(known: &Value, source: &str) -> Result<BTreeMap<String, Option<Value>>> {
    let mut catalog = declared(source)?;
    for (name, entry) in known
        .as_object()
        .context("expected Claude Code marketplaces")?
        .iter()
    {
        catalog
            .entry(name.to_owned())
            .or_insert_with(|| portable(entry));
    }
    Ok(catalog)
}

fn marketplace_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}

fn export(known: &Value, source: &str) -> Result<Settings> {
    let marketplaces = marketplaces(known, source)?
        .into_iter()
        .filter(|(name, _)| name != OFFICIAL && marketplace_name(name))
        .filter_map(|(name, value)| value.map(|value| (name, value)))
        .collect::<BTreeMap<_, _>>();
    let enabled = enabled(source)?
        .into_iter()
        .filter(|(id, value)| {
            *value == "true"
                && valid_id(id)
                && id.split_once('@').is_some_and(|(_, market)| {
                    market == OFFICIAL || marketplaces.contains_key(market)
                })
        })
        .map(|(id, _)| id)
        .collect();
    Ok(Settings {
        marketplaces,
        enabled,
    })
}

fn merge(known: &Value, source: &str, incoming: Settings) -> Result<(String, ImportReport)> {
    let existing = marketplaces(known, source)?;
    let flags = enabled(source)?;
    let mut output = source.to_owned();
    let mut report = ImportReport::default();
    let mut eligible = BTreeSet::from([OFFICIAL.to_owned()]);
    for (name, entry) in incoming.marketplaces {
        ensure!(
            marketplace_name(&name) && name != OFFICIAL,
            "invalid shared Claude Code marketplace name"
        );
        let entry = portable(&entry)
            .context("shared marketplace needs a portable GitHub or HTTPS source")?;
        let action = if let Some(current) = existing.get(&name) {
            if current.as_ref().and_then(|value| value.get("source")) == entry.get("source") {
                eligible.insert(name.clone());
                "existing"
            } else {
                "conflict"
            }
        } else {
            output = patch_setting(&output, MARKETPLACES, &name, Some(entry))?;
            eligible.insert(name.clone());
            "add"
        };
        report.marketplaces.insert(name, action);
    }
    for id in incoming.enabled {
        ensure!(valid_id(&id), "invalid shared Claude Code plugin ID");
        let (_, market) = id.split_once('@').expect("validated plugin ID");
        let action = if flags.contains_key(&id) {
            "existing"
        } else if eligible.contains(market) {
            output = patch_setting(&output, ENABLED, &id, Some(Value::from(true)))?;
            "add"
        } else {
            "marketplace_unavailable"
        };
        report.plugins.insert(id, action);
    }
    Ok((output, report))
}

fn with_profile<T>(
    dir: &Path,
    lock: bool,
    action: impl FnOnce(&Path, Option<&str>, Value) -> Result<T>,
) -> Result<T> {
    let _guard = lock.then(|| native_settings::lock(dir)).transpose()?;
    let home = NativeAgent::Claude
        .home()
        .context("Claude Code profile is unavailable")?;
    let settings = native_settings::path(&home.join("settings.json"))?;
    let source = native_settings::read(&settings)?;
    let known = read_object(&home.join("plugins/known_marketplaces.json"))?;
    action(&settings, source.as_deref(), known)
}

pub fn export_settings(dir: &Path) -> Result<String> {
    with_profile(dir, true, |_, source, known| {
        encode(&export(&known, source.unwrap_or(EMPTY))?)
    })
}

pub fn import_settings_file(dir: &Path, path: &Path) -> Result<()> {
    let bytes = read_limited(path, LIMIT).context("read shared Claude Code plugin settings")?;
    import_settings(dir, std::str::from_utf8(&bytes)?)
}

pub fn import_settings(dir: &Path, text: &str) -> Result<()> {
    ensure!(
        text.len() <= LIMIT,
        "shared Claude Code settings exceed 1 MiB"
    );
    let incoming =
        nextjson::from_str(text.trim_start_matches('\u{feff}')).map_err(anyhow::Error::msg)?;
    merge_profile(dir, incoming, true)?;
    Ok(())
}

/// Restore portable marketplace sources and enabled plugin declarations from a Magpie backup.
pub fn import_magpie(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    apply: bool,
) -> Result<ImportReport> {
    let backup = open_magpie_backup(file, passphrase)?;
    let declarations = backup
        .get("library")
        .and_then(|library| library.get("claudePlugins"))
        .filter(|value| !value.is_null())
        .context("Magpie backup has no Claude Code plugin settings")?;
    let incoming = nextjson::from_value(declarations.clone())
        .map_err(|_| anyhow::anyhow!("invalid Magpie Claude Code plugin settings"))?;
    ensure!(
        encode(&incoming)?.len() <= LIMIT,
        "shared Claude Code settings exceed 1 MiB"
    );
    merge_profile(dir, incoming, apply)
}

fn merge_profile(dir: &Path, incoming: Settings, apply: bool) -> Result<ImportReport> {
    with_profile(dir, apply, |path, source, known| {
        let before = source.unwrap_or(EMPTY);
        let (after, mut report) = merge(&known, before, incoming)?;
        if apply && after != before {
            native_settings::write(path, source, Some(&after))?;
        }
        report.applied = apply;
        Ok(report)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carries_portable_sources_and_preserves_local_choices() -> Result<()> {
        let source = r#"{
            "extraKnownMarketplaces": {
                "alpha": {"source": {/* keep */ "source": "github", "repo": "org/alpha",}, "autoUpdate":true},
                "local": {"source": {"source": "directory", "path": "C:/local"}},
                "private": {"source": {"source": "url", "url": "https://example.com/catalog?token=secret"}}
            },
            "enabledPlugins": {"a@alpha":true,"b@alpha":true,"c@local":true,"d@private":true,"e@claude-plugins-official":true}
        }"#;
        let known = Value::Object(Map::new());
        let outgoing = export(&known, source)?;
        assert_eq!(
            outgoing.marketplaces.keys().collect::<Vec<_>>(),
            [&"alpha".to_owned()]
        );
        assert_eq!(
            outgoing.enabled,
            ["a@alpha", "b@alpha", "e@claude-plugins-official"]
                .map(str::to_owned)
                .into()
        );
        let target =
            "\u{feff}{\r\n  // keep\r\n  \"enabledPlugins\": {\"a@alpha\":false},\r\n}\r\n";
        let (after, report) = merge(&known, target, outgoing)?;
        assert!(after.contains("// keep\r\n") && after.starts_with('\u{feff}'));
        let flags = enabled(&after)?;
        assert_eq!(flags.get("a@alpha"), Some(&"false"));
        assert_eq!(flags.get("b@alpha"), Some(&"true"));
        assert_eq!(flags.get("e@claude-plugins-official"), Some(&"true"));
        assert_eq!(report.plugins.get("a@alpha"), Some(&"existing"));
        let target = r#"{"extraKnownMarketplaces":{"alpha":{"source":{"source":"directory","path":"C:/local"}}}}"#;
        let (after, report) = merge(&known, target, export(&known, source)?)?;
        assert!(!enabled(&after)?.contains_key("a@alpha"));
        assert!(after.contains("C:/local"));
        assert_eq!(report.marketplaces.get("alpha"), Some(&"conflict"));
        assert_eq!(
            report.plugins.get("a@alpha"),
            Some(&"marketplace_unavailable")
        );
        Ok(())
    }
}
