use super::{Entry, Kind, MAX_DOCUMENT, Store, discovery, encode, hash, path_key};
use crate::skills::{is_repository_url, visit_repository_markdown};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Read,
    path::Path,
};

const MAX_FILES: usize = 5000;
const MAX_PATHS: usize = 32768;
const MAX_CANDIDATES: usize = 512;
const MAX_DEPTH: usize = 8;
const MAX_SOURCE: usize = 16 * 1024 * 1024;

#[derive(NsonDeserialize)]
pub struct Request {
    pub source: String,
    files: Vec<Selection>,
    #[nextjson(default)]
    agents: Vec<String>,
}

#[derive(NsonDeserialize)]
struct Selection {
    path: String,
    revision: String,
}

impl Request {
    pub fn from_json(source: &str) -> Result<Self> {
        ensure!(source.len() <= MAX_DOCUMENT, "import request exceeds 1 MiB");
        let source = source.strip_prefix('\u{feff}').unwrap_or(source);
        let request: Self = nextjson::from_str(source).map_err(anyhow::Error::msg)?;
        ensure!(
            (1..=MAX_CANDIDATES).contains(&request.files.len()),
            "select between 1 and {MAX_CANDIDATES} subagent files"
        );
        Ok(request)
    }
}

#[derive(NsonDeserialize, NsonSerialize)]
pub(super) struct Origin {
    key: String,
    pub(super) source: String,
    baseline: String,
}

struct SourceFile {
    path: String,
    key: String,
    source: String,
    revision: String,
    entry: Entry,
}

#[derive(Default)]
pub(super) struct Scan {
    candidates: Vec<SourceFile>,
    skill_folders: BTreeSet<String>,
    files: usize,
    paths: usize,
    bytes: usize,
    truncated: bool,
}

fn name(input: &str) -> String {
    let name = input
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .to_ascii_lowercase();
    name[..name.len().min(50)].trim_end_matches('-').to_owned()
}

fn entry(path: &str, local_folder: Option<&str>, raw: &str) -> Option<Entry> {
    let normalized = discovery::normalize(raw);
    let (front, body) = discovery::markdown(&normalized).ok()?;
    let front = front?;
    let field = |key| front.as_mapping_get(key).and_then(|value| value.as_str());
    let description = field("description")?
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let explicit = field("name").map(name).unwrap_or_default();
    let name = if explicit.is_empty() {
        let (parent, filename) = path.rsplit_once('/').unwrap_or(("", path));
        let folder = local_folder.unwrap_or_else(|| parent.rsplit('/').next().unwrap_or_default());
        if !matches!(
            folder.to_ascii_lowercase().as_str(),
            "agent" | "agents" | "droids"
        ) {
            return None;
        }
        let (stem, _) = filename.rsplit_once('.')?;
        name(stem.strip_suffix(".agent").unwrap_or(stem))
    } else {
        explicit
    };
    let mut entry = Entry {
        name,
        description,
        prompt: body.trim_matches('\n').to_owned(),
        agents: Vec::new(),
        model: None,
        effort: None,
        skills: Vec::new(),
        mcp: Vec::new(),
    };
    entry.validate(Kind::Subagent, &[]).ok()?;
    Some(entry)
}

fn markdown_file(path: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("md"))
}

impl Scan {
    pub(super) fn open(source: &str) -> Result<Self> {
        let source = source.trim();
        ensure!(
            !source.is_empty() && source.len() <= 4096 && !source.contains('\0'),
            "enter a local Markdown file, folder or Git URL"
        );
        let mut scan = Self::default();
        if is_repository_url(source) {
            visit_repository_markdown(source, |path, url, content| {
                scan.inspect(path, url.clone(), url, None, content)
            })?;
        } else {
            let root = fs::canonicalize(source).context("open subagent source")?;
            let metadata = fs::metadata(&root)?;
            if metadata.is_dir() {
                scan.folder(&root, &root, 0)?;
            } else {
                ensure!(metadata.is_file(), "expected a regular Markdown file");
                scan.local_file(&root, &root)?;
            }
        }
        scan.candidates.retain(|file| {
            let mut path = file.path.as_str();
            loop {
                let folder = path.rsplit_once('/').map_or("", |(parent, _)| parent);
                if scan.skill_folders.contains(folder) {
                    return false;
                }
                if folder.is_empty() {
                    return true;
                }
                path = folder;
            }
        });
        scan.candidates.sort_by(|left, right| {
            let order = |file: &SourceFile| file.path.bytes().filter(|&byte| byte == b'/').count();
            order(left)
                .cmp(&order(right))
                .then(left.path.cmp(&right.path))
        });
        let mut names = BTreeSet::new();
        scan.candidates
            .retain(|file| names.insert(file.entry.name.clone()));
        if scan.candidates.len() > MAX_CANDIDATES {
            scan.candidates.truncate(MAX_CANDIDATES);
            scan.truncated = true;
        }
        scan.candidates
            .sort_by(|left, right| left.path.cmp(&right.path));
        Ok(scan)
    }

    fn folder(&mut self, root: &Path, folder: &Path, depth: usize) -> Result<()> {
        if folder.join("SKILL.md").is_file() {
            return Ok(());
        }
        if depth > MAX_DEPTH {
            self.truncated = true;
            return Ok(());
        }
        for item in fs::read_dir(folder)? {
            if self.paths == MAX_PATHS || self.files == MAX_FILES || self.bytes >= MAX_SOURCE {
                self.truncated = true;
                break;
            }
            self.paths += 1;
            let item = item?;
            let metadata = fs::symlink_metadata(item.path())?;
            if crate::skills::is_link(&metadata) {
                continue;
            }
            if metadata.is_dir() {
                if !matches!(
                    item.file_name().to_str(),
                    Some(".git" | ".github" | "node_modules" | "vendor")
                ) {
                    self.folder(root, &item.path(), depth + 1)?;
                }
            } else if metadata.is_file() {
                self.local_file(root, &item.path())?;
            }
        }
        Ok(())
    }

