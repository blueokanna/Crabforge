use crate::config::read_limited;
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, Metadata},
    io::Write,
    path::{Path, PathBuf},
};

pub(super) const MAX_BYTES: usize = 16 * 1024 * 1024;
pub(super) const MAX_ENTRIES: usize = 1024;
pub(super) const MAX_DEPTH: usize = 16;
pub(super) const DOCUMENT: &str = "SKILL.md";

enum Entry {
    Directory,
    File { bytes: Vec<u8>, executable: bool },
}

#[derive(Default)]
pub(super) struct Tree(BTreeMap<PathBuf, Entry>);

pub(super) fn is_link(metadata: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // Directory junctions are reparse points, but are not all symlinks.
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.is_symlink()
    }
}

fn executable(_metadata: &Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        _metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        false
    }
}

pub(super) fn windows_filename(name: &str) -> bool {
    let base = name.split('.').next().unwrap_or_default().trim_end();
    let upper = base.to_ascii_uppercase();
    !name.is_empty()
        && !name.ends_with(['.', ' '])
        && !name.contains(['\\', '/', ':', '<', '>', '"', '|', '?', '*'])
        && !name.chars().any(char::is_control)
        && !matches!(
            upper.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
        )
        && !upper
            .strip_prefix("COM")
            .or_else(|| upper.strip_prefix("LPT"))
            .is_some_and(|suffix| {
                suffix.len() == 1 && suffix.as_bytes()[0].is_ascii_digit()
                    || matches!(suffix, "¹" | "²" | "³")
            })
}

impl Tree {
    pub(super) fn read(root: &Path) -> Result<Self> {
        let metadata = fs::symlink_metadata(root)?;
        ensure!(
            metadata.is_dir() && !is_link(&metadata),
            "expected a regular skill directory"
        );
        let mut tree = Self::default();
        let mut remaining = MAX_BYTES;
        tree.scan(root, Path::new(""), 0, &mut remaining)?;
        Ok(tree)
    }

    fn scan(
        &mut self,
        root: &Path,
        relative: &Path,
        depth: usize,
        remaining: &mut usize,
    ) -> Result<()> {
        ensure!(depth <= MAX_DEPTH, "skill path depth exceeds {MAX_DEPTH}");
        let mut names = std::collections::BTreeSet::new();
        for entry in fs::read_dir(root.join(relative))? {
            let entry = entry?;
            ensure!(
                self.0.len() < MAX_ENTRIES,
                "skill exceeds {MAX_ENTRIES} files and folders"
            );
            let name = entry.file_name();
            let name = name.to_str().context("skill filenames must be UTF-8")?;
            ensure!(
                windows_filename(name) && names.insert(name.to_uppercase()),
                "skill filename {name:?} is ambiguous or unsupported on Windows"
            );
            let path = relative.join(name);
            let metadata = fs::symlink_metadata(entry.path())?;
            ensure!(!is_link(&metadata), "linked skill entry {}", path.display());
            if metadata.is_dir() {
                self.0.insert(path.clone(), Entry::Directory);
                self.scan(root, &path, depth + 1, remaining)?;
            } else {
                ensure!(
                    metadata.is_file(),
                    "expected a regular file at {}",
                    path.display()
                );
                let bytes = read_limited(&entry.path(), *remaining).with_context(|| {
                    format!("read skill file {} within the size limit", path.display())
                })?;
                *remaining -= bytes.len();
                self.0.insert(
                    path,
                    Entry::File {
                        bytes,
                        executable: executable(&metadata),
                    },
                );
            }
        }
        Ok(())
    }

    pub(super) fn document(&self) -> Result<&str> {
        match self.0.get(Path::new(DOCUMENT)) {
            Some(Entry::File { bytes, .. }) => Ok(std::str::from_utf8(bytes)?),
            _ => anyhow::bail!("a skill folder must contain {DOCUMENT}"),
        }
    }

    pub(super) fn hash(&self) -> String {
        let mut hash = Sha256::new();
        for (path, entry) in &self.0 {
            for component in path.components() {
                let text = component.as_os_str().to_str().expect("validated filename");
                hash.update((text.len() as u64).to_be_bytes());
                hash.update(text);
            }
            hash.update(0u64.to_be_bytes());
            match entry {
                Entry::Directory => hash.update([0]),
                Entry::File { bytes, executable } => {
                    hash.update([1, u8::from(*executable)]);
                    hash.update((bytes.len() as u64).to_be_bytes());
                    hash.update(bytes);
                }
            }
        }
        hex::encode(hash.finalize())
    }

    pub(super) fn write(&self, root: &Path) -> Result<()> {
        fs::create_dir(root)?;
        for (path, entry) in &self.0 {
            let path = root.join(path);
            match entry {
                Entry::Directory => fs::create_dir(path)?,
                Entry::File { bytes, executable } => {
                    let mut file = File::create_new(path)?;
                    file.write_all(bytes)?;
                    #[cfg(unix)]
                    if *executable {
                        use std::os::unix::fs::PermissionsExt;
                        let mut permissions = file.metadata()?.permissions();
                        permissions.set_mode(permissions.mode() | 0o111);
                        file.set_permissions(permissions)?;
                    }
                    #[cfg(not(unix))]
                    let _ = executable;
                    file.sync_all()?;
                }
            }
        }
        Ok(())
    }
}
