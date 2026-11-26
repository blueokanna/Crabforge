use crate::{
    agents::NativeAgent, config::read_limited, project_instructions::replace,
    rules::resolved_folder,
};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

mod native;

const AGENTS: &[&str] = &["claude", "codex", "pi"];
const MAX_CONFIG: usize = 1024 * 1024;
const MAX_LIBRARY: usize = 4 * 1024 * 1024;
pub const MAX_VALUE: usize = 8192;
type Values = BTreeMap<String, String>;

#[derive(NsonDeserialize, NsonSerialize)]
struct Variable {
    name: String,
    value: String,
    agents: BTreeSet<String>,
}

impl Variable {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.name.is_empty()
                && self.name.len() <= 128
                && self
                    .name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                && !self.name.as_bytes()[0].is_ascii_digit(),
            "use up to 128 letters, digits or underscores, starting with a letter or underscore"
        );
        ensure!(
            self.value.len() <= MAX_VALUE && !self.value.contains('\0'),
            "invalid environment value"
        );
        ensure!(
            self.agents.iter().all(|id| AGENTS.contains(&id.as_str())),
            "unsupported environment client"
        );
        Ok(())
    }

    fn revision(&self) -> Result<String> {
        Ok(hash(&encode(self)?))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, NsonDeserialize, NsonSerialize)]
struct Destination {
    agent: String,
    path: String,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Applied {
    destination: Destination,
    values: Values,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Pending {
    after: Option<String>,
    applied: Applied,
}

#[derive(Default, NsonDeserialize, NsonSerialize)]
struct Store {
    variables: BTreeMap<String, Variable>,
    applied: Vec<Applied>,
    pending: Option<Pending>,
}

#[derive(NsonSerialize)]
struct Entry<'a> {
    name: &'a str,
    agents: &'a BTreeSet<String>,
    #[njson(skip_serializing_if = "Option::is_none")]
    value: Option<&'a str>,
    revision: String,
}

#[derive(NsonSerialize)]
struct Placement<'a> {
    name: String,
    target: &'a Destination,
    status: &'static str,
    detail: Option<String>,
}

#[derive(NsonSerialize)]
struct View<'a> {
    variables: Vec<Entry<'a>>,
    placements: Vec<Placement<'a>>,
}

pub enum Edit {
    Read,
    Save {
        name: String,
        value: String,
        agents: Vec<String>,
        previous: Option<(String, String)>,
    },
    Remove {
        name: String,
        revision: String,
    },
    Sync,
}

fn encode(value: &impl NsonSerialize) -> Result<String> {
    nextjson::to_string(value).map_err(anyhow::Error::msg)
}

fn hash(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn name_key(name: &str) -> String {
    if cfg!(windows) {
        name.to_ascii_uppercase()
    } else {
        name.to_owned()
    }
}

fn destination(agent: &str, home: &Path, file: &str) -> Result<Destination> {
    let path = std::path::absolute(home.join(file))?;
    let parent = resolved_folder(path.parent().context("missing environment config folder")?)?;
    let path = parent.join(
        path.file_name()
            .context("missing environment config filename")?,
    );
    Ok(Destination {
        agent: agent.to_owned(),
        path: path
            .into_os_string()
            .into_string()
            .ok()
            .context("environment config path is not UTF-8")?,
    })
}

fn destinations(dir: &Path) -> Result<BTreeSet<Destination>> {
    let claude = NativeAgent::Claude
        .home()
        .context("Claude Code profile is unavailable")?;
    let codex = NativeAgent::Codex
        .home()
        .context("Codex profile is unavailable")?;
    let pi = env::var_os("PI_CODING_AGENT_DIR")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::home_dir().map(|home| home.join(".pi/agent")))
        .context("Pi profile is unavailable")?;
    [
        ("claude", claude, "settings.json"),
        ("codex", codex, "config.toml"),
        ("pi", pi, "extensions/crabforge-env.ts"),
        ("pi", dir.join("agents/pi"), "extensions/crabforge-env.ts"),
    ]
    .into_iter()
    .map(|(agent, home, file)| destination(agent, &home, file))
    .collect()
}

