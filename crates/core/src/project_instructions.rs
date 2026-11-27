use anyhow::{Context, Result, bail, ensure};
use nextjson::{
    NsonDeserialize, NsonSerialize, Value,
    formats::{Format, Json5},
};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{ErrorKind, Read, Write},
    ops::Range,
    path::{Component, Path, PathBuf},
};

// Keeping Magpie's markers lets an existing project move between the two editors.
const BEGIN: &str = "<!-- magpie:begin · written by magpie from its Library -->";
const END: &str = "<!-- magpie:end -->";
const MEMORY_BEGIN: &str = "<!-- crabforge:project-memory:begin -->";
const MEMORY_END: &str = "<!-- crabforge:project-memory:end -->";
const AGENT_MEMORY_BEGIN: &str = "<!-- crabforge:agent-memory:begin -->";
const AGENT_MEMORY_END: &str = "<!-- crabforge:agent-memory:end -->";

#[derive(Clone, Copy)]
enum Section {
    Instructions,
    ProjectMemory,
    AgentMemory,
}

impl Section {
    const ALL: [Self; 3] = [Self::Instructions, Self::ProjectMemory, Self::AgentMemory];

    fn markers(self) -> (&'static str, &'static str) {
        match self {
            Self::Instructions => (BEGIN, END),
            Self::ProjectMemory => (MEMORY_BEGIN, MEMORY_END),
            Self::AgentMemory => (AGENT_MEMORY_BEGIN, AGENT_MEMORY_END),
        }
    }
}
const MAX_DOCUMENT: u64 = 256 * 1024;
const MAX_BACKUPS: usize = 16;
const MAX_HISTORY: u64 = MAX_DOCUMENT * 6 * MAX_BACKUPS as u64 + 4096;
const AGENTS: &[(&str, &[&str])] = &[
    ("claude", &["CLAUDE.md", ".claude/CLAUDE.md"]),
    ("codex", &["AGENTS.override.md", "AGENTS.md"]),
    (
        "pi",
        &[
            "AGENTS.override.md",
            "AGENTS.md",
            "AGENTS.MD",
            "CLAUDE.md",
            "CLAUDE.MD",
        ],
    ),
    ("opencode", &["AGENTS.md", "CLAUDE.md"]),
    ("gemini", &["GEMINI.md"]),
    ("qwen", &["QWEN.md"]),
    ("qoder", &["AGENTS.md"]),
    ("qoder-cn", &["AGENTS.md"]),
    ("codebuddy", &["CODEBUDDY.md"]),
    ("cursor", &["AGENTS.md"]),
    ("copilot", &["AGENTS.md"]),
    ("kimi", &["AGENTS.md"]),
    ("droid", &["AGENTS.md"]),
    ("crush", &["AGENTS.md"]),
    ("grok", &["AGENTS.md"]),
    ("cline", &["AGENTS.md"]),
    ("commandcode", &["AGENTS.md"]),
    ("goose", &["AGENTS.md"]),
];

pub fn agents() -> impl Iterator<Item = &'static str> {
    AGENTS.iter().map(|(agent, _)| *agent).chain(["omp"])
}

fn context_filename(settings: &Path) -> Result<Option<String>> {
    let Some(source) = optional(settings, MAX_DOCUMENT)? else {
        return Ok(None);
    };
    let value: Value = Json5
        .decode(source.trim_start_matches('\u{feff}').as_bytes())
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("cannot read settings from {}", settings.display()))?;
    let Some(value) = value.pointer("/context/fileName") else {
        return Ok(None);
    };
    fn name(value: &Value) -> Option<&str> {
        value
            .as_str()
            .map(str::trim)
            .filter(|name| !name.is_empty())
    }
    Ok(match value.as_array() {
        Some(names) => names.iter().find_map(name),
        None => name(value),
    }
    .map(str::to_owned))
}

