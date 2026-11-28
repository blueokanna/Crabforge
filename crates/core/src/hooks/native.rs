use anyhow::{Context, Result, ensure};
use nextjson::{
    Map, Value,
    formats::{Format, Json5},
    json,
};
use std::collections::{BTreeMap, BTreeSet};

use super::{Agent, Hook, antigravity};
use crate::project_mcp::jsonc;

const PORTABLE: [&str; 5] = [
    "turn-end",
    "after-tool",
    "before-tool",
    "prompt",
    "session-start",
];
const STARTS: &[&str] = &["startup", "resume", "clear", "compact", "fork"];

struct Events {
    mapped: [&'static str; 5],
    additional: &'static [&'static str],
    shell: &'static str,
    edit: &'static str,
}

fn events(agent: Agent) -> Events {
    match agent {
        Agent::Claude => Events {
            mapped: [
                "Stop",
                "PostToolUse",
                "PreToolUse",
                "UserPromptSubmit",
                "SessionStart",
            ],
            additional: &[
                "PostToolUseFailure",
                "PostToolBatch",
                "Notification",
                "UserPromptExpansion",
                "SessionEnd",
                "StopFailure",
                "SubagentStart",
                "SubagentStop",
                "PreCompact",
                "PostCompact",
                "PreModelSwitch",
                "PostModelSwitch",
                "PermissionRequest",
                "PermissionDenied",
                "Setup",
                "TeammateIdle",
                "TaskCreated",
                "TaskCompleted",
                "Elicitation",
                "ElicitationResult",
                "ConfigChange",
                "WorktreeCreate",
                "WorktreeRemove",
                "InstructionsLoaded",
                "CwdChanged",
                "FileChanged",
                "DirectoryAdded",
                "MessageDisplay",
            ],
            shell: "Bash",
            edit: "Edit|Write|MultiEdit",
        },
        Agent::Codex => Events {
            mapped: [
                "Stop",
                "PostToolUse",
                "PreToolUse",
                "UserPromptSubmit",
                "SessionStart",
            ],
            additional: &[
                "PermissionRequest",
                "PreCompact",
                "PostCompact",
                "SessionEnd",
                "SubagentStart",
                "SubagentStop",
                "Interrupt",
            ],
            shell: "Bash",
            edit: "apply_patch",
        },
        Agent::Cursor => Events {
            mapped: [
                "stop",
                "postToolUse",
                "preToolUse",
                "beforeSubmitPrompt",
                "sessionStart",
            ],
            additional: &[
                "beforeShellExecution",
                "afterShellExecution",
                "beforeMCPExecution",
                "afterMCPExecution",
                "beforeReadFile",
                "afterFileEdit",
                "afterAgentResponse",
                "afterAgentThought",
                "sessionEnd",
                "postToolUseFailure",
                "subagentStart",
                "subagentStop",
                "preCompact",
            ],
            shell: "Shell",
            edit: "Write",
        },
        Agent::Antigravity => Events {
            mapped: ["Stop", "PostToolUse", "PreToolUse", "", "SessionStart"],
            additional: &["PreInvocation", "PostInvocation"],
            shell: "run_command",
            edit: "write_to_file|replace_file_content|multi_replace_file_content",
        },
        Agent::Grok => Events {
            mapped: [
                "Stop",
                "PostToolUse",
                "PreToolUse",
                "UserPromptSubmit",
                "SessionStart",
            ],
            additional: &[
                "SessionEnd",
                "PostToolUseFailure",
                "PermissionDenied",
                "StopFailure",
                "StopCancelled",
                "Notification",
                "SubagentStart",
                "SubagentStop",
                "PreCompact",
                "PostCompact",
            ],
            shell: "run_terminal_command",
            edit: "search_replace",
        },
        Agent::Dsh => Events {
            mapped: [
                "Stop",
                "PostToolUse",
                "PreToolUse",
                "UserPromptSubmit",
                "SessionStart",
            ],
            additional: &["SubagentStart", "SubagentStop"],
            shell: "bash|pwsh",
            edit: "write|edit|str_replace_editor",
        },
    }
}

pub(super) fn native_events(agent: Agent) -> Vec<&'static str> {
    let spec = events(agent);
    let mut names = spec
        .mapped
        .into_iter()
        .chain(spec.additional.iter().copied())
        .filter(|event| !event.is_empty())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

