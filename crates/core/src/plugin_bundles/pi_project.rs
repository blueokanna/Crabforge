use super::{Bundle, PI, Tree, check_revision, encode, revision};
use crate::{
    config::read_limited, native_settings, project_instructions::replace, project_mcp::jsonc,
};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

const PACKAGES: &str = "packages";
const EMPTY: &str = "{}\n";
const MAX_LEDGER: usize = 1024 * 1024;

pub enum Edit<'a> {
    Read,
    Set {
        name: &'a str,
        project_revision: &'a str,
        bundle_revision: Option<&'a str>,
    },
    Export {
        project_revision: &'a str,
    },
    Archive {
        name: &'a str,
        project_revision: &'a str,
        output: &'a Path,
    },
    Apply {
        project_revision: &'a str,
        assignments: &'a str,
    },
}

#[derive(Default, NsonDeserialize, NsonSerialize)]
struct Store {
    entries: BTreeMap<String, String>,
    pending: Option<Pending>,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Pending {
    entries: BTreeMap<String, Option<String>>,
    after: String,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Assignments {
    kind: String,
    version: u32,
    packages: BTreeMap<String, String>,
}

const ASSIGNMENT_KIND: &str = "crabforge-pi-project";

#[derive(NsonSerialize)]
struct View<'a> {
    revision: String,
    packages: Vec<Row<'a>>,
}

#[derive(NsonSerialize)]
struct Row<'a> {
    name: &'a str,
    version: Option<&'a str>,
    bundle_revision: Option<String>,
    assigned: bool,
    can_export_snapshot: bool,
    status: &'static str,
}

fn hash(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn path_key(path: &Path) -> Result<String> {
    let path = native_settings::conventional_path(path);
    let text = path.to_str().context("Pi package paths must be UTF-8")?;
    Ok(if cfg!(windows) {
        text.replace('/', "\\").to_lowercase()
    } else {
        text.to_owned()
    })
}

fn home_relative(path: &Path) -> Option<PathBuf> {
    let relative = path.strip_prefix("~").ok()?;
    env::home_dir().map(|home| home.join(relative))
}

fn source_key(project: &Path, source: &str) -> Option<String> {
    let source = source.trim();
    if source.starts_with("npm:") || source.starts_with("git:") || source.contains("://") {
        return None;
    }
    let path = home_relative(Path::new(source))
        .unwrap_or_else(|| native_settings::conventional_path(project).join(source));
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            component => normalized.push(component),
        }
    }
    path_key(&normalized).ok()
}

fn source(project: &Path, folder: &Path) -> Result<String> {
    let absolute = if folder.is_absolute() {
        folder.to_path_buf()
    } else {
        std::path::absolute(folder)?
    };
    let candidate = native_settings::conventional_path(&absolute);
    let source = candidate
        .to_str()
        .context("Pi package paths must be UTF-8")?;
    ensure!(
        source.trim() == source && source_key(project, source).is_some(),
        "invalid native Pi package path"
    );
    Ok(source.to_owned())
}

impl Store {
    fn load(path: &Path, settings: Option<&str>) -> Result<Self> {
        let mut state: Self = match read_limited(path, MAX_LEDGER) {
            Ok(bytes) => nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?,
            Err(error) if error.kind() == ErrorKind::NotFound => Self::default(),
            Err(error) => return Err(error.into()),
        };
        if let Some(pending) = state.pending.take()
            && settings.is_some_and(|text| hash(text) == pending.after)
        {
            state.update(pending.entries);
        }
        Ok(state)
    }

    fn update(&mut self, updates: BTreeMap<String, Option<String>>) {
        for (name, source) in updates {
            if let Some(source) = source {
                self.entries.insert(name, source);
            } else {
                self.entries.remove(&name);
            }
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        let text = encode(self)?;
        ensure!(
            text.len() <= MAX_LEDGER,
            "Pi project assignments exceed 1 MiB"
        );
        fs::create_dir_all(path.parent().context("missing Pi project ledger folder")?)?;
        replace(path, &text)
    }

    fn revision(&self, identity: &str, settings: Option<&str>) -> Result<String> {
        Ok(hash(&encode(&(identity, settings, &self.entries))?))
    }

    fn export(&self, project: &Path, objects: &Path, packages: &Packages) -> Result<String> {
        let mut selections = BTreeMap::new();
        for (name, assigned) in &self.entries {
            let (content, _) = assigned_snapshot(name, assigned, project, objects, packages)?;
            selections.insert(name.clone(), content.to_owned());
        }
        encode(&Assignments {
            kind: ASSIGNMENT_KIND.to_owned(),
            version: 1,
            packages: selections,
        })
    }
}

fn assigned_snapshot<'a>(
    name: &str,
    assigned: &'a str,
    project: &Path,
    objects: &Path,
    packages: &Packages,
) -> Result<(&'a str, Tree)> {
    ensure!(
        packages.owned(project, assigned).is_some(),
        "Pi package assignment for {name} has changed; resolve it before exporting"
    );
    let content = Path::new(assigned)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| checksum(value))
        .context("invalid Pi package assignment hash")?;
    let (folder, tree) = verified_tree(objects, content)?;
    ensure!(
        source(project, &folder)? == assigned,
        "Pi package assignment for {name} points outside the library"
    );
    Ok((content, tree))
}

