use super::{Agent, Target};
use crate::{config::read_limited, native_settings};
use anyhow::{Context, Result, ensure};
use nextjson::Value;
use std::{
    collections::BTreeMap,
    env, fs,
    io::{ErrorKind, Read, Seek},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use yaml_edit::{Mapping, Sequence, YamlFile, YamlNode};

const FILE: &str = "crabforge-hooks.json";
const PATCH: &str = "cordis.patch.yml";
const ID: &str = "crabforge-hooks";
const PLUGIN: &str = "@deepseek-ai/dsh-hooks-claude-code";
const PROTOCOL: &str = "@deepseek-ai/dsh-hook-protocol";

pub(super) fn discover(home: &Path, targets: &mut BTreeMap<Target, &'static str>) -> Result<()> {
    let profiles = home.join("profiles");
    let entries = match fs::read_dir(&profiles) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).context("read DeepSeek Harness profiles"),
    };
    for entry in entries {
        let entry = entry?;
        let profile = entry.path();
        if entry.file_type()?.is_dir() && profile.join(PATCH).is_file() {
            let path = native_settings::path(&profile.join(FILE))?;
            if let Some(path) = path.to_str() {
                targets.insert(
                    Target {
                        agent: Agent::Dsh,
                        path: path.into(),
                    },
                    "{}\n",
                );
            }
        }
    }
    Ok(())
}

fn field(mapping: &Mapping, key: &str) -> Option<String> {
    mapping
        .get(key)
        .and_then(|node| node.as_scalar().map(|scalar| scalar.as_string()))
}

fn desired(path: &str) -> YamlNode {
    let config = Mapping::new();
    config.set("configPath", path);
    let plugin = Mapping::new();
    plugin.set("id", ID);
    plugin.set("name", PLUGIN);
    plugin.set("config", config);
    let inserts = Sequence::new();
    inserts.push(plugin);
    let row = Mapping::new();
    row.set("insert", inserts);
    YamlNode::Mapping(row)
}

fn update(source: &str, path: &str, active: bool) -> Result<Option<String>> {
    let content = source.strip_prefix('\u{feff}').unwrap_or(source);
    let document: YamlFile = content
        .parse()
        .context("invalid DeepSeek Harness profile patch")?;
    ensure!(
        document.documents().count() == 1,
        "expected one DeepSeek Harness profile patch document"
    );
    let sequence = document
        .document()
        .and_then(|doc| doc.as_sequence())
        .context("expected a DeepSeek Harness profile patch list")?;
    let row = desired(path);
    let mut owned = None;
    for (index, entry) in sequence.values().enumerate() {
        let Some(mapping) = entry.as_mapping() else {
            continue;
        };
        ensure!(
            field(mapping, "id").as_deref() != Some(ID),
            "DeepSeek Harness hook profile entry is overridden"
        );
        let Some(insert) = mapping.get("insert") else {
            continue;
        };
        let list = insert
            .as_sequence()
            .context("expected a DeepSeek Harness insert list")?;
        for node in list.values() {
            let Some(plugin) = node.as_mapping() else {
                continue;
            };
            if field(plugin, "id").as_deref() == Some(ID) {
                ensure!(
                    owned.is_none() && entry.yaml_eq(&row),
                    "DeepSeek Harness hook profile entry was edited"
                );
                owned = Some(index);
            } else if active {
                ensure!(
                    field(plugin, "name").as_deref() != Some(PLUGIN),
                    "DeepSeek Harness already loads this hook plugin in the profile"
                );
            }
        }
    }
    match (active, owned) {
        (true, None) => sequence.push(row),
        (false, Some(index)) => {
            sequence.remove(index);
        }
        _ => return Ok(None),
    }
    let mut text = document.to_string();
    if content.contains("\r\n") && !content.replace("\r\n", "").contains('\n') {
        text = text.replace("\r\n", "\n").replace('\n', "\r\n");
    }
    if source.starts_with('\u{feff}') {
        text.insert(0, '\u{feff}');
    }
    Ok(Some(text))
}

pub(super) fn validate_config(source: &str) -> Result<()> {
    let value: Value = nextjson::from_str(source).map_err(anyhow::Error::msg)?;
    ensure!(
        !source.starts_with('\u{feff}') && value.is_object(),
        "DeepSeek Harness hook configuration must be a JSON object without a BOM"
    );
    Ok(())
}

