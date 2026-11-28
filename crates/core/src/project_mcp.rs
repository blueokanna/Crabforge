use crate::{config::read_limited, rules::resolved_folder};
use anyhow::{Context, Result, ensure};
use nextjson::{
    NsonSerialize, Value,
    formats::{Format, Json5},
    json,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, Item, Table};

pub(crate) mod jsonc;
pub mod library;

const MAX_CONFIG: usize = 1024 * 1024;
const MAX_SERVER: usize = 64 * 1024;

#[derive(Clone, Copy)]
struct Target {
    file: &'static str,
    alternate: Option<&'static str>,
    container: &'static [&'static str],
    template: &'static str,
}

const STDIO: &str = "{\n  \"command\": \"path-to-server\",\n  \"args\": []\n}";
const TYPED_STDIO: &str =
    "{\n  \"type\": \"stdio\",\n  \"command\": \"path-to-server\",\n  \"args\": []\n}";
const OPENCODE_STDIO: &str = "{\n  \"type\": \"local\",\n  \"command\": [\"path-to-server\"]\n}";

const fn json_target(file: &'static str) -> Target {
    Target {
        file,
        alternate: None,
        container: &["mcpServers"],
        template: STDIO,
    }
}

fn target(agent: &str) -> Result<Target> {
    Ok(match agent {
        "claude" => Target {
            template: TYPED_STDIO,
            ..json_target(".mcp.json")
        },
        "codebuddy" => Target {
            alternate: Some("mcp.json"),
            template: TYPED_STDIO,
            ..json_target(".mcp.json")
        },
        "codex" => Target {
            file: ".codex/config.toml",
            alternate: None,
            container: &["mcp_servers"],
            template: "command = \"path-to-server\"\nargs = []\n",
        },
        "cursor" => json_target(".cursor/mcp.json"),
        "opencode" => Target {
            file: "opencode.json",
            alternate: Some("opencode.jsonc"),
            container: &["mcp"],
            template: "{\n  \"type\": \"local\",\n  \"command\": [\"path-to-server\"],\n  \"enabled\": true\n}",
        },
        "zcode" => Target {
            container: &["mcp", "servers"],
            template: TYPED_STDIO,
            ..json_target(".zcode/config.json")
        },
        "pi" => json_target(".pi/mcp.json"),
        "qoder" | "qoder-cn" => json_target(".qoder/settings.json"),
        "droid" => Target {
            template: TYPED_STDIO,
            ..json_target(".factory/mcp.json")
        },
        "trae-cn" => json_target(".trae/mcp.json"),
        "agy" | "antigravity" => json_target(".agents/mcp_config.json"),
        _ => anyhow::bail!("this agent has no supported project MCP file"),
    })
}

fn optional(path: &Path) -> Result<Option<String>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
    };
    ensure!(
        metadata.is_file(),
        "expected a regular file at {}",
        path.display()
    );
    Ok(Some(String::from_utf8(read_limited(path, MAX_CONFIG)?)?))
}

fn config_path(root: &Path, target: Target) -> Result<PathBuf> {
    let primary = root.join(target.file);
    let path = match target.alternate {
        Some(alternate)
            if primary
                .symlink_metadata()
                .is_err_and(|err| err.kind() == ErrorKind::NotFound) =>
        {
            let alternate = root.join(alternate);
            if alternate
                .symlink_metadata()
                .is_err_and(|err| err.kind() == ErrorKind::NotFound)
            {
                primary
            } else {
                alternate
            }
        }
        _ => primary,
    };
    let parent = resolved_folder(path.parent().context("missing project MCP parent")?)?;
    ensure!(
        parent.starts_with(root),
        "MCP configuration must stay inside the project"
    );
    Ok(parent.join(path.file_name().context("missing project MCP filename")?))
}

fn identity(path: &Path) -> String {
    hex::encode(Sha256::digest(path.as_os_str().as_encoded_bytes()))
}

