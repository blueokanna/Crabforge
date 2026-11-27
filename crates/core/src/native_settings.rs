use crate::{config::read_limited, rules::resolved_folder};
use anyhow::{Context, Result, ensure};
use std::{
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

const MAX_CONFIG: usize = 1024 * 1024;

pub(crate) fn lock(dir: &Path) -> Result<File> {
    fs::create_dir_all(dir)?;
    let guard = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join("native-settings.lock"))?;
    guard.lock()?;
    Ok(guard)
}

pub(crate) fn path(file: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(file)?;
    let parent = resolved_folder(
        absolute
            .parent()
            .context("missing native settings folder")?,
    )?;
    Ok(parent.join(
        absolute
            .file_name()
            .context("missing native settings filename")?,
    ))
}

pub(crate) fn conventional_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    if let Some(path) = path.to_str() {
        if let Some(path) = path.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{path}"));
        }
        if let Some(path) = path.strip_prefix(r"\\?\") {
            return PathBuf::from(path);
        }
    }
    path.to_path_buf()
}

pub(crate) fn read(path: &Path) -> Result<Option<String>> {
    let parent = path.parent().context("missing native settings folder")?;
    ensure!(
        resolved_folder(parent)? == parent,
        "native settings folder moved"
    );
    match fs::symlink_metadata(path) {
        Ok(metadata) => ensure!(
            metadata.is_file(),
            "expected a regular native settings file"
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    Ok(Some(
        String::from_utf8(read_limited(path, MAX_CONFIG)?)
            .map_err(|_| anyhow::anyhow!("native settings must be UTF-8"))?,
    ))
}

pub(crate) fn write(path: &Path, before: Option<&str>, after: Option<&str>) -> Result<()> {
    let parent = path.parent().context("missing native settings folder")?;
    ensure!(
        resolved_folder(parent)? == parent,
        "native settings folder moved"
    );
    fs::create_dir_all(parent)?;
    let staged = if let Some(after) = after {
        ensure!(after.len() <= MAX_CONFIG, "native settings exceed 1 MiB");
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        if before.is_some() {
            file.as_file()
                .set_permissions(path.metadata()?.permissions())?;
        }
        file.write_all(after.as_bytes())?;
        file.as_file().sync_all()?;
        Some(file)
    } else {
        None
    };
    ensure!(
        read(path)?.as_deref() == before,
        "native settings changed; reload them"
    );
    match staged {
        Some(file) if before.is_some() => {
            file.persist(path)?;
        }
        Some(file) => {
            file.persist_noclobber(path)?;
        }
        None => fs::remove_file(path)?,
    }
    Ok(())
}
