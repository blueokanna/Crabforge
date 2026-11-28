use super::{Edit as NativeEdit, View as NativeView, edit_native, json_document, name, target};
use crate::{config::read_limited, project_instructions::replace};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::ErrorKind,
    path::{Path, PathBuf},
};
use toml_edit::{Array, DocumentMut, InlineTable, value};
use url::Url;

mod backup;
pub use backup::import_backup;

const MAX_LIBRARY: usize = 4 * 1024 * 1024;

#[derive(PartialEq, Eq, NsonDeserialize, NsonSerialize)]
struct Config {
    transport: String,
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<BTreeMap<String, String>>,
    url: Option<String>,
    headers: Option<BTreeMap<String, String>>,
}

impl Config {
    fn validate(&self) -> Result<()> {
        match self.transport.as_str() {
            "stdio" => ensure!(
                self.command
                    .as_deref()
                    .is_some_and(|text| !text.trim().is_empty())
                    && self.url.is_none()
                    && self.headers.is_none(),
                "a stdio server needs a command and accepts args and env"
            ),
            "http" | "sse" => {
                let url = Url::parse(self.url.as_deref().context("the server needs a URL")?)?;
                ensure!(
                    matches!(url.scheme(), "http" | "https")
                        && url.has_host()
                        && url.username().is_empty()
                        && url.password().is_none()
                        && self.command.is_none()
                        && self.args.is_none()
                        && self.env.is_none(),
                    "a remote server needs an HTTP or HTTPS URL and accepts headers"
                );
            }
            _ => anyhow::bail!("MCP transport must be stdio, http or sse"),
        }
        Ok(())
    }

    fn native(&self, agent: &str) -> Result<String> {
        let stdio = self.transport == "stdio";
        let command = self.command.as_deref().unwrap_or_default();
        let args = self.args.as_deref().unwrap_or_default();
        let url = self.url.as_deref().unwrap_or_default();
        let sse = self.transport == "sse";
        ensure!(
            !sse || !matches!(agent, "agy" | "antigravity" | "codex" | "pi" | "trae-cn"),
            "{agent} project MCP requires stdio or streamable HTTP"
        );
        if agent == "codex" {
            let mut document = DocumentMut::new();
            let (key, fields) = if stdio {
                document["command"] = value(command);
                document["args"] = value(args.iter().map(String::as_str).collect::<Array>());
                ("env", &self.env)
            } else {
                document["url"] = value(url);
                ("http_headers", &self.headers)
            };
            if let Some(fields) = fields {
                let mut table = InlineTable::new();
                for (key, val) in fields {
                    table.insert(key, val.as_str().into());
                }
                document[key] = value(table);
            }
            return Ok(document.to_string());
        }
        let typed = matches!(agent, "claude" | "codebuddy" | "droid" | "zcode");
        let entry = if stdio && agent == "opencode" {
            let command = std::iter::once(command)
                .chain(args.iter().map(String::as_str))
                .collect::<Vec<_>>();
            let mut entry = nextjson::Map::from_iter([
                ("type", json!("local")),
                ("command", json!(command)),
                ("enabled", json!(true)),
            ]);
            if let Some(env) = &self.env {
                entry.insert("environment".into(), json!(env));
            }
            entry
        } else if stdio {
            let mut entry =
                nextjson::Map::from_iter([("command", json!(command)), ("args", json!(args))]);
            if typed {
                entry.insert("type".into(), json!("stdio"));
            }
            if let Some(env) = &self.env {
                entry.insert("env".into(), json!(env));
            }
            entry
        } else {
            let key = match agent {
                "agy" | "antigravity" => "serverUrl",
                _ => "url",
            };
            let mut entry = nextjson::Map::from_iter([(key, json!(url))]);
            if agent == "opencode" {
                entry.insert("type".into(), json!("remote"));
                entry.insert("enabled".into(), json!(true));
            } else if typed || matches!(agent, "qoder" | "qoder-cn") {
                entry.insert("type".into(), json!(self.transport));
            }
            if let Some(headers) = &self.headers {
                entry.insert("headers".into(), json!(headers));
            }
            entry
        };
        encode(&nextjson::Value::Object(entry))
    }
}

