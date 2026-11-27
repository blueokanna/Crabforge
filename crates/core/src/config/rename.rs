use super::{Config, MemberPattern, load, settings_lock, valid_id, write_settings};
use anyhow::{Context, Result, ensure};
use std::path::Path;

pub fn rename_provider(dir: &Path, previous: &str, from: &str, to: &str) -> Result<()> {
    let _lock = settings_lock(dir)?;
    let mut cfg = load(dir)?;
    cfg.check_snapshot(previous)?;
    let to = to.trim();
    if from == to {
        ensure!(
            cfg.providers.iter().any(|p| p.id == from),
            "unknown provider"
        );
        return Ok(());
    }
    // Retrying after one profile failed resumes the remaining profile edits.
    if cfg
        .former_providers
        .get(from)
        .is_none_or(|current| current != to)
    {
        cfg.rename_provider(from, to)?;
        write_settings(dir, &cfg)?;
    }
    crate::agents::omp_settings::rename_provider(dir, from, to)
        .with_context(|| format!("provider renamed; to retry the profile migration run carbforge-cli provider rename {from} {to}"))?;
    Ok(())
}

impl Config {
    /// Resolve an older qualified model ID only while its provider still offers that model.
    pub fn renamed_model(&self, id: &str) -> Option<String> {
        let (former, model) = id.split_once('/')?;
        let current = self.former_providers.get(former)?;
        let provider = self.providers.iter().find(|p| &p.id == current)?;
        provider
            .models
            .iter()
            .any(|m| m == model)
            .then(|| format!("{current}/{model}"))
    }

    fn rename_provider(&mut self, from: &str, to: &str) -> Result<()> {
        let index = self
            .providers
            .iter()
            .position(|p| p.id == from)
            .context("unknown provider")?;
        ensure!(
            valid_id(to) && !matches!(to, "auto" | "group" | "crabforge" | "magpie"),
            "invalid or reserved provider ID {to:?}"
        );
        ensure!(
            !self.providers.iter().any(|p| p.id == to),
            "provider {to} already exists"
        );
        let provider = &mut self.providers[index];
        if provider.catalog.is_empty() {
            provider.catalog = super::groups::default_catalog(from).to_owned();
        }
        provider.id = to.to_owned();
        let prefix = format!("{from}/");
        let rename = |id: &str| {
            id.strip_prefix(&prefix)
                .map(|model| format!("{to}/{model}"))
        };
        for members in self
            .groups
            .values_mut()
            .chain(self.group_placed.values_mut())
            .chain(self.providers.iter_mut().map(|p| &mut p.fallback))
        {
            for id in members {
                if let Some(new) = rename(id) {
                    *id = new;
                }
            }
        }
        for rules in self.group_match.values_mut() {
            for rule in rules {
                let text = rule.as_str();
                let (qualifier, glob) = text
                    .strip_prefix("free:")
                    .map_or(("", text), |s| ("free:", s));
                if let Some((provider, tail)) = glob.split_once('/')
                    && provider.eq_ignore_ascii_case(from)
                {
                    *rule = MemberPattern::try_from(format!("{qualifier}{to}/{tail}"))?;
                }
            }
        }
        for off in self.group_off.values_mut() {
            *off = std::mem::take(off)
                .into_iter()
                .map(|id| rename(&id).unwrap_or(id))
                .collect();
        }
        for rule in self.group_rules.values_mut().flatten() {
            rule.remap(|id| Some(rename(id).unwrap_or_else(|| id.to_owned())));
        }
        for pick in self
            .group_picks
            .values_mut()
            .chain(self.group_last_picks.values_mut())
        {
            if let Some(new) = rename(pick) {
                *pick = new;
            }
        }
        for weights in self
            .group_weights
            .values_mut()
            .chain(self.group_last_weights.values_mut())
        {
            *weights = std::mem::take(weights)
                .into_iter()
                .map(|(id, weight)| (rename(&id).unwrap_or(id), weight))
                .collect();
        }
        self.model_same_as = std::mem::take(&mut self.model_same_as)
            .into_iter()
            .map(|(id, model)| (rename(&id).unwrap_or(id), model))
            .collect();
        if self.auto_group_skip.remove(from) {
            self.auto_group_skip.insert(to.to_owned());
        }
        self.former_providers.remove(to);
        for current in self.former_providers.values_mut().filter(|id| *id == from) {
            *current = to.to_owned();
        }
        self.former_providers.insert(from.to_owned(), to.to_owned());
        self.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn former_names_follow_the_provider_until_reclaimed() -> Result<()> {
        let mut cfg = Config::parse(
            r#"{"providers":[{"id":"auto","models":["m"],"endpoints":{"chat":"http://localhost/v1"}},{"id":"other","models":["n"],"endpoints":{"chat":"http://localhost/v1"}}]}"#,
        )?;
        cfg.rename_provider("auto", "first")?;
        cfg.rename_provider("first", "second")?;
        let mut cfg = Config::parse(&cfg.to_json()?)?;
        assert_eq!(cfg.renamed_model("auto/m").as_deref(), Some("second/m"));
        assert!(cfg.renamed_model("first/n").is_none());
        cfg.rename_provider("other", "first")?;
        assert!(cfg.renamed_model("first/m").is_none());
        assert_eq!(cfg.renamed_model("other/n").as_deref(), Some("first/n"));
        Ok(())
    }
}
