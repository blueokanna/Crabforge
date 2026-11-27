use super::session::Caller;
use crate::{
    config::{
        Config, ResolvedGroups,
        groups::rules::{
            Clock, EffortTest, Predicate, RoutingRule, TextField, TextTest, agent_key, effort_rank,
            folder_key,
        },
    },
    translate::Format,
};
use anyhow::Result;
use http::HeaderMap;
use nextjson::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::LazyLock,
};

pub(super) fn compile(
    config: &Config,
    resolved: &ResolvedGroups,
) -> Result<BTreeMap<String, Vec<Rule>>> {
    config
        .group_rules
        .iter()
        .filter(|(_, rules)| !rules.is_empty())
        .map(|(group, rules)| {
            let names = resolved
                .expanded
                .get(group)
                .unwrap_or(&config.groups[group]);
            let canonical = &resolved.members[group];
            let mapping = names
                .iter()
                .map(String::as_str)
                .zip(canonical)
                .collect::<HashMap<_, _>>();
            let rules = rules
                .iter()
                .enumerate()
                .map(|(index, rule)| {
                    let targets = rule
                        .targets()
                        .map(|target| {
                            mapping
                                .get(target)
                                .map_or_else(|| target.to_owned(), |id| (*id).clone())
                        })
                        .collect();
                    Rule::compile(index, rule, targets)
                })
                .collect::<Result<_>>()?;
            Ok((group.clone(), rules))
        })
        .collect()
}

pub(super) struct Order<'a> {
    pub members: Vec<(bool, usize, &'a str)>,
    pub primary: Option<&'a Rule>,
    pub grew: bool,
}

pub(super) fn ordered<'a>(
    members: &'a [String],
    rules: &'a [Rule],
    facts: &Facts,
    intent: Option<&str>,
    held: Option<Option<usize>>,
    context: impl Fn(&str) -> Option<u32>,
) -> Order<'a> {
    let key = intent.map(str::to_lowercase);
    let matching = rules
        .iter()
        .filter(|rule| {
            (rule.intent.is_empty() || key.as_deref() == Some(rule.intent_key.as_str()))
                && rule.matches(facts)
        })
        .collect::<Vec<_>>();
    let paused = matching
        .iter()
        .filter(|rule| rule.pause)
        .flat_map(|rule| rule.targets.iter().map(String::as_str))
        .collect::<HashSet<_>>();
    let live_target =
        |target: &String| !paused.contains(target.as_str()) && members.contains(target);
    let available = |rule: &&Rule| !rule.pause && rule.targets.iter().any(live_target);
    let mut matching = matching.into_iter().filter(available).collect::<Vec<_>>();
    let mut grew = false;
    if let Some(saved) = held {
        let pinned = saved.and_then(|index| rules.get(index));
        let limit = pinned.map_or_else(
            || {
                members
                    .iter()
                    .filter(|id| live_target(id))
                    .filter_map(|id| context(id))
                    .min()
            },
            |rule| rule.targets.first().and_then(|id| context(id)),
        );
        grew = limit.is_some_and(|limit| {
            facts.tokens as u64 * 100 >= u64::from(limit) * 95
                && matching.first().is_some_and(|rule| {
                    Some(rule.index) != saved
                        && rule
                            .targets
                            .iter()
                            .find(|target| live_target(target))
                            .and_then(|id| context(id))
                            .is_some_and(|next| next > limit && next as usize >= facts.tokens)
                })
        });
        if !grew {
            match pinned {
                Some(rule) if available(&rule) => matching.insert(0, rule),
                None => matching.clear(),
                _ => {}
            }
        }
    }
    let primary = matching.first().copied();
    let mut ranked = HashMap::new();
    for rule in matching {
        for target in rule.targets.iter().map(String::as_str) {
            if paused.contains(target) {
                continue;
            }
            let rank = ranked.len();
            ranked.entry(target).or_insert(rank);
        }
    }
    let mut order = members
        .iter()
        .map(String::as_str)
        .enumerate()
        .filter(|(_, member)| !paused.contains(member))
        .collect::<Vec<_>>();
    order.sort_by_key(|(_, member)| ranked.get(member).copied().unwrap_or(usize::MAX));
    Order {
        members: order
            .into_iter()
            .map(|(index, member)| (ranked.contains_key(member), index, member))
            .collect(),
        primary,
        grew,
    }
}

