use anyhow::{Context, Result};
use crabforge_core::gateway::NativeProtocols;
use serde_yaml::{Mapping, Value};
use std::{collections::BTreeMap, fs, io::Write, path::Path, process::Command};

pub(super) mod profiles;

pub(super) fn supports_session_header(command: &Command) -> bool {
    let Ok(output) = Command::new(command.get_program())
        .args(command.get_args())
        .env("OMP_PROFILE", "")
        .env_remove("PI_PROFILE")
        .arg("--version")
        .output()
    else {
        return false;
    };
    let version = String::from_utf8_lossy(&output.stdout);
    let version = version.split_ascii_whitespace().find_map(|s| {
        semver::Version::parse(s.trim_start_matches("omp/").trim_start_matches('v')).ok()
    });
    output.status.success() && version.is_some_and(|v| v >= semver::Version::new(16, 0, 6))
}

pub(super) fn profile(
    home: &Path,
    ids: Vec<String>,
    root: &str,
    protocols: &BTreeMap<String, NativeProtocols>,
    supports_session: bool,
) -> Result<()> {
    fs::create_dir_all(home)?;
    let lock = fs::File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(home.join("crabforge.lock"))?;
    lock.lock()?;
    // omp reads these in priority order and can migrate the older JSON file.
    let path = ["models.yml", "models.yaml", "models.json"]
        .iter()
        .map(|name| home.join(name))
        .find(|path| path.is_file());
    let mut doc: Mapping = match path.as_deref() {
        Some(path) => serde_yaml::from_reader(fs::File::open(path)?)
            .with_context(|| format!("invalid {}", path.display()))?,
        None => Mapping::new(),
    };
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
    let api = provider
        .entry("api".into())
        .or_insert_with(|| "openai-completions".into())
        .as_str()
        .context("invalid omp provider API")?;
    let base = match api {
        "openai-completions" | "openai-responses" => format!("{root}/v1"),
        "anthropic-messages" => root.to_owned(),
        "google-generative-ai" => format!("{root}/v1beta"),
        other => anyhow::bail!("unsupported gateway protocol for omp: {other}"),
    };
    let models: Vec<_> = ids
        .into_iter()
        .map(|id| {
            let mut model = serde_json::json!({"id":id});
            if api == "openai-completions"
                && let Some(native) = protocols.get(&id)
            {
                let native = native.omp_api();
                if native != api {
                    model["api"] = native.into();
                    match native {
                        "anthropic-messages" => model["baseUrl"] = root.into(),
                        "google-generative-ai" => {
                            model["baseUrl"] = format!("{root}/v1beta").into()
                        }
                        _ => (),
                    }
                }
            }
            model
        })
        .collect();
    provider.remove("auth");
    provider.insert("baseUrl".into(), base.into());
    provider.insert("apiKey".into(), "CRABFORGE_GATEWAY_KEY".into());
    provider.insert("models".into(), serde_yaml::to_value(models)?);
    if supports_session {
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
    if *provider != original {
        let target = path
            .filter(|p| p.extension().is_none_or(|e| e != "json"))
            .unwrap_or_else(|| home.join("models.yml"));
        let mut tmp = tempfile::NamedTempFile::new_in(home)?;
        serde_yaml::to_writer(&mut tmp, &doc)?;
        tmp.flush()?;
        tmp.persist(target)?;
    }
    Ok(())
}
