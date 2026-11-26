mod discovery;
mod import;
mod native;
pub use import::Request as ImportRequest;

use crate::{config::read_limited, native_settings, project_instructions::replace};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::ErrorKind, path::Path};

const MAX_LIBRARY: usize = 16 * 1024 * 1024;
pub const MAX_DOCUMENT: usize = 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Subagent,
    Command,
}

impl Kind {
    fn folder(self) -> &'static str {
        match self {
            Self::Subagent => "subagents",
            Self::Command => "commands",
        }
    }
}

#[derive(Clone, NsonDeserialize, NsonSerialize)]
pub struct Entry {
    pub name: String,
    #[nextjson(default)]
    pub description: String,
    pub prompt: String,
    pub agents: Vec<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    #[nextjson(default)]
    pub skills: Vec<String>,
    #[nextjson(default)]
    pub mcp: Vec<String>,
}

impl Entry {
    pub fn from_json(source: &str) -> Result<Self> {
        ensure!(
            source.len() <= MAX_DOCUMENT,
            "resource document exceeds 1 MiB"
        );
        nextjson::from_str(source.strip_prefix('\u{feff}').unwrap_or(source))
            .map_err(anyhow::Error::msg)
    }
}

pub enum Edit {
    Read,
    Discover,
    ProbeSource {
        source: String,
    },
    ImportSource {
        request: ImportRequest,
    },
    Adopt {
        agent: String,
        path: String,
        revision: String,
    },
    Save {
        previous: Option<(String, String)>,
        entry: Entry,
    },
    Remove {
        name: String,
        revision: String,
    },
    Sync,
}

#[derive(Clone, NsonDeserialize, NsonSerialize)]
struct Target {
    agent: String,
    name: String,
    path: String,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Applied {
    target: Target,
    hash: String,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Pending {
    target: Target,
    after: Option<String>,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Adopted {
    original: String,
    definition: String,
}

#[derive(Default, NsonDeserialize, NsonSerialize)]
struct Store {
    entries: BTreeMap<String, Entry>,
    applied: BTreeMap<String, Applied>,
    pending: Option<Pending>,
    #[nextjson(default)]
    adopted: BTreeMap<String, Adopted>,
    #[nextjson(default)]
    imported: BTreeMap<String, import::Origin>,
}

#[derive(NsonSerialize)]
struct Saved<'a> {
    entry: &'a Entry,
    revision: String,
    source: Option<&'a str>,
}

#[derive(NsonSerialize)]
struct Placement {
    agent: String,
    name: String,
    path: Option<String>,
    status: &'static str,
    error: Option<String>,
}

#[derive(NsonSerialize)]
struct View<'a> {
    entries: Vec<Saved<'a>>,
    placements: Vec<Placement>,
    agents: &'a [&'static str],
}

fn encode(value: &impl NsonSerialize) -> Result<String> {
    nextjson::to_string(value).map_err(anyhow::Error::msg)
}

fn hash(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}

fn revision(entry: &Entry) -> Result<String> {
    Ok(hash(&encode(entry)?))
}

fn definition(entry: &Entry) -> Result<String> {
    let mut entry = entry.clone();
    entry.agents.clear();
    entry.prompt = entry
        .prompt
        .trim_matches(['\r', '\n'])
        .replace("\r\n", "\n");
    revision(&entry)
}

fn validate_name(kind: Kind, name: &str) -> Result<()> {
    let bytes = name.as_bytes();
    let alnum = |byte: &u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    let valid_length = match kind {
        Kind::Subagent => (2..=50).contains(&bytes.len()),
        Kind::Command => (1..=64).contains(&bytes.len()),
    };
    ensure!(
        valid_length
            && bytes.first().is_some_and(alnum)
            && bytes.last().is_some_and(alnum)
            && bytes.iter().all(|byte| alnum(byte)
                || *byte == b'-'
                || (kind == Kind::Command && *byte == b'_')),
        "invalid resource name; use lowercase letters, digits and hyphens, with underscores also allowed in command names"
    );
    ensure!(
        !matches!(name, "con" | "prn" | "aux" | "nul")
            && !(bytes.len() == 4
                && matches!(&bytes[..3], b"com" | b"lpt")
                && matches!(bytes[3], b'1'..=b'9')),
        "the resource name is reserved on Windows"
    );
    ensure!(
        kind != Kind::Subagent
            || !matches!(
                name,
                "self" | "system" | "user" | "model" | "tool" | "config" | "default" | "main"
            ),
        "the name is reserved by a subagent client"
    );
    Ok(())
}

