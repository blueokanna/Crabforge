use anyhow::{Result, bail};
use nextjson::{Map, Value};
use serde_yaml::Value as Yaml;

pub(super) fn same(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => crate::json::equivalent(a, b),
        (None, None) => true,
        _ => false,
    }
}

/// Encode a YAML node for profile snapshots and recovery records.
pub(crate) fn json_node(node: &Yaml) -> Result<Value> {
    Ok(match node {
        Yaml::Null => Value::Null,
        Yaml::Bool(b) => (*b).into(),
        Yaml::String(s) => s.into(),
        Yaml::Number(n) => {
            if let Some(i) = n.as_u64() {
                i.into()
            } else if let Some(i) = n.as_i64() {
                i.into()
            } else {
                n.as_f64().and_then(nextjson::Number::from_f64).into()
            }
        }
        Yaml::Sequence(list) => Value::Array(list.iter().map(json_node).collect::<Result<_>>()?),
        Yaml::Mapping(mapping) => {
            let mut object = Map::with_capacity(mapping.len());
            for (key, value) in mapping {
                let key = match key {
                    Yaml::String(s) => s.clone(),
                    Yaml::Bool(b) => b.to_string(),
                    Yaml::Number(n) => {
                        let number = json_node(key)?;
                        anyhow::ensure!(!number.is_null(), "non-finite YAML key {n}");
                        number.to_string()
                    }
                    _ => bail!("profile mapping key cannot be represented in JSON"),
                };
                object.insert(key, json_node(value)?);
            }
            Value::Object(object)
        }
        Yaml::Tagged(tagged) => {
            let mut object = Map::new();
            object.insert(tagged.tag.to_string(), json_node(&tagged.value)?);
            Value::Object(object)
        }
    })
}
