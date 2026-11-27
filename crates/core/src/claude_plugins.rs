use crate::{
    agents::NativeAgent, native_settings, project_instructions::replace, project_mcp::jsonc,
};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    io::{ErrorKind, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

mod transfer;
pub use transfer::{export_settings, import_magpie, import_settings, import_settings_file};

pub(crate) const ENABLED: &str = "enabledPlugins";
pub(crate) const MARKETPLACES: &str = "extraKnownMarketplaces";
const EMPTY: &str = "{}\n";

pub enum Edit<'a> {
    Read,
    Set {
        id: &'a str,
        revision: &'a str,
        enabled: bool,
    },
}

pub enum Manage {
    AddMarketplace,
    UpdateMarketplace,
    RemoveMarketplace,
    Install,
    Update,
    Uninstall,
}

#[derive(NsonSerialize)]
struct Plugin<'a> {
    id: &'a str,
    versions: BTreeSet<&'a str>,
    scopes: BTreeSet<&'a str>,
    enabled: bool,
    managed: bool,
    can_enable: bool,
    can_disable: bool,
}

#[derive(NsonSerialize)]
struct View<'a> {
    plugins: Vec<Plugin<'a>>,
    marketplaces: BTreeSet<&'a str>,
    revision: String,
}

#[derive(Default, NsonSerialize, NsonDeserialize)]
struct Ownership {
    entries: BTreeSet<String>,
    pending: Option<Pending>,
}

#[derive(NsonSerialize, NsonDeserialize)]
struct Pending {
    id: String,
    enabled: bool,
    after: String,
}

fn encode(value: &impl NsonSerialize) -> Result<String> {
    nextjson::to_string(value).map_err(anyhow::Error::msg)
}

fn hash(source: &str) -> String {
    hex::encode(Sha256::digest(source.as_bytes()))
}

fn read(path: &Path) -> Result<Option<String>> {
    native_settings::read(&native_settings::path(path)?)
}

pub(crate) fn read_object(path: &Path) -> Result<Value> {
    let source = read(path)?;
    let value = nextjson::from_str::<Value>(
        source
            .as_deref()
            .unwrap_or(EMPTY)
            .trim_start_matches('\u{feff}'),
    )
    .map_err(anyhow::Error::msg)
    .with_context(|| format!("read Claude Code plugin metadata at {}", path.display()))?;
    ensure!(
        value.is_object(),
        "expected a Claude Code plugin metadata object"
    );
    Ok(value)
}

fn valid_id(id: &str) -> bool {
    id.split_once('@').is_some_and(|(name, marketplace)| {
        !name.is_empty() && !marketplace.is_empty() && !marketplace.contains('@')
    }) && id.len() <= 256
        && !id.chars().any(|ch| ch.is_control() || ch.is_whitespace())
}

fn enabled(source: &str) -> Result<BTreeMap<String, &str>> {
    Ok(jsonc::parse(source, &[ENABLED])?
        .into_iter()
        .flat_map(|object| object.entries(source))
        .collect())
}

fn patch(source: &str, id: &str, value: Option<bool>) -> Result<String> {
    let value = value.map(Value::from);
    patch_setting(source, ENABLED, id, value)
}

fn patch_setting(source: &str, field: &str, id: &str, value: Option<Value>) -> Result<String> {
    if let Some(map) = jsonc::parse(source, &[field])? {
        map.edit(source, id, value.as_ref())
    } else if let Some(value) = value {
        jsonc::parse(source, &[])?
            .context("missing Claude Code settings")?
            .edit(
                source,
                field,
                Some(&Value::Object(nextjson::Map::from_iter([(id, value)]))),
            )
    } else {
        Ok(source.to_owned())
    }
}

pub(crate) fn project_target(project: &Path, home: &Path) -> Result<(PathBuf, PathBuf)> {
    let root = project
        .canonicalize()
        .context("project directory is unavailable")?;
    ensure!(root.is_dir(), "expected a project directory");
    let local = root.join(".claude/settings.local.json");
    let shared = root.join(".claude/settings.json");
    ensure!(
        native_settings::path(&local)? == local && native_settings::path(&shared)? == shared,
        "project Claude Code settings folder was redirected"
    );
    ensure!(
        native_settings::path(&home.join("settings.json"))? != shared,
        "project Claude Code settings overlap the user profile"
    );
    Ok((root, local))
}