impl Entry {
    fn validate(&mut self, kind: Kind, agents: &[&str]) -> Result<()> {
        validate_name(kind, &self.name)?;
        self.description = self.description.trim().to_owned();
        ensure!(
            self.description.chars().count() <= 1000 && !self.description.contains('\0'),
            "description must be at most 1000 characters without NUL"
        );
        ensure!(
            kind == Kind::Command || !self.description.is_empty(),
            "a subagent needs a description"
        );
        ensure!(
            self.prompt.len() <= 100_000
                && !self.prompt.trim().is_empty()
                && !self.prompt.contains('\0'),
            "provide a nonempty prompt of at most 100000 bytes without NUL"
        );
        self.agents.sort_unstable();
        self.agents.dedup();
        ensure!(
            self.agents
                .iter()
                .all(|agent| agents.contains(&agent.as_str())),
            "unsupported agent assignment"
        );
        if kind == Kind::Command {
            ensure!(
                self.model.is_none()
                    && self.effort.is_none()
                    && self.skills.is_empty()
                    && self.mcp.is_empty(),
                "subagent options do not apply to slash commands"
            );
        }
        if let Some(model) = &self.model {
            ensure!(
                !model.is_empty()
                    && model.len() <= 200
                    && !model.contains(char::is_whitespace)
                    && !model.contains('\0'),
                "model must be a nonempty ID of at most 200 bytes without whitespace"
            );
        }
        if let Some(effort) = &self.effort {
            ensure!(
                matches!(effort.as_str(), "low" | "medium" | "high" | "xhigh" | "max"),
                "unsupported reasoning effort"
            );
        }
        for references in [&self.skills, &self.mcp] {
            ensure!(
                references.len() <= 100
                    && references.iter().all(|name| !name.trim().is_empty()
                        && name.len() <= 200
                        && !name.contains(['\n', '\r', '\0'])),
                "invalid skill or MCP server reference"
            );
        }
        Ok(())
    }
}

fn path_key(path: &str) -> String {
    if cfg!(windows) {
        path.to_lowercase()
    } else {
        path.to_owned()
    }
}

