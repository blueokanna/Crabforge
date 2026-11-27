use super::{
    Bundle, CLAUDE, MAX_MANIFEST, Tree, check_revision, encode,
    project::{self, Edit, MAX_LEDGER, Row, View, checksum, hash, path_key, verified_tree},
    revision, valid_name,
};
use crate::{
    agents::NativeAgent,
    claude_plugins::{ENABLED, MARKETPLACES, project_target, read_object, run_plugin_command},
    native_settings,
    project_instructions::replace,
    rules::resolved_folder,
};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const ASSIGNMENT_KIND: &str = "crabforge-claude-project";
const SCOPE: &[&str] = &["--scope", "local", "--json"];

#[derive(Default, NsonDeserialize, NsonSerialize)]
struct Store {
    entries: BTreeMap<String, Assignment>,
}

// Save the desired snapshot and all claimed installations before invoking Claude Code.
// A retry can continue installation or withdraw any of the recorded snapshots.
#[derive(Default, NsonDeserialize, NsonSerialize)]
struct Assignment {
    desired: Option<String>,
    placements: BTreeMap<String, Pin>,
}

#[derive(Clone, NsonDeserialize, NsonSerialize)]
struct Pin {
    name: String,
    receipt: Option<String>,
}

struct Project {
    root: PathBuf,
    home: PathBuf,
    target: PathBuf,
    identity: String,
    ledger: PathBuf,
    markets: PathBuf,
    objects: PathBuf,
}

#[derive(NsonSerialize)]
struct Native {
    local: Value,
    shared: Value,
    user: Value,
    known: Value,
    installed: Value,
}

struct Placement<'a> {
    id: String,
    marketplace: String,
    directory: PathBuf,
    local_flag: Option<&'a Value>,
    local_market: Option<&'a Value>,
    known_market: Option<&'a Value>,
    installed: Option<&'a Value>,
    conflict: bool,
}

impl Project {
    fn open(library: &Path, objects: &Path, root: &Path) -> Result<Self> {
        let home = NativeAgent::Claude
            .home()
            .context("Claude Code profile is unavailable")?;
        let (root, target) = project_target(root, &home)?;
        let home = resolved_folder(&std::path::absolute(home)?)?;
        ensure!(
            !target.starts_with(&home),
            "project overlaps the Claude Code user profile"
        );
        let identity = hash(&encode(&(path_key(&target)?, path_key(&home)?))?);
        let library = resolved_folder(&std::path::absolute(library)?)?;
        let markets = library.join("claude-marketplaces");
        let ledger = library
            .join("claude-projects")
            .join(&identity)
            .with_extension("json");
        let objects = resolved_folder(&std::path::absolute(objects)?)?;
        ensure!(
            resolved_folder(&markets)? == markets,
            "Claude Code marketplace directory was redirected"
        );
        ensure!(
            native_settings::path(&ledger)? == ledger,
            "Claude Code assignment directory was redirected"
        );
        Ok(Self {
            root,
            home,
            target,
            identity,
            ledger,
            markets,
            objects,
        })
    }

    fn native(&self) -> Result<Native> {
        ensure!(
            project_target(&self.root, &self.home)?.1 == self.target,
            "Claude Code project settings were redirected"
        );
        Ok(Native {
            local: read_object(&self.target)?,
            shared: read_object(&self.root.join(".claude/settings.json"))?,
            user: read_object(&self.home.join("settings.json"))?,
            known: read_object(&self.home.join("plugins/known_marketplaces.json"))?,
            installed: read_object(&self.home.join("plugins/installed_plugins.json"))?,
        })
    }

    fn revision(&self, store: &Store, native: &Native) -> Result<String> {
        Ok(hash(&encode(&(&self.identity, store, native))?))
    }

    fn save(&self, store: &Store) -> Result<()> {
        let text = encode(store)?;
        ensure!(
            text.len() <= MAX_LEDGER,
            "Claude Code assignments exceed 1 MiB"
        );
        let before = native_settings::read(&self.ledger)?;
        native_settings::write(&self.ledger, before.as_deref(), Some(&text))
    }