impl Ownership {
    fn load(path: &Path, source: Option<&str>) -> Result<Self> {
        let mut state: Self = match crate::config::read_limited(path, 1024 * 1024) {
            Ok(bytes) => nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?,
            Err(error) if error.kind() == ErrorKind::NotFound => Self::default(),
            Err(error) => return Err(error.into()),
        };
        if let Some(pending) = state.pending.take()
            && source.is_some_and(|source| hash(source) == pending.after)
        {
            state.set(pending.id, pending.enabled);
        }
        Ok(state)
    }

    fn set(&mut self, id: String, enabled: bool) {
        if enabled {
            self.entries.insert(id);
        } else {
            self.entries.remove(&id);
        }
    }
}

pub fn edit(dir: &Path, project: Option<&Path>, operation: Edit<'_>) -> Result<String> {
    let _guard = native_settings::lock(dir)?;
    let home = NativeAgent::Claude
        .home()
        .context("Claude Code profile is unavailable")?;
    let user = native_settings::path(&home.join("settings.json"))?;
    let project = project
        .map(|path| project_target(path, &home))
        .transpose()?;
    let target = project.as_ref().map_or(user.as_path(), |(_, path)| path);
    let before = native_settings::read(target)?;
    let user_source = if project.is_some() {
        native_settings::read(&user)?
    } else {
        None
    };
    let shared_source = project
        .as_ref()
        .map(|(root, _)| read(&root.join(".claude/settings.json")))
        .transpose()?
        .flatten();
    let user_flags = enabled(user_source.as_deref().unwrap_or(EMPTY))?;
    let shared_flags = enabled(shared_source.as_deref().unwrap_or(EMPTY))?;
    let installed = read_object(&home.join("plugins/installed_plugins.json"))?;
    let known = read_object(&home.join("plugins/known_marketplaces.json"))?;
    let plugins = installed
        .get("plugins")
        .map(|plugins| {
            plugins
                .as_object()
                .context("expected installed Claude Code plugins")
        })
        .transpose()?;
    let identity = target
        .to_str()
        .context("Claude Code settings path must be UTF-8")?;
    let ledger = dir
        .join("claude-plugins")
        .join(hash(identity))
        .with_extension("json");
    let mut ownership = if project.is_some() {
        Ownership::load(&ledger, before.as_deref())?
    } else {
        Ownership::default()
    };
    let revision = |source: Option<&str>, entries: &BTreeSet<String>| -> Result<String> {
        Ok(hash(&encode(&(
            identity,
            source,
            &user_source,
            &shared_source,
            &installed,
            &known,
            entries,
        ))?))
    };
    let mut placements = before.clone();
    if let Edit::Set {
        id,
        revision: expected,
        enabled: on,
    } = operation
    {
        ensure!(
            expected == revision(before.as_deref(), &ownership.entries)?,
            "Claude Code plugins changed; reload the list"
        );
        let existing = enabled(before.as_deref().unwrap_or(EMPTY))?;
        let value = existing.get(id).copied();
        ensure!(
            valid_id(id),
            "expected a Claude Code plugin ID in name@marketplace format"
        );
        let registered = plugins.and_then(|all| all.get(id));
        if let Some((root, _)) = &project {
            if on {
                ensure!(
                    registered.is_some_and(|entries| installed_here(entries, Some(root))
                        .next()
                        .is_some()),
                    "plugin is unavailable in this Claude Code project"
                );
                ensure!(
                    value.is_none() || value == Some("true"),
                    "Claude Code project already sets this plugin; existing value kept"
                );
            }
            if (on && value.is_none())
                || (!on && value == Some("true") && ownership.entries.contains(id))
            {
                let after = patch(before.as_deref().unwrap_or(EMPTY), id, on.then_some(true))?;
                ownership.pending = Some(Pending {
                    id: id.into(),
                    enabled: on,
                    after: hash(&after),
                });
                std::fs::create_dir_all(
                    ledger
                        .parent()
                        .context("missing Claude Code plugin ledger folder")?,
                )?;
                replace(&ledger, &encode(&ownership)?)?;
                native_settings::write(target, before.as_deref(), Some(&after))?;
                ownership.pending = None;
                ownership.set(id.into(), on);
                replace(&ledger, &encode(&ownership)?)?;
                placements = Some(after);
            } else if !on && ownership.entries.remove(id) {
                replace(&ledger, &encode(&ownership)?)?;
            }
        } else {
            ensure!(
                registered.is_some_and(|entries| installed_here(entries, None).next().is_some()),
                "plugin is unavailable in the Claude Code user profile"
            );
            ensure!(
                matches!(value, None | Some("true" | "false")),
                "Claude Code user settings contain a non-boolean plugin value"
            );
            if value != Some(if on { "true" } else { "false" }) {
                let after = patch(before.as_deref().unwrap_or(EMPTY), id, Some(on))?;
                native_settings::write(target, before.as_deref(), Some(&after))?;
                placements = Some(after);
            }
        }
    }
    let user_settings = if project.is_some() {
        user_source.as_deref()
    } else {
        placements.as_deref()
    };
    let declared = jsonc::parse(user_settings.unwrap_or(EMPTY), &[MARKETPLACES])?;
    let marketplaces = known
        .as_object()
        .context("expected Claude Code marketplaces")?
        .keys()
        .map(str::to_owned)
        .chain(
            declared
                .iter()
                .flat_map(|object| object.keys().map(str::to_owned)),
        )
        .collect::<BTreeSet<_>>();
    let flags = enabled(placements.as_deref().unwrap_or(EMPTY))?;
    let ids = plugins
        .into_iter()
        .flat_map(|plugins| plugins.keys())
        .chain(ownership.entries.iter().map(String::as_str))
        .filter(|id| valid_id(id))
        .collect::<BTreeSet<_>>();
    let root = project.as_ref().map(|(root, _)| root.as_path());
    let plugins = ids
        .into_iter()
        .filter_map(|id| {
            let installed = plugins
                .and_then(|object| object.get(id))
                .into_iter()
                .flat_map(|entry| installed_here(entry, root))
                .collect::<Vec<_>>();
            let managed = ownership.entries.contains(id);
            if installed.is_empty() && !managed {
                return None;
            }
            let flag = flags.get(id).copied();
            let effective = flag
                .or_else(|| shared_flags.get(id).copied())
                .or_else(|| user_flags.get(id).copied());
            Some(Plugin {
                id,
                managed,
                versions: installed
                    .iter()
                    .filter_map(|entry| entry.get("version").and_then(Value::as_str))
                    .collect(),
                scopes: installed
                    .iter()
                    .filter_map(|entry| entry.get("scope").and_then(Value::as_str))
                    .collect(),
                enabled: effective == Some("true"),
                can_enable: !installed.is_empty()
                    && if root.is_some() {
                        flag.is_none()
                    } else {
                        matches!(flag, None | Some("false"))
                    },
                can_disable: if root.is_some() {
                    managed
                } else {
                    flag == Some("true")
                },
            })
        })
        .collect();
    encode(&View {
        plugins,
        marketplaces: marketplaces.iter().map(String::as_str).collect(),
        revision: revision(placements.as_deref(), &ownership.entries)?,
    })
}

