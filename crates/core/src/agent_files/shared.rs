use super::{Entry, Kind, Placement, Store, hash, native::Profiles, path_key, placements};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

type Assignments = BTreeMap<String, Vec<String>>;

#[derive(NsonDeserialize, NsonSerialize)]
pub(super) struct Project {
    path: String,
    assignments: Assignments,
}

impl Project {
    pub(super) fn retire_assignments(&mut self) {
        for agents in self.assignments.values_mut() {
            agents.retain(|agent| agent != "gemini");
        }
    }
}

pub(super) fn store_path(dir: &Path, key: &str) -> PathBuf {
    dir.join("subagents/projects")
        .join(format!("{}.json", hash(key)))
}

fn assigned<'a>(user: &'a Store, key: &str) -> Option<&'a Assignments> {
    user.projects.get(key).map(|project| &project.assignments)
}

pub(super) fn entries(user: &Store, key: &str) -> BTreeMap<String, Entry> {
    assigned(user, key)
        .into_iter()
        .flatten()
        .filter_map(|(name, agents)| {
            let mut entry = user.entries.get(name)?.clone();
            entry.agents.clone_from(agents);
            Some((name.clone(), entry))
        })
        .collect()
}

fn revision(entry: &Entry, agents: &[String]) -> Result<String> {
    Ok(hash(&format!(
        "{}\0{}",
        super::revision(entry)?,
        agents.join("\0")
    )))
}

#[derive(NsonSerialize)]
pub(super) struct Choice<'a> {
    name: &'a str,
    description: &'a str,
    agents: &'a [String],
    revision: String,
    conflict: bool,
}

pub(super) fn choices<'a>(user: &'a Store, key: &str, local: &Store) -> Result<Vec<Choice<'a>>> {
    let assignments = assigned(user, key);
    user.entries
        .values()
        .map(|entry| {
            let agents = assignments
                .and_then(|rows| rows.get(&entry.name))
                .map_or(&[][..], Vec::as_slice);
            Ok(Choice {
                name: &entry.name,
                description: &entry.description,
                agents,
                revision: revision(entry, agents)?,
                conflict: local.entries.contains_key(&entry.name),
            })
        })
        .collect()
}

pub struct Assignment {
    pub name: String,
    pub revision: String,
    pub agents: Vec<String>,
}

pub(super) fn assign(
    user: &mut Store,
    key: &str,
    project: &Path,
    local: &Store,
    profiles: &Profiles,
    assignment: Assignment,
) -> Result<()> {
    let Assignment {
        name,
        revision: expected,
        mut agents,
    } = assignment;
    let Some(entry) = user.entries.get(&name) else {
        anyhow::bail!("subagent was removed; reload the library");
    };
    let previous = assigned(user, key)
        .and_then(|rows| rows.get(&name))
        .map_or(&[][..], Vec::as_slice);
    ensure!(
        revision(entry, previous)? == expected,
        "assignment changed; reload the library"
    );
    ensure!(
        agents.is_empty() || !local.entries.contains_key(&name),
        "a project subagent already has that name"
    );
    agents.sort_unstable();
    agents.dedup();
    ensure!(
        agents
            .iter()
            .all(|agent| profiles.agents.contains(&agent.as_str())),
        "unsupported project agent assignment"
    );
    if agents.is_empty() {
        if let Some(project) = user.projects.get_mut(key) {
            project.assignments.remove(&name);
        }
    } else {
        let project_path = project.to_str().context("project path must be UTF-8")?;
        user.projects
            .entry(key.to_owned())
            .or_insert_with(|| Project {
                path: project_path.to_owned(),
                assignments: BTreeMap::new(),
            })
            .assignments
            .insert(name, agents);
    }
    Ok(())
}

pub(super) fn rename(user: &mut Store, dir: &Path, old: &str, name: &str) -> Result<()> {
    if old != name {
        for (key, project) in &mut user.projects {
            if let Some(agents) = project.assignments.remove(old) {
                let local = Store::load(&store_path(dir, key))?;
                ensure!(
                    !local.entries.contains_key(name),
                    "a project subagent in {} already has that name",
                    project.path
                );
                project.assignments.insert(name.to_owned(), agents);
            }
        }
    }
    Ok(())
}

pub(super) fn withdraw(user: &mut Store, name: &str) {
    for project in user.projects.values_mut() {
        project.assignments.remove(name);
    }
}

#[derive(NsonSerialize)]
pub(super) struct Report<'a> {
    project: &'a str,
    placements: Vec<Placement>,
    error: Option<String>,
}

pub(super) fn sync<'a>(dir: &Path, user: &'a Store) -> Vec<Report<'a>> {
    user.projects
        .iter()
        .map(|(key, project)| {
            let result = (|| -> Result<Vec<Placement>> {
                let path =
                    fs::canonicalize(&project.path).context("project directory is unavailable")?;
                let identity = path.to_str().context("project path must be UTF-8")?;
                ensure!(
                    path.is_dir() && path_key(identity) == *key,
                    "project directory moved"
                );
                let store_path = store_path(dir, key);
                let profiles = Profiles::load(dir, Kind::Subagent, Some(&path))?;
                let mut store = Store::load(&store_path)?;
                store.recover(&store_path, &profiles)?;
                placements(
                    &mut store,
                    &store_path,
                    &profiles,
                    &entries(user, key),
                    true,
                    false,
                )
            })();
            match result {
                Ok(placements) => Report {
                    project: &project.path,
                    placements,
                    error: None,
                },
                Err(error) => Report {
                    project: &project.path,
                    placements: Vec::new(),
                    error: Some(format!("{error:#}")),
                },
            }
        })
        .collect()
}
