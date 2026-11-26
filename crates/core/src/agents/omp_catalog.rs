use crate::gateway::NativeProtocols;
use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use super::omp_profiles as profiles;
mod protocol;

pub fn installed_version(command: &Command) -> Option<semver::Version> {
    let output = Command::new(command.get_program())
        .args(command.get_args())
        .env("OMP_PROFILE", "")
        .env_remove("PI_PROFILE")
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .split_ascii_whitespace()
        .find_map(|s| {
            semver::Version::parse(s.trim_start_matches("omp/").trim_start_matches('v')).ok()
        })
}

fn sync_user_agent(provider: &mut Mapping, version: Option<&semver::Version>) -> Result<()> {
    let headers = provider
        .entry("headers".into())
        .or_insert_with(|| Value::Mapping(Mapping::new()))
        .as_mapping_mut()
        .context("invalid omp provider headers")?;
    let mut customized = false;
    headers.retain(|key, value| {
        if !key
            .as_str()
            .is_some_and(|k| k.eq_ignore_ascii_case("user-agent"))
        {
            return true;
        }
        let ours = value.as_str().is_some_and(|v| {
            v == "omp"
                || v.strip_prefix("omp/")
                    .is_some_and(|s| semver::Version::parse(s.trim_start_matches('v')).is_ok())
        });
        customized |= !ours;
        !ours
    });
    if !customized && version.is_none_or(|v| *v < semver::Version::new(18, 1, 6)) {
        let ua = version.map_or_else(|| "omp".to_owned(), |v| format!("omp/{v}"));
        headers.insert("User-Agent".into(), ua.into());
    }
    if headers.is_empty() {
        provider.remove("headers");
    }
    Ok(())
}

pub fn sync(dir: &Path, selected: Option<&str>) -> Result<usize> {
    configure(dir, selected, None)
}

pub fn read_protocol(dir: &Path, selected: Option<&str>) -> Result<String> {
    let home = profiles::settings_directory(dir, selected)?;
    let _lock = profiles::lock(&home)?;
    let preference = protocol::Preference::load(&home)?;
    Ok(preference
        .effective(protocol::catalog_api(&home)?.as_deref())
        .to_owned())
}

pub fn set_protocol(
    dir: &Path,
    selected: Option<&str>,
    choice: &str,
    previous: Option<&str>,
) -> Result<()> {
    configure(dir, selected, Some((choice, previous)))?;
    Ok(())
}

fn configure(
    dir: &Path,
    selected: Option<&str>,
    change: Option<(&str, Option<&str>)>,
) -> Result<usize> {
    let cfg = crate::config::load(dir)?;
    anyhow::ensure!(
        cfg.listen.port() != 0,
        "configure a fixed gateway port before syncing Oh My Pi"
    );
    let mut command = profiles::installed()?;
    let home = profiles::agent_directory(dir, &mut command, selected)?;
    let groups = cfg.resolved_groups();
    let hidden = cfg.unlisted_model_ids(&groups);
    let mut ids = cfg.agent_model_ids(&groups);
    ids.retain(|id| !hidden.contains(id));
    let count = ids.len();
    let _lock = profiles::lock(&home)?;
    write_provider_with_protocol(
        &home,
        &ids,
        &format!("http://{}/agent/omp", cfg.listen),
        &crate::gateway::native_protocols(&cfg, &groups)?,
        installed_version(&command).as_ref(),
        change,
    )?;
    super::omp_settings::sync_scope(&home, &ids)?;
    Ok(count)
}

pub fn profile(
    home: &Path,
    ids: &[String],
    root: &str,
    protocols: &BTreeMap<String, NativeProtocols>,
    version: Option<&semver::Version>,
) -> Result<()> {
    let _lock = profiles::lock(home)?;
    write_provider(home, ids, root, protocols, version)?;
    super::omp_settings::sync_scope(home, ids)
}

pub(super) fn catalog_path(home: &Path) -> Option<PathBuf> {
    // omp reads these in priority order and can migrate the older JSON file.
    ["models.yml", "models.yaml", "models.json"]
        .iter()
        .map(|name| home.join(name))
        .find(|path| path.is_file())
}

pub(super) fn write_provider(
    home: &Path,
    ids: &[String],
    root: &str,
    protocols: &BTreeMap<String, NativeProtocols>,
    version: Option<&semver::Version>,
) -> Result<()> {
    write_provider_with_protocol(home, ids, root, protocols, version, None)
}

