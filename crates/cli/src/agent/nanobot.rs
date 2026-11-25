use anyhow::{Context, Result};
use serde_json::{Map, Value, json};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::Path,
};
use tempfile::NamedTempFile;

fn section<'a>(
    parent: &'a mut Map<String, Value>,
    key: &str,
) -> Result<&'a mut Map<String, Value>> {
    parent
        .entry(key)
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .with_context(|| format!("expected an object for nanobot {key}"))
}

pub(super) fn profile(home: &Path, root: &str, model: &str) -> Result<NamedTempFile> {
    fs::create_dir_all(home)?;
    let path = home.join("config.json");
    let mut config: Map<String, Value> = match fs::read(&path) {
        Ok(bytes) => {
            serde_json::from_slice(&bytes).with_context(|| format!("invalid {}", path.display()))?
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Map::new(),
        Err(e) => return Err(e.into()),
    };
    let defaults = section(section(&mut config, "agents")?, "defaults")?;
    defaults.remove("modelPreset");
    defaults.remove("model_preset");
    defaults.insert("provider".into(), json!("crabforge"));
    defaults.insert("model".into(), json!(format!("crabforge/{model}")));
    section(&mut config, "providers")?.insert(
        "crabforge".into(),
        json!({
            "apiBase": format!("{root}/v1"), "apiKey": "${CRABFORGE_GATEWAY_KEY}",
            "extraHeaders": {"User-Agent":"nanobot"}
        }),
    );
    // nanobot keeps sessions beside the config. Isolate the selection for each
    // invocation while allowing the client to reopen its own session history.
    let mut file = tempfile::Builder::new()
        .prefix("launch-")
        .suffix(".json")
        .tempfile_in(home)?;
    serde_json::to_writer(file.as_file_mut(), &config)?;
    file.flush()?;
    Ok(file)
}
