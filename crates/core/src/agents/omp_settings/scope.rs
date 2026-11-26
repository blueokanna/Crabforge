use super::{configuration, persist, source, thinking::single_model};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{
    collections::HashSet,
    fs,
    io::{ErrorKind, Write},
    path::Path,
};
use yaml_edit::{Sequence, YamlFile};

#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
struct Remembered {
    entries: Vec<String>,
    emptied: bool,
}

// Keep enough information to resume if a process exits between the YAML and history writes.
#[derive(Deserialize, Serialize)]
struct History {
    saved: Remembered,
    pending: Option<Pending>,
}

#[derive(Deserialize, Serialize)]
struct Pending {
    before: Option<Value>,
    after: Remembered,
}

fn write_history(path: &Path, saved: &Remembered, pending: Option<Pending>) -> Result<()> {
    if saved.entries.is_empty() && pending.is_none() {
        return match fs::remove_file(path) {
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            result => Ok(result?),
        };
    }
    let mut tmp =
        tempfile::NamedTempFile::new_in(path.parent().context("missing profile directory")?)?;
    serde_json::to_writer(
        &mut tmp,
        &History {
            saved: saved.clone(),
            pending,
        },
    )?;
    tmp.flush()?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;
    Ok(())
}

fn direct_model(entry: &str) -> Option<String> {
    let model = single_model(entry.trim())?.model.to_lowercase();
    (model.starts_with("crabforge/") && !model.contains(['*', '?', '['])).then_some(model)
}

pub(super) fn rename_history(
    home: &Path,
    before: Option<&Value>,
    rename: impl Fn(&str) -> String,
) -> Result<()> {
    let path = home.join("crabforge-scope-history.json");
    let file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    let History { saved, pending } = serde_json::from_reader(file)?;
    let recovering = pending.is_some();
    let mut saved = pending
        .filter(|p| p.before.as_ref() != before)
        .map_or(saved, |p| p.after);
    let renamed: Vec<_> = saved.entries.iter().map(|entry| rename(entry)).collect();
    if recovering || renamed != saved.entries {
        saved.entries = renamed;
        write_history(&path, &saved, None)?;
    }
    Ok(())
}

fn scope_takes(pattern: &str, model: &str) -> bool {
    if pattern.eq_ignore_ascii_case(model) {
        return true;
    }
    if let Some(base) = pattern.strip_suffix("**")
        && (base.is_empty() || base.ends_with('/'))
        && !base.contains(['*', '?', '['])
    {
        return model
            .get(..base.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(base));
    }
    if !pattern.contains(['*', '?', '[']) {
        return false;
    }
    let Ok(glob) = glob::Pattern::new(pattern) else {
        return false;
    };
    let options = glob::MatchOptions {
        case_sensitive: false,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };
    glob.matches_with(model, options)
        || model
            .split_once('/')
            .is_some_and(|(_, id)| glob.matches_with(id, options))
}

/// An existing global list must include a newly selected default model. Folder scopes stay local.
pub(super) fn include_default(
    root: &mut serde_yaml::Mapping,
    doc: &yaml_edit::Document,
    model: &str,
) -> Result<()> {
    let Some(items) = root
        .get_mut("enabledModels")
        .and_then(Value::as_sequence_mut)
    else {
        return Ok(());
    };
    let plain: Vec<_> = items.iter().filter_map(Value::as_str).collect();
    if plain.is_empty() {
        return Ok(());
    }
    let gateway = model.to_ascii_lowercase().starts_with("crabforge/");
    let mut covered = false;
    let mut custom = false;
    for name in plain {
        let Some(role) = single_model(name.trim()) else {
            continue;
        };
        let matched = scope_takes(&role.model, model);
        let glob = role.model.contains(['*', '?', '[']);
        covered |= matched;
        custom |= (matched && glob)
            || (role.model.to_ascii_lowercase().starts_with("crabforge/")
                && (glob || role.level.is_some()));
    }
    let seq = doc
        .get_sequence("enabledModels")
        .context("cannot edit enabledModels")?;
    let addition = if gateway && !custom {
        for i in (0..items.len()).rev() {
            if items[i].as_str().and_then(direct_model).is_some() {
                items.remove(i);
                seq.remove(i);
            }
        }
        "crabforge/**"
    } else if covered {
        return Ok(());
    } else {
        model
    };
    items.push(addition.into());
    seq.push(addition);
    Ok(())
}

/// Called under the same profile lock as the model catalog write.
pub(super) fn without_gateway(
    root: &mut serde_yaml::Mapping,
    doc: &yaml_edit::Document,
) -> Result<()> {
    let Some(items) = root
        .get_mut("enabledModels")
        .and_then(Value::as_sequence_mut)
    else {
        return Ok(());
    };
    let removed: Vec<_> = items
        .iter()
        .enumerate()
        .filter_map(|(i, v)| {
            let value = v.as_str()?.trim().to_ascii_lowercase();
            (value == "crabforge/**"
                || value.starts_with("crabforge/") && !value.contains(['*', '?', '[']))
            .then_some(i)
        })
        .collect();
    if removed.is_empty() {
        return Ok(());
    }
    if removed.len() == items.len() {
        root.remove("enabledModels");
        doc.remove("enabledModels");
    } else {
        let seq = doc
            .get_sequence("enabledModels")
            .context("cannot edit enabledModels")?;
        for idx in removed.into_iter().rev() {
            items.remove(idx);
            seq.remove(idx);
        }
    }
    Ok(())
}

