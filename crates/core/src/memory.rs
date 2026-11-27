use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use saphyr::{LoadableYamlNode, YamlOwned};
use sha2::{Digest, Sha256};
use std::{
    cmp::Reverse,
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use yaml_edit::{Mapping, Sequence, YamlFile};

mod digest;
pub mod distill;
mod magpie;
pub mod mcp;
mod project;

pub use digest::{agent_digest, project_digest};
pub use magpie::import_magpie;
pub use project::key as project_key;

const MAX_FILE: usize = 256 * 1024;
const KINDS: &[&str] = &[
    "profile",
    "preference",
    "rule",
    "reference",
    "fact",
    "procedure",
    "episode",
];
const STATES: &[&str] = &["active", "candidate", "archived", "superseded"];

fn scope_key(prefix: &str, key: &str) -> Result<String> {
    let key = key.trim();
    ensure!(
        !key.is_empty() && key.len() <= 1016 && !key.chars().any(char::is_control),
        "invalid {prefix} scope key"
    );
    Ok(format!("{prefix}:{key}"))
}

#[derive(NsonSerialize)]
pub struct Entry {
    pub id: String,
    pub text: String,
    pub kind: String,
    pub scope: String,
    pub status: String,
    pub tags: Vec<String>,
    pub why: String,
    pub pinned: bool,
    pub origin: String,
    #[njson(skip_serializing_if = "Option::is_none")]
    pub source_quote: Option<String>,
    pub updated: String,
    pub revision: String,
}

#[derive(Default, NsonDeserialize)]
#[njson(deny_unknown_fields)]
pub struct Changes {
    pub text: Option<String>,
    pub kind: Option<String>,
    pub scope: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub why: Option<String>,
    pub pinned: Option<bool>,
}

#[derive(Default, NsonDeserialize)]
#[njson(default, deny_unknown_fields)]
pub struct Query {
    pub text: String,
    pub status: Option<String>,
    pub scope: Option<String>,
    pub kind: Option<String>,
    pub tags: Vec<String>,
    pub offset: usize,
    pub limit: Option<usize>,
}

#[derive(NsonSerialize)]
pub struct Results {
    pub entries: Vec<Entry>,
    pub total: usize,
    pub unreadable: Vec<String>,
}

fn filename(dir: &Path, id: &str) -> Result<PathBuf> {
    ensure!(
        !id.is_empty()
            && id.len() <= 100
            && !id.starts_with('.')
            && !id.ends_with(['.', ' '])
            && !id.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
            && !id.chars().any(char::is_control),
        "invalid memory ID"
    );
    Ok(dir.join(format!("{id}.md")))
}

fn split(source: &str) -> Result<(&str, &str)> {
    let Some(front) = source.strip_prefix("---\n") else {
        return Ok(("", source));
    };
    if let Some(body) = front.strip_prefix("---\n") {
        return Ok(("", body));
    }
    front
        .split_once("\n---\n")
        .context("unclosed memory front matter")
}

fn tags(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut tags: Vec<_> = values
        .into_iter()
        .map(|tag| tag.trim().to_lowercase())
        .filter(|tag| !tag.is_empty())
        .collect();
    tags.sort_unstable();
    tags.dedup();
    tags
}

fn markdown_text(text: &str) -> String {
    text.trim_start_matches('\u{feff}')
        .trim()
        .replace("\r\n", "\n")
}

fn parse(id: &str, front: &str, body: &str, updated: &str, revision: String) -> Result<Entry> {
    let documents = YamlOwned::load_from_str(front).map_err(anyhow::Error::msg)?;
    let root = documents.first();
    ensure!(
        documents.len() <= 1 && root.is_none_or(YamlOwned::is_mapping),
        "expected a memory settings mapping"
    );
    let field = |key: &str| root.and_then(|root| root.as_mapping_get(key));
    let string = |key: &str, fallback: &str| -> Result<String> {
        Ok(field(key)
            .map(|node| {
                node.as_str()
                    .with_context(|| format!("memory field {key} must be a string"))
            })
            .transpose()?
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(fallback)
            .to_owned())
    };
    let list = field("tags")
        .map(|value| {
            value
                .as_vec()
                .context("memory tags must be a list")?
                .iter()
                .map(|tag| {
                    tag.as_str()
                        .map(str::to_owned)
                        .context("memory tags must be strings")
                })
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();
    let entry = Entry {
        id: id.into(),
        text: body.trim().into(),
        kind: string("kind", "fact")?.to_ascii_lowercase(),
        scope: string("scope", "global")?,
        status: string("status", "active")?.to_ascii_lowercase(),
        tags: tags(list),
        why: string("why", "")?,
        pinned: field("pinned")
            .map(|value| {
                value
                    .as_bool()
                    .context("memory pinned flag must be a boolean")
            })
            .transpose()?
            .unwrap_or_default(),
        origin: string("origin", "user")?,
        source_quote: field("sources")
            .and_then(YamlOwned::as_vec)
            .into_iter()
            .flatten()
            .find_map(|item| item.as_mapping_get("quote").and_then(YamlOwned::as_str))
            .map(|quote| quote.chars().take(300).collect()),
        updated: string("updated", updated)?,
        revision,
    };
    entry.validate()?;
    Ok(entry)
}

impl Entry {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.text.is_empty() && self.text.chars().count() <= 2000,
            "memory text must contain 1 to 2000 characters"
        );
        ensure!(KINDS.contains(&self.kind.as_str()), "invalid memory kind");
        ensure!(
            STATES.contains(&self.status.as_str()),
            "invalid memory status"
        );
        ensure!(
            self.scope == "global"
                || ["project:", "agent:"].iter().any(|prefix| {
                    self.scope.strip_prefix(prefix).is_some_and(|key| {
                        !key.trim().is_empty() && !key.chars().any(char::is_control)
                    })
                }),
            "memory scope must be global, project:<key> or agent:<id>"
        );
        ensure!(
            self.scope.len() <= 1024
                && self.why.len() <= 8192
                && self.tags.len() <= 64
                && self
                    .tags
                    .iter()
                    .all(|tag| tag.len() <= 128 && !tag.chars().any(char::is_control)),
            "memory metadata exceeds its limits"
        );
        Ok(())
    }
}

fn source(path: &Path) -> Result<Option<(String, String, String)>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    ensure!(metadata.is_file(), "expected a regular memory file");
    let raw = crate::config::read_limited(path, MAX_FILE)?;
    let revision = hex::encode(Sha256::digest(&raw));
    let text = std::str::from_utf8(&raw)?
        .trim_start_matches('\u{feff}')
        .replace("\r\n", "\n");
    let modified = OffsetDateTime::from(metadata.modified()?).format(&Rfc3339)?;
    Ok(Some((text, modified, revision)))
}

