use crate::{
    agents::NativeAgent, config::read_limited, project_instructions::replace,
    rules::resolved_folder,
};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use saphyr::{LoadableYamlNode, YamlOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fs::{self, File},
    io::ErrorKind,
    path::{Path, PathBuf},
};

mod remote;
mod tree;
use tree::Tree;

const AGENTS: &[&str] = &["claude", "codex", "gemini", "opencode", "pi"];
const MAX_LIBRARY: usize = 4 * 1024 * 1024;

#[derive(Clone, NsonDeserialize, NsonSerialize)]
struct Skill {
    name: String,
    description: String,
    source: String,
    content: String,
    agents: Vec<String>,
}

fn encode(value: &impl NsonSerialize) -> Result<String> {
    nextjson::to_string(value).map_err(anyhow::Error::msg)
}

impl Skill {
    fn revision(&self) -> Result<String> {
        Ok(hex::encode(Sha256::digest(encode(self)?.as_bytes())))
    }
}

#[derive(NsonSerialize)]
struct Entry<'a> {
    skill: &'a Skill,
    revision: String,
}

#[derive(NsonSerialize)]
struct Placement {
    name: String,
    agent: String,
    path: Option<String>,
    status: &'static str,
    error: Option<String>,
}

#[derive(NsonSerialize)]
struct View<'a> {
    skills: Vec<Entry<'a>>,
    placements: Vec<Placement>,
}