pub(crate) fn bundle_config(server_name: &str, source: &nextjson::Value) -> Result<String> {
    use nextjson::Value;
    name(server_name)?;
    let object = source
        .as_object()
        .context("expected an MCP server object")?;
    ensure!(
        object
            .keys()
            .all(|key| matches!(key, "type" | "command" | "args" | "env" | "url" | "headers")),
        "the MCP server uses unsupported fields"
    );
    let transport = match object.get("type") {
        Some(value) => value.as_str().context("expected an MCP transport string")?,
        None if object.get("url").is_some() => "http",
        None => "stdio",
    };
    let mut converted = object.clone();
    converted.remove("type");
    converted.insert("transport".into(), json!(transport));
    let config: Config =
        nextjson::from_value(Value::Object(converted)).map_err(anyhow::Error::msg)?;
    config.validate()?;
    encode(&config)
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, NsonDeserialize, NsonSerialize)]
struct Assignment {
    project: String,
    agent: String,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Server {
    config: Config,
    assignments: BTreeSet<Assignment>,
}

#[derive(Clone, PartialEq, Eq, NsonDeserialize, NsonSerialize)]
struct Placement {
    name: String,
    assignment: Assignment,
    path: String,
    content: String,
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Pending {
    name: String,
    assignment: Assignment,
    path: String,
    revision: String,
    content: Option<String>,
}

#[derive(Default, NsonDeserialize, NsonSerialize)]
struct Store {
    servers: BTreeMap<String, Server>,
    applied: Vec<Placement>,
    pending: Option<Pending>,
}

#[derive(NsonSerialize)]
struct Entry<'a> {
    name: &'a str,
    revision: String,
    server: &'a Server,
}

#[derive(NsonSerialize)]
struct Status {
    name: String,
    project: String,
    agent: String,
    path: Option<String>,
    status: &'static str,
    detail: Option<String>,
}

#[derive(NsonSerialize)]
struct Report<'a> {
    servers: Vec<Entry<'a>>,
    placements: Vec<Status>,
}

struct Reconciliation<'a> {
    dir: &'a Path,
    path: &'a Path,
    sync: bool,
}

pub enum Edit {
    Read,
    Save {
        name: String,
        source: String,
        expected: Option<String>,
    },
    Assign {
        name: String,
        project: PathBuf,
        agent: String,
        enabled: bool,
        expected: String,
    },
    Remove {
        name: String,
        expected: String,
    },
    Sync,
}

fn encode(data: &impl NsonSerialize) -> Result<String> {
    nextjson::to_string(data).map_err(anyhow::Error::msg)
}

fn hash(data: &str) -> String {
    hex::encode(Sha256::digest(data))
}

fn revision(server: &Server) -> Result<String> {
    Ok(hash(&encode(server)?))
}

fn placement_view(dir: &Path, assignment: &Assignment) -> Result<NativeView> {
    edit_native(
        dir,
        Path::new(&assignment.project),
        &assignment.agent,
        NativeEdit::Read,
        |_| Ok(()),
    )
}

