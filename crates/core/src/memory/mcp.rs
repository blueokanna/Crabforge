use super::{
    Changes, Entry, KINDS, Query, lock_store, markdown_text, read_in, save_locked, scope_key,
    search_where,
};
use anyhow::{Context, Result, bail, ensure};
use nextjson::{NsonDeserialize, Value, json};
use std::{
    io::{BufRead, Read, Write},
    path::Path,
};

const MAX_MESSAGE: usize = 1024 * 1024;
const VERSIONS: &[&str] = &["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

#[derive(Default, NsonDeserialize)]
#[njson(default, deny_unknown_fields)]
struct Arguments {
    id: Option<String>,
    query: String,
    text: Option<String>,
    kind: Option<String>,
    scope: Option<String>,
    tags: Option<Vec<String>>,
    why: Option<String>,
    limit: Option<usize>,
}

struct Server<'a> {
    dir: &'a Path,
    agent: Option<String>,
    project: Option<String>,
}

pub fn serve(
    dir: &Path,
    agent: Option<&str>,
    project_key: Option<&str>,
    mut input: impl BufRead,
    mut output: impl Write,
) -> Result<()> {
    let server = Server {
        dir,
        agent: agent.map(|key| scope_key("agent", key)).transpose()?,
        project: project_key
            .map(|key| scope_key("project", key))
            .transpose()?,
    };
    let mut line = Vec::new();
    loop {
        line.clear();
        if (&mut input)
            .take((MAX_MESSAGE + 1) as u64)
            .read_until(b'\n', &mut line)?
            == 0
        {
            return Ok(());
        }
        ensure!(line.len() <= MAX_MESSAGE, "MCP message exceeds 1 MiB");
        let response = match std::str::from_utf8(&line)
            .ok()
            .and_then(|line| nextjson::from_str::<Value>(line).ok())
        {
            Some(message) => server.message(&message),
            None => Some(rpc_error(&Value::Null, -32700, "Parse error")),
        };
        if let Some(response) = response {
            writeln!(output, "{response}")?;
            output.flush()?;
        }
    }
}

