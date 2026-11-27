use super::{ANTIGRAVITY, Bundle, CLAUDE, CODEX, MAX_MANIFEST, PI, Tree, encode};
use crate::{agent_files, hooks, project_mcp::library as mcp, skills};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonSerialize, Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path};

#[derive(NsonSerialize)]
struct Resource {
    key: String,
    name: String,
    definition: Option<Value>,
    detail: Option<String>,
}

impl Resource {
    fn new(key: String, name: String, definition: Result<Value>) -> Self {
        let (definition, detail) = match definition {
            Ok(value) => (Some(value), None),
            Err(error) => (None, Some(error.to_string())),
        };
        Self {
            key,
            name,
            definition,
            detail,
        }
    }
}

fn parse(source: &str) -> Result<Value> {
    nextjson::from_str(source.strip_prefix('\u{feff}').unwrap_or(source))
        .map_err(anyhow::Error::msg)
}

fn read(tree: &Tree, path: &str) -> Result<Value> {
    let source = tree
        .text(Path::new(path))?
        .context("bundle JSON file is missing")?;
    ensure!(source.len() <= MAX_MANIFEST, "bundle JSON exceeds 256 KiB");
    parse(source)
}

fn check_text(text: &str) -> Result<()> {
    ensure!(
        ![
            "${CLAUDE_PLUGIN_",
            "${CODEX_PLUGIN_",
            "${ANTIGRAVITY_PLUGIN_",
            "${PLUGIN_ROOT}",
            "${PLUGIN_DATA}"
        ]
        .iter()
        .any(|marker| text.contains(marker)),
        "this definition depends on the native plugin environment"
    );
    Ok(())
}

fn check_variables(value: &Value) -> Result<()> {
    match value {
        Value::String(text) => check_text(text)?,
        Value::Array(list) => list.iter().try_for_each(check_variables)?,
        Value::Object(object) => object.values().try_for_each(check_variables)?,
        _ => {}
    }
    Ok(())
}

fn hook_clients(tree: &Tree, bundle: &Bundle, path: &str) -> Result<BTreeSet<&'static str>> {
    let mut clients = BTreeSet::new();
    for (manifest, client) in [(CLAUDE, "claude"), (CODEX, "codex"), (ANTIGRAVITY, "agy")] {
        if !bundle.manifests.iter().any(|file| file == manifest) {
            continue;
        }
        let value = read(tree, manifest)?;
        if let Some(hooks) = value.get("hooks") {
            let paths = hooks
                .as_array()
                .map_or_else(|| std::slice::from_ref(hooks), Vec::as_slice);
            if paths.iter().any(|value| {
                value.as_str().and_then(super::relative).as_deref() == Some(Path::new(path))
            }) {
                clients.insert(client);
            }
        }
    }
    if path == "hooks.json" && bundle.manifests.iter().any(|file| file == ANTIGRAVITY) {
        clients.insert("agy");
    } else if path == "hooks/hooks.json"
        && (clients.is_empty() || bundle.manifests.iter().any(|file| file == CLAUDE))
    {
        clients.insert(
            if bundle.manifests.iter().any(|file| file == CODEX)
                && !bundle.manifests.iter().any(|file| file == CLAUDE)
            {
                "codex"
            } else {
                "claude"
            },
        );
    }
    Ok(clients)
}

fn hook_rows(path: &str, client: &str, bundle: &Bundle, document: &Value) -> Result<Vec<Resource>> {
    let events = document
        .get("hooks")
        .unwrap_or(document)
        .as_object()
        .context("expected a hook events object")?;
    let mut rows = Vec::new();
    for (event, groups) in events.iter() {
        let Some(groups) = groups.as_array() else {
            rows.push(Resource::new(
                format!("{client}:{event}"),
                event.to_owned(),
                Err(anyhow::anyhow!("expected an array of hook handlers")),
            ));
            continue;
        };
        for (group_index, group) in groups.iter().enumerate() {
            let handlers = group.get("hooks").and_then(Value::as_array);
            let size = handlers.map_or(1, Vec::len);
            for index in 0..size {
                let key = format!("{client}:{event}:{group_index}:{index}");
                let digest = hex::encode(Sha256::digest(format!("{path}\0{key}")));
                let prefix = &bundle.name[..bundle.name.len().min(40)];
                let name = format!("{prefix}-{}", &digest[..16]);
                let definition = (|| {
                    let entry = if let Some(handlers) = handlers {
                        let mut object = group
                            .as_object()
                            .context("expected a hook group object")?
                            .clone();
                        object.insert("hooks".into(), Value::Array(vec![handlers[index].clone()]));
                        Value::Object(object)
                    } else {
                        group.clone()
                    };
                    check_variables(&entry)?;
                    parse(&hooks::bundle_hook(client, name, event, &entry)?)
                })();
                rows.push(Resource::new(
                    key,
                    format!("{client} · {event} · {}", rows.len() + 1),
                    definition,
                ));
            }
        }
    }
    Ok(rows)
}

