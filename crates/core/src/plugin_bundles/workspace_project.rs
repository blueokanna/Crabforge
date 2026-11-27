use super::{
    ANTIGRAVITY, Bundle, CODEX, Tree, check_revision, encode,
    project::{self, Edit, Row, Store, View, checksum, hash, path_key, stage_tree, verified_tree},
    revision, valid_name, windows_filename,
};
use crate::{agents::NativeAgent, native_settings, project_mcp::jsonc, rules::resolved_folder};
use anyhow::{Context, Result, ensure};
use nextjson::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, Item, Table, value};

#[derive(Clone, Copy)]
pub enum Agent {
    Codex,
    Antigravity,
}

impl Agent {
    fn key(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Antigravity => "antigravity",
        }
    }

    fn manifest(self) -> &'static str {
        match self {
            Self::Codex => CODEX,
            Self::Antigravity => ANTIGRAVITY,
        }
    }

    fn includes(self, bundle: &Bundle) -> bool {
        bundle.manifests.iter().any(|path| path == self.manifest())
    }
}

struct Workspace {
    agent: Agent,
    root: PathBuf,
    profile: PathBuf,
    objects: PathBuf,
    library: PathBuf,
    identity: String,
    target: PathBuf,
    ledger: PathBuf,
}

struct Pin<'a> {
    content: &'a str,
    key: &'a str,
    id: String,
    market: String,
    catalog: PathBuf,
    source: String,
}

impl Workspace {
    fn open(agent: Agent, library: &Path, objects: &Path, root: &Path) -> Result<Self> {
        let root = root
            .canonicalize()
            .context("project directory is unavailable")?;
        ensure!(root.is_dir(), "expected a project directory");
        let (profile, relative) = match agent {
            Agent::Codex => (NativeAgent::Codex.home(), ".codex/config.toml"),
            Agent::Antigravity => (
                env::home_dir().map(|home| home.join(".gemini/config")),
                ".agents/plugins.json",
            ),
        };
        let profile = resolved_folder(&std::path::absolute(
            profile.context("user profile is unavailable")?,
        )?)?;
        let target = root.join(relative);
        ensure!(
            native_settings::path(&target)? == target,
            "project plugin configuration was redirected"
        );
        ensure!(
            !target.starts_with(&profile),
            "project plugin configuration overlaps the user profile"
        );
        if matches!(agent, Agent::Codex) {
            ensure!(
                native_settings::path(&profile.join("config.toml"))? != target,
                "project plugin configuration overlaps the user profile"
            );
        }
        let identity = hash(&encode(&(path_key(&target)?, path_key(&profile)?))?);
        let library = resolved_folder(&std::path::absolute(library)?)?;
        let objects = resolved_folder(&std::path::absolute(objects)?)?;
        let ledger = library
            .join(format!("{}-projects", agent.key()))
            .join(&identity)
            .with_extension("json");
        Ok(Self {
            agent,
            root,
            profile,
            objects,
            library,
            identity,
            target,
            ledger,
        })
    }

    fn pin<'a>(&self, key: &'a str, content: &'a str) -> Result<Pin<'a>> {
        ensure!(
            valid_name(key) && checksum(content),
            "invalid project plugin assignment"
        );
        let hash = hash(&format!("{}\0{content}", self.identity));
        let market = format!("crabforge-{}", &hash[..24]);
        Ok(Pin {
            content,
            key,
            id: format!("{key}@{market}"),
            catalog: self.library.join("codex-marketplaces").join(&market),
            market,
            source: format!(".agents/crabforge-plugin-bundles/{content}/{key}"),
        })
    }

    fn stage(&self, pin: &Pin<'_>) -> Result<()> {
        let (_, tree) = verified_tree(&self.objects, pin.content)?;
        let manifest = tree
            .text(Path::new(self.agent.manifest()))?
            .context("snapshot lacks the native plugin manifest")?;
        let manifest: Value = nextjson::from_str(manifest.trim_start_matches('\u{feff}'))
            .map_err(anyhow::Error::msg)?;
        let name = manifest
            .get("name")
            .and_then(Value::as_str)
            .context("missing native plugin name")?;
        ensure!(
            name.eq_ignore_ascii_case(pin.key),
            "native plugin name differs from the bundle name"
        );
        match self.agent {
            Agent::Antigravity => stage_tree(&self.root.join(&pin.source), &tree, pin.content),
            Agent::Codex => {
                ensure!(name == pin.key, "Codex plugin names must be lowercase");
                let version = manifest
                    .get("version")
                    .and_then(Value::as_str)
                    .unwrap_or("0.0.0");
                ensure!(
                    windows_filename(version) && !matches!(version, "." | ".."),
                    "invalid Codex plugin version"
                );
                self.stage_codex(pin, version, &tree)
            }
        }
    }

