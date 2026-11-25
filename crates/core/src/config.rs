use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env,
    fs::{self, File},
    io::{Read, Write},
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use reqwest::{RequestBuilder, Url};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
mod groups;
mod origins;
pub use crate::anthropic_auth::AuthHeader;
pub use groups::{ImportedGroup, ResolvedGroups};

pub const MAX_CONFIG: usize = 1024 * 1024;
pub const MAX_MEMBER_WEIGHT: u32 = 1000;
pub const PRESETS: &str = include_str!("presets.json");

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Endpoints {
    #[serde(skip_serializing_if = "AuthHeader::is_default")]
    pub auth_header: AuthHeader,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertex: Option<crate::vertex::Vertex>,
    pub chat: String,
    pub responses: String,
    pub anthropic: String,
    pub gemini: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub models_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    pub id: String,
    /// Ordered models.dev catalogs for matching vendor-specific model IDs.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub catalog: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_site: Option<crate::webchat::Site>,
    #[serde(default)]
    pub endpoints: Endpoints,
    #[serde(default)]
    pub key_env: String,
    pub models: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub listen: SocketAddr,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub cors_origins: BTreeSet<String>,
    pub providers: Vec<Provider>,
    pub groups: BTreeMap<String, Vec<String>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_labels: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_notes: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_off: BTreeMap<String, BTreeSet<String>>,
    /// Groups whose direct model members are omitted from client catalogs.
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub group_unlists: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub disabled_groups: BTreeSet<String>,
    /// Group IDs in the user's preferred display order, including temporarily absent auto groups.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub group_display_order: Vec<String>,
    /// Former group name to its current name, retained for saved client conversations.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub former_groups: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub no_auto_groups: bool,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub hidden_auto_groups: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub auto_group_skip: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub model_same_as: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub rotating_groups: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_weights: BTreeMap<String, BTreeMap<String, u32>>,
    /// A manual group routes only to its selected member, or its first if the selection is gone.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_picks: BTreeMap<String, String>,
    /// Remember a manual choice while a group uses another strategy.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_last_picks: BTreeMap<String, String>,
    pub peer_listen: String,
    pub allowed_peers: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: ([127, 0, 0, 1], 3425).into(),
            cors_origins: BTreeSet::new(),
            providers: vec![],
            groups: BTreeMap::new(),
            group_labels: BTreeMap::new(),
            group_notes: BTreeMap::new(),
            group_off: BTreeMap::new(),
            group_unlists: BTreeSet::new(),
            disabled_groups: BTreeSet::new(),
            group_display_order: vec![],
            former_groups: BTreeMap::new(),
            no_auto_groups: false,
            hidden_auto_groups: BTreeSet::new(),
            auto_group_skip: BTreeSet::new(),
            model_same_as: BTreeMap::new(),
            rotating_groups: BTreeSet::new(),
            group_weights: BTreeMap::new(),
            group_picks: BTreeMap::new(),
            group_last_picks: BTreeMap::new(),
            peer_listen: String::new(),
            allowed_peers: vec![],
        }
    }
}

fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 96
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

