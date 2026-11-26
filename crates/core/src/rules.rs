use crate::{agents::NativeAgent, config::read_limited, project_instructions::replace};
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

const AGENTS: [&str; 3] = ["claude", "cline", "kiro-cli"];
const MAX_BODY: usize = 100_000;
const MAX_RULE_FILE: usize = 256 * 1024;
const MAX_STORE: usize = 16 * 1024 * 1024;

#[derive(Clone, NsonDeserialize, NsonSerialize)]
pub struct Rule {
    pub name: String,
    pub body: String,
    pub paths: Vec<String>,
    pub agents: Vec<String>,
}

#[derive(NsonSerialize)]
struct RuleView<'a> {
    rule: &'a Rule,
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
    rules: Vec<RuleView<'a>>,
    placements: Vec<Placement>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, NsonDeserialize, NsonSerialize)]
struct Destination {
    agent: String,
    name: String,
    path: String,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Applied {
    target: Destination,
    hash: String,
}

// Record the expected result before touching an agent file. If interrupted,
// reconcile the ownership record with the bytes on disk on the next sync.
#[derive(NsonDeserialize, NsonSerialize)]
struct Pending {
    target: Destination,
    after: Option<String>,
}

#[derive(Default, NsonDeserialize, NsonSerialize)]
struct Store {
    rules: BTreeMap<String, Rule>,
    applied: Vec<Applied>,
    pending: Option<Pending>,
}

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn encode(value: &impl NsonSerialize) -> Result<String> {
    nextjson::to_string(value).map_err(anyhow::Error::msg)
}

fn revision(rule: &Rule) -> Result<String> {
    Ok(hash(encode(rule)?.as_bytes()))
}

fn validate_name(name: &str) -> Result<()> {
    let alphanumeric = |b: &u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    let bytes = name.as_bytes();
    ensure!(
        bytes.len() <= 64
            && bytes.first().is_some_and(alphanumeric)
            && bytes.last().is_some_and(alphanumeric)
            && bytes
                .iter()
                .all(|b| alphanumeric(b) || matches!(b, b'-' | b'_')),
        "use 1–64 lowercase letters, digits, hyphens or underscores for a rule name, starting and ending with a letter or digit"
    );
    ensure!(
        !matches!(name, "con" | "prn" | "aux" | "nul")
            && !(name.len() == 4
                && matches!(&bytes[..3], b"com" | b"lpt")
                && matches!(bytes[3], b'1'..=b'9')),
        "the rule name is reserved on Windows"
    );
    Ok(())
}

impl Rule {
    fn normalize(mut self) -> Result<Self> {
        self.name = self.name.trim().to_owned();
        validate_name(&self.name)?;
        self.body = self
            .body
            .replace("\r\n", "\n")
            .trim_end_matches([' ', '\t', '\n'])
            .to_owned();
        ensure!(
            !self.body.trim().is_empty()
                && self.body.len() <= MAX_BODY
                && !self.body.contains('\0'),
            "a rule needs 1–{MAX_BODY} bytes of Markdown without NUL characters"
        );
        let mut seen = BTreeSet::new();
        self.paths = self
            .paths
            .iter()
            .flat_map(|value| value.lines())
            .map(str::trim)
            .filter(|value| !value.is_empty() && seen.insert(*value))
            .map(str::to_owned)
            .collect();
        ensure!(
            self.paths.len() <= 50
                && self
                    .paths
                    .iter()
                    .all(|value| value.len() <= 500 && !value.contains(['\r', '\0'])),
            "a rule may have up to 50 path patterns of at most 500 bytes each"
        );
        ensure!(
            self.agents
                .iter()
                .all(|agent| AGENTS.contains(&agent.as_str())),
            "unsupported rule agent"
        );
        self.agents.sort_unstable();
        self.agents.dedup();
        Ok(self)
    }

