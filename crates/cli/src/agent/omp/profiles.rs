use anyhow::{Context, Result, ensure};
use std::{
    env,
    ffi::OsString,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::Command,
};

fn selected(explicit: Option<&str>) -> String {
    explicit
        .map(str::to_owned)
        .or_else(|| env::var("OMP_PROFILE").ok())
        .or_else(|| env::var("PI_PROFILE").ok())
        .unwrap_or_default()
        .trim()
        .to_owned()
}

fn valid_name(name: &str) -> bool {
    let alnum = |c: u8| c.is_ascii_lowercase() || c.is_ascii_digit();
    let device = name.split('.').next().unwrap_or_default();
    let numbered = device
        .strip_prefix("com")
        .or_else(|| device.strip_prefix("lpt"));
    name.len() <= 64
        && name.bytes().next().is_some_and(alnum)
        && name.bytes().all(|c| alnum(c) || b"._-".contains(&c))
        && !name.ends_with('.')
        && name != "default"
        && !matches!(device, "aux" | "nul" | "con" | "prn")
        && !numbered.is_some_and(|n| n.len() == 1 && n.as_bytes()[0].is_ascii_digit())
}

fn config_path(command: &Command, profile: &str) -> Result<PathBuf> {
    let mut probe = Command::new(command.get_program());
    probe
        .args(command.get_args())
        .env("OMP_PROFILE", profile)
        .env_remove("PI_PROFILE")
        .env_remove("PI_CODING_AGENT_DIR");
    if !profile.is_empty() {
        probe.args(["--profile", profile]);
    }
    let out = probe
        .args(["config", "path"])
        .output()
        .context("run Oh My Pi config path")?;
    ensure!(
        out.status.success(),
        "could not resolve Oh My Pi profile {profile:?}: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
    let path = PathBuf::from(std::str::from_utf8(&out.stdout)?.trim());
    ensure!(
        path.is_absolute() && path.ends_with("agent"),
        "Oh My Pi reported an unexpected agent directory"
    );
    Ok(path)
}

pub(crate) fn list(dir: &Path, command: &Command) -> Result<()> {
    let default = config_path(command, "")?;
    let root = default
        .parent()
        .context("missing Oh My Pi config root")?
        .join("profiles");
    let active = selected(None);
    ensure!(
        active.is_empty() || active == "default" || valid_name(&active),
        "invalid Oh My Pi profile {active:?}"
    );
    let mut names = std::collections::BTreeSet::new();
    match fs::read_dir(&root) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str()
                    && valid_name(name)
                    && entry.path().is_dir()
                {
                    names.insert(name.to_owned());
                }
            }
        }
        Err(e) if e.kind() == ErrorKind::NotFound => (),
        Err(e) => return Err(e.into()),
    }
    if valid_name(&active) {
        names.insert(active.clone());
    }
    let active = if active.is_empty() {
        "default"
    } else {
        active.as_str()
    };
    for (name, path) in std::iter::once(("default", std::path::absolute(dir.join("agents/omp"))?))
        .chain(
            names
                .iter()
                .map(|name| (name.as_str(), root.join(name).join("agent"))),
        )
    {
        let state = if active == name {
            "active"
        } else {
            "available"
        };
        println!("{name}\t{state}\t{}", path.display());
    }
    Ok(())
}

pub(in crate::agent) fn agent_directory(
    dir: &Path,
    command: &mut Command,
    explicit: Option<&str>,
) -> Result<PathBuf> {
    let profile = selected(explicit);
    let home = if profile.is_empty() || profile == "default" {
        command.env("OMP_PROFILE", "");
        std::path::absolute(dir.join("agents/omp"))?
    } else {
        ensure!(valid_name(&profile), "invalid Oh My Pi profile {profile:?}");
        let path = config_path(command, &profile)?;
        ensure!(
            path.ends_with(Path::new("profiles").join(&profile).join("agent")),
            "Oh My Pi reported an unexpected directory for {profile}"
        );
        command.env("OMP_PROFILE", profile);
        path
    };
    command
        .env_remove("PI_PROFILE")
        .env("PI_CODING_AGENT_DIR", &home);
    Ok(home)
}

/// The launcher prepends --provider and --model. From there the native client's
/// bootstrap scans all launch arguments up to its own --. String flags consume
/// even a flag-looking value, while optional and extension flags do not.
pub(in crate::agent) fn native_profile(args: &[OsString]) -> Result<Option<&str>> {
    let mut selected = None;
    let mut tokens = args.iter();
    while let Some(token) = tokens.next() {
        let Some(flag) = token.to_str() else { continue };
        match flag {
            "--" => break,
            "--profile" => {
                let name = tokens
                    .next()
                    .and_then(|n| n.to_str())
                    .filter(|n| !n.is_empty() && !n.starts_with('-'))
                    .context("Oh My Pi --profile requires a name")?;
                selected = Some(name);
            }
            flag if flag.starts_with("--profile=") => {
                let name = &flag["--profile=".len()..];
                ensure!(!name.is_empty(), "Oh My Pi --profile requires a name");
                selected = Some(name);
            }
            "--cwd"
            | "--config"
            | "--add-dir"
            | "--mode"
            | "--fork"
            | "--provider"
            | "--model"
            | "--smol"
            | "--slow"
            | "--goal"
            | "--prewalk-into"
            | "--plan-yolo-into"
            | "--max-time"
            | "--warn-cost"
            | "--max-cost"
            | "--service-tier"
            | "--spinner-interval"
            | "--max-fps"
            | "--motion"
            | "--api-key"
            | "--system-prompt"
            | "--system-prompt-template"
            | "--append-system-prompt"
            | "--provider-session-id"
            | "--prompt-cache-key"
            | "--session-dir"
            | "--models"
            | "--tools"
            | "--thinking"
            | "--export"
            | "--hook"
            | "--extension"
            | "-e"
            | "--trusted-extension"
            | "--plugin-dir"
            | "--skills"
            | "--approval-mode" => {
                tokens.next();
            }
            _ => (),
        }
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn respects_native_launch_argument_boundaries() {
        for (args, selected) in [
            (vec!["--system-prompt", "--profile", "work"], None),
            (
                vec![
                    "--session",
                    "--profile=first",
                    "--plan",
                    "--profile",
                    "last",
                ],
                Some("last"),
            ),
            (
                vec!["--profile=work", "--", "--profile=other"],
                Some("work"),
            ),
        ] {
            let args: Vec<_> = args.into_iter().map(OsString::from).collect();
            assert_eq!(native_profile(&args).unwrap(), selected);
        }
        for name in [
            "work.",
            "con.device",
            "aux",
            "com0",
            "lpt9.d",
            "A",
            "../escape",
            "default",
        ] {
            assert!(!valid_name(name));
        }
    }
}