pub(super) fn intents<'a>(rules: &'a [Rule], facts: &Facts) -> Vec<&'a str> {
    let mut seen = HashSet::new();
    rules
        .iter()
        .filter(|rule| {
            !rule.intent.is_empty() && rule.matches(facts) && seen.insert(&rule.intent_key)
        })
        .map(|rule| rule.intent.as_str())
        .collect()
}

pub(super) struct Rule {
    pub index: usize,
    pub targets: Vec<String>,
    pub pause: bool,
    pub use_effort: String,
    min_tokens: usize,
    images: bool,
    effort: String,
    intent: String,
    intent_key: String,
    compact: bool,
    agents: Vec<String>,
    folders: Vec<String>,
    except: Vec<String>,
    clock: Option<Clock>,
    conditions: Vec<Predicate>,
}

impl Rule {
    pub fn compile(index: usize, rule: &RoutingRule, targets: Vec<String>) -> Result<Self> {
        let intent = rule.intent.split_whitespace().collect::<Vec<_>>().join(" ");
        Ok(Self {
            index,
            targets,
            pause: rule.pause,
            use_effort: rule.use_effort.clone(),
            min_tokens: rule.tokens as usize,
            images: rule.images,
            effort: rule.effort.clone(),
            intent_key: intent.to_lowercase(),
            intent,
            compact: rule.compact,
            agents: rule.agents.iter().map(|a| agent_key(a)).collect(),
            folders: rule.folders.iter().filter_map(|p| folder_key(p)).collect(),
            except: rule
                .except_folders
                .iter()
                .filter_map(|p| folder_key(p))
                .collect(),
            clock: rule.time.as_ref().map(|w| w.compile()).transpose()?,
            conditions: rule
                .conditions
                .iter()
                .map(|c| c.compile())
                .collect::<Result<_>>()?,
        })
    }

    pub fn matches(&self, facts: &Facts) -> bool {
        let under = |root: &String| {
            facts.folder.as_ref().is_some_and(|folder| {
                folder == root
                    || root == "/"
                    || folder
                        .strip_prefix(root)
                        .is_some_and(|tail| tail.starts_with('/'))
            })
        };
        facts.tokens >= self.min_tokens
            && (!self.images || facts.images)
            && (!self.compact || facts.compact)
            && match self.effort.as_str() {
                "" => true,
                "on" => facts.reasoning,
                level => facts.effort >= effort_rank(level),
            }
            && (self.agents.is_empty() || self.agents.contains(&facts.agent))
            && self.clock.as_ref().is_none_or(|c| c.contains(facts.at))
            && (self.folders.is_empty() || self.folders.iter().any(under))
            && !self.except.iter().any(under)
            && self.conditions.iter().all(|test| facts.matches(test))
    }
}

pub(super) struct Facts {
    at: jiff::Timestamp,
    tokens: usize,
    images: bool,
    reasoning: bool,
    effort: Option<usize>,
    agent: String,
    compact: bool,
    subagent: bool,
    folder: Option<String>,
    message: String,
    previous_message: String,
    previous_reply: String,
    turn_identity: Option<[u8; 32]>,
    continued: bool,
    system: Vec<String>,
    last: Vec<String>,
}

impl Facts {
    pub(super) fn requests_reasoning(&self) -> bool {
        self.reasoning
    }

