use super::{Tree, encode};
use crate::native_settings;
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub(super) const MAX_LEDGER: usize = 1024 * 1024;

#[derive(Default, NsonDeserialize, NsonSerialize)]
pub(super) struct Store {
    pub entries: BTreeMap<String, String>,
    pending: Option<Pending>,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Pending {
    entries: BTreeMap<String, Option<String>>,
    after: String,
}

impl Store {
    pub fn load(path: &Path, settings: Option<&str>) -> Result<Self> {
        let mut store: Self = match native_settings::read(path)? {
            Some(text) => nextjson::from_str(&text).map_err(anyhow::Error::msg)?,
            None => Self::default(),
        };
        if let Some(pending) = store.pending.take()
            && settings.is_some_and(|text| hash(text) == pending.after)
        {
            store.update(pending.entries);
        }
        Ok(store)
    }

    fn update(&mut self, updates: BTreeMap<String, Option<String>>) {
        for (name, content) in updates {
            if let Some(content) = content {
                self.entries.insert(name, content);
            } else {
                self.entries.remove(&name);
            }
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        let text = encode(self)?;
        let before = native_settings::read(path)?;
        native_settings::write(path, before.as_deref(), Some(&text))
    }

    pub fn revision(&self, identity: &str, settings: Option<&str>) -> Result<String> {
        Ok(hash(&encode(&(identity, settings, &self.entries))?))
    }

    pub fn commit(
        &mut self,
        ledger: &Path,
        target: &Path,
        before: Option<&str>,
        after: Option<&str>,
        updates: BTreeMap<String, Option<String>>,
    ) -> Result<()> {
        if let Some(after) = after.filter(|after| Some(*after) != before) {
            self.pending = Some(Pending {
                entries: updates.clone(),
                after: hash(after),
            });
            self.save(ledger)?;
            native_settings::write(target, before, Some(after))?;
            self.pending = None;
        }
        if !updates.is_empty() {
            self.update(updates);
            self.save(ledger)?;
        }
        Ok(())
    }
}

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

impl<'a> Edit<'a> {
    pub(super) fn expected(&self) -> Option<&'a str> {
        match *self {
            Self::Read => None,
            Self::Set {
                project_revision, ..
            }
            | Self::Export { project_revision }
            | Self::Archive {
                project_revision, ..
            }
            | Self::Apply {
                project_revision, ..
            } => Some(project_revision),
        }
    }
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Assignments {
    kind: String,
    version: u32,
    packages: BTreeMap<String, String>,
}

pub(super) fn export(kind: &str, packages: BTreeMap<String, String>) -> Result<String> {
    encode(&Assignments {
        kind: kind.into(),
        version: 1,
        packages,
    })
}

pub(super) fn import(kind: &str, source: &str) -> Result<BTreeMap<String, String>> {
    ensure!(
        source.len() <= MAX_LEDGER,
        "project assignment document exceeds 1 MiB"
    );
    let plan: Assignments = nextjson::from_str(source.strip_prefix('\u{feff}').unwrap_or(source))
        .map_err(anyhow::Error::msg)?;
    ensure!(
        plan.kind == kind && plan.version == 1,
        "unsupported project assignment document"
    );
    Ok(plan.packages)
}

#[derive(NsonSerialize)]
pub(super) struct View<'a> {
    pub revision: String,
    pub packages: Vec<Row<'a>>,
}

#[derive(NsonSerialize)]
pub(super) struct Row<'a> {
    pub name: &'a str,
    pub version: Option<&'a str>,
    pub bundle_revision: Option<String>,
    pub assigned: bool,
    pub can_export_snapshot: bool,
    pub status: &'static str,
}

pub(super) fn hash(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

pub(super) fn path_key(path: &Path) -> Result<String> {
    let path = native_settings::conventional_path(path);
    let text = path
        .to_str()
        .context("project bundle paths must be UTF-8")?;
    Ok(if cfg!(windows) {
        text.replace('/', "\\").to_lowercase()
    } else {
        text.to_owned()
    })
}

pub(super) fn checksum(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(super) fn verified_tree(objects: &Path, content: &str) -> Result<(PathBuf, Tree)> {
    ensure!(checksum(content), "invalid saved bundle hash");
    let expected = objects.join(content);
    let folder = expected.canonicalize()?;
    ensure!(folder == expected, "saved bundle was redirected");
    let tree = Tree::read(&folder)?;
    ensure!(tree.hash() == content, "saved plugin snapshot was modified");
    Ok((folder, tree))
}

pub(super) fn stage_tree(target: &Path, tree: &Tree, content: &str) -> Result<()> {
    let parent = target
        .parent()
        .context("missing plugin staging directory")?;
    ensure!(
        crate::rules::resolved_folder(parent)? == parent,
        "plugin staging directory was redirected"
    );
    if target.try_exists()? {
        ensure!(
            Tree::read(target)?.hash() == content,
            "staged plugin snapshot was modified"
        );
        return Ok(());
    }
    fs::create_dir_all(parent)?;
    let staging = tempfile::tempdir_in(parent)?;
    let folder = staging.path().join("snapshot");
    tree.write(&folder)?;
    fs::rename(folder, target)?;
    Ok(())
}
