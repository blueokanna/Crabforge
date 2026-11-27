use super::{Store, Variable, name_key};
use crate::{config::magpie_library_section, native_settings};
use anyhow::{Context, Result, bail, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    fs,
    path::Path,
};

#[derive(NsonDeserialize)]
struct MagpieVariable {
    name: String,
    value: String,
    agents: Option<BTreeSet<String>>,
    #[njson(flatten)]
    other: BTreeMap<String, Value>,
}

#[derive(NsonSerialize)]
pub struct ImportReport {
    variables: Vec<PlannedVariable>,
    other_library_sections: Vec<String>,
    applied: bool,
}

#[derive(NsonSerialize)]
struct PlannedVariable {
    name: String,
    agents: BTreeSet<String>,
    has_value: bool,
    action: &'static str,
    skipped_fields: Vec<String>,
}

/// Copy environment definitions into the library without publishing to clients.
pub fn import_magpie(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    selected: &[String],
    apply: bool,
) -> Result<ImportReport> {
    let (entries, other_library_sections) = magpie_library_section(file, passphrase, "env")?;
    let selected: BTreeSet<_> = selected.iter().map(|name| name_key(name)).collect();
    let mut variables = BTreeMap::new();
    for entry in entries {
        let name = entry
            .pointer("/name")
            .and_then(Value::as_str)
            .context("missing Magpie environment variable name")?;
        let key = name_key(name);
        if !selected.is_empty() && !selected.contains(&key) {
            continue;
        }
        let imported: MagpieVariable = nextjson::from_value(entry)
            .map_err(|_| anyhow::anyhow!("invalid Magpie environment variable"))?;
        let variable = Variable {
            name: imported.name,
            value: imported.value,
            agents: imported.agents.unwrap_or_default(),
        };
        variable
            .validate()
            .with_context(|| format!("invalid Magpie environment variable {key:?}"))?;
        let Entry::Vacant(slot) = variables.entry(key) else {
            bail!("duplicate Magpie environment variable {:?}", variable.name);
        };
        slot.insert((variable, imported.other.into_keys().collect()));
    }
    for key in selected {
        ensure!(
            variables.contains_key(&key),
            "Magpie environment variable {key:?} was not found"
        );
    }

    let folder = dir.join("environment");
    let _lock = if apply && !variables.is_empty() {
        Some(native_settings::lock(dir)?)
    } else {
        None
    };
    let path = folder.join("library.json");
    let mut store = Store::load(&path)?;
    let mut planned = Vec::new();
    let mut changed = false;
    for (key, (variable, skipped_fields)) in variables {
        let name = variable.name.clone();
        let agents = variable.agents.clone();
        let has_value = !variable.value.is_empty();
        let action = match store.variables.entry(key) {
            Entry::Occupied(slot) if slot.get() == &variable => "identical",
            Entry::Occupied(_) if apply => bail!("environment variable {name} already exists"),
            Entry::Occupied(_) => "conflict",
            Entry::Vacant(slot) => {
                slot.insert(variable);
                changed = true;
                "create"
            }
        };
        planned.push(PlannedVariable {
            name,
            agents,
            has_value,
            action,
            skipped_fields,
        });
    }
    if changed {
        if apply {
            fs::create_dir_all(&folder)?;
            store.save(&path)?;
        } else {
            store.contents()?;
        }
    }
    Ok(ImportReport {
        variables: planned,
        other_library_sections,
        applied: apply,
    })
}
