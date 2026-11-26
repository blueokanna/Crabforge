use serde_json::Value;
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

include!(concat!(env!("OUT_DIR"), "/provider_model_ids.rs"));

fn load_registry() -> Option<Value> {
    let command = super::super::omp_profiles::installed().ok()?;
    let executable = Path::new(command.get_program());
    let script = command.get_args().next().map(PathBuf::from).or_else(|| {
        if executable.is_absolute() {
            return Some(executable.to_owned());
        }
        env::split_paths(&env::var_os("PATH")?)
            .map(|dir| dir.join(executable))
            .find(|p| p.is_file())
    })?;
    let script = script.canonicalize().ok()?;
    let modules = script.ancestors().find(|p| p.ends_with("node_modules"))?;
    for dir in [
        modules.join("@oh-my-pi/pi-coding-agent/node_modules"),
        modules.to_owned(),
    ] {
        if let Ok(file) = fs::File::open(dir.join("@oh-my-pi/pi-catalog/src/models.json"))
            && let Ok(data) = serde_json::from_reader(file)
        {
            return Some(data);
        }
    }
    None
}

// Oh My Pi 19 stores the provider defaults and individual model fields in one node table.
fn packed_field<'a>(catalog: &'a Value, encoded: &Value, key: usize) -> Option<&'a Value> {
    let node = catalog
        .get("nodes")?
        .get(usize::try_from(encoded.get(0)?.as_u64()?).ok()?)?;
    let pair = node
        .get(1)?
        .as_array()?
        .as_chunks::<2>()
        .0
        .iter()
        .find(|pair| pair[0].as_u64() == Some(key as u64))?;
    Some(&pair[1])
}

fn chat_ids<'a>(catalog: &'a Value, provider: &str) -> Vec<&'a str> {
    let chat = |kind: Option<&str>| matches!(kind, None | Some("" | "tiny"));
    if catalog.get("version").and_then(Value::as_u64) == Some(1) {
        let Some(rows) = catalog.get("providers").and_then(Value::as_array) else {
            return Vec::new();
        };
        let Some(row) = rows
            .iter()
            .find(|r| r.get(0).and_then(Value::as_str) == Some(provider))
        else {
            return Vec::new();
        };
        let field = |name| {
            let index = catalog["keys"]
                .as_array()
                .and_then(|v| v.iter().position(|k| k == name));
            let base = index.and_then(|i| packed_field(catalog, &row[1], i));
            move |model: &Value| {
                index
                    .and_then(|i| packed_field(catalog, model, i))
                    .or(base)
                    .and_then(Value::as_str)
            }
        };
        let kind = field("kind");
        let model_type = field("type");
        let id = field("id");
        return row[2]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|model| {
                let overrides = &model[1];
                if chat(kind(overrides))
                    && matches!(model_type(overrides), None | Some("" | "chat"))
                {
                    id(overrides)
                        .or(model[0].as_str())
                        .filter(|id| !id.is_empty())
                } else {
                    None
                }
            })
            .collect();
    }
    catalog
        .get(provider)
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|m| m.values())
        .filter(|m| {
            chat(m["kind"].as_str()) && matches!(m["type"].as_str(), None | Some("" | "chat"))
        })
        .filter_map(|m| m["id"].as_str().filter(|s| !s.is_empty()))
        .collect()
}

pub(super) fn add_provider_models(names: BTreeSet<&str>, models: &mut BTreeSet<String>) {
    if names.is_empty() {
        return;
    }
    let catalog = load_registry().unwrap_or(Value::Null);
    for name in names {
        let mut ids = chat_ids(&catalog, name);
        if ids.is_empty()
            && let Ok(i) = PROVIDERS.binary_search_by_key(&name, |(id, _)| *id)
        {
            ids.extend(PROVIDERS[i].1.lines());
        }
        models.extend(ids.into_iter().map(|id| format!("{name}/{id}")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_model_overrides_take_precedence_over_provider_defaults() {
        let compact = serde_json::json!({
            "version": 1, "keys": ["kind", "type", "id"],
            "nodes": [[0,[0,"image"],0,[]],[0,[0,"tiny",2,"chat"],0,[]],
                      [0,[],0,[]],[0,[0,"tiny",1,"embedding"],0,[]]],
            "providers": [["example",[0],[["alias",[1]],["image",[2]],["vector",[3]]]]]
        });
        assert_eq!(chat_ids(&compact, "example"), ["chat"]);
        let legacy =
            serde_json::json!({"example":{"m":{"id":"chat"},"v":{"id":"voice","kind":"tts"}}});
        assert_eq!(chat_ids(&legacy, "example"), ["chat"]);
    }
}
