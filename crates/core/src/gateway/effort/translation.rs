use super::{fitted, gemini, rank};
use crate::{json::unsigned, translate::Format};
use anyhow::{Context, Result, bail, ensure};
use nextjson::{Map, Value, json};

enum Thinking<'a> {
    Disabled,
    Enabled { level: &'a str, budget: Option<u64> },
}

pub(in crate::gateway) fn apply(
    from: Format,
    to: Format,
    input: &Value,
    output: &mut Map,
    model: &str,
    rule: Option<&str>,
    levels: &[String],
) -> Result<()> {
    let Some(thinking) = read(from, input)? else {
        return Ok(());
    };
    let native = || format!("{model} cannot represent the requested reasoning on {to:?}");
    match to {
        Format::Chat | Format::Responses => {
            let target = match thinking {
                Thinking::Disabled => levels
                    .iter()
                    .find(|level| rank(level) == Some(0))
                    .map(String::as_str),
                Thinking::Enabled { level, .. } => {
                    fitted(rule.unwrap_or(level), levels.iter().map(String::as_str))
                }
            }
            .with_context(native)?;
            if to == Format::Chat {
                output.insert("reasoning_effort".into(), target.into());
            } else {
                output.insert("reasoning".into(), json!({"effort":target}));
            }
        }
        Format::Messages => match thinking {
            Thinking::Disabled => {
                output.insert("thinking".into(), json!({"type":"disabled"}));
            }
            Thinking::Enabled { level, budget } => {
                let target = fitted(rule.unwrap_or(level), levels.iter().map(String::as_str))
                    .with_context(native)?;
                if claude_adaptive(model) {
                    output.insert("thinking".into(), json!({"type":"adaptive"}));
                    output.insert("output_config".into(), json!({"effort":target}));
                } else {
                    let max = output.get("max_tokens").and_then(unsigned).unwrap_or(0);
                    let amount = budget
                        .filter(|_| rule.is_none() && target == level)
                        .or_else(|| super::budget(target))
                        .with_context(native)?;
                    let amount = amount.min(max.saturating_sub(1));
                    ensure!(
                        amount >= 1024,
                        "Anthropic translation requires max_tokens greater than 1024 for thinking"
                    );
                    output.insert(
                        "thinking".into(),
                        json!({"type":"enabled","budget_tokens":amount}),
                    );
                }
            }
        },
        Format::Gemini => {
            let (key, value) = match thinking {
                Thinking::Disabled => gemini::disabled(model),
                Thinking::Enabled { level, .. } => {
                    gemini::configured(model, rule.unwrap_or(level), levels)
                }
            }
            .with_context(native)?;
            if !output.contains_key("generationConfig") {
                output.insert("generationConfig".into(), json!({}));
            }
            let mut config = Map::new();
            config.insert(key[0].into(), value);
            output
                .get_mut("generationConfig")
                .and_then(Value::as_object_mut)
                .context("invalid Gemini generationConfig")?
                .insert("thinkingConfig".into(), config.into());
        }
    }
    Ok(())
}

fn claude_adaptive(model: &str) -> bool {
    let model = model.rsplit('/').next().unwrap_or(model);
    let model = model
        .split_once("anthropic.")
        .map_or(model, |(_, leaf)| leaf);
    ["claude-opus-4-", "claude-sonnet-4-"]
        .into_iter()
        .any(|prefix| {
            model
                .strip_prefix(prefix)
                .and_then(|tail| tail.split(['-', ':']).next()?.parse::<u32>().ok())
                .is_some_and(|minor| minor >= 6)
        })
}