fn revision(path: &Path, content: Option<&str>) -> String {
    let mut hash = Sha256::new();
    hash.update(identity(path));
    hash.update([u8::from(content.is_some())]);
    hash.update(content.unwrap_or_default());
    hex::encode(hash.finalize())
}

#[derive(NsonSerialize)]
struct View {
    project: String,
    path: String,
    syntax: &'static str,
    revision: String,
    template: &'static str,
    servers: BTreeMap<String, String>,
}

pub enum Edit {
    Read,
    Save {
        name: String,
        source: String,
        expected: Option<String>,
        replace: bool,
    },
    Remove {
        name: String,
        expected: String,
    },
}

fn name(value: &str) -> Result<&str> {
    ensure!(
        !value.is_empty()
            && value.len() <= 64
            && value
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c)),
        "use 1–64 letters, digits, underscores or hyphens for the server name"
    );
    Ok(value)
}

fn parts(text: &str) -> (&str, &str) {
    text.strip_prefix('\u{feff}')
        .map_or(("", text), |rest| ("\u{feff}", rest))
}

fn json_document(text: &str) -> Result<Value> {
    jsonc::parse(text, &[])?;
    Json5
        .decode(parts(text).1.as_bytes())
        .map_err(anyhow::Error::msg)
}

fn json_path(value: &Value, target: Target) -> &'static [&'static str] {
    if target.file == "opencode.json"
        && value
            .pointer("/mcp/servers")
            .and_then(Value::as_object)
            .is_some_and(|servers| {
                ["type", "command", "url"]
                    .into_iter()
                    .all(|key| servers.get(key).is_none_or(Value::is_object))
            })
    {
        &["mcp", "servers"]
    } else {
        target.container
    }
}

fn json_entries(text: &str, value: &Value, target: Target) -> Result<BTreeMap<String, String>> {
    let entries_at = |path| -> Result<_> {
        Ok(jsonc::parse(text, path)?
            .into_iter()
            .flat_map(|object| object.entries(text))
            .map(|(name, source)| (name, source.to_owned())))
    };
    let keys = json_path(value, target);
    let mut entries = entries_at(target.container)?.collect::<BTreeMap<_, _>>();
    if keys != target.container {
        entries.remove("servers");
        entries.extend(entries_at(keys)?);
    }
    Ok(entries)
}

fn server_path(value: &Value, target: Target, name: &str) -> &'static [&'static str] {
    let native = json_path(value, target);
    if native != target.container
        && name != "servers"
        && value.get("mcp").and_then(|mcp| mcp.get(name)).is_some()
        && value
            .pointer("/mcp/servers")
            .and_then(|servers| servers.get(name))
            .is_none()
    {
        return target.container;
    }
    native
}

fn wrap_json(path: &[&str], value: Value) -> Value {
    path.iter()
        .rev()
        .fold(value, |value, key| json!({(*key): value}))
}

fn json_edit(text: &str, path: &[&str], name: &str, source: Option<&str>) -> Result<String> {
    let replacement = source.map(json_document).transpose()?;
    if let Some(replacement) = &replacement {
        ensure!(
            replacement.is_object(),
            "an MCP server must be a JSON object"
        );
    }
    if let Some(object) = jsonc::parse(text, path)? {
        return object.edit(text, name, replacement.as_ref());
    }
    let Some(replacement) = replacement else {
        return Ok(text.to_owned());
    };
    let entry = json!({(name): replacement});
    for i in (0..path.len()).rev() {
        if let Some(parent) = jsonc::parse(text, &path[..i])? {
            return parent.edit(text, path[i], Some(&wrap_json(&path[i + 1..], entry)));
        }
    }
    anyhow::bail!("missing MCP configuration object")
}

fn toml_document(text: &str) -> Result<DocumentMut> {
    Ok(parts(text).1.parse()?)
}