    pub fn read(
        format: Format,
        bytes: &[u8],
        caller: &Caller<'_>,
        headers: &HeaderMap,
    ) -> Option<Self> {
        let body: Value = nextjson::from_slice(bytes).ok()?;
        let (reasoning, effort) = reasoning(format, &body);
        let mut facts = Self {
            at: jiff::Timestamp::now(),
            tokens: 0,
            images: false,
            reasoning,
            effort,
            agent: agent_key(caller.agent.as_deref().unwrap_or_default()),
            compact: false,
            subagent: [
                "x-magpie-subagent",
                "x-crabforge-subagent",
                "x-codex-subagent",
                "x-claude-code-agent-id",
                "x-claude-code-parent-agent-id",
            ]
            .iter()
            .any(|name| {
                headers
                    .get(*name)
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|v| !v.trim().is_empty())
            }),
            folder: None,
            message: String::new(),
            previous_message: String::new(),
            previous_reply: String::new(),
            turn_identity: None,
            continued: false,
            system: Vec::new(),
            last: Vec::new(),
        };
        let mut length = 0;
        let top_system = match format {
            Format::Chat => None,
            Format::Messages => body.get("system"),
            Format::Responses => body.get("instructions"),
            Format::Gemini => body.pointer("/systemInstruction/parts"),
        };
        if let Some(system) = top_system {
            let mut text = Vec::new();
            collect_text(system, &mut text, &mut facts.images);
            length += text.iter().map(|s| s.len()).sum::<usize>();
            facts.extend_system(text);
        }
        let key = match format {
            Format::Chat | Format::Messages => "messages",
            Format::Responses => "input",
            Format::Gemini => "contents",
        };
        let mut turn_hash = caller.session.map(|session| {
            let mut hash = Sha256::new();
            hash_text(&mut hash, session);
            hash_text(&mut hash, &facts.agent);
            hash
        });
        if let Some(input) = body.get(key) {
            if let Some(prompt) = input.as_str() {
                length += prompt.len();
                facts.message = prompt.to_owned();
                facts.last.push(prompt.to_owned());
                if let Some(hash) = &mut turn_hash {
                    hash_text(hash, "");
                    hash_text(hash, prompt);
                }
            } else if let Some(turns) = input.as_array() {
                let mut reply = String::new();
                for turn in turns {
                    let role = field(turn, "role").unwrap_or("user");
                    let mut text = Vec::new();
                    let content = turn.get(if format == Format::Gemini {
                        "parts"
                    } else {
                        "content"
                    });
                    if let Some(content) = content {
                        collect_text(content, &mut text, &mut facts.images);
                    }
                    length +=
                        text.iter().map(|s| s.len()).sum::<usize>() + tool_length(turn, content);
                    if matches!(role, "system" | "developer") {
                        facts.extend_system(text);
                    } else if tool_result(turn, role, content) {
                        facts.continued = !facts.message.is_empty();
                    } else if role == "user"
                        && (turn.get("type").is_none() || field(turn, "type") == Some("message"))
                    {
                        for text in &text {
                            if let Some(folder) = declared_folder(text, false) {
                                facts.folder = Some(folder);
                            }
                        }
                        let said = text
                            .iter()
                            .copied()
                            .filter(|text| !is_context(text))
                            .collect::<Vec<_>>();
                        if !said.is_empty() {
                            facts.continued = false;
                            let message = said.join("\n");
                            if let Some(hash) = &mut turn_hash {
                                hash_text(hash, &reply);
                                hash_text(hash, &message);
                            }
                            facts.previous_message = std::mem::replace(&mut facts.message, message);
                            facts.previous_reply = std::mem::take(&mut reply);
                        }
                        facts.last = text.into_iter().map(str::to_owned).collect();
                    } else if matches!(role, "assistant" | "model") {
                        facts.continued = false;
                        if !text.is_empty() {
                            reply = text.join("\n");
                        }
                    }
                }
            }
        }
        if let Some(tools) = body.get("tools").and_then(Value::as_array) {
            length += tools
                .iter()
                .map(|tool| tool.to_string().len())
                .sum::<usize>();
        }
        facts.tokens = length / 4;
        facts.compact = compacting(&facts.system, &facts.last);
        if !facts.compact {
            facts.turn_identity = turn_hash.map(|hash| hash.finalize().into());
        }
        Some(facts)
    }

    pub fn conversation(&self) -> (&str, &str, &str) {
        (&self.message, &self.previous_message, &self.previous_reply)
    }

    pub fn turn_identity(&self) -> Option<[u8; 32]> {
        self.turn_identity
    }

    pub fn affinity_turn(&self) -> Option<[u8; 32]> {
        self.turn_identity
            .filter(|_| !self.message.trim().is_empty())
    }

    pub fn continued(&self) -> bool {
        self.continued
    }

    fn extend_system(&mut self, text: Vec<&str>) {
        for text in text {
            if let Some(folder) = declared_folder(text, true) {
                self.folder = Some(folder);
            }
            self.system.push(text.to_owned());
        }
    }

    fn matches(&self, predicate: &Predicate) -> bool {
        match predicate {
            Predicate::Tokens(op, n) => op.matches(self.tokens, *n),
            Predicate::Effort(op, level) => {
                let same = match level.as_str() {
                    "on" => self.reasoning,
                    "off" => !self.reasoning,
                    level => self.effort == effort_rank(level),
                };
                match op {
                    EffortTest::Is => same,
                    EffortTest::Not => !same,
                    EffortTest::AtLeast => self.effort >= effort_rank(level),
                    EffortTest::AtMost => {
                        !self.reasoning
                            || self
                                .effort
                                .is_some_and(|actual| Some(actual) <= effort_rank(level))
                    }
                }
            }
            Predicate::Agent(yes, agents) => agents.contains(&self.agent) == *yes,
            Predicate::Subagent(yes) => self.subagent == *yes,
            Predicate::Text(field, test) => {
                let texts = match field {
                    TextField::Message => std::slice::from_ref(&self.message),
                    TextField::System => &self.system,
                    TextField::Last => &self.last,
                };
                match test {
                    TextTest::Contains(value) => {
                        texts.iter().any(|s| s.to_lowercase().contains(value))
                    }
                    TextTest::Excludes(value) => {
                        !texts.iter().any(|s| s.to_lowercase().contains(value))
                    }
                    TextTest::Starts(value) => texts
                        .iter()
                        .any(|s| s.trim().to_lowercase().starts_with(value)),
                    TextTest::Regex(regex) => texts.iter().any(|s| regex.is_match(s)),
                }
            }
        }
    }
}