fn installed_here<'a>(entries: &'a Value, root: Option<&Path>) -> impl Iterator<Item = &'a Value> {
    entries
        .as_array()
        .into_iter()
        .flatten()
        .filter(
            move |entry| match entry.get("scope").and_then(Value::as_str) {
                Some("user" | "managed") => true,
                Some("local" | "project") => root.is_some_and(|root| {
                    entry
                        .get("projectPath")
                        .and_then(Value::as_str)
                        .and_then(|path| Path::new(path).canonicalize().ok())
                        .as_deref()
                        == Some(root)
                }),
                _ => false,
            },
        )
}

pub fn manage(dir: &Path, action: Manage, target: &str) -> Result<String> {
    ensure!(
        !target.trim().is_empty() && target.len() <= 4096 && !target.chars().any(char::is_control),
        "expected a Claude Code plugin ID, marketplace name or source"
    );
    let args: &[&str] = match action {
        Manage::AddMarketplace => &["marketplace", "add", "--json"],
        Manage::UpdateMarketplace => &["marketplace", "update", "--json"],
        Manage::RemoveMarketplace => &["marketplace", "remove", "--scope", "user", "--json"],
        Manage::Install => &["install", "--scope", "user", "--json"],
        Manage::Update => &["update", "--scope", "user", "--json"],
        Manage::Uninstall => &["uninstall", "--scope", "user", "--keep-data", "--json"],
    };
    if matches!(action, Manage::Install | Manage::Update | Manage::Uninstall) {
        ensure!(
            valid_id(target),
            "expected a Claude Code plugin ID in name@marketplace format"
        );
    }
    run_claude(dir, args, Some(target))?;
    edit(dir, None, Edit::Read)
}

