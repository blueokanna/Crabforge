use super::{MAX_EXCERPT_SCAN, Messages, decode_line, message_parts, string};
use anyhow::{Context, Result, ensure};
use nextjson::Value;
use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader, Read},
};

pub(super) fn version(row: &Value) -> Option<u64> {
    if string(row, "/type") != "session" {
        return None;
    }
    match row.pointer("/version") {
        None => Some(1),
        Some(value) => value.as_u64().filter(|version| (1..=3).contains(version)),
    }
}

pub(super) fn user_message(row: &Value) -> bool {
    string(row, "/type") == "message" && string(row, "/message/role") == "user"
}

struct Entry {
    parent: Option<usize>,
    content: Content,
}

enum Content {
    User { id: String, parts: Vec<String> },
    Edit(String),
    Other,
}

pub(super) fn read_excerpt(source: impl Read) -> Result<Vec<String>> {
    let mut reader = BufReader::new(source.take(MAX_EXCERPT_SCAN + 1));
    let mut line = Vec::new();
    let mut total = reader.read_until(b'\n', &mut line)? as u64;
    let version = decode_line(&line)
        .as_ref()
        .and_then(version)
        .context("invalid Pi session header")?;
    let mut entries: Vec<Entry> = Vec::new();
    let mut positions = HashMap::new();
    loop {
        line.clear();
        let count = reader.read_until(b'\n', &mut line)? as u64;
        total += count;
        ensure!(
            total <= MAX_EXCERPT_SCAN,
            "Pi session exceeds the preview size limit"
        );
        if count == 0 {
            break;
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let row = decode_line(&line).context("invalid Pi session entry")?;
        let id = string(&row, "/id");
        let parent = if version == 1 {
            entries.len().checked_sub(1)
        } else {
            let parent = match row.pointer("/parentId") {
                Some(Value::Null) => None,
                Some(Value::String(parent)) => Some(
                    *positions
                        .get(parent.as_str())
                        .context("missing Pi session parent entry")?,
                ),
                _ => anyhow::bail!("invalid Pi session parent entry"),
            };
            ensure!(
                !id.is_empty() && positions.insert(id.to_owned(), entries.len()).is_none(),
                "invalid or duplicate Pi session entry ID"
            );
            parent
        };
        let content = if user_message(&row) {
            Content::User {
                id: id.to_owned(),
                parts: message_parts(row.pointer("/message/content"))
                    .map(str::to_owned)
                    .collect(),
            }
        } else if string(&row, "/type") == "context_edit" {
            Content::Edit(string(&row, "/targetId").to_owned())
        } else {
            Content::Other
        };
        entries.push(Entry { parent, content });
    }

    let mut edited = HashSet::new();
    let mut user_messages = Vec::new();
    let mut active = entries.last();
    while let Some(entry) = active {
        match &entry.content {
            Content::Edit(target) => {
                edited.insert(target);
            }
            Content::User { id, parts } if !edited.contains(id) => user_messages.push(parts),
            _ => (),
        }
        active = entry.parent.map(|parent| &entries[parent]);
    }
    let mut messages = Messages::default();
    for parts in user_messages.into_iter().rev() {
        messages.push(parts.iter().map(String::as_str));
    }
    Ok(messages.values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_only_original_user_messages_on_the_active_branch() -> Result<()> {
        let log = concat!(
            "{\"type\":\"session\",\"version\":3,\"id\":\"pi-session\",\"cwd\":\"/project\"}\n",
            "{\"type\":\"message\",\"id\":\"first\",\"parentId\":null,\"message\":{\"role\":\"user\",\"content\":\"Keep this first message\"}}\n",
            "{\"type\":\"message\",\"id\":\"abandoned\",\"parentId\":\"first\",\"message\":{\"role\":\"user\",\"content\":\"Abandoned branch\"}}\n",
            "{\"type\":\"branch_summary\",\"id\":\"summary\",\"parentId\":\"first\",\"summary\":\"Generated summary\"}\n",
            "{\"type\":\"message\",\"id\":\"original\",\"parentId\":\"summary\",\"message\":{\"role\":\"user\",\"content\":\"Edited prompt\"}}\n",
            "{\"type\":\"context_edit\",\"id\":\"edit\",\"parentId\":\"original\",\"targetId\":\"original\",\"replacement\":{\"content\":\"Replacement\"}}\n",
            "{\"type\":\"message\",\"id\":\"last\",\"parentId\":\"edit\",\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"text\",\"text\":\"Keep this last message\"},{\"type\":\"image\",\"data\":\"abc\"}]}}\n",
            "{\"type\":\"message\",\"id\":\"reply\",\"parentId\":\"last\",\"message\":{\"role\":\"assistant\",\"content\":\"Assistant reply\"}}\n",
        );
        assert_eq!(
            read_excerpt(log.as_bytes())?,
            ["Keep this first message", "Keep this last message"]
        );
        Ok(())
    }

    #[test]
    fn reads_legacy_linear_sessions() -> Result<()> {
        let log = concat!(
            "{\"type\":\"session\",\"id\":\"legacy-session\",\"cwd\":\"/project\"}\n",
            "{\"type\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"First user prompt\"}}\n",
            "{\"type\":\"message\",\"message\":{\"role\":\"hookMessage\",\"content\":\"Extension output\"}}\n",
            "{\"type\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"Second user prompt\"}}\n",
        );
        assert_eq!(
            read_excerpt(log.as_bytes())?,
            ["First user prompt", "Second user prompt"]
        );
        Ok(())
    }
}
