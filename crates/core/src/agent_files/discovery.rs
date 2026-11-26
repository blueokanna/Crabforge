use super::{Entry, Kind, Store, Target, definition, encode, hash, native::Profiles, path_key};
use crate::native_settings;
use anyhow::{Context, Result, ensure};
use nextjson::NsonSerialize;
use regex::{Captures, Regex};
use saphyr::{LoadableYamlNode, YamlOwned};
use std::{collections::BTreeMap, fs, io::ErrorKind, path::Path, sync::LazyLock};
use toml_edit::{DocumentMut, Item};

const MAX_CANDIDATES: usize = 512;

#[derive(NsonSerialize)]
struct Candidate {
    agent: String,
    name: String,
    path: String,
    revision: Option<String>,
    description: Option<String>,
    status: &'static str,
    error: Option<String>,
}

#[derive(NsonSerialize)]
struct Discovery {
    candidates: Vec<Candidate>,
    truncated: bool,
}

fn compatible(store: &Store, entry: &Entry) -> Result<bool> {
    let Some(saved) = store.entries.get(&entry.name) else {
        return Ok(true);
    };
    Ok(definition(saved)? == definition(entry)?)
}

pub(super) fn list(profiles: &Profiles, store: &Store) -> Result<String> {
    let mut candidates = BTreeMap::new();
    let mut truncated = false;
    'profiles: for &agent in profiles.agents {
        for root in profiles.homes.get(agent).into_iter().flatten() {
            for format in profiles.formats(agent) {
                let folder = profiles.target(agent, root, format, "crabforge-discovery")?;
                let folder = Path::new(&folder.path)
                    .parent()
                    .context("missing resource directory")?;
                let files = match fs::read_dir(folder) {
                    Ok(files) => files,
                    Err(error) if error.kind() == ErrorKind::NotFound => continue,
                    Err(error) => return Err(error.into()),
                };
                for file in files {
                    let file = file?;
                    if file.file_type()?.is_dir() {
                        continue;
                    }
                    let Some(name) = file
                        .file_name()
                        .to_str()
                        .and_then(|file| format.name(file).map(str::to_owned))
                    else {
                        continue;
                    };
                    let path = file.path();
                    let Some(path) = path.to_str() else { continue };
                    let key = (name.clone(), agent, path_key(path));
                    if candidates.contains_key(&key) {
                        continue;
                    }
                    let data = (|| -> Result<_> {
                        let target = profiles.target(agent, root, format, &name)?;
                        let text = native_settings::read(Path::new(&target.path))?
                            .context("resource file was removed")?;
                        let checksum = hash(&text);
                        if store
                            .applied
                            .get(&key.2)
                            .is_some_and(|record| record.hash == checksum)
                        {
                            return Ok(None);
                        }
                        let entry = parse(profiles, &target, &text)?;
                        let status = if compatible(store, &entry)? {
                            "available"
                        } else {
                            "conflict"
                        };
                        Ok(Some((checksum, entry.description, status)))
                    })();
                    let (revision, description, status, error) = match data {
                        Ok(None) => continue,
                        Ok(Some((revision, description, status))) => {
                            (Some(revision), Some(description), status, None)
                        }
                        Err(error) => (None, None, "error", Some(format!("{error:#}"))),
                    };
                    if candidates.len() == MAX_CANDIDATES {
                        truncated = true;
                        break 'profiles;
                    }
                    candidates.insert(
                        key,
                        Candidate {
                            agent: agent.to_owned(),
                            name,
                            path: path.to_owned(),
                            revision,
                            description,
                            status,
                            error,
                        },
                    );
                }
            }
        }
    }
    encode(&Discovery {
        candidates: candidates.into_values().collect(),
        truncated,
    })
}

pub(super) fn adopt(
    profiles: &Profiles,
    store: &mut Store,
    agent: &str,
    path: &str,
    revision: &str,
) -> Result<()> {
    ensure!(
        profiles.agents.contains(&agent),
        "unsupported agent assignment"
    );
    let filename = Path::new(path)
        .file_name()
        .and_then(|part| part.to_str())
        .context("invalid native filename")?;
    let key = path_key(path);
    let target = profiles
        .homes
        .get(agent)
        .into_iter()
        .flatten()
        .find_map(|root| {
            profiles.formats(agent).find_map(|format| {
                let name = format.name(filename)?;
                let target = profiles.target(agent, root, format, name).ok()?;
                (path_key(&target.path) == key).then_some(target)
            })
        })
        .context("file is outside the current client profile")?;
    let original =
        native_settings::read(Path::new(&target.path))?.context("resource file was removed")?;
    ensure!(
        hash(&original) == revision,
        "resource file changed; scan again"
    );
    let entry = parse(profiles, &target, &original)?;
    ensure!(compatible(store, &entry)?, "another resource has that name");
    let definition = definition(&entry)?;
    let saved = store.entries.entry(entry.name.clone()).or_insert(entry);
    if !saved.agents.iter().any(|id| id == agent) {
        saved.agents.push(agent.to_owned());
        saved.agents.sort_unstable();
    }
    store.own(target, Some(revision.to_owned()));
    store.adopted.insert(
        key,
        super::Adopted {
            original,
            definition,
        },
    );
    Ok(())
}

