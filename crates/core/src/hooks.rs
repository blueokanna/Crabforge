use crate::{
    agents::NativeAgent, config::read_limited, native_settings, project_instructions::replace,
};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

mod antigravity;
mod backup;
mod dsh;
mod grok;
mod native;
mod project;
pub use project::Assignment as ProjectAssignment;

const MAX_LIBRARY: usize = 4 * 1024 * 1024;
pub const MAX_ENTRY: usize = 64 * 1024;

string_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Agent { Claude => "claude", Codex => "codex", Gemini => "gemini", Cursor => "cursor", Antigravity => "agy", Grok => "grok", Dsh => "dsh" }
}

impl Agent {
    fn parse(id: &str) -> Result<Self> {
        Self::from_wire(id).context("unsupported hook client")
    }

    fn id(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::Cursor => "cursor",
            Self::Antigravity => "agy",
            Self::Grok => "grok",
            Self::Dsh => "dsh",
        }
    }
}

#[derive(Clone, NsonDeserialize, NsonSerialize)]
struct Hook {
    name: String,
    #[njson(default)]
    description: String,
    event: String,
    #[njson(default)]
    matcher: String,
    command: String,
    #[njson(default)]
    timeout: u32,
    #[njson(default)]
    agents: BTreeSet<Agent>,
}

impl Hook {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.name.is_empty()
                && self.name.len() <= 64
                && self
                    .name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte)),
            "use 1–64 letters, digits, underscores or hyphens for the hook name"
        );
        ensure!(
            self.description.chars().count() <= 200
                && !self.description.contains(['\0', '\r', '\n']),
            "the hook description must be a single line of up to 200 characters"
        );
        ensure!(
            !self.command.trim().is_empty()
                && self.command.len() <= 8192
                && !self.command.contains('\0'),
            "invalid hook command"
        );
        ensure!(
            self.matcher.len() <= 1024 && !self.matcher.contains('\0') && self.timeout <= 3600,
            "invalid hook matcher or timeout"
        );
        native::validate_event(&self.event, &self.matcher)?;
        for &agent in &self.agents {
            native::encode(agent, self)?;
        }
        Ok(())
    }

    fn revision(&self) -> Result<String> {
        Ok(hash(&encode(self)?))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, NsonDeserialize, NsonSerialize)]
struct Target {
    agent: Agent,
    path: String,
}

#[derive(Clone, NsonDeserialize, NsonSerialize)]
struct Entry {
    event: String,
    #[njson(default, skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    value: Value,
    #[njson(default, skip_serializing_if = "Option::is_none")]
    equivalent: Option<Value>,
    #[njson(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Publication {
    target: Target,
    entries: BTreeMap<String, Entry>,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Pending {
    after: String,
    publication: Publication,
}

#[derive(Default, NsonDeserialize, NsonSerialize)]
struct Store {
    hooks: BTreeMap<String, Hook>,
    publications: Vec<Publication>,
    pending: Option<Pending>,
    #[njson(default, skip_serializing_if = "BTreeMap::is_empty")]
    projects: BTreeMap<String, project::Assignments>,
}

#[derive(NsonSerialize)]
struct Status<'a> {
    name: String,
    target: &'a Target,
    status: &'static str,
    detail: Option<String>,
}

#[derive(NsonSerialize)]
struct Found<'a> {
    key: String,
    target: &'a Target,
    event: String,
    #[njson(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    entry: Value,
    hook: Option<Hook>,
    removable: bool,
    #[njson(skip_serializing)]
    index: usize,
}

#[derive(NsonSerialize)]
struct Saved<'a> {
    hook: &'a Hook,
    revision: String,
}

#[derive(NsonSerialize)]
struct View<'a> {
    hooks: Vec<Saved<'a>>,
    placements: Vec<Status<'a>>,
    installed: Vec<Found<'a>>,
    errors: Vec<(&'a Target, String)>,
    events: BTreeMap<&'static str, Vec<&'static str>>,
    backups: Vec<backup::Backup>,
    #[njson(skip_serializing_if = "Vec::is_empty")]
    shared: Vec<project::Choice<'a>>,
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    project_errors: BTreeMap<String, String>,
}

pub enum Edit {
    Read,
    Save {
        source: String,
        previous: Option<(String, String)>,
    },
    Remove {
        name: String,
        revision: String,
    },
    Adopt {
        key: String,
        name: String,
    },
    RemoveInstalled {
        key: String,
    },
    Restore {
        id: String,
    },
    Assign(ProjectAssignment),
    Sync,
}