fn valid_group_name(name: &str) -> bool {
    valid_id(name)
        || name.strip_prefix("auto-").is_some_and(|s| {
            !s.is_empty()
                && s.len() <= 256
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

pub fn endpoint(s: &str) -> Result<Url> {
    let mut u = catalog_url(s)?;
    if azure(&u) && u.query().is_some() && u.query_pairs().all(|(k, _)| k == "api-version") {
        u.set_query(None);
    }
    ensure!(u.query().is_none(), "API base must not contain a query");
    Ok(u)
}

pub(crate) fn catalog_url(s: &str) -> Result<Url> {
    let u = Url::parse(s).context("invalid endpoint URL")?;
    let host = u.host_str().context("endpoint needs a host")?;
    let ip = host
        .trim_matches(['[', ']'])
        .parse::<std::net::IpAddr>()
        .ok();
    ensure!(
        u.scheme() == "https"
            || (u.scheme() == "http"
                && (host == "localhost" || ip.is_some_and(|ip| ip.is_loopback()))),
        "use HTTPS or a loopback HTTP endpoint"
    );
    ensure!(
        u.username().is_empty() && u.password().is_none() && u.fragment().is_none(),
        "endpoint must not contain embedded credentials or a fragment"
    );
    if ip.is_none() {
        recurse_x::Name::from_ascii(host).map_err(|_| anyhow::anyhow!("invalid DNS hostname"))?;
    }
    Ok(u)
}

pub(crate) fn azure(url: &Url) -> bool {
    url.host_str().is_some_and(|host| {
        [
            ".openai.azure.com",
            ".cognitiveservices.azure.com",
            ".services.ai.azure.com",
        ]
        .iter()
        .any(|suffix| host.ends_with(suffix) && host.len() > suffix.len())
    })
}

pub(crate) fn api_base(s: &str, protocol: &str) -> Result<Url> {
    if protocol == "gemini" {
        // Accept either the base or an SDK method URL copied from the vendor.
        let s = s.trim();
        let mut url = if s.contains("://") {
            catalog_url(s)?
        } else {
            ensure!(
                !s.is_empty() && !s.starts_with('/'),
                "Gemini base needs a host"
            );
            catalog_url(&format!("https://{s}"))?
        };
        let path = url.path().trim_end_matches('/');
        let at = path.strip_suffix("/models").or_else(|| {
            let (prefix, model) = path.rsplit_once("/models/")?;
            (!model.contains('/')
                || model.ends_with(":generateContent")
                || model.ends_with(":streamGenerateContent")
                || model.ends_with(":countTokens"))
            .then_some(prefix)
        });
        if let Some(prefix) = at {
            let base = prefix.to_owned();
            url.set_path(&base);
            url.set_query(None);
        }
        if url.path() == "/" {
            url.set_path("/v1beta");
        }
        ensure!(url.query().is_none(), "API base must not contain a query");
        return Ok(url);
    }
    let mut url = endpoint(s)?;
    if azure(&url) && protocol != "anthropic" {
        url.set_path("/openai/v1");
    }
    Ok(url)
}

pub(crate) fn mantle(url: &Url) -> bool {
    url.host_str()
        .is_some_and(|h| h.starts_with("bedrock-mantle.") && h.ends_with(".api.aws"))
}

pub(crate) fn authenticate(
    request: RequestBuilder,
    url: &Url,
    protocol: &str,
    key: &str,
    mode: AuthHeader,
) -> RequestBuilder {
    if azure(url) && protocol != "anthropic" {
        return request.header("api-key", key);
    }
    if url.path().starts_with("/inference/") && crate::opencode::Console::request(url).is_some() {
        return request.bearer_auth(key);
    }
    match protocol {
        "anthropic" => {
            let host = url.host_str().unwrap_or("");
            let bedrock = (host.starts_with("bedrock-runtime.")
                || host.starts_with("bedrock-runtime-fips."))
                && (host.ends_with(".amazonaws.com") || host.ends_with(".amazonaws.com.cn"));
            if azure(url)
                || mantle(url)
                || bedrock
                || host == "anthropic.com"
                || host.ends_with(".anthropic.com")
            {
                request.header("x-api-key", key)
            } else {
                match mode {
                    AuthHeader::ApiKey => request.header("x-api-key", key),
                    AuthHeader::Bearer => request.bearer_auth(key),
                    AuthHeader::Both => request.header("x-api-key", key).bearer_auth(key),
                }
            }
        }
        "gemini" => request.header("x-goog-api-key", key),
        _ => request.bearer_auth(key),
    }
}

impl Config {
    /// Children precede parents so a group's capabilities can be computed once.
    pub(crate) fn group_order(groups: &BTreeMap<String, Vec<String>>) -> Result<Vec<&str>> {
        fn visit<'a>(
            groups: &'a BTreeMap<String, Vec<String>>,
            name: &'a str,
            path: &mut Vec<&'a str>,
            depths: &mut HashMap<&'a str, usize>,
            order: &mut Vec<&'a str>,
        ) -> Result<usize> {
            const MAX_NEST: usize = 8;
            ensure!(
                path.len() <= MAX_NEST,
                "group {name}: nesting exceeds {MAX_NEST} levels"
            );
            if let Some(depth) = depths.get(name) {
                return Ok(*depth);
            }
            ensure!(
                !path.contains(&name),
                "group {name}: circular group reference"
            );
            let members = groups
                .get(name)
                .with_context(|| format!("unknown group {name}"))?;
            path.push(name);
            let mut depth = 0;
            for child in members.iter().filter_map(|m| m.strip_prefix("group/")) {
                depth = depth.max(1 + visit(groups, child, path, depths, order)?);
            }
            path.pop();
            ensure!(
                depth <= MAX_NEST,
                "group {name}: nesting exceeds {MAX_NEST} levels"
            );
            depths.insert(name, depth);
            order.push(name);
            Ok(depth)
        }
        let mut order = Vec::new();
        let mut depths = HashMap::new();
        let mut path = Vec::new();
        for name in groups.keys() {
            visit(groups, name, &mut path, &mut depths, &mut order)?;
        }
        Ok(order)
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            origins::clean(&self.cors_origins)? == self.cors_origins,
            "browser origins must be canonical"
        );
        ensure!(
            self.listen.ip().is_loopback(),
            "gateway must listen on loopback"
        );
        let mut ids = HashSet::new();
        let mut refs = HashSet::new();
        for p in &self.providers {
            ensure!(
                valid_id(&p.id) && p.id != "group" && ids.insert(&p.id),
                "invalid or duplicate provider id: {}",
                p.id
            );
            ensure!(
                p.key_env.is_empty()
                    || (p
                        .key_env
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                        && !p.key_env.as_bytes()[0].is_ascii_digit()),
                "invalid key environment variable"
            );
            let bases = [
                &p.endpoints.chat,
                &p.endpoints.responses,
                &p.endpoints.anthropic,
                &p.endpoints.gemini,
            ];
            if let Some(site) = p.web_site {
                ensure!(
                    p.key_env.is_empty()
                        && p.endpoints.vertex.is_none()
                        && bases.iter().all(|s| s.is_empty())
                        && p.endpoints.models_url.is_empty(),
                    "web chat uses its browser sign-in only"
                );
                ensure!(
                    p.models.len() == 1 && p.models[0] == site.model(),
                    "web chat exposes {}",
                    site.model()
                );
            } else if let Some(vertex) = &p.endpoints.vertex {
                ensure!(
                    bases.iter().all(|s| s.is_empty()),
                    "Vertex generates its own protocol endpoints"
                );
                vertex.validate(!p.key_env.is_empty(), &p.endpoints.models_url)?;
            } else {
                ensure!(
                    bases.iter().any(|s| !s.is_empty()),
                    "{} has no endpoint",
                    p.id
                );
            }
            for (base, protocol) in
                bases
                    .into_iter()
                    .zip(["chat", "responses", "anthropic", "gemini"])
            {
                if !base.is_empty() {
                    api_base(base, protocol)?;
                }
            }
            if !p.endpoints.models_url.is_empty() {
                catalog_url(&p.endpoints.models_url)?;
            }
            ensure!(!p.models.is_empty(), "{} has no models", p.id);
            for m in &p.models {
                ensure!(
                    !m.is_empty()
                        && m.len() <= 256
                        && m.split('/')
                            .all(|part| !part.is_empty() && part != "." && part != "..")
                        && m.trim() == m
                        && !m.chars().any(char::is_control),
                    "invalid model id"
                );
                ensure!(
                    refs.insert(format!("{}/{m}", p.id)),
                    "duplicate model in {}",
                    p.id
                );
            }
        }
        ensure!(
            self.rotating_groups
                .iter()
                .all(|id| self.groups.contains_key(id)),
            "rotation references an unknown group"
        );
        let resolved = self.resolved_groups();
        ensure!(
            self.disabled_groups
                .iter()
                .all(|id| self.groups.contains_key(id)),
            "only saved route groups can be switched off"
        );
        ensure!(
            self.group_unlists
                .iter()
                .all(|name| self.groups.contains_key(name)),
            "member listing references an unknown group"
        );
        for (name, off) in &self.group_off {
            let members = self
                .groups
                .get(name)
                .context("disabled members for unknown group")?;
            ensure!(
                self.group_picks.contains_key(name) || members.iter().any(|m| !off.contains(m)),
                "all members of group {name} are switched off"
            );
        }
        ensure!(
            self.group_notes
                .keys()
                .all(|id| self.groups.contains_key(id)),
            "note for unknown route group"
        );
        ensure!(
            self.group_labels
                .iter()
                .all(|(id, label)| self.groups.contains_key(id)
                    && !label.trim().is_empty()
                    && !label.chars().any(char::is_control)),
            "invalid route group label"
        );
        let mut ordered = HashSet::new();
        ensure!(
            self.group_display_order
                .iter()
                .all(|name| valid_group_name(name) && ordered.insert(name)),
            "invalid or duplicate ID in group display order"
        );
        for (old, current) in &self.former_groups {
            ensure!(
                valid_group_name(old)
                    && !self.groups.contains_key(old)
                    && self.groups.contains_key(current),
                "invalid former route group {old}"
            );
        }
        for (group, members) in &resolved.members {
            ensure!(
                valid_group_name(group) && !members.is_empty(),
                "invalid or empty route group"
            );
            let mut seen = HashSet::new();
            for m in members {
                ensure!(
                    (refs.contains(m) || m.starts_with("group/")) && seen.insert(m),
                    "group {group}: missing or duplicate member {m}"
                );
            }
        }
        ensure!(
            self.model_same_as
                .values()
                .all(|m| !m.trim().is_empty() && m.len() <= 256),
            "invalid equivalent model name"
        );
        Self::group_order(&resolved.members)?;
        for group in self.group_last_picks.keys() {
            ensure!(
                self.groups.contains_key(group) && !self.group_picks.contains_key(group),
                "invalid remembered selection for group {group}"
            );
        }
        for group in self.group_picks.keys() {
            ensure!(
                self.groups.contains_key(group),
                "manual selection references an unknown group {group}"
            );
            ensure!(
                !self.rotating_groups.contains(group) && !self.group_weights.contains_key(group),
                "group {group}: select one routing strategy"
            );
        }
        for (group, weights) in &self.group_weights {
            let members = self
                .groups
                .get(group)
                .context("weights reference an unknown group")?;
            ensure!(
                !self.rotating_groups.contains(group),
                "group {group}: select one routing strategy"
            );
            let members: HashSet<_> = members.iter().collect();
            for (id, weight) in weights {
                ensure!(
                    members.contains(id) && (1..=MAX_MEMBER_WEIGHT).contains(weight),
                    "group {group}: invalid member or weight for {id} (expected 1 to {MAX_MEMBER_WEIGHT})"
                );
            }
        }
        if !self.peer_listen.is_empty() {
            crate::peer::listen_address(&self.peer_listen)?;
        }
        for id in &self.allowed_peers {
            id.parse::<libp2p::PeerId>()
                .context("invalid allowed peer ID")?;
        }
        Ok(())
    }

    pub fn parse(text: &str) -> Result<Self> {
        ensure!(text.len() <= MAX_CONFIG, "configuration exceeds 1 MiB");
        let mut config: Self = serde_json::from_str(text)?;
        config.cors_origins = origins::clean(&config.cors_origins)?;
        config.group_off.retain(|group, off| {
            if let Some(members) = config.groups.get(group) {
                off.retain(|m| members.contains(m));
            }
            !off.is_empty()
        });
        config.group_notes.retain(|_, note| {
            let line = note.split_whitespace().collect::<Vec<_>>().join(" ");
            *note = line
                .chars()
                .take(200)
                .collect::<String>()
                .trim_end()
                .to_owned();
            !note.is_empty()
        });
        config.validate()?;
        Ok(config)
    }
}

pub fn data_dir() -> Result<PathBuf> {
    if let Some(path) = env::var_os("CRABFORGE_HOME") {
        let path = PathBuf::from(path);
        ensure!(path.is_absolute(), "CRABFORGE_HOME must be absolute");
        return Ok(path);
    }
    let root = if cfg!(windows) {
        env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support"))
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
    };
    Ok(root
        .context("cannot locate user data folder")?
        .join(crate::APP_ID))
}

pub(crate) fn read_limited(path: &Path, limit: usize) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "file exceeds size limit",
        ));
    }
    Ok(bytes)
}