fn toml_entries(document: &DocumentMut) -> Result<BTreeMap<String, String>> {
    let Some(servers) = document.as_table().get("mcp_servers") else {
        return Ok(BTreeMap::new());
    };
    servers
        .as_table_like()
        .context("mcp_servers must be a table")?
        .iter()
        .map(|(key, value)| {
            let value = value
                .as_table_like()
                .with_context(|| format!("MCP server {key:?} must be a table"))?;
            let mut entry = DocumentMut::new();
            for (key, value) in value.iter() {
                entry.insert(key, value.clone());
            }
            Ok((key.to_owned(), entry.to_string()))
        })
        .collect()
}

fn toml_edit(
    mut document: DocumentMut,
    text: &str,
    name: &str,
    source: Option<&str>,
) -> Result<String> {
    if let Some(source) = source {
        let table = toml_document(source)?.into_table();
        ensure!(!table.is_empty(), "an MCP server table cannot be empty");
        let servers = document
            .as_table_mut()
            .entry("mcp_servers")
            .or_insert_with(|| {
                let mut table = Table::new();
                table.set_implicit(true);
                Item::Table(table)
            });
        servers
            .as_table_like_mut()
            .context("mcp_servers must be a table")?
            .insert(name, Item::Table(table));
    } else if let Some(servers) = document
        .get_mut("mcp_servers")
        .and_then(Item::as_table_like_mut)
    {
        servers.remove(name);
    }
    let after = document.to_string();
    let after = if text.contains("\r\n") && !text.replace("\r\n", "").contains('\n') {
        after.replace("\r\n", "\n").replace('\n', "\r\n")
    } else {
        after
    };
    Ok(format!("{}{after}", parts(text).0))
}

pub fn edit(dir: &Path, project: &Path, agent: &str, edit: Edit) -> Result<String> {
    let mut view = edit_native(dir, project, agent, edit, |_| Ok(()))?;
    if view.syntax == "json" {
        for source in view.servers.values_mut() {
            let value: Value = Json5
                .decode(source.as_bytes())
                .map_err(anyhow::Error::msg)?;
            *source = nextjson::to_string_pretty(&value).map_err(anyhow::Error::msg)?;
        }
    }
    nextjson::to_string(&view).map_err(anyhow::Error::msg)
}

