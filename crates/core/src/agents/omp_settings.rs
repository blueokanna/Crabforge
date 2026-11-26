use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};
use yaml_edit::YamlFile;

mod registry;
mod rename;
pub(crate) use rename::rename_provider;
pub mod roles;
mod scope;
pub(super) use scope::sync_scope;
mod thinking;
pub use thinking::{Thinking, ThinkingChanges, read_thinking, save_thinking};

/// `inherit` removes an explicit choice from the profile.
#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
pub struct Advisor {
    pub review: String,
    pub wait: String,
    pub wake: String,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdvisorChanges {
    pub review: Option<String>,
    pub wait: Option<String>,
    pub wake: Option<String>,
}

fn source(home: &Path) -> Result<(PathBuf, String)> {
    for filename in ["config.yml", "config.yaml"] {
        let path = home.join(filename);
        match fs::read_to_string(&path) {
            Ok(text) => return Ok((path, text)),
            Err(e) if e.kind() == ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok((home.join("config.yml"), String::new()))
}

fn configuration(text: &str) -> Result<Mapping> {
    Ok(serde_yaml::from_str::<Option<Mapping>>(text)?.unwrap_or_default())
}

fn advisor(root: &Mapping) -> Result<Advisor> {
    let section = root
        .get("advisor")
        .map(|v| v.as_mapping().context("invalid advisor settings"))
        .transpose()?;
    let field = |name: &str| section.and_then(|m| m.get(name));
    let string = |name| -> Result<String> {
        Ok(field(name)
            .map(|v| {
                v.as_str()
                    .with_context(|| format!("advisor.{name} must be a string"))
            })
            .transpose()?
            .unwrap_or("inherit")
            .into())
    };
    let enabled = field("enabled")
        .map(|v| v.as_bool().context("advisor.enabled must be a boolean"))
        .transpose()?;
    let review = match enabled {
        None => "inherit",
        Some(false) => "off",
        Some(true) if field("reviewMode").and_then(Value::as_str) == Some("agent-end") => {
            "agent-end"
        }
        Some(true) => "turn",
    };
    Ok(Advisor {
        review: review.into(),
        wait: string("syncBacklog")?,
        wake: string("continueSeverity")?,
    })
}

pub fn read_advisor(home: &Path) -> Result<Advisor> {
    let (path, text) = source(home)?;
    advisor(&configuration(&text).with_context(|| format!("invalid {}", path.display()))?)
}

impl AdvisorChanges {
    fn fields(&self) -> Result<Vec<(&'static str, Option<Value>)>> {
        let mut fields = Vec::new();
        if let Some(value) = self.review.as_deref() {
            let enabled = match value {
                "inherit" => None,
                "off" => Some(false),
                "turn" | "agent-end" => Some(true),
                _ => anyhow::bail!("invalid advisor review mode {value:?}"),
            };
            fields.push(("enabled", enabled.map(Value::Bool)));
            if enabled == Some(true) {
                fields.push(("reviewMode", Some(value.into())));
            }
        }
        for (name, value, options) in [
            (
                "syncBacklog",
                self.wait.as_deref(),
                &["off", "1", "3", "5", "strict"][..],
            ),
            (
                "continueSeverity",
                self.wake.as_deref(),
                &["nit", "concern", "blocker"][..],
            ),
        ] {
            if let Some(value) = value {
                ensure!(
                    value == "inherit" || options.contains(&value),
                    "invalid advisor.{name} {value:?}"
                );
                fields.push((name, (value != "inherit").then(|| Value::from(value))));
            }
        }
        Ok(fields)
    }
}

pub fn save_advisor(
    home: &Path,
    previous: Option<&Advisor>,
    changes: &AdvisorChanges,
) -> Result<Advisor> {
    let fields = changes.fields()?;
    if fields.is_empty() {
        return read_advisor(home);
    }
    let _lock = super::omp_profiles::lock(home)?;
    let (path, text) = source(home)?;
    let mut expected =
        configuration(&text).with_context(|| format!("invalid {}", path.display()))?;
    let current = advisor(&expected)?;
    ensure!(
        previous.is_none_or(|old| old == &current),
        "advisor settings changed on disk; reopen the profile"
    );
    if fields
        .iter()
        .all(|(key, value)| expected.get("advisor").and_then(|v| v.get(*key)) == value.as_ref())
    {
        if changes.review.is_some() {
            roles::forget_advisor_enable(home)?;
        }
        return Ok(current);
    }
    let yaml: YamlFile = text.parse()?;
    let doc = yaml.ensure_document();
    for (key, value) in fields {
        if expected.get("advisor").and_then(|v| v.get(key)) == value.as_ref() {
            continue;
        }
        let expected_section = expected
            .entry("advisor".into())
            .or_insert_with(|| Value::Mapping(Mapping::new()))
            .as_mapping_mut()
            .context("invalid advisor settings")?;
        if doc.get_mapping("advisor").is_none() {
            ensure!(
                doc.get("advisor").is_none() && doc.set("advisor", yaml_edit::Mapping::new()),
                "cannot edit advisor in this YAML document"
            );
        }
        let section = doc
            .get_mapping("advisor")
            .context("invalid advisor mapping")?;
        match &value {
            Some(Value::Bool(b)) => section.set(key, *b),
            Some(Value::String(s)) => section.set(key, s.as_str()),
            None => {
                section.remove(key);
            }
            _ => unreachable!(),
        }
        if let Some(v) = value {
            expected_section.insert(key.into(), v);
        } else {
            expected_section.remove(key);
        }
        if expected_section.is_empty() {
            expected.remove("advisor");
            doc.remove("advisor");
        }
    }
    let result = advisor(&expected)?;
    persist(home, &path, &text, &yaml, &expected)?;
    if changes.review.is_some() {
        roles::forget_advisor_enable(home)?;
    }
    Ok(result)
}

fn persist(
    home: &Path,
    path: &Path,
    text: &str,
    yaml: &YamlFile,
    expected: &Mapping,
) -> Result<()> {
    let rendered = yaml.to_string();
    if rendered != text {
        ensure!(
            configuration(&rendered)? == *expected,
            "edited YAML did not preserve the profile settings"
        );
        let mut file = tempfile::NamedTempFile::new_in(home)?;
        file.write_all(rendered.as_bytes())?;
        file.as_file().sync_all()?;
        ensure!(
            source(home)? == (path.to_owned(), text.to_owned()),
            "Oh My Pi config changed while saving"
        );
        file.persist(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advisor_changes_preserve_comments_types_and_the_previous_review_mode() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("config.yaml");
        fs::write(
            &path,
            "# personal\nmodelRoles: {tiny: local/m} # keep\nadvisor:\n  other: true # retained\n",
        )?;
        let old = read_advisor(dir.path())?;
        let all = AdvisorChanges {
            review: Some("agent-end".into()),
            wait: Some("1".into()),
            wake: Some("blocker".into()),
        };
        let result = save_advisor(dir.path(), Some(&old), &all)?;
        let text = fs::read_to_string(&path)?;
        assert!(
            text.contains("# personal\nmodelRoles: {tiny: local/m} # keep")
                && text.contains("other: true # retained")
        );
        assert_eq!(
            configuration(&text)?["advisor"]["syncBacklog"].as_str(),
            Some("1")
        );
        assert!(!dir.path().join("config.yml").exists());
        assert!(save_advisor(dir.path(), Some(&old), &all).is_err());
        let off = AdvisorChanges {
            review: Some("off".into()),
            ..Default::default()
        };
        save_advisor(dir.path(), Some(&result), &off)?;
        let clear = AdvisorChanges {
            review: Some("inherit".into()),
            wait: Some("inherit".into()),
            wake: Some("inherit".into()),
        };
        assert_eq!(save_advisor(dir.path(), None, &clear)?, old);
        let config = configuration(&fs::read_to_string(path)?)?;
        assert_eq!(config["advisor"]["reviewMode"].as_str(), Some("agent-end"));
        Ok(())
    }

    #[test]
    fn advisor_accepts_new_and_comment_only_profiles_and_rejects_bad_input() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file = dir.path().join("config.yml");
        let changes = AdvisorChanges {
            wait: Some("3".into()),
            ..Default::default()
        };
        save_advisor(dir.path(), None, &changes)?;
        for original in ["# my settings\n", "enabledModels: []\n"] {
            fs::write(&file, original)?;
            assert_eq!(save_advisor(dir.path(), None, &changes)?.wait, "3");
            assert!(fs::read_to_string(&file)?.contains(original));
        }
        fs::write(&file, "advisor: [oops]\n")?;
        assert!(save_advisor(dir.path(), None, &changes).is_err());
        assert_eq!(fs::read_to_string(file)?, "advisor: [oops]\n");
        Ok(())
    }
}