#[derive(Clone, PartialEq, Eq, NsonDeserialize, NsonSerialize)]
struct Target {
    name: String,
    agent: String,
    path: String,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Applied {
    target: Target,
    content: String,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Pending {
    target: Target,
    staging: String,
    before: Option<String>,
    after: Option<String>,
}

#[derive(Default, NsonDeserialize, NsonSerialize)]
struct Store {
    skills: BTreeMap<String, Skill>,
    applied: Vec<Applied>,
    pending: Option<Pending>,
}

pub enum Edit {
    Read,
    Import {
        source: PathBuf,
        agents: Vec<String>,
        expected: Option<String>,
    },
    Assign {
        name: String,
        agents: Vec<String>,
        expected: Option<String>,
    },
    Remove {
        name: String,
        expected: Option<String>,
    },
    Sync,
}

fn validate_name(name: &str) -> Result<()> {
    let alphanumeric = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    ensure!(
        name.len() <= 64
            && name.bytes().next().is_some_and(alphanumeric)
            && name.bytes().last().is_some_and(alphanumeric)
            && name.bytes().all(|b| alphanumeric(b) || b == b'-')
            && !name.contains("--"),
        "a skill name needs 1–64 lowercase letters, digits or single hyphens between words"
    );
    let reserved_number = name.len() == 4
        && matches!(&name.as_bytes()[..3], b"com" | b"lpt")
        && name.as_bytes()[3].is_ascii_digit();
    ensure!(
        !matches!(name, "con" | "prn" | "aux" | "nul") && !reserved_number,
        "skill name is reserved on Windows"
    );
    Ok(())
}

fn normalize_agents(mut agents: Vec<String>) -> Result<Vec<String>> {
    ensure!(
        agents.iter().all(|agent| AGENTS.contains(&agent.as_str())),
        "unsupported skill agent"
    );
    agents.sort_unstable();
    agents.dedup();
    Ok(agents)
}

fn details(tree: &Tree) -> Result<(String, String)> {
    let mut lines = tree.document()?.trim_start_matches('\u{feff}').lines();
    ensure!(
        lines.next() == Some("---"),
        "SKILL.md needs YAML front matter"
    );
    let mut header = String::new();
    for line in lines {
        if line == "---" {
            let yaml = YamlOwned::load_from_str(&header)?;
            let document = yaml.first().context("empty SKILL.md front matter")?;
            let field = |key: &str| -> Result<&str> {
                document[key]
                    .as_str()
                    .with_context(|| format!("SKILL.md requires a {key} string"))
            };
            let name = field("name")?;
            validate_name(name)?;
            let description = field("description")?.trim();
            ensure!(
                !description.is_empty() && description.chars().count() <= 1024,
                "skill description needs 1–1024 characters"
            );
            return Ok((name.to_owned(), description.to_owned()));
        }
        header.push_str(line);
        header.push('\n');
        ensure!(
            header.len() <= 64 * 1024,
            "SKILL.md front matter exceeds 64 KiB"
        );
    }
    anyhow::bail!("SKILL.md front matter is missing its closing delimiter")
}

type Homes = BTreeMap<&'static str, Vec<PathBuf>>;

fn homes(dir: &Path) -> Result<Homes> {
    let home = env::home_dir().context("user home directory is unavailable")?;
    let setting = |key| {
        env::var_os(key)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    let claude = NativeAgent::Claude
        .home()
        .context("Claude Code profile is unavailable")?;
    let codex = NativeAgent::Codex
        .home()
        .context("Codex profile is unavailable")?;
    let gemini = setting("GEMINI_CLI_HOME")
        .unwrap_or_else(|| home.clone())
        .join(".gemini");
    let opencode = setting("OPENCODE_CONFIG_DIR").unwrap_or_else(|| {
        setting("XDG_CONFIG_HOME")
            .unwrap_or_else(|| home.join(".config"))
            .join("opencode")
    });
    let pi = setting("PI_CODING_AGENT_DIR").unwrap_or_else(|| home.join(".pi/agent"));
    Ok(BTreeMap::from([
        ("claude", vec![claude]),
        ("codex", vec![codex]),
        ("gemini", vec![gemini, dir.join("agents/gemini/.gemini")]),
        ("opencode", vec![opencode]),
        ("pi", vec![pi, dir.join("agents/pi")]),
    ]))
}

fn target(profile: &Path, name: &str, agent: &str) -> Result<Target> {
    validate_name(name)?;
    let profile = resolved_folder(&std::path::absolute(profile)?)?;
    let skills = resolved_folder(&profile.join("skills"))?;
    ensure!(
        skills.starts_with(&profile) && skills != profile,
        "skill folder points outside the agent profile"
    );
    Ok(Target {
        name: name.to_owned(),
        agent: agent.to_owned(),
        path: skills
            .join(name)
            .to_str()
            .context("skill destination is not UTF-8")?
            .to_owned(),
    })
}

fn current(path: &Path) -> Result<Option<String>> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(Some(Tree::read(path)?.hash())),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

fn snapshot_path(library: &Path, hash: &str) -> Result<PathBuf> {
    ensure!(
        hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid skill content hash"
    );
    Ok(library.join("snapshots").join(hash))
}

fn snapshot(library: &Path, content: &str, tree: &Tree) -> Result<()> {
    let path = snapshot_path(library, content)?;
    let folder = path.parent().context("missing snapshot directory")?;
    fs::create_dir_all(folder)?;
    if let Some(existing) = current(&path)? {
        ensure!(existing == content, "skill library snapshot was modified");
        return Ok(());
    }
    let temp = tempfile::tempdir_in(folder)?;
    let staged = temp.path().join("content");
    tree.write(&staged)?;
    fs::rename(staged, path)?;
    Ok(())
}

impl Store {
    fn load(path: &Path) -> Result<Self> {
        match read_limited(path, MAX_LIBRARY) {
            Ok(bytes) => nextjson::from_slice(&bytes).map_err(anyhow::Error::msg),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        let text = encode(self)?;
        ensure!(
            text.len() <= MAX_LIBRARY,
            "skill library exceeds its size limit"
        );
        replace(path, &text)
    }

    fn skill_mut(&mut self, name: &str, expected: Option<&str>) -> Result<&mut Skill> {
        let skill = self
            .skills
            .get_mut(name)
            .context("skill was removed; reload the library")?;
        if let Some(expected) = expected {
            ensure!(
                skill.revision()? == expected,
                "skill changed; reload the library"
            );
        }
        Ok(skill)
    }

    fn owner(&self, target: &Target) -> Option<&str> {
        self.applied
            .iter()
            .find(|value| value.target == *target)
            .map(|value| value.content.as_str())
    }

    fn recover(&mut self, path: &Path) -> Result<()> {
        let Some(pending) = &self.pending else {
            return Ok(());
        };
        let Pending {
            target,
            staging,
            before,
            after,
        } = pending;
        let dest = Path::new(&target.path);
        let folder = dest.parent().context("missing skill parent")?;
        let staging = Path::new(staging);
        ensure!(
            staging.parent() == folder.parent()
                && staging
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(".crabforge-skill-"))
                && resolved_folder(folder)? == folder
                && resolved_folder(staging)? == staging,
            "skill destination changed during update. The recovery folder is {}",
            staging.display()
        );
        let backup = staging.join("previous");
        let old = current(&backup)?;
        let actual = current(dest)?;
        if actual.is_none() && old.is_some() && (after.is_some() || old != *before) {
            fs::rename(&backup, dest)?;
        } else if actual == *after {
            ensure!(
                old.is_none() || old == *before,
                "previous skill changed at {}",
                backup.display()
            );
            self.applied.retain(|value| value.target != *target);
            if let Some(content) = after {
                self.applied.push(Applied {
                    target: target.clone(),
                    content: content.clone(),
                });
            }
        } else {
            ensure!(
                old.is_none(),
                "skill destination now contains an external edit. The previous version is in {}",
                backup.display()
            );
        }
        let staging = staging.to_path_buf();
        let finished = self.pending.take();
        if let Err(error) = self.save(path) {
            self.pending = finished;
            return Err(error);
        }
        fs::remove_dir_all(&staging)
            .with_context(|| format!("clear completed skill staging at {}", staging.display()))?;
        Ok(())
    }

    fn publish(
        &mut self,
        store: &Path,
        target: &Target,
        after: Option<&str>,
        sync: bool,
    ) -> Result<&'static str> {
        let dest = Path::new(&target.path);
        let before = match current(dest) {
            Ok(value) => value,
            Err(error) if dest.symlink_metadata().is_ok() => {
                return Err(error).context("existing skill folder was kept");
            }
            Err(error) => return Err(error),
        };
        if before.is_some() && before.as_deref() != self.owner(target) {
            return Ok("conflict");
        }
        let status = if after.is_some() {
            "applied"
        } else {
            "removed"
        };
        if before.as_deref() == after {
            if before.is_none() && sync && self.owner(target).is_some() {
                self.applied.retain(|value| value.target != *target);
                self.save(store)?;
            }
            return Ok(status);
        }
        if !sync || self.pending.is_some() {
            return Ok("pending");
        }
        let folder = dest.parent().context("missing native skill folder")?;
        let profile = folder.parent().context("missing native skill profile")?;
        fs::create_dir_all(folder)?;
        ensure!(
            resolved_folder(folder)? == folder,
            "native skill folder changed"
        );
        let temp = tempfile::Builder::new()
            .prefix(".crabforge-skill-")
            .tempdir_in(profile)?;
        let next = temp.path().join("next");
        if let Some(content) = after {
            let library = store.parent().context("missing skill library directory")?;
            let tree = Tree::read(&snapshot_path(library, content)?)?;
            ensure!(
                tree.hash() == content,
                "skill library snapshot was modified"
            );
            tree.write(&next)?;
        }
        ensure!(
            resolved_folder(folder)? == folder && current(dest)? == before,
            "native skill changed during update"
        );
        self.pending = Some(Pending {
            target: target.clone(),
            staging: temp
                .path()
                .to_str()
                .context("skill staging path is not UTF-8")?
                .to_owned(),
            before: before.clone(),
            after: after.map(str::to_owned),
        });
        if let Err(error) = self.save(store) {
            self.pending = None;
            return Err(error);
        }
        let staging = temp.keep();
        let update = (|| -> Result<()> {
            ensure!(
                resolved_folder(folder)? == folder && current(dest)? == before,
                "native skill changed during update"
            );
            if before.is_some() {
                let backup = staging.join("previous");
                fs::rename(dest, &backup)?;
                ensure!(
                    current(&backup)? == before,
                    "native skill was modified during update"
                );
            }
            if after.is_some() {
                ensure!(
                    current(dest)?.is_none(),
                    "native skill destination appeared during update"
                );
                fs::rename(next, dest)?;
            }
            Ok(())
        })();
        self.recover(store)?;
        update?;
        Ok(status)
    }
}

pub fn library(dir: &Path, edit: Edit) -> Result<String> {
    library_in(dir, &homes(dir)?, edit)
}

fn load_source(source: &Path) -> Result<(String, Tree)> {
    if let Some(url) = source
        .to_str()
        .filter(|value| value.starts_with("https://"))
    {
        return remote::load(url);
    }
    ensure!(
        !tree::is_link(&fs::symlink_metadata(source).context("open the skill source folder")?),
        "the skill source must be a regular directory"
    );
    let source = source
        .canonicalize()
        .context("open the skill source folder")?;
    let tree = Tree::read(&source)?;
    Ok((
        source
            .into_os_string()
            .into_string()
            .ok()
            .context("skill source path is not UTF-8")?,
        tree,
    ))
}

fn library_in(dir: &Path, homes: &Homes, edit: Edit) -> Result<String> {
    // A remote archive can take time to download. Check the library revision
    // after reading it, under the same lock used for all local updates.
    let imported = match &edit {
        Edit::Import { source, .. } => Some(load_source(source)?),
        _ => None,
    };
    let library = dir.join("skills");
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
        store.recover(&path)?;
    }
    let mut changed = true;
    match edit {
        Edit::Read | Edit::Sync => changed = false,
        Edit::Import {
            agents, expected, ..
        } => {
            let agents = normalize_agents(agents)?;
            let (source, tree) = imported.expect("skill import source loaded");
            let (name, description) = details(&tree)?;
            if expected.is_some() {
                store.skill_mut(&name, expected.as_deref())?;
            } else {
                ensure!(
                    !store.skills.contains_key(&name),
                    "a skill named {name} already exists; reload its revision to update it"
                );
            }
            let content = tree.hash();
            snapshot(&library, &content, &tree)?;
            store.skills.insert(
                name.clone(),
                Skill {
                    name,
                    description,
                    source,
                    content,
                    agents,
                },
            );
        }
        Edit::Assign {
            name,
            agents,
            expected,
        } => {
            store.skill_mut(&name, expected.as_deref())?.agents = normalize_agents(agents)?;
        }
        Edit::Remove { name, expected } => {
            store.skill_mut(&name, expected.as_deref())?;
            store.skills.remove(&name);
        }
    }
    if changed {
        store.save(&path)?;
    }
    let mut wanted = BTreeMap::new();
    for skill in store.skills.values() {
        for agent in &skill.agents {
            for profile in homes.get(agent.as_str()).into_iter().flatten() {
                wanted.insert((&skill.name, agent, profile), Some(skill.content.as_str()));
            }
        }
    }
    for applied in &store.applied {
        let previous = &applied.target;
        for profile in homes.get(previous.agent.as_str()).into_iter().flatten() {
            if target(profile, &previous.name, &previous.agent).is_ok_and(|now| now == *previous) {
                wanted
                    .entry((&previous.name, &previous.agent, profile))
                    .or_insert(None);
            }
        }
    }
    let wanted: Vec<_> = wanted
        .into_iter()
        .map(|((name, agent, profile), content)| {
            (
                name.clone(),
                agent.clone(),
                target(profile, name, agent),
                content.map(str::to_owned),
            )
        })
        .collect();
    let mut seen = BTreeSet::new();
    let mut placements = Vec::new();
    for (name, agent, destination, content) in wanted {
        if let Ok(target) = &destination
            && !seen.insert((agent.clone(), target.path.clone()))
        {
            continue;
        }
        let mut placement = Placement {
            name,
            agent,
            path: None,
            status: "error",
            error: None,
        };
        let applied = (|| {
            let target = destination?;
            placement.path = Some(target.path.clone());
            store.publish(&path, &target, content.as_deref(), sync)
        })();
        match applied {
            Ok(status) => placement.status = status,
            Err(error) => placement.error = Some(format!("{error:#}")),
        }
        placements.push(placement);
    }
    let skills = store
        .skills
        .values()
        .map(|skill| {
            Ok(Entry {
                skill,
                revision: skill.revision()?,
            })
        })
        .collect::<Result<_>>()?;
    encode(&View { skills, placements })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nextjson::Value;

    fn report(dir: &Path, homes: &Homes, edit: Edit) -> Result<Value> {
        nextjson::from_str(&library_in(dir, homes, edit)?).map_err(anyhow::Error::msg)
    }

    fn revision(report: &Value) -> String {
        report
            .pointer("/skills/0/revision")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn statuses(report: &Value) -> Vec<&str> {
        report["placements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|target| target["status"].as_str().unwrap())
            .collect()
    }

    fn source(root: &Path) -> Result<PathBuf> {
        let folder = root.join("source");
        fs::create_dir_all(folder.join("references"))?;
        fs::write(folder.join("SKILL.md"), b"---\r\nname: diagnose\r\ndescription: Describe diagnostics.\r\n---\r\nShow the units.\r\n")?;
        fs::write(folder.join("references/units.bin"), [0, 100, 255])?;
        Ok(folder)
    }

    #[test]
    fn publishes_full_bundles_and_keeps_unowned_or_modified_copies() -> Result<()> {
        let root = tempfile::tempdir()?;
        let dir = root.path().join("data");
        let source = source(root.path())?;
        let claude = root.path().join("claude");
        let codex = root.path().join("codex");
        let claude_skill = claude.join("skills/diagnose");
        let codex_skill = codex.join("skills/diagnose");
        fs::create_dir_all(&codex_skill)?;
        fs::write(codex_skill.join("SKILL.md"), "External skill")?;
        let homes = BTreeMap::from([("claude", vec![claude]), ("codex", vec![codex])]);
        let agents = vec!["codex".into(), "claude".into()];
        let imported = report(
            &dir,
            &homes,
            Edit::Import {
                source: source.clone(),
                agents: agents.clone(),
                expected: None,
            },
        )?;
        assert_eq!(statuses(&imported), ["applied", "conflict"]);
        assert_eq!(
            fs::read(claude_skill.join("SKILL.md"))?,
            fs::read(source.join("SKILL.md"))?
        );
        assert_eq!(
            fs::read(claude_skill.join("references/units.bin"))?,
            [0, 100, 255]
        );

        fs::write(source.join("references/units.bin"), [255, 100, 0])?;
        assert!(
            report(
                &dir,
                &homes,
                Edit::Import {
                    source: source.clone(),
                    agents: agents.clone(),
                    expected: Some("stale".into())
                }
            )
            .is_err()
        );
        let updated = report(
            &dir,
            &homes,
            Edit::Import {
                source,
                agents,
                expected: Some(revision(&imported)),
            },
        )?;
        assert_eq!(statuses(&updated), ["applied", "conflict"]);
        assert_eq!(
            fs::read(claude_skill.join("references/units.bin"))?,
            [255, 100, 0]
        );
        fs::write(claude_skill.join("references/units.bin"), b"local edit")?;
        let removed = report(
            &dir,
            &homes,
            Edit::Remove {
                name: "diagnose".into(),
                expected: Some(revision(&updated)),
            },
        )?;
        assert_eq!(statuses(&removed), ["conflict"]);
        assert_eq!(
            fs::read(claude_skill.join("references/units.bin"))?,
            b"local edit"
        );
        assert_eq!(
            fs::read_to_string(codex_skill.join("SKILL.md"))?,
            "External skill"
        );
        Ok(())
    }

    #[test]
    fn recovers_a_process_interrupted_between_native_renames() -> Result<()> {
        let root = tempfile::tempdir()?;
        let dir = root.path().join("data");
        let source = source(root.path())?;
        let homes = BTreeMap::from([("claude", vec![root.path().join("claude")])]);
        report(
            &dir,
            &homes,
            Edit::Import {
                source,
                agents: vec!["claude".into()],
                expected: None,
            },
        )?;
        let store_path = dir.join("skills/library.json");
        let mut store = Store::load(&store_path)?;
        let target = target(&homes["claude"][0], "diagnose", "claude")?;
        let dest = Path::new(&target.path);
        let staging = dest
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(".crabforge-skill-interrupted");
        fs::create_dir(&staging)?;
        let before = current(dest)?;
        store.pending = Some(Pending {
            target: target.clone(),
            staging: staging.to_str().unwrap().into(),
            before: before.clone(),
            after: Some("b".repeat(64)),
        });
        store.save(&store_path)?;
        fs::rename(dest, staging.join("previous"))?;

        let recovered = report(&dir, &homes, Edit::Sync)?;
        assert_eq!(statuses(&recovered), ["applied"]);
        assert_eq!(current(dest)?, before);
        assert!(!staging.exists());
        let removed = report(
            &dir,
            &homes,
            Edit::Remove {
                name: "diagnose".into(),
                expected: Some(revision(&recovered)),
            },
        )?;
        assert_eq!(statuses(&removed), ["removed"]);
        assert!(!dest.exists());
        Ok(())
    }

    #[test]
    fn reconciles_the_native_and_gateway_profiles_for_one_agent() -> Result<()> {
        let root = tempfile::tempdir()?;
        let dir = root.path().join("data");
        let native = root.path().join("native/.gemini");
        let gateway = dir.join("agents/gemini/.gemini");
        let homes = BTreeMap::from([("gemini", vec![native.clone(), gateway.clone()])]);
        let imported = report(
            &dir,
            &homes,
            Edit::Import {
                source: source(root.path())?,
                agents: vec!["gemini".into()],
                expected: None,
            },
        )?;
        assert_eq!(statuses(&imported), ["applied", "applied"]);
        let folder = |profile: &Path| profile.join("skills/diagnose");
        for profile in [&native, &gateway] {
            assert_eq!(
                fs::read(folder(profile).join("references/units.bin"))?,
                [0, 100, 255]
            );
        }
        let aliased = BTreeMap::from([("gemini", vec![gateway.clone(), gateway.canonicalize()?])]);
        assert_eq!(statuses(&report(&dir, &aliased, Edit::Read)?), ["applied"]);
        let edited = folder(&native).join("references/units.bin");
        fs::write(&edited, b"local edit")?;
        let removed = report(
            &dir,
            &homes,
            Edit::Assign {
                name: "diagnose".into(),
                agents: vec![],
                expected: Some(revision(&imported)),
            },
        )?;
        assert_eq!(statuses(&removed), ["removed", "conflict"]);
        assert!(!folder(&gateway).exists());
        assert_eq!(fs::read(edited)?, b"local edit");
        Ok(())
    }
}