fn checksum(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_pi(bundle: &Bundle) -> bool {
    bundle.manifests.iter().any(|manifest| manifest == PI)
}

fn verified_source(project: &Path, objects: &Path, content: &str) -> Result<String> {
    let (folder, _) = verified_tree(objects, content)?;
    source(project, &folder)
}

fn verified_tree(objects: &Path, content: &str) -> Result<(PathBuf, Tree)> {
    let expected = objects.join(content);
    let folder = expected.canonicalize()?;
    ensure!(folder == expected, "saved Pi package was redirected");
    let tree = Tree::read(&folder)?;
    ensure!(tree.hash() == content, "saved plugin snapshot was modified");
    Ok((folder, tree))
}

struct Packages {
    root: jsonc::Object,
    array: Option<jsonc::Array>,
    entries: Vec<PackageEntry>,
}

struct PackageEntry {
    source: String,
    path_key: Option<String>,
    simple: bool,
}

impl Packages {
    fn read(project: &Path, source: &str) -> Result<Self> {
        let _: Value = nextjson::from_str(source.strip_prefix('\u{feff}').unwrap_or(source))
            .map_err(|error| anyhow::anyhow!("invalid Pi project JSON: {error}"))?;
        let root = jsonc::parse(source, &[])?.context("expected a Pi settings object")?;
        let array = if root.value(source, PACKAGES) == Some("null") {
            None
        } else {
            root.array(source, PACKAGES)?
        };
        let entries = array
            .iter()
            .flat_map(|array| array.values(source))
            .map(|text| {
                let value: Value = nextjson::from_str(text).map_err(anyhow::Error::msg)?;
                let simple = value.is_string();
                let source = value
                    .as_str()
                    .or_else(|| value.get("source").and_then(Value::as_str))
                    .filter(|source| !source.trim().is_empty())
                    .context("expected a Pi package source string")?;
                Ok(PackageEntry {
                    source: source.to_owned(),
                    path_key: source_key(project, source),
                    simple,
                })
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            root,
            array,
            entries,
        })
    }

    fn matches<'a>(&'a self, project: &Path, source: &str) -> impl Iterator<Item = usize> + 'a {
        let key = source_key(project, source);
        self.entries
            .iter()
            .enumerate()
            .filter_map(move |(index, row)| (key.is_some() && row.path_key == key).then_some(index))
    }

    fn owned(&self, project: &Path, source: &str) -> Option<usize> {
        let mut matches = self.matches(project, source);
        let index = matches.next()?;
        let entry = &self.entries[index];
        (entry.simple && entry.source == source && matches.next().is_none()).then_some(index)
    }

    fn index_for_change(
        &self,
        project: &Path,
        previous: Option<&str>,
        desired: Option<&str>,
    ) -> Result<Option<usize>> {
        let owned = previous.and_then(|source| self.owned(project, source));
        if let Some(desired) = desired {
            ensure!(
                previous.is_none_or(|source| {
                    owned.is_some() || self.matches(project, source).next().is_none()
                }) && self
                    .matches(project, desired)
                    .all(|index| Some(index) == owned),
                "Pi project already configures this package; existing entry kept"
            );
        }
        Ok(owned)
    }

    fn change(self, settings: &str, index: Option<usize>, desired: Option<&str>) -> Result<String> {
        match (self.array, index, desired) {
            (Some(array), Some(index), Some(source)) => {
                array.replace(settings, index, &encode(&source)?)
            }
            (Some(array), Some(index), None) => array.remove(settings, index),
            (Some(array), None, Some(source)) => array.append(settings, &Value::from(source)),
            (None, None, Some(source)) => self.root.edit(
                settings,
                PACKAGES,
                Some(&Value::Array(vec![Value::from(source)])),
            ),
            (_, None, None) => Ok(settings.to_owned()),
            (None, Some(_), _) => unreachable!(),
        }
    }
}

