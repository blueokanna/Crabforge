use super::{configuration, roles::ROLES, scope, source};
use crate::agents::{omp_catalog, omp_profiles};
use anyhow::{Context, Result, ensure};
use serde_yaml::{Mapping, Value};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::Path,
};
use yaml_edit::{YamlFile, YamlNode};

pub(crate) fn rename_provider(dir: &Path, from: &str, to: &str) -> Result<()> {
    let profiles = omp_profiles::installed().and_then(|cmd| omp_profiles::list(dir, &cmd));
    let homes = match profiles {
        Ok(profiles) => profiles.into_iter().map(|p| p.directory).collect(),
        Err(e)
            if e.downcast_ref::<std::io::Error>()
                .is_some_and(|io| io.kind() == ErrorKind::NotFound) =>
        {
            vec![dir.join("agents/omp")]
        }
        Err(e) => return Err(e),
    };
    for home in homes.into_iter().filter(|p| p.is_dir()) {
        let _lock = omp_profiles::lock(&home)?;
        rename_profile(&home, from, to)
            .with_context(|| format!("updating {} after provider rename", home.display()))?;
    }
    Ok(())
}

fn rename_profile(home: &Path, from: &str, to: &str) -> Result<()> {
    let prefix = format!("crabforge/{from}/");
    let rename = |value: &str| {
        value
            .split(',')
            .map(|part| {
                let model = part.trim_start_matches([' ', '\t']);
                if let Some(rest) = model.strip_prefix(&prefix) {
                    format!("{}crabforge/{to}/{rest}", &part[..part.len() - model.len()])
                } else {
                    part.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    };
    let (path, text) = source(home)?;
    let mut expected = configuration(&text)?;
    scope::rename_history(home, expected.get("enabledModels"), rename)?;
    let yaml: YamlFile = text.parse()?;
    let doc = yaml.ensure_document();
    for (parent, field) in [
        (None, "modelRoles"),
        (Some("retry"), "fallbackChains"),
        (None, "enabledModels"),
        (Some("task"), "agentModelOverrides"),
    ] {
        let (value, node) = match parent {
            Some(parent) => (
                expected.get_mut(parent).and_then(|p| p.get_mut(field)),
                doc.get_mapping(parent).and_then(|p| p.get(field)),
            ),
            None => (expected.get_mut(field), doc.get(field)),
        };
        if let (Some(value), Some(node)) = (value, node) {
            edit_refs(value, node, &rename)?;
        }
    }
    if !text.is_empty() {
        super::persist(home, &path, &text, &yaml, &expected)?;
    }
    for role in ROLES {
        let backup = home.join(format!("crabforge-role-{role}.yml"));
        if !backup.try_exists()? {
            continue;
        }
        let text = fs::read_to_string(&backup)?;
        let mut expected = configuration(&text)?;
        let yaml: YamlFile = text.parse()?;
        if let (Some(value), Some(node)) = (
            expected.get_mut("previous"),
            yaml.ensure_document().get("previous"),
        ) {
            edit_refs(value, node, &rename)?;
        }
        let rendered = yaml.to_string();
        if rendered != text {
            ensure!(
                configuration(&rendered)? == expected,
                "could not preserve saved {role} role"
            );
            let mut tmp = tempfile::NamedTempFile::new_in(home)?;
            tmp.write_all(rendered.as_bytes())?;
            tmp.as_file().sync_all()?;
            ensure!(
                fs::read_to_string(&backup)? == text,
                "saved {role} role changed on disk"
            );
            tmp.persist(backup)?;
        }
    }
    if let Some(path) = omp_catalog::catalog_path(home) {
        let text = fs::read_to_string(&path)?;
        let mut catalog = configuration(&text)?;
        if let Some(models) = catalog
            .get_mut("providers")
            .and_then(|v| v.get_mut("crabforge"))
            .and_then(|p| p.get_mut("models"))
            .and_then(Value::as_sequence_mut)
        {
            let mut changed = false;
            for model in models {
                if let Some(id) = model.get_mut("id")
                    && let Some((provider, name)) = id.as_str().and_then(|m| m.split_once('/'))
                    && provider == from
                {
                    *id = format!("{to}/{name}").into();
                    changed = true;
                }
            }
            if changed {
                omp_catalog::persist_catalog(home, Some(&path), Some(&text), &catalog)?;
            }
        }
    }
    Ok(())
}

// Edit the lossless syntax tree and check it against the parsed YAML before saving.
fn edit_refs(value: &mut Value, node: YamlNode, rename: &impl Fn(&str) -> String) -> Result<()> {
    match (value, node) {
        (Value::String(original), YamlNode::Scalar(scalar)) => {
            let new = rename(original);
            if *original != new {
                scalar.set_value(&serde_json::to_string(&new)?);
                *original = new;
            }
        }
        (Value::Sequence(values), YamlNode::Sequence(sequence)) => {
            for (value, node) in values.iter_mut().zip(sequence.values().collect::<Vec<_>>()) {
                edit_refs(value, node, rename)?;
            }
        }
        (Value::Mapping(values), YamlNode::Mapping(mapping)) => {
            let mut updated = Mapping::new();
            for (mut key, mut value) in std::mem::take(values) {
                if let Some(name) = key.as_str() {
                    if name != "path"
                        && let Some(node) = mapping.get(name)
                    {
                        edit_refs(&mut value, node, rename)?;
                    }
                    let new = rename(name);
                    if new != name {
                        ensure!(
                            mapping.get(&new).is_none(),
                            "duplicate model key after rename: {new}"
                        );
                        ensure!(mapping.rename_key(name, &new), "could not rename model key");
                        key = new.into();
                    }
                }
                updated.insert(key, value);
            }
            *values = updated;
        }
        _ => (),
    }
    Ok(())
}