fn rpc_error(id: &Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

fn tools() -> Value {
    let tool = |name, description, properties, required: &[&str], read_only| {
        json!({"name": name, "description": description,
               "inputSchema": {"type":"object","properties":properties,"required":required,"additionalProperties":false},
               "annotations": {"readOnlyHint":read_only,"openWorldHint":false}})
    };
    json!({"tools": [
        tool("memory_search", "Read active memories in your global, project and agent scopes.",
            json!({"query":{"type":"string"}, "kind":{"type":"string","enum":KINDS},
                   "scope":{"enum":["all","global","project","agent"]},
                   "tags":{"type":"array","items":{"type":"string"}},
                   "limit":{"type":"integer","minimum":1,"maximum":50}}), &[], true),
        tool("memory_save", "Remember a single fact or preference. Use an ID to update an agent-created memory. Keep credentials out of memory.",
            json!({"text":{"type":"string","maxLength":2000}, "kind":{"type":"string","enum":KINDS},
                   "scope":{"enum":["global","project","agent"]}, "why":{"type":"string"},
                   "id":{"type":"string"}, "tags":{"type":"array","items":{"type":"string"}}}), &["text"], false),
        tool("memory_forget", "Archive a memory by ID when the user asks to forget it.",
            json!({"id":{"type":"string"}}), &["id"], false),
        tool("memory_context", "Show the agent and project scopes available to this MCP process.",
            json!({}), &[], true)
    ]})
}

impl Server<'_> {
    fn message(&self, message: &Value) -> Option<Value> {
        let id = message.get("id");
        let method = message.get("method").and_then(Value::as_str);
        if message.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
            || method.is_none()
            || id.is_some_and(|id| !matches!(id, Value::String(_) | Value::Number(_)))
        {
            return Some(rpc_error(&Value::Null, -32600, "Invalid Request"));
        }
        let id = id?;
        let params = message.get("params");
        let result = match method? {
            "initialize" => {
                let version = params
                    .and_then(|p| p.get("protocolVersion"))
                    .and_then(Value::as_str);
                let Some(version) = version else {
                    return Some(rpc_error(id, -32602, "A protocol version is required"));
                };
                let version = VERSIONS
                    .iter()
                    .copied()
                    .find(|v| *v == version)
                    .unwrap_or(VERSIONS[0]);
                json!({"protocolVersion":version,"capabilities":{"tools":{}},
                       "serverInfo":{"name":"crabforge-memory","version":env!("CARGO_PKG_VERSION")}})
            }
            "ping" => json!({}),
            "tools/list" => tools(),
            "tools/call" => {
                let name = params.and_then(|p| p.get("name")).and_then(Value::as_str);
                let Some(name) = name else {
                    return Some(rpc_error(id, -32602, "A tool name is required"));
                };
                if !matches!(
                    name,
                    "memory_search" | "memory_save" | "memory_forget" | "memory_context"
                ) {
                    return Some(rpc_error(id, -32602, "Unknown tool"));
                }
                let response = self.call(name, params.and_then(|p| p.get("arguments")));
                match response {
                    Ok(value) => json!({"content":[{"type":"text","text":value.to_string()}]}),
                    Err(error) => {
                        json!({"content":[{"type":"text","text":error.to_string()}],"isError":true})
                    }
                }
            }
            _ => return Some(rpc_error(id, -32601, "Method not found")),
        };
        Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
    }

    fn scopes(&self) -> Vec<&str> {
        [
            Some("global"),
            self.project.as_deref(),
            self.agent.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn scope(&self, name: &str) -> Result<&str> {
        match name {
            "global" => Ok("global"),
            "project" => self
                .project
                .as_deref()
                .context("no project scope; set --project-key when starting the server"),
            "agent" => self
                .agent
                .as_deref()
                .context("no agent scope; set --agent when starting the server"),
            _ => bail!("invalid memory scope"),
        }
    }

    fn call(&self, tool: &str, arguments: Option<&Value>) -> Result<Value> {
        let args: Arguments = match arguments {
            Some(value) if value.is_object() => {
                nextjson::from_str(&value.to_string()).map_err(anyhow::Error::msg)?
            }
            Some(_) => bail!("arguments must be an object"),
            None => Arguments::default(),
        };
        match tool {
            "memory_context" => Ok(json!({"scopes":self.scopes()})),
            "memory_search" => self.search(args),
            "memory_save" => self.save(args),
            "memory_forget" => self.forget(args.id.as_deref().context("memory ID is required")?),
            _ => unreachable!("tool validated before dispatch"),
        }
    }

    fn search(&self, args: Arguments) -> Result<Value> {
        let scopes = match args.scope.as_deref() {
            None | Some("all") => self.scopes(),
            Some(name) => vec![self.scope(name)?],
        };
        let results = search_where(
            &self.dir.join("memory"),
            Query {
                text: args.query,
                status: Some("active".into()),
                kind: args.kind,
                tags: args.tags.unwrap_or_default(),
                limit: Some(args.limit.unwrap_or(10).clamp(1, 50)),
                ..Default::default()
            },
            |entry| scopes.contains(&entry.scope.as_str()),
        )?;
        Ok(
            json!({"memories":results.entries.iter().map(agent_entry).collect::<Vec<_>>(),
                  "total": results.total, "scopes": scopes}),
        )
    }

    fn save(&self, args: Arguments) -> Result<Value> {
        let text = args
            .text
            .as_deref()
            .map(markdown_text)
            .filter(|text| !text.is_empty())
            .context("memory text is required")?;
        let scopes = self.scopes();
        let (folder, _lock) = lock_store(self.dir)?;
        let previous = args
            .id
            .as_deref()
            .map(|id| read_in(&folder, id))
            .transpose()?;
        if let Some(entry) = &previous {
            ensure!(
                scopes.contains(&entry.scope.as_str()) && entry.status == "active",
                "memory is outside the active scopes"
            );
            ensure!(
                entry.origin == "agent",
                "edit this memory from the desktop or CLI"
            );
        }
        let kind = args
            .kind
            .as_deref()
            .or(previous.as_ref().map(|entry| entry.kind.as_str()))
            .unwrap_or("fact");
        let scope = match (args.scope.as_deref(), &previous) {
            (Some(scope), _) => self.scope(scope)?,
            (None, Some(entry)) => &entry.scope,
            (None, None) if matches!(kind, "profile" | "preference" | "rule") => "global",
            (None, None) => self.project.as_deref().unwrap_or("global"),
        };
        if previous.is_none() {
            let duplicate = search_where(
                &folder,
                Query {
                    status: Some("active".into()),
                    limit: Some(1),
                    ..Default::default()
                },
                |entry| entry.scope == scope && entry.kind == kind && entry.text == text,
            )?;
            if let Some(entry) = duplicate.entries.first() {
                return Ok(agent_entry(entry));
            }
        }
        let existing = previous
            .as_ref()
            .map(|entry| (entry.id.as_str(), entry.revision.as_str()));
        let saved = save_locked(
            &folder,
            existing,
            Changes {
                text: Some(text),
                kind: Some(kind.into()),
                scope: Some(scope.into()),
                tags: args.tags,
                why: args.why,
                ..Default::default()
            },
            "agent",
            None,
        )?;
        Ok(agent_entry(&saved))
    }

    fn forget(&self, id: &str) -> Result<Value> {
        let (folder, _lock) = lock_store(self.dir)?;
        let entry = read_in(&folder, id)?;
        ensure!(
            self.scopes().contains(&entry.scope.as_str()),
            "memory is outside the active scopes"
        );
        if entry.status != "archived" {
            ensure!(
                entry.status == "active",
                "memory is outside the active scopes"
            );
            save_locked(
                &folder,
                Some((id, &entry.revision)),
                Changes {
                    status: Some("archived".into()),
                    ..Default::default()
                },
                "agent",
                None,
            )?;
        }
        Ok(json!({"id":id,"status":"archived"}))
    }
}

fn agent_entry(entry: &Entry) -> Value {
    json!({"id":entry.id,"text":entry.text,"kind":entry.kind,"scope":entry.scope,
           "tags":entry.tags,"why":entry.why,"pinned":entry.pinned})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdio_memory_obeys_project_and_agent_scopes() -> Result<()> {
        fn run(root: &Path, project: &str, messages: &[Value]) -> Result<Vec<Value>> {
            let input = messages
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join("\n");
            let mut output = Vec::new();
            serve(
                root,
                Some("gemini"),
                Some(project),
                input.as_bytes(),
                &mut output,
            )?;
            String::from_utf8(output)?
                .lines()
                .map(|line| nextjson::from_str(line).map_err(anyhow::Error::msg))
                .collect()
        }
        fn call(id: i32, name: &str, arguments: Value) -> Value {
            json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":arguments}})
        }
        fn content(value: &Value) -> Value {
            let text = value
                .pointer("/result/content/0/text")
                .and_then(Value::as_str)
                .unwrap();
            nextjson::from_str(text).unwrap()
        }

        let home = tempfile::tempdir()?;
        let root = home.path();
        let response = run(
            root,
            "example/current",
            &[
                json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}),
                json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                call(
                    2,
                    "memory_save",
                    json!({"scope":"project","text":"Run the Windows tests."}),
                ),
                call(
                    3,
                    "memory_save",
                    json!({"scope":"project","text":"Run the Windows tests."}),
                ),
                call(4, "memory_search", json!({})),
            ],
        )?;
        assert_eq!(response.len(), 4);
        assert_eq!(
            response[0].pointer("/result/protocolVersion"),
            Some(&json!("2025-06-18"))
        );
        let created = content(&response[1]);
        let id = created.get("id").and_then(Value::as_str).unwrap();
        assert_eq!(content(&response[2]).get("id"), Some(&json!(id)));
        assert_eq!(content(&response[3]).get("total"), Some(&json!(1)));

        let other = run(
            root,
            "example/other",
            &[
                call(5, "memory_search", json!({})),
                call(6, "memory_forget", json!({"id":id})),
            ],
        )?;
        assert_eq!(content(&other[0]).get("total"), Some(&json!(0)));
        assert_eq!(other[1].pointer("/result/isError"), Some(&json!(true)));
        let archived = run(
            root,
            "example/current",
            &[
                call(7, "memory_forget", json!({"id":id})),
                call(8, "memory_search", json!({})),
            ],
        )?;
        assert_eq!(
            content(&archived[0]).get("status"),
            Some(&json!("archived"))
        );
        assert_eq!(content(&archived[1]).get("total"), Some(&json!(0)));
        assert_eq!(super::super::read(root, id)?.status, "archived");
        Ok(())
    }
}