    fn render(&self, agent: &str) -> Result<String> {
        ensure!(
            agent != "kiro-cli" || self.name != "agents",
            "Kiro CLI reserves steering/AGENTS.md for user instructions"
        );
        ensure!(
            agent != "cline" || self.paths.is_empty(),
            "Cline only supports rules that apply to every conversation"
        );
        let body = &self.body;
        if self.paths.is_empty() {
            return Ok(if body.starts_with("---") {
                format!("---\n---\n\n{body}\n")
            } else {
                format!("{body}\n")
            });
        }
        let paths = encode(&self.paths)?;
        if agent == "claude" {
            Ok(format!("---\npaths: {paths}\n---\n\n{body}\n"))
        } else {
            Ok(format!(
                "---\ninclusion: fileMatch\nfileMatchPattern: {paths}\n---\n\n{body}\n"
            ))
        }
    }
}

type Homes = BTreeMap<&'static str, PathBuf>;

fn rule_homes() -> Result<Homes> {
    let home = env::home_dir().context("user home directory is unavailable")?;
    let cline = env::var_os("CLINE_DIR")
        .filter(|value| !value.is_empty())
        .map_or_else(|| home.join(".cline"), PathBuf::from);
    let claude = NativeAgent::Claude
        .home()
        .context("Claude Code profile is unavailable")?;
    Ok(BTreeMap::from([
        ("claude", claude.join("rules")),
        ("cline", cline.join("rules")),
        ("kiro-cli", home.join(".kiro/steering")),
    ]))
}

fn destination(homes: &Homes, agent: &str, name: &str) -> Result<Destination> {
    validate_name(name)?;
    let folder = homes.get(agent).context("unsupported rule agent")?;
    let folder = std::path::absolute(folder)?;
    let parent = folder.parent().context("missing agent profile folder")?;
    let parent = resolved_folder(parent)?;
    let folder = parent.join(folder.file_name().context("missing agent rule folder")?);
    let resolved = resolved_folder(&folder)?;
    ensure!(
        resolved.starts_with(&parent),
        "agent rule folder points outside the agent profile"
    );
    let path = resolved.join(format!("{name}.md"));
    Ok(Destination {
        agent: agent.to_owned(),
        name: name.to_owned(),
        path: path
            .to_str()
            .context("agent rule path is not UTF-8")?
            .to_owned(),
    })
}

fn resolved_folder(path: &Path) -> Result<PathBuf> {
    match path.canonicalize() {
        Ok(path) => {
            ensure!(path.is_dir(), "expected a directory at {}", path.display());
            Ok(path)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            match path.symlink_metadata() {
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                _ => {
                    return Err(error)
                        .with_context(|| format!("cannot resolve {}", path.display()));
                }
            }
            let parent = resolved_folder(path.parent().context("cannot resolve agent folder")?)?;
            Ok(parent.join(path.file_name().context("missing agent folder name")?))
        }
        Err(error) => Err(error).with_context(|| format!("cannot resolve {}", path.display())),
    }
}

fn file_hash(path: &Path) -> Result<Option<String>> {
    match path.symlink_metadata() {
        Ok(meta) => ensure!(
            meta.is_file(),
            "expected a regular rule file at {}",
            path.display()
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    Ok(Some(hash(&read_limited(path, MAX_RULE_FILE)?)))
}

impl Store {
    fn load(path: &Path) -> Result<Self> {
        match read_limited(path, MAX_STORE) {
            Ok(bytes) => nextjson::from_slice(&bytes).map_err(anyhow::Error::msg),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    fn persist(&self, path: &Path) -> Result<()> {
        let contents = encode(self)?;
        ensure!(
            contents.len() <= MAX_STORE,
            "rule library exceeds the storage limit"
        );
        replace(path, &contents)
    }

    fn applied_hash(&self, target: &Destination) -> Option<&str> {
        self.applied
            .iter()
            .find(|entry| entry.target == *target)
            .map(|entry| entry.hash.as_str())
    }

    fn set_applied(&mut self, target: Destination, hash: Option<String>) {
        self.applied.retain(|entry| entry.target != target);
        if let Some(hash) = hash {
            self.applied.push(Applied { target, hash });
        }
    }

    fn recover(&mut self, path: &Path, homes: &Homes) -> Result<()> {
        let Some(pending) = self.pending.take() else {
            return Ok(());
        };
        let target = &pending.target;
        let matches =
            destination(homes, &target.agent, &target.name).is_ok_and(|current| current == *target);
        if matches && file_hash(Path::new(&target.path))? == pending.after {
            self.set_applied(pending.target, pending.after);
        }
        self.persist(path)
    }

    fn check_revision(&self, name: &str, expected: Option<&str>) -> Result<()> {
        let rule = self
            .rules
            .get(name)
            .context("rule was deleted; reload the library")?;
        if let Some(expected) = expected {
            ensure!(
                revision(rule)? == expected,
                "rule changed; reload the library"
            );
        }
        Ok(())
    }

    fn inspect(&self, target: &Destination, desired: Option<&str>) -> Result<&'static str> {
        let current = file_hash(Path::new(&target.path))?;
        let owned = current
            .as_deref()
            .is_some_and(|hash| self.applied_hash(target) == Some(hash));
        Ok(match (current, desired, owned) {
            (Some(hash), Some(text), true) if hash == self::hash(text.as_bytes()) => "applied",
            (Some(_), _, false) => "conflict",
            (None, None, _) => "removed",
            _ => "pending",
        })
    }

    fn sync_file(
        &mut self,
        store: &Path,
        target: &Destination,
        desired: Option<&str>,
    ) -> Result<&'static str> {
        let path = Path::new(&target.path);
        let current = file_hash(path)?;
        let before = current.as_deref();
        let after = desired.map(|text| hash(text.as_bytes()));
        if before.is_some() && before != self.applied_hash(target) {
            if self.applied_hash(target).is_some() {
                self.set_applied(target.clone(), None);
                self.persist(store)?;
            }
            return Ok("conflict");
        }
        let status = if desired.is_some() {
            "applied"
        } else {
            "removed"
        };
        if current == after && before == self.applied_hash(target) {
            return Ok(status);
        }
        if current != after {
            self.pending = Some(Pending {
                target: target.clone(),
                after: after.clone(),
            });
            self.persist(store)?;
            let applied = (|| {
                let folder = path.parent().context("missing rule folder")?;
                ensure!(
                    resolved_folder(folder)? == folder,
                    "agent rule folder changed; reload the library"
                );
                let verify = || -> Result<()> {
                    ensure!(
                        resolved_folder(folder)? == folder && file_hash(path)? == current,
                        "rule file changed; reload the library"
                    );
                    Ok(())
                };
                if let Some(text) = desired {
                    fs::create_dir_all(folder)?;
                    verify()?;
                    let mut file = tempfile::NamedTempFile::new_in(folder)?;
                    if before.is_some() {
                        file.as_file()
                            .set_permissions(path.metadata()?.permissions())?;
                    }
                    file.write_all(text.as_bytes())?;
                    file.as_file().sync_all()?;
                    verify()?;
                    if before.is_some() {
                        file.persist(path)?;
                    } else {
                        file.persist_noclobber(path)?;
                    }
                } else {
                    verify()?;
                    fs::remove_file(path)?;
                }
                Ok::<_, anyhow::Error>(())
            })();
            self.pending = None;
            if let Err(error) = applied {
                self.persist(store)?;
                return Err(error);
            }
        }
        self.set_applied(target.clone(), after);
        self.persist(store)?;
        Ok(status)
    }
}

#[derive(Default)]
pub enum Edit {
    #[default]
    Read,
    Save {
        old: Option<String>,
        expected: Option<String>,
        rule: Rule,
    },
    Remove {
        name: String,
        expected: Option<String>,
    },
    Sync,
}

pub fn library(dir: &Path, edit: Edit) -> Result<String> {
    library_in(dir, &rule_homes()?, edit)
}

fn library_in(dir: &Path, homes: &Homes, edit: Edit) -> Result<String> {
    let folder = dir.join("rules");
    fs::create_dir_all(&folder)?;
    let lock = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(folder.join("library.lock"))?;
    lock.lock()?;
    let path = folder.join("library.json");
    let mut store = Store::load(&path)?;
    let sync = !matches!(edit, Edit::Read);
    if sync {
        store.recover(&path, homes)?;
    }
    match edit {
        Edit::Read | Edit::Sync => {}
        Edit::Save {
            old,
            expected,
            rule,
        } => {
            let rule = rule.normalize()?;
            if let Some(old) = &old {
                store.check_revision(old, expected.as_deref())?;
            }
            ensure!(
                old.as_deref() == Some(&rule.name) || !store.rules.contains_key(&rule.name),
                "a rule named {} already exists",
                rule.name
            );
            if let Some(old) = old {
                store.rules.remove(&old);
            }
            store.rules.insert(rule.name.clone(), rule);
            store.persist(&path)?;
        }
        Edit::Remove { name, expected } => {
            store.check_revision(&name, expected.as_deref())?;
            store.rules.remove(&name);
            store.persist(&path)?;
        }
    }
    let placements = placements(&mut store, &path, homes, sync)?;
    let rules = store
        .rules
        .values()
        .map(|rule| {
            Ok(RuleView {
                rule,
                revision: revision(rule)?,
            })
        })
        .collect::<Result<_>>()?;
    encode(&View { rules, placements })
}

fn placements(store: &mut Store, path: &Path, homes: &Homes, sync: bool) -> Result<Vec<Placement>> {
    let mut jobs = BTreeMap::new();
    for applied in &store.applied {
        let target = &applied.target;
        if destination(homes, &target.agent, &target.name).is_ok_and(|current| current == *target) {
            jobs.insert((target.name.clone(), target.agent.clone()), None);
        }
    }
    for rule in store.rules.values() {
        for agent in &rule.agents {
            jobs.insert((rule.name.clone(), agent.clone()), Some(rule.render(agent)));
        }
    }

    let mut jobs: Vec<_> = jobs.into_iter().collect();
    jobs.sort_by_key(|(_, desired)| desired.as_ref().is_some_and(Result::is_ok));
    let mut placements = Vec::new();
    let mut active_paths = BTreeSet::new();
    for ((name, agent), desired) in jobs {
        let mut placement = Placement {
            name,
            agent,
            path: None,
            status: "error",
            error: None,
        };
        let result = (|| {
            let target = destination(homes, &placement.agent, &placement.name)?;
            placement.path = Some(target.path.clone());
            let desired = match desired.transpose() {
                Ok(desired) => desired,
                Err(error) => {
                    if sync && store.applied_hash(&target).is_some() {
                        store.sync_file(path, &target, None)?;
                    }
                    return Err(error);
                }
            };
            if desired.is_some() {
                ensure!(
                    active_paths.insert(target.path.clone()),
                    "multiple agents resolve to the same rule file"
                );
            }
            if sync {
                store.sync_file(path, &target, desired.as_deref())
            } else {
                store.inspect(&target, desired.as_deref())
            }
        })();
        match result {
            Ok(status) => placement.status = status,
            Err(error) => {
                placement.error = Some(format!("{error:#}"));
                if store.pending.is_some() {
                    store.recover(path, homes)?;
                }
            }
        }
        placements.push(placement);
    }
    placements.sort_by(|left, right| (&left.name, &left.agent).cmp(&(&right.name, &right.agent)));
    Ok(placements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nextjson::Value;

    fn homes(root: &Path) -> Homes {
        BTreeMap::from([
            ("claude", root.join("claude/rules")),
            ("cline", root.join("cline/rules")),
            ("kiro-cli", root.join(".kiro/steering")),
        ])
    }

    fn scoped_rule(name: &str) -> Rule {
        Rule {
            name: name.into(),
            body: "---\r\nKeep diagnostics concise.\r\n".into(),
            paths: vec![" src/**/*.rs\r\nsrc/**/*.rs \n docs/*.md".into()],
            agents: AGENTS.iter().map(|agent| (*agent).into()).collect(),
        }
    }

    fn save(rule: Rule) -> Edit {
        Edit::Save {
            old: None,
            expected: None,
            rule,
        }
    }

    #[test]
    fn scopes_native_rules_without_widening_them() -> Result<()> {
        let root = tempfile::tempdir()?;
        let homes = homes(root.path());
        let data = root.path().join("data");
        let view = library_in(&data, &homes, save(scoped_rule("diagnostics")))?;
        let view: Value = nextjson::from_str(&view).map_err(anyhow::Error::msg)?;
        assert_eq!(
            view.pointer("/placements/0/status").and_then(Value::as_str),
            Some("applied")
        );
        assert_eq!(
            view.pointer("/placements/1/status").and_then(Value::as_str),
            Some("error")
        );
        assert_eq!(
            view.pointer("/placements/2/status").and_then(Value::as_str),
            Some("applied")
        );
        assert_eq!(
            fs::read_to_string(homes["claude"].join("diagnostics.md"))?,
            "---\npaths: [\"src/**/*.rs\",\"docs/*.md\"]\n---\n\n---\nKeep diagnostics concise.\n"
        );
        assert!(
            fs::read_to_string(homes["kiro-cli"].join("diagnostics.md"))?.starts_with(
                "---\ninclusion: fileMatch\nfileMatchPattern: [\"src/**/*.rs\",\"docs/*.md\"]\n"
            )
        );
        assert!(!homes["cline"].join("diagnostics.md").exists());
        assert!(library_in(&data, &homes, save(scoped_rule("../diagnostics"))).is_err());
        assert!(library_in(&data, &homes, save(scoped_rule("nul"))).is_err());
        let reserved: Value =
            nextjson::from_str(&library_in(&data, &homes, save(scoped_rule("agents")))?)
                .map_err(anyhow::Error::msg)?;
        assert_eq!(
            reserved
                .pointer("/placements/2/status")
                .and_then(Value::as_str),
            Some("error")
        );
        assert!(!homes["kiro-cli"].join("agents.md").exists());
        Ok(())
    }

    #[test]
    fn preserves_unowned_and_edited_files_through_rename_and_remove() -> Result<()> {
        let root = tempfile::tempdir()?;
        let homes = homes(root.path());
        let data = root.path().join("data");
        let cline = homes["cline"].join("shared.md");
        fs::create_dir_all(&homes["cline"])?;
        fs::write(&cline, "A rule from the user.\n")?;
        let mut rule = scoped_rule("shared");
        rule.paths.clear();
        let first = library_in(&data, &homes, save(rule.clone()))?;
        assert!(first.contains("conflict"));
        let claude = homes["claude"].join("shared.md");
        let kiro = homes["kiro-cli"].join("shared.md");
        fs::write(&claude, "A local edit.\r\n")?;

        assert!(
            library_in(
                &data,
                &homes,
                Edit::Remove {
                    name: "shared".into(),
                    expected: Some("stale".into())
                }
            )
            .is_err()
        );
        assert!(kiro.exists());
        let old_revision = revision(&rule.clone().normalize()?)?;
        rule.name = "renamed".into();
        library_in(
            &data,
            &homes,
            Edit::Save {
                old: Some("shared".into()),
                expected: Some(old_revision),
                rule,
            },
        )?;
        assert_eq!(fs::read_to_string(&claude)?, "A local edit.\r\n");
        assert_eq!(fs::read_to_string(&cline)?, "A rule from the user.\n");
        assert!(!kiro.exists());
        assert_eq!(
            fs::read_to_string(kiro.with_file_name("renamed.md"))?,
            "---\n---\n\n---\nKeep diagnostics concise.\n"
        );
        library_in(
            &data,
            &homes,
            Edit::Remove {
                name: "renamed".into(),
                expected: None,
            },
        )?;
        for folder in homes.values() {
            assert!(!folder.join("renamed.md").exists());
        }
        Ok(())
    }

    #[test]
    fn resumes_file_ownership_after_an_interrupted_write() -> Result<()> {
        let root = tempfile::tempdir()?;
        let homes = homes(root.path());
        let data = root.path().join("data");
        let mut rule = scoped_rule("retry").normalize()?;
        rule.agents.retain(|agent| agent == "claude");
        let target = destination(&homes, "claude", &rule.name)?;
        let body = rule.render("claude")?;
        let path = PathBuf::from(&target.path);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, &body)?;
        let state = data.join("rules/library.json");
        fs::create_dir_all(state.parent().unwrap())?;
        let store = Store {
            rules: BTreeMap::from([(rule.name.clone(), rule)]),
            applied: vec![],
            pending: Some(Pending {
                target,
                after: Some(hash(body.as_bytes())),
            }),
        };
        store.persist(&state)?;
        assert!(library_in(&data, &homes, Edit::Sync)?.contains("applied"));
        library_in(
            &data,
            &homes,
            Edit::Remove {
                name: "retry".into(),
                expected: None,
            },
        )?;
        assert!(!path.exists());
        Ok(())
    }
}
