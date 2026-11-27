use super::{Rule, Store, lock_library, revision};
use crate::config::magpie_library_section;
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    path::Path,
};

#[derive(NsonDeserialize)]
struct MagpieRule {
    name: String,
    body: String,
    paths: Option<Vec<String>>,
    agents: Option<Vec<String>>,
    #[njson(flatten)]
    other: BTreeMap<String, Value>,
}

#[derive(NsonSerialize)]
pub struct ImportReport {
    rules: Vec<PlannedRule>,
    other_library_sections: Vec<String>,
    applied: bool,
}

#[derive(NsonSerialize)]
struct PlannedRule {
    name: String,
    agents: Vec<String>,
    paths: Vec<String>,
    action: &'static str,
    skipped_fields: Vec<String>,
}

/// Stage portable Markdown rules in the user library for review and sync.
pub fn import_magpie(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    selected: &[String],
    apply: bool,
) -> Result<ImportReport> {
    let (entries, other_library_sections) = magpie_library_section(file, passphrase, "rules")?;
    let selected: BTreeSet<_> = selected.iter().map(String::as_str).collect();
    let mut rules = BTreeMap::new();
    for entry in entries {
        let name = entry
            .pointer("/name")
            .and_then(Value::as_str)
            .context("missing Magpie rule name")?;
        if !selected.is_empty() && !selected.contains(name) {
            continue;
        }
        let rule: MagpieRule = nextjson::from_value(entry)
            .map_err(|_| anyhow::anyhow!("invalid Magpie rule definition"))?;
        let rule_name = rule.name.clone();
        let rule_definition = Rule {
            name: rule.name,
            body: rule.body,
            paths: rule.paths.unwrap_or_default(),
            agents: rule.agents.unwrap_or_default(),
        }
        .normalize()
        .with_context(|| format!("invalid Magpie rule {rule_name:?}"))?;
        for agent in &rule_definition.agents {
            rule_definition
                .render(agent)
                .with_context(|| format!("cannot render Magpie rule {rule_name:?} for {agent}"))?;
        }
        let Entry::Vacant(slot) = rules.entry(rule_definition.name.clone()) else {
            anyhow::bail!("duplicate Magpie rule name {rule_name:?}");
        };
        slot.insert((rule_definition, rule.other.into_keys().collect()));
    }
    for name in selected {
        ensure!(
            rules.contains_key(name),
            "Magpie rule {name:?} was not found"
        );
    }

    let folder = dir.join("rules");
    let _lock = if apply && !rules.is_empty() {
        Some(lock_library(&folder)?)
    } else {
        None
    };
    let path = folder.join("library.json");
    let mut store = Store::load(&path)?;
    let mut planned = Vec::new();
    let mut changed = false;
    for (name, (rule, skipped_fields)) in rules {
        let agents = rule.agents.clone();
        let paths = rule.paths.clone();
        let action = match store.rules.entry(name.clone()) {
            Entry::Occupied(slot) if revision(slot.get())? == revision(&rule)? => "identical",
            Entry::Occupied(_) if apply => anyhow::bail!("a rule named {name} already exists"),
            Entry::Occupied(_) => "conflict",
            Entry::Vacant(slot) => {
                slot.insert(rule);
                changed = true;
                "create"
            }
        };
        planned.push(PlannedRule {
            name,
            agents,
            paths,
            action,
            skipped_fields,
        });
    }
    if changed {
        if apply {
            store.persist(&path)?;
        } else {
            store.contents()?;
        }
    }
    Ok(ImportReport {
        rules: planned,
        other_library_sections,
        applied: apply,
    })
}
