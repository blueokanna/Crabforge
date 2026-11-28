mod dav;

use super::{
    backup::{Memory, Source, timestamp},
    filename, lock_store, markdown_text, parse, source, split, tags,
};
use crate::config::{self, LEGACY_FORMAT_PREFIX, MAX_BACKUP};
use crate::skills::tree::{is_link, windows_filename};
use anyhow::{Context, Result, ensure};
use dav::{Dav, Document};
use nextjson::{Map, NsonSerialize, Value, json};
use saphyr::{LoadableYamlNode, YamlOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

pub struct Options<'a> {
    pub url: &'a str,
    pub user: &'a str,
    pub password: &'a str,
    pub passphrase: &'a str,
    pub revision: Option<&'a str>,
}

#[derive(NsonSerialize)]
pub struct Report {
    revision: String,
    downloaded: usize,
    updated: usize,
    removed: usize,
    uploaded: usize,
    applied: bool,
}

struct Local {
    path: PathBuf,
    memory: Memory,
    revision: String,
}

struct Remote {
    memory: Memory,
    fields: Map,
}

struct WriteMemory {
    path: PathBuf,
    before: Option<String>,
    after: Option<Vec<u8>>,
}

/// Preview a two-way memory sync, or apply it while both snapshots match the preview.
pub fn synchronize(dir: &Path, options: Options<'_>) -> Result<Report> {
    ensure!(
        !options.passphrase.is_empty(),
        "enter the encrypted backup passphrase"
    );
    let dav = Dav::new(options.url, options.user, options.password)?;
    let folder = dir.join("memory");
    let _lock = options.revision.map(|_| lock_store(dir)).transpose()?;
    let document = dav.read()?;
    let (format, mut bundle) = match &document {
        Some(document) => config::decode_backup(&document.content, options.passphrase)
            .context("could not open the WebDAV backup")?,
        None => {
            let format = if dav.url.path().ends_with(".magpie-backup") {
                format!("{LEGACY_FORMAT_PREFIX}-backup")
            } else {
                "crabforge-backup".into()
            };
            let created = OffsetDateTime::now_utc().format(&Rfc3339)?;
            let bundle = BTreeMap::from([
                ("version".into(), json!(2)),
                ("created".into(), json!(created)),
                ("keys".into(), json!(false)),
                ("providers".into(), Value::Null),
            ]);
            (format, bundle)
        }
    };
    let locals = local_memories(&folder)?;
    let mut section = match bundle.remove("memory") {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(section)) => section,
        _ => anyhow::bail!("invalid WebDAV memory section"),
    };
    let removed: BTreeMap<String, String> = match section.get("removed") {
        None | Some(Value::Null) => BTreeMap::new(),
        Some(value) => nextjson::from_value(value.clone()).map_err(anyhow::Error::msg)?,
    };
    let mut tombstones = BTreeMap::new();
    let oldest_deletion = OffsetDateTime::now_utc() - Duration::days(180);
    for (id, time) in removed {
        sync_filename(&folder, &id)?;
        let time = timestamp(&time)?;
        if time < oldest_deletion {
            continue;
        }
        ensure!(
            tombstones.insert(id.to_lowercase(), (id, time)).is_none(),
            "case-conflicting WebDAV memory deletions"
        );
    }
    let revision = revision(&dav, document.as_ref(), &locals, tombstones.keys());
    ensure!(
        options.revision.is_none_or(|expected| expected == revision),
        "the local or WebDAV memory changed; preview the sync again"
    );
    let mut report = Report {
        revision,
        downloaded: 0,
        updated: 0,
        removed: 0,
        uploaded: 0,
        applied: options.revision.is_some(),
    };
    let remotes = remote_memories(&folder, section.remove("memories"))?;
    let ids = locals.keys().chain(remotes.keys()).collect::<BTreeSet<_>>();
    let mut writes = Vec::new();
    let mut upload = false;
    let mut merged = Vec::new();
    for id in ids {
        let local = locals.get(id);
        let remote = remotes.get(id);
        if let (Some(local), Some(remote)) = (local, remote) {
            ensure!(
                local.memory.id == remote.memory.id,
                "case-conflicting memory ID {}",
                local.memory.id
            );
        }
        let winner = match (local, remote) {
            (Some(local), Some(remote)) if !prefer_local(&local.memory, &remote.memory)? => {
                &remote.memory
            }
            (Some(local), _) => &local.memory,
            (None, Some(remote)) => &remote.memory,
            (None, None) => unreachable!("memory ID comes from the local or remote snapshot"),
        };
        if let Some((name, time)) = tombstones.get(id) {
            ensure!(
                &winner.id == name,
                "case-conflicting memory deletion {}",
                winner.id
            );
            if timestamp(&winner.updated)? <= *time {
                if let Some(local) = local {
                    writes.push(local.write(None));
                    report.removed += 1;
                }
                upload |= remote.is_some();
                continue;
            }
        }
        let markdown = winner.markdown()?;
        match local {
            None => {
                writes.push(WriteMemory {
                    path: sync_filename(&folder, &winner.id)?,
                    before: None,
                    after: Some(markdown),
                });
                report.downloaded += 1;
            }
            Some(local) if &local.memory != winner => {
                writes.push(local.write(Some(markdown)));
                report.updated += 1;
            }
            Some(_) => {}
        }
        let value = match remote {
            Some(remote) if &remote.memory == winner => Value::Object(remote.fields.clone()),
            remote => {
                upload = true;
                report.uploaded += 1;
                with_memory(remote.map(|note| note.fields.clone()), winner)?
            }
        };
        merged.push(value);
    }
    if report.applied {
        if upload {
            let removed = tombstones
                .into_values()
                .map(|(id, time)| Ok((id, time.format(&Rfc3339)?)))
                .collect::<Result<BTreeMap<_, _>>>()?;
            if removed.is_empty() {
                section.remove("removed");
            } else {
                section.insert(
                    "removed".into(),
                    nextjson::to_value(&removed).map_err(anyhow::Error::msg)?,
                );
            }
            section.insert("memories".into(), Value::Array(merged));
            bundle.insert("memory".into(), Value::Object(section));
            let bytes = config::encode_backup(&bundle, options.passphrase, format)?;
            dav.write(document.as_ref(), &bytes)?;
        }
        for write in writes {
            write.apply()?;
        }
    }
    Ok(report)
}