    fn local_file(&mut self, root: &Path, file: &Path) -> Result<()> {
        let path = if root == file {
            Path::new(file.file_name().context("missing source filename")?)
        } else {
            file.strip_prefix(root)?
        };
        let Some(path) = path.to_str() else {
            return Ok(());
        };
        if !markdown_file(path)
            || file
                .parent()
                .is_some_and(|parent| parent.join("SKILL.md").is_file())
        {
            return Ok(());
        }
        let source = file.to_str().context("subagent source path is not UTF-8")?;
        let folder = file
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str());
        self.inspect(
            &path.replace('\\', "/"),
            path_key(source),
            source.to_owned(),
            folder,
            &mut File::open(file)?,
        )
    }

    fn inspect(
        &mut self,
        path: &str,
        key: String,
        source: String,
        local_folder: Option<&str>,
        input: &mut dyn Read,
    ) -> Result<()> {
        let (folder, file) = path.rsplit_once('/').unwrap_or(("", path));
        if file.eq_ignore_ascii_case("SKILL.md") {
            self.skill_folders.insert(folder.to_owned());
            return Ok(());
        }
        if folder
            .split('/')
            .any(|part| matches!(part, ".git" | ".github" | "node_modules" | "vendor"))
        {
            return Ok(());
        }
        if self.files == MAX_FILES || self.bytes >= MAX_SOURCE {
            self.truncated = true;
            return Ok(());
        }
        self.files += 1;
        let mut bytes = Vec::new();
        input
            .take(MAX_DOCUMENT as u64 + 1)
            .read_to_end(&mut bytes)?;
        self.bytes += bytes.len();
        if bytes.len() <= MAX_DOCUMENT
            && let Ok(raw) = std::str::from_utf8(&bytes)
            && let Some(entry) = entry(path, local_folder, raw)
        {
            self.candidates.push(SourceFile {
                path: path.to_owned(),
                key,
                source,
                revision: hash(raw),
                entry,
            });
        }
        Ok(())
    }
}

fn baseline(entry: &Entry) -> String {
    hash(&format!("{}\0{}", entry.description, entry.prompt))
}

fn existing<'a>(store: &'a Store, file: &SourceFile) -> Option<(&'a Entry, &'a Origin)> {
    store.imported.iter().find_map(|(name, origin)| {
        if origin.key == file.key {
            Some((store.entries.get(name)?, origin))
        } else {
            None
        }
    })
}

#[derive(NsonSerialize)]
struct Preview<'a> {
    candidates: Vec<Candidate<'a>>,
    agents: &'a [&'a str],
    truncated: bool,
}

#[derive(NsonSerialize)]
struct Candidate<'a> {
    path: &'a str,
    name: &'a str,
    description: &'a str,
    revision: &'a str,
    status: &'static str,
}

pub(super) fn preview(scan: &Scan, store: &Store, agents: &[&str]) -> Result<String> {
    let candidates = scan
        .candidates
        .iter()
        .map(|file| {
            let (name, status) = match existing(store, file) {
                Some((entry, _)) if baseline(entry) == baseline(&file.entry) => {
                    (&entry.name, "unchanged")
                }
                Some((entry, origin)) if baseline(entry) == origin.baseline => {
                    (&entry.name, "update")
                }
                Some((entry, _)) => (&entry.name, "edited"),
                None => (&file.entry.name, "new"),
            };
            Candidate {
                path: &file.path,
                name,
                description: &file.entry.description,
                revision: &file.revision,
                status,
            }
        })
        .collect();
    encode(&Preview {
        candidates,
        agents,
        truncated: scan.truncated,
    })
}

pub(super) fn apply(
    scan: Scan,
    request: Request,
    store: &mut Store,
    agents: &[&str],
) -> Result<()> {
    let mut files: BTreeMap<_, _> = scan
        .candidates
        .into_iter()
        .map(|file| (file.path.clone(), file))
        .collect();
    for Selection { path, revision } in request.files {
        let file = files
            .remove(&path)
            .context("subagent source file was removed; scan again")?;
        ensure!(
            file.revision == revision,
            "subagent source file changed; scan again"
        );
        let mut entry = file.entry.clone();
        let baseline = baseline(&entry);
        if let Some((saved, origin)) = existing(store, &file) {
            if self::baseline(saved) != origin.baseline || baseline == origin.baseline {
                continue;
            }
            let mut updated = saved.clone();
            updated.description = entry.description;
            updated.prompt = entry.prompt;
            entry = updated;
        } else {
            let base = entry.name.clone();
            for index in 2.. {
                if !store.entries.contains_key(&entry.name) {
                    break;
                }
                let suffix = format!("-{index}");
                entry.name = format!(
                    "{}{}",
                    base[..base.len().min(50 - suffix.len())].trim_end_matches('-'),
                    suffix
                );
            }
            entry.agents = request.agents.clone();
        }
        entry.validate(Kind::Subagent, agents)?;
        store.imported.insert(
            entry.name.clone(),
            Origin {
                key: file.key,
                source: file.source,
                baseline,
            },
        );
        store.entries.insert(entry.name.clone(), entry);
    }
    Ok(())
}
