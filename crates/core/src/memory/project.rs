use anyhow::{Context, Result, ensure};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use url::Url;

pub fn key(folder: &Path) -> Result<String> {
    let folder = folder
        .canonicalize()
        .context("cannot open project folder")?;
    ensure!(folder.is_dir(), "expected a project folder");
    for root in folder.ancestors() {
        let Some(git_dir) = git_dir(root)? else {
            continue;
        };
        let common = match read(&git_dir.join("commondir"))? {
            Some(path) if !path.trim().is_empty() => git_dir.join(path.trim()).canonicalize()?,
            _ => git_dir,
        };
        if let Some(config) = read(&common.join("config"))?
            && let Some(remote) = remote_key(&config)
        {
            return Ok(remote);
        }
        let main = if common.ends_with(".git") {
            common.parent().unwrap_or(root)
        } else {
            root
        };
        return folder_key(main);
    }
    folder_key(&folder)
}

fn read(path: &Path) -> Result<Option<String>> {
    match crate::config::read_limited(path, 256 * 1024) {
        Ok(bytes) => Ok(Some(String::from_utf8(bytes)?)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("cannot read {}", path.display())),
    }
}

fn git_dir(root: &Path) -> Result<Option<PathBuf>> {
    let marker = root.join(".git");
    match fs::metadata(&marker) {
        Ok(metadata) if metadata.is_dir() => Ok(Some(marker.canonicalize()?)),
        Ok(metadata) if metadata.is_file() => {
            let instruction = read(&marker)?.context("Git worktree marker disappeared")?;
            let path = instruction
                .trim()
                .strip_prefix("gitdir:")
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .context("invalid Git worktree marker")?;
            Ok(Some(root.join(path).canonicalize()?))
        }
        Ok(_) => anyhow::bail!("unsupported Git checkout marker"),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn folder_key(folder: &Path) -> Result<String> {
    let path = folder.to_str().context("project path must be Unicode")?;
    #[cfg(windows)]
    let path = path.strip_prefix(r"\\?\UNC\").map_or_else(
        || {
            path.strip_prefix(r"\\?\")
                .unwrap_or(path)
                .replace('\\', "/")
        },
        |rest| format!("//{}", rest.replace('\\', "/")),
    );
    Ok(format!("dir:{path}"))
}

fn remote_key(config: &str) -> Option<String> {
    let mut remote = None;
    let mut first = None;
    for line in config.trim_start_matches('\u{feff}').lines().map(str::trim) {
        if let Some(header) = line.strip_prefix('[') {
            let section = header.split_once(']').map(|(name, _)| name.trim());
            remote = section.and_then(|name| {
                let (kind, instance) = name.split_once(char::is_whitespace)?;
                kind.eq_ignore_ascii_case("remote")
                    .then(|| instance.trim().strip_prefix('"')?.strip_suffix('"'))
                    .flatten()
            });
            continue;
        }
        let Some(remote) = remote else { continue };
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        if !name.trim().eq_ignore_ascii_case("url") {
            continue;
        }
        let value = value.trim();
        let value = if let Some(quoted) = value.strip_prefix('"') {
            quoted.split_once('"').map(|(url, _)| url).unwrap_or("")
        } else {
            value.split(['#', ';']).next().unwrap_or_default().trim()
        };
        if value.is_empty() {
            continue;
        }
        if remote.eq_ignore_ascii_case("origin") {
            return from_remote(value);
        }
        first.get_or_insert(value);
    }
    first.and_then(from_remote)
}

fn from_remote(value: &str) -> Option<String> {
    let url = if value.contains("://") {
        Url::parse(value).ok()?
    } else {
        // Git's scp form uses a colon after the host, or after a bracketed IPv6 address.
        let offset = value.find(']').map_or(0, |index| index + 1);
        let (host, path) = value[offset..].split_once(':')?;
        let host = &value[..offset + host.len()];
        if host.len() <= 1 || host.contains(['/', '\\']) || path.contains('\\') {
            return None;
        }
        Url::parse(&format!("ssh://{host}/{}", path.trim_start_matches('/'))).ok()?
    };
    if url.scheme() == "file" {
        return None;
    }
    let host = url.host_str()?.to_ascii_lowercase();
    let path = url.path().trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    (!path.is_empty()).then(|| format!("{host}/{path}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_key_follows_origin_and_the_main_checkout() -> Result<()> {
        let home = tempfile::tempdir()?;
        let root = home.path().canonicalize()?;
        let main = root.join("main");
        let checkout = root.join("topic checkout");
        let nested = checkout.join("src");
        let git = main.join(".git");
        fs::create_dir_all(git.join("worktrees/topic"))?;
        fs::create_dir_all(&nested)?;
        fs::write(
            checkout.join(".git"),
            "gitdir: ../main/.git/worktrees/topic\n",
        )?;
        fs::write(git.join("worktrees/topic/commondir"), "../..\n")?;
        fs::write(
            git.join("config"),
            concat!(
                "[remote \"upstream\"]\nurl = git@backup.example:group/repo.git\n",
                "[REMOTE \"origin\"] # preferred\nURL = \"https://user:token@GIT.example:443/team/Repo.git\" # review\n",
            ),
        )?;
        assert_eq!(key(&main)?, "git.example/team/Repo");
        assert_eq!(key(&nested)?, "git.example/team/Repo");
        fs::write(
            git.join("config"),
            "[remote \"main\"]\nurl = git@Git.Example:team/Repo.git\n",
        )?;
        assert_eq!(key(&nested)?, "git.example/team/Repo");
        fs::write(
            git.join("config"),
            "[remote \"origin\"]\nurl = file:///somewhere/repo.git\n[remote \"backup\"]\nurl = git@backup.example:team/Repo.git\n",
        )?;
        assert_eq!(key(&main)?, key(&nested)?);
        assert!(key(&main)?.starts_with("dir:"));
        assert!(key(&root)?.starts_with("dir:"));
        Ok(())
    }
}
