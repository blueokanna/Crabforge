use super::yaml::{self, Mapping, Value};
use super::{configuration, persist, source};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use std::{collections::BTreeMap, path::Path};
use yaml_edit::YamlFile;

const ROLES: &[&str] = &[
    "task", "smol", "slow", "plan", "vision", "advisor", "commit", "tiny", "memory",
];
const LEVELS: &[&str] = &["auto", "minimal", "low", "medium", "high", "xhigh", "max"];

#[derive(Debug, NsonDeserialize, NsonSerialize, PartialEq, Eq)]
pub struct RoleThinking {
    pub model: String,
    pub level: Option<String>,
}

#[derive(Debug, NsonDeserialize, NsonSerialize, PartialEq, Eq)]
pub struct Thinking {
    pub session: Option<String>,
    pub roles: BTreeMap<String, RoleThinking>,
}

#[derive(Default, NsonDeserialize)]
#[njson(deny_unknown_fields)]
pub struct ThinkingChanges {
    pub session: Option<String>,
    #[njson(default)]
    pub roles: BTreeMap<String, String>,
}

pub(super) fn single_model(value: &str) -> Option<RoleThinking> {
    if value.trim().is_empty() || value.contains(',') {
        return None;
    }
    let suffix = value
        .rsplit_once(':')
        .filter(|(model, level)| !model.is_empty() && (*level == "off" || LEVELS.contains(level)));
    let (model, level) = match suffix {
        Some((model, level)) => (model, Some(level.to_owned())),
        None => (value, None),
    };
    Some(RoleThinking {
        model: model.into(),
        level,
    })
}

fn thinking(root: &Mapping) -> Result<Thinking> {
    let session = root
        .get(&yaml::text("defaultThinkingLevel"))
        .map(|v| v.as_str().context("defaultThinkingLevel must be a string"))
        .transpose()?
        .map(str::to_owned);
    let roles = root
        .get(&yaml::text("modelRoles"))
        .map(|v| v.as_mapping().context("modelRoles must be a mapping"))
        .transpose()?;
    let roles = ROLES
        .iter()
        .filter_map(|&name| {
            Some((
                name.to_owned(),
                single_model(roles?.get(&yaml::text(name))?.as_str()?)?,
            ))
        })
        .collect();
    Ok(Thinking { session, roles })
}

pub fn read_thinking(home: &Path) -> Result<Thinking> {
    let (path, text) = source(home)?;
    thinking(&configuration(&text).with_context(|| format!("invalid {}", path.display()))?)
}

pub fn save_thinking(
    home: &Path,
    previous: Option<&Thinking>,
    changes: &ThinkingChanges,
) -> Result<Thinking> {
    if changes.session.is_none() && changes.roles.is_empty() {
        return read_thinking(home);
    }
    let _lock = super::super::omp_profiles::lock(home)?;
    let (path, text) = source(home)?;
    let mut expected =
        configuration(&text).with_context(|| format!("invalid {}", path.display()))?;
    let current = thinking(&expected)?;
    let yaml: YamlFile = text.parse()?;
    let doc = yaml.ensure_document();
    if let Some(level) = changes.session.as_deref() {
        ensure!(
            previous.is_none_or(|old| old.session == current.session),
            "session thinking level changed on disk; reopen the profile"
        );
        ensure!(
            level == "inherit" || LEVELS.contains(&level),
            "invalid session thinking level {level:?}"
        );
        if level == "inherit" {
            if expected
                .remove(&yaml::text("defaultThinkingLevel"))
                .is_some()
            {
                doc.remove("defaultThinkingLevel");
            }
        } else if current.session.as_deref() != Some(level) {
            expected.insert(yaml::text("defaultThinkingLevel"), yaml::text(level));
            doc.set("defaultThinkingLevel", level);
        }
    }
    for (name, level) in &changes.roles {
        ensure!(
            ROLES.contains(&name.as_str()),
            "unknown Oh My Pi thinking role {name:?}"
        );
        ensure!(
            level == "inherit" || level == "off" || LEVELS.contains(&level.as_str()),
            "invalid thinking level {level:?}"
        );
        let role = current.roles.get(name);
        ensure!(
            previous.is_none_or(|old| old.roles.get(name) == role),
            "{name} role changed on disk; reopen the profile"
        );
        let Some(role) = role else {
            ensure!(
                level == "inherit",
                "select a single model for the {name} role first"
            );
            continue;
        };
        let suffix = (level != "inherit").then_some(level.as_str());
        if role.level.as_deref() == suffix {
            continue;
        }
        let value = suffix.map_or_else(|| role.model.clone(), |s| format!("{}:{s}", role.model));
        doc.get_mapping("modelRoles")
            .context("cannot edit modelRoles in this YAML document")?
            .set(name.as_str(), value.as_str());
        expected
            .get_mut(&yaml::text("modelRoles"))
            .and_then(Value::as_mapping_mut)
            .context("invalid modelRoles")?
            .insert(yaml::text(name), yaml::text(value));
    }
    let updated = thinking(&expected)?;
    if updated != current {
        persist(home, &path, &text, &yaml, &expected)?;
    }
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn role_suffix_edits_preserve_model_tags_fallback_lists_and_external_edits() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let file = tmp.path().join("config.yaml");
        let original = "# personal\nmodelRoles:\n  slow: local/qwen3:8b # note\n  tiny: [a, b:high]\n  task: a,b:off\n";
        fs::write(&file, original)?;
        let old = read_thinking(tmp.path())?;
        assert_eq!(old.roles.len(), 1);
        let mut changes = ThinkingChanges {
            session: Some("high".into()),
            roles: BTreeMap::from([("slow".into(), "off".into())]),
        };
        let updated = save_thinking(tmp.path(), Some(&old), &changes)?;
        let text = fs::read_to_string(&file)?;
        assert!(text.contains("slow: local/qwen3:8b:off # note"));
        assert!(text.contains("tiny: [a, b:high]\n  task: a,b:off"));
        changes.roles.insert("tiny".into(), "max".into());
        assert!(save_thinking(tmp.path(), Some(&updated), &changes).is_err());
        assert_eq!(fs::read_to_string(&file)?, text);
        fs::write(&file, text.replace("local/qwen3:8b:off", "local/other:off"))?;
        changes.roles.remove("tiny");
        changes.session = Some("inherit".into());
        changes.roles.insert("slow".into(), "inherit".into());
        assert!(save_thinking(tmp.path(), Some(&updated), &changes).is_err());
        let reset = save_thinking(tmp.path(), None, &changes)?;
        assert_eq!(reset.session, None);
        assert_eq!(reset.roles["slow"].model, "local/other");
        assert_eq!(reset.roles["slow"].level, None);
        Ok(())
    }
}
