use crate::{config::effort::rank, translate::Format};
use bytes::Bytes;
use nextjson::{Value, json};

mod gemini;
pub(super) use gemini::thinking as gemini_thinking;

pub(super) fn rewrite(
    format: Format,
    bytes: &[u8],
    model: &str,
    rewrite_model: bool,
    requested: &str,
    levels: &[String],
) -> Option<Bytes> {
    let target = match format {
        Format::Gemini => requested,
        _ => fitted(requested, levels.iter().map(String::as_str))?,
    };
    let mut body: Value = nextjson::from_slice(bytes).ok()?;
    match format {
        Format::Chat => {
            let current = body.get("reasoning_effort")?.as_str()?;
            if !changes(current, target) {
                return None;
            }
            body.as_object_mut()?
                .insert("reasoning_effort".into(), target.into());
        }
        Format::Responses => {
            let reasoning = body.get_mut("reasoning")?.as_object_mut()?;
            if !changes(reasoning.get("effort")?.as_str()?, target) {
                return None;
            }
            reasoning.insert("effort".into(), target.into());
        }
        Format::Messages => messages(&mut body, target)?,
        Format::Gemini => gemini::rewrite(&mut body, model, target, levels)?,
    }
    if rewrite_model {
        body.as_object_mut()?.insert("model".into(), model.into());
    }
    Some(nextjson::to_vec(&body).ok()?.into())
}

fn fitted<'a>(requested: &str, levels: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let desired = rank(requested)?;
    levels
        .filter_map(|level| {
            let place = rank(level)?;
            (1..=6).contains(&place).then_some((level, place))
        })
        .min_by_key(|&(level, place)| {
            (desired.abs_diff(place), level != requested, u8::MAX - place)
        })
        .map(|(level, _)| level)
}

fn changes(current: &str, target: &str) -> bool {
    let current = current.trim();
    !current.is_empty()
        && !current.eq_ignore_ascii_case("none")
        && !current.eq_ignore_ascii_case("off")
        && current != target
}

fn messages(body: &mut Value, target: &str) -> Option<()> {
    let adaptive = match body.pointer("/thinking/type")?.as_str()? {
        "adaptive" => true,
        "enabled" => false,
        _ => return None,
    };
    if !adaptive {
        let max = body.get("max_tokens").and_then(Value::as_u64).unwrap_or(0);
        let budget = budget(target)?;
        let budget = if max > 0 { budget.min(max - 1) } else { budget };
        if budget < 1024 {
            return None;
        }
        body.get_mut("thinking")?
            .as_object_mut()?
            .insert("budget_tokens".into(), budget.into());
    }
    if adaptive
        || body
            .pointer("/output_config/effort")
            .is_some_and(|value| !value.is_null())
    {
        if body.get("output_config").is_none_or(Value::is_null) {
            body.as_object_mut()?
                .insert("output_config".into(), json!({}));
        }
        body.get_mut("output_config")?
            .as_object_mut()?
            .insert("effort".into(), target.into());
    }
    Some(())
}

fn budget(level: &str) -> Option<u64> {
    Some(match rank(level)? {
        1 | 2 => 4096,
        3 => 10_000,
        4 => 24_000,
        5..=7 => 32_000,
        _ => return None,
    })
}