fn read(path: &Path) -> Result<Option<String>> {
    let parent = path.parent().context("missing environment config folder")?;
    ensure!(
        resolved_folder(parent)? == parent,
        "environment config folder moved"
    );
    match fs::symlink_metadata(path) {
        Ok(metadata) => ensure!(
            metadata.is_file(),
            "expected a regular environment config file"
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    Ok(Some(
        String::from_utf8(read_limited(path, MAX_CONFIG)?)
            .map_err(|_| anyhow::anyhow!("environment settings must be UTF-8"))?,
    ))
}

fn write(path: &Path, before: Option<&str>, after: Option<&str>) -> Result<()> {
    let parent = path.parent().context("missing environment config folder")?;
    fs::create_dir_all(parent)?;
    let staged = if let Some(after) = after {
        ensure!(
            after.len() <= MAX_CONFIG,
            "environment config exceeds 1 MiB"
        );
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        if before.is_some() {
            file.as_file()
                .set_permissions(path.metadata()?.permissions())?;
        }
        file.write_all(after.as_bytes())?;
        file.as_file().sync_all()?;
        Some(file)
    } else {
        None
    };
    ensure!(
        resolved_folder(parent)? == parent && read(path)?.as_deref() == before,
        "native environment config changed; reload it"
    );
    match staged {
        Some(file) if before.is_some() => {
            file.persist(path)?;
        }
        Some(file) => {
            file.persist_noclobber(path)?;
        }
        None => fs::remove_file(path)?,
    }
    Ok(())
}

impl Store {
    fn load(path: &Path) -> Result<Self> {
        match read_limited(path, MAX_LIBRARY) {
            Ok(bytes) => nextjson::from_slice(&bytes)
                .map_err(|_| anyhow::anyhow!("invalid environment library")),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        let text = encode(self)?;
        ensure!(
            text.len() <= MAX_LIBRARY,
            "environment library exceeds 4 MiB"
        );
        replace(path, &text)
    }

    fn check(&self, name: &str, revision: &str) -> Result<()> {
        let variable = self
            .variables
            .get(&name_key(name))
            .context("environment variable was removed")?;
        ensure!(
            variable.revision()? == revision,
            "environment variable changed; reload the library"
        );
        Ok(())
    }

    fn set_applied(&mut self, applied: Applied) {
        self.applied
            .retain(|old| old.destination != applied.destination);
        if !applied.values.is_empty() {
            self.applied.push(applied);
        }
    }

    fn recover(&mut self, path: &Path, targets: &BTreeSet<Destination>) -> Result<()> {
        if let Some(pending) = &self.pending {
            let target = &pending.applied.destination;
            let published = targets.contains(target)
                && read(Path::new(&target.path))?.as_deref().map(hash) == pending.after;
            let pending = self.pending.take().expect("environment journal");
            if published {
                self.set_applied(pending.applied);
            }
            self.save(path)?;
        }
        Ok(())
    }
}

fn reconcile<'a>(
    store: &mut Store,
    path: &Path,
    targets: &'a BTreeSet<Destination>,
    target: &'a Destination,
    wanted: &Values,
    sync: bool,
) -> Result<Vec<Placement<'a>>> {
    let old = store
        .applied
        .iter()
        .find(|old| old.destination == *target)
        .map(|old| old.values.clone())
        .unwrap_or_default();
    if old.is_empty() && wanted.is_empty() {
        return Ok(Vec::new());
    }
    let config_path = Path::new(&target.path);
    let before = read(config_path)?;
    let mut native = native::Config::read(&target.agent, before.as_deref(), &old)?;
    let mut owned = old.clone();
    owned.retain(|key, value| native.values.get(key).and_then(Option::as_ref) == Some(value));
    let mut changes = BTreeMap::new();
    let mut placements = Vec::new();
    owned.retain(|key, _| {
        if wanted.contains_key(key) {
            return true;
        }
        native.values.remove(key);
        changes.insert(key.clone(), None);
        placements.push(Placement {
            name: key.clone(),
            target,
            status: if sync { "removed" } else { "pending" },
            detail: None,
        });
        false
    });
    let names: BTreeSet<_> = native.values.keys().map(|name| name_key(name)).collect();
    for (key, value) in wanted {
        let current = native.values.get(key).and_then(Option::as_deref);
        let status = if current == Some(value.as_str()) {
            if owned.contains_key(key) {
                "applied"
            } else {
                "existing"
            }
        } else if owned.contains_key(key) || !names.contains(&name_key(key)) {
            changes.insert(key.clone(), Some(value.clone()));
            owned.insert(key.clone(), value.clone());
            if sync { "applied" } else { "pending" }
        } else {
            "conflict"
        };
        placements.push(Placement {
            name: key.clone(),
            target,
            status,
            detail: None,
        });
    }
    if sync && !changes.is_empty() {
        ensure!(
            store.pending.is_none(),
            "another environment update is awaiting recovery"
        );
        let after = native.render(changes, &owned)?;
        store.pending = Some(Pending {
            after: after.as_deref().map(hash),
            applied: Applied {
                destination: target.clone(),
                values: owned,
            },
        });
        store.save(path)?;
        if let Err(error) = write(config_path, before.as_deref(), after.as_deref()) {
            store.pending = None;
            store.save(path)?;
            return Err(error);
        }
        store.recover(path, targets)?;
    } else if sync && owned != old {
        store.set_applied(Applied {
            destination: target.clone(),
            values: owned,
        });
        store.save(path)?;
    }
    Ok(placements)
}

fn edit_in(
    dir: &Path,
    targets: &BTreeSet<Destination>,
    edit: Edit,
    reveal: bool,
) -> Result<String> {
    let library = dir.join("environment");
    fs::create_dir_all(&library)?;
    let lock = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(library.join("library.lock"))?;
    lock.lock()?;
    let path = library.join("library.json");
    let mut store = Store::load(&path)?;
    let sync = !matches!(edit, Edit::Read);
    if sync {
        store.recover(&path, targets)?;
    }
    match edit {
        Edit::Read | Edit::Sync => {}
        Edit::Save {
            name,
            value,
            agents,
            previous,
        } => {
            let variable = Variable {
                name,
                value,
                agents: agents.into_iter().collect(),
            };
            variable.validate()?;
            let key = name_key(&variable.name);
            if let Some((name, revision)) = previous {
                store.check(&name, &revision)?;
                let previous = name_key(&name);
                ensure!(
                    previous == key || !store.variables.contains_key(&key),
                    "environment variable already exists"
                );
                store.variables.remove(&previous);
            } else {
                ensure!(
                    !store.variables.contains_key(&key),
                    "environment variable already exists"
                );
            }
            store.variables.insert(key, variable);
            store.save(&path)?;
        }
        Edit::Remove { name, revision } => {
            store.check(&name, &revision)?;
            store.variables.remove(&name_key(&name));
            store.save(&path)?;
        }
    }
    let mut placements = Vec::new();
    for target in targets {
        let wanted = store
            .variables
            .values()
            .filter(|entry| entry.agents.contains(&target.agent) && !entry.value.is_empty())
            .map(|entry| (entry.name.clone(), entry.value.clone()))
            .collect::<Values>();
        match reconcile(&mut store, &path, targets, target, &wanted, sync) {
            Ok(found) => placements.extend(found),
            Err(error) => {
                let old = store.applied.iter().find(|old| old.destination == *target);
                let names = wanted
                    .keys()
                    .chain(old.into_iter().flat_map(|old| old.values.keys()))
                    .collect::<BTreeSet<_>>();
                for name in names {
                    placements.push(Placement {
                        name: name.clone(),
                        target,
                        status: "error",
                        detail: Some(format!("{error:#}")),
                    });
                }
            }
        }
    }
    for old in &store.applied {
        if !targets.contains(&old.destination) {
            placements.extend(old.values.keys().map(|name| Placement {
                name: name.clone(),
                target: &old.destination,
                status: "pending",
                detail: Some("Select the original profile to synchronize this file".into()),
            }));
        }
    }
    let variables = store
        .variables
        .values()
        .map(|variable| {
            Ok(Entry {
                name: &variable.name,
                agents: &variable.agents,
                value: reveal.then_some(variable.value.as_str()),
                revision: variable.revision()?,
            })
        })
        .collect::<Result<_>>()?;
    encode(&View {
        variables,
        placements,
    })
}

pub fn edit(dir: &Path, change: Edit, reveal: bool) -> Result<String> {
    edit_in(dir, &destinations(dir)?, change, reveal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_a_committed_native_write_before_removing_it() -> Result<()> {
        let root = tempfile::tempdir()?;
        let dir = root.path();
        let library = dir.join("environment");
        fs::create_dir(&library)?;
        let path = library.join("library.json");
        let target = destination("claude", dir, "settings.json")?;
        let targets = BTreeSet::from([target.clone()]);
        let variable = Variable {
            name: "FIXTURE".into(),
            value: "owned".into(),
            agents: BTreeSet::from(["claude".into()]),
        };
        let revision = variable.revision()?;
        let native = r#"{"env":{"FIXTURE":"owned","EXISTING":"keep"}}"#;
        let store = Store {
            variables: BTreeMap::from([(variable.name.clone(), variable)]),
            applied: Vec::new(),
            pending: Some(Pending {
                after: Some(hash(native)),
                applied: Applied {
                    destination: target.clone(),
                    values: Values::from([("FIXTURE".into(), "owned".into())]),
                },
            }),
        };
        store.save(&path)?;
        fs::write(&target.path, native)?;

        let view = edit_in(
            dir,
            &targets,
            Edit::Remove {
                name: "FIXTURE".into(),
                revision,
            },
            false,
        )?;
        assert!(view.contains("\"status\":\"removed\""));
        assert_eq!(
            fs::read_to_string(target.path)?,
            r#"{"env":{"EXISTING":"keep"}}"#
        );
        let store = Store::load(&path)?;
        assert!(store.applied.is_empty() && store.pending.is_none());
        Ok(())
    }
}