fn read(format: Format, body: &Value) -> Result<Option<Thinking<'_>>> {
    let (level, budget) = match format {
        Format::Chat => (string(body, "reasoning_effort")?, None),
        Format::Responses => {
            let Some(config) = body.get("reasoning").filter(|v| !v.is_null()) else {
                return Ok(None);
            };
            check_keys(config, &["effort", "summary"])?;
            ensure!(
                string(config, "summary")?.is_none_or(|summary| summary == "none"),
                "reasoning summaries require a native endpoint"
            );
            (string(config, "effort")?, None)
        }
        Format::Messages => {
            let thinking = body.get("thinking").unwrap_or(&Value::Null);
            let output = body.get("output_config").unwrap_or(&Value::Null);
            if !output.is_null() {
                check_keys(output, &["effort"])?;
            }
            let effort = string(output, "effort")?;
            if thinking.is_null() {
                return effort.map(|level| level_setting(level, None)).transpose();
            }
            check_keys(thinking, &["type", "budget_tokens", "display"])?;
            ensure!(
                string(thinking, "display")?.is_none_or(|display| display == "omitted"),
                "displayed thinking requires a native endpoint"
            );
            let budget = number(thinking, "budget_tokens")?;
            match string(thinking, "type")? {
                Some("disabled") if budget.is_none() => return Ok(Some(Thinking::Disabled)),
                Some("adaptive") if budget.is_none() => (Some(effort.unwrap_or("high")), None),
                Some("enabled") if budget.is_some_and(|n| n >= 1024) => (effort, budget),
                _ => bail!("invalid or unsupported Messages thinking configuration"),
            }
        }
        Format::Gemini => {
            let Some(config) = body
                .pointer("/generationConfig/thinkingConfig")
                .filter(|v| !v.is_null())
            else {
                return Ok(None);
            };
            check_keys(
                config,
                &["thinkingLevel", "thinkingBudget", "includeThoughts"],
            )?;
            ensure!(
                matches!(
                    config.get("includeThoughts"),
                    None | Some(Value::Null | Value::Bool(false))
                ),
                "displayed thoughts require a native endpoint"
            );
            let level = string(config, "thinkingLevel")?;
            let budget = number(config, "thinkingBudget")?;
            ensure!(
                level.is_none() || budget.is_none(),
                "Gemini translation requires either a thinking level or a budget"
            );
            (level, budget)
        }
    };
    let level = level.map(|level| match level {
        "MINIMAL" => "minimal",
        "LOW" => "low",
        "MEDIUM" => "medium",
        "HIGH" => "high",
        "XHIGH" => "xhigh",
        level => level,
    });
    match (level, budget) {
        (Some(level), budget) => {
            level_setting(level, budget.and_then(|n| u64::try_from(n).ok())).map(Some)
        }
        (None, Some(0)) => Ok(Some(Thinking::Disabled)),
        (None, Some(n @ -1..)) => {
            let level = match n {
                -1 => "medium",
                1..=4096 => "low",
                4097..=10_000 => "medium",
                10_001..=24_000 => "high",
                _ => "xhigh",
            };
            Ok(Some(Thinking::Enabled {
                level,
                budget: u64::try_from(n).ok(),
            }))
        }
        (None, Some(_)) => bail!("invalid reasoning budget"),
        (None, None) => Ok(None),
    }
}

fn level_setting(level: &str, budget: Option<u64>) -> Result<Thinking<'_>> {
    if level.eq_ignore_ascii_case("off") || level.eq_ignore_ascii_case("none") {
        return Ok(Thinking::Disabled);
    }
    match rank(level) {
        Some(_) => Ok(Thinking::Enabled { level, budget }),
        None => bail!("unsupported reasoning level {level:?}"),
    }
}

fn string<'a>(object: &'a Value, key: &str) -> Result<Option<&'a str>> {
    object
        .get(key)
        .filter(|v| !v.is_null())
        .map(|v| {
            v.as_str()
                .with_context(|| format!("invalid value for {key}"))
        })
        .transpose()
}

fn number(object: &Value, key: &str) -> Result<Option<i64>> {
    object
        .get(key)
        .filter(|v| !v.is_null())
        .map(|v| {
            v.as_i64()
                .with_context(|| format!("invalid value for {key}"))
        })
        .transpose()
}

fn check_keys(config: &Value, allowed: &[&str]) -> Result<()> {
    for key in config
        .as_object()
        .context("expected a thinking options object")?
        .keys()
    {
        ensure!(
            allowed.contains(&key),
            "the reasoning option {key} requires a native endpoint"
        );
    }
    Ok(())
}
