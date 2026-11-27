use crate::{
    config::read_limited,
    project_instructions::replace,
    skills::{
        is_repository_url, load_repository_tree, read_plugin_archive,
        tree::{Tree, is_link, windows_filename},
    },
};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

const CLAUDE: &str = ".claude-plugin/plugin.json";
const CODEX: &str = ".codex-plugin/plugin.json";
const ANTIGRAVITY: &str = ".antigravity-plugin/plugin.json";
const PI: &str = "package.json";
const MARKET: &str = ".claude-plugin/marketplace.json";
const MAX_INDEX: usize = 8 * 1024 * 1024;
const MAX_MANIFEST: usize = 256 * 1024;

mod claude_project;
mod marketplace;
mod pi;
mod pi_project;
mod project;
mod resources;
mod workspace_project;

pub use project::Edit as ProjectEdit;
pub use project::Edit as PiProjectEdit;
pub use workspace_project::Agent as WorkspaceAgent;

pub enum Edit<'a> {
    Read,
    Preview {
        source: &'a str,
    },
    Marketplace {
        source: &'a str,
    },
    Save {
        source: &'a str,
        content: &'a str,
        revision: Option<&'a str>,
    },
    Remove {
        name: &'a str,
        revision: &'a str,
    },
    Export {
        name: &'a str,
        revision: &'a str,
        output: &'a Path,
    },
    Resources {
        name: &'a str,
        revision: &'a str,
        kind: &'a str,
        path: &'a str,
        copy: Option<&'a str>,
    },
    PiProject {
        project: &'a Path,
        edit: PiProjectEdit<'a>,
    },
    ClaudeProject {
        project: &'a Path,
        edit: ProjectEdit<'a>,
    },
    WorkspaceProject {
        agent: WorkspaceAgent,
        project: &'a Path,
        edit: ProjectEdit<'a>,
    },
}

#[derive(NsonDeserialize, NsonSerialize)]
struct Bundle {
    name: String,
    description: String,
    version: String,
    source: String,
    content: String,
    manifests: Vec<String>,
    components: BTreeMap<String, Vec<String>>,
    other_manifest_entries: Vec<String>,
    files: usize,
}

#[derive(NsonSerialize)]
struct Entry<'a> {
    bundle: &'a Bundle,
    revision: String,
    folder: String,
}

fn encode(value: &impl NsonSerialize) -> Result<String> {
    nextjson::to_string(value).map_err(anyhow::Error::msg)
}

fn revision(bundle: &Bundle) -> Result<String> {
    Ok(hex::encode(Sha256::digest(encode(bundle)?)))
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && windows_filename(name)
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
}

fn local_root(source: &str) -> Result<PathBuf> {
    let path = Path::new(source);
    let root = if path.is_file() && path.file_name().is_some_and(|name| name == PI) {
        path.parent().context("Pi package needs a folder")?
    } else if path.is_file()
        && path
            .file_name()
            .is_some_and(|name| name == "plugin.json" || name == "marketplace.json")
    {
        let parent = path.parent().context("plugin manifest needs a folder")?;
        ensure!(
            matches!(
                parent.file_name().and_then(|name| name.to_str()),
                Some(".claude-plugin" | ".codex-plugin" | ".antigravity-plugin")
            ),
            "expected a Claude Code, Codex or Antigravity plugin manifest"
        );
        parent
            .parent()
            .context("plugin manifest needs a bundle folder")?
    } else {
        path
    };
    let root = if root.as_os_str().is_empty() {
        Path::new(".")
    } else {
        root
    };
    ensure!(
        !is_link(&fs::symlink_metadata(root)?),
        "select the plugin's original folder"
    );
    root.canonicalize().context("open the plugin bundle folder")
}

fn load(source: &str) -> Result<(Bundle, Tree)> {
    let source = source.trim();
    ensure!(!source.is_empty(), "choose a plugin folder or Git URL");
    let (source, tree) = if is_repository_url(source) {
        load_repository_tree(source, &[CLAUDE, CODEX, ANTIGRAVITY, PI])?
    } else {
        let path = Path::new(source);
        let (path, tree) = if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("tar"))
            && path.is_file()
        {
            let path = path.canonicalize()?;
            let tree = read_plugin_archive(&path)?;
            (path, tree)
        } else {
            let root = local_root(source)?;
            let tree = Tree::read_repository(&root)?;
            (root, tree)
        };
        let source = path
            .into_os_string()
            .into_string()
            .ok()
            .context("plugin path must be UTF-8")?;
        (source, tree)
    };
    let bundle = inspect(&tree, source)?;
    Ok((bundle, tree))
}