pub(super) fn validate_event(key: &str, matcher: &str) -> Result<()> {
    if let Some((agent, event)) = key.split_once(':') {
        let spec = events(Agent::parse(agent)?);
        ensure!(
            !event.is_empty() && (spec.mapped.contains(&event) || spec.additional.contains(&event)),
            "unsupported native hook event"
        );
        return Ok(());
    }
    ensure!(PORTABLE.contains(&key), "unsupported hook event");
    if matcher.is_empty() {
        return Ok(());
    }
    match key {
        "session-start" => {
            let mut seen = BTreeSet::new();
            ensure!(
                matcher
                    .split('|')
                    .all(|part| STARTS.contains(&part) && seen.insert(part)),
                "invalid session start matcher"
            );
        }
        "after-tool" | "before-tool" => ensure!(
            matches!(matcher, "shell" | "edit"),
            "use shell or edit for a portable tool event"
        ),
        _ => anyhow::bail!("this portable hook event has no matcher"),
    }
    Ok(())
}

pub(super) fn encode(agent: Agent, hook: &Hook) -> Result<(String, Value)> {
    let spec = events(agent);
    let (event, matcher) = if let Some((owner, event)) = hook.event.split_once(':') {
        ensure!(
            agent.id() == owner,
            "native event belongs to another client"
        );
        (event, hook.matcher.as_str())
    } else {
        let index = PORTABLE
            .iter()
            .position(|key| *key == hook.event)
            .context("unknown portable event")?;
        let matcher = match hook.matcher.as_str() {
            "shell" => spec.shell,
            "edit" => spec.edit,
            starts if !starts.is_empty() && hook.event == "session-start" => match agent {
                Agent::Cursor => anyhow::bail!("Cursor cannot filter session start sources"),
                Agent::Grok => {
                    let requested = |source| starts.split('|').any(|start| start == source);
                    let fresh = requested("startup");
                    ensure!(
                        fresh == requested("clear"),
                        "Grok runs startup and clear under the same session start source"
                    );
                    match (fresh, requested("resume")) {
                        (true, true) => "",
                        (true, false) => "new",
                        (false, true) => "load",
                        (false, false) => {
                            anyhow::bail!("Grok has no requested session start source")
                        }
                    }
                }
                Agent::Antigravity => {
                    ensure!(
                        starts.split('|').any(|start| start == "startup"),
                        "Antigravity has no requested session start source"
                    );
                    ""
                }
                _ => starts,
            },
            _ => "",
        };
        (spec.mapped[index], matcher)
    };
    ensure!(
        !event.is_empty(),
        "this client has no equivalent hook event"
    );
    ensure!(
        agent != Agent::Antigravity || matcher.is_empty() || antigravity::tool_event(event),
        "Antigravity only supports hook matchers on tool events"
    );
    let mut command = Map::from_iter([("command", Value::from(hook.command.as_str()))]);
    if hook.timeout != 0 {
        command.insert("timeout".into(), json!(hook.timeout));
    }
    let mut entry = if agent == Agent::Cursor {
        command
    } else {
        command.insert("type".into(), json!("command"));
        if agent == Agent::Antigravity && !antigravity::tool_event(event) {
            command
        } else {
            Map::from_iter([("hooks", Value::Array(vec![Value::Object(command)]))])
        }
    };
    if !matcher.is_empty() {
        entry.insert("matcher".into(), json!(matcher));
    }
    Ok((event.to_owned(), Value::Object(entry)))
}

pub(super) fn decode(agent: Agent, event: &str, entry: &Value) -> Option<Hook> {
    let object = entry.as_object()?;
    let (command, timeout) = if agent == Agent::Antigravity && !antigravity::tool_event(event) {
        if object
            .keys()
            .any(|key| !["type", "command", "timeout"].contains(&key))
            || object
                .get("type")
                .is_some_and(|ty| ty.as_str() != Some("command"))
        {
            return None;
        }
        (object.get("command")?, object.get("timeout"))
    } else if agent == Agent::Cursor {
        if object
            .keys()
            .any(|key| !["command", "matcher", "timeout"].contains(&key))
        {
            return None;
        }
        (object.get("command")?, object.get("timeout"))
    } else {
        if object
            .keys()
            .any(|key| !["hooks", "matcher"].contains(&key))
        {
            return None;
        }
        let [handler] = object.get("hooks")?.as_array()?.as_slice() else {
            return None;
        };
        let handler = handler.as_object()?;
        let command_type = handler
            .get("type")
            .map_or(agent == Agent::Antigravity, |ty| {
                ty.as_str() == Some("command")
            });
        if !command_type
            || handler
                .keys()
                .any(|key| !["type", "command", "timeout"].contains(&key))
        {
            return None;
        }
        (handler.get("command")?, handler.get("timeout"))
    };
    let timeout = match timeout {
        Some(timeout) => timeout
            .as_u64()
            .filter(|value| timeout.as_f64() == Some(*value as f64))?,
        None => 0,
    };
    let matcher = match object.get("matcher") {
        Some(matcher) => matcher.as_str()?.to_owned(),
        None => String::new(),
    };
    Some(Hook {
        name: "imported".into(),
        description: String::new(),
        event: format!("{}:{event}", agent.id()),
        matcher,
        command: command.as_str()?.to_owned(),
        timeout: u32::try_from(timeout).ok()?,
        agents: BTreeSet::from([agent]),
    })
}

