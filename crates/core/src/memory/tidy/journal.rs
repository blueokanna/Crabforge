use super::{Plan, fixed};
use crate::{
    config,
    memory::{Entry, KINDS, MAX_FILE, filename, parse, split},
};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use saphyr::{LoadableYamlNode, YamlOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::{ErrorKind, Write},
    path::Path,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use yaml_edit::{Mapping, Sequence, YamlFile};

const REVIEW_AGE: i64 = 30 * 24 * 60 * 60;

#[derive(PartialEq, NsonDeserialize, NsonSerialize)]
#[njson(rename_all = "snake_case")]
pub(super) enum Phase {
    Applying,
    Applied,
    Undoing,
    Undone,
}

impl Phase {
    pub(super) fn pending(&self) -> bool {
        matches!(self, Self::Applying | Self::Undoing)
    }
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Change {
    id: String,
    before: Option<String>,
    after: String,
    #[njson(default)]
    applied: bool,
}

impl Change {
    fn writes(&self) -> bool {
        self.before.as_deref() != Some(&self.after)
    }
}

#[derive(NsonDeserialize, NsonSerialize)]
pub(super) struct Journal {
    pub(super) phase: Phase,
    groups: Vec<Vec<Change>>,
}

impl Journal {
    #[cfg(test)]
    pub(super) fn write_first_change(&self, folder: &Path) -> Result<()> {
        let change = self
            .groups
            .first()
            .and_then(|group| group.first())
            .context("no change")?;
        ensure!(
            replace(
                folder,
                &change.id,
                change.before.as_deref(),
                Some(&change.after)
            )?,
            "change was not written"
        );
        Ok(())
    }

    pub(super) fn plan(folder: &Path, plans: Vec<Plan>, now: i64) -> Result<Self> {
        let updated = OffsetDateTime::from_unix_timestamp(now)?.format(&Rfc3339)?;
        let mut groups = Vec::new();
        for plan in plans {
            let mut available = Vec::with_capacity(plan.entries.len());
            for entry in &plan.entries {
                available.push(Note::load(folder, entry)?);
            }
            for merge in plan.suggestions.merge {
                let ids: BTreeSet<_> = merge.ids.into_iter().collect();
                let Some(members) = ids
                    .iter()
                    .map(|id| note(&available, *id))
                    .collect::<Option<Vec<_>>>()
                else {
                    continue;
                };
                if members.len() < 2 {
                    continue;
                }
                let protected: Vec<_> = members.iter().filter(|note| fixed(note.entry)).collect();
                let mut group = Vec::new();
                let target = match protected.as_slice() {
                    [] => {
                        let text = merge.text.trim();
                        if text.is_empty() || text.chars().count() > 2000 {
                            continue;
                        }
                        let kind = if KINDS.contains(&merge.kind.as_str()) {
                            &merge.kind
                        } else {
                            &members[0].entry.kind
                        };
                        let created = merged_note(&members, text, kind, &updated)?;
                        let target = created.id.clone();
                        group.push(created);
                        target
                    }
                    [anchor]
                        if anchor.entry.status == "active"
                            || members.iter().all(|note| note.entry.status == "candidate") =>
                    {
                        group.push(Change {
                            id: anchor.entry.id.clone(),
                            before: Some(anchor.raw.clone()),
                            after: anchor.raw.clone(),
                            applied: false,
                        });
                        anchor.entry.id.clone()
                    }
                    _ => continue,
                };
                for member in members {
                    if member.entry.id != target {
                        group.push(member.change(
                            "superseded",
                            Some((&target, "merge")),
                            &updated,
                        )?);
                    }
                }
                for id in ids {
                    available[id - 1] = None;
                }
                groups.push(group);
            }
            for stale in plan.suggestions.stale {
                if let Some(note) = note(&available, stale.id).filter(|note| !fixed(note.entry)) {
                    let reason = stale.why.trim().chars().take(300).collect::<String>();
                    groups.push(vec![note.change(
                        "archived",
                        Some(("", &reason)),
                        &updated,
                    )?]);
                    available[stale.id - 1] = None;
                }
            }
            for note in available.into_iter().flatten() {
                if !fixed(note.entry)
                    && note.entry.status == "candidate"
                    && OffsetDateTime::parse(&note.entry.updated, &Rfc3339)
                        .is_ok_and(|time| time.unix_timestamp() < now.saturating_sub(REVIEW_AGE))
                {
                    groups.push(vec![note.change(
                        "archived",
                        Some(("", "unreviewed")),
                        &updated,
                    )?]);
                }
            }
        }
        Ok(Self {
            phase: Phase::Applying,
            groups,
        })
    }

    pub(super) fn apply(&mut self, folder: &Path) -> Result<usize> {
        let mut changed = 0;
        for group in &mut self.groups {
            let current = group
                .iter()
                .map(|change| read(folder, &change.id))
                .collect::<Result<Vec<_>>>()?;
            let can_apply = group.iter().zip(&current).all(|(change, value)| {
                value.as_deref() == change.before.as_deref()
                    || value.as_deref() == Some(&change.after)
            });
            for (change, current) in group.iter_mut().zip(current) {
                change.applied = change.writes()
                    && (current.as_deref() == Some(&change.after)
                        || (can_apply
                            && replace(
                                folder,
                                &change.id,
                                change.before.as_deref(),
                                Some(&change.after),
                            )?));
                changed += usize::from(change.applied);
            }
        }
        Ok(changed)
    }

    pub(super) fn undo(&self, folder: &Path) -> Result<usize> {
        let mut restored = 0;
        for group in self.groups.iter().rev() {
            let mut originals_restored = true;
            for change in group.iter().rev().filter(|change| change.applied) {
                if change.before.is_none() && !originals_restored {
                    continue;
                }
                let current = read(folder, &change.id)?;
                if current == change.before
                    || (current.as_deref() == Some(&change.after)
                        && replace(
                            folder,
                            &change.id,
                            Some(&change.after),
                            change.before.as_deref(),
                        )?)
                {
                    restored += 1;
                } else {
                    originals_restored = false;
                }
            }
        }
        Ok(restored)
    }
}

fn note<'a, 'entry>(notes: &'a [Option<Note<'entry>>], id: usize) -> Option<&'a Note<'entry>> {
    notes.get(id.checked_sub(1)?)?.as_ref()
}

struct Note<'a> {
    entry: &'a Entry,
    raw: String,
}

