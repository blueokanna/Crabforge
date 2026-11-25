use anyhow::{Context, Result};
use std::{collections::HashSet, path::Path};
use toml_edit::{Array, Item, Table, TableLike, value};

fn table<'a>(parent: &'a mut dyn TableLike, key: &str) -> Result<&'a mut dyn TableLike> {
    parent
        .entry(key)
        .or_insert_with(|| Item::Table(Table::new()))
        .as_table_like_mut()
        .with_context(|| format!("invalid Kimi table {key}"))
}

pub(super) fn profile(home: &Path, ids: Vec<String>, root: &str) -> Result<()> {
    super::edit_toml(home, |doc| {
        let provider = table(table(doc.as_table_mut(), "providers")?, "crabforge")?;
        provider.remove("api_key");
        provider.insert("type", value("kimi"));
        provider.insert("base_url", value(format!("{root}/v1")));
        provider.insert("api_key_env", value("CRABFORGE_GATEWAY_KEY"));
        let models = table(doc.as_table_mut(), "models")?;
        let current: HashSet<_> = ids.iter().map(String::as_str).collect();
        let stale: Vec<_> = models
            .iter()
            .filter(|(k, v)| {
                k.strip_prefix("crabforge/")
                    .is_some_and(|id| !current.contains(id))
                    && v.get("provider").and_then(Item::as_str) == Some("crabforge")
            })
            .map(|(k, _)| k.to_owned())
            .collect();
        for alias in stale {
            models.remove(&alias);
        }
        for id in ids {
            let model = table(models, &format!("crabforge/{id}"))?;
            model.insert("provider", value("crabforge"));
            model.insert("model", value(id));
            model.entry("max_context_size").or_insert(value(128000));
            model
                .entry("capabilities")
                .or_insert(value(Array::from_iter(["tool_use"])));
        }
        Ok(())
    })
}
