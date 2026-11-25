use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, value};

pub(super) fn profile(dir: &Path, ids: Vec<String>, root: &str) -> Result<PathBuf> {
    let home = std::path::absolute(dir.join("agents/vibe"))?;
    super::edit_toml(&home, |doc| {
        let mut provider = Table::new();
        provider["name"] = value("crabforge");
        provider["api_base"] = value(format!("{root}/v1"));
        provider["api_style"] = value("openai");
        provider["backend"] = value("generic");
        provider["api_key_env_var"] = value("CRABFORGE_GATEWAY_KEY");
        let providers = tables(doc, "providers")?;
        providers.retain(|p| p.get("name").and_then(Item::as_str) != Some("crabforge"));
        providers.push(provider);
        let models = tables(doc, "models")?;
        models.retain(|m| m.get("provider").and_then(Item::as_str) != Some("crabforge"));
        for id in ids {
            let mut model = Table::new();
            model["alias"] = value(format!("crabforge/{id}"));
            model["name"] = value(id);
            model["provider"] = value("crabforge");
            model["thinking_levels"] = value(Array::from_iter(["off"]));
            models.push(model);
        }
        Ok(())
    })?;
    Ok(home)
}

/// Vibe materializes plugin blobs under the session store on startup. Two
/// Windows processes cannot rename the same open blob into that store.
pub(super) fn session_directory(command: &mut Command, home: &Path) -> Result<fs::File> {
    for slot in 0usize.. {
        let lock = fs::File::options()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(home.join(format!("session-{slot}.lock")))?;
        match lock.try_lock() {
            Ok(()) => {
                if slot > 0 && std::env::var_os("VIBE_SESSION_LOGGING__SAVE_DIR").is_none() {
                    command.env(
                        "VIBE_SESSION_LOGGING__SAVE_DIR",
                        home.join(format!("logs/parallel-{slot}")),
                    );
                }
                return Ok(lock);
            }
            Err(fs::TryLockError::WouldBlock) => continue,
            Err(fs::TryLockError::Error(e)) => return Err(e.into()),
        }
    }
    unreachable!()
}

fn tables<'a>(doc: &'a mut DocumentMut, name: &str) -> Result<&'a mut ArrayOfTables> {
    let item = doc
        .entry(name)
        .or_insert(Item::ArrayOfTables(ArrayOfTables::new()));
    // Vibe also writes inline arrays of tables when saving an edited configuration.
    if item.is_array() {
        *item = item
            .clone()
            .into_array_of_tables()
            .map(Item::ArrayOfTables)
            .map_err(|_| anyhow::anyhow!("invalid Vibe {name} array"))?;
    }
    item.as_array_of_tables_mut()
        .with_context(|| format!("invalid Vibe {name} array"))
}