fn encode(value: &impl NsonSerialize) -> Result<String> {
    nextjson::to_string(value).map_err(anyhow::Error::msg)
}

fn hash(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn profile(key: &str, fallback: PathBuf) -> PathBuf {
    env::var_os(key)
        .filter(|path| !path.is_empty())
        .map_or(fallback, PathBuf::from)
}

fn targets(dir: &Path) -> Result<BTreeMap<Target, &'static str>> {
    let home = env::home_dir().context("user profile is unavailable")?;
    let claude = NativeAgent::Claude
        .home()
        .context("Claude Code profile is unavailable")?;
    let codex = NativeAgent::Codex
        .home()
        .context("Codex profile is unavailable")?;
    let gemini = profile("GEMINI_CLI_HOME", home.clone());
    let cursor = profile("CURSOR_HOME", home.join(".cursor"));
    let grok = profile("GROK_HOME", home.join(".grok"));
    let mut targets = [
        (Agent::Claude, claude.join("settings.json"), "{}\n"),
        (Agent::Codex, codex.join("hooks.json"), "{}\n"),
        (
            Agent::Antigravity,
            home.join(".gemini/config/hooks.json"),
            "{}\n",
        ),
        (Agent::Grok, grok.join("hooks").join(grok::FILE), "{}\n"),
        (
            Agent::Cursor,
            cursor.join("hooks.json"),
            "{\"version\":1}\n",
        ),
        (Agent::Gemini, gemini.join(".gemini/settings.json"), "{}\n"),
        (
            Agent::Gemini,
            dir.join("agents/gemini/.gemini/settings.json"),
            "{\"security\":{\"auth\":{\"selectedType\":\"gemini-api-key\"}}}\n",
        ),
    ]
    .into_iter()
    .map(|(agent, path, default)| {
        let path = native_settings::path(&path)?
            .into_os_string()
            .into_string()
            .ok()
            .context("native hook path is not UTF-8")?;
        Ok((Target { agent, path }, default))
    })
    .collect::<Result<BTreeMap<_, _>>>()?;
    grok::discover_targets(&mut targets)?;
    dsh::discover(&profile("DSH_HOME", home.join(".dsh")), &mut targets)?;
    Ok(targets)
}

impl Store {
    fn load(path: &Path) -> Result<Self> {
        match read_limited(path, MAX_LIBRARY) {
            Ok(bytes) => {
                nextjson::from_slice(&bytes).map_err(|_| anyhow::anyhow!("invalid hook library"))
            }
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        let text = encode(self)?;
        ensure!(text.len() <= MAX_LIBRARY, "hook library exceeds 4 MiB");
        replace(path, &text)
    }

    fn check(&self, name: &str, revision: &str) -> Result<()> {
        let hook = self.hooks.get(name).context("hook was removed")?;
        ensure!(
            hook.revision()? == revision,
            "hook changed; reload the library"
        );
        Ok(())
    }

    fn entries(&self, target: &Target) -> Option<&BTreeMap<String, Entry>> {
        self.publications
            .iter()
            .find(|item| item.target == *target)
            .map(|item| &item.entries)
    }

    fn record(&mut self, publication: Publication) {
        self.publications
            .retain(|item| item.target != publication.target);
        if !publication.entries.is_empty() {
            self.publications.push(publication);
        }
    }

    fn recover(&mut self, path: &Path, targets: &BTreeMap<Target, &str>) -> Result<()> {
        if let Some(pending) = &self.pending {
            let target = &pending.publication.target;
            let applied = targets.contains_key(target)
                && native_settings::read(Path::new(&target.path))?
                    .is_some_and(|text| hash(&text) == pending.after);
            let pending = self.pending.take().expect("hook journal");
            if applied {
                self.record(pending.publication);
            }
            self.save(path)?;
        }
        Ok(())
    }

    fn commit(
        &mut self,
        path: &Path,
        publication: Publication,
        before: Option<&str>,
        after: &str,
    ) -> Result<()> {
        ensure!(
            self.pending.is_none(),
            "another hook update is awaiting recovery"
        );
        let native_path = PathBuf::from(&publication.target.path);
        self.pending = Some(Pending {
            after: hash(after),
            publication,
        });
        self.save(path)?;
        if let Err(error) = native_settings::write(&native_path, before, Some(after)) {
            self.pending = None;
            self.save(path)?;
            return Err(error);
        }
        let pending = self.pending.take().expect("hook journal");
        self.record(pending.publication);
        self.save(path)
    }
}

fn indexes(entries: &native::Locations, entry: &Entry) -> Vec<usize> {
    entries
        .get(&(entry.group.clone(), entry.event.clone()))
        .into_iter()
        .flat_map(|location| &location.entries)
        .enumerate()
        .filter_map(|(index, value)| (*value == entry.value).then_some(index))
        .collect()
}

fn discover<'a>(
    target: &'a Target,
    source: &str,
    claimed: Option<&BTreeMap<String, Entry>>,
) -> Result<Vec<Found<'a>>> {
    let mut output = Vec::new();
    let revision = hash(source);
    let events = native::read_locations(target.agent, source)?;
    for ((group, event), location) in &events {
        let entries = &location.entries;
        for (index, entry) in entries.iter().enumerate() {
            if target.agent == Agent::Codex && *entry == nextjson::json!({"hooks": []}) {
                continue;
            }
            let managed = claimed.is_some_and(|claimed| {
                claimed.values().any(|owned| {
                    owned.group == *group && owned.event == *event && owned.value == *entry
                })
            });
            if managed && entries.iter().filter(|value| *value == entry).count() == 1 {
                continue;
            }
            let hook = if location.enabled {
                native::decode(target.agent, event, entry).filter(|hook| hook.validate().is_ok())
            } else {
                None
            };
            let key = hash(&match group {
                Some(group) => encode(&(target, &revision, group, event, index))?,
                None => encode(&(target, &revision, event, index))?,
            });
            output.push(Found {
                key,
                target,
                event: event.clone(),
                group: group.clone(),
                entry: entry.clone(),
                hook,
                removable: !managed,
                index,
            });
        }
    }
    Ok(output)
}

