use crate::agents::NativeAgent as Agent;
use anyhow::{Context, Result};
use nextjson::{NsonSerialize, Value};
use std::{
    cmp::Reverse,
    collections::HashSet,
    fs,
    io::{BufRead, BufReader, ErrorKind, Read},
    path::{Path, PathBuf},
    time::SystemTime,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const MAX_HEADER: u64 = 512 * 1024;

#[derive(NsonSerialize)]
pub struct Session {
    pub agent: &'static str,
    pub id: String,
    pub cwd: String,
    pub title: String,
    pub modified: String,
    pub archived: bool,
}

#[derive(NsonSerialize)]
pub struct Listing {
    pub sessions: Vec<Session>,
    pub unreadable: usize,
}

struct Log {
    agent: Agent,
    path: PathBuf,
    modified: SystemTime,
    archived: bool,
}

pub fn list(agent: Option<Agent>, limit: usize) -> Result<Listing> {
    let roots = [Agent::Claude, Agent::Codex]
        .into_iter()
        .filter(|item| agent.is_none_or(|agent| agent == *item))
        .filter_map(|agent| agent.home().map(|home| (agent, home)));
    list_in(roots, limit)
}

fn list_in(roots: impl IntoIterator<Item = (Agent, PathBuf)>, limit: usize) -> Result<Listing> {
    let mut logs = Vec::new();
    for (agent, home) in roots {
        match agent {
            Agent::Claude => visit(&home.join("projects"), 1, agent, false, &mut logs)?,
            Agent::Codex => {
                visit(&home.join("sessions"), 3, agent, false, &mut logs)?;
                visit(&home.join("archived_sessions"), 0, agent, true, &mut logs)?;
            }
        }
    }
    logs.sort_unstable_by(|a, b| {
        (Reverse(a.modified), a.archived, &a.path).cmp(&(Reverse(b.modified), b.archived, &b.path))
    });
    let mut result = Listing {
        sessions: Vec::new(),
        unreadable: 0,
    };
    let mut seen = HashSet::new();
    for log in logs {
        match log.session() {
            Ok(Some(session)) if seen.insert((log.agent, session.id.clone())) => {
                result.sessions.push(session);
                if result.sessions.len() == limit.clamp(1, 500) {
                    break;
                }
            }
            Ok(_) => (),
            Err(_) => result.unreadable += 1,
        }
    }
    Ok(result)
}

fn visit(
    root: &Path,
    depth: usize,
    agent: Agent,
    archived: bool,
    logs: &mut Vec<Log>,
) -> Result<()> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).with_context(|| format!("cannot list {}", root.display())),
    };
    for entry in entries {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() && depth > 0 {
            visit(&entry.path(), depth - 1, agent, archived, logs)?;
        } else if kind.is_file()
            && let Some(name) = entry.file_name().to_str()
            && name.ends_with(".jsonl")
            && match agent {
                Agent::Claude => depth == 0,
                Agent::Codex => name.starts_with("rollout-"),
            }
        {
            match entry.metadata().and_then(|metadata| metadata.modified()) {
                Ok(modified) => logs.push(Log {
                    agent,
                    path: entry.path(),
                    modified,
                    archived,
                }),
                Err(error) if error.kind() == ErrorKind::NotFound => (),
                Err(error) => return Err(error.into()),
            }
        }
    }
    Ok(())
}