#[derive(NsonSerialize)]
struct Available<'a> {
    id: &'a str,
    description: &'a str,
}

pub fn available(dir: &Path) -> Result<String> {
    let stdout = run_claude(dir, &["list", "--available", "--json"], None)?;
    let mut bytes = Vec::new();
    const LIMIT: u64 = 8 * 1024 * 1024;
    stdout.take(LIMIT + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= LIMIT,
        "Claude Code plugin catalog exceeds 8 MiB"
    );
    let response: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
    let mut rows = response
        .get("available")
        .and_then(Value::as_array)
        .context("Claude Code did not return an available plugin catalog")?
        .iter()
        .filter_map(|row| {
            let id = row.get("pluginId")?.as_str()?;
            valid_id(id).then(|| Available {
                id,
                description: row
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.id);
    rows.dedup_by_key(|row| row.id);
    encode(&rows)
}

fn read_tail(mut file: std::fs::File) -> Result<String> {
    let len = file.metadata()?.len();
    file.seek(SeekFrom::Start(len.saturating_sub(4096)))?;
    let mut text = Vec::new();
    file.take(4096).read_to_end(&mut text)?;
    Ok(String::from_utf8_lossy(&text).trim().to_owned())
}

fn run_claude(dir: &Path, args: &[&str], target: Option<&str>) -> Result<std::fs::File> {
    let _guard = native_settings::lock(dir)?;
    run_plugin_command(None, args, target)
}

pub(crate) fn run_plugin_command(
    project: Option<&Path>,
    args: &[&str],
    target: Option<&str>,
) -> Result<std::fs::File> {
    let path = env::var_os("PATH").context("PATH is unavailable")?;
    let names: &[&str] = if cfg!(windows) {
        &["claude.exe", "claude.cmd"]
    } else {
        &["claude"]
    };
    let executable = env::split_paths(&path)
        .filter(|dir| dir.is_absolute())
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|path| path.is_file())
        .context("Claude Code must be on PATH to manage marketplaces and installations")?;
    let mut stdout = tempfile::tempfile()?;
    let stderr = tempfile::tempfile()?;
    let mut command = Command::new(executable);
    if let Some(project) = project {
        command.current_dir(native_settings::conventional_path(project));
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    command.arg("plugin").args(args);
    if let Some(target) = target {
        command.args(["--", target]);
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(stdout.try_clone()?)
        .stderr(stderr.try_clone()?)
        .spawn()
        .context("start Claude Code plugin management")?;
    let deadline = Instant::now() + Duration::from_secs(120);
    let status = loop {
        match child.try_wait()? {
            Some(status) => break status,
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(40)),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                anyhow::bail!("Claude Code plugin operation timed out");
            }
        }
    };
    if !status.success() {
        let output = read_tail(stdout)?;
        let error = read_tail(stderr)?;
        anyhow::bail!("Claude Code plugin operation failed ({status}): {error}\n{output}");
    }
    stdout.rewind()?;
    Ok(stdout)
}
