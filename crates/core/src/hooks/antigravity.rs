use super::native::{self, Location, Locations};
use crate::project_mcp::jsonc;
use anyhow::{Context, Result, ensure};
use nextjson::{Value, json};

pub(super) const GROUP: &str = "crabforge";
const EVENTS: &[&str] = &[
    "SessionStart",
    "PreToolUse",
    "PostToolUse",
    "PreInvocation",
    "PostInvocation",
    "Stop",
];

pub(super) fn tool_event(event: &str) -> bool {
    matches!(event, "PreToolUse" | "PostToolUse")
}

fn enabled(source: &str, definition: &jsonc::Object) -> bool {
    definition.value(source, "enabled") != Some("false")
}

pub(super) fn read(source: &str) -> Result<Locations> {
    let root = jsonc::parse(source, &[])?.context("missing Antigravity hooks object")?;
    let mut locations = Locations::new();
    for (group, text) in root.entries(source) {
        let definition = jsonc::parse(text, &[])?.context("missing Antigravity hook name")?;
        let enabled = enabled(text, &definition);
        for (event, entries) in
            native::read_events(text, &definition, |event| EVENTS.contains(&event))?
        {
            locations.insert(
                (Some(group.to_owned()), event),
                Location { enabled, entries },
            );
        }
    }
    Ok(locations)
}

pub(super) fn add(source: &str, group: &str, event: &str, entry: Value) -> Result<String> {
    match jsonc::parse(source, &[group])? {
        Some(definition) => {
            ensure!(
                enabled(source, &definition),
                "Antigravity hook group {group} is disabled"
            );
            match definition.array(source, event)? {
                Some(array) => array.append(source, &entry),
                None => definition.edit(source, event, Some(&Value::Array(vec![entry]))),
            }
        }
        None => jsonc::parse(source, &[])?
            .context("missing Antigravity hooks object")?
            .edit(source, group, Some(&json!({(event): [entry]}))),
    }
}
