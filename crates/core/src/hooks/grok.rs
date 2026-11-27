use super::{Agent, Target};
use anyhow::{Context, Result};
use std::{collections::BTreeMap, fs, io::ErrorKind, path::Path};

pub(super) const FILE: &str = "crabforge.json";

pub(super) fn is_managed(target: &Target) -> bool {
    target.agent == Agent::Grok
        && Path::new(&target.path)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == FILE || (cfg!(windows) && name.eq_ignore_ascii_case(FILE)))
}

pub(super) fn is_external(target: &Target) -> bool {
    target.agent == Agent::Grok && !is_managed(target)
}

pub(super) fn discover_targets(targets: &mut BTreeMap<Target, &'static str>) -> Result<()> {
    let Some(managed) = targets.keys().find(|target| is_managed(target)) else {
        return Ok(());
    };
    let folder = Path::new(&managed.path)
        .parent()
        .context("missing Grok hooks folder")?;
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).context("read Grok hooks folder"),
    };
    let mut discovered = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
            && entry.file_type()?.is_file()
            && let Some(path) = path.to_str()
        {
            let target = Target {
                agent: Agent::Grok,
                path: path.into(),
            };
            if is_external(&target) {
                discovered.push(target);
            }
        }
    }
    targets.extend(discovered.into_iter().map(|target| (target, "{}\n")));
    Ok(())
}
