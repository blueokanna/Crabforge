use anyhow::{Context, Result};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::ErrorKind,
    path::Path,
};

/// Remember the protocols generated for each model so a catalog refresh can tell
/// a personal API override from an older gateway choice. The caller holds the profile lock.
pub(super) fn keep_model_apis(
    home: &Path,
    existing: Option<&Value>,
    models: &mut [Value],
) -> Result<()> {
    let path = home.join("crabforge-model-apis.json");
    let previous: BTreeMap<String, BTreeSet<String>> = match fs::File::open(&path) {
        Ok(file) => {
            serde_json::from_reader(file).with_context(|| format!("invalid {}", path.display()))?
        }
        Err(e) if e.kind() == ErrorKind::NotFound => BTreeMap::new(),
        Err(e) => return Err(e.into()),
    };
    let saved: BTreeMap<_, _> = existing
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|m| Some((m["id"].as_str()?, m)))
        .collect();
    let mut wanted = BTreeMap::new();
    for model in models {
        let id = model["id"]
            .as_str()
            .context("missing Pi model id")?
            .to_owned();
        let api = model["api"].as_str().unwrap_or_default().to_owned();
        let mut history = previous.get(&id).cloned().unwrap_or_default();
        if let Some(old) = saved.get(id.as_str())
            && let Some(custom) = old["api"].as_str().filter(|s| !s.is_empty())
            && !history.is_empty()
            && custom != api
            && !history.contains(custom)
        {
            let fields = model
                .as_object_mut()
                .context("invalid generated Pi model")?;
            for key in ["api", "baseUrl", "compat"] {
                if let Some(value) = old.get(key) {
                    fields.insert(key.into(), value.clone());
                } else {
                    fields.remove(key);
                }
            }
        }
        history.insert(api);
        wanted.insert(id, history);
    }
    if wanted != previous {
        // Persist first. A retry after a failed models.json replacement still
        // recognizes both the old and the proposed protocol as gateway choices.
        super::write_catalog(&path, &serde_json::to_value(wanted)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn personal_api_keeps_its_url_and_compat_until_removed() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let sync = |saved: Value, wanted: Value| -> Result<Value> {
            let mut models = [wanted];
            keep_model_apis(dir.path(), Some(&json!([saved])), &mut models)?;
            Ok(models.into_iter().next().unwrap())
        };
        let chat = json!({"id":"relay/m"});
        let responses =
            json!({"id":"relay/m", "api":"openai-responses", "compat":{"supportsStrictMode":true}});
        let custom = json!({"id":"relay/m", "api":"anthropic-messages", "baseUrl":"http://127.0.0.1:9", "compat":{"forceAdaptiveThinking":true}});
        // No history in an older profile. A generated choice follows the upstream on refresh.
        assert_eq!(sync(custom.clone(), responses.clone())?, responses);
        assert_eq!(sync(responses.clone(), chat.clone())?, chat);
        for target in [chat.clone(), responses.clone()] {
            assert_eq!(sync(custom.clone(), target)?, custom);
        }
        // The user clears the override, or restores an older generated API.
        assert_eq!(sync(chat.clone(), responses.clone())?, responses);
        assert_eq!(sync(responses.clone(), chat.clone())?, chat);
        Ok(())
    }
}
