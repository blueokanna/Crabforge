use super::{Entry, Kind, Target, encode, path_key, validate_name};
use crate::{agents::NativeAgent, native_settings, rules::resolved_folder};
use anyhow::{Context, Result, ensure};
use regex::Regex;
use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
    sync::LazyLock,
};
use toml_edit::{DocumentMut, value};

const SUBAGENTS: &[&str] = &[
    "claude", "codex", "opencode", "gemini", "qwen", "droid", "grok", "cursor", "copilot",
];
const COMMANDS: &[&str] = &["claude", "opencode", "pi", "gemini", "qwen", "droid"];
const PROJECT_SUBAGENTS: &[&str] = &[
    "claude", "codex", "opencode", "gemini", "qwen", "droid", "cursor", "copilot", "agy",
];

pub(super) fn user_agents(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::Subagent => SUBAGENTS,
        Kind::Command => COMMANDS,
    }
}

pub(super) struct Profiles {
    pub(super) kind: Kind,
    pub(super) agents: &'static [&'static str],
    pub(super) homes: BTreeMap<&'static str, Vec<PathBuf>>,
    pub(super) project_scope: bool,
}

#[derive(Clone, Copy)]
pub(super) struct Format {
    pub(super) folder: &'static str,
    pub(super) suffix: &'static str,
}

impl Format {
    pub(super) fn name(self, file: &str) -> Option<&str> {
        file.strip_suffix(self.suffix)
    }
}

impl Profiles {
    pub(super) fn load(data: &Path, kind: Kind, project: Option<&Path>) -> Result<Self> {
        if let Some(project) = project {
            return Ok(Self {
                kind,
                agents: PROJECT_SUBAGENTS,
                project_scope: true,
                homes: BTreeMap::from([
                    ("claude", vec![project.join(".claude")]),
                    ("codex", vec![project.join(".codex")]),
                    ("opencode", vec![project.join(".opencode")]),
                    ("gemini", vec![project.join(".gemini")]),
                    ("qwen", vec![project.join(".qwen")]),
                    ("droid", vec![project.join(".factory")]),
                    ("cursor", vec![project.join(".cursor")]),
                    ("copilot", vec![project.join(".github")]),
                    ("agy", vec![project.join(".agents")]),
                ]),
            });
        }
        let home = env::home_dir().context("user home directory is unavailable")?;
        let setting = |key| {
            env::var_os(key)
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        };
        let profile = |key, default: &str| setting(key).unwrap_or_else(|| home.join(default));
        let claude = NativeAgent::Claude
            .home()
            .context("Claude Code profile is unavailable")?;
        let codex = NativeAgent::Codex
            .home()
            .context("Codex profile is unavailable")?;
        let opencode = profile("XDG_CONFIG_HOME", ".config").join("opencode");
        let gemini = setting("GEMINI_CLI_HOME")
            .unwrap_or_else(|| home.clone())
            .join(".gemini");
        let droid = setting("FACTORY_HOME_OVERRIDE")
            .unwrap_or_else(|| home.clone())
            .join(".factory");
        let homes = BTreeMap::from([
            ("claude", vec![claude]),
            ("codex", vec![codex]),
            ("opencode", vec![opencode]),
            (
                "pi",
                vec![
                    profile("PI_CODING_AGENT_DIR", ".pi/agent"),
                    data.join("agents/pi"),
                ],
            ),
            ("gemini", vec![gemini, data.join("agents/gemini/.gemini")]),
            (
                "qwen",
                vec![profile("QWEN_HOME", ".qwen"), data.join("agents/qwen")],
            ),
            ("droid", vec![droid]),
            ("grok", vec![profile("GROK_HOME", ".grok")]),
            ("cursor", vec![profile("CURSOR_HOME", ".cursor")]),
            (
                "copilot",
                vec![
                    profile("COPILOT_HOME", ".copilot"),
                    data.join("agents/copilot"),
                ],
            ),
        ]);
        Ok(Self {
            kind,
            agents: user_agents(kind),
            homes,
            project_scope: false,
        })
    }

    pub(super) fn formats(&self, agent: &str) -> impl Iterator<Item = Format> {
        let (folder, suffix) = match (self.kind, agent) {
            (Kind::Subagent, "codex") => ("agents", ".toml"),
            (Kind::Subagent, "droid") => ("droids", ".md"),
            (Kind::Subagent, "copilot") => ("agents", ".agent.md"),
            (Kind::Subagent, _) => ("agents", ".md"),
            (Kind::Command, "gemini") => ("commands", ".toml"),
            (Kind::Command, "pi") => ("prompts", ".md"),
            (Kind::Command, _) => ("commands", ".md"),
        };
        let alternative = match (self.kind, agent) {
            (Kind::Subagent, "opencode") => Some(("agent", ".md")),
            (Kind::Command, "opencode") => Some(("command", ".md")),
            (Kind::Command, "qwen") => Some(("commands", ".toml")),
            _ => None,
        };
        std::iter::once((folder, suffix))
            .chain(alternative)
            .map(|(folder, suffix)| Format { folder, suffix })
    }

    pub(super) fn target(
        &self,
        agent: &str,
        root: &Path,
        format: Format,
        name: &str,
    ) -> Result<Target> {
        validate_name(self.kind, name)?;
        let absolute = std::path::absolute(root)?;
        let root = resolved_folder(&absolute)?;
        if self.project_scope {
            ensure!(
                root == absolute,
                "project client directory resolves outside its declared path"
            );
        }
        let path = native_settings::path(
            &root
                .join(format.folder)
                .join(format!("{name}{}", format.suffix)),
        )?;
        ensure!(
            path.starts_with(&root) && path.parent() != Some(root.as_path()),
            "resource directory points outside the client profile"
        );
        let path = path
            .to_str()
            .context("resource destination is not UTF-8")?
            .to_owned();
        Ok(Target {
            agent: agent.to_owned(),
            name: name.to_owned(),
            path,
        })
    }