fn parse(profiles: &Profiles, target: &Target, original: &str) -> Result<Entry> {
    let source = original
        .strip_prefix('\u{feff}')
        .unwrap_or(original)
        .replace("\r\n", "\n");
    let mut entry = Entry {
        name: target.name.clone(),
        description: String::new(),
        prompt: String::new(),
        agents: vec![target.agent.clone()],
        model: None,
        effort: None,
        skills: Vec::new(),
        mcp: Vec::new(),
    };
    if target.path.ends_with(".toml") {
        let doc: DocumentMut = source.parse()?;
        let string = |key: &str| doc.get(key).and_then(Item::as_str).map(str::to_owned);
        entry.description = string("description").unwrap_or_default();
        entry.prompt = match profiles.kind {
            Kind::Command => string("prompt"),
            Kind::Subagent => string("developer_instructions"),
        }
        .context("missing prompt in TOML")?;
        if profiles.kind == Kind::Subagent {
            entry.model = string("model");
            entry.effort = string("model_reasoning_effort");
        }
    } else {
        let (front, body) = if let Some(front) = source.strip_prefix("---\n") {
            if let Some(body) = front.strip_prefix("---\n") {
                ("", body)
            } else {
                front
                    .split_once("\n---\n")
                    .context("unclosed Markdown front matter")?
            }
        } else {
            ("", source.as_str())
        };
        let docs = YamlOwned::load_from_str(front).map_err(anyhow::Error::msg)?;
        let root = docs.first();
        ensure!(
            docs.len() <= 1 && root.is_none_or(YamlOwned::is_mapping),
            "expected a front matter mapping"
        );
        let field = |key| root.and_then(|root| root.as_mapping_get(key));
        let string = |key| field(key).and_then(YamlOwned::as_str).map(str::to_owned);
        let strings = |key| match field(key) {
            Some(value) => value
                .as_vec()
                .into_iter()
                .flatten()
                .filter_map(YamlOwned::as_str)
                .map(str::to_owned)
                .collect(),
            None => Vec::new(),
        };
        entry.description = string("description").unwrap_or_default();
        entry.prompt = body.to_owned();
        if profiles.kind == Kind::Subagent && target.agent == "claude" {
            entry.effort = string("effort");
            entry.skills = strings("skills");
        }
        if profiles.kind == Kind::Subagent && matches!(target.agent.as_str(), "claude" | "droid") {
            entry.mcp = strings("mcpServers");
        }
    }
    entry.prompt = entry.prompt.trim_matches('\n').to_owned();
    if profiles.kind == Kind::Command {
        entry.prompt = portable_arguments(&target.agent, &entry.prompt);
    }
    entry.validate(profiles.kind, profiles.agents)?;
    Ok(entry)
}

static CLAUDE_ARGUMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\\$[0-9]+|\$ARGUMENTS\[([0-9]+)\]|\$([0-9]+)").unwrap());

fn portable_arguments(agent: &str, prompt: &str) -> String {
    match agent {
        "claude" => CLAUDE_ARGUMENT
            .replace_all(prompt, |parts: &Captures<'_>| {
                let token = &parts[0];
                parts
                    .get(1)
                    .or_else(|| parts.get(2))
                    .and_then(|position| position.as_str().parse::<usize>().ok())
                    .and_then(|position| position.checked_add(1))
                    .map(|position| format!("${position}"))
                    .unwrap_or_else(|| {
                        token
                            .strip_prefix("\\$0")
                            .filter(|rest| rest.is_empty())
                            .map_or_else(|| token.to_owned(), |_| "$0".into())
                    })
            })
            .into_owned(),
        "pi" => prompt.replace("$@", "$ARGUMENTS"),
        "gemini" | "qwen" => prompt.replace("{{args}}", "$ARGUMENTS"),
        _ => prompt.to_owned(),
    }
}
