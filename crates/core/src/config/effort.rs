use super::{Config, Provider, valid_model_ref};
use anyhow::{Result, ensure};
use std::collections::HashSet;

pub(crate) fn rank(level: &str) -> Option<u8> {
    Some(match level {
        "none" => 0,
        "minimal" => 1,
        "low" | "light" => 2,
        "medium" => 3,
        "high" => 4,
        "xhigh" | "extra_high" => 5,
        "max" => 6,
        "ultra" => 7,
        _ => return None,
    })
}

pub(super) fn validate_levels(levels: &[String], catalog: bool) -> Result<()> {
    let mut seen = HashSet::new();
    for level in levels {
        ensure!(
            rank(level).is_some()
                && (catalog || !matches!(level.as_str(), "light" | "extra_high" | "ultra")),
            "unsupported reasoning level {level:?}"
        );
        ensure!(seen.insert(level), "duplicate reasoning level {level:?}");
    }
    Ok(())
}

impl Config {
    pub(crate) fn model_levels<'a>(&'a self, provider: &'a Provider, model: &str) -> &'a [String] {
        provider
            .model_reasoning_levels
            .get(model)
            .filter(|levels| !levels.is_empty())
            .or_else(|| self.model_efforts.get(&format!("{}/{model}", provider.id)))
            .map_or(&[], Vec::as_slice)
    }

    pub fn set_model_efforts(&mut self, id: &str, levels: Vec<String>) -> Result<()> {
        ensure!(valid_model_ref(id), "invalid model reference {id:?}");
        let (provider, model) = id.split_once('/').expect("validated model reference");
        ensure!(
            self.providers
                .iter()
                .any(|p| p.id == provider && p.models.iter().any(|m| m == model)),
            "unknown model {id:?}"
        );
        validate_levels(&levels, false)?;
        if levels.is_empty() {
            self.model_efforts.remove(id);
        } else {
            self.model_efforts.insert(id.to_owned(), levels);
        }
        Ok(())
    }
}
