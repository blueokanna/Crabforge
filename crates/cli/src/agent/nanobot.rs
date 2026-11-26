use super::object_section;
use anyhow::{Context, Result};
use nextjson::{Map, json};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::Path,
};
use tempfile::NamedTempFile;

pub(super) fn profile(home: &Path, root: &str, model: &str) -> Result<NamedTempFile> {
    fs::create_dir_all(home)?;
    let path = home.join("config.json");
    let mut config: Map = match fs::read(&path) {
        Ok(bytes) => nextjson::from_slice(&bytes)
            .map_err(anyhow::Error::msg)
            .with_context(|| format!("invalid {}", path.display()))?,
        Err(e) if e.kind() == ErrorKind::NotFound => Map::new(),
        Err(e) => return Err(e.into()),
    };
    let defaults = object_section(object_section(&mut config, "agents")?, "defaults")?;
    defaults.remove("modelPreset");
    defaults.remove("model_preset");
    defaults.insert("provider".into(), json!("crabforge"));
    defaults.insert("model".into(), json!(format!("crabforge/{model}")));
    object_section(&mut config, "providers")?.insert(
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
    file.write_all(&nextjson::to_vec(&config).map_err(anyhow::Error::msg)?)?;
    file.flush()?;
    Ok(file)
}