    pub(super) fn destinations<'a>(
        &'a self,
        agent: &'a str,
        name: &'a str,
    ) -> impl Iterator<Item = impl Iterator<Item = Result<Target>> + 'a> + 'a {
        self.homes
            .get(agent)
            .into_iter()
            .flatten()
            .map(move |root| {
                self.formats(agent)
                    .map(move |format| self.target(agent, root, format, name))
            })
    }

    pub(super) fn includes(&self, target: &Target) -> bool {
        self.agents.contains(&target.agent.as_str())
            && self
                .destinations(&target.agent, &target.name)
                .flatten()
                .filter_map(Result::ok)
                .any(|current| path_key(&current.path) == path_key(&target.path))
    }

    pub(super) fn render(&self, target: &Target, entry: &Entry) -> Result<String> {
        match self.kind {
            Kind::Subagent => subagent(&target.agent, entry),
            Kind::Command => command(&target.agent, target.path.ends_with(".toml"), entry),
        }
    }
}

fn markdown(fields: Vec<(&str, String)>, body: &str) -> String {
    let mut text = String::new();
    if !fields.is_empty() || body.lines().next() == Some("---") {
        text.push_str("---\n");
        for (key, value) in fields {
            text.push_str(key);
            text.push_str(": ");
            text.push_str(&value);
            text.push('\n');
        }
        text.push_str("---\n\n");
    }
    text.push_str(body);
    text
}

fn body(prompt: &str) -> String {
    let mut body = prompt.replace("\r\n", "\n");
    if !body.ends_with('\n') {
        body.push('\n');
    }
    body
}

fn subagent(agent: &str, entry: &Entry) -> Result<String> {
    let prompt = body(&entry.prompt);
    if agent == "codex" {
        let mut doc = DocumentMut::new();
        doc["name"] = value(&entry.name);
        doc["description"] = value(&entry.description);
        if let Some(model) = &entry.model {
            doc["model"] = value(model);
        }
        if let Some(effort) = &entry.effort {
            doc["model_reasoning_effort"] = value(effort);
        }
        doc["developer_instructions"] = value(prompt);
        return Ok(doc.to_string());
    }
    let mut fields = vec![("description", encode(&entry.description)?)];
    if agent == "opencode" {
        fields.push(("mode", "subagent".into()));
    } else {
        fields.push(("name", encode(&entry.name)?));
    }
    if agent == "claude" {
        if !entry.skills.is_empty() {
            fields.push(("skills", encode(&entry.skills)?));
        }
        if let Some(effort) = &entry.effort {
            fields.push(("effort", encode(effort)?));
        }
    }
    if matches!(agent, "claude" | "droid") && !entry.mcp.is_empty() {
        fields.push(("mcpServers", encode(&entry.mcp)?));
    }
    if agent == "agy" {
        ensure!(
            !matches!(
                entry.name.as_str(),
                "browser" | "research" | "deepagent" | "review-agent"
            ),
            "Antigravity reserves this subagent name"
        );
        fields.push(("subagent", "true".into()));
        fields.push(("mainAgent", "false".into()));
        fields.push((
            "tools",
            "[read_file, edit_file, write_file, run_command, manage_task, search_web]".into(),
        ));
    }
    Ok(markdown(fields, &prompt))
}

static POSITIONAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\$[0-9]+").unwrap());

pub(super) fn command(agent: &str, toml: bool, entry: &Entry) -> Result<String> {
    let mut prompt = body(&entry.prompt);
    let positions = POSITIONAL.find_iter(&prompt).filter(|token| {
        prompt.as_bytes()[..token.start()]
            .iter()
            .rev()
            .take_while(|&&byte| byte == b'\\')
            .count()
            % 2
            == 0
    });
    if matches!(agent, "gemini" | "qwen" | "droid") {
        ensure!(
            !positions
                .into_iter()
                .any(|token| token.as_str()[1..].bytes().any(|byte| byte != b'0')),
            "this client does not support positional slash command arguments"
        );
    } else if agent == "claude" {
        let mut expanded = String::new();
        let mut cursor = 0;
        for token in positions {
            expanded.push_str(&prompt[cursor..token.start()]);
            let position = token.as_str()[1..]
                .parse::<usize>()
                .context("slash command argument index is too large")?;
            if let Some(index) = position.checked_sub(1) {
                expanded.push_str(&format!("$ARGUMENTS[{index}]"));
            } else {
                expanded.push('\\');
                expanded.push_str(token.as_str());
            }
            cursor = token.end();
        }
        expanded.push_str(&prompt[cursor..]);
        prompt = expanded;
    }
    if matches!(agent, "gemini" | "qwen") {
        prompt = prompt.replace("$ARGUMENTS", "{{args}}");
    }
    if toml {
        let mut doc = DocumentMut::new();
        if !entry.description.is_empty() {
            doc["description"] = value(&entry.description);
        }
        doc["prompt"] = value(prompt);
        return Ok(doc.to_string());
    }
    let fields = if entry.description.is_empty() {
        Vec::new()
    } else {
        vec![("description", encode(&entry.description)?)]
    };
    Ok(markdown(fields, &prompt))
}
