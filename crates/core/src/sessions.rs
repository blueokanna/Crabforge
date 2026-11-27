use crate::agents::NativeAgent as Agent;
use anyhow::{Context, Result, ensure};
use nextjson::{NsonSerialize, Value};
use ruzstd::decoding::StreamingDecoder;
use std::{
    cmp::Reverse,
    collections::HashSet,
    env, fs,
    io::{BufRead, BufReader, ErrorKind, Read, Seek},
    path::{Path, PathBuf},
    time::SystemTime,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

mod pi;

const MAX_HEADER: u64 = 512 * 1024;
const MAX_CODEX_WINDOW: u64 = 16 * 1024 * 1024;
const MAX_EXCERPT_SCAN: u64 = 16 * 1024 * 1024;
const MAX_EXCERPT_CHARS: usize = 24_000;

#[derive(NsonSerialize)]
pub struct Session {
    pub agent: &'static str,
    pub profile: String,
    pub id: String,
    pub cwd: String,
    pub title: String,
    pub modified: String,
    pub archived: bool,
}

#[derive(NsonSerialize)]
pub struct Listing {
    pub sessions: Vec<Session>,
    pub unreadable: usize,
}

pub(crate) struct Excerpt {
    pub session: Session,
    pub messages: Vec<String>,
}

struct Log<'a> {
    agent: Agent,
    profile: &'a Path,
    path: PathBuf,
    modified: SystemTime,
    archived: bool,
}

pub fn list(dir: &Path, agent: Option<Agent>, limit: usize) -> Result<Listing> {
    let roots = [Agent::Claude, Agent::Codex, Agent::Pi]
        .into_iter()
        .filter(|item| agent.is_none_or(|agent| agent == *item))
        .flat_map(|agent| {
            session_profiles(dir, agent)
                .into_iter()
                .map(move |path| (agent, path))
        });
    list_in(roots, limit)
}

fn session_profiles(dir: &Path, agent: Agent) -> Vec<PathBuf> {
    if agent == Agent::Pi {
        let configured = env::var_os("PI_CODING_AGENT_SESSION_DIR")
            .filter(|path| !path.is_empty())
            .map(PathBuf::from);
        let native = agent.home().map(|home| home.join("sessions"));
        let managed = dir.join("agents/pi/sessions");
        let mut seen = HashSet::new();
        return configured
            .into_iter()
            .chain(native)
            .chain([managed])
            .filter(|path| seen.insert(path.canonicalize().unwrap_or_else(|_| path.clone())))
            .collect();
    }
    agent.home().map_or_else(Vec::new, |home| {
        profiles(agent, home, env::home_dir().as_deref())
    })
}

fn profiles(agent: Agent, primary: PathBuf, user: Option<&Path>) -> Vec<PathBuf> {
    fn directories(path: &Path) -> impl Iterator<Item = PathBuf> {
        fs::read_dir(path)
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                entry.file_type().ok()?.is_dir().then(|| entry.path())
            })
    }

    let has_sessions = |path: &Path| match agent {
        Agent::Claude => path.join("projects").is_dir(),
        Agent::Codex => path.join("sessions").is_dir() || path.join("archived_sessions").is_dir(),
        Agent::Pi => false,
    };
    let mut seen: HashSet<_> = primary.canonicalize().into_iter().collect();
    let mut additional = Vec::new();
    let mut add = |path: PathBuf| {
        if let Ok(resolved) = path.canonicalize()
            && seen.insert(resolved)
        {
            additional.push(path);
        }
    };
    let parent = primary.parent().filter(|path| !path.as_os_str().is_empty());
    for root in user
        .into_iter()
        .chain(parent.filter(|path| Some(*path) != user))
    {
        for folder in directories(root).filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.trim_start_matches('.')
                        .to_ascii_lowercase()
                        .starts_with(agent.id())
                })
        }) {
            if has_sessions(&folder) {
                add(folder);
            } else {
                directories(&folder)
                    .filter(|path| has_sessions(path))
                    .for_each(&mut add);
            }
        }
    }
    additional.sort_unstable();
    let mut roots = vec![primary];
    roots.extend(additional);
    roots
}

fn list_in(roots: impl IntoIterator<Item = (Agent, PathBuf)>, limit: usize) -> Result<Listing> {
    let roots: Vec<_> = roots.into_iter().collect();
    let mut result = Listing {
        sessions: Vec::new(),
        unreadable: 0,
    };
    let mut seen = HashSet::new();
    for log in ordered_logs(&roots)? {
        match log.session() {
            Ok(Some(session)) if seen.insert((log.agent, log.profile, session.id.clone())) => {
                result.sessions.push(session);
                if result.sessions.len() == limit.clamp(1, 500) {
                    break;
                }
            }
            Ok(_) => (),
            Err(_) => result.unreadable += 1,
        }
    }
    Ok(result)
}

