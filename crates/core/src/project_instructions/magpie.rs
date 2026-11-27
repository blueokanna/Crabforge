use super::{MAX_DOCUMENT, Section, edit_agent, read_agent, updated};
use crate::config::open_magpie_backup;
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use std::{collections::BTreeMap, path::Path};

#[derive(NsonDeserialize)]
struct Library {
    instructions: Instructions,
    texts: Option<BTreeMap<String, String>>,
    extra: Option<BTreeMap<String, String>>,
}

#[derive(NsonDeserialize)]
struct Instructions {
    agents: Option<Vec<String>>,
    sets: Option<Vec<InstructionSet>>,
    active: Option<String>,
    #[njson(rename = "noMemory")]
    no_memory: Option<bool>,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct InstructionSet {
    id: String,
    name: String,
}

pub struct MagpieInstructions<'a> {
    pub agent: &'a str,
    pub set: Option<&'a str>,
    pub revision: Option<&'a str>,
    pub apply: bool,
}

#[derive(NsonSerialize)]
pub struct ImportReport {
    sets: Vec<InstructionSet>,
    active_set: String,
    selected_set: String,
    source_agents: Vec<String>,
    source_no_memory: bool,
    agent: String,
    path: String,
    revision: String,
    text: String,
    action: &'static str,
    applied: bool,
}

/// Import a shared instruction set and the selected agent's additions into its user profile.
pub fn import_magpie(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    options: MagpieInstructions<'_>,
) -> Result<ImportReport> {
    let MagpieInstructions {
        agent,
        set,
        revision,
        apply,
    } = options;
    let mut backup = open_magpie_backup(file, passphrase)?;
    let library: Library = nextjson::from_value(
        backup
            .remove("library")
            .filter(|value| !value.is_null())
            .context("Magpie backup has no library section")?,
    )
    .map_err(|_| anyhow::anyhow!("invalid Magpie instruction sets"))?;
    let instructions = library.instructions;
    let mut named_sets = BTreeMap::new();
    for set in instructions.sets.unwrap_or_default() {
        ensure!(!set.id.is_empty(), "empty Magpie instruction set ID");
        ensure!(
            named_sets.insert(set.id.clone(), set).is_none(),
            "duplicate Magpie instruction set ID"
        );
    }
    named_sets
        .entry("default".into())
        .or_insert(InstructionSet {
            id: "default".into(),
            name: String::new(),
        });
    let active_set = instructions
        .active
        .filter(|id| named_sets.contains_key(id))
        .unwrap_or_else(|| "default".into());
    let selected_set = set.unwrap_or(&active_set);
    ensure!(
        named_sets.contains_key(selected_set),
        "Magpie instruction set {selected_set:?} was not found"
    );
    let mut texts = library.texts.unwrap_or_default();
    let mut extras = library.extra.unwrap_or_default();
    let shared = texts.remove(selected_set).unwrap_or_default();
    let extra = extras.remove(agent).unwrap_or_default();
    let text = [shared.as_str(), extra.as_str()]
        .into_iter()
        .map(|part| part.trim_start_matches('\u{feff}').trim())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
        .replace("\r\n", "\n");
    ensure!(
        text.len() as u64 <= MAX_DOCUMENT,
        "instructions exceed the {MAX_DOCUMENT}-byte limit"
    );
    updated("", &text, Section::Instructions)?;

    let mut current = read_agent(dir, agent)?;
    if let Some(expected) = revision {
        ensure!(
            expected == current.revision,
            "instructions changed; reload the file"
        );
    }
    let action = if current.text == text {
        "identical"
    } else if current.text.is_empty() {
        "create"
    } else if revision.is_some() {
        "replace"
    } else {
        "conflict"
    };
    if apply && action != "identical" {
        ensure!(
            action != "conflict",
            "managed instructions already exist; review the preview revision before replacing them"
        );
        edit_agent(dir, agent, &current.revision, Some(&text))?;
        current = read_agent(dir, agent)?;
    }
    Ok(ImportReport {
        sets: named_sets.into_values().collect(),
        active_set: active_set.clone(),
        selected_set: selected_set.into(),
        source_agents: instructions.agents.unwrap_or_default(),
        source_no_memory: instructions.no_memory.unwrap_or_default(),
        agent: agent.into(),
        path: current.path,
        revision: current.revision,
        text,
        action,
        applied: apply,
    })
}
