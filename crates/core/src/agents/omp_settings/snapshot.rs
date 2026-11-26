use crate::agents::yaml::{Scalar, Value as Yaml};
use anyhow::{Result, bail};
use nextjson::{Map, Value};

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
        Yaml::Value(Scalar::Null) => Value::Null,
        Yaml::Value(Scalar::Boolean(b)) => (*b).into(),
        Yaml::Value(Scalar::String(s)) => s.into(),
        Yaml::Value(Scalar::Integer(i)) => (*i).into(),
        Yaml::Value(Scalar::FloatingPoint(f)) => nextjson::Number::from_f64(f.0).into(),
        Yaml::Representation(n, _, None) => n.parse::<u64>()?.into(),
        Yaml::Sequence(list) => Value::Array(list.iter().map(json_node).collect::<Result<_>>()?),
        Yaml::Mapping(mapping) => {
            let mut object = Map::with_capacity(mapping.len());
            for (key, value) in mapping {
                let key = match key {
                    Yaml::Value(Scalar::String(s)) => s.clone(),
                    Yaml::Value(Scalar::Boolean(b)) => b.to_string(),
                    Yaml::Value(Scalar::Integer(_) | Scalar::FloatingPoint(_))
                    | Yaml::Representation(..) => {
                        let number = json_node(key)?;
                        anyhow::ensure!(!number.is_null(), "non-finite YAML key");
                        number.to_string()
                    }
                    _ => bail!("profile mapping key cannot be represented in JSON"),
                };
                object.insert(key, json_node(value)?);
            }
            Value::Object(object)
        }
        Yaml::Tagged(tag, inner) => {
            let mut object = Map::new();
            object.insert(tag.to_string(), json_node(inner)?);
            Value::Object(object)
        }
        _ => bail!("unresolved YAML node in profile snapshot"),
    })
}