pub(super) fn hash_text(hash: &mut Sha256, text: &str) {
    hash.update((text.len() as u64).to_le_bytes());
    hash.update(text);
}

fn field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key)?.as_str()
}

fn reasoning(format: Format, body: &Value) -> (bool, Option<usize>) {
    let (gemini_level, gemini_budget) = if format == Format::Gemini {
        super::effort::gemini_thinking(body)
    } else {
        (None, None)
    };
    let level = match format {
        Format::Chat => field(body, "reasoning_effort"),
        Format::Responses => body.pointer("/reasoning/effort").and_then(Value::as_str),
        Format::Messages => body
            .pointer("/output_config/effort")
            .and_then(Value::as_str),
        Format::Gemini => gemini_level,
    }
    .map(str::to_ascii_lowercase);
    let level = level.as_deref().unwrap_or_default();
    let thinking = body.pointer("/thinking/type").and_then(Value::as_str);
    let budget = if format == Format::Gemini {
        gemini_budget.filter(|&n| n >= -1)
    } else {
        body.pointer("/thinking/budget_tokens")
            .and_then(Value::as_i64)
    };
    if matches!(level, "none" | "off") || thinking == Some("disabled") || budget == Some(0) {
        return (false, None);
    }
    let active =
        !level.is_empty() || matches!(thinking, Some("enabled" | "adaptive")) || budget.is_some();
    let effort = effort_rank(level).or_else(|| {
        budget.filter(|&n| n > 0).and_then(|n| {
            effort_rank(match n {
                1..=2048 => "low",
                2049..=8192 => "medium",
                _ => "high",
            })
        })
    });
    (active, effort)
}