impl Store {
    fn load(path: &Path) -> Result<Self> {
        match read_limited(path, MAX_LIBRARY) {
            Ok(bytes) => nextjson::from_slice(&bytes).map_err(anyhow::Error::msg),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    fn persist(&self, path: &Path) -> Result<()> {
        let contents = encode(self)?;
        ensure!(
            contents.len() <= MAX_LIBRARY,
            "agent resource library exceeds 16 MiB"
        );
        fs::create_dir_all(path.parent().context("missing library directory")?)?;
        replace(path, &contents)
    }

    fn check(&self, name: &str, expected: &str) -> Result<()> {
        let entry = self
            .entries
            .get(name)
            .context("resource was removed; reload the library")?;
        ensure!(
            revision(entry)? == expected,
            "resource changed; reload the library"
        );
        Ok(())
    }

    fn own(&mut self, target: Target, after: Option<String>) {
        let key = path_key(&target.path);
        if let Some(hash) = after {
            self.applied.insert(key, Applied { target, hash });
        } else {
            self.applied.remove(&key);
            self.adopted.remove(&key);
        }
    }

    fn recover(&mut self, path: &Path, profiles: &native::Profiles) -> Result<()> {
        let Some(pending) = self.pending.take() else {
            return Ok(());
        };
        if profiles.includes(&pending.target)
            && native_settings::read(Path::new(&pending.target.path))?
                .as_deref()
                .map(hash)
                == pending.after
        {
            self.own(pending.target, pending.after);
        }
        self.persist(path)
    }

    fn apply(
        &mut self,
        store_path: &Path,
        target: Target,
        desired: Option<&str>,
        sync: bool,
    ) -> Result<&'static str> {
        let file = Path::new(&target.path);
        let current = native_settings::read(file)?;
        let key = path_key(&target.path);
        let owned = self.applied.get(&key);
        if let Some(before) = &current
            && !owned.is_some_and(|record| hash(before) == record.hash)
        {
            if sync && self.applied.remove(&key).is_some() {
                self.adopted.remove(&key);
                self.persist(store_path)?;
            }
            return Ok("conflict");
        }
        if !sync && current.as_deref() != desired {
            return Ok("pending");
        }
        if sync && current.as_deref() != desired {
            ensure!(
                self.pending.is_none(),
                "another resource write is awaiting recovery"
            );
            let after = desired.map(hash);
            self.pending = Some(Pending {
                target: target.clone(),
                after: after.clone(),
            });
            self.persist(store_path)?;
            if let Err(error) = native_settings::write(file, current.as_deref(), desired) {
                self.pending = None;
                self.persist(store_path)?;
                return Err(error);
            }
            self.own(target, after);
            self.pending = None;
            self.persist(store_path)?;
        } else if sync && current.is_none() && self.applied.remove(&key).is_some() {
            self.adopted.remove(&key);
            self.persist(store_path)?;
        }
        Ok(if desired.is_some() {
            "applied"
        } else {
            "removed"
        })
    }
}

pub fn library(dir: &Path, kind: Kind, project: Option<&Path>, edit: Edit) -> Result<String> {
    ensure!(
        kind == Kind::Subagent || project.is_none(),
        "slash command assignments are user-wide"
    );
    let project = project.map(fs::canonicalize).transpose()?;
    if let Some(project) = &project {
        ensure!(project.is_dir(), "project must be a directory");
    }
    let profiles = native::Profiles::load(dir, kind, project.as_deref())?;
    let folder = dir.join(kind.folder());
    let store = match project {
        Some(project) => {
            let name = project.to_str().context("project path must be UTF-8")?;
            folder
                .join("projects")
                .join(format!("{}.json", hash(&path_key(name))))
        }
        None => folder.join("library.json"),
    };
    library_in(dir, kind, &store, &profiles, edit)
}

fn library_in(
    dir: &Path,
    kind: Kind,
    path: &Path,
    profiles: &native::Profiles,
    edit: Edit,
) -> Result<String> {
    let source = match &edit {
        Edit::ProbeSource { source } => Some(source),
        Edit::ImportSource { request } => Some(&request.source),
        _ => None,
    };
    let scan = source
        .map(|source| {
            ensure!(
                kind == Kind::Subagent && !profiles.project_scope,
                "import subagents into the user library"
            );
            import::Scan::open(source)
        })
        .transpose()?;
    let _guard = native_settings::lock(dir)?;
    let mut store = Store::load(path)?;
    if matches!(edit, Edit::Discover) {
        return discovery::list(profiles, &store);
    }
    if matches!(edit, Edit::ProbeSource { .. }) {
        return import::preview(&scan.expect("source scan"), &store, profiles.agents);
    }
    let sync = !matches!(edit, Edit::Read);
    if sync {
        store.recover(path, profiles)?;
    }
    match edit {
        Edit::Read | Edit::Sync => {}
        Edit::Discover | Edit::ProbeSource { .. } => unreachable!(),
        Edit::ImportSource { request } => {
            import::apply(
                scan.expect("source scan"),
                request,
                &mut store,
                profiles.agents,
            )?;
            store.persist(path)?;
        }
        Edit::Adopt {
            agent,
            path: file,
            revision,
        } => {
            discovery::adopt(profiles, &mut store, &agent, &file, &revision)?;
            store.persist(path)?;
        }
        Edit::Save {
            previous,
            mut entry,
        } => {
            entry.validate(kind, profiles.agents)?;
            if let Some((old, revision)) = &previous {
                store.check(old, revision)?;
            }
            ensure!(
                previous.as_ref().is_some_and(|(old, _)| old == &entry.name)
                    || !store.entries.contains_key(&entry.name),
                "a resource with that name already exists"
            );
            if let Some((old, _)) = previous {
                store.entries.remove(&old);
                if let Some(origin) = store.imported.remove(&old) {
                    store.imported.insert(entry.name.clone(), origin);
                }
            }
            store.entries.insert(entry.name.clone(), entry);
            store.persist(path)?;
        }
        Edit::Remove { name, revision } => {
            store.check(&name, &revision)?;
            store.entries.remove(&name);
            store.imported.remove(&name);
            store.persist(path)?;
        }
    }
    let placements = placements(&mut store, path, profiles, sync)?;
    let entries = store
        .entries
        .values()
        .map(|entry| {
            Ok(Saved {
                entry,
                revision: revision(entry)?,
                source: store
                    .imported
                    .get(&entry.name)
                    .map(|origin| origin.source.as_str()),
            })
        })
        .collect::<Result<_>>()?;
    encode(&View {
        entries,
        placements,
        agents: profiles.agents,
    })
}

fn placements(
    store: &mut Store,
    path: &Path,
    profiles: &native::Profiles,
    sync: bool,
) -> Result<Vec<Placement>> {
    let mut jobs = BTreeMap::new();
    for applied in store
        .applied
        .values()
        .filter(|record| profiles.includes(&record.target))
    {
        jobs.insert(
            path_key(&applied.target.path),
            (applied.target.clone(), None),
        );
    }
    let mut rows = Vec::new();
    for entry in store.entries.values() {
        let definition = definition(entry)?;
        for agent in &entry.agents {
            for mut targets in profiles.destinations(agent, &entry.name) {
                let Some(primary) = targets.next() else {
                    continue;
                };
                let alternatives: Vec<_> = targets.filter_map(Result::ok).collect();
                let adopted: Vec<_> = alternatives
                    .iter()
                    .filter(|target| store.adopted.contains_key(&path_key(&target.path)))
                    .cloned()
                    .collect();
                let primary_adopted = primary
                    .as_ref()
                    .is_ok_and(|target| store.adopted.contains_key(&path_key(&target.path)));
                let destinations = if adopted.is_empty() {
                    if let Some(existing) = alternatives.into_iter().find(|target| {
                        primary.as_ref().is_ok_and(|primary| {
                            fs::symlink_metadata(&primary.path)
                                .is_err_and(|error| error.kind() == ErrorKind::NotFound)
                        }) && !primary_adopted
                            && fs::symlink_metadata(&target.path).is_ok()
                    }) {
                        vec![Ok(existing)]
                    } else {
                        vec![primary]
                    }
                } else {
                    primary
                        .into_iter()
                        .filter(|_| primary_adopted)
                        .chain(adopted)
                        .map(Ok)
                        .collect()
                };
                for target in destinations {
                    let target = match target {
                        Ok(target) => target,
                        Err(error) => {
                            rows.push(error_row(agent, &entry.name, None, error));
                            continue;
                        }
                    };
                    let key = path_key(&target.path);
                    let desired = store
                        .adopted
                        .get(&key)
                        .filter(|record| record.definition == definition)
                        .map(|record| Ok(record.original.clone()))
                        .unwrap_or_else(|| profiles.render(&target, entry));
                    match desired {
                        Ok(text) => {
                            if let Some((_, Some(other))) = jobs.get(&key) {
                                if other != &text {
                                    rows.push(error_row(
                                        agent,
                                        &entry.name,
                                        Some(target.path),
                                        "different client formats resolve to the same file",
                                    ));
                                }
                                continue;
                            }
                            jobs.insert(key, (target, Some(text)));
                        }
                        Err(error) => {
                            if jobs.get(&key).is_some_and(|(_, pending)| pending.is_none()) {
                                jobs.remove(&key);
                            }
                            rows.push(error_row(agent, &entry.name, Some(target.path), error))
                        }
                    }
                }
            }
        }
    }
    let mut jobs: Vec<_> = jobs.into_values().collect();
    jobs.sort_by_key(|(_, desired)| desired.is_some());
    for (target, desired) in jobs {
        let Target {
            agent,
            name,
            path: file,
        } = target.clone();
        match store.apply(path, target, desired.as_deref(), sync) {
            Ok(status) => rows.push(Placement {
                agent,
                name,
                path: Some(file),
                status,
                error: None,
            }),
            Err(error) => {
                rows.push(error_row(&agent, &name, Some(file), error));
                if sync && store.pending.is_some() {
                    store.recover(path, profiles)?;
                }
            }
        }
    }
    rows.sort_by(|left, right| {
        (&left.name, &left.agent, &left.path).cmp(&(&right.name, &right.agent, &right.path))
    });
    Ok(rows)
}

fn error_row(
    agent: &str,
    name: &str,
    path: Option<String>,
    error: impl std::fmt::Display,
) -> Placement {
    Placement {
        agent: agent.to_owned(),
        name: name.to_owned(),
        path,
        status: "error",
        error: Some(format!("{error:#}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nextjson::Value;

    fn parse(source: &str) -> Entry {
        Entry::from_json(source).unwrap()
    }

    fn run(dir: &Path, project: &Path, edit: Edit) -> Value {
        let view = library(dir, Kind::Subagent, Some(project), edit).unwrap();
        nextjson::from_str(&view).unwrap()
    }

    #[test]
    fn project_subagents_reject_stale_edits_and_preserve_external_files() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("data");
        let project = temp.path().join("project");
        let other = temp.path().join("other");
        fs::create_dir(&project).unwrap();
        fs::create_dir(&other).unwrap();
        let entry = parse(
            r#"{"name":"reviewer","description":"Reviews changes","prompt":"Report findings.","agents":["claude","codex","agy"],"skills":["review"],"mcp":["local"],"model":"local/m","effort":"high"}"#,
        );
        let added = run(
            &dir,
            &project,
            Edit::Save {
                previous: None,
                entry: entry.clone(),
            },
        );
        assert_eq!(added["placements"].as_array().unwrap().len(), 3);
        assert!(
            added["placements"]
                .as_array()
                .unwrap()
                .iter()
                .all(|item| item["status"].as_str() == Some("applied"))
        );
        assert!(
            run(&dir, &other, Edit::Read)["entries"]
                .as_array()
                .unwrap()
                .is_empty()
        );

        let claude = project.join(".claude/agents/reviewer.md");
        let codex = project.join(".codex/agents/reviewer.toml");
        let antigravity = project.join(".agents/agents/reviewer.md");
        assert!(
            fs::read_to_string(&claude)
                .unwrap()
                .contains("skills: [\"review\"]")
        );
        let role: toml_edit::DocumentMut = fs::read_to_string(&codex).unwrap().parse().unwrap();
        assert_eq!(role["model"].as_str(), Some("local/m"));
        assert_eq!(
            role["developer_instructions"].as_str(),
            Some("Report findings.\n")
        );
        assert!(
            fs::read_to_string(&antigravity)
                .unwrap()
                .contains("subagent: true")
        );
        fs::write(&claude, "external edit").unwrap();

        let stale = library(
            &dir,
            Kind::Subagent,
            Some(&project),
            Edit::Save {
                previous: Some((entry.name.clone(), "stale".into())),
                entry: entry.clone(),
            },
        );
        assert!(stale.unwrap_err().to_string().contains("resource changed"));
        let removed = run(
            &dir,
            &project,
            Edit::Remove {
                name: entry.name,
                revision: added["entries"][0]["revision"].as_str().unwrap().to_owned(),
            },
        );
        assert!(removed["entries"].as_array().unwrap().is_empty());
        assert_eq!(fs::read_to_string(claude).unwrap(), "external edit");
        assert!(!codex.exists());
        assert!(!antigravity.exists());
    }

    #[test]
    fn command_arguments_follow_each_clients_syntax() {
        let temp = tempfile::tempdir().unwrap();
        let profiles = native::Profiles {
            kind: Kind::Command,
            agents: &["claude", "gemini", "opencode"],
            project_scope: false,
            homes: BTreeMap::from([
                ("claude", vec![temp.path().join("claude")]),
                (
                    "gemini",
                    vec![
                        temp.path().join("gemini"),
                        temp.path().join("gateway/gemini"),
                    ],
                ),
                ("opencode", vec![temp.path().join("opencode")]),
            ]),
        };
        let dir = temp.path().join("data");
        let path = dir.join("commands/library.json");
        let mut entry = parse(
            r#"{"name":"check","description":"Read the plan","prompt":"Check $1 then $2. All: $ARGUMENTS.","agents":["claude","opencode","gemini"]}"#,
        );
        let saved = library_in(
            &dir,
            Kind::Command,
            &path,
            &profiles,
            Edit::Save {
                previous: None,
                entry: entry.clone(),
            },
        )
        .unwrap();
        let saved: Value = nextjson::from_str(&saved).unwrap();
        assert_eq!(
            saved["placements"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| row["status"].as_str() == Some("error"))
                .count(),
            2
        );
        let command = |client| temp.path().join(client).join("commands/check.md");
        assert!(
            fs::read_to_string(command("claude"))
                .unwrap()
                .contains("Check $ARGUMENTS[0] then $ARGUMENTS[1]. All: $ARGUMENTS.")
        );
        assert!(
            fs::read_to_string(command("opencode"))
                .unwrap()
                .contains(&entry.prompt)
        );
        entry.prompt = "Check $ARGUMENTS.".into();
        library_in(
            &dir,
            Kind::Command,
            &path,
            &profiles,
            Edit::Save {
                previous: Some((
                    entry.name.clone(),
                    saved["entries"][0]["revision"].as_str().unwrap().into(),
                )),
                entry,
            },
        )
        .unwrap();
        for profile in ["gemini", "gateway/gemini"] {
            let file = temp.path().join(profile).join("commands/check.toml");
            let document: toml_edit::DocumentMut =
                fs::read_to_string(file).unwrap().parse().unwrap();
            assert_eq!(document["prompt"].as_str(), Some("Check {{args}}.\n"));
        }
    }

    #[test]
    fn pending_publication_recovers_before_withdrawal() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        let profiles = native::Profiles {
            kind: Kind::Command,
            agents: &["claude"],
            project_scope: false,
            homes: BTreeMap::from([("claude", vec![dir.join("claude")])]),
        };
        let path = dir.join("commands/library.json");
        let entry = parse(r#"{"name":"check","prompt":"Check $ARGUMENTS.","agents":["claude"]}"#);
        let saved = revision(&entry).unwrap();
        let target = profiles
            .destinations("claude", &entry.name)
            .flatten()
            .next()
            .unwrap()
            .unwrap();
        let file = Path::new(&target.path).to_owned();
        let contents = profiles.render(&target, &entry).unwrap();
        let store = Store {
            entries: BTreeMap::from([(entry.name.clone(), entry)]),
            applied: BTreeMap::new(),
            pending: Some(Pending {
                target,
                after: Some(hash(&contents)),
            }),
            ..Store::default()
        };
        store.persist(&path).unwrap();
        native_settings::write(&file, None, Some(&contents)).unwrap();
        library_in(
            dir,
            Kind::Command,
            &path,
            &profiles,
            Edit::Remove {
                name: "check".into(),
                revision: saved,
            },
        )
        .unwrap();
        assert!(!file.exists());
        assert!(Store::load(&path).unwrap().pending.is_none());
    }

    #[test]
    fn adopt_legacy_command_in_place_and_reject_external_changes() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let dir = temp.path();
        let opencode = dir.join("opencode");
        let profiles = native::Profiles {
            kind: Kind::Command,
            agents: &["opencode", "qwen"],
            project_scope: false,
            homes: BTreeMap::from([
                ("opencode", vec![opencode.clone()]),
                ("qwen", vec![dir.join("qwen")]),
            ]),
        };
        let path = dir.join("commands/library.json");
        let call = |edit| -> Result<Value> {
            nextjson::from_str(&library_in(dir, Kind::Command, &path, &profiles, edit)?)
                .map_err(anyhow::Error::msg)
        };
        let legacy = opencode.join("command/check.md");
        fs::create_dir_all(legacy.parent().unwrap())?;
        let original = "\u{feff}---\r\ndescription: Review a file\r\nagent: plan\r\n---\r\n\r\nCheck $ARGUMENTS.\r\n";
        fs::write(&legacy, original)?;
        let discovery = call(Edit::Discover)?;
        let row = &discovery["candidates"][0];
        let native_path = row["path"].as_str().unwrap().to_owned();
        let checksum = row["revision"].as_str().unwrap().to_owned();
        let take = |revision| Edit::Adopt {
            agent: "opencode".into(),
            path: native_path.clone(),
            revision,
        };
        fs::write(&legacy, format!("{original}Updated externally.\r\n"))?;
        assert!(
            call(take(checksum.clone()))
                .unwrap_err()
                .to_string()
                .contains("changed")
        );
        fs::write(&legacy, original)?;
        let adopted = call(take(checksum))?;
        assert_eq!(adopted["placements"][0]["status"].as_str(), Some("applied"));
        assert_eq!(fs::read_to_string(&legacy)?, original);
        assert!(!opencode.join("commands/check.md").exists());
        let duplicate = opencode.join("commands/check.md");
        fs::create_dir_all(duplicate.parent().unwrap())?;
        fs::write(&duplicate, "An unrelated command")?;
        let candidates = call(Edit::Discover)?;
        let row = &candidates["candidates"][0];
        assert_eq!(row["status"].as_str(), Some("conflict"));
        assert!(
            call(Edit::Adopt {
                agent: "opencode".into(),
                path: row["path"].as_str().unwrap().into(),
                revision: row["revision"].as_str().unwrap().into(),
            })
            .unwrap_err()
            .to_string()
            .contains("another resource has that name")
        );
        fs::remove_file(duplicate)?;
        let qwen = dir.join("qwen/commands/check.toml");
        fs::create_dir_all(qwen.parent().unwrap())?;
        let toml =
            "# Native command\r\ndescription = 'Review a file'\r\nprompt = 'Check {{args}}.'\r\n";
        fs::write(&qwen, toml)?;
        let candidates = call(Edit::Discover)?;
        let row = &candidates["candidates"][0];
        assert_eq!(row["agent"].as_str(), Some("qwen"));
        let joined = call(Edit::Adopt {
            agent: "qwen".into(),
            path: row["path"].as_str().unwrap().into(),
            revision: row["revision"].as_str().unwrap().into(),
        })?;
        assert_eq!(joined["entries"].as_array().unwrap().len(), 1);
        assert_eq!(fs::read_to_string(&qwen)?, toml);
        fs::write(&legacy, "External replacement")?;
        call(Edit::Remove {
            name: "check".into(),
            revision: joined["entries"][0]["revision"].as_str().unwrap().into(),
        })?;
        assert_eq!(fs::read_to_string(&legacy)?, "External replacement");
        assert!(!qwen.exists());
        Ok(())
    }

    #[test]
    fn source_import_rechecks_the_preview_and_preserves_user_edits() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        let source = root.join("source");
        let input = source.join("agents");
        let skill = source.join("skill");
        fs::create_dir_all(&input)?;
        fs::create_dir_all(&skill)?;
        let file = input.join("review.md");
        let write = |prompt| {
            fs::write(
                &file,
                format!("---\ndescription: Review a change\n---\n{prompt}\n"),
            )
        };
        fs::write(skill.join("SKILL.md"), "A skill")?;
        fs::write(
            skill.join("example.md"),
            "---\nname: exclude\ndescription: Example\n---\nIgnore me",
        )?;
        write("Version one.")?;

        let profiles = native::Profiles {
            kind: Kind::Subagent,
            agents: &["claude"],
            project_scope: false,
            homes: BTreeMap::from([("claude", vec![root.join("claude")])]),
        };
        let path = root.join("data/subagents/library.json");
        let call = |edit| -> Result<Value> {
            nextjson::from_str(&library_in(root, Kind::Subagent, &path, &profiles, edit)?)
                .map_err(anyhow::Error::msg)
        };
        let source = source.to_str().unwrap();
        let scan = || {
            call(Edit::ProbeSource {
                source: source.to_owned(),
            })
        };
        let import = |preview: &Value| {
            let chosen = &preview["candidates"][0];
            let request = nextjson::json!({
                "source": source,
                "files": [{"path": chosen["path"].as_str().unwrap(), "revision": chosen["revision"].as_str().unwrap()}],
                "agents": ["claude"],
            });
            call(Edit::ImportSource {
                request: ImportRequest::from_json(&request.to_string())?,
            })
        };
        let preview = scan()?;
        assert_eq!(preview["candidates"].as_array().unwrap().len(), 1);
        assert_eq!(preview["candidates"][0]["name"].as_str(), Some("review"));
        assert!(!path.exists());
        write("Version two.")?;
        assert!(
            import(&preview)
                .unwrap_err()
                .to_string()
                .contains("changed")
        );
        let added = import(&scan()?)?;
        assert_eq!(added["entries"].as_array().unwrap().len(), 1);
        let target = root.join("claude/agents/review.md");
        assert!(fs::read_to_string(&target)?.contains("Version two."));

        write("Version three.")?;
        let preview = scan()?;
        assert_eq!(preview["candidates"][0]["status"].as_str(), Some("update"));
        let updated = import(&preview)?;
        assert!(fs::read_to_string(&target)?.contains("Version three."));
        let mut entry = Store::load(&path)?.entries.remove("review").unwrap();
        entry.prompt = "Local instructions.".into();
        call(Edit::Save {
            previous: Some((
                "review".into(),
                updated["entries"][0]["revision"].as_str().unwrap().into(),
            )),
            entry,
        })?;
        write("Version four.")?;
        let preview = scan()?;
        assert_eq!(preview["candidates"][0]["status"].as_str(), Some("edited"));
        let kept = import(&preview)?;
        assert_eq!(kept["entries"].as_array().unwrap().len(), 1);
        assert!(fs::read_to_string(target)?.contains("Local instructions."));
        Ok(())
    }
}
