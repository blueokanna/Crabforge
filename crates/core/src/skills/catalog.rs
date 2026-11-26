use super::{document_details, tree};
use anyhow::{Context, Result, ensure};
use nextjson::NsonSerialize;
use std::{
    fs::{self, File},
    io::{ErrorKind, Read},
    path::Path,
};

const DOCUMENT_PREFIX: u64 = 2 * super::MAX_FRONT_MATTER as u64 + 16;
const MAX_METADATA: u64 = 8 * 1024 * 1024;
const MAX_VISITED: usize = 8192;

#[derive(NsonSerialize)]
pub(super) struct Candidate {
    pub(super) name: String,
    description: String,
    pub(super) source: String,
    folder: String,
}

#[derive(Default)]
pub(super) struct Catalog {
    candidates: Vec<Candidate>,
    bytes: u64,
    visited: usize,
}

impl Catalog {
    pub(super) fn inspect(
        &mut self,
        document: impl Read,
        length: u64,
        source: String,
        folder: String,
    ) -> Result<()> {
        let prefix = length.min(DOCUMENT_PREFIX);
        self.bytes += prefix;
        ensure!(
            self.bytes <= MAX_METADATA,
            "skill catalog exceeds 8 MiB of metadata"
        );
        let mut body = Vec::with_capacity(prefix as usize);
        document.take(prefix).read_to_end(&mut body)?;
        ensure!(
            body.len() as u64 == prefix,
            "skill document ended before its declared size"
        );
        let text = match std::str::from_utf8(&body) {
            Ok(text) => text,
            Err(error) if length > prefix && error.error_len().is_none() => {
                std::str::from_utf8(&body[..error.valid_up_to()])?
            }
            Err(_) => return Ok(()),
        };
        if let Ok((name, description)) = document_details(text) {
            ensure!(
                self.candidates.len() < tree::MAX_ENTRIES,
                "too many available skills"
            );
            self.candidates.push(Candidate {
                name,
                description,
                source,
                folder,
            });
        }
        Ok(())
    }

    pub(super) fn finish(self) -> Vec<Candidate> {
        self.candidates
    }

    fn walk(&mut self, root: &Path, folder: &str, depth: usize) -> Result<()> {
        let path = root.join(folder);
        let document = path.join(tree::DOCUMENT);
        match fs::symlink_metadata(&document) {
            Ok(metadata) if metadata.is_file() && !tree::is_link(&metadata) => {
                let source = path.to_str().context("skill source path is not UTF-8")?;
                self.inspect(
                    File::open(&document)?,
                    metadata.len(),
                    source.to_owned(),
                    folder.to_owned(),
                )?;
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("inspect {}", document.display()));
            }
        }
        if depth == tree::MAX_DEPTH {
            return Ok(());
        }
        for entry in fs::read_dir(&path)? {
            self.visited += 1;
            ensure!(
                self.visited <= MAX_VISITED,
                "skill folder scan exceeds {MAX_VISITED} entries"
            );
            let entry = entry?;
            let filename = entry.file_name();
            let Some(name) = filename
                .to_str()
                .filter(|name| tree::windows_filename(name) && !name.starts_with('.'))
            else {
                continue;
            };
            if !entry.file_type()?.is_dir() || tree::is_link(&fs::symlink_metadata(entry.path())?) {
                continue;
            }
            let child = if folder.is_empty() {
                name.to_owned()
            } else {
                format!("{folder}/{name}")
            };
            self.walk(root, &child, depth + 1)?;
        }
        Ok(())
    }
}

pub(super) fn folder(path: &Path) -> Result<Vec<Candidate>> {
    let metadata = fs::symlink_metadata(path).context("open the skill collection folder")?;
    ensure!(
        metadata.is_dir() && !tree::is_link(&metadata),
        "expected a regular skill collection directory"
    );
    let path = path.canonicalize()?;
    let mut catalog = Catalog::default();
    catalog.walk(&path, "", 0)?;
    Ok(catalog.finish())
}