fn target(dir: &Path, project: &Path) -> Result<(PathBuf, PathBuf)> {
    let root = project
        .canonicalize()
        .context("project directory is unavailable")?;
    ensure!(root.is_dir(), "expected a project directory");
    let target = root.join(".pi/settings.json");
    ensure!(
        native_settings::path(&target)? == target,
        "Pi project settings folder was redirected"
    );
    let user = env::var_os("PI_CODING_AGENT_DIR")
        .filter(|value| !value.is_empty())
        .map(|path| home_relative(Path::new(&path)).unwrap_or_else(|| PathBuf::from(path)))
        .or_else(|| env::home_dir().map(|home| home.join(".pi/agent")));
    for profile in user.into_iter().chain([dir.join("agents/pi")]) {
        let profile = native_settings::path(&profile.join("settings.json"))?;
        ensure!(
            path_key(&profile)? != path_key(&target)?,
            "Pi project settings overlap a user profile"
        );
    }
    Ok((root, target))
}

pub(super) fn edit(
    dir: &Path,
    library: &Path,
    objects: &Path,
    bundles: &BTreeMap<String, Bundle>,
    project: &Path,
    edit: Edit<'_>,
) -> Result<String> {
    let _guard = native_settings::lock(dir)?;
    let (project, target) = target(dir, project)?;
    let identity = path_key(&target)?;
    let ledger = library
        .join("pi-projects")
        .join(hash(&identity))
        .with_extension("json");
    let mut settings = native_settings::read(&target)?;
    let mut store = Store::load(&ledger, settings.as_deref())?;
    let text = settings.as_deref().unwrap_or(EMPTY);
    let mut packages = Packages::read(&project, text)?;
    let objects = crate::rules::resolved_folder(&std::path::absolute(objects)?)?;
    let project_revision = match &edit {
        Edit::Read => None,
        Edit::Set {
            project_revision, ..
        }
        | Edit::Export { project_revision }
        | Edit::Archive {
            project_revision, ..
        }
        | Edit::Apply {
            project_revision, ..
        } => Some(*project_revision),
    };
    if let Some(expected) = project_revision {
        ensure!(
            store.revision(&identity, settings.as_deref())? == expected,
            "Pi project packages changed; reload them"
        );
    }
    let requested = match edit {
        Edit::Read => BTreeMap::new(),
        Edit::Export { .. } => return store.export(&project, &objects, &packages),
        Edit::Archive { name, output, .. } => {
            let assigned = store
                .entries
                .get(&name.to_ascii_lowercase())
                .context("Pi project package assignment was removed")?;
            let (content, tree) = assigned_snapshot(name, assigned, &project, &objects, &packages)?;
            return super::export_tree(&tree, content, output);
        }
        Edit::Set {
            name,
            bundle_revision,
            ..
        } => {
            let desired = bundle_revision
                .map(|expected| {
                    let bundle = check_revision(bundles, name, expected)?;
                    ensure!(is_pi(bundle), "expected a Pi package bundle");
                    verified_source(&project, &objects, &bundle.content)
                })
                .transpose()?;
            BTreeMap::from([(name.to_ascii_lowercase(), desired)])
        }
        Edit::Apply { assignments, .. } => {
            ensure!(
                assignments.len() <= MAX_LEDGER,
                "Pi project assignment document exceeds 1 MiB"
            );
            let plan: Assignments =
                nextjson::from_str(assignments.strip_prefix('\u{feff}').unwrap_or(assignments))
                    .map_err(anyhow::Error::msg)?;
            ensure!(
                plan.kind == ASSIGNMENT_KIND && plan.version == 1,
                "unsupported Pi project assignment document"
            );
            plan.packages
                .into_iter()
                .map(|(name, content)| {
                    let bundle = bundles
                        .get(&name)
                        .filter(|bundle| is_pi(bundle) && bundle.content == content)
                        .with_context(|| {
                            format!("import the matching Pi package snapshot for {name}")
                        })?;
                    let desired = verified_source(&project, &objects, &bundle.content)?;
                    ensure!(
                        store
                            .entries
                            .get(&name)
                            .is_none_or(|previous| *previous == desired),
                        "Pi project already has a different assignment for {name}"
                    );
                    Ok((name, Some(desired)))
                })
                .collect::<Result<_>>()?
        }
    };

    let mut updates = BTreeMap::new();
    let mut after = None;
    for (name, desired) in requested {
        let previous = store.entries.get(&name).map(String::as_str);
        let owned = packages
            .index_for_change(&project, previous, desired.as_deref())
            .with_context(|| format!("update Pi package {name}"))?;
        if previous != desired.as_deref() || (desired.is_some() && owned.is_none()) {
            let base = after.as_deref().unwrap_or(text);
            let updated = packages.change(base, owned, desired.as_deref())?;
            packages = Packages::read(&project, &updated)?;
            after = Some(updated);
            updates.insert(name, desired);
        }
    }
    if let Some(after) = after {
        if after != text {
            store.pending = Some(Pending {
                entries: updates.clone(),
                after: hash(&after),
            });
            store.save(&ledger)?;
            native_settings::write(&target, settings.as_deref(), Some(&after))?;
            settings = Some(after);
            store.pending = None;
        }
        store.update(updates);
        store.save(&ledger)?;
    }

    let mut rows = BTreeMap::new();
    for (key, bundle) in bundles.iter().filter(|(_, bundle)| is_pi(bundle)) {
        let folder = objects.join(&bundle.content);
        let desired = source(&project, &folder)?;
        let previous = store.entries.get(key);
        let installed = previous.and_then(|source| packages.owned(&project, source));
        let status = if previous.is_some_and(|source| {
            installed.is_none() && packages.matches(&project, source).next().is_some()
        }) {
            "conflict"
        } else if installed.is_some() && previous == Some(&desired) {
            "applied"
        } else if packages
            .matches(&project, &desired)
            .any(|index| Some(index) != installed)
        {
            "conflict"
        } else if previous.is_some() {
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
                assigned: previous.is_some(),
                can_export_snapshot: installed.is_some(),
                status,
            },
        );
    }
    for (key, source) in &store.entries {
        let installed = packages.owned(&project, source).is_some();
        rows.entry(key).or_insert_with(|| Row {
            name: key,
            version: None,
            bundle_revision: None,
            assigned: true,
            can_export_snapshot: installed,
            status: if installed { "detached" } else { "conflict" },
        });
    }
    encode(&View {
        revision: store.revision(&identity, settings.as_deref())?,
        packages: rows.into_values().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_bundles::{self, Edit as LibraryEdit};

    fn project_edit(data: &Path, project: &Path, edit: Edit<'_>) -> Result<Value> {
        let text = plugin_bundles::edit(data, LibraryEdit::PiProject { project, edit })?;
        nextjson::from_str(&text).map_err(anyhow::Error::msg)
    }

    #[test]
    fn syncs_only_owned_pi_packages_and_withdraws_after_bundle_removal() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let data = temp.path().join("data");
        let project = temp.path().join("project");
        let other = temp.path().join("other");
        let source = temp.path().join("source");
        fs::create_dir_all(project.join(".pi"))?;
        fs::create_dir(&other)?;
        fs::create_dir(&source)?;
        let settings = project.join(".pi/settings.json");
        let original = "\u{feff}{\r\n  \"packages\": [{\"source\":\"npm:unrelated\",\"autoload\":false}],\r\n  \"theme\" : \"dark\"\r\n}\r\n";
        fs::write(&settings, original)?;
        let source_text = source.to_str().context("fixture path must be UTF-8")?;
        let manifest = source.join(PI);
        let save = |version: &str, revision| -> Result<Bundle> {
            fs::write(
                &manifest,
                format!(
                    r#"{{"name":"reviewer","version":"{version}","pi":{{"extensions":["example.ts"]}}}}"#
                ),
            )?;
            fs::write(
                source.join("example.ts"),
                "export default function example() {}",
            )?;
            let (bundle, _) = plugin_bundles::load(source_text)?;
            plugin_bundles::edit(
                &data,
                LibraryEdit::Save {
                    source: source_text,
                    content: &bundle.content,
                    revision,
                },
            )?;
            Ok(bundle)
        };
        let set = |path: &Path, view: &Value, bundle_revision: Option<&str>| {
            project_edit(
                &data,
                path,
                Edit::Set {
                    name: "reviewer",
                    project_revision: view["revision"].as_str().expect("project revision"),
                    bundle_revision,
                },
            )
        };
        let first = save("1.0.0", None)?;
        let first_revision = revision(&first)?;
        let initial = project_edit(&data, &project, Edit::Read)?;
        let alternate = project_edit(&data, &other, Edit::Read)?;
        assert!(set(&other, &initial, Some(&first_revision)).is_err());
        let applied = set(&project, &initial, Some(&first_revision))?;
        assert_eq!(applied["packages"][0]["status"].as_str(), Some("applied"));
        assert!(set(&project, &initial, None).is_err());
        assert!(!other.join(".pi/settings.json").exists());
        set(&other, &alternate, Some(&first_revision))?;
        let second = save("1.1.0", Some(&first_revision))?;
        let second_revision = revision(&second)?;
        let pending = project_edit(&data, &project, Edit::Read)?;
        assert_eq!(pending["packages"][0]["status"].as_str(), Some("pending"));
        assert!(set(&project, &pending, Some(&first_revision)).is_err());
        set(&project, &pending, Some(&second_revision))?;
        let second_settings = fs::read_to_string(&settings)?;
        assert!(
            second_settings.contains(&second.content) && !second_settings.contains(&first.content)
        );
        assert!(fs::read_to_string(other.join(".pi/settings.json"))?.contains(&first.content));

        let synced = project_edit(&data, &project, Edit::Read)?;
        let travel = plugin_bundles::edit(
            &data,
            LibraryEdit::PiProject {
                project: &project,
                edit: Edit::Export {
                    project_revision: synced["revision"].as_str().unwrap(),
                },
            },
        )?;
        let carried: Assignments = nextjson::from_str(&travel).map_err(anyhow::Error::msg)?;
        assert_eq!(
            carried.packages,
            BTreeMap::from([("reviewer".to_owned(), second.content.clone())])
        );
        let archive = temp.path().join("carry.tar");
        plugin_bundles::edit(
            &data,
            LibraryEdit::Export {
                name: "reviewer",
                revision: &second_revision,
                output: &archive,
            },
        )?;
        let destination = temp.path().join("destination");
        fs::create_dir(&destination)?;
        let destination_data = temp.path().join("destination-data");
        let destination_view = project_edit(&destination_data, &destination, Edit::Read)?;
        let destination_revision = destination_view["revision"].as_str().unwrap();
        let apply = |assignments: &str| {
            project_edit(
                &destination_data,
                &destination,
                Edit::Apply {
                    project_revision: destination_revision,
                    assignments,
                },
            )
        };
        let destination_file = destination.join(".pi/settings.json");
        assert!(apply(&travel).is_err());
        assert!(!destination_file.exists());
        plugin_bundles::edit(
            &destination_data,
            LibraryEdit::Save {
                source: archive.to_str().unwrap(),
                content: &second.content,
                revision: None,
            },
        )?;
        let mut invalid = carried;
        invalid
            .packages
            .insert("zz-missing".to_owned(), second.content.clone());
        assert!(apply(&encode(&invalid)?).is_err());
        assert!(!destination_file.exists());
        assert_eq!(
            apply(&travel)?["packages"][0]["status"].as_str(),
            Some("applied")
        );
        let destination_settings = fs::read_to_string(&destination_file)?;
        let local_source = &Packages::read(&destination, &destination_settings)?.entries[0].source;
        assert!(Path::new(local_source).starts_with(&destination_data));
        assert!(local_source.ends_with(&second.content));

        let root = project.canonicalize()?;
        let packages = Packages::read(&root, &second_settings)?;
        let recorded = packages.entries[1].source.clone();
        let external = packages.change(&second_settings, Some(1), None)?;
        let root_object = jsonc::parse(&external, &[])?.context("expected fixture object")?;
        let array = root_object
            .array(&external, PACKAGES)?
            .context("expected fixture array")?;
        let external = array.append(
            &external,
            &nextjson::json!({"source": recorded, "extensions": []}),
        )?;
        fs::write(&settings, &external)?;
        let conflict = project_edit(&data, &project, Edit::Read)?;
        assert_eq!(conflict["packages"][0]["status"].as_str(), Some("conflict"));
        assert!(set(&project, &conflict, Some(&second_revision)).is_err());
        set(&project, &conflict, None)?;
        assert_eq!(fs::read_to_string(&settings)?, external);

        plugin_bundles::edit(
            &data,
            LibraryEdit::Remove {
                name: "reviewer",
                revision: &second_revision,
            },
        )?;
        let detached = project_edit(&data, &other, Edit::Read)?;
        assert_eq!(detached["packages"][0]["status"].as_str(), Some("detached"));
        let withdrawn = set(&other, &detached, None)?;
        assert!(withdrawn["packages"].as_array().is_some_and(Vec::is_empty));
        assert!(!fs::read_to_string(other.join(".pi/settings.json"))?.contains(&first.content));
        Ok(())
    }
}
