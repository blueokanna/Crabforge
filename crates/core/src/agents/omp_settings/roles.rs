use super::{configuration, persist, scope, source, thinking::single_model};
use crate::{
    agents::{omp_catalog, omp_profiles},
    config::Config,
};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use serde_yaml::{Mapping, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{ErrorKind, Write},
    path::Path,
};
use yaml_edit::YamlFile;

pub const ROLES: &[&str] = &[
    "default", "task", "smol", "slow", "plan", "vision", "advisor", "commit", "tiny", "memory",
];

#[derive(NsonDeserialize, NsonSerialize)]
pub struct Roles {
    pub assignments: BTreeMap<String, Option<nextjson::Value>>,
    pub models: Vec<String>,
    // Legacy editor snapshots used JSON null for both missing and explicit YAML null.
    explicit_nulls: Option<BTreeSet<String>>,
}

fn gateway_ids(cfg: &Config) -> Vec<String> {
    let groups = cfg.resolved_groups();
    let hidden = cfg.unlisted_model_ids(&groups);
    cfg.agent_model_ids(&groups)
        .into_iter()
        .filter(|id| !hidden.contains(id))
        .collect()
}

pub fn read_roles(home: &Path, cfg: &Config) -> Result<Roles> {
    let (_, text) = source(home)?;
    let root = configuration(&text)?;
    let roles = root
        .get("modelRoles")
        .map(|v| v.as_mapping().context("modelRoles must be a mapping"))
        .transpose()?;
    let assignments: BTreeMap<String, Option<nextjson::Value>> = ROLES
        .iter()
        .map(|&name| Ok((name.into(), assignment(roles.and_then(|r| r.get(name)))?)))
        .collect::<Result<_>>()?;
    let mut models: BTreeSet<_> = gateway_ids(cfg)
        .into_iter()
        .map(|id| format!("crabforge/{id}"))
        .collect();
    if let Some(path) = omp_catalog::catalog_path(home) {
        let catalog: Value = serde_yaml::from_reader(fs::File::open(path)?)?;
        if let Some(providers) = catalog.get("providers").and_then(Value::as_mapping) {
            for (name, provider) in providers {
                let Some(name) = name.as_str().filter(|n| *n != "crabforge") else {
                    continue;
                };
                if let Some(entries) = provider.get("models").and_then(Value::as_sequence) {
                    models.extend(
                        entries
                            .iter()
                            .filter_map(|m| m.get("id")?.as_str().filter(|id| !id.is_empty()))
                            .map(|id| format!("{name}/{id}")),
                    );
                }
            }
        }
    }
    let mut providers = BTreeSet::new();
    for assigned in assignments.values().flatten() {
        let values = assigned
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(std::slice::from_ref(assigned));
        for entry in values
            .iter()
            .filter_map(nextjson::Value::as_str)
            .flat_map(|v| v.split(','))
        {
            let entry = entry.trim();
            if let Some((name, _)) = entry.split_once('/')
                && name != "crabforge"
            {
                if let Some(role) = single_model(entry) {
                    models.insert(role.model);
                }
                providers.insert(name);
            }
        }
    }
    super::registry::add_provider_models(providers, &mut models);
    Ok(Roles {
        assignments,
        models: models.into_iter().collect(),
        explicit_nulls: Some(
            ROLES
                .iter()
                .copied()
                .filter(|name| roles.and_then(|r| r.get(*name)).is_some_and(Value::is_null))
                .map(str::to_owned)
                .collect(),
        ),
    })
}

fn assignment(node: Option<&Value>) -> Result<Option<nextjson::Value>> {
    Ok(node
        .map(super::snapshot::json_node)
        .transpose()?
        .filter(|v| !v.is_null()))
}

fn remove_file(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        result => Ok(result?),
    }
}

pub(super) fn forget_advisor_enable(home: &Path) -> Result<()> {
    remove_file(&home.join("crabforge-advisor-enabled"))
}

fn on_gateway(value: &Value) -> bool {
    value
        .as_str()
        .and_then(single_model)
        .is_some_and(|v| v.model.starts_with("crabforge/"))
}

fn references_gateway(value: &Value) -> bool {
    match value {
        Value::String(s) => s.split(',').any(|s| s.trim().starts_with("crabforge/")),
        Value::Sequence(s) => s.iter().any(references_gateway),
        Value::Mapping(m) => m
            .iter()
            .any(|(k, v)| references_gateway(k) || references_gateway(v)),
        _ => false,
    }
}

fn needs_provider(root: &Mapping) -> bool {
    let referenced = [
        root.get("modelRoles"),
        root.get("retry").and_then(|v| v.get("fallbackChains")),
        root.get("task").and_then(|v| v.get("agentModelOverrides")),
    ];
    referenced.into_iter().flatten().any(references_gateway)
        || root
            .get("enabledModels")
            .and_then(Value::as_sequence)
            .is_some_and(|list| list.iter().any(|v| v.is_mapping() && references_gateway(v)))
}