impl<'a> Note<'a> {
    fn load(folder: &Path, entry: &'a Entry) -> Result<Option<Self>> {
        Ok(read(folder, &entry.id)?
            .filter(|raw| hex::encode(Sha256::digest(raw.as_bytes())) == entry.revision)
            .map(|raw| Self { entry, raw }))
    }

    fn change(&self, status: &str, link: Option<(&str, &str)>, updated: &str) -> Result<Change> {
        let normalized = self
            .raw
            .trim_start_matches('\u{feff}')
            .replace("\r\n", "\n");
        let (front, body) = split(&normalized)?;
        let yaml: YamlFile = front.parse()?;
        let doc = yaml.ensure_document();
        set(&doc, "status", status)?;
        set(&doc, "updated", updated)?;
        if let Some((target, reason)) = link {
            if !target.is_empty() {
                set(&doc, "supersededBy", target)?;
            }
            set(&doc, "tidyReason", reason)?;
        }
        let mut after = markdown(&self.entry.id, &yaml, body, updated)?;
        if self.raw.contains("\r\n") {
            after = after.replace('\n', "\r\n");
        }
        if self.raw.starts_with('\u{feff}') {
            after.insert(0, '\u{feff}');
        }
        ensure!(
            after.len() <= MAX_FILE,
            "memory file exceeds its size limit"
        );
        Ok(Change {
            id: self.entry.id.clone(),
            before: Some(self.raw.clone()),
            after,
            applied: false,
        })
    }
}