pub(crate) fn excerpt(
    dir: &Path,
    agent: Agent,
    profile: Option<&Path>,
    id: &str,
) -> Result<Excerpt> {
    let profiles = match profile {
        Some(profile) => {
            let resolved = profile
                .canonicalize()
                .context("cannot open client profile")?;
            let known = session_profiles(dir, agent)
                .into_iter()
                .find(|path| path.canonicalize().is_ok_and(|path| path == resolved))
                .context("profile was not found among the client's local profiles")?;
            vec![known]
        }
        None if agent == Agent::Pi => session_profiles(dir, agent),
        None => vec![agent.home().context("client profile is unavailable")?],
    };
    let roots: Vec<_> = profiles.into_iter().map(|path| (agent, path)).collect();
    for log in ordered_logs(&roots)? {
        let Ok(mut file) = fs::File::open(&log.path) else {
            continue;
        };
        let Ok(Some(session)) = log.reader(&file).and_then(|reader| log.read_header(reader)) else {
            continue;
        };
        if session.id == id {
            file.rewind()?;
            let messages = log.read_excerpt(log.reader(&file)?)?;
            ensure!(!messages.is_empty(), "session contains no user messages");
            return Ok(Excerpt { session, messages });
        }
    }
    anyhow::bail!("session was not found in the selected profile")
}

fn ordered_logs(roots: &[(Agent, PathBuf)]) -> Result<Vec<Log<'_>>> {
    let mut logs = Vec::new();
    for (agent, home) in roots {
        match agent {
            Agent::Claude => visit(&home.join("projects"), 1, *agent, home, false, &mut logs)?,
            Agent::Codex => {
                visit(&home.join("sessions"), 3, *agent, home, false, &mut logs)?;
                visit(
                    &home.join("archived_sessions"),
                    0,
                    *agent,
                    home,
                    true,
                    &mut logs,
                )?;
            }
            Agent::Pi => visit(home, 1, *agent, home, false, &mut logs)?,
        }
    }
    logs.sort_unstable_by(|a, b| {
        (Reverse(a.modified), a.archived, &a.path).cmp(&(Reverse(b.modified), b.archived, &b.path))
    });
    Ok(logs)
}

fn visit<'a>(
    root: &Path,
    depth: usize,
    agent: Agent,
    profile: &'a Path,
    archived: bool,
    logs: &mut Vec<Log<'a>>,
) -> Result<()> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).with_context(|| format!("cannot list {}", root.display())),
    };
    for entry in entries {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() && depth > 0 {
            visit(&entry.path(), depth - 1, agent, profile, archived, logs)?;
        } else if kind.is_file()
            && let Some(name) = entry.file_name().to_str()
            && match agent {
                Agent::Claude => depth == 0 && name.ends_with(".jsonl"),
                Agent::Codex => {
                    name.starts_with("rollout-")
                        && (name.ends_with(".jsonl") || name.ends_with(".jsonl.zst"))
                }
                Agent::Pi => name.ends_with(".jsonl"),
            }
        {
            match entry.metadata().and_then(|metadata| metadata.modified()) {
                Ok(modified) => logs.push(Log {
                    agent,
                    profile,
                    path: entry.path(),
                    modified,
                    archived,
                }),
                Err(error) if error.kind() == ErrorKind::NotFound => (),
                Err(error) => return Err(error.into()),
            }
        }
    }
    Ok(())
}