fn write_provider_with_protocol(
    home: &Path,
    ids: &[String],
    root: &str,
    protocols: &BTreeMap<String, NativeProtocols>,
    version: Option<&semver::Version>,
    change: Option<(&str, Option<&str>)>,
) -> Result<()> {
    let path = catalog_path(home);
    let source = path.as_ref().map(fs::read_to_string).transpose()?;
    let mut doc: Mapping =
        serde_yaml::from_str::<Option<Mapping>>(source.as_deref().unwrap_or_default())?
            .unwrap_or_default();
    let providers = doc
        .entry("providers".into())
        .or_insert_with(|| Value::Mapping(Mapping::new()))
        .as_mapping_mut()
        .context("invalid omp providers")?;
    let provider = providers
        .entry("crabforge".into())
        .or_insert_with(|| Value::Mapping(Mapping::new()))
        .as_mapping_mut()
        .context("invalid omp crabforge provider")?;
    let original = provider.clone();
    let mut preference = protocol::Preference::load(home)?;
    let file_api = provider
        .get("api")
        .map(|v| v.as_str().context("invalid omp provider API"))
        .transpose()?;
    if let Some((choice, previous)) = change {
        anyhow::ensure!(
            previous.is_none_or(|v| v == preference.effective(file_api)),
            "Oh My Pi protocol changed on disk; reopen the profile"
        );
        preference.select(choice, file_api)?;
    }
    let selection = preference.effective(file_api);
    let automatic = selection == protocol::AUTO;
    let api = if automatic {
        "openai-completions"
    } else {
        selection
    };
    let base = match api {
        "openai-completions" | "openai-responses" => format!("{root}/v1"),
        "anthropic-messages" => root.to_owned(),
        "google-generative-ai" => format!("{root}/v1beta"),
        other => anyhow::bail!("unsupported gateway protocol for omp: {other}"),
    };
    let models: Vec<_> = ids
        .iter()
        .map(|id| {
            super::pi_model(
                id,
                root,
                api,
                protocols.get(id).filter(|_| automatic),
                "omp",
            )
        })
        .collect();
    let models = serde_yaml::from_str(&nextjson::to_string(&models).map_err(anyhow::Error::msg)?)?;
    provider.insert("api".into(), api.into());
    provider.remove("auth");
    provider.insert("baseUrl".into(), base.into());
    provider.insert("apiKey".into(), "CRABFORGE_GATEWAY_KEY".into());
    provider.insert("models".into(), models);
    sync_user_agent(provider, version)?;
    if version.is_some_and(|v| *v >= semver::Version::new(16, 0, 6)) {
        provider
            .entry("compat".into())
            .or_insert_with(|| Value::Mapping(Mapping::new()))
            .as_mapping_mut()
            .context("invalid omp provider compat settings")?
            .entry("promptCacheSessionHeader".into())
            .or_insert_with(|| "x-session-affinity".into());
    } else if let Some(compat) = provider.get_mut("compat").and_then(Value::as_mapping_mut)
        && compat
            .get("promptCacheSessionHeader")
            .and_then(Value::as_str)
            == Some("x-session-affinity")
    {
        compat.remove("promptCacheSessionHeader");
    }
    if change.is_some() {
        preference.persist(home)?;
    }
    if *provider != original {
        persist_catalog(home, path.as_deref(), source.as_deref(), &doc)?;
    }
    preference.applied(home)
}

pub(super) fn drop_provider(home: &Path) -> Result<()> {
    let Some(path) = catalog_path(home) else {
        return Ok(());
    };
    let original = fs::read_to_string(&path)?;
    let mut expected: Mapping = serde_yaml::from_str(&original)?;
    let Some(providers) = expected
        .get_mut("providers")
        .and_then(Value::as_mapping_mut)
    else {
        return Ok(());
    };
    if providers.remove("crabforge").is_none() {
        return Ok(());
    }
    persist_catalog(home, Some(&path), Some(&original), &expected)
}

