use super::{Bundle, MAX_MANIFEST, PI, Tree, component_file, display, relative, valid_name};
use anyhow::{Context, Result, ensure};
use nextjson::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

const MATCH: glob::MatchOptions = glob::MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
};

enum Selector {
    Path(PathBuf),
    Glob(glob::Pattern),
}

impl Selector {
    fn parse(input: &str) -> Result<Self> {
        let input = input.trim_start_matches("./");
        if input.contains(['*', '?', '[']) {
            ensure!(
                !input.contains(['\\', ':'])
                    && input
                        .split('/')
                        .all(|part| !matches!(part, "" | "." | "..")),
                "invalid Pi resource pattern"
            );
            return Ok(Self::Glob(glob::Pattern::new(input)?));
        }
        Ok(Self::Path(
            relative(input).context("invalid Pi resource path")?,
        ))
    }

    fn matches(&self, path: &Path) -> bool {
        match self {
            Self::Path(root) => path.starts_with(root),
            Self::Glob(pattern) => path
                .ancestors()
                .take_while(|part| !part.as_os_str().is_empty())
                .any(|part| pattern.matches_with(&display(part), MATCH)),
        }
    }
}

fn text<'a>(doc: &'a Value, field: &str) -> &'a str {
    doc.get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
}

pub(super) fn inspect(tree: &Tree, source: String) -> Result<Bundle> {
    let package = tree.text(Path::new(PI))?.context(
        "no Claude Code, Codex, Antigravity or Pi plugin manifest in the selected folder",
    )?;
    ensure!(
        package.len() <= MAX_MANIFEST,
        "Pi package manifest exceeds 256 KiB"
    );
    let doc: Value = nextjson::from_str(package.strip_prefix('\u{feff}').unwrap_or(package))
        .map_err(anyhow::Error::msg)
        .context("read Pi package.json")?;
    let config = doc
        .get("pi")
        .filter(|pi| pi.is_object())
        .context("Pi package.json needs a pi object")?;
    let package_name = text(&doc, "name");
    let name = if let Some((scope, name)) = package_name
        .strip_prefix('@')
        .and_then(|name| name.split_once('/'))
    {
        ensure!(
            valid_name(scope) && valid_name(name),
            "invalid scoped Pi package name"
        );
        format!("{scope}--{name}")
    } else {
        package_name.to_owned()
    };
    ensure!(valid_name(&name), "invalid Pi package name");
    let files = tree.files().collect::<Vec<_>>();
    let mut components = BTreeMap::new();
    let mut other = Vec::new();
    for (field, kind) in [
        ("skills", "skills"),
        ("prompts", "commands"),
        ("extensions", ""),
        ("themes", ""),
    ] {
        let Some(value) = config.get(field) else {
            continue;
        };
        let Some(values) = value.as_array() else {
            other.push(format!("{PI} / pi.{field}"));
            continue;
        };
        let mut include = Vec::new();
        let mut exclude = Vec::new();
        for entry in values.iter().filter_map(Value::as_str) {
            if kind.is_empty() || entry.starts_with('+') {
                other.push(format!("{PI} / pi.{field} / {entry}"));
                continue;
            }
            let (path, selectors) = if let Some(path) = entry.strip_prefix(['!', '-']) {
                (path, &mut exclude)
            } else {
                (entry, &mut include)
            };
            match Selector::parse(path) {
                Ok(selector) => selectors.push(selector),
                Err(_) => other.push(format!("{PI} / pi.{field} / {entry}")),
            }
        }
        let paths = files
            .iter()
            .filter(|path| {
                component_file(kind, path)
                    && (kind != "commands"
                        || path
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("md")))
                    && include.iter().any(|selector| selector.matches(path))
                    && !exclude.iter().any(|selector| selector.matches(path))
            })
            .map(|path| display(path))
            .collect::<Vec<_>>();
        if !paths.is_empty() {
            components.insert(kind.to_owned(), paths);
        }
    }
    Ok(Bundle {
        name,
        description: text(&doc, "description").to_owned(),
        version: text(&doc, "version").to_owned(),
        source,
        content: tree.hash(),
        manifests: vec![PI.to_owned()],
        components,
        other_manifest_entries: other,
        files: files.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_bundles::{Edit, edit};
    use std::fs;

    #[test]
    fn reviews_declared_pi_resources_and_copies_prompt_arguments() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().join("bundle");
        for folder in ["tools/check", "tools/hidden", "templates", "extensions"] {
            fs::create_dir_all(root.join(folder))?;
        }
        let manifest = root.join(PI);
        fs::write(
            &manifest,
            r#"{"name":"@team/reviewer","version":"1.0.0","pi":{"skills":["./tools/**","!tools/hidden"],"prompts":["templates/*.md"],"extensions":["extensions/ready.ts"]}}"#,
        )?;
        fs::write(
            root.join("tools/check/SKILL.md"),
            "---\nname: check\ndescription: Check the output\n---\nCheck it.\n",
        )?;
        fs::write(root.join("tools/hidden/SKILL.md"), "Private fixture")?;
        fs::write(root.join("templates/review.md"), "Review $1 with $@.")?;
        fs::write(
            root.join("extensions/ready.ts"),
            "export default function ready() {}",
        )?;
        let source = manifest.to_str().context("fixture path must be UTF-8")?;
        let (bundle, _) = super::super::load(source)?;
        assert_eq!(bundle.name, "team--reviewer");
        assert_eq!(bundle.components["skills"], ["tools/check/SKILL.md"]);
        assert_eq!(bundle.components["commands"], ["templates/review.md"]);
        assert_eq!(
            bundle.other_manifest_entries,
            ["package.json / pi.extensions / extensions/ready.ts"]
        );
        let data = temp.path().join("data");
        let output = edit(
            &data,
            Edit::Save {
                source,
                content: &bundle.content,
                revision: None,
            },
        )?;
        let saved: Value = nextjson::from_str(&output).map_err(anyhow::Error::msg)?;
        let revision = saved[0]["revision"].as_str().context("missing revision")?;
        let output = edit(
            &data,
            Edit::Resources {
                name: &bundle.name,
                revision,
                kind: "commands",
                path: "templates/review.md",
                copy: Some("templates/review.md"),
            },
        )?;
        let copied: Value = nextjson::from_str(&output).map_err(anyhow::Error::msg)?;
        assert_eq!(
            copied["definition"]["prompt"].as_str(),
            Some("Review $1 with $ARGUMENTS.")
        );
        Ok(())
    }
}
