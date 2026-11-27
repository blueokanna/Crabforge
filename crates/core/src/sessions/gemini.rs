use super::{MAX_EXCERPT_SCAN, Messages, decode_line, flag, string};
use crate::config::read_limited;
use anyhow::{Context, Result, ensure};
use nextjson::Value;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::Read,
    path::Path,
};

pub(super) struct Conversation {
    pub id: String,
    pub messages: Vec<String>,
}

pub(super) fn working_folder(chat: &Path) -> String {
    let Some(project) = chat.parent().and_then(Path::parent) else {
        return String::new();
    };
    let marker = project.join(".project_root");
    if !fs::symlink_metadata(&marker).is_ok_and(|metadata| metadata.is_file()) {
        return String::new();
    }
    read_limited(&marker, 32 * 1024)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .map(|folder| folder.trim_end_matches(['\r', '\n']).to_owned())
        .filter(|folder| Path::new(folder).is_absolute())
        .unwrap_or_default()
}

pub(super) fn read(source: impl Read) -> Result<Option<Conversation>> {
    let mut bytes = Vec::new();
    source.take(MAX_EXCERPT_SCAN + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAX_EXCERPT_SCAN,
        "Gemini session exceeds the preview size limit"
    );
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes);
    let mut lines = bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.iter().all(u8::is_ascii_whitespace));
    let Some(first) = lines.next() else {
        return Ok(None);
    };
    let (metadata, rows) = match decode_line(first) {
        Some(metadata) => (metadata, Some(lines)),
        None => (decode_line(bytes).context("invalid Gemini session")?, None),
    };
    let id = string(&metadata, "/sessionId");
    if id.is_empty()
        || string(&metadata, "/projectHash").is_empty()
        || string(&metadata, "/kind") == "subagent"
    {
        return Ok(None);
    }
    let mut history = History::default();
    history.replace(metadata.pointer("/messages"));
    if let Some(rows) = rows {
        for line in rows {
            let entry = decode_line(line).context("invalid Gemini session entry")?;
            if let Some(rewind) = entry.pointer("/$rewindTo").and_then(Value::as_str) {
                history.rewind(rewind);
            } else if let Some(patch) = entry.pointer("/$patch") {
                history.patch(patch);
            } else if let Some(update) = entry.pointer("/$set") {
                if string(update, "/kind") == "subagent" {
                    return Ok(None);
                }
                history.replace(update.pointer("/messages"));
            } else {
                history.push(&entry);
            }
        }
    }
    let mut messages = Messages::default();
    for record in history.records.into_values() {
        messages.push(record.parts.iter().map(String::as_str));
    }
    Ok((!messages.values.is_empty()).then(|| Conversation {
        id: id.to_owned(),
        messages: messages.values,
    }))
}

#[derive(Default)]
struct History {
    positions: HashMap<String, usize>,
    records: BTreeMap<usize, Record>,
    next: usize,
}

struct Record {
    id: String,
    parts: Vec<String>,
}

impl History {
    fn push(&mut self, entry: &Value) {
        let id = string(entry, "/id");
        if id.is_empty() {
            return;
        }
        let record = Record {
            id: id.to_owned(),
            parts: user_parts(entry),
        };
        if let Some(&position) = self.positions.get(id) {
            self.records.insert(position, record);
        } else {
            self.append(record);
        }
    }

    fn append(&mut self, record: Record) {
        self.positions.insert(record.id.clone(), self.next);
        self.records.insert(self.next, record);
        self.next += 1;
    }

    fn remove(&mut self, id: &str) -> Option<Record> {
        let position = self.positions.remove(id)?;
        self.records.remove(&position)
    }

    fn replace(&mut self, messages: Option<&Value>) {
        if let Some(messages) = messages.and_then(Value::as_array) {
            *self = Self::default();
            for message in messages {
                self.push(message);
            }
        }
    }

    fn rewind(&mut self, id: &str) {
        if let Some(&position) = self.positions.get(id) {
            for record in self.records.split_off(&position).into_values() {
                self.positions.remove(&record.id);
            }
        } else {
            *self = Self::default();
        }
    }

