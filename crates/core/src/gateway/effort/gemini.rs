use super::{fitted, rank};
use nextjson::Value;

const LEVEL: [&str; 2] = ["thinkingLevel", "thinking_level"];
const BUDGET: [&str; 2] = ["thinkingBudget", "thinking_budget"];

pub(in crate::gateway) fn thinking(body: &Value) -> (Option<&str>, Option<i64>) {
    let Some((generation, thinking)) = thinking_keys(body) else {
        return (None, None);
    };
    let config = &body[generation][thinking];
    let level = LEVEL.iter().find_map(|key| config.get(key)?.as_str());
    let budget = BUDGET.iter().find_map(|key| config.get(key)?.as_i64());
    (level, budget)
}

pub(super) fn rewrite(
    body: &mut Value,
    model: &str,
    desired: &str,
    advertised: &[String],
) -> Option<()> {
    let (level, budget) = thinking(body);
    if budget == Some(0)
        || level.is_some_and(|level| {
            level.eq_ignore_ascii_case("none") || level.eq_ignore_ascii_case("off")
        })
        || (budget.is_none_or(|n| n < -1) && level.is_none_or(str::is_empty))
    {
        return None;
    }
    let (keys, value) = configured(model, desired, advertised)?;
    let (generation, thinking) = thinking_keys(body)?;
    let config = body
        .get_mut(generation)?
        .get_mut(thinking)?
        .as_object_mut()?;
    let snake = thinking == "thinking_config"
        || config.contains_key(BUDGET[1])
        || config.contains_key(LEVEL[1]);
    for key in LEVEL.into_iter().chain(BUDGET) {
        config.remove(key);
    }
    config.insert(keys[usize::from(snake)].into(), value);
    Some(())
}

fn thinking_keys(body: &Value) -> Option<(&'static str, &'static str)> {
    for generation in ["generationConfig", "generation_config"] {
        if let Some(config) = body.get(generation) {
            return ["thinkingConfig", "thinking_config"]
                .into_iter()
                .find(|&key| config.get(key).is_some())
                .map(|key| (generation, key));
        }
    }
    None
}

pub(super) fn configured(
    model: &str,
    desired: &str,
    advertised: &[String],
) -> Option<([&'static str; 2], Value)> {
    let model = model.rsplit('/').next()?.to_ascii_lowercase();
    let is = |id: &str| matches_model(&model, id);
    let max_budget = if is("gemini-2.5-pro") {
        Some(32768_u64)
    } else if is("gemini-2.5-flash") {
        Some(24576_u64)
    } else {
        None
    };
    if let Some(max) = max_budget {
        let target = if advertised.is_empty() {
            desired
        } else {
            fitted(desired, advertised.iter().map(String::as_str))?
        };
        let budget = super::budget(target)?;
        return Some((BUDGET, budget.min(max).into()));
    }
    let defaults: &[&str] = if is("gemini-4-neon") || is("gemini-4-hydrogen") {
        &["low", "medium", "high", "xhigh"]
    } else if is("gemini-3.1-flash-lite-image") {
        &["minimal", "high"]
    } else if is("gemini-3-pro") {
        &["low", "high"]
    } else if [
        "gemini-4-carbon",
        "gemini-4-argon",
        "gemini-3.8-flash",
        "gemini-3.1-pro",
    ]
    .into_iter()
    .any(is)
    {
        &["low", "medium", "high"]
    } else if [
        "gemini-3.6-flash",
        "gemini-3.5-flash-lite",
        "gemini-3.1-flash-lite",
        "gemini-3-flash",
        "gemini-robotics-er-2",
    ]
    .into_iter()
    .any(is)
    {
        &["minimal", "low", "medium", "high"]
    } else {
        &[]
    };
    let target = if advertised.is_empty() {
        fitted(desired, defaults.iter().copied())?
    } else if !defaults.is_empty() || model.starts_with("gemini-3") || model.starts_with("gemini-4")
    {
        fitted(
            desired,
            advertised
                .iter()
                .map(String::as_str)
                .filter(|level| matches!(rank(level), Some(1..=5))),
        )?
    } else {
        return None;
    };
    let value = match rank(target)? {
        1 => "MINIMAL",
        2 => "LOW",
        3 => "MEDIUM",
        4 => "HIGH",
        5 => "XHIGH",
        _ => return None,
    };
    Some((LEVEL, value.into()))
}

fn matches_model(model: &str, id: &str) -> bool {
    model == id
        || model
            .strip_prefix(id)
            .is_some_and(|tail| tail.starts_with('-'))
}
