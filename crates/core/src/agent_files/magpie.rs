use super::{Entry, Kind, Store, native, revision};
use crate::{config::magpie_library_section, native_settings};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(NsonDeserialize)]
struct Resource {
    name: String,
    description: String,
    prompt: String,
    agents: Option<Vec<String>>,
    model: Option<String>,
    effort: Option<String>,
    skills: Option<Vec<String>>,
    mcp: Option<Vec<String>>,
    bundle: Option<String>,
    #[njson(flatten)]
    other: BTreeMap<String, Value>,
}

#[derive(NsonSerialize)]
pub struct ImportReport {
    section: &'static str,
    resources: Vec<PlannedResource>,
    other_library_sections: Vec<String>,
    applied: bool,
}

#[derive(NsonSerialize)]
struct PlannedResource {
    name: String,
    agents: Vec<String>,
    action: &'static str,
    skipped_fields: Vec<String>,
}

/// Restore portable resources to the user library for later review and synchronization.
pub fn import_magpie(
    dir: &Path,
    kind: Kind,
    file: &Path,
    passphrase: &str,
    selected: &[String],
    apply: bool,
) -> Result<ImportReport> {
    let section = kind.folder();
    let (resources, other_library_sections) = magpie_library_section(file, passphrase, section)?;
    let selected: BTreeSet<_> = selected.iter().map(String::as_str).collect();
    let mut imported = BTreeMap::new();
    for resource in resources {
        let name = resource
            .pointer("/name")
            .and_then(Value::as_str)
            .context("missing Magpie resource name")?;
        if selected.is_empty() || selected.contains(name) {
            let (entry, skipped_fields) = convert(kind, resource)?;
            ensure!(
                !imported.contains_key(&entry.name),
                "duplicate Magpie resource name {:?}",
                entry.name
            );
            imported.insert(entry.name.clone(), (entry, skipped_fields));
        }
    }
    for name in selected {
        ensure!(
            imported.contains_key(name),
            "Magpie resource {name:?} was not found in {section}"
        );
    }

    let _guard = if apply && !imported.is_empty() {
        Some(native_settings::lock(dir)?)
    } else {
        None
    };
    let path = dir.join(section).join("library.json");
    let mut store = Store::load(&path)?;
    let mut plan = Vec::new();
    let mut changed = false;
    for (name, (entry, skipped_fields)) in imported {
        let agents = entry.agents.clone();
        let action = match store.entries.get(&name) {
            Some(current) if revision(current)? == revision(&entry)? => "identical",
            Some(_) if apply => anyhow::bail!("resource name already in use: {name}"),
            Some(_) => "conflict",
            None => {
                store.entries.insert(name.clone(), entry);
                changed = true;
                "create"
            }
        };
        plan.push(PlannedResource {
            name,
            agents,
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
        section,
        resources: plan,
        other_library_sections,
        applied: apply,
    })
}

fn convert(kind: Kind, value: Value) -> Result<(Entry, Vec<String>)> {
    let resource: Resource = nextjson::from_value(value)
        .map_err(|_| anyhow::anyhow!("invalid Magpie resource definition"))?;
    ensure!(
        resource.bundle.as_deref().is_none_or(str::is_empty),
        "{} belongs to a Magpie plugin bundle",
        resource.name
    );
    let mut entry = Entry {
        name: resource.name,
        description: resource.description,
        prompt: resource.prompt,
        agents: resource.agents.unwrap_or_default(),
        model: resource.model.filter(|model| !model.is_empty()),
        effort: resource.effort.filter(|effort| !effort.is_empty()),
        skills: resource.skills.unwrap_or_default(),
        mcp: resource.mcp.unwrap_or_default(),
    };
    entry
        .validate(kind, native::user_agents(kind))
        .with_context(|| format!("invalid Magpie resource {:?}", entry.name))?;
    if kind == Kind::Command {
        for agent in &entry.agents {
            native::command(agent, agent == "gemini", &entry).with_context(|| {
                format!("cannot render Magpie command {:?} for {agent}", entry.name)
            })?;
        }
    }
    Ok((entry, resource.other.into_keys().collect()))
}
