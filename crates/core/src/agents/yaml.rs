//! Semantic profile values used to verify edits to the lossless YAML syntax tree.
use anyhow::{Context, Result, bail, ensure};
pub(super) use saphyr::{MappingOwned as Mapping, ScalarOwned as Scalar, YamlOwned as Value};
use saphyr::{ScalarStyle, Tag, YamlEmitter, YamlLoader};
use saphyr_parser::Parser;

pub(super) fn text(value: impl Into<String>) -> Value {
    Value::Value(Scalar::String(value.into()))
}

pub(super) fn boolean(value: bool) -> Value {
    Value::Value(Scalar::Boolean(value))
}

pub(super) fn parse(source: &str) -> Result<Value> {
    let mut loader = YamlLoader::<Value>::default();
    loader.early_parse(false);
    Parser::new_from_str(source).load(&mut loader, true)?;
    if let Some(error) = loader.error() {
        bail!("{error}");
    }
    let mut docs = loader.into_documents();
    ensure!(docs.len() <= 1, "expected a single YAML document");
    match docs.pop() {
        None => Ok(Value::Value(Scalar::Null)),
        Some(doc) => resolve(doc),
    }
}

fn resolve(node: Value) -> Result<Value> {
    Ok(match node {
        Value::Representation(value, style, tag) => scalar(value, style, tag)?,
        Value::Sequence(items) => {
            Value::Sequence(items.into_iter().map(resolve).collect::<Result<_>>()?)
        }
        Value::Mapping(fields) => {
            let mut mapping = Mapping::new();
            for (key, value) in fields {
                ensure!(
                    mapping.insert(resolve(key)?, resolve(value)?).is_none(),
                    "duplicate YAML key"
                );
            }
            Value::Mapping(mapping)
        }
        Value::Tagged(tag, node) => Value::Tagged(tag, Box::new(resolve(*node)?)),
        Value::Value(_) => node,
        Value::BadValue | Value::Alias(_) => bail!("unresolved YAML value"),
    })
}

fn scalar(value: String, style: ScalarStyle, tag: Option<Tag>) -> Result<Value> {
    let core = tag
        .as_ref()
        .filter(|t| t.is_yaml_core_schema())
        .map(|t| t.suffix.as_str());
    let parsed = match core {
        Some("null") => {
            ensure!(
                matches!(value.as_str(), "~" | "null" | "Null" | "NULL"),
                "invalid YAML null"
            );
            Value::Value(Scalar::Null)
        }
        Some("bool") => {
            let b = Scalar::parse_from_cow(value.into())
                .as_bool()
                .context("invalid YAML boolean")?;
            boolean(b)
        }
        Some("int") => integer(&value)?.context("invalid YAML integer")?,
        Some("float") => Value::Value(Scalar::FloatingPoint(
            saphyr::parse_core_schema_fp(&value)
                .context("invalid YAML float")?
                .into(),
        )),
        Some(_) => text(value),
        None if style != ScalarStyle::Plain => text(value),
        None if value.is_empty() => Value::Value(Scalar::Null),
        None if leading_zero(&value) => text(value),
        None => match integer(&value)? {
            Some(integer) => integer,
            None => Value::Value(Scalar::parse_from_cow(value.into())),
        },
    };
    Ok(
        match tag.filter(|t| !t.is_yaml_core_schema() && t.to_string() != "!") {
            Some(tag) => Value::Tagged(tag, Box::new(parsed)),
            None => parsed,
        },
    )
}

fn leading_zero(value: &str) -> bool {
    let digits = value.strip_prefix(['+', '-']).unwrap_or(value);
    digits.len() > 1 && digits.starts_with('0') && digits.bytes().all(|b| b.is_ascii_digit())
}

// Existing profiles distinguish leading-zero strings, signed base-prefixed integers and u64.
fn integer(value: &str) -> Result<Option<Value>> {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    let (digits, radix) = [("0x", 16), ("0o", 8), ("0b", 2)]
        .into_iter()
        .find_map(|(prefix, radix)| unsigned.strip_prefix(prefix).map(|digits| (digits, radix)))
        .unwrap_or((unsigned, 10));
    if leading_zero(value) || digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return Ok(None);
    }
    let n = u64::from_str_radix(digits, radix).context("integer outside YAML profile range")?;
    let signed = if value.starts_with('-') {
        -i128::from(n)
    } else {
        i128::from(n)
    };
    if let Ok(i) = i64::try_from(signed) {
        return Ok(Some(Value::Value(Scalar::Integer(i))));
    }
    ensure!(signed >= 0, "integer outside YAML profile range");
    // Saphyr's integer variant is signed. A canonical plain scalar preserves u64 exactly.
    Ok(Some(Value::Representation(
        n.to_string(),
        ScalarStyle::Plain,
        None,
    )))
}

pub(super) fn configuration(source: &str) -> Result<Mapping> {
    let node = parse(source)?;
    if node.is_null() {
        return Ok(Mapping::new());
    }
    node.into_mapping().context("expected a YAML mapping")
}

pub(super) fn render(node: &Value) -> Result<String> {
    let mut output = String::new();
    let mut emitter = YamlEmitter::new(&mut output);
    emitter.quoted_value(true);
    emitter.dump(&node.into())?;
    Ok(output)
}

pub(super) fn same_mapping(a: &Mapping, b: &Mapping) -> bool {
    a.len() == b.len()
        && a.iter()
            .all(|(k, v)| b.get(k).is_some_and(|other| same(v, other)))
}

fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Mapping(a), Value::Mapping(b)) => same_mapping(a, b),
        (Value::Sequence(a), Value::Sequence(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same(a, b))
        }
        (Value::Tagged(ta, a), Value::Tagged(tb, b)) => ta == tb && same(a, b),
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::omp_settings::snapshot::json_node;

    #[test]
    fn profile_scalars_aliases_and_tags_keep_their_types_after_rendering() -> Result<()> {
        let source = "model: &role !fallback [!!str 3, 01, !!bool 'TRUE', 0xffffffffffffffff, -0b11, ~]\ncopy: *role\n'1': text\n1: number\nnote: |-\n  one\n  two\n";
        let mapping = configuration(source)?;
        let role = json_node(&mapping[&text("model")])?;
        let expected = nextjson::json!({"!fallback": ["3", "01", true, u64::MAX, -3, null]});
        assert_eq!(role, expected);
        assert_eq!(json_node(&mapping[&text("copy")])?, expected);
        let rendered = render(&Value::Mapping(mapping.clone()))?;
        assert!(same_mapping(&configuration(&rendered)?, &mapping));
        for invalid in [
            "1: a\n0x1: b",
            "a: 1\n---\nb: 2",
            "value: *missing",
            "value: !!int '01'",
        ] {
            assert!(configuration(invalid).is_err(), "{invalid}");
        }
        Ok(())
    }
}