fn reconcile<'a>(
    store: &mut Store,
    path: &Path,
    target: &'a Target,
    default: &str,
    sync: bool,
    wanted: BTreeMap<String, Hook>,
) -> Result<Vec<Status<'a>>> {
    let previous = store.entries(target).cloned().unwrap_or_default();
    let selected = !wanted.is_empty();
    let dsh = target.agent == Agent::Dsh;
    let empty = !selected && previous.is_empty();
    if empty && !(dsh && sync) {
        return Ok(Vec::new());
    }
    let before = native_settings::read(Path::new(&target.path))?;
    if empty && before.is_none() {
        return Ok(Vec::new());
    }
    let original = before.as_deref().unwrap_or(default);
    let mut source = original.to_owned();
    let mut events = native::read_locations(target.agent, &source)?;
    let mut placements = Vec::new();
    let mut managed = BTreeMap::new();
    let mut wanted = wanted;
    let mut groups = BTreeMap::new();
    for (id, mut owned) in previous {
        let selected = wanted.remove(&id);
        let name = if let Some(hook) = &selected {
            owned.name = (hook.name != id).then(|| hook.name.clone());
            hook.name.clone()
        } else {
            owned.name.clone().unwrap_or_else(|| id.clone())
        };
        match indexes(&events, &owned).as_slice() {
            [index] => {
                if let Some(hook) = &selected {
                    let (event, value) = native::encode(target.agent, hook)?;
                    if let Some(group) = owned.group.as_deref() {
                        if !events[&(owned.group.clone(), owned.event.clone())].enabled {
                            placements.push(Status {
                                name,
                                target,
                                status: "conflict",
                                detail: None,
                            });
                            managed.insert(id, owned);
                            continue;
                        }
                        groups.insert(id.clone(), group.to_owned());
                    }
                    if event == owned.event
                        && &native::canonical(&value)
                            == owned.equivalent.as_ref().unwrap_or(&owned.value)
                    {
                        placements.push(Status {
                            name: name.clone(),
                            target,
                            status: "applied",
                            detail: None,
                        });
                        managed.insert(id, owned);
                        continue;
                    }
                    wanted.insert(id, hook.clone());
                } else {
                    placements.push(Status {
                        name: name.clone(),
                        target,
                        status: if sync { "removed" } else { "pending" },
                        detail: None,
                    });
                }
                source = native::remove(
                    target.agent,
                    &source,
                    owned.group.as_deref(),
                    &owned.event,
                    *index,
                )?;
                events = native::read_locations(target.agent, &source)?;
            }
            _ => {
                placements.push(Status {
                    name: name.clone(),
                    target,
                    status: "conflict",
                    detail: None,
                });
                if selected.is_some() {
                    managed.insert(id, owned);
                }
            }
        }
    }
    for (id, hook) in wanted {
        let name = hook.name.clone();
        let (event, value) = native::encode(target.agent, &hook)?;
        let entry = Entry {
            event,
            group: (target.agent == Agent::Antigravity).then(|| {
                groups
                    .remove(&id)
                    .unwrap_or_else(|| antigravity::GROUP.into())
            }),
            value: native::canonical(&value),
            equivalent: None,
            name: (name != id).then(|| name.clone()),
        };
        let equivalent = events.iter().any(|((_, event), location)| {
            *event == entry.event && location.enabled && location.entries.contains(&entry.value)
        });
        let status = if !equivalent {
            if target.agent == Agent::Cursor {
                source = native::cursor_version(&source)?;
            }
            source = match &entry.group {
                Some(group) => antigravity::add(&source, group, &entry.event, value)?,
                None => native::add(&source, &entry.event, value)?,
            };
            events = native::read_locations(target.agent, &source)?;
            managed.insert(id, entry);
            if sync { "applied" } else { "pending" }
        } else {
            "existing"
        };
        placements.push(Status {
            name,
            target,
            status,
            detail: None,
        });
    }
    let patch = if dsh {
        let active = events.values().any(|location| !location.entries.is_empty());
        if sync && (!empty || !active) {
            dsh::plan(target, active, selected)?
        } else {
            if !sync && selected && dsh::plan(target, true, false)?.is_some() {
                for row in &mut placements {
                    if row.status == "applied" {
                        row.status = "pending";
                    }
                }
            }
            None
        }
    } else {
        None
    };
    if sync && !empty {
        let publication = Publication {
            target: target.clone(),
            entries: managed,
        };
        if source != original {
            store.commit(path, publication, before.as_deref(), &source)?;
        } else {
            store.record(publication);
            store.save(path)?;
        }
    }
    if let Some(patch) = patch {
        patch.apply()?;
    }
    Ok(placements)
}