/// Called under the same profile lock as the model catalog write.
pub(crate) fn sync_scope(home: &Path, ids: &[String]) -> Result<()> {
    let offered: HashSet<_> = ids
        .iter()
        .filter(|id| !id.trim().is_empty())
        .map(|id| format!("crabforge/{}", id.to_lowercase()))
        .collect();
    if offered.is_empty() {
        return Ok(());
    }
    let (path, text) = source(home)?;
    let mut expected = configuration(&text)?;
    let before = expected.get("enabledModels").cloned();
    let history_file = home.join("crabforge-scope-history.json");
    let saved = match fs::File::open(&history_file) {
        Ok(f) => {
            let History { saved, pending } =
                serde_json::from_reader(f).context("invalid Crabforge scope history")?;
            if let Some(pending) = pending {
                let recovered = if pending.before == before {
                    saved
                } else {
                    pending.after
                };
                write_history(&history_file, &recovered, None)?;
                recovered
            } else {
                saved
            }
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Remembered::default(),
        Err(err) => return Err(err.into()),
    };
    let list = match before.as_ref() {
        Some(Value::Sequence(items)) => items.as_slice(),
        Some(_) => return Ok(()),
        None if saved.emptied => &[],
        None => {
            if !saved.entries.is_empty() {
                write_history(&history_file, &Remembered::default(), None)?;
            }
            return Ok(());
        }
    };
    if !saved.emptied && !list.iter().any(|v| v.as_str().is_some()) {
        return Ok(());
    }
    let mut after = Remembered::default();
    let mut present = HashSet::new();
    let mut next = Vec::new();
    let mut removed = Vec::new();
    for (idx, value) in list.iter().enumerate() {
        if let Some(entry) = value.as_str()
            && let Some(model) = direct_model(entry)
        {
            if !offered.contains(&model) {
                let entry = entry.to_owned();
                if !after.entries.contains(&entry) {
                    after.entries.push(entry);
                }
                removed.push(idx);
                continue;
            }
            present.insert(model);
        }
        next.push(value.clone());
    }
    let mut restored = Vec::new();
    for entry in &saved.entries {
        let Some(model) = direct_model(entry) else {
            continue;
        };
        if offered.contains(&model) {
            if present.insert(model) {
                restored.push(entry.as_str());
                next.push(entry.as_str().into());
            }
        } else if !after.entries.contains(entry) {
            after.entries.push(entry.clone());
        }
    }
    after.emptied = !after.entries.is_empty() && !next.iter().any(|v| v.as_str().is_some());
    if removed.is_empty() && restored.is_empty() {
        if saved != after {
            write_history(&history_file, &after, None)?;
        }
        return Ok(());
    }
    let yaml: YamlFile = text.parse()?;
    let doc = yaml.ensure_document();
    if next.is_empty() {
        expected.remove("enabledModels");
        doc.remove("enabledModels");
    } else {
        if before.is_none() {
            doc.set("enabledModels", Sequence::new());
        }
        let sequence = doc
            .get_sequence("enabledModels")
            .context("cannot edit enabledModels in this YAML document")?;
        for idx in removed.into_iter().rev() {
            sequence.remove(idx);
        }
        for value in restored {
            sequence.push(value);
        }
        expected.insert("enabledModels".into(), Value::Sequence(next));
    }
    ensure!(
        configuration(&yaml.to_string())? == expected,
        "edited YAML did not preserve the model scope"
    );
    write_history(
        &history_file,
        &saved,
        Some(Pending {
            before,
            after: after.clone(),
        }),
    )?;
    persist(home, &path, &text, &yaml, &expected)?;
    write_history(&history_file, &after, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removed_models_return_after_catalog_refresh_and_manual_removal_wins() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let home = dir.path();
        let path = home.join("config.yml");
        let both = ["kept".into(), "vendor/m:8b".into()];
        let folder = "  - {path: project, models: [crabforge/vendor/m:8b]} # personal\n";
        let original = format!(
            "# profile\nenabledModels:\n  - CRABFORGE/vendor/m:8b:high\n{folder}  - other/x\n  - crabforge/**\n"
        );
        fs::write(&path, &original)?;
        sync_scope(home, &both[..1])?;
        let pruned = fs::read_to_string(&path)?;
        assert!(!pruned.contains("CRABFORGE/vendor/m:8b:high"));
        assert!(pruned.contains(folder) && pruned.contains("crabforge/**"));
        sync_scope(home, &both)?;
        let restored = fs::read_to_string(&path)?;
        assert_eq!(
            configuration(&restored)?["enabledModels"]
                .as_sequence()
                .unwrap()
                .last()
                .unwrap(),
            "CRABFORGE/vendor/m:8b:high"
        );
        assert!(restored.contains(folder));
        sync_scope(home, &both[..1])?;
        fs::write(&path, "# I removed my list\n")?;
        sync_scope(home, &both)?;
        assert_eq!(fs::read_to_string(&path)?, "# I removed my list\n");
        fs::write(&path, "enabledModels: [crabforge/vendor/m:8b:off]\n")?;
        sync_scope(home, &both[..1])?;
        assert!(
            configuration(&fs::read_to_string(&path)?)?
                .get("enabledModels")
                .is_none()
        );
        sync_scope(home, &both)?;
        assert_eq!(
            configuration(&fs::read_to_string(&path)?)?["enabledModels"][0],
            "crabforge/vendor/m:8b:off"
        );
        assert!(!home.join("crabforge-scope-history.json").exists());
        Ok(())
    }
}