fn configured_filename(dir: &Path, project: &Path, agent: &str) -> Result<Option<String>> {
    let (folder, env_key, profile) = match agent {
        "gemini" => (".gemini", "GEMINI_CLI_HOME", Some("agents/gemini/.gemini")),
        "qwen" => (".qwen", "QWEN_HOME", Some("agents/qwen")),
        "qoder" => (".qoder", "QODER_CONFIG_DIR", None),
        "qoder-cn" => (".qoder-cn", "QODERCN_CONFIG_DIR", None),
        _ => return Ok(None),
    };
    let native = match env::var_os(env_key).filter(|value| !value.is_empty()) {
        Some(home) if agent == "gemini" => Some(PathBuf::from(home).join(folder)),
        Some(home) => Some(PathBuf::from(home)),
        None => env::home_dir().map(|home| home.join(folder)),
    };
    let paths = [
        Some(project.join(folder)),
        profile.map(|profile| dir.join(profile)),
        native,
    ];
    for path in paths.into_iter().flatten() {
        if let Some(name) = context_filename(&path.join("settings.json"))? {
            return Ok(Some(name));
        }
    }
    Ok(None)
}

fn target(dir: &Path, project: &Path, agent: &str) -> Result<PathBuf> {
    let agent = if agent == "omp" { "pi" } else { agent };
    let root = project
        .canonicalize()
        .context("cannot open the project folder")?;
    ensure!(root.is_dir(), "expected a project folder");
    let (_, names) = AGENTS
        .iter()
        .find(|(id, _)| *id == agent)
        .context("unsupported project instructions agent")?;
    let configured = configured_filename(dir, &root, agent)?;
    let custom = [configured.as_deref().unwrap_or_default()];
    let candidates = if configured.is_some() {
        &custom[..]
    } else {
        names
    };
    for name in candidates {
        ensure!(
            Path::new(name)
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
                && !name.contains([':', '\0']),
            "instruction filename must be relative to the project folder"
        );
        let path = root.join(name);
        if let Some(existing) = existing_in(&root, &path)? {
            return Ok(existing);
        }
        if configured.is_some() {
            let parent = path.parent().context("missing instruction parent folder")?;
            let parent = parent
                .canonicalize()
                .context("cannot open the instruction parent folder")?;
            ensure!(
                parent.starts_with(&root) && parent.is_dir(),
                "instructions must be a file inside the project"
            );
            return Ok(parent.join(path.file_name().context("missing instruction filename")?));
        }
    }
    let default = if matches!(agent, "codex" | "pi") {
        "AGENTS.md"
    } else {
        names[0]
    };
    Ok(root.join(default))
}

fn existing_in(root: &Path, path: &Path) -> Result<Option<PathBuf>> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            let resolved = path.canonicalize()?;
            ensure!(
                resolved.starts_with(root) && resolved.is_file(),
                "instructions must be a file inside {}",
                root.display()
            );
            Ok(Some(resolved))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn agent_target(agent: &str) -> Result<PathBuf> {
    use crate::agents::NativeAgent;

    let agent =
        NativeAgent::by_id(agent).context("user instructions support Claude Code and Codex")?;
    let names: &[&str] = match agent {
        NativeAgent::Claude => &["CLAUDE.md"],
        NativeAgent::Codex => &["AGENTS.override.md", "AGENTS.md"],
        NativeAgent::Gemini | NativeAgent::Pi => {
            anyhow::bail!("user instructions support Claude Code and Codex")
        }
    };
    let root = agent
        .home()
        .context("cannot locate the agent's configuration folder")?
        .canonicalize()
        .context("cannot open the agent's configuration folder")?;
    ensure!(root.is_dir(), "expected an agent configuration folder");
    for name in names {
        if let Some(path) = existing_in(&root, &root.join(name))? {
            return Ok(path);
        }
    }
    Ok(root.join(names.last().unwrap()))
}

struct Block {
    whole: Range<usize>,
    body: Range<usize>,
}

fn block(source: &str, (start_marker, end_marker): (&str, &str)) -> Result<Option<Block>> {
    let mut begin = None;
    let mut found = None;
    let mut pos = if source.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    for line in source[pos..].split_inclusive('\n') {
        let end = pos + line.len();
        match line.trim_end_matches(['\r', '\n']) {
            line if line == start_marker => {
                ensure!(
                    begin.is_none() && found.is_none(),
                    "multiple instruction blocks"
                );
                begin = Some((pos, end));
            }
            line if line == end_marker => {
                let (start, content) = begin.take().context("unmatched instruction end marker")?;
                found = Some(Block {
                    whole: start..end,
                    body: content..pos,
                });
            }
            _ => (),
        }
        pos = end;
    }
    ensure!(begin.is_none(), "unclosed instruction block");
    Ok(found)
}

