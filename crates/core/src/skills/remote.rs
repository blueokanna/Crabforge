use super::tree::{DOCUMENT, MAX_BYTES, MAX_DEPTH, MAX_ENTRIES, Tree, windows_filename};
use anyhow::{Context, Result, ensure};
use flate2::read::MultiGzDecoder;
use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{self, Cursor, Read, Seek},
    path::{Path, PathBuf},
    time::Duration,
};
use tar::Archive;
use ureq::tls::{RootCerts, TlsConfig};
use url::Url;

mod git;

const MAX_DOWNLOAD: u64 = 32 * 1024 * 1024;
const MAX_TAR: u64 = 128 * 1024 * 1024;
const MAX_HEADERS: usize = 32_768;

pub(crate) fn is_repository_url(input: &str) -> bool {
    input.contains("://")
        || input
            .split_once(':')
            .is_some_and(|(host, _)| host.contains('@') && !host.contains(['/', '\\']))
}

struct Source {
    owner: String,
    repo: String,
    reference: String,
    folder: Option<String>,
}

enum Repository {
    GitHub(Source),
    Git(git::Source),
}

impl Repository {
    fn open(input: &str) -> Result<Self> {
        Self::open_with_document(input, Some(DOCUMENT))
    }

    fn open_with_document(input: &str, document: Option<&str>) -> Result<Self> {
        if Url::parse(input).is_ok_and(|url| {
            url.scheme() == "https"
                && matches!(url.host_str(), Some("github.com" | "www.github.com"))
        }) {
            Ok(Self::GitHub(Source::parse(input, document)?))
        } else {
            Ok(Self::Git(git::Source::parse(input)?))
        }
    }

    fn folder(&self) -> Option<&str> {
        match self {
            Self::GitHub(source) => source.folder.as_deref(),
            Self::Git(source) => source.folder.as_deref(),
        }
    }

    fn archive(&self) -> Result<File> {
        match self {
            Self::GitHub(source) => source.download(),
            Self::Git(source) => source.fetch(),
        }
    }

    fn canonical(&self, folder: &str) -> String {
        match self {
            Self::GitHub(source) => source.canonical(folder),
            Self::Git(source) => source.canonical(folder),
        }
    }
}

fn unescape(text: &str) -> Result<String> {
    Ok(percent_decode_str(text).decode_utf8()?.into_owned())
}

fn valid_path(path: &str) -> bool {
    path.is_empty() || (path.len() <= 1024 && path.split('/').all(windows_filename))
}