pub fn load(dir: &Path) -> Result<Config> {
    match read_limited(&dir.join("settings.json"), MAX_CONFIG) {
        Ok(bytes) => Config::parse(std::str::from_utf8(&bytes)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn snapshot(dir: &Path) -> Result<String> {
    Ok(serde_json::to_string_pretty(&load(dir)?)?)
}

/// Persist a learned choice only while the configured provider and endpoint still match.
pub(crate) fn remember_auth(dir: &Path, provider: &Provider, mode: AuthHeader) -> Result<()> {
    let _lock = settings_lock(dir)?;
    let mut cfg = load(dir)?;
    if let Some(current) = cfg.providers.iter_mut().find(|p| p.id == provider.id)
        && current.endpoints.auth_header == AuthHeader::Both
        && current.endpoints.anthropic == provider.endpoints.anthropic
        && current.endpoints.models_url == provider.endpoints.models_url
        && current.key_env == provider.key_env
    {
        current.endpoints.auth_header = mode;
        write_settings(dir, &cfg)?;
    }
    Ok(())
}

fn settings_lock(dir: &Path) -> Result<File> {
    fs::create_dir_all(dir)?;
    let lock = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join("settings.lock"))?;
    lock.lock()?;
    Ok(lock)
}

pub fn save(dir: &Path, previous: &str, text: &str) -> Result<()> {
    let cfg = Config::parse(text)?;
    let _lock = settings_lock(dir)?;
    let current = load(dir)?;
    ensure!(
        serde_json::to_string_pretty(&current)? == previous,
        "configuration changed on disk; reload before saving"
    );
    ensure!(
        !cfg.providers.iter().any(|p| p.id == "auto")
            || current.providers.iter().any(|p| p.id == "auto"),
        "provider ID auto is reserved for automatic model groups"
    );
    write_settings(dir, &cfg)
}

fn write_settings(dir: &Path, cfg: &Config) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(cfg)?;
    ensure!(
        bytes.len() <= MAX_CONFIG,
        "formatted configuration exceeds 1 MiB"
    );
    let mut file = NamedTempFile::new_in(dir)?;
    file.write_all(&bytes)?;
    file.as_file().sync_all()?;
    file.persist(dir.join("settings.json"))?;
    Ok(())
}

pub(crate) fn private_file(
    dir: &Path,
    name: &str,
    create: impl FnOnce() -> Result<Vec<u8>>,
) -> Result<Vec<u8>> {
    let path = dir.join(name);
    match read_limited(&path, 4096) {
        Ok(bytes) => return Ok(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.into()),
    }
    fs::create_dir_all(dir)?;
    let bytes = create()?;
    let mut f = NamedTempFile::new_in(dir)?;
    f.write_all(&bytes)?;
    f.as_file().sync_all()?;
    match f.persist_noclobber(&path) {
        Ok(_) => Ok(bytes),
        Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => {
            Ok(read_limited(&path, 4096)?)
        }
        Err(e) => Err(e.error.into()),
    }
}

pub fn token(dir: &Path) -> Result<String> {
    let bytes = private_file(dir, "gateway.key", || {
        let mut b = [0; 32];
        getrandom::fill(&mut b).map_err(|e| anyhow::anyhow!("OS randomness: {e}"))?;
        Ok(b.to_vec())
    })?;
    ensure!(bytes.len() == 32, "invalid gateway.key");
    Ok(hex::encode(bytes))
}

pub fn open_web(site: &str) -> Result<()> {
    let url = match site {
        "deepseek" => "https://chat.deepseek.com/",
        "chatglm" => "https://chatglm.cn/",
        _ => bail!("unknown chat site"),
    };
    open::that(url)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_headers_match_vendor_and_protocol() -> Result<()> {
        let client = reqwest::Client::new();
        for (base, protocol, header, value) in [
            (
                "https://resource.openai.azure.com/openai/deployments/old?api-version=2024-01-01",
                "chat",
                "api-key",
                "example",
            ),
            (
                "https://bedrock-mantle.us-east-1.api.aws/anthropic",
                "anthropic",
                "x-api-key",
                "example",
            ),
            (
                "https://bedrock-runtime.us-east-1.amazonaws.com/anthropic",
                "anthropic",
                "x-api-key",
                "example",
            ),
            (
                "https://bedrock-runtime.us-east-1.amazonaws.com/openai/v1",
                "chat",
                "authorization",
                "Bearer example",
            ),
            (
                "https://api.anthropic.com",
                "anthropic",
                "x-api-key",
                "example",
            ),
            (
                "https://generativelanguage.googleapis.com/v1beta",
                "gemini",
                "x-goog-api-key",
                "example",
            ),
            (
                "https://api.openai.com/v1",
                "chat",
                "authorization",
                "Bearer example",
            ),
        ] {
            let url = api_base(base, protocol)?;
            let request = authenticate(
                client.get(url.clone()),
                &url,
                protocol,
                "example",
                AuthHeader::Both,
            )
            .build()?;
            assert_eq!(request.headers().len(), 1);
            assert_eq!(request.headers()[header], value);
            if header == "api-key" {
                assert_eq!(request.url().path(), "/openai/v1");
            }
        }
        let url = api_base("https://api.deepseek.com/anthropic", "anthropic")?;
        let request = authenticate(
            client.get(url.clone()),
            &url,
            "anthropic",
            "example",
            AuthHeader::Both,
        )
        .build()?;
        assert_eq!(request.headers()["authorization"], "Bearer example");
        assert_eq!(request.headers()["x-api-key"], "example");
        Ok(())
    }
}