impl Log<'_> {
    fn session(&self) -> Result<Option<Session>> {
        self.read_header(self.reader(fs::File::open(&self.path)?)?)
    }

    fn reader<'file>(&self, file: impl Read + 'file) -> Result<Box<dyn Read + 'file>> {
        Ok(if self.path.extension().is_some_and(|ext| ext == "zst") {
            Box::new(StreamingDecoder::new_with_max_window_size(
                BufReader::new(file),
                MAX_CODEX_WINDOW,
            )?)
        } else {
            Box::new(file)
        })
    }

    fn read_header(&self, source: impl Read) -> Result<Option<Session>> {
        let mut reader = BufReader::new(source.take(MAX_HEADER));
        let mut bytes = Vec::new();
        let mut id = String::new();
        let mut cwd = String::new();
        let mut title = String::new();
        while reader.read_until(b'\n', &mut bytes)? > 0 {
            let row = decode_line(&bytes);
            bytes.clear();
            let Some(row) = row else { continue };
            let kind = string(&row, "/type");
            match self.agent {
                Agent::Claude => {
                    if flag(&row, "/isSidechain") || flag(&row, "/isMeta") {
                        continue;
                    }
                    if id.is_empty() {
                        id = string(&row, "/sessionId").to_owned();
                    }
                    if cwd.is_empty() {
                        cwd = string(&row, "/cwd").to_owned();
                    }
                    if title.is_empty() && kind == "user" && !flag(&row, "/isCompactSummary") {
                        title = message_title(message_parts(row.pointer("/message/content")));
                    }
                }
                Agent::Codex => {
                    if kind == "session_meta" {
                        if !string(&row, "/payload/parent_thread_id").is_empty()
                            || row.pointer("/payload/source/subagent").is_some()
                        {
                            return Ok(None);
                        }
                        id = string(&row, "/payload/id").to_owned();
                        cwd = string(&row, "/payload/cwd").to_owned();
                    } else if title.is_empty()
                        && kind == "event_msg"
                        && string(&row, "/payload/type") == "user_message"
                    {
                        title = concise(string(&row, "/payload/message"));
                    } else if title.is_empty()
                        && kind == "response_item"
                        && string(&row, "/payload/type") == "message"
                        && string(&row, "/payload/role") == "user"
                    {
                        title = message_title(message_parts(row.pointer("/payload/content")));
                    }
                }
                Agent::Pi => {
                    if id.is_empty() {
                        if pi::version(&row).is_none() {
                            return Ok(None);
                        }
                        id = string(&row, "/id").to_owned();
                        cwd = string(&row, "/cwd").to_owned();
                        if id.is_empty() || cwd.is_empty() {
                            return Ok(None);
                        }
                    } else if pi::user_message(&row) {
                        title = message_title(message_parts(row.pointer("/message/content")));
                    }
                }
            }
            if !id.is_empty() && !cwd.is_empty() && !title.is_empty() {
                break;
            }
        }
        if id.is_empty() || title.is_empty() {
            return Ok(None);
        }
        Ok(Some(Session {
            agent: self.agent.id(),
            profile: self.profile.display().to_string(),
            id,
            cwd,
            title,
            archived: self.archived,
            modified: OffsetDateTime::from(self.modified).format(&Rfc3339)?,
        }))
    }

    fn read_excerpt(&self, source: impl Read) -> Result<Vec<String>> {
        if self.agent == Agent::Pi {
            return pi::read_excerpt(source);
        }
        let mut reader = BufReader::new(source.take(MAX_EXCERPT_SCAN));
        let mut bytes = Vec::new();
        let mut native = Messages::default();
        let mut fallback = Messages::default();
        while reader.read_until(b'\n', &mut bytes)? > 0 {
            let row = decode_line(&bytes);
            bytes.clear();
            let Some(row) = row else { continue };
            let kind = string(&row, "/type");
            match self.agent {
                Agent::Claude
                    if kind == "user"
                        && !["/isMeta", "/isSidechain", "/isCompactSummary"]
                            .iter()
                            .any(|p| flag(&row, p)) =>
                {
                    native.push(message_parts(row.pointer("/message/content")));
                }
                Agent::Codex
                    if kind == "event_msg" && string(&row, "/payload/type") == "user_message" =>
                {
                    native.push([string(&row, "/payload/message")]);
                }
                Agent::Codex
                    if kind == "response_item"
                        && string(&row, "/payload/type") == "message"
                        && string(&row, "/payload/role") == "user" =>
                {
                    fallback.push(message_parts(row.pointer("/payload/content")));
                }
                _ => {}
            }
            if native.chars == MAX_EXCERPT_CHARS {
                break;
            }
        }
        Ok(if native.values.is_empty() {
            fallback.values
        } else {
            native.values
        })
    }
}

fn decode_line(bytes: &[u8]) -> Option<Value> {
    nextjson::from_slice(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes)).ok()
}

#[derive(Default)]
struct Messages {
    chars: usize,
    values: Vec<String>,
}

impl Messages {
    fn push<'a>(&mut self, parts: impl IntoIterator<Item = &'a str>) {
        let wrappers = [
            "<command-name>",
            "<command-message>",
            "<command-args>",
            "<task-notification>",
            "Caveat: The messages below were generated",
        ];
        let mut text = String::new();
        for part in parts
            .into_iter()
            .filter_map(user_text)
            .filter(|part| !wrappers.iter().any(|prefix| part.starts_with(prefix)))
        {
            if self.chars == MAX_EXCERPT_CHARS {
                break;
            }
            if !text.is_empty() {
                text.push('\n');
                self.chars += 1;
            }
            for ch in part.chars().take(MAX_EXCERPT_CHARS - self.chars) {
                text.push(ch);
                self.chars += 1;
            }
        }
        if !text.is_empty() {
            self.values.push(text);
        }
    }
}

fn string<'a>(row: &'a Value, pointer: &str) -> &'a str {
    row.pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or_default()
}

fn flag(row: &Value, pointer: &str) -> bool {
    row.pointer(pointer)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn message_parts(content: Option<&Value>) -> impl Iterator<Item = &str> {
    content.and_then(Value::as_str).into_iter().chain(
        content
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|block| matches!(string(block, "/type"), "text" | "input_text"))
            .map(|block| string(block, "/text")),
    )
}

