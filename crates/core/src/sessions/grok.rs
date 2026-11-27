use super::{
    MAX_EXCERPT_CHARS, MAX_EXCERPT_SCAN, MAX_HEADER, Messages, concise, decode_line, flag, string,
};
use crate::config::read_limited;
use anyhow::{Context, Result};
use nextjson::Value;
use std::{
    fs,
    io::{BufRead, BufReader, ErrorKind, Read},
    path::Path,
};

pub(super) struct Summary {
    pub id: String,
    pub cwd: String,
    pub title: String,
}

pub(super) fn summary(updates: &Path) -> Result<Option<Summary>> {
    let file = updates.with_file_name("summary.json");
    match fs::symlink_metadata(&file) {
        Ok(metadata) if !metadata.is_file() => return Ok(None),
        Ok(_) => (),
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    let bytes = read_limited(&file, MAX_HEADER as usize)?;
    let row = decode_line(&bytes).context("invalid Grok session summary")?;
    let id = string(&row, "/info/id");
    let folder = updates.parent().and_then(Path::file_name);
    if id.is_empty()
        || folder.and_then(|name| name.to_str()) != Some(id)
        || string(&row, "/session_kind").starts_with("subagent")
    {
        return Ok(None);
    }
    let mut title = concise(string(&row, "/generated_title"));
    if title.is_empty() {
        title = concise(string(&row, "/session_summary"));
    }
    Ok(Some(Summary {
        id: id.to_owned(),
        cwd: string(&row, "/info/cwd").to_owned(),
        title,
    }))
}

pub(super) fn excerpt(path: &Path) -> Result<Vec<String>> {
    let bytes =
        read_limited(path, MAX_EXCERPT_SCAN as usize).context("read Grok session updates")?;
    messages(bytes.as_slice())
}

pub(super) fn messages(source: impl Read) -> Result<Vec<String>> {
    let mut reader = BufReader::new(source);
    let mut bytes = Vec::new();
    let mut messages = Messages::default();
    let mut prompt = String::new();
    let mut index = None;
    while reader.read_until(b'\n', &mut bytes)? > 0 {
        let row = decode_line(&bytes);
        bytes.clear();
        let Some(row) = row else { continue };
        let Some(update) = row.pointer("/params/update") else {
            continue;
        };
        let kind = string(update, "/sessionUpdate");
        let user = kind == "user_message_chunk";
        let host = flag(update, "/_meta/hostTurn");
        let next_index = update.pointer("/_meta/promptIndex").and_then(Value::as_u64);
        if matches!(kind, "turn_completed" | "agent_message_chunk")
            || user && (host || next_index != index)
        {
            messages.push([prompt.as_str()]);
            prompt.clear();
            if messages.chars == MAX_EXCERPT_CHARS {
                break;
            }
        }
        if user && !host && string(update, "/content/type") == "text" {
            index = next_index;
            prompt.push_str(string(update, "/content/text"));
        }
    }
    messages.push([prompt.as_str()]);
    Ok(messages.values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{agents::NativeAgent, sessions::list_in};

    #[test]
    fn reads_coordinator_prompts_across_turns_and_excludes_generated_events() -> Result<()> {
        let home = tempfile::tempdir()?;
        let project = home.path().join("sessions/project");
        let main = project.join("main");
        let child = project.join("child");
        fs::create_dir_all(&main)?;
        fs::create_dir_all(&child)?;
        fs::write(
            main.join("summary.json"),
            br#"{"info":{"id":"main","cwd":"/project"}}"#,
        )?;
        fs::write(
            child.join("summary.json"),
            br#"{"info":{"id":"child","cwd":"/project"},"session_kind":"subagent:explore"}"#,
        )?;
        fs::write(child.join("updates.jsonl"), b"")?;
        let updates = main.join("updates.jsonl");
        fs::write(&updates, br#"{"params":{"update":{"sessionUpdate":"user_message_chunk","content":{"type":"text","text":"Keep "},"_meta":{"promptIndex":0}}}}
{"params":{"update":{"sessionUpdate":"user_message_chunk","content":{"type":"text","text":"both chunks"},"_meta":{"promptIndex":0}}}}
{"params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"Generated reply"}}}}
{"params":{"update":{"sessionUpdate":"turn_completed"}}}
{"params":{"update":{"sessionUpdate":"user_message_chunk","content":{"type":"text","text":"/compact"},"_meta":{"hostTurn":true}}}}
{"params":{"update":{"sessionUpdate":"user_message_chunk","content":{"type":"text","text":"Keep the resumed prompt"},"_meta":{"promptIndex":0}}}}
{"params":{"update":{"sessionUpdate":"tool_call","content":{"type":"text","text":"Tool output"}}}}
{"params":{"update":{"sessionUpdate":"turn_completed"}}}
"#)?;
        let listing = list_in([(NativeAgent::Grok, home.path().to_owned())], 50)?;
        assert_eq!(listing.unreadable, 0);
        assert_eq!(listing.sessions.len(), 1);
        assert_eq!(listing.sessions[0].id, "main");
        assert_eq!(listing.sessions[0].title, "Keep both chunks");
        assert_eq!(
            excerpt(&updates)?,
            ["Keep both chunks", "Keep the resumed prompt"]
        );
        Ok(())
    }
}