impl Source {
    fn parse(input: &str, document: Option<&str>) -> Result<Self> {
        let url = Url::parse(input).context("invalid GitHub source URL")?;
        ensure!(
            url.scheme() == "https"
                && matches!(url.host_str(), Some("github.com" | "www.github.com"))
                && url.port().is_none()
                && url.username().is_empty()
                && url.password().is_none(),
            "use an HTTPS GitHub repository, folder or file URL"
        );
        let components = url
            .path()
            .trim_matches('/')
            .split('/')
            .map(unescape)
            .collect::<Result<Vec<_>>>()?;
        let [owner, repo, tail @ ..] = components.as_slice() else {
            anyhow::bail!("the GitHub URL needs an owner and repository");
        };
        let repo = repo.strip_suffix(".git").unwrap_or(repo);
        for part in [owner.as_str(), repo] {
            ensure!(
                !part.is_empty()
                    && part != "."
                    && part != ".."
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b)),
                "invalid GitHub owner or repository name"
            );
        }
        let mut reference = "HEAD".to_owned();
        let mut folder = None;
        if tail.is_empty() {
            let mut refs = url.query_pairs().filter(|(key, _)| key == "ref");
            if let Some((_, value)) = refs.next() {
                ensure!(refs.next().is_none(), "use one GitHub ref per source");
                reference = value.into_owned();
            }
            folder = url.fragment().map(unescape).transpose()?;
        } else if let [kind, branch, path @ ..] = tail {
            ensure!(url.fragment().is_none(), "use one GitHub path per source");
            let path = match kind.as_str() {
                "tree" => path,
                "blob" => {
                    let (file, directories) =
                        path.split_last().context("missing GitHub filename")?;
                    if let Some(document) = document {
                        ensure!(
                            file == document,
                            "the GitHub file URL must point to {document}"
                        );
                        directories
                    } else {
                        path
                    }
                }
                _ => anyhow::bail!("use a GitHub repository, folder or file URL"),
            };
            reference = branch.clone();
            if kind == "blob" || !path.is_empty() {
                folder = Some(path.join("/"));
            }
        } else {
            anyhow::bail!("the GitHub URL needs a branch, tag or commit after tree or blob");
        }
        ensure!(
            !reference.is_empty()
                && reference.len() <= 256
                && !reference.chars().any(char::is_control)
                && reference
                    .split('/')
                    .all(|part| !matches!(part, "" | "." | "..")),
            "invalid GitHub ref"
        );
        ensure!(
            folder.as_deref().is_none_or(valid_path),
            "invalid GitHub path"
        );
        Ok(Self {
            owner: owner.clone(),
            repo: repo.into(),
            reference,
            folder,
        })
    }

    fn canonical(&self, folder: &str) -> String {
        let mut url = Url::parse(&format!("https://github.com/{}/{}", self.owner, self.repo))
            .expect("validated repository name");
        if self.reference != "HEAD" {
            url.query_pairs_mut().append_pair("ref", &self.reference);
        }
        // An empty fragment records a selection of the repository root.
        let escaped = utf8_percent_encode(folder, NON_ALPHANUMERIC).to_string();
        url.set_fragment(Some(&escaped));
        url.into()
    }

    fn download(&self) -> Result<File> {
        let reference = utf8_percent_encode(&self.reference, NON_ALPHANUMERIC);
        let url = format!(
            "https://codeload.github.com/{}/{}/tar.gz/{reference}",
            self.owner, self.repo
        );
        let client = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_global(Some(Duration::from_secs(90)))
            .tls_config(
                TlsConfig::builder()
                    .root_certs(RootCerts::PlatformVerifier)
                    .build(),
            )
            .user_agent("crabforge")
            .build()
            .new_agent();
        let mut response = client
            .get(&url)
            .call()
            .context("download GitHub skill archive")?;
        ensure!(
            response.status().is_success(),
            "GitHub skill archive returned HTTP {}",
            response.status().as_u16()
        );
        let mut body = response.body_mut().as_reader().take(MAX_DOWNLOAD + 1);
        let archive = unpack(&mut body)?;
        ensure!(
            body.limit() != 0,
            "GitHub skill archive exceeds the download limit"
        );
        Ok(archive)
    }
}

fn unpack(reader: impl Read) -> Result<File> {
    let mut file = tempfile::tempfile()?;
    let length = io::copy(
        &mut MultiGzDecoder::new(reader).take(MAX_TAR + 1),
        &mut file,
    )
    .context("decode GitHub skill archive")?;
    ensure!(
        length <= MAX_TAR,
        "GitHub skill archive exceeds the uncompressed size limit"
    );
    file.rewind()?;
    Ok(file)
}

fn relative(path: &[u8]) -> Result<&str> {
    let path = std::str::from_utf8(path)?.trim_end_matches('/');
    let (root, nested) = path.split_once('/').unwrap_or((path, ""));
    ensure!(windows_filename(root), "invalid archive root directory");
    Ok(nested)
}

fn document_folder(path: &str) -> Option<&str> {
    if path == DOCUMENT {
        Some("")
    } else {
        path.strip_suffix("/SKILL.md")
            .filter(|path| valid_path(path))
    }
}