fn relative(text: &str) -> Option<PathBuf> {
    let path = Path::new(text);
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::Normal(name) if name.to_str().is_some_and(windows_filename) => {
                result.push(name)
            }
            _ => return None,
        }
    }
    (!result.as_os_str().is_empty()).then_some(result)
}

fn display(path: &Path) -> String {
    path.iter()
        .map(|part| part.to_str().expect("validated plugin path"))
        .collect::<Vec<_>>()
        .join("/")
}

fn inspect(tree: &Tree, source: String) -> Result<Bundle> {
    let mut docs = Vec::new();
    for path in [CLAUDE, CODEX, ANTIGRAVITY] {
        if let Some(text) = tree.text(Path::new(path))? {
            ensure!(
                text.len() <= MAX_MANIFEST,
                "plugin manifest exceeds 256 KiB: {path}"
            );
            let text = text.strip_prefix('\u{feff}').unwrap_or(text);
            let doc: Value = nextjson::from_str(text)
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("read {path}"))?;
            ensure!(doc.is_object(), "expected a plugin manifest object: {path}");
            docs.push((path, doc));
        }
    }
    if docs.is_empty() {
        return pi::inspect(tree, source);
    }
    let primary = &docs[0].1;
    let str_field = |field: &str| {
        primary
            .get(field)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
    };
    let name = str_field("name");
    ensure!(
        valid_name(name),
        "plugin names use 1–128 letters, digits, hyphens or underscores"
    );
    let mut found = BTreeMap::<String, BTreeSet<String>>::new();
    let mut other = docs
        .iter()
        .flat_map(|(path, doc)| {
            ["lspServers", "outputStyles", "themes", "monitors"]
                .into_iter()
                .filter(|key| doc.get(key).is_some())
                .map(move |key| format!("{path} / {key}"))
        })
        .collect::<BTreeSet<_>>();
    let files = tree.files().collect::<Vec<_>>();
    for (kind, field, defaults) in [
        ("skills", "skills", &["skills"][..]),
        ("subagents", "agents", &["agents"][..]),
        ("hooks", "hooks", &["hooks/hooks.json"][..]),
        ("mcp", "mcpServers", &[".mcp.json"][..]),
        ("commands", "commands", &["commands"][..]),
    ] {
        let entries = found.entry(kind.to_owned()).or_default();
        for default in defaults.iter().copied().chain(
            (kind == "hooks" && docs.iter().any(|(path, _)| *path == ANTIGRAVITY))
                .then_some("hooks.json"),
        ) {
            collect(&files, kind, Path::new(default), entries);
        }
        for (path, doc) in &docs {
            let Some(value) = doc.get(field) else {
                continue;
            };
            let sources = value
                .as_array()
                .map_or_else(|| std::slice::from_ref(value), Vec::as_slice);
            for value in sources {
                if let Some(text) = value.as_str() {
                    if let Some(relative) = relative(text)
                        && collect(&files, kind, &relative, entries)
                    {
                        continue;
                    }
                    other.insert(format!("{path} / {field} / {text}"));
                } else if matches!(kind, "hooks" | "mcp") && value.is_object() {
                    entries.insert(format!("{path} / {field}"));
                } else {
                    other.insert(format!("{path} / {field}"));
                }
            }
        }
    }
    Ok(Bundle {
        name: name.to_owned(),
        description: str_field("description").to_owned(),
        version: str_field("version").to_owned(),
        source,
        content: tree.hash(),
        manifests: docs.iter().map(|(path, _)| (*path).to_owned()).collect(),
        components: found
            .into_iter()
            .filter(|(_, paths)| !paths.is_empty())
            .map(|(kind, paths)| (kind, paths.into_iter().collect()))
            .collect(),
        other_manifest_entries: other.into_iter().collect(),
        files: files.len(),
    })
}

fn collect(files: &[&Path], kind: &str, root: &Path, entries: &mut BTreeSet<String>) -> bool {
    let mut has_entries = false;
    for &file in files.iter().filter(|file| file.starts_with(root)) {
        if component_file(kind, file) {
            entries.insert(display(file));
            has_entries = true;
        }
    }
    has_entries
}