pub(super) fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted = map
                .iter()
                .map(|(key, item)| (key, canonical(item)))
                .collect::<BTreeMap<_, _>>();
            Value::Object(Map::from_iter(sorted))
        }
        Value::Array(list) => Value::Array(list.iter().map(canonical).collect()),
        other => other.clone(),
    }
}

pub(super) fn read(source: &str) -> Result<BTreeMap<String, Vec<Value>>> {
    jsonc::parse(source, &["hooks"])?.map_or_else(
        || Ok(BTreeMap::new()),
        |object| read_events(source, &object, |_| true),
    )
}

pub(super) fn read_events(
    source: &str,
    object: &jsonc::Object,
    include: impl Fn(&str) -> bool,
) -> Result<BTreeMap<String, Vec<Value>>> {
    object
        .keys()
        .filter(|key| include(key))
        .map(|event| {
            let array = object
                .array(source, event)?
                .context("missing native hooks event")?;
            let entries = array
                .values(source)
                .map(|text| {
                    let entry: Value = Json5
                        .decode(text.as_bytes())
                        .map_err(|_| anyhow::anyhow!("invalid native hook entry"))?;
                    Ok(canonical(&entry))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok((event.to_owned(), entries))
        })
        .collect()
}

pub(super) struct Location {
    pub(super) enabled: bool,
    pub(super) entries: Vec<Value>,
}

pub(super) type Locations = BTreeMap<(Option<String>, String), Location>;

pub(super) fn read_locations(agent: Agent, source: &str) -> Result<Locations> {
    if agent == Agent::Antigravity {
        antigravity::read(source)
    } else {
        Ok(read(source)?
            .into_iter()
            .map(|(event, entries)| {
                (
                    (None, event),
                    Location {
                        enabled: true,
                        entries,
                    },
                )
            })
            .collect())
    }
}

pub(super) fn add(source: &str, event: &str, entry: Value) -> Result<String> {
    match jsonc::parse(source, &["hooks"])? {
        Some(object) => match object.array(source, event)? {
            Some(array) => array.append(source, &entry),
            None => object.edit(source, event, Some(&Value::Array(vec![entry]))),
        },
        None => jsonc::parse(source, &[])?
            .context("missing native hooks configuration object")?
            .edit(source, "hooks", Some(&json!({(event): [entry]}))),
    }
}

pub(super) fn cursor_version(source: &str) -> Result<String> {
    let root = jsonc::parse(source, &[])?.context("missing Cursor hooks configuration")?;
    if let Some((_, version)) = root.entries(source).find(|(key, _)| key == "version") {
        ensure!(version == "1", "unsupported Cursor hooks version");
        return Ok(source.to_owned());
    }
    jsonc::parse(source, &[])?
        .context("missing Cursor hooks configuration")?
        .edit(source, "version", Some(&json!(1)))
}

pub(super) fn remove(
    agent: Agent,
    source: &str,
    group: Option<&str>,
    event: &str,
    index: usize,
) -> Result<String> {
    let entries = jsonc::parse(source, &[group.unwrap_or("hooks")])?
        .context("missing native hooks object")?
        .array(source, event)?
        .context("hook event was removed")?;
    if agent == Agent::Codex {
        // The group position is part of Codex's hook trust key.
        entries.replace(source, index, "{\"hooks\":[]}")
    } else {
        entries.remove(source, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_session_start_preserves_client_source_filters() -> Result<()> {
        let mut hook = Hook {
            name: "resume-audit".into(),
            description: String::new(),
            event: "session-start".into(),
            matcher: "resume".into(),
            command: "echo fixture".into(),
            timeout: 0,
            agents: BTreeSet::new(),
        };
        assert_eq!(
            encode(Agent::Grok, &hook)?.1["matcher"].as_str(),
            Some("load")
        );
        assert!(encode(Agent::Antigravity, &hook).is_err());
        hook.matcher = "startup".into();
        assert!(encode(Agent::Grok, &hook).is_err());
        assert!(
            encode(Agent::Antigravity, &hook)?
                .1
                .get("matcher")
                .is_none()
        );
        hook.matcher = "clear|startup|compact".into();
        assert_eq!(
            encode(Agent::Grok, &hook)?.1["matcher"].as_str(),
            Some("new")
        );
        hook.matcher.push_str("|resume");
        assert!(encode(Agent::Grok, &hook)?.1.get("matcher").is_none());
        Ok(())
    }
}