    fn marketplace(&self, content: &str) -> String {
        let id = hash(&format!("{}\0{content}", self.identity));
        format!("crabforge-{}", &id[..24])
    }

    fn placement<'a>(&self, native: &'a Native, content: &str, pin: &Pin) -> Result<Placement<'a>> {
        let marketplace = self.marketplace(content);
        let directory = self.markets.join(&marketplace);
        let id = format!("{}@{marketplace}", pin.name);
        let flag = |settings: &'a Value| settings.get(ENABLED).and_then(|entries| entries.get(&id));
        let market = |settings: &'a Value| {
            settings
                .get(MARKETPLACES)
                .and_then(|entries| entries.get(&marketplace))
        };
        for settings in [&native.local, &native.shared, &native.user] {
            for key in [ENABLED, MARKETPLACES] {
                ensure!(
                    settings.get(key).is_none_or(Value::is_object),
                    "Claude Code settings require an object at {key}"
                );
            }
        }
        let local_flag = flag(&native.local);
        let local_market = market(&native.local);
        let known_market = native.known.get(&marketplace);
        let installed = native
            .installed
            .get("plugins")
            .and_then(|plugins| plugins.get(&id));
        let location = path_key(&directory)?;
        let matching_market = |row: &Value| {
            row.get("source").is_some_and(|source| {
                source.get("source").and_then(Value::as_str) == Some("directory")
                    && matching_path(source.get("path"), &location)
            })
        };
        let project = path_key(&self.root)?;
        let matching_install = |row: &Value| {
            let Some(record) = single_install(row) else {
                return false;
            };
            record.get("scope").and_then(Value::as_str) == Some("local")
                && matching_path(record.get("projectPath"), &project)
                && pin
                    .receipt
                    .as_ref()
                    .is_none_or(|receipt| *receipt == installation_receipt(record))
        };
        let conflict = [&native.shared, &native.user]
            .into_iter()
            .any(|settings| flag(settings).is_some() || market(settings).is_some())
            || local_flag.is_some_and(|flag| flag.as_bool() != Some(true))
            || local_market.is_some_and(|row| !matching_market(row))
            || known_market.is_some_and(|row| !matching_market(row))
            || (installed.is_some() && local_flag.is_none() && pin.receipt.is_some())
            || installed.is_some_and(|row| !matching_install(row));
        Ok(Placement {
            id,
            marketplace,
            directory,
            local_flag,
            local_market,
            known_market,
            installed,
            conflict,
        })
    }

    fn stage_market(&self, placement: &Placement<'_>, pin: &Pin, content: &str) -> Result<()> {
        let (_, tree) = verified_tree(&self.objects, content)?;
        let catalog = encode(&json!({
            "name": &placement.marketplace,
            "owner": {"name": "Crabforge"},
            "plugins": [{"name": &pin.name, "source": "./plugin"}]
        }))?;
        let manifest = Path::new(".claude-plugin/marketplace.json");
        ensure!(
            resolved_folder(&self.markets)? == self.markets,
            "Claude Code marketplace directory was redirected"
        );
        fs::create_dir_all(&self.markets)?;
        if placement.directory.try_exists()? {
            ensure!(
                resolved_folder(&placement.directory)? == placement.directory,
                "Claude Code marketplace was redirected"
            );
            let source = native_settings::read(&placement.directory.join(manifest))?;
            ensure!(
                source.as_deref() == Some(&catalog)
                    && Tree::read(&placement.directory.join("plugin"))?.hash() == content,
                "saved Claude Code marketplace was modified"
            );
            return Ok(());
        }
        let staging = tempfile::tempdir_in(&self.markets)?;
        let directory = staging.path().join("market");
        fs::create_dir_all(directory.join(".claude-plugin"))?;
        replace(&directory.join(manifest), &catalog)?;
        tree.write(&directory.join("plugin"))?;
        fs::rename(directory, &placement.directory)?;
        Ok(())
    }

    fn verify_install(&self, place: &Placement<'_>, name: &str, content: &str) -> Result<()> {
        let cache = self
            .home
            .join("plugins/cache")
            .join(&place.marketplace)
            .join(name);
        let record = place
            .installed
            .and_then(single_install)
            .context("missing Claude Code installation record")?;
        let path = record
            .get("installPath")
            .and_then(Value::as_str)
            .context("missing Claude Code plugin installation path")?;
        let folder = Path::new(path).canonicalize()?;
        ensure!(
            folder.parent() == Some(cache.as_path()) && Tree::read(&folder)?.hash() == content,
            "installed Claude Code snapshot was modified"
        );
        Ok(())
    }

    fn call(&self, args: &[&str], target: &str) -> Result<()> {
        ensure!(
            project_target(&self.root, &self.home)?.1 == self.target,
            "Claude Code project settings were redirected"
        );
        let args = [args, SCOPE].concat();
        run_plugin_command(Some(&self.root), &args, Some(target))?;
        Ok(())
    }

    fn install(&self, pin: &Pin, content: &str, native: &mut Native) -> Result<String> {
        let mut place = self.placement(native, content, pin)?;
        ensure!(
            !place.conflict,
            "Claude Code project plugin {} was modified",
            place.id
        );
        if !place.active() {
            self.stage_market(&place, pin, content)?;
            if place.local_market.is_none() || place.known_market.is_none() {
                let directory = native_settings::conventional_path(&place.directory);
                self.call(
                    &["marketplace", "add"],
                    directory.to_str().context("non-UTF-8 marketplace path")?,
                )?;
                *native = self.native()?;
                place = self.placement(native, content, pin)?;
                ensure!(
                    !place.conflict && place.local_market.is_some() && place.known_market.is_some(),
                    "Claude Code did not register the project marketplace"
                );
            }
            if !place.active() {
                self.call(&["install"], &place.id)?;
                *native = self.native()?;
                place = self.placement(native, content, pin)?;
            }
        }
        ensure!(
            place.active(),
            "Claude Code did not activate the project plugin"
        );
        let record = place
            .installed
            .and_then(single_install)
            .context("missing Claude Code installation record")?;
        self.verify_install(&place, &pin.name, content)?;
        Ok(installation_receipt(record))
    }

    fn withdraw(&self, pin: &Pin, content: &str, native: &mut Native) -> Result<()> {
        let mut place = self.placement(native, content, pin)?;
        if place.conflict {
            return Ok(());
        }
        if place.installed.is_some() || place.local_flag.is_some() {
            self.call(&["uninstall", "--keep-data"], &place.id)?;
            *native = self.native()?;
            place = self.placement(native, content, pin)?;
            ensure!(
                place.installed.is_none() && place.local_flag.is_none(),
                "Claude Code kept the project installation"
            );
        }
        if !place.conflict && place.local_market.is_some() {
            self.call(&["marketplace", "remove"], &place.marketplace)?;
            *native = self.native()?;
            ensure!(
                self.placement(native, content, pin)?.local_market.is_none(),
                "Claude Code kept the project marketplace"
            );
        }
        Ok(())
    }
}