pub fn read(dir: &Path, id: &str) -> Result<Entry> {
    read_in(&dir.join("memory"), id)
}

fn read_in(folder: &Path, id: &str) -> Result<Entry> {
    let path = filename(folder, id)?;
    let (source, updated, revision) = source(&path)?.context("memory does not exist")?;
    let (front, body) = split(&source)?;
    parse(id, front, body, &updated, revision).with_context(|| format!("cannot read memory {id}"))
}

pub fn save(
    dir: &Path,
    id: Option<&str>,
    expected: Option<&str>,
    changes: Changes,
) -> Result<Entry> {
    ensure!(
        id.is_some() == expected.is_some(),
        "editing a memory requires its revision"
    );
    let (folder, _lock) = lock_store(dir)?;
    save_locked(&folder, id.zip(expected), changes, "user", None)
}

pub(crate) fn lock_store(dir: &Path) -> Result<(PathBuf, fs::File)> {
    let folder = dir.join("memory");
    fs::create_dir_all(&folder)?;
    let lock = fs::File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join("memory.lock"))?;
    lock.lock()?;
    Ok((folder, lock))
}

fn save_locked(
    folder: &Path,
    existing: Option<(&str, &str)>,
    changes: Changes,
    origin: &str,
    evidence: Option<distill::Evidence<'_>>,
) -> Result<Entry> {
    let now = OffsetDateTime::now_utc().format(&Rfc3339)?;
    let text = changes.text.as_deref().map(markdown_text);
    let new_id;
    let (id, expected) = match existing {
        Some((id, revision)) => (id, Some(revision)),
        None => {
            let mut bytes = [0; 6];
            getrandom::fill(&mut bytes).map_err(anyhow::Error::msg)?;
            new_id = format!("m_{}", hex::encode(bytes));
            (new_id.as_str(), None)
        }
    };
    let path = filename(folder, id)?;
    let previous = source(&path)?;
    ensure!(
        previous.as_ref().map(|(_, _, revision)| revision.as_str()) == expected,
        "memory changed on disk; reload it before saving"
    );
    ensure!(
        previous.is_some() || text.as_ref().is_some_and(|text| !text.is_empty()),
        "a new memory needs its text"
    );
    let (old, created, _) = previous.unwrap_or_else(|| (String::new(), now.clone(), String::new()));
    let (front, body) = split(&old)?;
    if expected.is_some() {
        parse(id, front, body, &created, String::new())?;
    }
    let file: YamlFile = front.parse()?;
    let doc = file.ensure_document();
    for (key, value) in [
        ("kind", changes.kind),
        ("scope", changes.scope),
        ("status", changes.status),
        ("why", changes.why),
    ] {
        if let Some(value) = value {
            ensure!(
                doc.set(key, value.trim()),
                "cannot write memory field {key}"
            );
        }
    }
    if let Some(values) = changes.tags {
        let list = Sequence::new();
        for tag in tags(values) {
            list.push(tag.as_str());
        }
        ensure!(doc.set("tags", list), "cannot write memory tags");
    }
    if let Some(pinned) = changes.pinned {
        ensure!(doc.set("pinned", pinned), "cannot write memory pinned flag");
    }
    if let Some(evidence) = evidence {
        let source = Mapping::new();
        for (key, value) in [
            ("agent", evidence.agent),
            ("session", evidence.session),
            ("dir", evidence.folder),
            ("quote", evidence.quote),
        ] {
            source.set(key, value);
        }
        let sources = Sequence::new();
        sources.push(source);
        ensure!(doc.set("sources", sources), "cannot write memory source");
    }
    if doc.get("created").is_none() {
        ensure!(
            doc.set("created", created.as_str()),
            "cannot write memory creation time"
        );
    }
    if origin == "user" {
        ensure!(
            doc.set("edited", now.as_str()),
            "cannot write memory edit time"
        );
    }
    let origin = if existing.is_some() && origin != "user" {
        None
    } else {
        Some(origin)
    };
    for (key, value) in [("origin", origin), ("updated", Some(now.as_str()))] {
        let Some(value) = value else { continue };
        ensure!(doc.set(key, value), "cannot write memory field {key}");
    }
    let front = file.to_string();
    let front = front
        .strip_prefix("---\n")
        .unwrap_or(&front)
        .trim_end_matches('\n');
    let body = text.as_deref().unwrap_or(body).trim();
    let contents = format!("---\n{front}\n---\n\n{body}\n");
    ensure!(
        contents.len() <= MAX_FILE,
        "memory file exceeds its size limit"
    );
    let entry = parse(
        id,
        front,
        body,
        &now,
        hex::encode(Sha256::digest(contents.as_bytes())),
    )?;
    ensure!(
        source(&path)?
            .as_ref()
            .map(|(_, _, revision)| revision.as_str())
            == expected,
        "memory changed on disk; reload it before saving"
    );
    let mut temp = tempfile::NamedTempFile::new_in(folder)?;
    temp.write_all(contents.as_bytes())?;
    temp.as_file().sync_all()?;
    if expected.is_some() {
        temp.persist(path)?;
    } else {
        temp.persist_noclobber(path)?;
    }
    Ok(entry)
}

