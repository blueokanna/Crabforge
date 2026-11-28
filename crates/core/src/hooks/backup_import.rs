use super::{Agent, Hook, MAX_ENTRY, Store, encode};
use crate::{config::backup_library_section, native_settings};
use anyhow::{Context, Result, bail, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    fs,
    path::Path,
};

#[derive(NsonDeserialize)]
struct BackupHook {
    name: String,
    description: Option<String>,
    event: String,
    matcher: Option<String>,
    command: String,
    timeout: Option<u32>,
    agents: Option<Vec<String>>,
    bundle: Option<String>,
    #[njson(flatten)]
    other: BTreeMap<String, Value>,
}

#[derive(NsonSerialize)]
pub struct ImportReport {
    hooks: Vec<PlannedHook>,
    other_library_sections: Vec<String>,
    applied: bool,
}

#[derive(NsonSerialize)]
struct PlannedHook {
    name: String,
    event: String,
    agents: BTreeSet<Agent>,
    action: &'static str,
    skipped_fields: Vec<String>,
}

/// Save the selected definitions to the shared hook library for later publication.
pub fn import_backup(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    selected: &[String],
    apply: bool,
) -> Result<ImportReport> {
    let (entries, other_library_sections) = backup_library_section(file, passphrase, "hooks")?;
    let selected: BTreeSet<_> = selected.iter().map(String::as_str).collect();
    let mut hooks = BTreeMap::new();
    for entry in entries {
        let name = entry
            .pointer("/name")
            .and_then(Value::as_str)
            .context("missing backup hook name")?;
        if !selected.is_empty() && !selected.contains(name) {
            continue;
        }
        let imported: BackupHook = nextjson::from_value(entry)
            .map_err(|_| anyhow::anyhow!("invalid backup hook definition"))?;
        ensure!(
            imported.bundle.as_deref().is_none_or(str::is_empty),
            "backup hook {:?} belongs to a plugin",
            imported.name
        );
        let agents = imported
            .agents
            .unwrap_or_default()
            .iter()
            .map(|agent| Agent::parse(agent))
            .collect::<Result<_>>()
            .with_context(|| format!("invalid assignments for backup hook {:?}", imported.name))?;
        let hook = Hook {
            name: imported.name,
            description: imported.description.unwrap_or_default(),
            event: imported.event,
            matcher: imported.matcher.unwrap_or_default(),
            command: imported.command,
            timeout: imported.timeout.unwrap_or_default(),
            agents,
        };
        hook.validate()
            .with_context(|| format!("invalid backup hook {:?}", hook.name))?;
        ensure!(
            encode(&hook)?.len() <= MAX_ENTRY,
            "hook entry exceeds 64 KiB"
        );
        let Entry::Vacant(slot) = hooks.entry(hook.name.clone()) else {
            bail!("duplicate backup hook name {:?}", hook.name);
        };
        slot.insert((hook, imported.other.into_keys().collect()));
    }
    for name in selected {
        ensure!(
            hooks.contains_key(name),
            "backup hook {name:?} was not found"
        );
    }

    let library = dir.join("hooks");
    let _lock = if apply && !hooks.is_empty() {
        Some(native_settings::lock(dir)?)
    } else {
        None
    };
    let path = library.join("library.json");
    let mut store = Store::load(&path)?;
    let mut planned = Vec::new();
    let mut changed = false;
    for (name, (hook, skipped_fields)) in hooks {
        let agents = hook.agents.clone();
        let event = hook.event.clone();
        let action = match store.hooks.entry(name.clone()) {
            Entry::Occupied(slot) if slot.get().revision()? == hook.revision()? => "identical",
            Entry::Occupied(_) if apply => bail!("a hook named {name} already exists"),
            Entry::Occupied(_) => "conflict",
            Entry::Vacant(slot) => {
                slot.insert(hook);
                changed = true;
                "create"
            }
        };
        planned.push(PlannedHook {
            name,
            event,
            agents,
            action,
            skipped_fields,
        });
    }
    if changed {
        let contents = store.contents()?;
        if apply {
            fs::create_dir_all(&library)?;
            super::replace(&path, &contents)?;
        }
    }
    Ok(ImportReport {
        hooks: planned,
        other_library_sections,
        applied: apply,
    })
}
