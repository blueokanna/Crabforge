use super::{
    Agent, Edit, Hook, Store, Target, backup, encode, grok, hash, native, reconcile, render,
};
use crate::native_settings;
use anyhow::{Context, Result, bail, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const FILES: &[(Agent, &str)] = &[
    (Agent::Claude, ".claude/settings.local.json"),
    (Agent::Codex, ".codex/hooks.json"),
    (Agent::Antigravity, ".agents/hooks.json"),
    (Agent::Grok, ".grok/hooks/crabforge.json"),
];

#[derive(NsonDeserialize, NsonSerialize)]
pub(super) struct Assignments {
    path: String,
    next_publication: u64,
    pub(super) hooks: BTreeMap<String, Assigned>,
}

#[derive(NsonDeserialize, NsonSerialize)]
pub(super) struct Assigned {
    publication: String,
    agents: BTreeSet<Agent>,
}

pub struct Assignment {
    pub name: String,
    pub revision: String,
    pub agents: Vec<String>,
}

#[derive(Clone, NsonSerialize)]
pub(super) struct Choice<'a> {
    hook: &'a Hook,
    agents: &'a BTreeSet<Agent>,
    compatible: Vec<Agent>,
    revision: String,
}

pub(super) struct View<'a> {
    pub(super) shared: Vec<Choice<'a>>,
    pub(super) wanted: BTreeMap<String, Hook>,
}

fn revision(key: &str, hook: &Hook, assigned: Option<&Assigned>) -> Result<String> {
    Ok(hash(&encode(&(key, hook, assigned))?))
}

fn identity(path: &Path) -> Result<(String, &str)> {
    let text = path.to_str().context("project path must be UTF-8")?;
    Ok((hash(text), text))
}

fn canonical(root: &Path) -> Result<PathBuf> {
    let root = root
        .canonicalize()
        .context("project directory is unavailable")?;
    ensure!(root.is_dir(), "expected a project directory");
    Ok(root)
}