fn edit_in(dir: &Path, targets: &BTreeMap<Target, &str>, edit: Edit) -> Result<String> {
    let _lock = native_settings::lock(dir)?;
    let library = dir.join("hooks");
    fs::create_dir_all(&library)?;
    let path = library.join("library.json");
    let backups = library.join("backups");
    let mut store = Store::load(&path)?;
    let sync = !matches!(
        edit,
        Edit::Read | Edit::RemoveInstalled { .. } | Edit::Restore { .. }
    );
    if !matches!(edit, Edit::Read) {
        store.recover(&path, targets)?;
    }
    match edit {
        Edit::Read | Edit::Sync => {}
        Edit::Assign(_) => anyhow::bail!("select a project to assign a hook"),
        Edit::Save { source, previous } => {
            ensure!(source.len() <= MAX_ENTRY, "hook entry exceeds 64 KiB");
            let hook: Hook =
                nextjson::from_str(&source).map_err(|_| anyhow::anyhow!("invalid hook entry"))?;
            hook.validate()?;
            ensure!(
                !hook.agents.contains(&Agent::Dsh)
                    || targets.keys().any(|target| target.agent == Agent::Dsh),
                "initialize a DeepSeek Harness profile before assigning hooks to it"
            );
            if let Some((name, revision)) = previous {
                store.check(&name, &revision)?;
                ensure!(
                    name == hook.name || !store.hooks.contains_key(&hook.name),
                    "hook name already exists"
                );
                project::update(&mut store.projects, &name, &hook)?;
                store.hooks.remove(&name);
                for publication in &mut store.publications {
                    if let Some(entry) = publication.entries.remove(&name) {
                        publication.entries.insert(hook.name.clone(), entry);
                    }
                }
            } else {
                ensure!(
                    !store.hooks.contains_key(&hook.name),
                    "hook name already exists"
                );
            }
            store.hooks.insert(hook.name.clone(), hook);
            store.save(&path)?;
        }
        Edit::Remove { name, revision } => {
            store.check(&name, &revision)?;
            store.hooks.remove(&name);
            for project in store.projects.values_mut() {
                project.hooks.remove(&name);
            }
            store.save(&path)?;
        }
        Edit::RemoveInstalled { key } => remove_installed(targets, &store, &backups, &key)?,
        Edit::Restore { id } => backup::restore(&backups, targets, &id)?,
        Edit::Adopt { key, name } => {
            ensure!(!store.hooks.contains_key(&name), "hook name already exists");
            let (found, before) = find_installed(targets, &store, &key)?;
            let target = found.target;
            let mut hook = found
                .hook
                .context("the native hook cannot be represented as a single command")?;
            hook.name = name;
            hook.validate()?;
            let (_, encoded) = native::encode(target.agent, &hook)?;
            let encoded = native::canonical(&encoded);
            let entry = Entry {
                event: found.event,
                group: found.group,
                equivalent: (encoded != found.entry).then_some(encoded),
                value: found.entry,
                name: None,
            };
            ensure!(
                indexes(&native::read_locations(target.agent, &before)?, &entry).len() == 1,
                "identical native hooks cannot be adopted individually"
            );
            ensure!(
                native_settings::read(Path::new(&target.path))?.as_deref() == Some(before.as_str()),
                "native hooks changed; rediscover them"
            );
            let mut entries = store.entries(target).cloned().unwrap_or_default();
            entries.insert(hook.name.clone(), entry);
            store.record(Publication {
                target: target.clone(),
                entries,
            });
            store.hooks.insert(hook.name.clone(), hook);
            store.save(&path)?;
        }
    }
    let project_errors = if sync {
        project::sync(&library, &store, targets)
    } else {
        BTreeMap::new()
    };
    render(&library, targets, &mut store, sync, None, project_errors)
}

