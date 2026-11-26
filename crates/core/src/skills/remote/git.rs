use super::{DOCUMENT, MAX_DOWNLOAD, MAX_TAR, unescape, valid_path};
use anyhow::{Context, Result, ensure};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use std::{
    fs::{self, File},
    io::{ErrorKind, Read, Seek},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use url::Url;

const TIMEOUT: Duration = Duration::from_secs(120);
const MAX_DIAGNOSTIC: u64 = 64 * 1024;

pub(super) struct Source {
    repo: Url,
    reference: String,
    pub(super) folder: Option<String>,
}

impl Source {
    pub(super) fn parse(input: &str) -> Result<Self> {
        ensure!(
            !input.starts_with('-')
                && !input.contains('\\')
                && !input.chars().any(char::is_control),
            "invalid Git repository address"
        );
        let mut url = if input.contains("://") {
            Url::parse(input)?
        } else {
            let (host, path) = input.split_once(':').context("use a Git repository URL")?;
            ensure!(
                host.contains('@') && !path.starts_with('-') && !path.is_empty(),
                "invalid Git SSH address"
            );
            let path = if path.starts_with('/') {
                path.to_owned()
            } else {
                format!("/~/{}", path.strip_prefix("~/").unwrap_or(path))
            };
            Url::parse(&format!("ssh://{host}{path}"))?
        };
        ensure!(
            matches!(url.scheme(), "https" | "http" | "ssh" | "git" | "file"),
            "use an HTTPS, HTTP, SSH, Git or local file repository URL"
        );
        ensure!(
            url.password().is_none() && (url.username().is_empty() || url.scheme() == "ssh"),
            "leave credentials out of the Git URL and use the Git credential helper or SSH agent"
        );
        ensure!(
            url.scheme() == "file"
                || (url.host_str().is_some_and(|host| !host.starts_with('-'))
                    && url
                        .username()
                        .bytes()
                        .all(|byte| { byte.is_ascii_alphanumeric() || b"-_.".contains(&byte) })),
            "invalid Git host or SSH user"
        );
        let mut folder = url.fragment().map(unescape).transpose()?;
        let mut reference = None;
        for (key, value) in url.query_pairs() {
            ensure!(
                key == "ref" && reference.is_none(),
                "use a single ref query in the Git URL"
            );
            reference = Some(value.into_owned());
        }
        if matches!(url.scheme(), "https" | "http") {
            let path = url.path().to_owned();
            if let Some((separator, (base, location))) = [
                "/-/tree/",
                "/-/blob/",
                "/src/branch/",
                "/src/tag/",
                "/src/commit/",
            ]
            .into_iter()
            .find_map(|separator| path.split_once(separator).map(|parts| (separator, parts)))
            {
                ensure!(
                    folder.is_none() && reference.is_none(),
                    "use one Git ref and skill folder"
                );
                let (git_ref, suffix) = location.split_once('/').unwrap_or((location, ""));
                reference = Some(unescape(git_ref)?);
                let suffix = unescape(suffix.trim_end_matches('/'))?;
                ensure!(
                    separator != "/-/blob/" || !suffix.is_empty(),
                    "the Git file URL must point to {DOCUMENT}"
                );
                let selected = match suffix.as_str() {
                    DOCUMENT => "",
                    path if path.ends_with("/SKILL.md") => path.trim_end_matches("/SKILL.md"),
                    path if separator != "/-/blob/" => path,
                    _ => anyhow::bail!("the Git file URL must point to {DOCUMENT}"),
                };
                folder = (!suffix.is_empty()).then(|| selected.to_owned());
                url.set_path(base);
            }
        }
        let reference = reference.unwrap_or_else(|| "HEAD".to_owned());
        ensure!(
            !reference.is_empty()
                && reference.len() <= 256
                && !reference.starts_with('-')
                && !reference.contains("..")
                && reference
                    .split('/')
                    .all(|part| !part.is_empty() && !part.starts_with('.') && !part.ends_with('.'))
                && reference
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_.+/".contains(&byte)),
            "invalid Git branch, tag or commit"
        );
        ensure!(
            folder.as_deref().is_none_or(valid_path),
            "invalid Git skill folder"
        );
        url.set_fragment(None);
        url.set_query(None);
        ensure!(
            !url.path().trim_matches('/').is_empty(),
            "the Git URL needs a repository path"
        );
        if url.scheme() == "file" {
            ensure!(
                url.host_str().is_none() && url.to_file_path().is_ok(),
                "a file Git URL must point to an absolute local repository path"
            );
        }
        Ok(Self {
            repo: url,
            reference,
            folder,
        })
    }

    pub(super) fn canonical(&self, folder: &str) -> String {
        let mut url = self.repo.clone();
        if self.reference != "HEAD" {
            url.query_pairs_mut().append_pair("ref", &self.reference);
        }
        let escaped = utf8_percent_encode(folder, NON_ALPHANUMERIC).to_string();
        url.set_fragment(Some(&escaped));
        url.into()
    }

    pub(super) fn fetch(&self) -> Result<File> {
        let temp = tempfile::tempdir()?;
        let dir = temp.path();
        let deadline = Instant::now() + TIMEOUT;
        let mut output = tempfile::tempfile()?;
        let local = |args: &[&str], output: &File, limit| {
            run(dir, self.repo.scheme(), args, output, limit, deadline)
        };
        local(
            &["init", "--quiet", "--bare", "--template="],
            &output,
            MAX_DIAGNOSTIC,
        )?;
        local(
            &[
                "fetch",
                "--quiet",
                "--no-tags",
                "--no-recurse-submodules",
                "--no-auto-maintenance",
                "--depth=1",
                "--keep",
                "--",
                self.repo.as_str(),
                &self.reference,
            ],
            &output,
            MAX_DIAGNOSTIC,
        )?;
        // Copy blob bytes even when the repository defines release filters.
        fs::create_dir_all(dir.join("info"))?;
        fs::write(
            dir.join("info/attributes"),
            "* -export-ignore -export-subst -filter -text -eol -ident -working-tree-encoding\n",
        )?;
        output.set_len(0)?;
        output.rewind()?;
        local(
            &["archive", "--format=tar", "--prefix=repo/", "FETCH_HEAD"],
            &output,
            MAX_TAR,
        )?;
        output.rewind()?;
        Ok(output)
    }
}

fn run(
    dir: &Path,
    protocol: &str,
    args: &[&str],
    output: &File,
    limit: u64,
    deadline: Instant,
) -> Result<()> {
    let mut diagnostic = tempfile::tempfile()?;
    let mut git = Command::new("git");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        git.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let mut child = git
        .current_dir(dir)
        .args([
            "-c",
            "core.hooksPath=.no-hooks",
            "-c",
            "fetch.fsckObjects=true",
            "-c",
            "http.lowSpeedLimit=1024",
            "-c",
            "http.lowSpeedTime=25",
        ])
        .args(args)
        .env("GIT_ALLOW_PROTOCOL", protocol)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .env("SSH_ASKPASS_REQUIRE", "never")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .stdin(Stdio::null())
        .stdout(output.try_clone()?)
        .stderr(diagnostic.try_clone()?)
        .spawn()
        .context("run Git for the skill source (Git must be on PATH)")?;
    let result = (|| {
        loop {
            let status = child.try_wait()?;
            ensure!(Instant::now() < deadline, "Git skill fetch timed out");
            ensure!(
                output.metadata()?.len() <= limit && diagnostic.metadata()?.len() <= MAX_DIAGNOSTIC,
                "Git skill source exceeds the output limit"
            );
            ensure!(
                pack_size(&dir.join("objects/pack"))? <= MAX_DOWNLOAD,
                "Git skill source exceeds the pack limit"
            );
            if let Some(status) = status {
                if !status.success() {
                    diagnostic.rewind()?;
                    let mut bytes = Vec::new();
                    diagnostic.take(MAX_DIAGNOSTIC).read_to_end(&mut bytes)?;
                    let message = String::from_utf8_lossy(&bytes);
                    let message = message
                        .lines()
                        .rev()
                        .find(|line| !line.trim().is_empty())
                        .unwrap_or("");
                    let message: String = message
                        .chars()
                        .filter(|character| !character.is_control())
                        .take(400)
                        .collect();
                    anyhow::bail!("git {} failed ({status}): {message}", args[0]);
                }
                return Ok(());
            }
            thread::sleep(Duration::from_millis(40));
        }
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn pack_size(dir: &Path) -> Result<u64> {
    let files = match fs::read_dir(dir) {
        Ok(files) => files,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = 0u64;
    for file in files {
        match file?.metadata() {
            Ok(metadata) => bytes = bytes.saturating_add(metadata.len()),
            Err(error) if error.kind() == ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_clone_and_forge_links_without_treating_input_as_git_options() -> Result<()> {
        let nested = Source::parse(
            "https://gitlab.example/group/sub/repo/-/blob/feature%2Fdocs/tools/check/SKILL.md",
        )?;
        assert_eq!(
            nested.repo.as_str(),
            "https://gitlab.example/group/sub/repo"
        );
        assert_eq!(nested.reference, "feature/docs");
        assert_eq!(nested.folder.as_deref(), Some("tools/check"));
        let gitea = Source::parse("https://gitea.example/team/repo/src/tag/v1/tools/check")?;
        assert_eq!(gitea.reference, "v1");
        assert_eq!(gitea.folder.as_deref(), Some("tools/check"));
        let ssh = Source::parse("git@forge.example:team/repo.git?ref=main#tools/check")?;
        assert_eq!(ssh.repo.as_str(), "ssh://git@forge.example/~/team/repo.git");
        assert_eq!(ssh.reference, "main");
        assert_eq!(
            Source::parse(&ssh.canonical("tools/check"))?
                .folder
                .as_deref(),
            Some("tools/check")
        );
        for input in [
            "ext::git-upload-pack anything",
            "ssh://-option/repo.git",
            "https://token@forge.example/team/repo",
            "https://forge.example/team/repo?ref=--upload-pack",
            "https://forge.example/team/repo?ref=refs%2Fheads%2Fx%3Arefs%2Ftags%2Fx",
            "https://forge.example/team/repo#tools/%2e%2e",
            "https://forge.example/team/repo/-/blob/main/tools",
        ] {
            assert!(Source::parse(input).is_err(), "accepted {input}");
        }
        Ok(())
    }
}
