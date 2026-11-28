use super::{Skill, Store, Tree, details, lock_library, normalize_agents, snapshot};
use crate::config::backup_library_section;
use anyhow::{Context, Result, bail, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    path::Path,
};

#[derive(NsonDeserialize)]
struct BackupSkill {
    name: String,
    agents: Option<Vec<String>>,
    files: BTreeMap<String, String>,
    exec: Option<BTreeSet<String>>,
    left: Option<Vec<String>>,
    fetch: Option<bool>,
    bundle: Option<String>,
    #[njson(flatten)]
    other: BTreeMap<String, Value>,
}

#[derive(NsonSerialize)]
pub struct ImportReport {
    skills: Vec<PlannedSkill>,
    other_library_sections: Vec<String>,
    applied: bool,
}

#[derive(NsonSerialize)]
struct PlannedSkill {
    name: String,
    agents: Vec<String>,
    content: String,
    files: usize,
    action: &'static str,
    skipped_fields: Vec<String>,
}

/// Restore complete, embedded skill snapshots without contacting their sources.
pub fn import_backup(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    selected: &[String],
    apply: bool,
) -> Result<ImportReport> {
    let (entries, other_library_sections) = backup_library_section(file, passphrase, "skills")?;
    let selected: BTreeSet<_> = selected.iter().map(String::as_str).collect();
    let mut skills = BTreeMap::new();
    for entry in entries {
        let key = entry
            .pointer("/name")
            .and_then(Value::as_str)
            .context("missing backup skill name")?;
        if !selected.is_empty() && !selected.contains(key) {
            continue;
        }
        let imported: BackupSkill = nextjson::from_value(entry)
            .map_err(|_| anyhow::anyhow!("invalid backup skill definition"))?;
        ensure!(
            !imported.fetch.unwrap_or_default() && imported.left.as_ref().is_none_or(Vec::is_empty),
            "backup skill {:?} requires files from outside the backup",
            imported.name
        );
        ensure!(
            imported.bundle.as_deref().is_none_or(str::is_empty),
            "backup skill {:?} belongs to a plugin",
            imported.name
        );
        let mut skipped_fields: Vec<_> = imported.other.into_keys().collect();
        let executable = imported.exec.unwrap_or_default();
        if !cfg!(unix) && !executable.is_empty() {
            skipped_fields.push("exec".into());
            skipped_fields.sort_unstable();
        }
        let tree = Tree::from_backup(imported.files, &executable)
            .with_context(|| format!("read backup skill {:?}", imported.name))?;
        let (name, description) =
            details(&tree).with_context(|| format!("read backup skill {:?}", imported.name))?;
        ensure!(
            name == imported.name,
            "backup skill name differs from SKILL.md"
        );
        let agents = normalize_agents(imported.agents.unwrap_or_default())
            .with_context(|| format!("invalid assignments for backup skill {name:?}"))?;
        let content = tree.hash();
        let Entry::Vacant(slot) = skills.entry(name.clone()) else {
            bail!("duplicate backup skill name {name:?}");
        };
        let skill = Skill {
            name,
            description,
            source: file.to_str().context("backup path must be UTF-8")?.into(),
            content,
            agents,
        };
        slot.insert((skill, tree, skipped_fields));
    }
    for name in selected {
        ensure!(
            skills.contains_key(name),
            "backup skill {name:?} was not found"
        );
    }

    let library = dir.join("skills");
    let _lock = if apply && !skills.is_empty() {
        Some(lock_library(&library)?)
    } else {
        None
    };
    let path = library.join("library.json");
    let mut store = Store::load(&path)?;
    let mut planned = Vec::new();
    let mut staged = Vec::new();
    for (name, (skill, tree, skipped_fields)) in skills {
        let agents = skill.agents.clone();
        let content = skill.content.clone();
        let files = tree.files().count();
        let action = match store.skills.entry(name.clone()) {
            Entry::Occupied(slot)
                if slot.get().content == skill.content && slot.get().agents == skill.agents =>
            {
                "identical"
            }
            Entry::Occupied(_) if apply => bail!("a skill named {name} already exists"),
            Entry::Occupied(_) => "conflict",
            Entry::Vacant(slot) => {
                staged.push((content.clone(), tree));
                slot.insert(skill);
                "create"
            }
        };
        planned.push(PlannedSkill {
            name,
            agents,
            content,
            files,
            action,
            skipped_fields,
        });
    }
    if !staged.is_empty() {
        let contents = store.contents()?;
        if apply {
            for (hash, tree) in staged {
                snapshot(&library, &hash, &tree)?;
            }
            super::replace(&path, &contents)?;
        }
    }
    Ok(ImportReport {
        skills: planned,
        other_library_sections,
        applied: apply,
    })
}