fn collect_text<'a>(content: &'a Value, text: &mut Vec<&'a str>, images: &mut bool) {
    if let Some(s) = content.as_str() {
        text.push(s);
    } else if let Some(blocks) = content.as_array() {
        for block in blocks {
            if let Some(s) = field(block, "text").or_else(|| block.as_str()) {
                text.push(s);
            }
            *images |= matches!(
                field(block, "type"),
                Some("image" | "image_url" | "input_image")
            ) || ["/inlineData/mimeType", "/fileData/mimeType"]
                .iter()
                .any(|p| {
                    block
                        .pointer(p)
                        .and_then(Value::as_str)
                        .is_some_and(|mime| mime.starts_with("image/"))
                });
        }
    }
}

fn tool_result(turn: &Value, role: &str, content: Option<&Value>) -> bool {
    role == "tool"
        || field(turn, "type") == Some("function_call_output")
        || content.and_then(Value::as_array).is_some_and(|parts| {
            parts.iter().any(|part| {
                field(part, "type") == Some("tool_result") || part.get("functionResponse").is_some()
            })
        })
}

fn tool_length(turn: &Value, content: Option<&Value>) -> usize {
    ["name", "arguments", "output"]
        .iter()
        .filter_map(|key| field(turn, key))
        .map(str::len)
        .sum::<usize>()
        + turn
            .get("tool_calls")
            .map_or(0, |calls| calls.to_string().len())
        + content
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|part| {
                let value = if field(part, "type") == Some("tool_result") {
                    part.get("content")
                } else {
                    part.get("functionResponse")
                        .or_else(|| part.get("functionCall"))
                };
                value.map_or(0, |value| {
                    value
                        .as_str()
                        .map_or_else(|| value.to_string().len(), str::len)
                })
            })
            .sum::<usize>()
}

fn is_context(text: &str) -> bool {
    let text = text.trim();
    [
        "<environment_context>",
        "<system-reminder>",
        "<task-notification>",
        "<env>",
        "<cwd>",
    ]
    .iter()
    .any(|prefix| text.starts_with(prefix) && text.ends_with(&format!("</{}", &prefix[1..])))
}

fn declared_folder(text: &str, system: bool) -> Option<String> {
    static PATH: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
        r"(?m)<cwd>\s*([^<\r\n]+?)\s*</cwd>|^[ \t]*(?:-[ \t]+)?(?:Primary working directory|Current working directory|Working directory|CWD):[ \t]*(.+)$"
    ).expect("fixed request folder pattern")
    });
    if !system && !is_context(text) {
        return None;
    }
    let captures = PATH.captures_iter(text).last()?;
    let path = captures.get(1).or_else(|| captures.get(2))?.as_str().trim();
    folder_key(
        path.split_once(" (was ")
            .map_or(path, |(current, _)| current),
    )
}

fn compacting(system: &[String], last: &[String]) -> bool {
    let has = |texts: &[String], markers: &[&str]| {
        texts.iter().any(|text| {
            markers
                .iter()
                .any(|marker| text.find(marker).is_some_and(|at| at <= 2048))
        })
    };
    has(
        system,
        &[
            "tasked with summarizing conversations",
            "You are a context summarization",
            "responsible for distilling chat history",
            "summarizes a conversation when its context window",
            "You are a context compaction engine",
            "compacts conversation context",
            "conversation so it can continue WITHOUT losing context",
            "conversation summary with new activity so the conversation can continue",
        ],
    ) || has(
        last,
        &[
            "Your task is to create a detailed summary of the conversation so far",
            "You are performing a CONTEXT CHECKPOINT COMPACTION.",
            "You are now given a task to compact this conversation context",
            "You are now acting as a compaction engine for this AI coding assistant",
            "Merge the previous summary with the new conversation.",
            "Here is the full conversation to compact.",
        ],
    )
}