fn revision<'a>(
    dav: &Dav,
    document: Option<&Document>,
    locals: &BTreeMap<String, Local>,
    tombstones: impl Iterator<Item = &'a String>,
) -> String {
    let mut digest = Sha256::new();
    digest.update(dav.url.as_str());
    digest.update([0]);
    if let Some(document) = document {
        digest.update(Sha256::digest(&document.content));
        digest.update(document.etag.as_deref().unwrap_or_default());
    }
    digest.update([0]);
    for id in tombstones {
        digest.update(id);
        digest.update([0]);
    }
    digest.update([0]);
    for (id, local) in locals {
        digest.update(id);
        digest.update([0]);
        digest.update(&local.revision);
        digest.update(&local.memory.updated);
        digest.update([0]);
        digest.update(&local.memory.created);
    }
    hex::encode(digest.finalize())
}

fn local_memories(folder: &Path) -> Result<BTreeMap<String, Local>> {
    let metadata = match fs::symlink_metadata(folder) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error.into()),
    };
    ensure!(
        metadata.is_dir() && !is_link(&metadata),
        "expected a regular memory directory"
    );
    let mut notes = BTreeMap::new();
    let mut size = 0u64;
    for path in fs::read_dir(folder)? {
        let path = path?;
        let file = path.path();
        if file
            .extension()
            .is_none_or(|ext| !ext.eq_ignore_ascii_case("md"))
        {
            continue;
        }
        let id = file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .context("invalid local memory filename")?;
        sync_filename(folder, id)?;
        let metadata = path.metadata()?;
        ensure!(
            metadata.is_file() && !is_link(&metadata),
            "expected a regular memory file"
        );
        size = size.saturating_add(metadata.len());
        ensure!(
            size <= MAX_BACKUP as u64,
            "the local memory store exceeds the sync size limit"
        );
        let (text, modified, revision) =
            source(&file)?.context("local memory changed while reading")?;
        let memory = from_markdown(id, &text, &modified)?;
        let key = id.to_lowercase();
        ensure!(
            notes
                .insert(
                    key,
                    Local {
                        path: file,
                        memory,
                        revision
                    }
                )
                .is_none(),
            "case-conflicting local memory filenames"
        );
    }
    Ok(notes)
}

fn sync_filename(folder: &Path, id: &str) -> Result<PathBuf> {
    ensure!(windows_filename(id), "invalid portable memory ID");
    filename(folder, id)
}