fn run(profile: &Path, home: &Path, args: &[&str], timeout: Duration) -> Result<String> {
    let path = env::var_os("PATH").context("PATH is unavailable")?;
    let names = if cfg!(windows) {
        &["dsh.exe", "dsh.cmd"][..]
    } else {
        &["dsh"][..]
    };
    let mut executable = None;
    for folder in env::split_paths(&path).filter(|folder| !folder.as_os_str().is_empty()) {
        if let Some(file) = names
            .iter()
            .map(|name| folder.join(name))
            .find(|file| file.is_file())
        {
            executable = Some(std::path::absolute(file)?);
            break;
        }
    }
    let executable = executable.context("DeepSeek Harness (dsh) must be on PATH")?;
    let mut output = tempfile::tempfile()?;
    let mut diagnostic = tempfile::tempfile()?;
    let mut command = Command::new(executable);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let mut child = command
        .args(args)
        .current_dir(loader_path(profile))
        .env("DSH_HOME", loader_path(home))
        .stdin(Stdio::null())
        .stdout(output.try_clone()?)
        .stderr(diagnostic.try_clone()?)
        .spawn()
        .context("run dsh from PATH for DeepSeek Harness hooks")?;
    let deadline = Instant::now() + timeout;
    let result = (|| loop {
        if let Some(status) = child.try_wait()? {
            if !status.success() {
                let length = diagnostic.metadata()?.len();
                diagnostic.seek(std::io::SeekFrom::Start(length.saturating_sub(2048)))?;
                let mut bytes = Vec::new();
                diagnostic.by_ref().take(2048).read_to_end(&mut bytes)?;
                let text = String::from_utf8_lossy(&bytes);
                let detail = text
                    .lines()
                    .rev()
                    .find(|line| !line.trim().is_empty())
                    .unwrap_or("");
                anyhow::bail!("dsh {} failed ({status}) {detail}", args[0]);
            }
            output.rewind()?;
            let mut text = String::new();
            output.by_ref().take(4096).read_to_string(&mut text)?;
            return Ok(text);
        }
        ensure!(Instant::now() < deadline, "dsh {} timed out", args[0]);
        thread::sleep(Duration::from_millis(40));
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn loader_path(path: &Path) -> PathBuf {
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

fn ensure_plugin(profile: &Path) -> Result<()> {
    let home = profile
        .parent()
        .and_then(Path::parent)
        .context("missing DeepSeek Harness home")?;
    let name = profile
        .file_name()
        .and_then(|name| name.to_str())
        .context("invalid DeepSeek Harness profile name")?;
    let version = run(profile, home, &["--version"], Duration::from_secs(10))?;
    let version = version.trim();
    ensure!(
        version.starts_with(|first: char| first.is_ascii_digit())
            && version.len() <= 80
            && version
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b".-+".contains(&byte)),
        "invalid DeepSeek Harness version"
    );
    let installed = || {
        [PLUGIN, PROTOCOL].iter().all(|package| {
            let name = profile
                .join("node_modules")
                .join(package)
                .join("package.json");
            read_limited(&name, 64 * 1024)
                .ok()
                .and_then(|bytes| nextjson::from_slice::<Value>(&bytes).ok())
                .and_then(|data| {
                    data.as_object()?
                        .get("version")?
                        .as_str()
                        .map(str::to_owned)
                })
                .is_some_and(|actual| actual == version)
        })
    };
    if !installed() {
        run(
            profile,
            home,
            &[
                "plugin",
                "--profile",
                name,
                "add",
                "--save-exact",
                &format!("{PROTOCOL}@{version}"),
                &format!("{PLUGIN}@{version}"),
            ],
            Duration::from_secs(180),
        )
        .context("install the matching official hook plugin and protocol (pnpm must be on PATH)")?;
        ensure!(
            installed(),
            "DeepSeek Harness hook packages must match dsh {version}"
        );
    }
    Ok(())
}

pub(super) struct Patch {
    path: PathBuf,
    before: String,
    after: String,
}

impl Patch {
    pub(super) fn apply(self) -> Result<()> {
        native_settings::write(&self.path, Some(&self.before), Some(&self.after))
    }
}

pub(super) fn plan(target: &Target, active: bool, install: bool) -> Result<Option<Patch>> {
    let profile = Path::new(&target.path)
        .parent()
        .context("missing DeepSeek Harness profile folder")?;
    let path = profile.join(PATCH);
    let before =
        native_settings::read(&path)?.context("DeepSeek Harness profile patch is unavailable")?;
    // Cordis resolves the configured file against the launch directory. A Windows
    // canonical path may use the Win32 verbatim prefix, which Node normalizes differently.
    let config = loader_path(Path::new(&target.path));
    let config = config
        .to_str()
        .context("invalid DeepSeek Harness config path")?;
    let after = update(&before, config, active)?;
    if active && install {
        ensure_plugin(profile)?;
    }
    Ok(after.map(|after| Patch {
        path,
        before,
        after,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_entry_preserves_comments_and_refuses_foreign_plugin_configuration() -> Result<()> {
        let source =
            "\u{feff}# user's profile\r\n- id: unrelated\r\n  config: {enabled: false} # keep\r\n";
        let path = "C:/project's profile/crabforge-hooks.json";
        let installed = update(source, path, true)?.unwrap();
        assert!(installed.starts_with(source));
        assert!(update(&installed, path, true)?.is_none());
        assert_eq!(update(&installed, path, false)?.as_deref(), Some(source));
        let foreign = installed.replace("id: crabforge-hooks", "id: another-hooks");
        assert!(update(&foreign, path, true).is_err());
        assert!(update(&installed, "C:/changed.json", false).is_err());
        let blank = "# default\n[]\n";
        let installed = update(blank, path, true)?.unwrap();
        assert_eq!(update(&installed, path, false)?.as_deref(), Some(blank));
        Ok(())
    }
}
