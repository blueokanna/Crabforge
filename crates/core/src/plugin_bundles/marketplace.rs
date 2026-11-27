use super::{MARKET, MAX_MANIFEST, display, encode, local_root, relative, valid_name};
use crate::{
    config::read_limited,
    skills::{is_repository_url, load_repository_document, tree::is_link},
};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonSerialize, Value};
use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use std::{
    fs,
    path::{Path, PathBuf},
};
use url::Url;

#[derive(NsonSerialize)]
struct Marketplace {
    name: String,
    candidates: Vec<Candidate>,
}

#[derive(NsonSerialize)]
struct Candidate {
    name: String,
    description: String,
    version: String,
    source: Option<String>,
    detail: Option<String>,
}

enum Root {
    Local(PathBuf),
    Git(Url),
}

fn field<'a>(value: &'a Value, name: &str) -> &'a str {
    value
        .get(name)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
}

fn relative_folder(path: &str) -> Result<PathBuf> {
    if matches!(path, "." | "./") {
        return Ok(PathBuf::new());
    }
    relative(path).context("plugin source leaves the marketplace folder or uses an invalid path")
}

fn selected(root: &Root, plugin_root: &str, folder: &str) -> Result<String> {
    let prefix = if plugin_root.is_empty() {
        PathBuf::new()
    } else {
        relative_folder(plugin_root)?
    };
    let folder = prefix.join(relative_folder(folder)?);
    match root {
        Root::Local(root) => {
            let mut path = root.clone();
            for part in folder.components() {
                path.push(part);
                let metadata =
                    fs::symlink_metadata(&path).context("plugin directory is unavailable")?;
                ensure!(
                    metadata.is_dir() && !is_link(&metadata),
                    "plugin source must be a regular directory inside the marketplace"
                );
            }
            path.into_os_string()
                .into_string()
                .ok()
                .context("plugin path must be UTF-8")
        }
        Root::Git(root) => {
            let mut source = root.clone();
            let base = percent_decode_str(root.fragment().unwrap_or_default()).decode_utf8()?;
            let path = display(&Path::new(base.as_ref()).join(folder));
            source.set_fragment(Some(
                &utf8_percent_encode(&path, NON_ALPHANUMERIC).to_string(),
            ));
            Ok(source.into())
        }
    }
}

fn external(value: &Value) -> Result<String> {
    let address = match field(value, "source") {
        "github" => {
            let repo = field(value, "repo");
            ensure!(
                repo.split_once('/').is_some_and(|(owner, name)| {
                    [owner, name].iter().all(|part| {
                        !matches!(*part, "" | "." | "..")
                            && part
                                .bytes()
                                .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
                    })
                }),
                "invalid GitHub repository in the marketplace"
            );
            format!("https://github.com/{repo}")
        }
        "url" | "git" | "git-subdir" => field(value, "url").to_owned(),
        _ => anyhow::bail!("unsupported marketplace plugin source"),
    };
    let mut url = Url::parse(&address)?;
    ensure!(
        matches!(url.scheme(), "https" | "http" | "ssh" | "git")
            && url.password().is_none()
            && (url.username().is_empty() || url.scheme() == "ssh"),
        "use a Git address without embedded credentials"
    );
    let reference = field(value, "ref");
    let path = field(value, "path");
    if !reference.is_empty() {
        ensure!(
            url.query().is_none(),
            "marketplace plugin specifies two Git refs"
        );
        url.query_pairs_mut().append_pair("ref", reference);
    }
    if !path.is_empty() {
        ensure!(
            url.fragment().is_none(),
            "marketplace plugin specifies two Git paths"
        );
        let path = display(&relative_folder(path)?);
        url.set_fragment(Some(
            &utf8_percent_encode(&path, NON_ALPHANUMERIC).to_string(),
        ));
    }
    Ok(url.into())
}

pub(super) fn browse(source: &str) -> Result<String> {
    let source = source.trim();
    ensure!(!source.is_empty(), "choose a marketplace folder or Git URL");
    let (root, text) = if is_repository_url(source) {
        let (root, text) = load_repository_document(source, MARKET, MAX_MANIFEST)?;
        (Root::Git(Url::parse(&root)?), text)
    } else {
        let root = local_root(source)?;
        let manifest = root.join(MARKET);
        ensure!(
            !is_link(&fs::symlink_metadata(
                manifest.parent().context("missing manifest folder")?
            )?) && !is_link(&fs::symlink_metadata(&manifest)?),
            "choose the original marketplace manifest"
        );
        let text = String::from_utf8(read_limited(&manifest, MAX_MANIFEST)?)?;
        (Root::Local(root), text)
    };
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let doc: Value = nextjson::from_str(text).map_err(anyhow::Error::msg)?;
    let name = field(&doc, "name").to_owned();
    let prefix = doc
        .get("metadata")
        .map_or("", |meta| field(meta, "pluginRoot"));
    let entries = doc
        .get("plugins")
        .and_then(Value::as_array)
        .context("marketplace needs a plugins array")?;
    ensure!(
        entries.len() <= 1024,
        "marketplace exceeds 1,024 plugin entries"
    );
    let mut candidates = Vec::new();
    for entry in entries {
        let name = field(entry, "name");
        let source = entry.get("source");
        let resolved = if !valid_name(name) {
            Err(anyhow::anyhow!("invalid marketplace plugin name"))
        } else if let Some(path) = source.and_then(Value::as_str) {
            selected(&root, prefix, path)
        } else {
            source
                .context("marketplace plugin has no source")
                .and_then(external)
        };
        let (source, detail) = match resolved {
            Ok(source) => (Some(source), None),
            Err(error) => (None, Some(error.to_string())),
        };
        candidates.push(Candidate {
            name: name.to_owned(),
            description: field(entry, "description").to_owned(),
            version: field(entry, "version").to_owned(),
            source,
            detail,
        });
    }
    ensure!(!candidates.is_empty(), "the marketplace lists no plugins");
    encode(&Marketplace { name, candidates })
}