fn from_markdown(id: &str, text: &str, modified: &str) -> Result<Memory> {
    let (front, body) = split(text)?;
    let entry = parse(id, front, body, modified, String::new())?;
    let documents = YamlOwned::load_from_str(front).map_err(anyhow::Error::msg)?;
    let field = |key: &str| documents.first().and_then(|doc| doc.as_mapping_get(key));
    let string = |key: &str| field(key).and_then(YamlOwned::as_str).map(str::to_owned);
    let mut memory = Memory {
        id: entry.id,
        text: entry.text,
        kind: entry.kind,
        scope: entry.scope,
        status: entry.status,
        origin: entry.origin,
        created: string("created").unwrap_or_else(|| entry.updated.clone()),
        updated: entry.updated,
        edited: string("edited"),
        why: Some(entry.why),
        tags: Some(entry.tags),
        pinned: Some(entry.pinned),
        superseded_by: string("supersededBy"),
        skill: string("skill"),
        merged_from: field("mergedFrom")
            .map(|value| {
                value
                    .as_vec()
                    .context("merged memory IDs must be a list")?
                    .iter()
                    .map(|id| {
                        id.as_str()
                            .map(str::to_owned)
                            .context("merged memory IDs must be strings")
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?,
        sources: field("sources").and_then(YamlOwned::as_vec).map(|entries| {
            entries
                .iter()
                .map(|item| {
                    let value = |key: &str| {
                        item.as_mapping_get(key)
                            .and_then(YamlOwned::as_str)
                            .unwrap_or("")
                            .to_owned()
                    };
                    Source {
                        agent: value("agent"),
                        session: value("session"),
                        dir: value("dir"),
                        file: value("file"),
                        quote: value("quote"),
                        at: value("at"),
                    }
                })
                .collect()
        }),
    };
    normalize(&mut memory)?;
    Ok(memory)
}

fn remote_memories(folder: &Path, value: Option<Value>) -> Result<BTreeMap<String, Remote>> {
    let values = match value {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(values)) => values,
        _ => anyhow::bail!("invalid WebDAV memory list"),
    };
    let mut notes = BTreeMap::new();
    for value in values {
        let mut memory: Memory = nextjson::from_value(value.clone())
            .map_err(anyhow::Error::msg)
            .context("invalid WebDAV memory entry")?;
        let Value::Object(fields) = value else {
            unreachable!("a memory entry is an object")
        };
        sync_filename(folder, &memory.id)?;
        normalize(&mut memory)?;
        ensure!(
            notes
                .insert(memory.id.to_lowercase(), Remote { memory, fields })
                .is_none(),
            "duplicate or case-conflicting WebDAV memory ID"
        );
    }
    Ok(notes)
}

fn normalize(memory: &mut Memory) -> Result<()> {
    memory.created = timestamp(&memory.created)?
        .to_offset(time::UtcOffset::UTC)
        .format(&Rfc3339)?;
    memory.updated = timestamp(&memory.updated)?
        .to_offset(time::UtcOffset::UTC)
        .format(&Rfc3339)?;
    if let Some(edited) = memory.edited.take().filter(|value| !value.is_empty()) {
        let edited = timestamp(&edited)?
            .to_offset(time::UtcOffset::UTC)
            .format(&Rfc3339)?;
        memory.edited = (edited != "0001-01-01T00:00:00Z").then_some(edited);
    }
    memory.text = markdown_text(&memory.text);
    memory.pinned = memory.pinned.filter(|&pinned| pinned);
    memory.why = memory
        .why
        .take()
        .map(|why| why.trim().into())
        .filter(|why: &String| !why.is_empty());
    memory.tags = memory.tags.take().map(tags).filter(|tags| !tags.is_empty());
    memory.skill = memory.skill.take().filter(|skill| !skill.is_empty());
    memory.superseded_by = memory.superseded_by.take().filter(|id| !id.is_empty());
    memory.sources = memory.sources.take().filter(|sources| !sources.is_empty());
    memory.merged_from = memory.merged_from.take().filter(|ids| !ids.is_empty());
    for source in memory.sources.iter_mut().flatten() {
        if !source.at.is_empty() {
            source.at = timestamp(&source.at)?
                .to_offset(time::UtcOffset::UTC)
                .format(&Rfc3339)?;
            if source.at == "0001-01-01T00:00:00Z" {
                source.at.clear();
            }
        }
    }
    Ok(())
}

fn prefer_local(local: &Memory, remote: &Memory) -> Result<bool> {
    let local_time = timestamp(&local.updated)?;
    let remote_time = timestamp(&remote.updated)?;
    let newer_is_local = if local_time == remote_time {
        nextjson::to_vec(local).map_err(anyhow::Error::msg)?
            > nextjson::to_vec(remote).map_err(anyhow::Error::msg)?
    } else {
        local_time > remote_time
    };
    let (newer, older) = if newer_is_local {
        (local, remote)
    } else {
        (remote, local)
    };
    let fixed = |note: &Memory| note.pinned == Some(true) || note.origin == "user";
    let user_edit = newer.edited.as_ref() == Some(&newer.updated);
    Ok(if !user_edit && fixed(older) && !fixed(newer) {
        !newer_is_local
    } else {
        newer_is_local
    })
}

fn with_memory(existing: Option<Map>, memory: &Memory) -> Result<Value> {
    let mut fields = existing.unwrap_or_default();
    let value = nextjson::to_value(memory).map_err(anyhow::Error::msg)?;
    let Value::Object(memory) = value else {
        unreachable!("a memory entry is an object")
    };
    for (key, value) in memory {
        fields.insert(key, value);
    }
    Ok(Value::Object(fields))
}

impl Local {
    fn write(&self, after: Option<Vec<u8>>) -> WriteMemory {
        WriteMemory {
            path: self.path.clone(),
            before: Some(self.revision.clone()),
            after,
        }
    }
}

impl WriteMemory {
    fn apply(self) -> Result<()> {
        let current = source(&self.path)?.map(|(_, _, revision)| revision);
        ensure!(
            current == self.before,
            "a local memory changed; preview the sync again"
        );
        match (&self.before, self.after) {
            (_, None) => fs::remove_file(self.path)?,
            (Some(_), Some(bytes)) => config::replace_contents(
                &self.path,
                &bytes,
                Some(fs::metadata(&self.path)?.permissions()),
            )?,
            (None, Some(bytes)) => {
                let mut file = tempfile::NamedTempFile::new_in(
                    self.path.parent().context("memory path has no parent")?,
                )?;
                file.write_all(&bytes)?;
                file.as_file().sync_all()?;
                file.persist_noclobber(self.path)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabforge_test_support::server::{Incoming, Router, any, serve};
    use http::{HeaderMap, StatusCode, header};
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn sync_keeps_tidy_provenance_and_ignores_expired_deletions() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let folder = temp.path().join("memory");
        fs::create_dir(&folder)?;
        let date = OffsetDateTime::now_utc();
        let oldest = (date - Duration::days(200)).format(&Rfc3339)?;
        let deleted = (date - Duration::days(190)).format(&Rfc3339)?;
        let body = format!(
            "---\ncreated: \"{oldest}\"\nupdated: \"{oldest}\"\nmergedFrom: [m_first, m_second]\n---\n\nAn older memory.\n"
        );
        let file = folder.join("m_combined.md");
        fs::write(&file, &body)?;
        let bundle = BTreeMap::from([
            ("version".into(), json!(2)),
            (
                "memory".into(),
                json!({"removed": {"m_combined": deleted}, "declined": ["fixture"]}),
            ),
        ]);
        let initial = config::encode_backup(&bundle, "fixture key", "crabforge-backup".into())?;
        let remote = Arc::new(Mutex::new(initial));
        let app = Router::new().route(
            "/dav/crabforge.crabforge-backup",
            any({
                let remote = remote.clone();
                move |Incoming { headers, body, .. }| {
                    let remote = remote.clone();
                    async move {
                        let mut remote = remote.lock().unwrap();
                        if body.is_empty() {
                            let mut headers = HeaderMap::new();
                            headers.insert(header::ETAG, r#""fixture""#.parse().unwrap());
                            (StatusCode::OK, headers, remote.clone())
                        } else {
                            assert_eq!(headers[header::IF_MATCH], r#""fixture""#);
                            *remote = body.to_vec();
                            (StatusCode::NO_CONTENT, HeaderMap::new(), Vec::new())
                        }
                    }
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}/dav/", listener.local_addr()?);
        let server = tokio::spawn(serve(listener, app));
        let dir = temp.path().to_owned();
        let report = tokio::task::spawn_blocking(move || -> Result<Report> {
            let options = Options {
                url: &url,
                user: "",
                password: "",
                passphrase: "fixture key",
                revision: None,
            };
            let preview = synchronize(&dir, options)?;
            synchronize(
                &dir,
                Options {
                    url: &url,
                    user: "",
                    password: "",
                    passphrase: "fixture key",
                    revision: Some(&preview.revision),
                },
            )
        })
        .await??;
        server.abort();
        assert!(report.applied);
        assert_eq!((report.removed, report.uploaded), (0, 1));
        assert_eq!(fs::read_to_string(file)?, body);
        let (_, stored) = config::decode_backup(&remote.lock().unwrap(), "fixture key")?;
        let section = &stored["memory"];
        assert!(section.get("removed").is_none());
        assert_eq!(section["declined"], json!(["fixture"]));
        assert_eq!(
            section["memories"][0]["mergedFrom"],
            json!(["m_first", "m_second"])
        );
        Ok(())
    }
}