impl Store {
    fn load(path: &Path) -> Result<Self> {
        match read_limited(path, MAX_LIBRARY) {
            Ok(bytes) => {
                let mut store: Self = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
                for server in store.servers.values_mut() {
                    server
                        .assignments
                        .retain(|assignment| assignment.agent != "gemini");
                }
                store
                    .applied
                    .retain(|record| record.assignment.agent != "gemini");
                if store
                    .pending
                    .as_ref()
                    .is_some_and(|pending| pending.assignment.agent == "gemini")
                {
                    store.pending = None;
                }
                Ok(store)
            }
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    fn contents(&self) -> Result<String> {
        let text = encode(self)?;
        ensure!(
            text.len() <= MAX_LIBRARY,
            "MCP library exceeds its size limit"
        );
        Ok(text)
    }

    fn save(&self, path: &Path) -> Result<()> {
        replace(path, &self.contents()?)
    }

    fn current(&self, name: &str, expected: &str) -> Result<&Server> {
        let server = self
            .servers
            .get(name)
            .context("MCP library entry was removed")?;
        ensure!(
            revision(server)? == expected,
            "MCP library entry changed; reload it"
        );
        Ok(server)
    }

    fn finish(&mut self, path: &Path, committed: bool) -> Result<()> {
        let pending = self
            .pending
            .take()
            .context("missing MCP library operation")?;
        let original = self.applied.clone();
        if committed {
            self.applied
                .retain(|record| record.name != pending.name || record.path != pending.path);
            if let Some(content) = &pending.content {
                self.applied.push(Placement {
                    name: pending.name.clone(),
                    assignment: pending.assignment.clone(),
                    path: pending.path.clone(),
                    content: content.clone(),
                });
            }
        }
        if let Err(error) = self.save(path) {
            self.pending = Some(pending);
            self.applied = original;
            return Err(error);
        }
        Ok(())
    }

    fn recover(&mut self, dir: &Path, path: &Path) -> Result<()> {
        if let Some(pending) = &self.pending {
            let view = placement_view(dir, &pending.assignment)
                .with_context(|| format!("recover MCP library operation at {}", pending.path))?;
            let committed = view.path == pending.path && view.revision == pending.revision;
            self.finish(path, committed)?;
        }
        Ok(())
    }

    fn publish(
        &mut self,
        workspace: &Reconciliation<'_>,
        name: &str,
        assignment: &Assignment,
        previous: NativeView,
        source: Option<String>,
    ) -> Result<&'static str> {
        let Reconciliation { dir, path, sync } = *workspace;
        let current = previous.servers.get(name);
        let owned = self
            .applied
            .iter()
            .find(|record| record.name == name && record.path == previous.path);
        if let Some(current) = current {
            if owned.is_none_or(|record| record.content != hash(current)) {
                return Ok("conflict");
            }
        } else if source.is_none() {
            if sync && owned.is_some() {
                self.applied
                    .retain(|record| record.name != name || record.path != previous.path);
                self.save(path)?;
            }
            return Ok("removed");
        }
        let remove = source.is_none();
        let edit = if let Some(source) = source {
            if let Some(current) = current {
                let same = if previous.syntax == "toml" {
                    current.trim() == source.trim()
                } else {
                    encode(&json_document(current)?)? == source
                };
                if same {
                    return Ok("applied");
                }
            }
            NativeEdit::Save {
                name: name.into(),
                source,
                expected: Some(previous.revision),
                replace: current.is_some(),
            }
        } else {
            NativeEdit::Remove {
                name: name.into(),
                expected: previous.revision,
            }
        };
        if !sync || self.pending.is_some() {
            return Ok("pending");
        }
        let result = edit_native(
            dir,
            Path::new(&assignment.project),
            &assignment.agent,
            edit,
            |after| {
                let content = after.servers.get(name).map(|text| hash(text));
                ensure!(
                    content.is_none() == remove,
                    "another MCP entry with that name exists"
                );
                self.pending = Some(Pending {
                    name: name.into(),
                    assignment: assignment.clone(),
                    path: after.path.clone(),
                    revision: after.revision.clone(),
                    content,
                });
                self.save(path)
            },
        );
        if result.is_ok() {
            self.finish(path, true)?;
        } else {
            self.recover(dir, path)?;
        }
        result?;
        Ok(if remove { "removed" } else { "applied" })
    }
}

fn lock_library(library: &Path) -> Result<File> {
    fs::create_dir_all(library)?;
    let lock = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(library.join("library.lock"))?;
    lock.lock()?;
    Ok(lock)
}

pub fn edit(dir: &Path, edit: Edit) -> Result<String> {
    let library = dir.join("project-mcp");
    let _lock = lock_library(&library)?;
    let path = library.join("library.json");
    let mut store = Store::load(&path)?;
    let sync = !matches!(edit, Edit::Read);
    if sync {
        store.recover(dir, &path)?;
    }
    match edit {
        Edit::Read | Edit::Sync => {}
        Edit::Save {
            name: key,
            source,
            expected,
        } => {
            name(&key)?;
            ensure!(
                source.len() <= super::MAX_SERVER,
                "MCP library entry exceeds its size limit"
            );
            let config: Config = nextjson::from_str(&source).map_err(anyhow::Error::msg)?;
            config.validate()?;
            let assignments = if let Some(expected) = expected {
                store.current(&key, &expected)?.assignments.clone()
            } else {
                ensure!(
                    !store.servers.contains_key(&key),
                    "MCP server already exists in the library"
                );
                BTreeSet::new()
            };
            for assignment in &assignments {
                config.native(&assignment.agent)?;
            }
            store.servers.insert(
                key,
                Server {
                    config,
                    assignments,
                },
            );
            store.save(&path)?;
        }
        Edit::Assign {
            name,
            project,
            agent,
            enabled,
            expected,
        } => {
            target(&agent)?;
            let server = store.current(&name, &expected)?;
            let project = project.canonicalize().context("open the project folder")?;
            ensure!(project.is_dir(), "expected a project folder");
            let assignment = Assignment {
                project: project
                    .into_os_string()
                    .into_string()
                    .ok()
                    .context("project path is not UTF-8")?,
                agent,
            };
            let mut assignments = server.assignments.clone();
            if enabled {
                server.config.native(&assignment.agent)?;
                assignments.insert(assignment);
            } else {
                assignments.remove(&assignment);
            }
            store
                .servers
                .get_mut(&name)
                .expect("validated server")
                .assignments = assignments;
            store.save(&path)?;
        }
        Edit::Remove { name, expected } => {
            store.current(&name, &expected)?;
            store.servers.remove(&name);
            store.save(&path)?;
        }
    }

    let mut placements = Vec::new();
    let workspace = Reconciliation {
        dir,
        path: &path,
        sync,
    };
    let mut desired = BTreeSet::new();
    for (name, server) in &store.servers {
        for assignment in &server.assignments {
            desired.insert((name.clone(), assignment.clone()));
        }
    }
    let mut kept = BTreeSet::new();
    for (name, assignment) in desired {
        let result = placement_view(dir, &assignment);
        let mut status = Status::new(&name, &assignment);
        let applied = (|| {
            let view = result?;
            status.path = Some(view.path.clone());
            kept.insert((name.clone(), view.path.clone()));
            let source = store.servers[&name].config.native(&assignment.agent)?;
            store.publish(&workspace, &name, &assignment, view, Some(source))
        })();
        status.record(applied);
        placements.push(status);
    }
    for old in store.applied.clone() {
        if kept.contains(&(old.name.clone(), old.path.clone())) {
            continue;
        }
        let mut status = Status::new(&old.name, &old.assignment);
        status.path = Some(old.path.clone());
        status.record((|| {
            let view = placement_view(dir, &old.assignment)?;
            ensure!(
                view.path == old.path,
                "project MCP configuration moved; restore the original path to sync it"
            );
            store.publish(&workspace, &old.name, &old.assignment, view, None)
        })());
        placements.push(status);
    }
    placements.sort_by(|a, b| {
        (&a.name, &a.project, &a.agent, &a.path).cmp(&(&b.name, &b.project, &b.agent, &b.path))
    });
    let servers = store
        .servers
        .iter()
        .map(|(name, server)| {
            Ok(Entry {
                name,
                revision: revision(server)?,
                server,
            })
        })
        .collect::<Result<_>>()?;
    encode(&Report {
        servers,
        placements,
    })
}

impl Status {
    fn new(name: &str, assignment: &Assignment) -> Self {
        Self {
            name: name.into(),
            project: assignment.project.clone(),
            agent: assignment.agent.clone(),
            path: None,
            status: "error",
            detail: None,
        }
    }

