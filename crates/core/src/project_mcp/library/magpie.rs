use super::{Config, Server, Store, encode, lock_library, name};
use crate::config::magpie_library_section;
use anyhow::{Context, Result, bail, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    path::Path,
};

#[derive(NsonDeserialize)]
struct MagpieServer {
    name: String,
    agents: Option<BTreeSet<String>>,
    #[njson(flatten)]
    fields: BTreeMap<String, Value>,
}

#[derive(NsonSerialize)]
pub struct ImportReport {
    servers: Vec<PlannedServer>,
    other_library_sections: Vec<String>,
    applied: bool,
}

#[derive(NsonSerialize)]
struct PlannedServer {
    name: String,
    transport: String,
    source_agents: BTreeSet<String>,
    skipped_fields: Vec<String>,
    action: &'static str,
}

fn transport_config(fields: BTreeMap<String, Value>) -> Result<(Config, Vec<String>)> {
    let mut transport = nextjson::Map::new();
    let mut skipped_fields = Vec::new();
    for (key, value) in fields {
        match key.as_str() {
            "transport" | "command" | "args" | "env" | "url" | "headers" => {
                transport.insert(key, value);
            }
            "note" | "hub" | "idleTimeout" | "keepAlive" => skipped_fields.push(key),
            _ if value.is_null()
                || value.as_str() == Some("")
                || value.as_bool() == Some(false)
                || crate::json::unsigned(&value) == Some(0)
                || value.as_array().is_some_and(Vec::is_empty) => {}
            _ => bail!("unsupported Magpie MCP field {key:?}"),
        }
    }
    let config: Config = nextjson::from_value(Value::Object(transport))
        .map_err(|_| anyhow::anyhow!("invalid Magpie MCP transport configuration"))?;
    config.validate()?;
    ensure!(
        encode(&config)?.len() <= super::super::MAX_SERVER,
        "MCP library entry exceeds its size limit"
    );
    Ok((config, skipped_fields))
}

/// Import portable server definitions for explicit assignment to local projects.
pub fn import_magpie(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    selected: &[String],
    apply: bool,
) -> Result<ImportReport> {
    let (entries, other_library_sections) = magpie_library_section(file, passphrase, "mcp")?;
    let selected: BTreeSet<_> = selected.iter().map(String::as_str).collect();
    let mut servers = BTreeMap::new();
    for entry in entries {
        let key = entry
            .pointer("/name")
            .and_then(Value::as_str)
            .context("missing Magpie MCP server name")?;
        if !selected.is_empty() && !selected.contains(key) {
            continue;
        }
        let imported: MagpieServer = nextjson::from_value(entry)
            .map_err(|_| anyhow::anyhow!("invalid Magpie MCP server"))?;
        name(&imported.name)?;
        let (config, skipped_fields) = transport_config(imported.fields)
            .with_context(|| format!("cannot import Magpie MCP server {:?}", imported.name))?;
        let Entry::Vacant(slot) = servers.entry(imported.name) else {
            bail!("duplicate Magpie MCP server name");
        };
        slot.insert((config, imported.agents.unwrap_or_default(), skipped_fields));
    }
    for key in selected {
        ensure!(
            servers.contains_key(key),
            "Magpie MCP server {key:?} was not found"
        );
    }

    let library = dir.join("project-mcp");
    let _lock = if apply && !servers.is_empty() {
        Some(lock_library(&library)?)
    } else {
        None
    };
    let path = library.join("library.json");
    let mut store = Store::load(&path)?;
    let mut planned = Vec::new();
    let mut changed = false;
    for (name, (config, source_agents, skipped_fields)) in servers {
        let transport = config.transport.clone();
        let action = match store.servers.entry(name.clone()) {
            Entry::Occupied(slot) if slot.get().config == config => "identical",
            Entry::Occupied(_) if apply => bail!("MCP server {name:?} already exists"),
            Entry::Occupied(_) => "conflict",
            Entry::Vacant(slot) => {
                slot.insert(Server {
                    config,
                    assignments: BTreeSet::new(),
                });
                changed = true;
                "create"
            }
        };
        planned.push(PlannedServer {
            name,
            transport,
            source_agents,
            skipped_fields,
            action,
        });
    }
    if changed {
        if apply {
            store.save(&path)?;
        } else {
            store.contents()?;
        }
    }
    Ok(ImportReport {
        servers: planned,
        other_library_sections,
        applied: apply,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nextjson::json;

    #[cfg(windows)]
    #[test]
    fn library_can_be_updated_during_a_read_only_preview() -> Result<()> {
        use std::{fs::File, io::Read};

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("library.json");
        let mut library = Store::default();
        library.save(&path)?;
        let previous = library.contents()?;
        let mut reader = File::open(&path)?;
        let config = nextjson::from_value(json!({"transport":"stdio","command":"fixture"}))
            .map_err(anyhow::Error::msg)?;
        library.servers.insert(
            "fixture".into(),
            Server {
                config,
                assignments: BTreeSet::new(),
            },
        );
        library.save(&path)?;
        let mut original = String::new();
        reader.read_to_string(&mut original)?;
        assert_eq!(original, previous);
        assert!(Store::load(&path)?.servers.contains_key("fixture"));
        Ok(())
    }

    #[test]
    fn import_refuses_to_discard_mcp_access_settings() -> Result<()> {
        let fields = BTreeMap::from([
            ("transport".into(), json!("http")),
            ("url".into(), json!("https://example.invalid/mcp")),
            ("note".into(), json!("A reusable server")),
        ]);
        let (_, skipped) = transport_config(fields.clone())?;
        assert_eq!(skipped, ["note"]);
        for (key, value) in [
            ("disabledTools", json!(["write"])),
            ("oauth", json!({"clientId": "example"})),
            ("account", json!("secondary")),
            ("hidden", json!(true)),
        ] {
            let mut restricted = fields.clone();
            restricted.insert(key.into(), value);
            let Err(error) = transport_config(restricted) else {
                anyhow::bail!("import accepted {key}");
            };
            assert!(error.to_string().contains(key));
        }
        Ok(())
    }
}