impl Log {
    fn session(&self) -> Result<Option<Session>> {
        let mut reader = BufReader::new(fs::File::open(&self.path)?.take(MAX_HEADER));
        let mut bytes = Vec::new();
        let mut id = String::new();
        let mut cwd = String::new();
        let mut title = String::new();
        while reader.read_until(b'\n', &mut bytes)? > 0 {
            let row = nextjson::from_slice::<Value>(&bytes);
            bytes.clear();
            let Ok(row) = row else { continue };
            let kind = string(&row, "/type");
            match self.agent {
                Agent::Claude => {
                    if flag(&row, "/isSidechain") || flag(&row, "/isMeta") {
                        continue;
                    }
                    if id.is_empty() {
                        id = string(&row, "/sessionId").to_owned();
                    }
                    if cwd.is_empty() {
                        cwd = string(&row, "/cwd").to_owned();
                    }
                    if title.is_empty() && kind == "user" && !flag(&row, "/isCompactSummary") {
                        title = message_title(row.pointer("/message/content"));
                    }
                }
                Agent::Codex => {
                    if kind == "session_meta" {
                        if !string(&row, "/payload/parent_thread_id").is_empty()
                            || row.pointer("/payload/source/subagent").is_some()
                        {
                            return Ok(None);
                        }
                        id = string(&row, "/payload/id").to_owned();
                        cwd = string(&row, "/payload/cwd").to_owned();
                    } else if title.is_empty()
                        && kind == "event_msg"
                        && string(&row, "/payload/type") == "user_message"
                    {
                        title = concise(string(&row, "/payload/message"));
                    } else if title.is_empty()
                        && kind == "response_item"
                        && string(&row, "/payload/type") == "message"
                        && string(&row, "/payload/role") == "user"
                    {
                        title = message_title(row.pointer("/payload/content"));
                    }
                }
            }
            if !id.is_empty() && !cwd.is_empty() && !title.is_empty() {
                break;
            }
        }
        if id.is_empty() || title.is_empty() {
            return Ok(None);
        }
        Ok(Some(Session {
            agent: self.agent.id(),
            id,
            cwd,
            title,
            archived: self.archived,
            modified: OffsetDateTime::from(self.modified).format(&Rfc3339)?,
        }))
    }
}

fn string<'a>(row: &'a Value, pointer: &str) -> &'a str {
    row.pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or_default()
}

fn flag(row: &Value, pointer: &str) -> bool {
    row.pointer(pointer)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn message_title(content: Option<&Value>) -> String {
    let Some(content) = content else {
        return String::new();
    };
    if let Some(text) = content.as_str() {
        return concise(text);
    }
    content
        .as_array()
        .into_iter()
        .flatten()
        .filter(|block| matches!(string(block, "/type"), "text" | "input_text"))
        .map(|block| concise(string(block, "/text")))
        .find(|title| !title.is_empty())
        .unwrap_or_default()
}

fn concise(text: &str) -> String {
    let text = text.trim_start();
    let generated = [
        "<environment_context>",
        "<user_instructions>",
        "<local-command-",
        "<system-reminder>",
        "# AGENTS.md instructions",
    ];
    if generated.iter().any(|prefix| text.starts_with(*prefix)) {
        return String::new();
    }
    text.lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .chars()
        .take(160)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_local_sessions_and_excludes_agent_generated_context() -> Result<()> {
        let home = tempfile::tempdir()?;
        let claude = home.path().join("claude");
        let codex = home.path().join("codex");
        let projects = claude.join("projects/example");
        let active = codex.join("sessions/2026/11/27");
        let archived = codex.join("archived_sessions");
        for path in [&projects, &active, &archived] {
            fs::create_dir_all(path)?;
        }
        fs::write(
            projects.join("example.jsonl"),
            concat!(
                "{\"type\":\"user\",\"sessionId\":\"example\",\"cwd\":\"/project\",\"isMeta\":true,\"message\":{\"content\":\"side input\"}}\n",
                "{\"type\":\"user\",\"sessionId\":\"example\",\"cwd\":\"/project\",\"message\":{\"content\":[{\"type\":\"text\",\"text\":\"Review the build\"}]}}\n",
            ),
        )?;
        fs::write(
            archived.join("rollout-old.jsonl"),
            concat!(
                "{\"type\":\"session_meta\",\"payload\":{\"id\":\"older\",\"cwd\":\"/project\"}}\n",
                "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"<environment_context>\"}]}}\n",
                "{\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"Verify the test\"}}\n",
            ),
        )?;
        fs::write(
            active.join("rollout-subagent.jsonl"),
            concat!(
                "{\"type\":\"session_meta\",\"payload\":{\"id\":\"child\",\"source\":{\"subagent\":{\"thread_spawn\":{}}}}}\n",
                "{\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"a delegated task\"}}\n",
            ),
        )?;
        let result = list_in([(Agent::Claude, claude), (Agent::Codex, codex)], 20)?;
        assert_eq!(result.sessions.len(), 2);
        assert_eq!(result.unreadable, 0);
        assert!(result.sessions.iter().any(|entry| entry.id == "example"
            && entry.title == "Review the build"
            && !entry.archived));
        assert!(result.sessions.iter().any(|entry| entry.id == "older"
            && entry.title == "Verify the test"
            && entry.archived));
        Ok(())
    }
}