fn edit_native(
    dir: &Path,
    project: &Path,
    agent: &str,
    edit: Edit,
    before_write: impl FnOnce(&View) -> Result<()>,
) -> Result<View> {
    let target = target(agent)?;
    let root = project.canonicalize().context("open the project folder")?;
    ensure!(root.is_dir(), "expected a project folder");
    let path = config_path(&root, target)?;
    let change = !matches!(&edit, Edit::Read);
    let _lock = if change {
        let lock_dir = dir.join("project-mcp");
        fs::create_dir_all(&lock_dir)?;
        let lock = File::options()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_dir.join(format!("{}.lock", identity(&path))))?;
        lock.lock()?;
        ensure!(
            config_path(&root, target)? == path,
            "project MCP configuration moved; reload it"
        );
        Some(lock)
    } else {
        None
    };
    let before = optional(&path)?;
    let version = revision(&path, before.as_deref());
    let toml = target.file.ends_with(".toml");
    let text = before.as_deref().unwrap_or(if toml { "" } else { "{}\n" });
    let settings = if toml {
        None
    } else {
        Some(json_document(text)?)
    };
    let keys = settings
        .as_ref()
        .map_or(target.container, |value| json_path(value, target));
    let document = if toml {
        Some(toml_document(text)?)
    } else {
        None
    };
    let servers = match &document {
        Some(document) => toml_entries(document)?,
        None => json_entries(text, settings.as_ref().expect("JSON settings"), target)?,
    };
    let update = match edit {
        Edit::Read => None,
        Edit::Save {
            name: server,
            source,
            expected,
            replace,
        } => {
            name(&server)?;
            ensure!(
                target.file != "opencode.json" || keys.len() > 1 || server != "servers" || replace,
                "OpenCode reserves mcp.servers for its server map"
            );
            ensure!(
                source.len() <= MAX_SERVER,
                "MCP server configuration exceeds {MAX_SERVER} bytes"
            );
            if let Some(expected) = expected.as_deref() {
                ensure!(
                    expected == version,
                    "project MCP settings changed; reload them"
                );
            }
            if replace {
                ensure!(
                    expected.is_some(),
                    "a project MCP revision is required to replace a server"
                );
                ensure!(servers.contains_key(&server), "MCP server no longer exists");
            } else {
                ensure!(
                    !servers.contains_key(&server),
                    "server already exists; select it to edit its settings"
                );
            }
            Some((server, Some(source)))
        }
        Edit::Remove {
            name: server,
            expected,
        } => {
            ensure!(
                expected == version,
                "project MCP settings changed; reload them"
            );
            ensure!(servers.contains_key(&server), "MCP server no longer exists");
            Some((server, None))
        }
    };
    let mut view = View {
        project: root.display().to_string(),
        path: path.display().to_string(),
        syntax: if toml { "toml" } else { "json" },
        revision: version,
        template: if target.file == "opencode.json" && keys.len() > 1 {
            OPENCODE_STDIO
        } else {
            target.template
        },
        servers,
    };
    if let Some((server, source)) = update {
        let after = match document {
            Some(document) => toml_edit(document, text, &server, source.as_deref())?,
            None => {
                let path = server_path(settings.as_ref().expect("JSON settings"), target, &server);
                json_edit(text, path, &server, source.as_deref())?
            }
        };
        ensure!(
            after.len() <= MAX_CONFIG,
            "project MCP file exceeds {MAX_CONFIG} bytes"
        );
        view.servers = if toml {
            toml_entries(&toml_document(&after)?)?
        } else {
            json_entries(&after, &json_document(&after)?, target)?
        };
        let parent = path.parent().context("missing MCP config parent")?;
        fs::create_dir_all(parent)?;
        let mut staged = tempfile::NamedTempFile::new_in(parent)?;
        if before.is_some() {
            staged
                .as_file()
                .set_permissions(path.metadata()?.permissions())?;
        }
        staged.write_all(after.as_bytes())?;
        staged.as_file().sync_all()?;
        ensure!(
            config_path(&root, target)? == path && optional(&path)? == before,
            "project MCP settings changed; reload them"
        );
        view.revision = revision(&path, Some(&after));
        before_write(&view)?;
        ensure!(
            config_path(&root, target)? == path && optional(&path)? == before,
            "project MCP settings changed; reload them"
        );
        if before.is_some() {
            staged.persist(&path)?;
        } else {
            staged.persist_noclobber(&path)?;
        }
    }
    Ok(view)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(dir: &Path, project: &Path, agent: &str, edit: Edit) -> Result<Value> {
        nextjson::from_str(&super::edit(dir, project, agent, edit)?).map_err(anyhow::Error::msg)
    }

    fn version(view: &Value) -> String {
        view["revision"].as_str().unwrap().to_owned()
    }

    fn save(source: &str, expected: Option<String>, replace: bool) -> Edit {
        Edit::Save {
            name: "diagnostics".into(),
            source: source.into(),
            expected,
            replace,
        }
    }

    #[test]
    fn preserves_jsonc_bom_comments_and_siblings_across_crud() -> Result<()> {
        let root = tempfile::tempdir()?;
        let dir = root.path().join("data");
        let path = root.path().join("opencode.jsonc");
        let original = "\u{feff}{\r\n  // Keep this description\r\n  \"mcp\": {\"servers\": {\"existing\": {\"type\": \"local\", \"command\": [\"other\"]},}, \"legacy\": {\"type\": \"local\", \"command\": [\"legacy\"]}, \"other\": true},\r\n  \"model\": \"local/testing\",\r\n}\r\n";
        fs::write(&path, original)?;
        let first = report(&dir, root.path(), "opencode", Edit::Read)?;
        let entry = r#"{"type":"local","command":["server","--stdio"]}"#;
        let added = report(
            &dir,
            root.path(),
            "opencode",
            save(entry, Some(version(&first)), false),
        )?;
        let file = fs::read_to_string(&path)?;
        assert!(file.starts_with('\u{feff}'));
        assert!(file.contains("// Keep this description\r\n") && file.contains(r#""other": true"#));
        assert!(!file.replace("\r\n", "").contains('\n'));
        assert!(added.pointer("/servers/existing").is_some());
        assert!(added.pointer("/servers/legacy").is_some());
        assert!(added.pointer("/servers/diagnostics").is_some());
        assert!(
            report(
                &dir,
                root.path(),
                "opencode",
                save(entry, Some(version(&first)), true)
            )
            .is_err()
        );
        assert!(
            report(
                &dir,
                root.path(),
                "opencode",
                save(entry, Some(version(&added)), false)
            )
            .is_err()
        );
        let changed = report(
            &dir,
            root.path(),
            "opencode",
            save(
                r#"{"type":"remote","url":"http://127.0.0.1:8765/mcp"}"#,
                Some(version(&added)),
                true,
            ),
        )?;
        let removed = report(
            &dir,
            root.path(),
            "opencode",
            Edit::Remove {
                name: "diagnostics".into(),
                expected: version(&changed),
            },
        )?;
        assert!(removed.pointer("/servers/diagnostics").is_none());
        assert!(removed.pointer("/servers/existing").is_some());
        let after_legacy = report(
            &dir,
            root.path(),
            "opencode",
            Edit::Remove {
                name: "legacy".into(),
                expected: version(&removed),
            },
        )?;
        assert!(after_legacy.pointer("/servers/legacy").is_none());
        assert!(
            json_document(&fs::read_to_string(&path)?)?
                .pointer("/mcp/legacy")
                .is_none()
        );
        assert!(!root.path().join("opencode.json").exists());
        Ok(())
    }

    #[test]
    fn creates_project_json_and_preserves_other_codex_tables() -> Result<()> {
        let root = tempfile::tempdir()?;
        let dir = root.path().join("data");
        let pi = report(
            &dir,
            root.path(),
            "pi",
            save(r#"{"command":"example","args":[]}"#, None, false),
        )?;
        assert!(pi.pointer("/servers/diagnostics").is_some());
        let zcode = report(
            &dir,
            root.path(),
            "zcode",
            save(r#"{"type":"stdio","command":"example"}"#, None, false),
        )?;
        assert!(zcode.pointer("/servers/diagnostics").is_some());
        let codex = root.path().join(".codex");
        fs::create_dir(&codex)?;
        let path = codex.join("config.toml");
        let original = "# keep the model\r\nmodel = \"test\"\r\n\r\n[mcp_servers.other]\r\nurl = \"https://example.com/mcp\"\r\n";
        fs::write(&path, original)?;
        let changed = report(
            &dir,
            root.path(),
            "codex",
            save(
                "command = \"example\"\nargs = [\"subcommand\"]\n",
                None,
                false,
            ),
        )?;
        assert!(changed.pointer("/servers/diagnostics").is_some());
        let text = fs::read_to_string(&path)?;
        assert!(text.contains(original));
        assert!(!text.replace("\r\n", "").contains('\n'));
        let removed = report(
            &dir,
            root.path(),
            "codex",
            Edit::Remove {
                name: "diagnostics".into(),
                expected: version(&changed),
            },
        )?;
        assert!(removed.pointer("/servers/other").is_some());
        assert!(removed.pointer("/servers/diagnostics").is_none());
        Ok(())
    }
}
