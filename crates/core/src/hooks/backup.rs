use super::{Target, encode, hash};
use crate::native_settings;
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufRead, BufReader, ErrorKind, Read, Write},
    path::Path,
};

const MAX_HEADER: usize = 256 * 1024;
const MAX_SOURCE: usize = 1024 * 1024;

#[derive(NsonDeserialize, NsonSerialize)]
struct Snapshot {
    target: Target,
    event: String,
    before: String,
    after: String,
}

#[derive(NsonSerialize)]
pub(super) struct Backup {
    id: String,
    target: Target,
    event: String,
    status: &'static str,
}

fn is_id(id: &str) -> bool {
    id.len() == 64 && id.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn open(folder: &Path, id: &str) -> Result<(Snapshot, BufReader<File>)> {
    ensure!(is_id(id), "invalid hook backup ID");
    let path = folder.join(id);
    ensure!(
        fs::symlink_metadata(&path)?.is_file(),
        "expected a regular hook backup file"
    );
    let mut reader = BufReader::new(File::open(path)?);
    let mut header = String::new();
    (&mut reader)
        .take(MAX_HEADER as u64)
        .read_line(&mut header)?;
    ensure!(header.ends_with('\n'), "invalid hook backup header");
    header.pop();
    ensure!(hash(&header) == id, "hook backup metadata changed");
    let snapshot = nextjson::from_str(&header).map_err(anyhow::Error::msg)?;
    Ok((snapshot, reader))
}

pub(super) fn save(
    folder: &Path,
    target: &Target,
    event: String,
    before: &str,
    after: &str,
) -> Result<()> {
    let header = encode(&Snapshot {
        target: target.clone(),
        event,
        before: hash(before),
        after: hash(after),
    })?;
    ensure!(
        header.len() < MAX_HEADER,
        "hook backup metadata is too large"
    );
    fs::create_dir_all(folder)?;
    let mut staged = tempfile::NamedTempFile::new_in(folder)?;
    writeln!(staged, "{header}")?;
    staged.write_all(before.as_bytes())?;
    staged.as_file().sync_all()?;
    staged.persist(folder.join(hash(&header)))?;
    Ok(())
}

pub(super) fn list(folder: &Path, targets: &BTreeMap<Target, &str>) -> Result<Vec<Backup>> {
    let paths = match fs::read_dir(folder) {
        Ok(paths) => paths,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let mut current = BTreeMap::new();
    let mut backups = Vec::new();
    for path in paths {
        let path = path?;
        let Ok(id) = path.file_name().into_string() else {
            continue;
        };
        let Ok((snapshot, _)) = open(folder, &id) else {
            continue;
        };
        let target = snapshot.target;
        let status = if targets.contains_key(&target) {
            let digest = current.entry(target.clone()).or_insert_with(|| {
                native_settings::read(Path::new(&target.path))
                    .ok()
                    .flatten()
                    .map(|text| hash(&text))
            });
            match digest.as_deref() {
                Some(value) if value == snapshot.after => "ready",
                Some(value) if value == snapshot.before => "restored",
                Some(_) => "conflict",
                None => "unavailable",
            }
        } else {
            "unavailable"
        };
        backups.push(Backup {
            id,
            target,
            event: snapshot.event,
            status,
        });
    }
    backups
        .sort_unstable_by(|a, b| (&a.target, &a.event, &a.id).cmp(&(&b.target, &b.event, &b.id)));
    Ok(backups)
}

pub(super) fn restore(folder: &Path, targets: &BTreeMap<Target, &str>, id: &str) -> Result<()> {
    let (snapshot, reader) = open(folder, id)?;
    ensure!(
        targets.contains_key(&snapshot.target),
        "select the original hook profile"
    );
    let mut before = String::new();
    reader
        .take(MAX_SOURCE as u64 + 1)
        .read_to_string(&mut before)?;
    ensure!(
        before.len() <= MAX_SOURCE && hash(&before) == snapshot.before,
        "hook backup content changed"
    );
    let path = Path::new(&snapshot.target.path);
    let current = native_settings::read(path)?.context("native hook file was removed")?;
    let digest = hash(&current);
    if digest == snapshot.before {
        return Ok(());
    }
    ensure!(
        digest == snapshot.after,
        "native hooks changed since this backup"
    );
    native_settings::write(path, Some(&current), Some(&before))
}