    fn record(&mut self, result: Result<&'static str>) {
        match result {
            Ok(status) => self.status = status,
            Err(error) => self.detail = Some(format!("{error:#}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nextjson::Value;

    const SERVER: &str =
        r#"{"transport":"stdio","command":"diagnostics","args":["--measure"],"env":{"UNIT":"ms"}}"#;

    fn report(dir: &Path, change: Edit) -> Result<Value> {
        nextjson::from_str(&edit(dir, change)?).map_err(anyhow::Error::msg)
    }

    fn server_revision(view: &Value) -> String {
        view.pointer("/servers/0/revision")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    }

    #[test]
    fn synchronizes_shared_transport_and_leaves_the_users_entries() -> Result<()> {
        let root = tempfile::tempdir()?;
        let dir = root.path().join("data");
        let project = root.path();
        fs::create_dir(project.join(".agents"))?;
        let antigravity = project.join(".agents/mcp_config.json");
        fs::write(
            &antigravity,
            "{\n// keep this comment\n\"context\": {},\n\"mcpServers\": {\"existing\": {\"command\": \"mine\"}}\n}\n",
        )?;
        let saved = report(
            &dir,
            Edit::Save {
                name: "diagnostics".into(),
                source: SERVER.into(),
                expected: None,
            },
        )?;
        let assign = |agent: &str, view: &Value| Edit::Assign {
            name: "diagnostics".into(),
            project: project.into(),
            agent: agent.into(),
            enabled: true,
            expected: server_revision(view),
        };
        let first = report(&dir, assign("antigravity", &saved))?;
        assert!(report(&dir, assign("codex", &saved)).is_err());
        let second = report(&dir, assign("codex", &first))?;
        let settings = project.join(".codex/config.toml");
        let codex = fs::read_to_string(&settings)?;
        assert!(codex.contains("[mcp_servers.diagnostics]"));
        assert!(codex.contains("env = { UNIT = \"ms\" }"));
        let antigravity_text = fs::read_to_string(&antigravity)?;
        assert!(antigravity_text.contains("// keep this comment"));
        assert!(antigravity_text.contains("\"existing\": {\"command\": \"mine\"}"));
        fs::write(
            &antigravity,
            antigravity_text.replace("\"--measure\"", "/* local note */ \"--measure\""),
        )?;
        let updated = report(
            &dir,
            Edit::Save {
                name: "diagnostics".into(),
                source: SERVER.replace("--measure", "--retrieve"),
                expected: Some(server_revision(&second)),
            },
        )?;
        assert_eq!(
            updated.pointer("/placements/0/status").unwrap(),
            &json!("conflict")
        );
        let changed = fs::read_to_string(&settings)?;
        assert!(changed.contains("--retrieve"));
        report(&dir, Edit::Sync)?;
        assert_eq!(changed, fs::read_to_string(&settings)?);
        let removed = report(
            &dir,
            Edit::Remove {
                name: "diagnostics".into(),
                expected: server_revision(&updated),
            },
        )?;
        assert_eq!(
            removed.pointer("/placements/0/status").unwrap(),
            &json!("conflict")
        );
        assert_eq!(
            removed.pointer("/placements/1/status").unwrap(),
            &json!("removed")
        );
        assert!(!fs::read_to_string(settings)?.contains("diagnostics"));
        let antigravity_text = fs::read_to_string(antigravity)?;
        assert!(
            antigravity_text.contains("/* local note */") && antigravity_text.contains("existing")
        );
        Ok(())
    }

    #[test]
    fn recovers_native_write_without_claiming_an_unowned_server() -> Result<()> {
        let root = tempfile::tempdir()?;
        let dir = root.path().join("data");
        let assignment = Assignment {
            project: root
                .path()
                .canonicalize()?
                .into_os_string()
                .into_string()
                .unwrap(),
            agent: "claude".into(),
        };
        let path = dir.join("project-mcp/library.json");
        let name = "diagnostics";
        let mut store = Store::default();
        let source = r#"{"command":"diagnostics"}"#.to_owned();
        edit_native(
            &dir,
            root.path(),
            "claude",
            NativeEdit::Save {
                name: name.into(),
                source,
                expected: None,
                replace: false,
            },
            |after| {
                store.pending = Some(Pending {
                    name: name.into(),
                    assignment: assignment.clone(),
                    path: after.path.clone(),
                    revision: after.revision.clone(),
                    content: Some(hash(&after.servers[name])),
                });
                store.save(&path)
            },
        )?;
        let result = report(&dir, Edit::Sync)?;
        assert_eq!(
            result.pointer("/placements/0/status").unwrap(),
            &json!("removed")
        );
        let native = placement_view(&dir, &assignment)?;
        assert!(native.servers.is_empty());
        assert!(Store::load(&path)?.pending.is_none());
        Ok(())
    }
}