pub(crate) fn visit_repository_documents(
    input: &str,
    accepts: impl Fn(&str) -> bool,
    mut visit: impl FnMut(&str, String, bool, &mut dyn Read) -> Result<()>,
) -> Result<()> {
    let repo = Repository::open_with_document(input, None)?;
    let selected = repo.folder().unwrap_or_default();
    let mut archive = repo.archive()?;
    for (index, entry) in Archive::new(&mut archive).entries()?.enumerate() {
        ensure!(index < MAX_HEADERS, "too many entries in the Git archive");
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let raw = entry.path_bytes().into_owned();
        let Ok(path) = relative(&raw) else { continue };
        if !valid_path(path) || !accepts(path) {
            continue;
        }
        if path
            .rsplit('/')
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case(DOCUMENT))
            || selected.is_empty()
            || path == selected
            || path
                .strip_prefix(selected)
                .is_some_and(|tail| tail.starts_with('/'))
        {
            let directly_selected = path == selected
                || !selected.is_empty()
                    && path
                        .rsplit_once('/')
                        .is_some_and(|(parent, _)| parent == selected);
            visit(path, repo.canonical(path), directly_selected, &mut entry)?;
        }
    }
    Ok(())
}

fn find_folder(
    archive: &mut File,
    requested: Option<&str>,
    canonical: impl Fn(&str) -> String,
) -> Result<Option<String>> {
    let mut folders = BTreeSet::new();
    for (index, entry) in Archive::new(archive).entries()?.enumerate() {
        ensure!(
            index < MAX_HEADERS,
            "too many entries in the GitHub skill archive"
        );
        let entry = entry?;
        let path = entry.path_bytes();
        let path = relative(&path)?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        if let Some(folder) = document_folder(path) {
            if requested == Some(folder) || (requested.is_none() && folder.is_empty()) {
                return Ok(Some(folder.to_owned()));
            }
            if requested.is_none() {
                folders.insert(folder.to_owned());
            }
        }
    }
    if folders.len() <= 1 {
        return Ok(folders.pop_first());
    }
    let examples = folders
        .iter()
        .take(12)
        .map(|path| canonical(path))
        .collect::<Vec<_>>();
    anyhow::bail!(
        "the repository contains {} skill folders. Enter a folder or SKILL.md URL.\n{}",
        folders.len(),
        examples.join("\n")
    );
}

fn extract(
    archive: &mut (impl Read + Seek),
    folder: &str,
    root: &Path,
    archive_root: Option<&str>,
) -> Result<Tree> {
    let prefix = format!("{folder}/");
    let mut entries = BTreeSet::new();
    let mut names = BTreeMap::new();
    let mut remaining = MAX_BYTES as u64;
    archive.rewind()?;
    for (index, entry) in Archive::new(archive).entries()?.enumerate() {
        ensure!(index < MAX_HEADERS, "too many entries in the archive");
        let mut entry = entry?;
        let raw = entry.path_bytes();
        if let Some(archive_root) = archive_root {
            ensure!(
                raw.starts_with(archive_root.as_bytes()),
                "invalid Crabforge plugin archive path"
            );
        }
        let path = relative(&raw)?;
        let path = if folder.is_empty() {
            path
        } else if let Some(path) = path.strip_prefix(&prefix) {
            path
        } else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        ensure!(
            valid_path(path)
                && path.split('/').count() <= MAX_DEPTH + 1
                && entries.len() < MAX_ENTRIES
                && entries.insert(path.to_owned()),
            "invalid or oversized path in the selected skill: {path:?}"
        );
        let path = path.split('/').collect::<PathBuf>();
        for ancestor in path
            .ancestors()
            .take_while(|parent| !parent.as_os_str().is_empty())
        {
            let exact = ancestor.to_str().expect("validated UTF-8 archive path");
            let previous = names
                .entry(exact.to_uppercase())
                .or_insert_with(|| exact.to_owned());
            ensure!(
                previous == exact,
                "skill filename {exact:?} is ambiguous on Windows"
            );
        }
        ensure!(
            names.len() <= MAX_ENTRIES,
            "selected skill has too many files and folders"
        );
        let target = root.join(path);
        let kind = entry.header().entry_type();
        if kind.is_dir() {
            fs::create_dir_all(&target)?;
            continue;
        }
        ensure!(
            kind.is_file(),
            "only regular files and folders are supported in skills"
        );
        ensure!(
            entry.size() <= remaining,
            "selected skill exceeds its size limit"
        );
        fs::create_dir_all(target.parent().expect("path is inside the skill root"))?;
        let mut file = File::create_new(target)?;
        let expected = entry.size();
        let copied = io::copy(&mut entry, &mut file)?;
        ensure!(copied == expected, "skill archive ended inside a file");
        remaining -= copied;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(
                0o644 | (entry.header().mode()? & 0o111),
            ))?;
        }
    }
    Tree::read(root)
}