fn targets(root: &Path, user: &BTreeMap<Target, &str>) -> Result<BTreeMap<Target, &'static str>> {
    let mut targets = FILES
        .iter()
        .map(|&(agent, file)| {
            let expected = root.join(file);
            let path = native_settings::path(&expected)?;
            let path = path.to_str().context("project hook path must be UTF-8")?;
            let expected = expected
                .to_str()
                .context("project hook path must be UTF-8")?;
            ensure!(
                same_path(path, expected),
                "project hook settings folder was redirected: {expected}"
            );
            Ok((
                Target {
                    agent,
                    path: path.into(),
                },
                "{}\n",
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    grok::discover_targets(&mut targets)?;
    for target in targets.keys() {
        ensure!(
            user.keys().all(|user| !same_path(&user.path, &target.path)),
            "project hook settings overlap a user profile: {}",
            target.path
        );
    }
    Ok(targets)
}

fn same_path(left: &str, right: &str) -> bool {
    left == right || (cfg!(windows) && left.eq_ignore_ascii_case(right))
}

fn entries(user: &Store, key: &str) -> BTreeMap<String, Hook> {
    user.projects
        .get(key)
        .into_iter()
        .flat_map(|project| &project.hooks)
        .filter_map(|(name, assigned)| {
            let mut hook = user.hooks.get(name)?.clone();
            hook.agents.clone_from(&assigned.agents);
            Some((assigned.publication.clone(), hook))
        })
        .collect()
}

fn validate(hook: &Hook, agents: &BTreeSet<Agent>) -> Result<()> {
    for &agent in agents {
        ensure!(
            FILES.iter().any(|&(supported, _)| supported == agent),
            "unsupported project hook client: {}",
            agent.id()
        );
        native::encode(agent, hook)?;
    }
    Ok(())
}

pub(super) fn update(
    projects: &mut BTreeMap<String, Assignments>,
    old: &str,
    hook: &Hook,
) -> Result<()> {
    for project in projects.values_mut() {
        if let Some(assigned) = project.hooks.remove(old) {
            validate(hook, &assigned.agents)
                .with_context(|| format!("project {}", project.path))?;
            project.hooks.insert(hook.name.clone(), assigned);
        }
    }
    Ok(())
}

fn assign(user: &mut Store, key: &str, path: &str, change: Assignment) -> Result<()> {
    let hook = user
        .hooks
        .get(&change.name)
        .context("hook was removed; reload the library")?;
    let previous = user
        .projects
        .get(key)
        .and_then(|project| project.hooks.get(&change.name));
    ensure!(
        revision(key, hook, previous)? == change.revision,
        "hook assignment changed; reload the library"
    );
    let agents = change
        .agents
        .iter()
        .map(|agent| Agent::parse(agent))
        .collect::<Result<BTreeSet<_>>>()?;
    validate(hook, &agents)?;
    if agents.is_empty() {
        if let Some(project) = user.projects.get_mut(key) {
            project.hooks.remove(&change.name);
        }
    } else {
        let project = user
            .projects
            .entry(key.into())
            .or_insert_with(|| Assignments {
                path: path.into(),
                next_publication: 0,
                hooks: BTreeMap::new(),
            });
        if let Some(assigned) = project.hooks.get_mut(&change.name) {
            assigned.agents = agents;
        } else {
            let publication = project.next_publication.to_string();
            project.next_publication = project
                .next_publication
                .checked_add(1)
                .context("project hook ID limit reached")?;
            project.hooks.insert(
                change.name,
                Assigned {
                    publication,
                    agents,
                },
            );
        }
    }
    Ok(())
}

pub(super) fn edit(
    library: &Path,
    path: &Path,
    user_targets: &BTreeMap<Target, &str>,
    edit: Edit,
) -> Result<String> {
    let root = canonical(path)?;
    let (key, path) = identity(&root)?;
    let targets = targets(&root, user_targets)?;
    let user_path = library.join("library.json");
    let mut user = Store::load(&user_path)?;
    let library = library.join("projects").join(&key);
    let store_path = library.join("library.json");
    let mut store = Store::load(&store_path)?;
    if !matches!(edit, Edit::Read) {
        fs::create_dir_all(&library)?;
        store.recover(&store_path, &targets)?;
    }
    let sync = matches!(edit, Edit::Assign(_) | Edit::Sync);
    match edit {
        Edit::Read | Edit::Sync => {}
        Edit::Assign(change) => {
            assign(&mut user, &key, path, change)?;
            user.save(&user_path)?;
        }
        Edit::RemoveInstalled { key } => {
            super::remove_installed(&targets, &store, &library.join("backups"), &key)?
        }
        Edit::Restore { id } => backup::restore(&library.join("backups"), &targets, &id)?,
        _ => bail!("edit the shared hook definition in the user library"),
    }
    let empty = BTreeSet::new();
    let shared = user
        .hooks
        .values()
        .map(|hook| {
            let assigned = user
                .projects
                .get(&key)
                .and_then(|project| project.hooks.get(&hook.name));
            Ok(Choice {
                hook,
                agents: assigned.map_or(&empty, |assigned| &assigned.agents),
                compatible: FILES
                    .iter()
                    .filter_map(|&(agent, _)| native::encode(agent, hook).ok().map(|_| agent))
                    .collect(),
                revision: revision(&key, hook, assigned)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    render(
        &library,
        &targets,
        &mut store,
        sync,
        Some(View {
            shared,
            wanted: entries(&user, &key),
        }),
        BTreeMap::new(),
    )
}

pub(super) fn sync(
    library: &Path,
    user: &Store,
    user_targets: &BTreeMap<Target, &str>,
) -> BTreeMap<String, String> {
    let mut errors = BTreeMap::new();
    for (key, project) in &user.projects {
        let result = (|| -> Result<Vec<String>> {
            let root = canonical(Path::new(&project.path))?;
            ensure!(identity(&root)?.0 == *key, "project directory moved");
            let targets = targets(&root, user_targets)?;
            let folder = library.join("projects").join(key);
            fs::create_dir_all(&folder)?;
            let path = folder.join("library.json");
            let mut store = Store::load(&path)?;
            store.recover(&path, &targets)?;
            let entries = entries(user, key);
            let mut issues = Vec::new();
            for (target, default) in &targets {
                let wanted = super::target_wanted(&entries, &store, target);
                match reconcile(&mut store, &path, target, default, true, wanted) {
                    Ok(placements) => issues.extend(
                        placements
                            .iter()
                            .filter(|row| matches!(row.status, "conflict" | "existing"))
                            .map(|row| {
                                format!("{} ({}): {}", row.name, target.agent.id(), row.status)
                            }),
                    ),
                    Err(error) => issues.push(format!("{}: {error:#}", target.agent.id())),
                }
            }
            Ok(issues)
        })();
        match result {
            Ok(issues) if !issues.is_empty() => {
                errors.insert(project.path.clone(), issues.join("\n"));
            }
            Err(error) => {
                errors.insert(project.path.clone(), format!("{error:#}"));
            }
            _ => {}
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use nextjson::{Value, json};

    #[test]
    fn assignments_follow_shared_definitions_and_preserve_native_changes() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let data = temp.path().join("data");
        let library = data.join("hooks");
        let a = temp.path().join("a");
        let b = temp.path().join("b");
        fs::create_dir_all(&a)?;
        fs::create_dir_all(&b)?;
        let user_targets = BTreeMap::new();
        let parse = |text: String| nextjson::from_str::<Value>(&text).map_err(anyhow::Error::msg);
        let call = |path: &Path, change| parse(edit(&library, path, &user_targets, change)?);
        let save = |name: &str, command: &str, previous| Edit::Save {
            source: json!({"name": name, "event": "turn-end", "command": command}).to_string(),
            previous,
        };
        let first = parse(super::super::edit_in(
            &data,
            &user_targets,
            save("report", "echo first", None),
        )?)?;
        let choice = |path: &Path| -> Result<String> {
            Ok(call(path, Edit::Read)?["shared"][0]["revision"]
                .as_str()
                .unwrap()
                .into())
        };
        let assign = |name: &str, revision, agents: &[&str]| {
            Edit::Assign(Assignment {
                name: name.into(),
                revision,
                agents: agents.iter().map(|agent| (*agent).into()).collect(),
            })
        };
        let old = choice(&a)?;
        assert!(call(&b, assign("report", old.clone(), &["codex"])).is_err());
        call(&a, assign("report", old.clone(), &["claude"]))?;
        assert!(call(&a, assign("report", old, &["codex"])).is_err());
        call(&b, assign("report", choice(&b)?, &["codex"]))?;
        let claude = a.join(".claude/settings.local.json");
        let codex = b.join(".codex/hooks.json");
        assert!(fs::read_to_string(&claude)?.contains("echo first"));
        assert!(fs::read_to_string(&codex)?.contains("echo first"));

        let first_revision = first["hooks"][0]["revision"].as_str().unwrap().into();
        let update = save(
            "report",
            "echo updated",
            Some(("report".into(), first_revision)),
        );
        let updated = parse(super::super::edit_in(&data, &user_targets, update)?)?;
        assert!(fs::read_to_string(&claude)?.contains("echo updated"));
        assert!(fs::read_to_string(&codex)?.contains("echo updated"));
        let external = fs::read_to_string(&claude)?.replace("echo updated", "echo external");
        fs::write(&claude, &external)?;
        let update_revision = updated["hooks"][0]["revision"].as_str().unwrap().into();
        let rename = save(
            "renamed",
            "echo renamed",
            Some(("report".into(), update_revision)),
        );
        let renamed = parse(super::super::edit_in(&data, &user_targets, rename)?)?;
        assert!(
            renamed["project_errors"]
                .to_string()
                .contains("renamed (claude): conflict")
        );
        assert_eq!(fs::read_to_string(&claude)?, external);
        assert!(fs::read_to_string(&codex)?.contains("echo renamed"));
        let withdrawn = call(&a, assign("renamed", choice(&a)?, &[]))?;
        assert_eq!(
            withdrawn["placements"][0]["status"].as_str(),
            Some("conflict")
        );
        assert_eq!(fs::read_to_string(&claude)?, external);
        assert!(fs::read_to_string(&codex)?.contains("echo renamed"));

        let key = withdrawn["installed"][0]["key"].as_str().unwrap().into();
        let removed = call(&a, Edit::RemoveInstalled { key })?;
        let id = removed["backups"][0]["id"].as_str().unwrap().to_owned();
        assert!(call(&b, Edit::Restore { id: id.clone() }).is_err());
        call(&a, Edit::Restore { id })?;
        assert_eq!(fs::read_to_string(&claude)?, external);
        let revision = renamed["hooks"][0]["revision"].as_str().unwrap().into();
        super::super::edit_in(
            &data,
            &user_targets,
            Edit::Remove {
                name: "renamed".into(),
                revision,
            },
        )?;
        assert!(!fs::read_to_string(&codex)?.contains("echo renamed"));
        assert_eq!(fs::read_to_string(&claude)?, external);
        Ok(())
    }

    #[test]
    fn rejects_a_project_target_that_overlaps_a_user_profile() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = canonical(temp.path())?;
        let path = root.join(".codex/hooks.json");
        let user = BTreeMap::from([(
            Target {
                agent: Agent::Codex,
                path: path.to_str().unwrap().into(),
            },
            "{}\n",
        )]);
        assert!(
            targets(&root, &user)
                .err()
                .unwrap()
                .to_string()
                .contains("overlap")
        );
        assert!(!path.exists());
        Ok(())
    }
}
