use super::{Tree, encode_index, inspect, lock_library, read_index, replace, snapshot};
use crate::config::backup_library_section;
use anyhow::{Context, Result, bail, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    path::Path,
};

#[derive(NsonDeserialize)]
struct CarriedPlugin {
    name: String,
    agents: Option<Vec<String>>,
    off: Option<bool>,
    taken: Option<Vec<String>>,
    left: Option<Vec<String>>,
    files: Option<BTreeMap<String, String>>,
    exec: Option<BTreeSet<String>>,
}

#[derive(NsonSerialize)]
pub struct ImportReport {
    bundles: Vec<PlannedBundle>,
    other_library_sections: Vec<String>,
    applied: bool,
}

#[derive(NsonSerialize)]
struct PlannedBundle {
    name: String,
    content: String,
    files: usize,
    source_agents: Vec<String>,
    source_enabled: bool,
    executable_metadata_skipped: bool,
    action: &'static str,
}

/// Restore complete plugin snapshots into the shared catalog for explicit project assignment.
pub fn import_backup(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    selected: &[String],
    apply: bool,
) -> Result<ImportReport> {
    let (entries, other_library_sections) = backup_library_section(file, passphrase, "plugins")?;
    let selected: BTreeSet<_> = selected.iter().map(String::as_str).collect();
    let mut found = BTreeSet::new();
    let mut imported = BTreeMap::new();
    for entry in entries {
        let name = entry
            .get("name")
            .and_then(Value::as_str)
            .context("missing backup plugin name")?;
        if !selected.is_empty() && !selected.contains(name) {
            continue;
        }
        ensure!(
            found.insert(name.to_owned()),
            "duplicate backup plugin name {name:?}"
        );
        let entry: CarriedPlugin = nextjson::from_value(entry)
            .map_err(|_| anyhow::anyhow!("invalid backup plugin definition"))?;
        ensure!(
            entry.taken.as_ref().is_none_or(Vec::is_empty),
            "backup plugin {:?} has removed components",
            entry.name
        );
        let files = entry.files.filter(|files| !files.is_empty());
        ensure!(
            files.is_some() && entry.left.as_ref().is_none_or(Vec::is_empty),
            "backup plugin {:?} requires files from outside the backup",
            entry.name
        );
        let files = files.expect("checked for embedded plugin files");
        ensure!(
            files.keys().all(|path| !path
                .split('/')
                .any(|part| part.eq_ignore_ascii_case(".git"))),
            "backup plugin {:?} contains Git metadata",
            entry.name
        );
        let executable = entry.exec.unwrap_or_default();
        let tree = Tree::from_backup(files, &executable)
            .with_context(|| format!("read backup plugin {:?}", entry.name))?;
        let source = file
            .to_str()
            .context("backup path must be UTF-8")?
            .to_owned();
        let bundle = inspect(&tree, source)
            .with_context(|| format!("inspect backup plugin {:?}", entry.name))?;
        ensure!(
            bundle.name == entry.name,
            "backup plugin name differs from its manifest"
        );
        let report = PlannedBundle {
            name: bundle.name.clone(),
            content: bundle.content.clone(),
            files: bundle.files,
            source_agents: entry.agents.unwrap_or_default(),
            source_enabled: !entry.off.unwrap_or_default(),
            executable_metadata_skipped: !cfg!(unix) && !executable.is_empty(),
            action: "create",
        };
        let Entry::Vacant(slot) = imported.entry(bundle.name.to_ascii_lowercase()) else {
            bail!("ambiguous backup plugin name {:?}", bundle.name);
        };
        slot.insert((bundle, tree, report));
    }
    for name in selected {
        ensure!(found.contains(name), "backup plugin {name:?} was not found");
    }

    let library = dir.join("plugin-bundles");
    let _lock = (apply && !imported.is_empty())
        .then(|| lock_library(&library))
        .transpose()?;
    let index = library.join("index.json");
    let mut bundles = read_index(&index)?;
    let mut planned = Vec::new();
    let mut staged = Vec::new();
    for (name, (bundle, tree, mut report)) in imported {
        match bundles.entry(name) {
            Entry::Occupied(slot) if slot.get().content == bundle.content => {
                report.action = "identical"
            }
            Entry::Occupied(_) if apply => {
                bail!("a plugin bundle named {:?} already exists", bundle.name)
            }
            Entry::Occupied(_) => report.action = "conflict",
            Entry::Vacant(slot) => {
                staged.push((bundle.content.clone(), tree));
                slot.insert(bundle);
            }
        }
        planned.push(report);
    }
    if !staged.is_empty() {
        let contents = encode_index(&bundles)?;
        if apply {
            let objects = library.join("objects");
            for (hash, tree) in staged {
                snapshot(&objects, &hash, &tree)?;
            }
            replace(&index, &contents)?;
        }
    }
    Ok(ImportReport {
        bundles: planned,
        other_library_sections,
        applied: apply,
    })
}
