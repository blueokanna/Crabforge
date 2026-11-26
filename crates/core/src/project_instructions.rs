use anyhow::{Context, Result, bail, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{ErrorKind, Read, Write},
    ops::Range,
    path::{Path, PathBuf},
};

// Keeping Magpie's markers lets an existing project move between the two editors.
const BEGIN: &str = "<!-- magpie:begin · written by magpie from its Library -->";
const END: &str = "<!-- magpie:end -->";
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

fn target(project: &Path, agent: &str) -> Result<PathBuf> {
    let agent = if agent == "omp" { "pi" } else { agent };
    let root = project
        .canonicalize()
        .context("cannot open the project folder")?;
    ensure!(root.is_dir(), "expected a project folder");
    let (_, names) = AGENTS
        .iter()
        .find(|(id, _)| *id == agent)
        .context("unsupported project instructions agent")?;
    for name in *names {
        let path = root.join(name);
        match fs::symlink_metadata(&path) {
            Ok(_) => {
                let resolved = path.canonicalize()?;
                ensure!(
                    resolved.starts_with(&root) && resolved.is_file(),
                    "instructions must be a file inside the project"
                );
                return Ok(resolved);
            }
            Err(e) if e.kind() == ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    let default = if matches!(agent, "codex" | "pi") {
        "AGENTS.md"
    } else {
        names[0]
    };
    Ok(root.join(default))
}

struct Block {
    whole: Range<usize>,
    body: Range<usize>,
}

fn block(source: &str) -> Result<Option<Block>> {
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
            BEGIN => {
                ensure!(
                    begin.is_none() && found.is_none(),
                    "multiple instruction blocks"
                );
                begin = Some((pos, end));
            }
            END => {
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

fn updated(source: &str, text: &str) -> Result<String> {
    let old = block(source)?;
    let text = text
        .strip_prefix('\u{feff}')
        .unwrap_or(text)
        .replace("\r\n", "\n");
    let text = text.trim_end_matches('\n');
    ensure!(
        !text
            .split('\n')
            .any(|line| matches!(line.trim_end_matches('\r'), BEGIN | END)),
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
            result.push_str(&format!("{BEGIN}{eol}{text}{eol}{END}{eol}"));
        }
    }
    Ok(result)
}

#[derive(NsonSerialize)]
pub struct View {
    pub path: String,
    pub text: String,
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

fn replace(path: &Path, text: &str) -> Result<()> {
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
    let path = target(project, agent)?;
    let contents = optional(&path, MAX_DOCUMENT)?;
    let source = contents.as_deref().unwrap_or_default();
    let text = block(source)?
        .map(|b| {
            source[b.body]
                .trim_end_matches(['\r', '\n'])
                .replace("\r\n", "\n")
        })
        .unwrap_or_default();
    let revision = revision(&path, contents.as_deref());
    let (_, entries) = history(dir, &path)?;
    Ok(View {
        path: path.display().to_string(),
        text,
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
    let store = dir.join("project-instructions");
    fs::create_dir_all(&store)?;
    let lock = fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(store.join("library.lock"))?;
    lock.lock()?;
    let path = target(project, agent)?;
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
            target(project, agent)? == path
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
            let content = updated(before.as_deref().unwrap_or_default(), text)?;
            ensure!(
                content.len() as u64 <= MAX_DOCUMENT,
                "project file would exceed the {MAX_DOCUMENT}-byte limit"
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
            ("gemini", "GEMINI.md"),
            ("qwen", "QWEN.md"),
        ] {
            let name = root.join(name);
            assert_eq!(target(&root, agent)?, name);
        }
        assert!(updated("", &format!("Useful text\n{END}\r")).is_err());
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