fn blocks(source: &str) -> Result<[Option<Block>; 3]> {
    let [instructions, project, agent] = Section::ALL;
    let found = [
        block(source, instructions.markers())?,
        block(source, project.markers())?,
        block(source, agent.markers())?,
    ];
    let mut whole: Vec<_> = found.iter().flatten().map(|block| &block.whole).collect();
    whole.sort_unstable_by_key(|range| range.start);
    for pair in whole.windows(2) {
        ensure!(
            pair[0].end <= pair[1].start,
            "managed instructions and memory blocks overlap"
        );
    }
    Ok(found)
}

fn updated(source: &str, text: &str, section: Section) -> Result<String> {
    let mut blocks = blocks(source)?;
    let old = blocks[section as usize].take();
    let (begin, end) = section.markers();
    let text = text
        .strip_prefix('\u{feff}')
        .unwrap_or(text)
        .replace("\r\n", "\n");
    let text = text.trim_end_matches('\n');
    ensure!(
        !text.split('\n').any(|line| matches!(
            line.trim_end_matches('\r'),
            BEGIN | END | MEMORY_BEGIN | MEMORY_END | AGENT_MEMORY_BEGIN | AGENT_MEMORY_END
        )),
        "instructions contain a reserved marker line"
    );
    let eol = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut result = source.to_owned();
    if text.trim().is_empty() {
        if let Some(old) = old {
            result.replace_range(old.whole, "");
        }
    } else {
        let text = text.replace('\n', eol);
        if let Some(old) = old {
            result.replace_range(old.body, &format!("{text}{eol}"));
        } else {
            if !result.trim_start_matches('\u{feff}').is_empty() && !result.ends_with('\n') {
                result.push_str(eol);
            }
            result.push_str(&format!("{begin}{eol}{text}{eol}{end}{eol}"));
        }
    }
    Ok(result)
}

#[derive(NsonSerialize)]
pub struct View {
    pub path: String,
    pub text: String,
    pub memory_digest: String,
    pub revision: String,
    pub can_restore: bool,
}

#[derive(NsonSerialize, NsonDeserialize)]
struct Backup {
    after: String,
    before: Option<String>,
    created: bool,
}

fn optional(path: &Path, limit: u64) -> Result<Option<String>> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).with_context(|| format!("cannot open {}", path.display())),
    };
    let mut text = String::new();
    file.take(limit + 1)
        .read_to_string(&mut text)
        .with_context(|| format!("cannot read UTF-8 from {}", path.display()))?;
    ensure!(
        text.len() as u64 <= limit,
        "{} exceeds the {limit}-byte limit",
        path.display()
    );
    Ok(Some(text))
}

pub fn read_markdown(path: &Path) -> Result<String> {
    optional(path, MAX_DOCUMENT)?.with_context(|| format!("cannot find {}", path.display()))
}

fn identity(path: &Path) -> String {
    hex::encode(Sha256::digest(path.as_os_str().as_encoded_bytes()))
}

fn revision(path: &Path, text: Option<&str>) -> String {
    let mut hash = Sha256::new();
    hash.update(identity(path));
    hash.update([text.is_some() as u8]);
    hash.update(text.unwrap_or_default());
    hex::encode(hash.finalize())
}

pub(crate) fn replace(path: &Path, text: &str) -> Result<()> {
    let mut tmp = tempfile::NamedTempFile::new_in(path.parent().context("missing parent folder")?)?;
    if let Ok(meta) = path.metadata() {
        tmp.as_file().set_permissions(meta.permissions())?;
    }
    tmp.write_all(text.as_bytes())?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;
    Ok(())
}

fn history(dir: &Path, path: &Path) -> Result<(PathBuf, Vec<Backup>)> {
    let record = dir
        .join("project-instructions")
        .join(identity(path))
        .join("history.json");
    let entries = optional(&record, MAX_HISTORY)?
        .map(|s| nextjson::from_str(&s).map_err(anyhow::Error::msg))
        .transpose()?
        .unwrap_or_default();
    Ok((record, entries))
}