fn wanted(hooks: &BTreeMap<String, Hook>, agent: Agent) -> BTreeMap<String, Hook> {
    hooks
        .iter()
        .filter(|(_, hook)| hook.agents.contains(&agent))
        .map(|(name, hook)| (name.clone(), hook.clone()))
        .collect()
}

fn target_wanted(
    hooks: &BTreeMap<String, Hook>,
    store: &Store,
    target: &Target,
) -> BTreeMap<String, Hook> {
    let mut wanted = wanted(hooks, target.agent);
    if target.agent == Agent::Grok {
        if grok::is_managed(target) {
            for publication in &store.publications {
                let sibling = &publication.target;
                if grok::is_external(sibling)
                    && Path::new(&sibling.path).parent() == Path::new(&target.path).parent()
                {
                    wanted.retain(|name, _| !publication.entries.contains_key(name));
                }
            }
        } else {
            let adopted = store.entries(target);
            wanted.retain(|name, _| adopted.is_some_and(|entries| entries.contains_key(name)));
        }
    }
    wanted
}

fn render(
    library: &Path,
    targets: &BTreeMap<Target, &str>,
    store: &mut Store,
    sync: bool,
    project: Option<project::View<'_>>,
    project_errors: BTreeMap<String, String>,
) -> Result<String> {
    let path = library.join("library.json");
    let mut placements = Vec::new();
    let mut installed = Vec::new();
    let mut errors = Vec::new();
    for (target, default) in targets {
        let wanted = target_wanted(
            project
                .as_ref()
                .map_or(&store.hooks, |project| &project.wanted),
            store,
            target,
        );
        match reconcile(store, &path, target, default, sync, wanted) {
            Ok(rows) => placements.extend(rows),
            Err(error) => {
                errors.push((target, format!("{error:#}")));
                continue;
            }
        }
        match native_settings::read(Path::new(&target.path)).and_then(|text| {
            text.map(|text| discover(target, &text, store.entries(target)))
                .transpose()
        }) {
            Ok(Some(rows)) => installed.extend(rows),
            Ok(None) => {}
            Err(error) => errors.push((target, format!("{error:#}"))),
        }
    }
    for publication in &store.publications {
        if !targets.contains_key(&publication.target) {
            placements.extend(publication.entries.keys().map(|name| Status {
                name: name.clone(),
                target: &publication.target,
                status: "pending",
                detail: Some("Select the original profile to synchronize this file".into()),
            }));
        }
    }
    let hooks = project
        .as_ref()
        .map_or(&store.hooks, |project| &project.wanted)
        .values()
        .map(|hook| {
            Ok(Saved {
                hook,
                revision: hook.revision()?,
            })
        })
        .collect::<Result<_>>()?;
    encode(&View {
        hooks,
        placements,
        installed,
        errors,
        backups: backup::list(&library.join("backups"), targets)?,
        shared: project
            .as_ref()
            .map_or_else(Vec::new, |project| project.shared.clone()),
        project_errors,
        events: targets
            .keys()
            .map(|target| (target.agent.id(), native::native_events(target.agent)))
            .collect(),
    })
}

fn remove_installed(
    targets: &BTreeMap<Target, &str>,
    store: &Store,
    backups: &Path,
    key: &str,
) -> Result<()> {
    let (found, before) = find_installed(targets, store, key)?;
    ensure!(
        found.removable,
        "cannot distinguish this hook from a managed copy"
    );
    let after = native::remove(
        found.target.agent,
        &before,
        found.group.as_deref(),
        &found.event,
        found.index,
    )?;
    let event = found.group.map_or(found.event.clone(), |group| {
        format!("{group} · {}", found.event)
    });
    backup::save(backups, found.target, event, &before, &after)?;
    native_settings::write(Path::new(&found.target.path), Some(&before), Some(&after))
}