pub(super) fn persist_catalog(
    home: &Path,
    path: Option<&Path>,
    source: Option<&str>,
    expected: &Mapping,
) -> Result<()> {
    let legacy = path.and_then(Path::extension).is_some_and(|s| s == "json");
    let rendered = if legacy || source.is_none() {
        serde_yaml::to_string(expected)?
    } else {
        let yaml: yaml_edit::YamlFile = source.unwrap_or_default().parse()?;
        let doc = yaml.ensure_document();
        if doc.get("providers").is_none() {
            doc.set("providers", yaml_edit::Mapping::new());
        }
        let providers = doc
            .get_mapping("providers")
            .context("cannot edit providers")?;
        if let Some(provider) = expected["providers"].get("crabforge") {
            let fragment = if providers.is_flow_style() {
                nextjson::to_string(&super::omp_settings::snapshot::json_node(provider)?)
                    .map_err(anyhow::Error::msg)?
            } else {
                serde_yaml::to_string(provider)?
            };
            let node: yaml_edit::YamlFile = fragment.parse()?;
            providers.set(
                "crabforge",
                node.document()
                    .and_then(|d| d.as_mapping())
                    .context("cannot render Oh My Pi provider")?,
            );
        } else {
            providers.remove("crabforge");
        }
        yaml.to_string()
    };
    anyhow::ensure!(
        &serde_yaml::from_str::<Mapping>(&rendered)? == expected,
        "edited YAML did not preserve the model catalog"
    );
    let mut tmp = tempfile::NamedTempFile::new_in(home)?;
    tmp.write_all(rendered.as_bytes())?;
    tmp.as_file().sync_all()?;
    anyhow::ensure!(
        catalog_path(home).as_deref() == path
            && path.map(fs::read_to_string).transpose()?.as_deref() == source,
        "model catalog changed while saving"
    );
    if let Some(path) = path.filter(|_| !legacy) {
        tmp.persist(path)?;
    } else {
        tmp.persist_noclobber(home.join("models.yml"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_picks_override_hand_edits_and_restore_native_model_apis() -> Result<()> {
        use crate::{config::Config, gateway::native_protocols};
        let dir = tempfile::tempdir()?;
        let home = dir.path();
        let cfg = Config::parse(
            r#"{"providers":[{"id":"local","models":["m"],"endpoints":{"responses":"http://127.0.0.1:1/v1"}}]}"#,
        )?;
        let protocols = native_protocols(&cfg, &cfg.resolved_groups())?;
        let sync = || {
            write_provider(
                home,
                &["local/m".to_owned()],
                "http://127.0.0.1:3425/agent/omp",
                &protocols,
                None,
            )
        };
        let path = home.join("models.yaml");
        fs::write(
            &path,
            "providers:\n  crabforge: {api: anthropic-messages}\n",
        )?;
        let current = || -> Result<Value> {
            Ok(serde_yaml::from_reader::<_,Value>(fs::File::open(&path)?)?["providers"]["crabforge"].clone())
        };
        sync()?;
        assert_eq!(current()?["api"], "anthropic-messages");
        for choice in ["openai-completions", "auto"] {
            let mut preference = protocol::Preference::load(home)?;
            preference.select(choice, protocol::catalog_api(home)?.as_deref())?;
            preference.persist(home)?;
            // A fresh process can finish the write after the preference was recorded.
            sync()?;
            sync()?;
            let provider = current()?;
            assert_eq!(provider["api"], "openai-completions");
            assert_eq!(
                provider["models"][0].get("api").and_then(Value::as_str),
                (choice == "auto").then_some("openai-responses")
            );
        }
        Ok(())
    }

    #[test]
    fn catalog_updates_keep_unrelated_yaml_and_preserve_legacy_json() -> Result<()> {
        for (file, text) in [
            (
                "models.yml",
                "# mine\nproviders:\n  personal: {models: []} # keep\n",
            ),
            (
                "models.yaml",
                "providers: {personal: {models: []}} # keep\n",
            ),
            ("models.json", r#"{"providers":{"personal":{"models":[]}}}"#),
        ] {
            let dir = tempfile::tempdir()?;
            let home = dir.path();
            let path = home.join(file);
            fs::write(&path, text)?;
            write_provider(
                home,
                &["lab/m".to_owned()],
                "http://127.0.0.1:1234/agent/omp",
                &BTreeMap::new(),
                None,
            )?;
            let dest = catalog_path(home).unwrap();
            let after = fs::read_to_string(&dest)?;
            assert!(after.contains("lab/m"));
            if file.ends_with("json") {
                assert_eq!(fs::read_to_string(path)?, text);
            } else {
                assert!(after.contains("# keep"));
            }
            drop_provider(home)?;
            let cleared: Value = serde_yaml::from_reader(fs::File::open(dest)?)?;
            assert_eq!(cleared, serde_yaml::from_str::<Value>(text)?);
        }
        Ok(())
    }

    #[test]
    fn user_agent_follows_client_upgrades_and_preserves_custom_headers() -> Result<()> {
        let mut provider: Mapping =
            serde_yaml::from_str("headers:\n  X-Route: test\n  User-Agent: omp\n")?;
        for (version, agent) in [
            (Some("18.1.5"), Some("omp/18.1.5")),
            (Some("18.1.6"), None),
            (None, Some("omp")),
        ] {
            let version = version.map(semver::Version::parse).transpose()?;
            sync_user_agent(&mut provider, version.as_ref())?;
            assert_eq!(provider["headers"]["User-Agent"].as_str(), agent);
            assert_eq!(provider["headers"]["X-Route"].as_str(), Some("test"));
        }
        provider.insert(
            "headers".into(),
            serde_yaml::from_str("user-agent: my-relay/2")?,
        );
        sync_user_agent(&mut provider, None)?;
        assert_eq!(provider["headers"].as_mapping().unwrap().len(), 1);
        assert_eq!(
            provider["headers"]["user-agent"].as_str(),
            Some("my-relay/2")
        );
        Ok(())
    }
}