// Save the original YAML node, including the notation used for a fallback list.
fn remember(
    home: &Path,
    path: &Path,
    roles: Option<yaml_edit::Mapping>,
    name: &str,
    current: Option<&Value>,
) -> Result<()> {
    let saved = yaml_edit::Mapping::new();
    if let Some(node) = roles.and_then(|r| r.get(name)) {
        saved.set("previous", node);
    }
    let text = saved.to_string();
    ensure!(
        configuration(&text)?.get("previous") == current,
        "could not preserve the previous role assignment"
    );
    let mut tmp = tempfile::NamedTempFile::new_in(home)?;
    tmp.write_all(text.as_bytes())?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;
    Ok(())
}

fn recalled(path: &Path) -> Result<Option<(Value, yaml_edit::YamlNode)>> {
    match fs::read_to_string(path) {
        Ok(text) => {
            let value = configuration(&text)?.remove("previous");
            let node = text
                .parse::<YamlFile>()?
                .document()
                .and_then(|d| d.get("previous"));
            ensure!(
                node.is_some() == value.is_some(),
                "cannot read saved role assignment"
            );
            Ok(value.zip(node))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Save one role while keeping the model catalog and its profile under the same lock.
pub fn save_role(
    home: &Path,
    cfg: &Config,
    name: &str,
    pick: Option<&str>,
    previous: Option<&Roles>,
) -> Result<()> {
    ensure!(ROLES.contains(&name), "unknown Oh My Pi role {name:?}");
    ensure!(
        pick.is_none_or(|p| !p.trim().is_empty() && !p.chars().any(char::is_control)),
        "invalid Oh My Pi model"
    );
    let _lock = omp_profiles::lock(home)?;
    let (path, text) = source(home)?;
    let mut expected = configuration(&text)?;
    let current = expected
        .get("modelRoles")
        .map(|v| v.as_mapping().context("modelRoles must be a mapping"))
        .transpose()?
        .and_then(|v| v.get(name))
        .cloned();
    if let Some(old) = previous {
        let explicit_null = current.as_ref().is_some_and(Value::is_null);
        let current = assignment(current.as_ref())?;
        ensure!(
            old.explicit_nulls
                .as_ref()
                .is_none_or(|nulls| nulls.contains(name) == explicit_null)
                && old
                    .assignments
                    .get(name)
                    .is_some_and(|v| super::snapshot::same(v.as_ref(), current.as_ref())),
            "{name} role changed on disk; reopen the profile"
        );
    }
    let yaml: YamlFile = text.parse()?;
    let doc = yaml.ensure_document();
    let backup = home.join(format!("crabforge-role-{name}.yml"));
    let was_gateway = current.as_ref().is_some_and(on_gateway);
    let restore = if pick.is_none() && was_gateway {
        recalled(&backup)?
    } else {
        None
    };
    let picked = pick.map(str::trim).map(|s| {
        let old = current
            .as_ref()
            .and_then(Value::as_str)
            .and_then(single_model);
        match (single_model(s), old) {
            (Some(new), Some(old))
                if new.level.is_none() && new.model != old.model && old.level.is_some() =>
            {
                format!("{s}:{}", old.level.unwrap())
            }
            _ => s.to_owned(),
        }
    });
    let value = match (&picked, &restore) {
        (Some(s), _) => Some(Value::String(s.clone())),
        (_, Some((value, _))) => Some(value.clone()),
        _ => None,
    };
    let ids = gateway_ids(cfg);
    if let Some(role) = picked.as_deref().and_then(single_model)
        && let Some(id) = role.model.strip_prefix("crabforge/")
    {
        ensure!(
            ids.iter().any(|i| i == id),
            "model is missing from the gateway configuration: {id}"
        );
    }
    let takeover = value.as_ref().is_some_and(on_gateway) && !was_gateway;
    if takeover {
        remember(
            home,
            &backup,
            doc.get_mapping("modelRoles"),
            name,
            current.as_ref(),
        )?;
    }
    if let Some(v) = &value {
        let section = expected
            .entry("modelRoles".into())
            .or_insert_with(|| Value::Mapping(Mapping::new()))
            .as_mapping_mut()
            .context("invalid modelRoles")?;
        section.insert(name.into(), v.clone());
        if doc.get_mapping("modelRoles").is_none() {
            doc.set("modelRoles", yaml_edit::Mapping::new());
        }
        let roles = doc
            .get_mapping("modelRoles")
            .context("cannot edit modelRoles")?;
        if let Some(model) = picked.as_deref() {
            roles.set(name, model);
        } else if let Some((_, node)) = restore {
            roles.set(name, node);
        }
    } else if let Some(roles) = expected
        .get_mut("modelRoles")
        .and_then(Value::as_mapping_mut)
    {
        roles.remove(name);
        if roles.is_empty() {
            expected.remove("modelRoles");
            doc.remove("modelRoles");
        } else {
            doc.get_mapping("modelRoles")
                .context("cannot edit modelRoles")?
                .remove(name);
        }
    }
    if name == "default"
        && let Some(role) = picked.as_deref().and_then(single_model)
    {
        scope::include_default(&mut expected, &doc, &role.model)?;
    }
    let marker = home.join("crabforge-advisor-enabled");
    let mut enabled_by_pick = false;
    if name == "advisor" {
        let enabled = expected.get("advisor").and_then(|v| v.get("enabled"));
        if picked.is_some() && enabled.is_none() {
            expected
                .entry("advisor".into())
                .or_insert_with(|| Value::Mapping(Mapping::new()))
                .as_mapping_mut()
                .context("invalid advisor settings")?
                .insert("enabled".into(), true.into());
            if doc.get_mapping("advisor").is_none() {
                doc.set("advisor", yaml_edit::Mapping::new());
            }
            doc.get_mapping("advisor")
                .context("cannot edit advisor")?
                .set("enabled", true);
            enabled_by_pick = true;
        } else if picked.is_none()
            && marker.exists()
            && enabled.and_then(Value::as_bool) == Some(true)
        {
            let advisor = expected
                .get_mut("advisor")
                .and_then(Value::as_mapping_mut)
                .context("invalid advisor settings")?;
            advisor.remove("enabled");
            if advisor.is_empty() {
                expected.remove("advisor");
                doc.remove("advisor");
            } else {
                doc.get_mapping("advisor")
                    .context("cannot edit advisor")?
                    .remove("enabled");
            }
        }
    }
    let used = needs_provider(&expected);
    if !used {
        scope::without_gateway(&mut expected, &doc)?;
    }
    ensure!(
        configuration(&yaml.to_string())? == expected,
        "edited YAML did not preserve the role settings"
    );
    if used {
        ensure!(
            cfg.listen.port() != 0,
            "configure a fixed gateway port before setting a gateway role"
        );
        let version = omp_profiles::installed()
            .ok()
            .and_then(|c| omp_catalog::installed_version(&c));
        omp_catalog::write_provider(
            home,
            &ids,
            &format!("http://{}/agent/omp", cfg.listen),
            &crate::gateway::native_protocols(cfg, &cfg.resolved_groups())?,
            version.as_ref(),
        )?;
    }
    persist(home, &path, &text, &yaml, &expected)?;
    if enabled_by_pick {
        fs::write(&marker, b"1")?;
    }
    if !used {
        omp_catalog::drop_provider(home)?;
        remove_file(&home.join("crabforge-scope-history.json"))?;
    }
    if !value.as_ref().is_some_and(on_gateway) {
        remove_file(&backup)?;
    }
    if name == "advisor" && picked.is_none() {
        forget_advisor_enable(home)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config::parse(r#"{"providers":[{"id":"local","models":["a", "qwen3:8b"],"endpoints":{"chat":"http://127.0.0.1:1/v1"}}]}"#).unwrap()
    }

    #[test]
    fn restores_flow_and_block_fallbacks_after_multiple_picks() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let home = dir.path();
        let cfg = config();
        let path = home.join("config.yaml");
        for node in [
            "[other/x, other/y:low]",
            "\n    - other/x\n    - other/y:low",
        ] {
            fs::write(
                &path,
                format!("# personal\nmodelRoles:\n  task: {node}\n  slow: other/qwen3:8b:high\n"),
            )?;
            let before = read_roles(home, &cfg)?;
            save_role(home, &cfg, "task", Some("crabforge/local/a"), Some(&before))?;
            assert!(save_role(home, &cfg, "task", None, Some(&before)).is_err());
            save_role(home, &cfg, "task", Some("crabforge/local/qwen3:8b"), None)?;
            save_role(home, &cfg, "task", None, None)?;
            let state = read_roles(home, &cfg)?;
            assert_eq!(before.assignments, state.assignments);
            let restored = fs::read_to_string(&path)?;
            assert!(restored.contains(node.trim_start()), "{restored}");
            assert!(restored.contains("# personal"));
            save_role(home, &cfg, "slow", Some("crabforge/local/qwen3:8b"), None)?;
            assert_eq!(
                read_roles(home, &cfg)?.assignments["slow"],
                Some("crabforge/local/qwen3:8b:high".into())
            );
            save_role(home, &cfg, "slow", None, None)?;
        }
        Ok(())
    }

    #[test]
    fn editor_snapshots_preserve_null_and_compare_nested_objects_by_key() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let home = dir.path();
        let path = home.join("config.yml");
        let cfg = config();
        let snapshot = || -> Result<Roles> {
            let json = nextjson::to_string(&read_roles(home, &cfg)?).map_err(anyhow::Error::msg)?;
            nextjson::from_str(&json).map_err(anyhow::Error::msg)
        };
        let missing = snapshot()?;
        fs::write(&path, "modelRoles: {task: null}\n")?;
        assert!(
            save_role(
                home,
                &cfg,
                "task",
                Some("crabforge/local/a"),
                Some(&missing)
            )
            .is_err()
        );
        let null = snapshot()?;
        save_role(home, &cfg, "task", Some("crabforge/local/a"), Some(&null))?;
        save_role(home, &cfg, "task", None, None)?;
        assert_eq!(
            configuration(&fs::read_to_string(&path)?)?["modelRoles"]["task"],
            Value::Null
        );
        fs::write(&path, "modelRoles: {task: {models: [a,b], priority: 1}}\n")?;
        let prior = snapshot()?;
        fs::write(&path, "modelRoles: {task: {priority: 1, models: [b,a]}}\n")?;
        assert!(save_role(home, &cfg, "task", Some("crabforge/local/a"), Some(&prior)).is_err());
        fs::write(&path, "modelRoles: {task: {priority: 1, models: [a,b]}}\n")?;
        save_role(home, &cfg, "task", Some("crabforge/local/a"), Some(&prior))?;
        save_role(home, &cfg, "task", None, None)?;
        assert_eq!(
            fs::read_to_string(&path)?,
            "modelRoles: {task: {priority: 1, models: [a,b]}}\n"
        );
        Ok(())
    }

    #[test]
    fn retaking_an_externally_edited_role_backs_up_the_new_assignment() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let home = dir.path();
        let cfg = config();
        let path = home.join("config.yml");
        fs::write(&path, "modelRoles: {task: first/model}\n")?;
        save_role(home, &cfg, "task", Some("crabforge/local/a"), None)?;
        let previous = read_roles(home, &cfg)?;
        let edited = "modelRoles: {task: [new/first, new/second:high]}\n";
        fs::write(&path, edited)?;
        assert!(
            save_role(
                home,
                &cfg,
                "task",
                Some("crabforge/local/a"),
                Some(&previous)
            )
            .is_err()
        );
        assert_eq!(fs::read_to_string(&path)?, edited);
        save_role(home, &cfg, "task", Some("crabforge/local/a"), None)?;
        save_role(home, &cfg, "task", None, None)?;
        assert_eq!(fs::read_to_string(path)?, edited);
        Ok(())
    }

    #[test]
    fn default_scope_and_advisor_follow_role_ownership() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let home = dir.path();
        let cfg = config();
        let path = home.join("config.yml");
        fs::write(
            &path,
            "enabledModels:\n  - other/x\n  - crabforge/old\n  - {path: p, models: [other/y]}\n",
        )?;
        save_role(home, &cfg, "default", Some("crabforge/local/a"), None)?;
        let root = configuration(&fs::read_to_string(&path)?)?;
        assert_eq!(root["enabledModels"][2], "crabforge/**");
        save_role(home, &cfg, "advisor", Some("crabforge/local/a"), None)?;
        assert_eq!(super::super::read_advisor(home)?.review, "turn");
        save_role(home, &cfg, "advisor", None, None)?;
        assert_eq!(super::super::read_advisor(home)?.review, "inherit");
        save_role(home, &cfg, "advisor", Some("other/y"), None)?;
        super::super::save_advisor(
            home,
            None,
            &super::super::AdvisorChanges {
                review: Some("agent-end".into()),
                ..Default::default()
            },
        )?;
        save_role(home, &cfg, "advisor", None, None)?;
        assert_eq!(super::super::read_advisor(home)?.review, "agent-end");
        save_role(home, &cfg, "default", None, None)?;
        let root = configuration(&fs::read_to_string(&path)?)?;
        assert!(root.get("modelRoles").is_none());
        assert_eq!(root["enabledModels"].as_sequence().unwrap().len(), 2);
        assert!(!fs::read_to_string(home.join("models.yml"))?.contains("crabforge"));
        for enabled in [true, false] {
            fs::write(
                &path,
                format!("advisor: {{enabled: {enabled}, reviewMode: agent-end}}\n"),
            )?;
            save_role(home, &cfg, "advisor", Some("crabforge/local/a"), None)?;
            save_role(home, &cfg, "advisor", None, None)?;
            let settings = configuration(&fs::read_to_string(&path)?)?;
            assert_eq!(settings["advisor"]["enabled"].as_bool(), Some(enabled));
            assert_eq!(settings["advisor"]["reviewMode"], "agent-end");
        }
        Ok(())
    }
}