pub fn read(dir: &Path, project: &Path, agent: &str) -> Result<View> {
    read_path(dir, target(dir, project, agent)?, Section::ProjectMemory)
}

pub fn read_agent(dir: &Path, agent: &str) -> Result<View> {
    read_path(dir, agent_target(agent)?, Section::AgentMemory)
}

fn read_path(dir: &Path, path: PathBuf, memory_section: Section) -> Result<View> {
    let contents = optional(&path, MAX_DOCUMENT)?;
    let source = contents.as_deref().unwrap_or_default();
    let found = blocks(source)?;
    let text = |section: Section| {
        found[section as usize]
            .as_ref()
            .map(|b| {
                source[b.body.clone()]
                    .trim_end_matches(['\r', '\n'])
                    .replace("\r\n", "\n")
            })
            .unwrap_or_default()
    };
    let revision = revision(&path, contents.as_deref());
    let (_, entries) = history(dir, &path)?;
    Ok(View {
        path: path.display().to_string(),
        text: text(Section::Instructions),
        memory_digest: text(memory_section),
        can_restore: entries.iter().any(|e| e.after == revision),
        revision,
    })
}

/// An empty string clears the managed block. None restores the last matching file version.
pub fn edit(
    dir: &Path,
    project: &Path,
    agent: &str,
    previous: &str,
    text: Option<&str>,
) -> Result<()> {
    write(
        dir,
        || target(dir, project, agent),
        previous,
        text,
        Section::Instructions,
    )
}

pub fn publish_memory(
    dir: &Path,
    project: &Path,
    agent: &str,
    previous: &str,
    preview: Option<&str>,
) -> Result<()> {
    let (_folder, _lock) = crate::memory::lock_store(dir)?;
    let digest = crate::memory::project_digest(dir, project)?;
    ensure!(
        preview.is_none_or(|preview| digest == preview),
        "project memory changed; preview it again"
    );
    write(
        dir,
        || target(dir, project, agent),
        previous,
        Some(&digest),
        Section::ProjectMemory,
    )
}

pub fn publish_agent_memory(
    dir: &Path,
    agent: &str,
    previous: &str,
    preview: Option<&str>,
) -> Result<()> {
    let (_folder, _lock) = crate::memory::lock_store(dir)?;
    let digest = crate::memory::agent_digest(dir, Some(agent))?;
    ensure!(
        preview.is_none_or(|preview| preview == digest),
        "agent memory changed; preview it again"
    );
    write(
        dir,
        || agent_target(agent),
        previous,
        Some(&digest),
        Section::AgentMemory,
    )
}

pub fn edit_agent(dir: &Path, agent: &str, previous: &str, text: Option<&str>) -> Result<()> {
    write(
        dir,
        || agent_target(agent),
        previous,
        text,
        Section::Instructions,
    )
}