fn message_title<'a>(parts: impl Iterator<Item = &'a str>) -> String {
    parts
        .map(concise)
        .find(|title| !title.is_empty())
        .unwrap_or_default()
}

fn concise(text: &str) -> String {
    user_text(text)
        .and_then(|text| text.lines().next())
        .unwrap_or_default()
        .trim()
        .chars()
        .take(160)
        .collect()
}

fn user_text(text: &str) -> Option<&str> {
    let text = text.trim();
    let generated = [
        "<environment_context>",
        "<user_instructions>",
        "<local-command-",
        "<system-reminder>",
        "# AGENTS.md instructions",
    ];
    (!text.is_empty() && !generated.iter().any(|prefix| text.starts_with(*prefix))).then_some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_local_sessions_and_excludes_agent_generated_context() -> Result<()> {
        let home = tempfile::tempdir()?;
        let claude = home.path().join("claude");
        let codex = home.path().join("codex");
        let projects = claude.join("projects/example");
        let active = codex.join("sessions/2026/11/27");
        let archived = codex.join("archived_sessions");
        for path in [&projects, &active, &archived] {
            fs::create_dir_all(path)?;
        }
        fs::write(
            projects.join("example.jsonl"),
            concat!(
                "{\"type\":\"user\",\"sessionId\":\"example\",\"cwd\":\"/project\",\"isMeta\":true,\"message\":{\"content\":\"side input\"}}\n",
                "{\"type\":\"user\",\"sessionId\":\"example\",\"cwd\":\"/project\",\"message\":{\"content\":[{\"type\":\"text\",\"text\":\"Review the build\"}]}}\n",
            ),
        )?;
        fs::write(
            archived.join("rollout-old.jsonl"),
            concat!(
                "{\"type\":\"session_meta\",\"payload\":{\"id\":\"older\",\"cwd\":\"/project\"}}\n",
                "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"<environment_context>\"}]}}\n",
                "{\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"Verify the test\"}}\n",
            ),
        )?;
        fs::write(
            active.join("rollout-subagent.jsonl"),
            concat!(
                "{\"type\":\"session_meta\",\"payload\":{\"id\":\"child\",\"source\":{\"subagent\":{\"thread_spawn\":{}}}}}\n",
                "{\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"a delegated task\"}}\n",
            ),
        )?;
        let result = list_in([(Agent::Claude, claude), (Agent::Codex, codex)], 20)?;
        assert_eq!(result.sessions.len(), 2);
        assert_eq!(result.unreadable, 0);
        assert!(result.sessions.iter().any(|entry| entry.id == "example"
            && entry.title == "Review the build"
            && !entry.archived));
        assert!(result.sessions.iter().any(|entry| entry.id == "older"
            && entry.title == "Verify the test"
            && entry.archived));
        Ok(())
    }

    #[test]
    fn reads_packed_codex_sessions_and_counts_decode_failures() -> Result<()> {
        use ruzstd::encoding::{CompressionLevel, compress_to_vec};

        let home = tempfile::tempdir()?;
        let archived = home.path().join("archived_sessions");
        fs::create_dir(&archived)?;
        let records = concat!(
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"packed\",\"cwd\":\"/project\"}}\n",
            "{\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"Verify the archive\"}}\n",
        );
        let packed = compress_to_vec(records.as_bytes(), CompressionLevel::Fastest);
        fs::write(archived.join("rollout-archive.jsonl.zst"), &packed)?;
        fs::write(archived.join("rollout-truncated.jsonl.zst"), &packed[..8])?;

        let result = list_in([(Agent::Codex, home.path().to_owned())], 20)?;
        assert_eq!(result.unreadable, 1);
        assert_eq!(result.sessions.len(), 1);
        let entry = &result.sessions[0];
        assert_eq!(entry.id, "packed");
        assert_eq!(entry.title, "Verify the archive");
        assert_eq!(entry.cwd, "/project");
        assert!(entry.archived);
        Ok(())
    }

    #[test]
    fn discovers_sibling_and_nested_profiles_with_sessions() -> Result<()> {
        let root = tempfile::tempdir()?;
        let user = root.path().join("user");
        let primary = root.path().join("accounts/codex");
        let sibling = root.path().join("accounts/.codex-work");
        let nested = user.join(".codex-profiles/testing");
        for folder in [
            primary.join("sessions"),
            sibling.join("sessions"),
            nested.join("archived_sessions"),
            user.join("unrelated/sessions"),
            user.join(".codex-misplaced/level1/level2/sessions"),
        ] {
            fs::create_dir_all(folder)?;
        }
        assert_eq!(
            profiles(Agent::Codex, primary.clone(), Some(&user)),
            [primary, sibling, nested]
        );
        Ok(())
    }
}