pub(crate) fn read_plugin_archive(path: &Path) -> Result<Tree> {
    const MAX_ARCHIVE: usize = 24 * 1024 * 1024;
    let bytes = crate::config::read_limited(path, MAX_ARCHIVE)?;
    let staging = tempfile::tempdir()?;
    extract(
        &mut Cursor::new(bytes),
        "",
        staging.path(),
        Some("crabforge-plugin/"),
    )
}

fn selected_folder<'a>(selected: &'a str, manifests: &[&str]) -> &'a str {
    manifests
        .iter()
        .find_map(|manifest| {
            if selected == *manifest {
                Some("")
            } else {
                selected
                    .strip_suffix(manifest)
                    .and_then(|prefix| prefix.strip_suffix('/'))
            }
        })
        .unwrap_or(selected)
}

pub(crate) fn load_repository_document(
    input: &str,
    document: &str,
    limit: usize,
) -> Result<(String, String)> {
    let repo = Repository::open_with_document(input, None)?;
    let folder = selected_folder(repo.folder().unwrap_or_default(), &[document]);
    let expected = if folder.is_empty() {
        document.to_owned()
    } else {
        format!("{folder}/{document}")
    };
    let mut archive = repo.archive()?;
    for (index, entry) in Archive::new(&mut archive).entries()?.enumerate() {
        ensure!(index < MAX_HEADERS, "too many entries in the Git archive");
        let mut entry = entry?;
        if entry.header().entry_type().is_file() && relative(&entry.path_bytes())? == expected {
            ensure!(
                entry.size() <= limit as u64,
                "repository manifest exceeds its size limit"
            );
            let mut text = String::new();
            entry.read_to_string(&mut text)?;
            return Ok((repo.canonical(folder), text));
        }
    }
    anyhow::bail!("no {document} in the selected repository folder")
}

pub(crate) fn load_repository_tree(input: &str, manifests: &[&str]) -> Result<(String, Tree)> {
    let repo = Repository::open_with_document(input, None)?;
    let folder = selected_folder(repo.folder().unwrap_or_default(), manifests);
    let staging = tempfile::tempdir()?;
    let tree = extract(&mut repo.archive()?, folder, staging.path(), None)?;
    Ok((repo.canonical(folder), tree))
}

pub(super) fn load(input: &str) -> Result<(String, Tree)> {
    read(input)?.context("the selected Git source has no SKILL.md")
}

pub(super) fn read(input: &str) -> Result<Option<(String, Tree)>> {
    let repo = Repository::open(input)?;
    bundle(repo.archive()?, repo.folder(), |folder| {
        repo.canonical(folder)
    })
}

pub(super) fn catalog(input: &str) -> Result<Vec<super::catalog::Candidate>> {
    let repo = Repository::open(input)?;
    let mut archive = repo.archive()?;
    let mut catalog = super::catalog::Catalog::default();
    let mut seen = BTreeSet::new();
    for (index, entry) in Archive::new(&mut archive).entries()?.enumerate() {
        ensure!(index < MAX_HEADERS, "too many entries in the Git archive");
        let entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let raw = entry.path_bytes();
        let Some(folder) = relative(&raw).ok().and_then(document_folder) else {
            continue;
        };
        let source = repo.canonical(folder);
        ensure!(
            seen.len() < MAX_ENTRIES && seen.insert(source.clone()),
            "too many or duplicate skill documents in the Git archive"
        );
        let folder = folder.to_owned();
        let length = entry.size();
        catalog.inspect(entry, length, source, folder)?;
    }
    Ok(catalog.finish())
}