fn read(folder: &Path, id: &str) -> Result<Option<String>> {
    let path = filename(folder, id)?;
    match fs::symlink_metadata(&path) {
        Ok(metadata) => ensure!(metadata.is_file(), "expected a regular memory file"),
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    Ok(Some(String::from_utf8(config::read_limited(
        &path, MAX_FILE,
    )?)?))
}

fn replace(folder: &Path, id: &str, before: Option<&str>, after: Option<&str>) -> Result<bool> {
    let path = filename(folder, id)?;
    if read(folder, id)?.as_deref() != before {
        return Ok(false);
    }
    match (before, after) {
        (Some(_), Some(text)) => config::replace_contents(
            &path,
            text.as_bytes(),
            Some(fs::metadata(&path)?.permissions()),
        )?,
        (None, Some(text)) => {
            let mut file = tempfile::NamedTempFile::new_in(folder)?;
            file.write_all(text.as_bytes())?;
            file.as_file().sync_all()?;
            file.persist_noclobber(path)?;
        }
        (Some(_), None) => fs::remove_file(path)?,
        (None, None) => return Ok(false),
    }
    Ok(true)
}

fn set(doc: &yaml_edit::Document, key: &str, value: &str) -> Result<()> {
    ensure!(doc.set(key, value), "cannot write memory field {key}");
    Ok(())
}

fn markdown(id: &str, yaml: &YamlFile, body: &str, updated: &str) -> Result<String> {
    let yaml = yaml.to_string();
    let front = yaml
        .strip_prefix("---\n")
        .unwrap_or(&yaml)
        .trim_end_matches('\n');
    parse(id, front, body, updated, String::new())?;
    let text = format!("---\n{front}\n---\n\n{}\n", body.trim());
    ensure!(text.len() <= MAX_FILE, "memory file exceeds its size limit");
    Ok(text)
}

fn merged_note(members: &[&Note<'_>], body: &str, kind: &str, updated: &str) -> Result<Change> {
    let first = members.first().context("merge has no members")?;
    let mut random = [0; 6];
    getrandom::fill(&mut random).map_err(anyhow::Error::msg)?;
    let id = format!("m_{}", hex::encode(random));
    let yaml: YamlFile = "".parse()?;
    let doc = yaml.ensure_document();
    let status = if members.iter().any(|note| note.entry.status == "active") {
        "active"
    } else {
        "candidate"
    };
    for (key, value) in [
        ("kind", kind),
        ("scope", &first.entry.scope),
        ("status", status),
        ("origin", "inferred"),
        ("updated", updated),
    ] {
        set(&doc, key, value)?;
    }
    let tags: BTreeSet<_> = members.iter().flat_map(|note| &note.entry.tags).collect();
    let tags_yaml = Sequence::new();
    for tag in tags.into_iter().take(64) {
        tags_yaml.push(tag.as_str());
    }
    ensure!(doc.set("tags", tags_yaml), "cannot write memory tags");
    let mut created = OffsetDateTime::parse(updated, &Rfc3339)?;
    let fields = ["agent", "session", "dir", "file", "quote", "at"];
    let mut sources = BTreeSet::new();
    let ids = Sequence::new();
    for note in members {
        ids.push(note.entry.id.as_str());
        let raw = note
            .raw
            .trim_start_matches('\u{feff}')
            .replace("\r\n", "\n");
        let (front, _) = split(&raw)?;
        let yaml = YamlOwned::load_from_str(front).map_err(anyhow::Error::msg)?;
        let root = yaml.first();
        let old = root
            .and_then(|root| root.as_mapping_get("created"))
            .and_then(YamlOwned::as_str)
            .unwrap_or(&note.entry.updated);
        if let Ok(old) = OffsetDateTime::parse(old, &Rfc3339) {
            created = created.min(old);
        }
        for source in root
            .and_then(|root| root.as_mapping_get("sources"))
            .and_then(YamlOwned::as_vec)
            .into_iter()
            .flatten()
        {
            let values = fields.map(|key| {
                source
                    .as_mapping_get(key)
                    .and_then(YamlOwned::as_str)
                    .unwrap_or("")
                    .to_owned()
            });
            if values.iter().any(|value| !value.is_empty()) {
                sources.insert(values);
            }
        }
    }
    let sources_yaml = Sequence::new();
    for source in sources.into_iter().take(64) {
        let item = Mapping::new();
        for (key, value) in fields
            .into_iter()
            .zip(source)
            .filter(|(_, value)| !value.is_empty())
        {
            item.set(key, value.as_str());
        }
        sources_yaml.push(item);
    }
    ensure!(
        doc.set("sources", sources_yaml),
        "cannot write memory sources"
    );
    ensure!(doc.set("mergedFrom", ids), "cannot write merged memory IDs");
    set(&doc, "created", &created.format(&Rfc3339)?)?;
    if let Some(note) = members.iter().find(|note| !note.entry.why.is_empty()) {
        set(&doc, "why", &note.entry.why)?;
    }
    Ok(Change {
        after: markdown(&id, &yaml, body, updated)?,
        id,
        before: None,
        applied: false,
    })
}