pub fn search(dir: &Path, query: Query) -> Result<Results> {
    search_where(&dir.join("memory"), query, |_| true)
}

fn search_where(folder: &Path, query: Query, include: impl Fn(&Entry) -> bool) -> Result<Results> {
    let wanted = query.text.trim().to_lowercase();
    let terms: Vec<_> = wanted.split_whitespace().collect();
    let wanted_tags = tags(query.tags);
    let mut entries = Vec::new();
    let mut unreadable = scan(folder, |entry| {
        if !include(&entry)
            || query
                .status
                .as_ref()
                .is_some_and(|status| status != &entry.status)
            || query.kind.as_ref().is_some_and(|kind| kind != &entry.kind)
            || query
                .scope
                .as_ref()
                .is_some_and(|scope| scope != &entry.scope)
            || !wanted_tags.iter().all(|tag| entry.tags.contains(tag))
        {
            return;
        }
        let mut rank = 0;
        if !terms.is_empty() {
            let body = entry.text.to_lowercase();
            let searchable = format!(
                "{body}\n{}\n{}\n{}",
                entry.why.to_lowercase(),
                entry.kind,
                entry.tags.join(" ")
            );
            rank = terms
                .iter()
                .filter(|term| searchable.contains(**term))
                .count();
            if rank == 0 {
                return;
            }
            rank += usize::from(body.contains(&wanted)) * terms.len();
        }
        let updated = OffsetDateTime::parse(&entry.updated, &Rfc3339).ok();
        entries.push((
            Reverse(rank),
            Reverse(entry.pinned),
            Reverse(updated),
            entry,
        ));
    })?;
    entries.sort_unstable_by(|a, b| (&a.0, &a.1, &a.2, &a.3.id).cmp(&(&b.0, &b.1, &b.2, &b.3.id)));
    unreadable.sort_unstable();
    let total = entries.len();
    let entries = entries
        .into_iter()
        .skip(query.offset)
        .take(query.limit.unwrap_or(100).clamp(1, 500))
        .map(|(_, _, _, entry)| entry)
        .collect();
    Ok(Results {
        entries,
        total,
        unreadable,
    })
}