fn structured_rows(tree: &Tree, bundle: &Bundle, kind: &str, path: &str) -> Result<Vec<Resource>> {
    let document;
    let mut sources = Vec::new();
    if let Some((manifest, field)) = path.split_once(" / ") {
        document = read(tree, manifest)?;
        let values = document
            .get(field)
            .context("bundle manifest entry is missing")?;
        let values = values
            .as_array()
            .map_or_else(|| std::slice::from_ref(values), Vec::as_slice);
        for (index, value) in values
            .iter()
            .enumerate()
            .filter(|(_, value)| value.is_object())
        {
            sources.push((index, value));
        }
    } else {
        document = read(tree, path)?;
        sources.push((0, &document));
    }
    let mut rows = Vec::new();
    for (index, document) in sources {
        if kind == "mcp" {
            let entries = document
                .get("mcpServers")
                .filter(|value| value.get("command").is_none() && value.get("url").is_none())
                .unwrap_or(document);
            for (name, value) in entries
                .as_object()
                .context("expected an MCP servers object")?
                .iter()
            {
                let definition =
                    check_variables(value).and_then(|()| parse(&mcp::bundle_config(name, value)?));
                rows.push(Resource::new(
                    format!("{index}:{name}"),
                    name.to_owned(),
                    definition,
                ));
            }
        } else {
            let clients = match path.split_once(" / ").map(|(manifest, _)| manifest) {
                Some(CLAUDE) => BTreeSet::from(["claude"]),
                Some(CODEX) => BTreeSet::from(["codex"]),
                Some(ANTIGRAVITY) => BTreeSet::from(["agy"]),
                _ => hook_clients(tree, bundle, path)?,
            };
            for client in clients {
                let source = format!("{path} / {index}");
                for mut row in hook_rows(&source, client, bundle, document)? {
                    row.key = format!("{index}:{}", row.key);
                    rows.push(row);
                }
            }
        }
    }
    Ok(rows)
}

pub(super) fn edit(
    dir: &Path,
    bundle: &Bundle,
    folder: &Path,
    kind: &str,
    path: &str,
    copy: Option<&str>,
) -> Result<String> {
    ensure!(
        bundle
            .components
            .get(kind)
            .is_some_and(|paths| paths.iter().any(|file| file == path)),
        "the resource is missing from this bundle revision"
    );
    let tree = Tree::read(folder)?;
    ensure!(
        tree.hash() == bundle.content,
        "saved plugin snapshot was modified"
    );
    let mut skill = None;
    let mut agent_file = None;
    let rows = match kind {
        "skills" => {
            let parent = Path::new(path)
                .parent()
                .context("missing skill directory")?;
            let subtree = tree.subtree(parent);
            let (name, description) = skills::details(&subtree)?;
            let raw = tree
                .text(Path::new(path))?
                .context("bundle skill is missing")?;
            let definition = check_text(raw).map(|()| {
                let end = raw.floor_char_boundary(raw.len().min(48 * 1024));
                json!({"name":name,"description":description,"source":path,"document":&raw[..end],"truncated":end < raw.len()})
            });
            skill = Some((folder.join(parent), subtree));
            vec![Resource::new(path.into(), name, definition)]
        }
        "subagents" | "commands" => {
            let kind = if kind == "subagents" {
                agent_files::Kind::Subagent
            } else {
                agent_files::Kind::Command
            };
            let raw = tree
                .text(Path::new(path))?
                .context("bundle document is missing")?;
            let pi = bundle.manifests.iter().any(|manifest| manifest == PI);
            let entry = agent_files::bundle_entry(kind, path, raw, pi)?;
            let definition = parse(&encode(&entry)?).and_then(|value| {
                check_variables(&value)?;
                Ok(value)
            });
            let name = entry.name.clone();
            agent_file = Some((kind, entry));
            vec![Resource::new(path.into(), name, definition)]
        }
        "mcp" | "hooks" => structured_rows(&tree, bundle, kind, path)?,
        _ => anyhow::bail!("unknown plugin resource kind"),
    };
    let Some(key) = copy else {
        return encode(&rows);
    };
    let row = rows
        .iter()
        .find(|row| row.key == key)
        .context("plugin resource selection changed")?;
    let definition = row
        .definition
        .as_ref()
        .with_context(|| row.detail.clone().unwrap_or_default())?;
    if let Some((source, tree)) = skill {
        skills::copy_from_bundle(dir, source, tree)?;
    } else if let Some((kind, entry)) = agent_file {
        agent_files::library(
            dir,
            kind,
            None,
            agent_files::Edit::Save {
                previous: None,
                entry,
            },
        )?;
    } else if kind == "mcp" {
        mcp::edit(
            dir,
            mcp::Edit::Save {
                name: row.name.clone(),
                source: encode(definition)?,
                expected: None,
            },
        )?;
    } else {
        hooks::edit(
            dir,
            hooks::Edit::Save {
                source: encode(definition)?,
                previous: None,
            },
        )?;
    }
    encode(row)
}