    fn patch(&mut self, patch: &Value) {
        for update in std::iter::once(patch).chain(items(patch.pointer("/updates"))) {
            if update.pointer("/content").is_some()
                && let Some(position) = self.positions.get(string(update, "/id"))
                && let Some(record) = self.records.get_mut(position)
            {
                // A replacement can contain generated context or redact the original text.
                record.parts.clear();
            }
        }
        for id in items(patch.pointer("/removeIds")).filter_map(Value::as_str) {
            self.remove(id);
        }
        let reordered: Vec<_> = items(patch.pointer("/orderIds"))
            .filter_map(Value::as_str)
            .filter_map(|id| self.remove(id))
            .collect();
        for record in reordered {
            self.append(record);
        }
    }
}

fn items(value: Option<&Value>) -> impl Iterator<Item = &Value> {
    value.and_then(Value::as_array).into_iter().flatten()
}

fn user_parts(entry: &Value) -> Vec<String> {
    if string(entry, "/type") != "user" {
        return Vec::new();
    }
    let content = entry
        .pointer("/displayContent")
        .filter(|value| !value.is_null())
        .or_else(|| entry.pointer("/content"));
    if items(entry.pointer("/content")).any(|part| {
        part.pointer("/functionResponse")
            .is_some_and(|value| !value.is_null())
    }) {
        return Vec::new();
    }
    content
        .and_then(Value::as_str)
        .into_iter()
        .chain(
            items(content)
                .filter(|part| !flag(part, "/thought"))
                .map(|part| part.as_str().unwrap_or_else(|| string(part, "/text"))),
        )
        .filter(|part| {
            let part = part.trim_start();
            !part.starts_with(['/', '?'])
                && ![
                    "<session_context>",
                    "<hook_context>",
                    "<state_snapshot>",
                    "<untrusted_context",
                ]
                .iter()
                .any(|prefix| part.starts_with(prefix))
        })
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_display_text_from_the_current_native_history() -> Result<()> {
        let log = concat!(
            "{\"sessionId\":\"fixture\",\"projectHash\":\"hash\",\"kind\":\"main\"}\n",
            "{\"id\":\"last\",\"type\":\"user\",\"content\":\"Last in current order\"}\n",
            "{\"id\":\"edited\",\"type\":\"user\",\"content\":\"A subsequently edited prompt\"}\n",
            "{\"id\":\"deleted\",\"type\":\"user\",\"content\":\"Removed by a patch\"}\n",
            "{\"id\":\"first\",\"type\":\"user\",\"content\":[{\"text\":\"Expanded file content\"}],\"displayContent\":[{\"text\":\"Read @test.txt\"}]}\n",
            "{\"id\":\"reply\",\"type\":\"gemini\",\"content\":\"Assistant reply\"}\n",
            "{\"id\":\"rewound\",\"type\":\"user\",\"content\":\"Abandoned prompt\"}\n",
            "{\"$rewindTo\":\"reply\"}\n",
            "{\"id\":\"tool-result\",\"type\":\"user\",\"content\":[{\"functionResponse\":{\"name\":\"read\"}},{\"text\":\"Generated tool output\"}]}\n",
            "{\"id\":\"context\",\"type\":\"user\",\"content\":\"<hook_context>Generated by a hook</hook_context>\"}\n",
            "{\"id\":\"snapshot\",\"type\":\"user\",\"content\":[{\"text\":\"<state_snapshot>Generated by compression</state_snapshot>\"}]}\n",
            "{\"$patch\":{\"updates\":[{\"id\":\"edited\",\"content\":\"Synthesized replacement\"}],\"removeIds\":[\"deleted\"],\"orderIds\":[\"last\"]}}\n",
        );
        let conversation = read(log.as_bytes())?.context("expected Gemini session")?;
        assert_eq!(conversation.id, "fixture");
        assert_eq!(
            conversation.messages,
            ["Read @test.txt", "Last in current order"]
        );
        Ok(())
    }

    #[test]
    fn reads_legacy_documents_and_ignores_subagents() -> Result<()> {
        let legacy = r#"{
            "sessionId": "legacy",
            "projectHash": "project",
            "messages": [
                {"id":"one","type":"user","content":"Review the build"},
                {"id":"two","type":"gemini","content":"Assistant reply"}
            ]
        }"#;
        let conversation = read(legacy.as_bytes())?.context("expected legacy session")?;
        assert_eq!(conversation.id, "legacy");
        assert_eq!(conversation.messages, ["Review the build"]);
        let subagent = legacy.replace("\"messages\"", "\"kind\": \"subagent\", \"messages\"");
        assert!(read(subagent.as_bytes())?.is_none());
        Ok(())
    }
}