fn matching_path(value: Option<&Value>, expected: &str) -> bool {
    value
        .and_then(Value::as_str)
        .and_then(|path| path_key(Path::new(path)).ok())
        .as_deref()
        == Some(expected)
}

fn single_install(row: &Value) -> Option<&Value> {
    match row.as_array()?.as_slice() {
        [record] => Some(record),
        _ => None,
    }
}

fn installation_receipt(record: &Value) -> String {
    // Claude Code can reorder the record's keys when loading its plugin cache.
    let identity = ["installPath", "version", "installedAt", "lastUpdated"].map(|field| {
        record
            .get(field)
            .and_then(Value::as_str)
            .unwrap_or_default()
    });
    hash(&identity.join("\0"))
}

impl Placement<'_> {
    fn active(&self) -> bool {
        !self.conflict
            && self.local_flag.is_some()
            && self.local_market.is_some()
            && self.known_market.is_some()
            && self.installed.is_some()
    }

    fn absent(&self) -> bool {
        !self.conflict
            && self.local_flag.is_none()
            && self.local_market.is_none()
            && self.known_market.is_none()
            && self.installed.is_none()
    }
}

fn load(ledger: &Path) -> Result<Store> {
    let store: Store = match native_settings::read(ledger)? {
        Some(text) => nextjson::from_str(&text).map_err(anyhow::Error::msg)?,
        None => return Ok(Store::default()),
    };
    for (name, entry) in &store.entries {
        ensure!(
            valid_name(name)
                && entry
                    .desired
                    .as_ref()
                    .is_none_or(|content| entry.placements.contains_key(content)),
            "invalid Claude Code assignment record"
        );
        for (content, pin) in &entry.placements {
            ensure!(
                checksum(content) && valid_name(&pin.name) && pin.name.eq_ignore_ascii_case(name),
                "invalid Claude Code snapshot record"
            );
        }
    }
    Ok(store)
}