    fn stage_codex(&self, pin: &Pin<'_>, version: &str, tree: &Tree) -> Result<()> {
        let catalog = pin.catalog.join(".agents/plugins/marketplace.json");
        stage_tree(&pin.catalog.join("plugin"), tree, pin.content)?;
        let text = encode(&json!({
            "name": &pin.market,
            "plugins": [{"name": pin.key, "source": {"source": "local", "path": "./plugin"},
                "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
                "category": "Productivity"}]
        }))?;
        match native_settings::read(&catalog)? {
            Some(existing) => ensure!(existing == text, "Codex marketplace was modified"),
            None => native_settings::write(&catalog, None, Some(&text))?,
        }
        let cache = self
            .profile
            .join("plugins/cache")
            .join(&pin.market)
            .join(pin.key);
        ensure!(
            resolved_folder(&cache)? == cache,
            "Codex plugin cache was redirected"
        );
        if cache.try_exists()? {
            ensure!(
                fs::read_dir(&cache)?
                    .collect::<std::io::Result<Vec<_>>>()?
                    .iter()
                    .all(|entry| entry.file_name() == version),
                "Codex plugin cache contains another version"
            );
        }
        stage_tree(&cache.join(version), tree, pin.content)
    }

    fn check_external(&self, pin: &Pin<'_>, profile: Option<&str>) -> Result<()> {
        match self.agent {
            Agent::Codex => ensure!(
                !toml_entry(&toml(profile.unwrap_or(""))?, "plugins", &pin.id)?.is_some_and(
                    |entry| {
                        entry
                            .as_table_like()
                            .and_then(|table| table.get("enabled"))
                            .and_then(Item::as_bool)
                            != Some(false)
                    }
                ),
                "Codex user profile already enables this plugin"
            ),
            Agent::Antigravity => {
                for directory in [".agents", ".agent", "_agents", "_agent"] {
                    let existing = self.root.join(directory).join("plugins").join(pin.key);
                    match fs::symlink_metadata(existing) {
                        Ok(_) => {
                            anyhow::bail!("Antigravity project already contains plugin {}", pin.key)
                        }
                        Err(error) if error.kind() == ErrorKind::NotFound => {}
                        Err(error) => return Err(error.into()),
                    }
                }
            }
        }
        Ok(())
    }
}

fn toml(text: &str) -> Result<DocumentMut> {
    text.parse().context("invalid Codex plugin settings TOML")
}

fn table<'a>(document: &'a mut DocumentMut, key: &str) -> Result<&'a mut dyn toml_edit::TableLike> {
    let entry = document.entry(key).or_insert_with(|| {
        let mut table = Table::new();
        table.set_implicit(true);
        Item::Table(table)
    });
    entry
        .as_table_like_mut()
        .with_context(|| format!("expected a Codex {key} table"))
}

fn toml_entry<'a>(document: &'a DocumentMut, table: &str, id: &str) -> Result<Option<&'a Item>> {
    document
        .get(table)
        .map(|item| {
            Ok(item
                .as_table_like()
                .with_context(|| format!("expected a Codex {table} table"))?
                .get(id))
        })
        .transpose()
        .map(Option::flatten)
}

fn discovery(text: &str) -> Result<(jsonc::Object, Option<jsonc::Array>)> {
    let _: Value =
        nextjson::from_str(text.trim_start_matches('\u{feff}')).map_err(anyhow::Error::msg)?;
    let root =
        jsonc::parse(text, &[])?.context("expected an Antigravity plugin discovery object")?;
    let array = root.array(text, "entries")?;
    Ok((root, array))
}

fn discovery_entry(
    text: &str,
    array: Option<&jsonc::Array>,
    pin: &Pin<'_>,
) -> Result<(Option<usize>, bool)> {
    let mut found = None;
    let mut conflict = false;
    for (index, raw) in array
        .into_iter()
        .flat_map(|array| array.values(text))
        .enumerate()
    {
        let entry: Value = nextjson::from_str(raw).map_err(anyhow::Error::msg)?;
        let Some(path) = entry.get("path").and_then(Value::as_str) else {
            continue;
        };
        let path_key = path.replace('\\', "/");
        let normalized = path_key.trim_end_matches('/').trim_start_matches("./");
        if normalized.eq_ignore_ascii_case(&pin.source) {
            if path == pin.source
                && entry.as_object().is_some_and(|object| object.len() == 1)
                && found.is_none()
            {
                found = Some(index);
            } else {
                conflict = true;
            }
        }
    }
    Ok((found, conflict))
}

