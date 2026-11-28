use super::{MAX_FILE, filename, lock_store, markdown_text, parse, tags};
use crate::config::{open_import_backup, read_limited};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use yaml_edit::{Mapping, Sequence, YamlFile};

#[derive(NsonDeserialize)]
struct Bundle {
    memories: Option<Vec<Memory>>,
    removed: Option<BTreeMap<String, String>>,
    #[njson(flatten)]
    other: BTreeMap<String, Value>,
}

#[derive(NsonDeserialize)]
#[njson(rename_all = "camelCase")]
struct Memory {
    id: String,
    text: String,
    kind: String,
    scope: String,
    status: String,
    origin: String,
    created: String,
    updated: String,
    edited: Option<String>,
    why: Option<String>,
    tags: Option<Vec<String>>,
    pinned: Option<bool>,
    sources: Option<Vec<Source>>,
    superseded_by: Option<String>,
    skill: Option<String>,
}

#[derive(Default, NsonDeserialize)]
#[njson(default)]
struct Source {
    agent: String,
    session: String,
    dir: String,
    file: String,
    quote: String,
    at: String,
}

#[derive(NsonSerialize)]
pub struct ImportReport {
    memories: Vec<PlannedMemory>,
    removed: Vec<String>,
    skipped_fields: Vec<String>,
    applied: bool,
}

#[derive(NsonSerialize)]
struct PlannedMemory {
    id: String,
    status: String,
    scope: String,
    action: &'static str,
}

/// Restore local Markdown entries from the memory section of an encrypted backup.
pub fn import_backup(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    apply: bool,
) -> Result<ImportReport> {
    let mut backup = open_import_backup(file, passphrase)?;
    let bundle: Bundle = nextjson::from_value(
        backup
            .remove("memory")
            .filter(|value| !value.is_null())
            .context("encrypted backup has no memory section")?,
    )
    .map_err(|_| anyhow::anyhow!("invalid backup memory section"))?;
    let removed = bundle.removed.unwrap_or_default();
    let mut report = ImportReport {
        memories: Vec::new(),
        removed: Vec::new(),
        skipped_fields: bundle.other.into_keys().collect(),
        applied: apply,
    };
    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    let folder = dir.join("memory");
    for memory in bundle.memories.unwrap_or_default() {
        let path = filename(&folder, &memory.id)?;
        ensure!(
            seen.insert(memory.id.to_lowercase()),
            "duplicate backup memory ID {:?}",
            memory.id
        );
        if let Some(deleted) = removed.get(&memory.id)
            && timestamp(&memory.updated)? <= timestamp(deleted)?
        {
            report.removed.push(memory.id);
            continue;
        }
        let contents = memory.markdown()?;
        entries.push((memory, path, contents));
    }

    let _lock = if apply && !entries.is_empty() {
        Some(lock_store(dir)?.1)
    } else {
        None
    };
    let mut create = Vec::new();
    for (memory, path, contents) in entries {
        let action = match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == ErrorKind::NotFound => {
                create.push((path, contents));
                "create"
            }
            Ok(metadata) if metadata.is_file() && metadata.len() <= MAX_FILE as u64 => {
                let current = read_limited(&path, MAX_FILE)
                    .with_context(|| format!("cannot read memory {}", memory.id))?;
                if current == contents {
                    "identical"
                } else {
                    "conflict"
                }
            }
            Ok(_) => "conflict",
            Err(error) => {
                return Err(error).with_context(|| format!("cannot read memory {}", memory.id));
            }
        };
        report.memories.push(PlannedMemory {
            id: memory.id,
            status: memory.status,
            scope: memory.scope,
            action,
        });
    }
    if apply {
        if let Some(conflict) = report
            .memories
            .iter()
            .find(|item| item.action == "conflict")
        {
            anyhow::bail!("memory ID already in use: {}", conflict.id);
        }
        write_new(&folder, create)?;
    }
    Ok(report)
}

fn timestamp(value: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).context("invalid backup memory timestamp")
}

fn write_new(folder: &Path, create: Vec<(PathBuf, Vec<u8>)>) -> Result<()> {
    let mut staged = Vec::new();
    for (path, contents) in create {
        let mut temp = tempfile::NamedTempFile::new_in(folder)?;
        temp.write_all(&contents)?;
        temp.as_file().sync_all()?;
        staged.push((path, temp.into_temp_path()));
    }
    for (path, temp) in staged {
        temp.persist_noclobber(path)?;
    }
    Ok(())
}

impl Memory {
    fn markdown(&self) -> Result<Vec<u8>> {
        timestamp(&self.created)?;
        timestamp(&self.updated)?;
        let file: YamlFile = "".parse()?;
        let doc = file.ensure_document();
        for (key, value) in [
            ("kind", Some(self.kind.as_str())),
            ("scope", Some(self.scope.as_str())),
            ("status", Some(self.status.as_str())),
            ("origin", Some(self.origin.as_str())),
            ("created", Some(self.created.as_str())),
            ("updated", Some(self.updated.as_str())),
            ("edited", self.edited.as_deref()),
            ("why", self.why.as_deref()),
            ("supersededBy", self.superseded_by.as_deref()),
            ("skill", self.skill.as_deref()),
        ] {
            if let Some(value) = value {
                ensure!(doc.set(key, value), "cannot write memory field {key}");
            }
        }
        if let Some(pinned) = self.pinned {
            ensure!(doc.set("pinned", pinned), "cannot write memory pinned flag");
        }
        if let Some(values) = &self.tags {
            let list = Sequence::new();
            for value in tags(values.iter().cloned()) {
                list.push(value.as_str());
            }
            ensure!(doc.set("tags", list), "cannot write memory tags");
        }
        if let Some(sources) = &self.sources {
            let list = Sequence::new();
            for source in sources {
                let item = Mapping::new();
                for (key, value) in [
                    ("agent", &source.agent),
                    ("session", &source.session),
                    ("dir", &source.dir),
                    ("file", &source.file),
                    ("quote", &source.quote),
                    ("at", &source.at),
                ] {
                    if !value.is_empty() {
                        item.set(key, value.as_str());
                    }
                }
                list.push(item);
            }
            ensure!(doc.set("sources", list), "cannot write memory sources");
        }
        let yaml = file.to_string();
        let front = yaml
            .strip_prefix("---\n")
            .unwrap_or(&yaml)
            .trim_end_matches('\n');
        let body = markdown_text(&self.text);
        parse(&self.id, front, &body, &self.updated, String::new())?;
        let contents = format!("---\n{front}\n---\n\n{body}\n");
        ensure!(
            contents.len() <= MAX_FILE,
            "memory file exceeds its size limit"
        );
        Ok(contents.into_bytes())
    }
}