fn is_claude(bundle: &Bundle) -> bool {
    bundle.manifests.iter().any(|manifest| manifest == CLAUDE)
}

fn prepare(project: &Project, bundle: &Bundle) -> Result<Pin> {
    ensure!(is_claude(bundle), "expected a Claude Code plugin bundle");
    let (_, tree) = verified_tree(&project.objects, &bundle.content)?;
    let manifest = tree
        .text(Path::new(CLAUDE))?
        .context("missing Claude Code manifest")?;
    ensure!(
        manifest.len() <= MAX_MANIFEST && valid_name(&bundle.name),
        "invalid Claude Code manifest"
    );
    Ok(Pin {
        name: bundle.name.clone(),
        receipt: None,
    })
}

fn active_snapshot<'a>(
    project: &Project,
    native: &Native,
    entry: &'a Assignment,
) -> Result<&'a str> {
    if let Some(content) = &entry.desired
        && let Some(pin) = entry.placements.get(content)
        && entry.placements.len() == 1
        && project.placement(native, content, pin)?.active()
    {
        return Ok(content);
    }
    anyhow::bail!("Claude Code assignment changed; resolve it before exporting")
}

fn exportable_snapshot<'a>(
    project: &Project,
    native: &Native,
    entry: &'a Assignment,
) -> Result<(&'a str, Tree)> {
    let content = active_snapshot(project, native, entry)?;
    let pin = &entry.placements[content];
    project.verify_install(
        &project.placement(native, content, pin)?,
        &pin.name,
        content,
    )?;
    Ok((content, verified_tree(&project.objects, content)?.1))
}