fn placement(agent: Agent, settings: Option<&str>, pin: &Pin<'_>) -> Result<(bool, bool)> {
    match agent {
        Agent::Codex => {
            let document = toml(settings.unwrap_or(""))?;
            let entry = toml_entry(&document, "plugins", &pin.id)?;
            let market = toml_entry(&document, "marketplaces", &pin.market)?;
            let path = native_settings::conventional_path(&pin.catalog);
            let flag_owned = entry.and_then(Item::as_table_like).is_some_and(|entry| {
                entry.len() == 1 && entry.get("enabled").and_then(Item::as_bool) == Some(true)
            });
            let market_owned = market.and_then(Item::as_table_like).is_some_and(|entry| {
                entry.len() == 2
                    && entry.get("source_type").and_then(Item::as_str) == Some("local")
                    && entry.get("source").and_then(Item::as_str) == path.to_str()
            });
            let owned = flag_owned && market_owned;
            Ok((owned, !owned && (entry.is_some() || market.is_some())))
        }
        Agent::Antigravity => {
            let text = settings.unwrap_or("{}\n");
            let (_, array) = discovery(text)?;
            let (owned, conflict) = discovery_entry(text, array.as_ref(), pin)?;
            Ok((owned.is_some() && !conflict, conflict))
        }
    }
}

fn change(agent: Agent, settings: Option<&str>, pin: &Pin<'_>, active: bool) -> Result<String> {
    match agent {
        Agent::Codex => {
            let mut document = toml(settings.unwrap_or(""))?;
            let entries = table(&mut document, "plugins")?;
            if active {
                let mut entry = Table::new();
                entry.insert("enabled", value(true));
                entries.insert(&pin.id, Item::Table(entry));
            } else {
                entries.remove(&pin.id);
            }
            let markets = table(&mut document, "marketplaces")?;
            if active {
                let path = native_settings::conventional_path(&pin.catalog);
                let mut market = Table::new();
                market.insert("source_type", value("local"));
                market.insert(
                    "source",
                    value(
                        path.to_str()
                            .context("Codex marketplace path must be UTF-8")?,
                    ),
                );
                markets.insert(&pin.market, Item::Table(market));
            } else {
                markets.remove(&pin.market);
            }
            Ok(document.to_string())
        }
        Agent::Antigravity => {
            let text = settings.unwrap_or("{}\n");
            let (root, array) = discovery(text)?;
            if active {
                let entry = json!({"path": &pin.source});
                match array {
                    Some(array) => array.append(text, &entry),
                    None => root.edit(text, "entries", Some(&Value::Array(vec![entry]))),
                }
            } else {
                let array = array.context("missing Antigravity discovery entries")?;
                let (index, _) = discovery_entry(text, Some(&array), pin)?;
                array.remove(text, index.context("missing Antigravity plugin entry")?)
            }
        }
    }
}

