use super::{
    MAX_EXCERPT_SCAN, Messages, concise, decode_line, message_parts, message_title, string,
};
use anyhow::{Context, Result, ensure};
use nextjson::Value;
use std::{
    collections::{HashMap, HashSet},
    env,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

pub(super) fn is_shared_root(path: &Path) -> bool {
    env::var_os("PI_CODING_AGENT_SESSION_DIR").is_some_and(|base| path == Path::new(&base))
        || env::var_os("PI_CODING_AGENT_DIR")
            .is_some_and(|base| path == PathBuf::from(base).join("sessions"))
}

fn version(row: &Value) -> Option<u64> {
    if string(row, "/type") != "session" {
        return None;
    }
    match row.pointer("/version") {
        None => Some(1),
        Some(value) => value.as_u64().filter(|version| (1..=3).contains(version)),
    }
}

fn user_message(row: &Value) -> bool {
    string(row, "/type") == "message" && string(row, "/message/role") == "user"
}

fn title_slot(row: &Value) -> bool {
    string(row, "/type") == "title"
        && row.get("v").and_then(Value::as_u64) == Some(1)
        && ["title", "updatedAt", "pad"]
            .iter()
            .all(|key| row.get(key).and_then(Value::as_str).is_some())
}

pub(super) fn read_header(
    source: impl Read,
    omp: bool,
    shared: bool,
) -> Result<Option<(String, String, String)>> {
    let mut reader = BufReader::new(source);
    let mut line = Vec::new();
    let mut next = || -> Result<Option<Value>> {
        loop {
            line.clear();
            if reader.read_until(b'\n', &mut line)? == 0 {
                return Ok(None);
            }
            if let Some(row) = decode_line(&line) {
                return Ok(Some(row));
            }
        }
    };
    let Some(first) = next()? else {
        return Ok(None);
    };
    let slot = title_slot(&first);
    if (slot && !omp) || (shared && !slot) {
        return Ok(None);
    }
    let slot_title = if slot {
        concise(string(&first, "/title"))
    } else {
        String::new()
    };
    let header = if slot { next()? } else { Some(first) };
    let Some(header) = header.filter(|row| version(row).is_some()) else {
        return Ok(None);
    };
    let id = string(&header, "/id");
    let cwd = string(&header, "/cwd");
    if id.is_empty() || cwd.is_empty() {
        return Ok(None);
    }
    let mut title = slot_title;
    if omp && title.is_empty() {
        title = concise(string(&header, "/title"));
    }
    let mut prompt = String::new();
    while (title.is_empty() || (omp && !slot))
        && let Some(row) = next()?
    {
        if omp && !slot && string(&row, "/type") == "title_change" {
            title = concise(string(&row, "/title"));
        }
        if prompt.is_empty() && user_message(&row) {
            prompt = message_title(message_parts(row.pointer("/message/content")));
            if !omp || slot {
                break;
            }
        }
    }
    if title.is_empty() {
        title = prompt;
    }
    Ok((!title.is_empty()).then(|| (id.to_owned(), cwd.to_owned(), title)))
}

struct Entry {
    parent: Option<usize>,
    content: Content,
}

enum Content {
    User { id: String, parts: Vec<String> },
    Edit(String),
    Reset,
    Other,
}

pub(super) fn read_excerpt(source: impl Read, omp: bool) -> Result<Vec<String>> {
    let mut reader = BufReader::new(source.take(MAX_EXCERPT_SCAN + 1));
    let mut line = Vec::new();
    let mut total = reader.read_until(b'\n', &mut line)? as u64;
    let mut header = decode_line(&line).context("invalid session header")?;
    if omp && title_slot(&header) {
        line.clear();
        total += reader.read_until(b'\n', &mut line)? as u64;
        header = decode_line(&line).context("invalid Oh My Pi session header")?;
    }
    ensure!(
        total <= MAX_EXCERPT_SCAN,
        "session exceeds the preview size limit"
    );
    let version = version(&header).context("unsupported session version")?;
    let mut entries: Vec<Entry> = Vec::new();
    let mut positions = HashMap::new();
    let mut leaf = None;
    let mut last_self_selected = None;
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
        } else if omp && string(&row, "/type") == "reset_boundary" {
            Content::Reset
        } else {
            Content::Other
        };
        leaf = match omp.then_some(row.get("activeLeafId")).flatten() {
            Some(Value::Null) => Some(None),
            Some(Value::String(active)) => Some(Some(active.to_owned())),
            _ => None,
        };
        if leaf.is_none() || leaf.as_ref().and_then(|id| id.as_deref()) == Some(id) {
            last_self_selected = Some(entries.len());
        }
        entries.push(Entry { parent, content });
    }

    let mut edited = HashSet::new();
    let mut user_messages = Vec::new();
    let selected = match leaf.as_ref() {
        Some(None) => None,
        Some(Some(id)) => positions.get(id).copied().or(last_self_selected),
        None => entries.len().checked_sub(1),
    };
    let mut active = selected.map(|index| &entries[index]);
    while let Some(entry) = active {
        match &entry.content {
            Content::Edit(target) => {
                edited.insert(target);
            }
            Content::User { id, parts } if !edited.contains(id) => user_messages.push(parts),
            Content::Reset => break,
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
            read_excerpt(log.as_bytes(), false)?,
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
            read_excerpt(log.as_bytes(), false)?,
            ["First user prompt", "Second user prompt"]
        );
        Ok(())
    }

    #[test]
    fn reads_omp_title_slot_and_replays_the_selected_leaf() -> Result<()> {
        let log = concat!(
            "{\"type\":\"title\",\"v\":1,\"title\":\"Project review\",\"updatedAt\":\"\",\"pad\":\"\"}\n",
            "{\"type\":\"session\",\"version\":3,\"id\":\"omp-session\",\"cwd\":\"/project\"}\n",
            "{\"type\":\"message\",\"id\":\"old\",\"parentId\":null,\"message\":{\"role\":\"user\",\"content\":\"Cleared input\"}}\n",
            "{\"type\":\"reset_boundary\",\"id\":\"reset\",\"parentId\":\"old\"}\n",
            "{\"type\":\"message\",\"id\":\"active\",\"parentId\":\"reset\",\"message\":{\"role\":\"user\",\"content\":\"Verify the workspace\"}}\n",
            "{\"type\":\"message\",\"id\":\"hidden\",\"parentId\":\"reset\",\"message\":{\"role\":\"user\",\"content\":\"Unused branch\"}}\n",
            "{\"type\":\"model_usage\",\"id\":\"usage\",\"parentId\":\"hidden\",\"activeLeafId\":\"active\"}\n",
        );
        assert!(read_header(log.as_bytes(), false, false)?.is_none());
        let (id, cwd, title) = read_header(log.as_bytes(), true, true)?.unwrap();
        assert_eq!(
            (id.as_str(), cwd.as_str(), title.as_str()),
            ("omp-session", "/project", "Project review")
        );
        assert_eq!(
            read_excerpt(log.as_bytes(), true)?,
            ["Verify the workspace"]
        );

        let legacy = concat!(
            "{\"type\":\"session\",\"id\":\"legacy\",\"cwd\":\"/work\",\"title\":\"Original title\"}\n",
            "{\"type\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"First turn\"}}\n",
            "{\"type\":\"title_change\",\"title\":\"Revised title\"}\n",
        );
        assert!(read_header(legacy.as_bytes(), true, true)?.is_none());
        assert_eq!(
            read_header(legacy.as_bytes(), true, false)?.unwrap().2,
            "Revised title"
        );
        assert_eq!(read_excerpt(legacy.as_bytes(), true)?, ["First turn"]);
        Ok(())
    }
}