fn component_file(kind: &str, file: &Path) -> bool {
    match kind {
        "skills" => file.file_name().is_some_and(|name| name == "SKILL.md"),
        "subagents" | "commands" => {
            !file
                .file_stem()
                .is_some_and(|stem| stem.eq_ignore_ascii_case("readme"))
                && file.extension().is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("md")
                        || kind == "commands" && ext.eq_ignore_ascii_case("toml")
                })
        }
        _ => file
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json")),
    }
}

fn check_revision<'a>(
    bundles: &'a BTreeMap<String, Bundle>,
    name: &str,
    expected: &str,
) -> Result<&'a Bundle> {
    let entry = bundles
        .get(&name.to_ascii_lowercase())
        .context("plugin bundle was removed")?;
    ensure!(
        revision(entry)? == expected,
        "plugin bundle changed; reload it"
    );
    Ok(entry)
}

fn snapshot(root: &Path, content: &str, tree: &Tree) -> Result<()> {
    fs::create_dir_all(root)?;
    let folder = root.join(content);
    match fs::symlink_metadata(&folder) {
        Ok(_) => ensure!(
            Tree::read(&folder)?.hash() == content,
            "saved plugin snapshot was modified"
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let stage = tempfile::tempdir_in(root)?;
            let files = stage.path().join("files");
            tree.write(&files)?;
            fs::rename(files, folder)?;
        }
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

fn export(bundle: &Bundle, folder: &Path, output: &Path) -> Result<String> {
    let tree = Tree::read(folder)?;
    ensure!(
        tree.hash() == bundle.content,
        "saved plugin snapshot was modified"
    );
    export_tree(&tree, &bundle.content, output)
}

fn export_tree(tree: &Tree, content: &str, output: &Path) -> Result<String> {
    ensure!(
        output
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("tar")),
        "use a .tar filename for the plugin archive"
    );
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    tree.write_plugin_archive(temp.as_file_mut())?;
    temp.as_file().sync_all()?;
    temp.persist_noclobber(output)
        .with_context(|| format!("save plugin archive to {}", output.display()))?;
    encode(&content)
}

pub fn edit(dir: &Path, operation: Edit<'_>) -> Result<String> {
    let loaded = match operation {
        Edit::Preview { source } => return encode(&load(source)?.0),
        Edit::Marketplace { source } => return marketplace::browse(source),
        Edit::Save { source, .. } => Some(load(source)?),
        _ => None,
    };
    let library = dir.join("plugin-bundles");
    fs::create_dir_all(&library)?;
    let lock = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(library.join("library.lock"))?;
    lock.lock()?;
    let index = library.join("index.json");
    let snapshots = library.join("objects");
    let mut bundles: BTreeMap<String, Bundle> = match read_limited(&index, MAX_INDEX) {
        Ok(bytes) => nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?,
        Err(error) if error.kind() == ErrorKind::NotFound => BTreeMap::new(),
        Err(error) => return Err(error.into()),
    };
    for bundle in bundles.values() {
        ensure!(
            bundle.content.len() == 64
                && bundle.content.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "invalid saved plugin snapshot hash"
        );
    }
    match operation {
        Edit::Read => {}
        Edit::Preview { .. } | Edit::Marketplace { .. } => unreachable!(),
        Edit::Save {
            content,
            revision: expected,
            ..
        } => {
            let (bundle, tree) = loaded.expect("plugin source loaded for save");
            ensure!(
                bundle.content == content,
                "plugin source changed; preview it again"
            );
            let key = bundle.name.to_ascii_lowercase();
            if let Some(expected) = expected {
                check_revision(&bundles, &key, expected)?;
            } else {
                ensure!(
                    !bundles.contains_key(&key),
                    "a plugin bundle with this name already exists"
                );
            }
            snapshot(&snapshots, content, &tree)?;
            bundles.insert(key, bundle);
        }
        Edit::Remove { name, revision } => {
            check_revision(&bundles, name, revision)?;
            bundles.remove(&name.to_ascii_lowercase());
        }
        Edit::Export {
            name,
            revision,
            output,
        } => {
            let bundle = check_revision(&bundles, name, revision)?;
            return export(bundle, &snapshots.join(&bundle.content), output);
        }
        Edit::Resources {
            name,
            revision,
            kind,
            path,
            copy,
        } => {
            let bundle = check_revision(&bundles, name, revision)?;
            return resources::edit(
                dir,
                bundle,
                &snapshots.join(&bundle.content),
                kind,
                path,
                copy,
            );
        }
        Edit::PiProject { project, edit } => {
            return pi_project::edit(dir, &library, &snapshots, &bundles, project, edit);
        }
        Edit::ClaudeProject { project, edit } => {
            return claude_project::edit(dir, &library, &snapshots, &bundles, project, edit);
        }
        Edit::WorkspaceProject {
            agent,
            project,
            edit,
        } => {
            return workspace_project::edit(
                agent, dir, &library, &snapshots, &bundles, project, edit,
            );
        }
    }
    if !matches!(operation, Edit::Read) {
        let data = encode(&bundles)?;
        ensure!(
            data.len() <= MAX_INDEX,
            "plugin bundle catalog exceeds 8 MiB"
        );
        replace(&index, &data)?;
    }
    let entries = bundles
        .values()
        .map(|bundle| {
            Ok(Entry {
                bundle,
                revision: revision(bundle)?,
                folder: snapshots
                    .join(&bundle.content)
                    .into_os_string()
                    .into_string()
                    .ok()
                    .context("plugin library path must be UTF-8")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    encode(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_and_keeps_complete_bundles_with_revision_checks() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().join("source");
        let data = temp.path().join("data");
        for name in [".git", ".claude-plugin", "skills/check", "agents"] {
            fs::create_dir_all(root.join(name))?;
        }
        fs::write(root.join(".git/config"), "ignored")?;
        fs::write(
            root.join(CLAUDE),
            r#"{"name":"reviewer","mcpServers":{"sample":{}},"skills":["../../escape"]}"#,
        )?;
        fs::write(root.join("skills/check/SKILL.md"), "skill")?;
        fs::write(root.join("skills/check/data.bin"), [0, 255, 42])?;
        fs::write(root.join("agents/review.md"), "agent")?;
        let source = root.to_str().context("fixture path must be UTF-8")?;
        let bundle: Bundle = nextjson::from_str(&edit(&data, Edit::Preview { source })?)
            .map_err(anyhow::Error::msg)?;
        assert_eq!(bundle.components["skills"], ["skills/check/SKILL.md"]);
        assert_eq!(bundle.components["subagents"], ["agents/review.md"]);
        assert_eq!(
            bundle.components["mcp"],
            [".claude-plugin/plugin.json / mcpServers"]
        );
        assert_eq!(
            bundle.other_manifest_entries,
            [".claude-plugin/plugin.json / skills / ../../escape"]
        );
        assert_eq!(bundle.files, 4);
        let save = Edit::Save {
            source,
            content: &bundle.content,
            revision: None,
        };
        fs::write(root.join("skills/check/data.bin"), [1, 255, 42])?;
        assert!(edit(&data, save).is_err());
        fs::write(root.join("skills/check/data.bin"), [0, 255, 42])?;
        let output = edit(
            &data,
            Edit::Save {
                source,
                content: &bundle.content,
                revision: None,
            },
        )?;
        let value: Value = nextjson::from_str(&output).map_err(anyhow::Error::msg)?;
        let entry = value
            .as_array()
            .context("expected bundle rows")?
            .first()
            .context("missing bundle row")?;
        let folder = Path::new(
            entry
                .get("folder")
                .and_then(Value::as_str)
                .context("missing folder")?,
        );
        assert_eq!(
            fs::read(folder.join("skills/check/data.bin"))?,
            [0, 255, 42]
        );
        assert!(!folder.join(".git").exists());
        let revision = entry
            .get("revision")
            .and_then(Value::as_str)
            .context("missing revision")?;
        let archive = temp.path().join("carried.tar");
        let export = || Edit::Export {
            name: "reviewer",
            revision,
            output: &archive,
        };
        assert_eq!(edit(&data, export())?, encode(&bundle.content)?);
        assert!(edit(&data, export()).is_err());
        let source = archive.to_str().context("fixture path must be UTF-8")?;
        let moved = temp.path().join("moved");
        let (carried, _) = load(source)?;
        assert_eq!(carried.content, bundle.content);
        assert_eq!(carried.components, bundle.components);
        let imported: Value = nextjson::from_str(&edit(
            &moved,
            Edit::Save {
                source,
                content: &carried.content,
                revision: None,
            },
        )?)
        .map_err(anyhow::Error::msg)?;
        let imported = Path::new(
            imported[0]["folder"]
                .as_str()
                .context("missing imported folder")?,
        );
        assert_eq!(
            fs::read(imported.join("skills/check/data.bin"))?,
            [0, 255, 42]
        );
        assert!(
            edit(
                &data,
                Edit::Remove {
                    name: "reviewer",
                    revision: "stale"
                }
            )
            .is_err()
        );
        assert_eq!(
            edit(
                &data,
                Edit::Remove {
                    name: "reviewer",
                    revision
                }
            )?,
            "[]"
        );
        assert!(edit(&data, export()).is_err());
        Ok(())
    }

    #[test]
    fn copies_reviewed_bundle_resources_into_the_existing_libraries() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().join("source");
        let data = temp.path().join("data");
        for folder in [
            ".claude-plugin",
            "skills/check",
            "agents",
            "commands",
            "hooks",
        ] {
            fs::create_dir_all(root.join(folder))?;
        }
        fs::write(
            root.join(CLAUDE),
            r#"{"name":"reviewer","mcpServers":{"fixture":{"type":"http","url":"http://127.0.0.1:15423/mcp"},"native":{"command":"${CLAUDE_PLUGIN_ROOT}/run"}}}"#,
        )?;
        fs::write(
            root.join("skills/check/SKILL.md"),
            "---\nname: check\ndescription: Run checks\n---\nCheck the fixture.\n",
        )?;
        fs::write(
            root.join("agents/review.md"),
            "---\nname: bundle-review\ndescription: Review the fixture\n---\nReview the fixture.\n",
        )?;
        fs::write(root.join("commands/check.md"), "Check $ARGUMENTS.")?;
        fs::write(
            root.join("hooks/hooks.json"),
            r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"echo one"},{"type":"command","command":"echo two"}]}]}}"#,
        )?;
        let source = root.to_str().context("fixture path must be UTF-8")?;
        let parse =
            |source: String| nextjson::from_str::<Value>(&source).map_err(anyhow::Error::msg);
        let preview = parse(edit(&data, Edit::Preview { source })?)?;
        let content = preview["content"].as_str().context("missing content")?;
        let saved = parse(edit(
            &data,
            Edit::Save {
                source,
                content,
                revision: None,
            },
        )?)?;
        let revision = saved[0]["revision"].as_str().context("missing revision")?;
        let visit = |kind, path, copy| {
            edit(
                &data,
                Edit::Resources {
                    name: "reviewer",
                    revision,
                    kind,
                    path,
                    copy,
                },
            )
        };

        for (kind, path, key, library) in [
            (
                "skills",
                "skills/check/SKILL.md",
                "skills/check/SKILL.md",
                "skills",
            ),
            (
                "subagents",
                "agents/review.md",
                "agents/review.md",
                "subagents",
            ),
            (
                "commands",
                "commands/check.md",
                "commands/check.md",
                "commands",
            ),
            (
                "mcp",
                ".claude-plugin/plugin.json / mcpServers",
                "0:fixture",
                "project-mcp",
            ),
            (
                "hooks",
                "hooks/hooks.json",
                "0:claude:SessionStart:0:1",
                "hooks",
            ),
        ] {
            let preview = parse(visit(kind, path, None)?)?;
            let entries = preview.as_array().context("expected resource rows")?;
            let entry = entries
                .iter()
                .find(|entry| entry["key"].as_str() == Some(key))
                .context("missing resource")?;
            assert!(!entry["definition"].is_null());
            let stored = data.join(library).join("library.json");
            assert!(!stored.exists());
            assert_eq!(parse(visit(kind, path, Some(key))?)?, *entry);
            let before = fs::read(&stored)?;
            assert!(visit(kind, path, Some(key)).is_err());
            assert_eq!(fs::read(&stored)?, before);
        }
        assert!(
            visit(
                "mcp",
                ".claude-plugin/plugin.json / mcpServers",
                Some("0:native")
            )
            .is_err()
        );
        let snapshot = Path::new(saved[0]["folder"].as_str().context("missing folder")?);
        fs::write(snapshot.join("commands/check.md"), "changed")?;
        assert!(visit("commands", "commands/check.md", None).is_err());
        Ok(())
    }
}