pub(super) fn edit(
    agent: Agent,
    dir: &Path,
    library: &Path,
    objects: &Path,
    bundles: &BTreeMap<String, Bundle>,
    root: &Path,
    operation: Edit<'_>,
) -> Result<String> {
    let _guard = native_settings::lock(dir)?;
    let project = Workspace::open(agent, library, objects, root)?;
    let before = native_settings::read(&project.target)?;
    let profile = if matches!(agent, Agent::Codex) {
        native_settings::read(&project.profile.join("config.toml"))?
    } else {
        None
    };
    let mut store = Store::load(&project.ledger, before.as_deref())?;
    let kind = format!("crabforge-{}-project", agent.key());
    if let Some(expected) = operation.expected() {
        ensure!(
            store.revision(&project.identity, before.as_deref())? == expected,
            "project plugins changed; reload them"
        );
    }
    let snapshot = |key: &str, content: &str| -> Result<Tree> {
        let pin = project.pin(key, content)?;
        ensure!(
            placement(agent, before.as_deref(), &pin)?.0,
            "project plugin {key} was modified"
        );
        Ok(verified_tree(&project.objects, content)?.1)
    };
    let requested = match operation {
        Edit::Read => BTreeMap::new(),
        Edit::Export { .. } => {
            for (key, content) in &store.entries {
                snapshot(key, content)?;
            }
            return project::export(&kind, store.entries);
        }
        Edit::Archive { name, output, .. } => {
            let key = name.to_ascii_lowercase();
            let content = store
                .entries
                .get(&key)
                .context("project plugin assignment was removed")?;
            return super::export_tree(&snapshot(&key, content)?, content, output);
        }
        Edit::Set {
            name,
            bundle_revision,
            ..
        } => {
            let content = bundle_revision
                .map(|expected| -> Result<_> {
                    let bundle = check_revision(bundles, name, expected)?;
                    ensure!(
                        agent.includes(bundle),
                        "snapshot lacks the native plugin manifest"
                    );
                    Ok(bundle.content.clone())
                })
                .transpose()?;
            BTreeMap::from([(name.to_ascii_lowercase(), content)])
        }
        Edit::Apply { assignments, .. } => project::import(&kind, assignments)?
            .into_iter()
            .map(|(key, content)| {
                ensure!(
                    bundles
                        .get(&key)
                        .is_some_and(|bundle| agent.includes(bundle) && bundle.content == content),
                    "import the matching plugin snapshot for {key}"
                );
                ensure!(
                    store
                        .entries
                        .get(&key)
                        .is_none_or(|existing| *existing == content),
                    "project already has a different plugin assignment for {key}"
                );
                Ok((key, Some(content)))
            })
            .collect::<Result<_>>()?,
    };
    let mut after = before.clone();
    for (key, desired) in &requested {
        let prior = store.entries.get(key);
        if let Some(prior) = prior {
            let pin = project.pin(key, prior)?;
            let (owned, conflict) = placement(agent, after.as_deref(), &pin)?;
            ensure!(
                desired.is_none() || !conflict,
                "project plugin {key} was modified"
            );
            if desired.as_ref() != Some(prior) && owned {
                after = Some(change(agent, after.as_deref(), &pin, false)?);
            }
        }
        if let Some(content) = desired {
            let pin = project.pin(key, content)?;
            project.check_external(&pin, profile.as_deref())?;
            let (present, conflict) = placement(agent, after.as_deref(), &pin)?;
            ensure!(
                !conflict && (!present || prior == Some(content)),
                "project already configures plugin {key}"
            );
            if !present {
                after = Some(change(agent, after.as_deref(), &pin, true)?);
            }
        }
    }
    for (key, desired) in &requested {
        if let Some(content) = desired {
            project.stage(&project.pin(key, content)?)?;
        }
    }
    store.commit(
        &project.ledger,
        &project.target,
        before.as_deref(),
        after.as_deref(),
        requested,
    )?;

    let mut rows = BTreeMap::new();
    for (key, bundle) in bundles
        .iter()
        .filter(|(_, bundle)| project.agent.includes(bundle))
    {
        let prior = store.entries.get(key);
        let (owned, modified) = prior
            .map(|content| placement(agent, after.as_deref(), &project.pin(key, content)?))
            .transpose()?
            .unwrap_or_default();
        let pin = project.pin(key, &bundle.content)?;
        let (present, conflict) = placement(agent, after.as_deref(), &pin)?;
        let status = if modified
            || conflict
            || (present && prior != Some(&bundle.content))
            || project.check_external(&pin, profile.as_deref()).is_err()
        {
            "conflict"
        } else if owned && prior == Some(&bundle.content) {
            "applied"
        } else if prior.is_some() {
            "pending"
        } else {
            "available"
        };
        rows.insert(
            key,
            Row {
                name: &bundle.name,
                version: Some(&bundle.version),
                bundle_revision: Some(revision(bundle)?),
                assigned: prior.is_some(),
                can_export_snapshot: owned,
                status,
            },
        );
    }
    for (key, content) in &store.entries {
        if !rows.contains_key(key) {
            let owned = placement(agent, after.as_deref(), &project.pin(key, content)?)?.0;
            rows.insert(
                key,
                Row {
                    name: key,
                    version: None,
                    bundle_revision: None,
                    assigned: true,
                    can_export_snapshot: owned,
                    status: if owned { "detached" } else { "conflict" },
                },
            );
        }
    }
    encode(&View {
        revision: store.revision(&project.identity, after.as_deref())?,
        packages: rows.into_values().collect(),
    })
}