fn write(
    dir: &Path,
    resolve: impl Fn() -> Result<PathBuf>,
    previous: &str,
    text: Option<&str>,
    section: Section,
) -> Result<()> {
    let store = dir.join("project-instructions");
    fs::create_dir_all(&store)?;
    let lock = fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(store.join("library.lock"))?;
    lock.lock()?;
    let path = resolve()?;
    let before = optional(&path, MAX_DOCUMENT)?;
    let version = revision(&path, before.as_deref());
    ensure!(
        version == previous,
        "instructions changed; reload the project file"
    );
    let (record, mut entries) = history(dir, &path)?;
    let index = entries.iter().rposition(|e| e.after == version);
    let write_document = |contents: Option<&str>| -> Result<()> {
        ensure!(
            resolve()? == path
                && revision(&path, optional(&path, MAX_DOCUMENT)?.as_deref()) == version,
            "instructions changed; reload the project file"
        );
        match contents {
            Some(contents) => replace(&path, contents),
            None => Ok(fs::remove_file(&path)?),
        }
    };
    let after = match text {
        Some(text) => {
            ensure!(
                text.len() as u64 <= MAX_DOCUMENT,
                "instructions exceed the {MAX_DOCUMENT}-byte limit"
            );
            let content = updated(before.as_deref().unwrap_or_default(), text, section)?;
            ensure!(
                content.len() as u64 <= MAX_DOCUMENT,
                "instruction file would exceed the {MAX_DOCUMENT}-byte limit"
            );
            let created = before.is_none() || index.is_some_and(|i| entries[i].created);
            let result = (!(created && content.is_empty())).then_some(content);
            if before == result {
                return Ok(());
            }
            // Persist the backup first. An interrupted write can be retried, and an
            // unapplied backup is only visible if its resulting file revision exists.
            let after = revision(&path, result.as_deref());
            entries.push(Backup {
                after,
                before,
                created,
            });
            if entries.len() > MAX_BACKUPS {
                entries.drain(..entries.len() - MAX_BACKUPS);
            }
            fs::create_dir_all(record.parent().unwrap())?;
            replace(
                &record,
                &nextjson::to_string(&entries).map_err(anyhow::Error::msg)?,
            )?;
            result
        }
        None => {
            let Some(i) = index else {
                bail!("no backup matches the current file");
            };
            entries[i].before.clone()
        }
    };
    write_document(after.as_deref())?;
    if let (None, Some(index)) = (text, index) {
        entries.truncate(index);
        replace(
            &record,
            &nextjson::to_string(&entries).map_err(anyhow::Error::msg)?,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_existing_agent_files_and_respects_overrides() -> Result<()> {
        let project = tempfile::tempdir()?;
        let data = tempfile::tempdir()?;
        let root = project.path().canonicalize()?;
        fs::create_dir(root.join(".claude"))?;
        fs::write(root.join(".claude/CLAUDE.md"), "# Claude")?;
        fs::write(root.join("AGENTS.md"), "# Shared")?;
        fs::write(root.join("AGENTS.override.md"), "# Override")?;
        for (agent, name) in [
            ("claude", ".claude/CLAUDE.md"),
            ("opencode", "AGENTS.md"),
            ("codex", "AGENTS.override.md"),
            ("omp", "AGENTS.override.md"),
        ] {
            let name = root.join(name);
            assert_eq!(target(data.path(), &root, agent)?, name);
        }
        assert!(updated("", &format!("Useful text\n{END}\r"), Section::Instructions).is_err());
        Ok(())
    }

    #[test]
    fn uses_configured_instruction_names_and_rejects_escapes() -> Result<()> {
        let project = tempfile::tempdir()?;
        let data = tempfile::tempdir()?;
        let root = project.path().canonicalize()?;
        fs::create_dir(root.join("guide"))?;
        for (agent, folder) in [
            ("gemini", ".gemini"),
            ("qwen", ".qwen"),
            ("qoder", ".qoder"),
            ("qoder-cn", ".qoder-cn"),
        ] {
            fs::create_dir(root.join(folder))?;
            fs::write(
                root.join(folder).join("settings.json"),
                format!(
                    r#"{{ /* project */ "context": {{ "fileName": [null, " ", "guide/{agent}.md",], }}}}"#
                ),
            )?;
            let view = read(data.path(), &root, agent)?;
            let target = root.join("guide").join(format!("{agent}.md"));
            assert_eq!(Path::new(&view.path), target);
            edit(
                data.path(),
                &root,
                agent,
                &view.revision,
                Some("Custom instructions"),
            )?;
            assert!(fs::read_to_string(target)?.contains("Custom instructions"));
        }
        fs::write(
            root.join(".qwen/settings.json"),
            r#"{"context":{"fileName":"../outside.md"}}"#,
        )?;
        assert!(read(data.path(), &root, "qwen").is_err());
        Ok(())
    }

    #[test]
    fn shared_target_preserves_crlf_and_rejects_stale_snapshots() -> Result<()> {
        let project = tempfile::tempdir()?;
        let data = tempfile::tempdir()?;
        let path = project.path().join("AGENTS.md");
        let initial = "\u{feff}# Project\r\n\r\nUser instructions.\r\n";
        fs::write(&path, initial)?;
        let original = read(data.path(), project.path(), "codex")?;
        edit(
            data.path(),
            project.path(),
            "codex",
            &original.revision,
            Some("    code\n\nSecond line  "),
        )?;
        let saved = read(data.path(), project.path(), "cursor")?;
        assert_eq!(saved.text, "    code\n\nSecond line  ");
        assert!(saved.can_restore);
        assert!(fs::read_to_string(&path)?.starts_with(initial));
        assert!(
            edit(
                data.path(),
                project.path(),
                "codex",
                &original.revision,
                None
            )
            .is_err()
        );
        edit(data.path(), project.path(), "cursor", &saved.revision, None)?;
        assert_eq!(fs::read_to_string(&path)?, initial);
        Ok(())
    }

    #[test]
    fn project_memory_publication_is_scoped_reversible_and_checks_both_revisions() -> Result<()> {
        use crate::memory::{self, Changes};

        let project = tempfile::tempdir()?;
        let data = tempfile::tempdir()?;
        let root = project.path();
        let dir = data.path();
        let path = root.join("AGENTS.md");
        fs::write(&path, "\u{feff}# My instructions\r\n")?;
        let original = read(dir, root, "codex")?;
        edit(
            dir,
            root,
            "codex",
            &original.revision,
            Some("Run the tests."),
        )?;
        let before = read(dir, root, "codex")?;
        for (scope, text) in [
            ("global".into(), "A global note."),
            ("agent:codex".into(), "An agent note."),
            ("project:other/example".into(), "Another project's note."),
        ] {
            memory::save(
                dir,
                None,
                None,
                Changes {
                    scope: Some(scope),
                    text: Some(text.into()),
                    ..Default::default()
                },
            )?;
        }
        assert!(memory::project_digest(dir, root)?.is_empty());
        let note = memory::save(
            dir,
            None,
            None,
            Changes {
                scope: Some(format!("project:{}", memory::project_key(root)?)),
                text: Some(format!("Verify the build.\n{MEMORY_END}")),
                pinned: Some(true),
                ..Default::default()
            },
        )?;
        let digest = memory::project_digest(dir, root)?;
        assert!(publish_memory(dir, root, "codex", &before.revision, Some("")).is_err());
        assert!(publish_memory(dir, root, "codex", &original.revision, Some(&digest)).is_err());
        assert_eq!(read(dir, root, "codex")?.revision, before.revision);
        publish_memory(dir, root, "codex", &before.revision, Some(&digest))?;
        let published = read(dir, root, "codex")?;
        assert_eq!(published.text, before.text);
        assert_eq!(published.memory_digest, digest.trim_end());
        let source = fs::read_to_string(&path)?;
        assert!(source.starts_with("\u{feff}# My instructions\r\n"));
        assert!(source.contains("Verify the build.\r\n"));
        assert_eq!(source.matches(MEMORY_END).count(), 1);
        assert!(!source.contains(" note."));
        memory::save(
            dir,
            Some(&note.id),
            Some(&note.revision),
            Changes {
                status: Some("archived".into()),
                ..Default::default()
            },
        )?;
        publish_memory(dir, root, "codex", &published.revision, Some(""))?;
        let cleared = read(dir, root, "codex")?;
        assert_eq!(cleared.revision, before.revision);
        edit(dir, root, "codex", &cleared.revision, None)?;
        assert_eq!(
            read(dir, root, "codex")?.memory_digest,
            published.memory_digest
        );
        Ok(())
    }

    #[test]
    fn created_file_can_be_cleared_and_restored_in_order() -> Result<()> {
        let project = tempfile::tempdir()?;
        let data = tempfile::tempdir()?;
        let project = project.path();
        let data = data.path();
        let empty = read(data, project, "codex")?;
        edit(data, project, "codex", &empty.revision, Some("Use tests."))?;
        let first = read(data, project, "codex")?;
        edit(data, project, "codex", &first.revision, Some(""))?;
        let cleared = read(data, project, "codex")?;
        assert!(!project.join("AGENTS.md").exists());
        edit(data, project, "codex", &cleared.revision, None)?;
        assert_eq!(read(data, project, "codex")?.text, "Use tests.");
        edit(data, project, "codex", &first.revision, None)?;
        assert!(!read(data, project, "codex")?.can_restore);
        assert!(!project.join("AGENTS.md").exists());
        Ok(())
    }
}