fn scan(folder: &Path, mut visit: impl FnMut(Entry)) -> Result<Vec<String>> {
    let paths = match fs::read_dir(folder) {
        Ok(paths) => paths,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let mut unreadable = Vec::new();
    for path in paths {
        let path = path?.path();
        if path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        if let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) {
            match read_in(folder, id) {
                Ok(entry) => visit(entry),
                Err(_) => unreadable.push(id.to_owned()),
            }
        }
    }
    Ok(unreadable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_magpie_markdown_and_preserves_provenance_when_archiving() -> Result<()> {
        let home = tempfile::tempdir()?;
        let root = home.path();
        fs::create_dir(root.join("memory"))?;
        let path = root.join("memory/m_ported.md");
        let header = "---\r\nkind: procedure\r\nscope: project:example/app\r\ncreated: '2026-11-26T04:09:42Z'\r\nsources:\r\n  - agent: claude\r\n    session: original\r\ntags: [Release, Windows]\r\n---\r\n";
        let first = format!("\u{feff}{header}\r\nBuild the release with a clean profile.\r\n");
        fs::write(&path, first)?;
        fs::write(root.join("memory/broken.md"), "---\ninvalid: [\n---\n")?;
        let matches = search(
            root,
            Query {
                text: "release windows".into(),
                ..Default::default()
            },
        )?;
        assert_eq!(matches.total, 1);
        assert_eq!(matches.unreadable, ["broken"]);
        let initial = &matches.entries[0];
        let revised = save(
            root,
            Some(&initial.id),
            Some(&initial.revision),
            Changes {
                tags: Some(vec!["Windows".into(), " WINDOWS ".into(), "null".into()]),
                pinned: Some(true),
                status: Some("archived".into()),
                ..Default::default()
            },
        )?;
        assert_eq!(revised.status, "archived");
        assert_eq!(revised.tags, ["null", "windows"]);
        assert!(revised.pinned);
        let contents = fs::read_to_string(&path)?;
        assert!(contents.contains("session: original"));
        assert!(contents.contains("2026-11-26T04:09:42Z"));
        assert!(
            save(
                root,
                Some(&initial.id),
                Some(&initial.revision),
                Changes::default()
            )
            .is_err()
        );
        assert_eq!(contents, fs::read_to_string(&path)?);
        let new = save(
            root,
            None,
            None,
            Changes {
                text: Some("\u{feff}记住这条偏好。\r\n保留换行。\r\n".into()),
                ..Default::default()
            },
        )?;
        let reread = read(root, &new.id)?;
        assert_eq!(new.revision, reread.revision);
        assert_eq!(new.text, "记住这条偏好。\n保留换行。");
        assert_eq!(new.text, reread.text);
        Ok(())
    }
}