fn find_installed<'a>(
    targets: &'a BTreeMap<Target, &str>,
    store: &Store,
    key: &str,
) -> Result<(Found<'a>, String)> {
    for target in targets.keys() {
        let Ok(Some(before)) = native_settings::read(Path::new(&target.path)) else {
            continue;
        };
        let Ok(found) = discover(target, &before, store.entries(target)) else {
            continue;
        };
        if let Some(found) = found.into_iter().find(|found| found.key == key) {
            return Ok((found, before));
        }
    }
    anyhow::bail!("native hook changed; rediscover it")
}

pub fn edit(dir: &Path, change: Edit) -> Result<String> {
    edit_in(dir, &targets(dir)?, change)
}

pub fn edit_project(dir: &Path, root: &Path, change: Edit) -> Result<String> {
    let _lock = native_settings::lock(dir)?;
    project::edit(&dir.join("hooks"), root, &targets(dir)?, change)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grok_adoption_uses_the_discovered_file_without_publishing_a_copy() -> Result<()> {
        let root = tempfile::tempdir()?;
        let folder = root.path().canonicalize()?.join("hooks");
        fs::create_dir(&folder)?;
        let managed = folder.join(grok::FILE);
        let existing = folder.join("personal.json");
        let source =
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"echo personal"}]}]}}"#;
        fs::write(&existing, source)?;
        let mut targets = BTreeMap::from([(
            Target {
                agent: Agent::Grok,
                path: managed.to_string_lossy().into_owned(),
            },
            "{}\n",
        )]);
        grok::discover_targets(&mut targets)?;
        let view = |change| -> Result<Value> {
            nextjson::from_str(&edit_in(&root.path().join("data"), &targets, change)?)
                .map_err(anyhow::Error::msg)
        };
        let key = view(Edit::Read)?["installed"][0]["key"]
            .as_str()
            .unwrap()
            .to_owned();
        let adopted = view(Edit::Adopt {
            key,
            name: "personal".into(),
        })?;
        assert_eq!(adopted["placements"].as_array().unwrap().len(), 1);
        assert_eq!(fs::read_to_string(&existing)?, source);
        assert!(!managed.exists());

        fs::remove_file(&existing)?;
        let synced = view(Edit::Sync)?;
        assert_eq!(synced["placements"][0]["status"].as_str(), Some("conflict"));
        assert!(!managed.exists());
        assert!(!existing.exists());
        Ok(())
    }

    #[test]
    fn antigravity_preserves_named_hooks_and_restores_native_bytes() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = native_settings::path(&root.path().join("hooks.json"))?;
        let source = concat!(
            "\u{feff}{\r\n",
            "  // Existing hook\r\n",
            "  \"audit\": {\"enabled\": false, \"SessionStart\": [{\"command\": \"echo audit\"}]},\r\n",
            "  \"linter\": {\"PostToolUse\": [{\"matcher\": \"run_command\", \"hooks\": [{\"command\": \"echo lint\"}]}]}\r\n",
            "}\r\n",
        );
        fs::write(&path, source)?;
        let targets = BTreeMap::from([(
            Target {
                agent: Agent::Antigravity,
                path: path.to_string_lossy().into_owned(),
            },
            "{}\n",
        )]);
        let parse = |text: String| nextjson::from_str::<Value>(&text).map_err(anyhow::Error::msg);
        let read = parse(edit_in(root.path(), &targets, Edit::Read)?)?;
        let existing = read["installed"].as_array().unwrap();
        assert_eq!(existing[0]["group"].as_str(), Some("audit"));
        assert!(existing[0]["hook"].is_null());
        assert_eq!(existing[1]["hook"]["command"].as_str(), Some("echo lint"));
        let key = existing[1]["key"].as_str().unwrap().to_owned();
        let removed = parse(edit_in(
            root.path(),
            &targets,
            Edit::RemoveInstalled { key },
        )?)?;
        let id = removed["backups"][0]["id"].as_str().unwrap().to_owned();
        assert!(!fs::read_to_string(&path)?.contains("echo lint"));
        edit_in(root.path(), &targets, Edit::Restore { id })?;
        assert_eq!(fs::read_to_string(&path)?, source);

        let hook = nextjson::json!({"name":"marker","event":"session-start","command":"echo start","agents":["agy"],"timeout":5});
        edit_in(
            root.path(),
            &targets,
            Edit::Save {
                source: hook.to_string(),
                previous: None,
            },
        )?;
        let published = fs::read_to_string(&path)?;
        assert!(published.starts_with(source.trim_end().strip_suffix('}').unwrap().trim_end()));
        let events = native::read_locations(Agent::Antigravity, &published)?;
        let entries = &events[&(Some("crabforge".into()), "SessionStart".into())].entries;
        assert_eq!(entries[0]["command"].as_str(), Some("echo start"));
        assert_eq!(entries[0]["timeout"].as_u64(), Some(5));
        Ok(())
    }
    use nextjson::json;

    fn fixture(dir: &Path, agent: Agent) -> (Target, BTreeMap<Target, &'static str>) {
        let path = native_settings::path(&dir.join("settings.json")).unwrap();
        let target = Target {
            agent,
            path: path.to_str().unwrap().into(),
        };
        (target.clone(), BTreeMap::from([(target, "{}\n")]))
    }

    fn run(dir: &Path, targets: &BTreeMap<Target, &str>, edit: Edit) -> Value {
        nextjson::from_str(&edit_in(dir, targets, edit).unwrap()).unwrap()
    }

    fn save(name: &str, agent: Agent, command: &str) -> Edit {
        Edit::Save {
            source: json!({"name": name, "event": "session-start", "command": command, "agents": [agent.id()]}).to_string(),
            previous: None,
        }
    }

    #[test]
    fn adoption_keeps_native_bytes_and_external_edits() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let (target, targets) = fixture(dir, Agent::Claude);
        let path = Path::new(&target.path);
        let original = "\u{feff}{\r\n  // Keep this.\r\n  \"env\": {\"OTHER\": \"yes\"},\r\n  \"hooks\": {\"SessionStart\": [/* keep */{\"matcher\":\"\",\"hooks\":[{\"type\":\"command\",\"command\":\"echo native\"}]}]},\r\n}\r\n";
        fs::write(path, original).unwrap();
        let before = run(dir, &targets, Edit::Read);
        let key = before["installed"][0]["key"].as_str().unwrap().to_owned();
        fs::write(path, original.replace("Keep this", "Changed")).unwrap();
        assert!(
            edit_in(
                dir,
                &targets,
                Edit::Adopt {
                    key,
                    name: "probe".into()
                }
            )
            .is_err()
        );

        fs::write(path, original).unwrap();
        let key = before["installed"][0]["key"].as_str().unwrap().to_owned();
        let adopted = run(
            dir,
            &targets,
            Edit::Adopt {
                key,
                name: "probe".into(),
            },
        );
        assert_eq!(fs::read_to_string(path).unwrap(), original);
        assert_eq!(adopted["placements"][0]["status"].as_str(), Some("applied"));
        assert!(adopted["installed"].as_array().unwrap().is_empty());

        let edited = original.replace("echo native", "echo changed");
        fs::write(path, &edited).unwrap();
        let removed = run(
            dir,
            &targets,
            Edit::Remove {
                name: "probe".into(),
                revision: adopted["hooks"][0]["revision"].as_str().unwrap().into(),
            },
        );
        assert_eq!(
            removed["placements"][0]["status"].as_str(),
            Some("conflict")
        );
        assert_eq!(fs::read_to_string(path).unwrap(), edited);
    }

    #[test]
    fn codex_removal_keeps_later_group_indexes_and_comments() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let (target, targets) = fixture(dir, Agent::Codex);
        let path = Path::new(&target.path);
        let saved = run(dir, &targets, save("first", Agent::Codex, "echo managed"));
        let source = fs::read_to_string(path).unwrap();
        let later =
            json!({"matcher":"resume","hooks":[{"type":"command","command":"echo external"}]});
        let edited = native::add(&source, "SessionStart", later.clone())
            .unwrap()
            .replace(
                "{\"matcher\":\"resume\"",
                "/* external */{\"matcher\":\"resume\"",
            );
        assert!(edited.contains("/* external */"));
        fs::write(path, &edited).unwrap();
        run(
            dir,
            &targets,
            Edit::Remove {
                name: "first".into(),
                revision: saved["hooks"][0]["revision"].as_str().unwrap().into(),
            },
        );
        let removed = fs::read_to_string(path).unwrap();
        assert!(removed.contains("/* external */"));
        assert_eq!(
            native::read(&removed).unwrap()["SessionStart"],
            [json!({"hooks": []}), native::canonical(&later)]
        );
        run(dir, &targets, save("second", Agent::Codex, "echo new"));
        let new = native::read(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(new["SessionStart"].len(), 3);
        assert_eq!(new["SessionStart"][1], native::canonical(&later));
    }

    #[test]
    fn installed_hook_removal_restores_exact_bytes_and_checks_revisions() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let dir = tmp.path();
        let (target, targets) = fixture(dir, Agent::Codex);
        let path = Path::new(&target.path);
        let original = concat!(
            "\u{feff}{\r\n  // Keep this comment.\r\n",
            "  \"hooks\": {\"SessionStart\": [",
            "{\"hooks\":[{\"type\":\"command\",\"command\":\"echo first\"}]}, ",
            "{\"hooks\":[{\"type\":\"command\",\"command\":\"echo later\"}]}]},\r\n}\r\n"
        );
        fs::write(path, original)?;
        let view = run(dir, &targets, Edit::Read);
        let key = view["installed"][0]["key"].as_str().unwrap();
        let remove = || Edit::RemoveInstalled { key: key.into() };
        fs::write(path, original.replace("Keep this", "Changed this"))?;
        assert!(edit_in(dir, &targets, remove()).is_err());
        assert!(!dir.join("hooks/backups").exists());
        fs::write(path, original)?;

        let removed = run(dir, &targets, remove());
        let after = fs::read_to_string(path)?;
        let groups = native::read(&after)?;
        assert_eq!(groups["SessionStart"][0], json!({"hooks": []}));
        assert_eq!(
            groups["SessionStart"][1]["hooks"][0]["command"].as_str(),
            Some("echo later")
        );
        assert_eq!(removed["backups"][0]["status"].as_str(), Some("ready"));
        let id = removed["backups"][0]["id"].as_str().unwrap();
        let restore = || Edit::Restore { id: id.into() };
        let changed = after.replace("echo later", "echo external edit");
        fs::write(path, &changed)?;
        assert!(edit_in(dir, &targets, restore()).is_err());
        assert_eq!(fs::read_to_string(path)?, changed);
        assert_eq!(
            run(dir, &targets, Edit::Read)["backups"][0]["status"].as_str(),
            Some("conflict")
        );
        fs::write(path, after)?;
        assert!(edit_in(dir, &BTreeMap::new(), restore()).is_err());
        let restored = run(dir, &targets, restore());
        assert_eq!(restored["backups"][0]["status"].as_str(), Some("restored"));
        assert_eq!(fs::read(path)?, original.as_bytes());
        assert_eq!(restored["installed"].as_array().unwrap().len(), 2);
        run(dir, &targets, restore());
        assert_eq!(fs::read(path)?, original.as_bytes());
        Ok(())
    }

    #[test]
    fn installed_hook_removal_rejects_ambiguous_managed_copies() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let dir = tmp.path();
        let (target, targets) = fixture(dir, Agent::Claude);
        let path = Path::new(&target.path);
        run(dir, &targets, save("probe", Agent::Claude, "echo managed"));
        let source = fs::read_to_string(path)?;
        let row = native::read(&source)?["SessionStart"][0].clone();
        let duplicated = native::add(&source, "SessionStart", row)?;
        fs::write(path, &duplicated)?;
        let view = run(dir, &targets, Edit::Read);
        assert_eq!(view["installed"][0]["removable"], json!(false));
        let key = view["installed"][0]["key"].as_str().unwrap().into();
        assert!(edit_in(dir, &targets, Edit::RemoveInstalled { key }).is_err());
        assert_eq!(fs::read_to_string(path)?, duplicated);
        Ok(())
    }

    #[test]
    fn synchronization_recovers_a_committed_hook_write() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let (target, targets) = fixture(dir, Agent::Claude);
        let saved = run(dir, &targets, save("probe", Agent::Claude, "echo managed"));
        let path = dir.join("hooks/library.json");
        let mut store = Store::load(&path).unwrap();
        let publication = store.publications.pop().unwrap();
        let native_path = Path::new(&target.path);
        let after = fs::read_to_string(native_path).unwrap();
        store.pending = Some(Pending {
            after: hash(&after),
            publication,
        });
        store.save(&path).unwrap();

        let removal = run(
            dir,
            &targets,
            Edit::Remove {
                name: "probe".into(),
                revision: saved["hooks"][0]["revision"].as_str().unwrap().into(),
            },
        );
        assert_eq!(removal["placements"][0]["status"].as_str(), Some("removed"));
        assert!(
            !fs::read_to_string(native_path)
                .unwrap()
                .contains("echo managed")
        );
        assert!(Store::load(&path).unwrap().pending.is_none());
    }
}