fn bundle(
    mut archive: File,
    requested: Option<&str>,
    canonical: impl Fn(&str) -> String,
) -> Result<Option<(String, Tree)>> {
    let Some(folder) = find_folder(&mut archive, requested, &canonical)? else {
        return Ok(None);
    };
    let staging = tempfile::tempdir()?;
    let tree = extract(&mut archive, &folder, staging.path(), None)?;
    Ok(Some((canonical(&folder), tree)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compression, write::GzEncoder};
    use tar::{Builder, EntryType, Header};

    fn fixture(entries: &[(&str, &[u8], EntryType)]) -> Result<File> {
        let mut tar = Builder::new(GzEncoder::new(Vec::new(), Compression::fast()));
        for &(path, bytes, kind) in entries {
            let mut header = Header::new_gnu();
            header.set_entry_type(kind);
            header.set_mode(0o644);
            header.set_size(bytes.len() as u64);
            tar.append_data(&mut header, path, bytes)?;
        }
        let bytes = tar.into_inner()?.finish()?;
        unpack(bytes.as_slice())
    }

    #[test]
    fn selects_a_skill_and_only_its_attachments() -> Result<()> {
        let doc = b"---\nname: report\ndescription: Report units.\n---\n";
        let mut archive = fixture(&[
            ("repo-main/tools/report/SKILL.md", doc, EntryType::Regular),
            (
                "repo-main/tools/report/ref/units.bin",
                &[0, 100, 255],
                EntryType::Regular,
            ),
            (
                "repo-main/tools/sibling.txt",
                b"outside the skill",
                EntryType::Regular,
            ),
        ])?;
        let source = Source::parse("https://github.com/team/repo", Some(DOCUMENT))?;
        let folder = find_folder(&mut archive, source.folder.as_deref(), |folder| {
            source.canonical(folder)
        })?
        .context("missing test skill")?;
        assert_eq!(folder, "tools/report");
        let canonical = source.canonical(&folder);
        assert_eq!(
            Source::parse(&canonical, Some(DOCUMENT))?.folder.as_deref(),
            Some(folder.as_str())
        );
        let temp = tempfile::tempdir()?;
        let tree = extract(&mut archive, &folder, temp.path(), None)?;
        assert_eq!(tree.document()?.as_bytes(), doc);
        assert_eq!(fs::read(temp.path().join("ref/units.bin"))?, [0, 100, 255]);
        assert!(!temp.path().join("sibling.txt").exists());
        let pinned = Source::parse(
            "https://github.com/team/repo/blob/feature%2Fdocs/tools/report/SKILL.md",
            Some(DOCUMENT),
        )?;
        assert_eq!(pinned.reference, "feature/docs");
        assert_eq!(pinned.folder.as_deref(), Some("tools/report"));
        Ok(())
    }

    #[test]
    fn rejects_ambiguous_repositories_and_invalid_bundle_paths() -> Result<()> {
        let source = Source::parse("https://github.com/team/repo", Some(DOCUMENT))?;
        let mut archive = fixture(&[
            ("repo-main/one/SKILL.md", b"one", EntryType::Regular),
            ("repo-main/two/SKILL.md", b"two", EntryType::Regular),
        ])?;
        let error = find_folder(&mut archive, None, |folder| source.canonical(folder))
            .unwrap_err()
            .to_string();
        assert!(error.contains("/repo#one") && error.contains("/repo#two"));
        for (first, second, kind) in [
            (
                "repo-main/one/Refs/a",
                "repo-main/one/refs/b",
                EntryType::Regular,
            ),
            (
                "repo-main/one/refs/a",
                "repo-main/one/link",
                EntryType::Symlink,
            ),
        ] {
            let mut archive = fixture(&[(first, b"", EntryType::Regular), (second, b"", kind)])?;
            let temp = tempfile::tempdir()?;
            assert!(extract(&mut archive, "one", temp.path(), None).is_err());
        }
        assert!(Source::parse("https://github.com/user/repo#one/%2E%2E", Some(DOCUMENT)).is_err());
        assert!(
            Source::parse("https://github.com@other.example/user/repo", Some(DOCUMENT)).is_err()
        );
        Ok(())
    }
}