pub(super) fn edit(
    dir: &Path,
    library: &Path,
    objects: &Path,
    bundles: &BTreeMap<String, Bundle>,
    project: &Path,
    operation: Edit<'_>,
) -> Result<String> {
    let _guard = native_settings::lock(dir)?;
    let project = Project::open(library, objects, project)?;
    let mut store = load(&project.ledger)?;
    let mut native = project.native()?;
    if let Some(expected) = operation.expected() {
        ensure!(
            project.revision(&store, &native)? == expected,
            "Claude Code project plugins changed; reload them"
        );
    }
    let mut changes = BTreeMap::new();
    match operation {
        Edit::Read => {}
        Edit::Set {
            name,
            bundle_revision,
            ..
        } => {
            changes.insert(
                name.to_ascii_lowercase(),
                bundle_revision
                    .map(|expected| check_revision(bundles, name, expected))
                    .transpose()?,
            );
        }
        Edit::Apply { assignments, .. } => {
            for (name, content) in project::import(ASSIGNMENT_KIND, assignments)? {
                let bundle = bundles
                    .get(&name)
                    .filter(|bundle| bundle.content == content)
                    .with_context(|| {
                        format!("import the matching Claude Code snapshot for {name}")
                    })?;
                ensure!(
                    store
                        .entries
                        .get(&name)
                        .and_then(|entry| entry.desired.as_ref())
                        .is_none_or(|desired| desired == &content),
                    "Claude Code project already has a different assignment for {name}"
                );
                changes.insert(name, Some(bundle));
            }
        }
        Edit::Export { .. } => {
            let packages = store
                .entries
                .iter()
                .map(|(name, entry)| {
                    let (content, _) = exportable_snapshot(&project, &native, entry)
                        .with_context(|| name.clone())?;
                    Ok((name.clone(), content.to_owned()))
                })
                .collect::<Result<_>>()?;
            return project::export(ASSIGNMENT_KIND, packages);
        }
        Edit::Archive { name, output, .. } => {
            let entry = store
                .entries
                .get(&name.to_ascii_lowercase())
                .context("Claude Code assignment was removed")?;
            let (content, tree) = exportable_snapshot(&project, &native, entry)?;
            return super::export_tree(&tree, content, output);
        }
    }
    for (name, bundle) in &changes {
        ensure!(valid_name(name), "invalid Claude Code plugin name");
        if let Some(bundle) = bundle {
            let prepared = prepare(&project, bundle)?;
            let entry = store.entries.entry(name.clone()).or_default();
            let previous = entry.placements.get(&bundle.content);
            for (content, pin) in &entry.placements {
                if content != &bundle.content {
                    let prior = project.placement(&native, content, pin)?;
                    ensure!(
                        !prior.conflict,
                        "Claude Code project plugin {} was modified",
                        prior.id
                    );
                }
            }
            let place =
                project.placement(&native, &bundle.content, previous.unwrap_or(&prepared))?;
            ensure!(
                !place.conflict && (previous.is_some() || place.absent()),
                "Claude Code project already configures {}",
                place.id
            );
            entry
                .placements
                .entry(bundle.content.clone())
                .or_insert(prepared);
            entry.desired = Some(bundle.content.clone());
        } else if let Some(entry) = store.entries.get_mut(name) {
            entry.desired = None;
        }
    }
    if !changes.is_empty() {
        project.save(&store)?;
    }
    for name in changes.keys() {
        let Some(entry) = store.entries.get_mut(name) else {
            continue;
        };
        if let Some(content) = &entry.desired {
            let pin = entry
                .placements
                .get_mut(content)
                .expect("validated assignment");
            pin.receipt = Some(project.install(pin, content, &mut native)?);
            project.save(&store)?;
        }
        let entry = &store.entries[name];
        for (content, pin) in &entry.placements {
            if entry.desired.as_ref() != Some(content) {
                project.withdraw(pin, content, &mut native)?;
            }
        }
        let entry = store.entries.get_mut(name).expect("assignment exists");
        entry
            .placements
            .retain(|content, _| entry.desired.as_ref() == Some(content));
        if entry.desired.is_none() {
            store.entries.remove(name);
        }
        project.save(&store)?;
    }

    let mut rows = BTreeMap::new();
    for (key, bundle) in bundles.iter().filter(|(_, bundle)| is_claude(bundle)) {
        let entry = store.entries.get(key);
        let active = entry.and_then(|entry| active_snapshot(&project, &native, entry).ok());
        let pin = Pin {
            name: bundle.name.clone(),
            receipt: None,
        };
        let candidate = entry.and_then(|entry| entry.placements.get(&bundle.content));
        let place = project.placement(&native, &bundle.content, candidate.unwrap_or(&pin))?;
        let status = if place.conflict || (candidate.is_none() && !place.absent()) {
            "conflict"
        } else if active == Some(&bundle.content) {
            "applied"
        } else if entry.is_some() {
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
                assigned: entry.is_some(),
                can_export_snapshot: active.is_some(),
                status,
            },
        );
    }
    for (key, entry) in &store.entries {
        let active = active_snapshot(&project, &native, entry).is_ok();
        rows.entry(key).or_insert(Row {
            name: key,
            version: None,
            bundle_revision: None,
            assigned: true,
            can_export_snapshot: active,
            status: if active { "detached" } else { "conflict" },
        });
    }
    encode(&View {
        revision: project.revision(&store, &native)?,
        packages: rows.into_values().collect(),
    })
}
