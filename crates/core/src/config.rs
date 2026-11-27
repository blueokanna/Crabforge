use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env,
    fs::{self, File},
    io::{Read, Write},
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use http::request::Builder as RequestBuilder;
use nextjson::{NsonDeserialize, NsonSerialize};
use tempfile::NamedTempFile;
use url::Url;
mod groups;
mod magpie;
mod origins;
mod rename;
pub use crate::anthropic_auth::AuthHeader;
pub use groups::{ImportedGroup, MemberPattern, ResolvedGroups};
pub use magpie::{
    MagpieImportOptions, MagpieSettingsMode, import_magpie_backup, import_magpie_groups,
    import_magpie_providers, import_magpie_routing,
};
pub(crate) use magpie::{
    library_section as magpie_library_section, open_backup as open_magpie_backup,
};
pub use rename::rename_provider;

pub const MAX_CONFIG: usize = 1024 * 1024;
pub const MAX_MEMBER_WEIGHT: u32 = 1000;
pub const PRESETS: &str = include_str!("presets.json");

#[derive(Clone, Debug, Default, NsonSerialize, NsonDeserialize)]
#[njson(default, deny_unknown_fields)]
pub struct Endpoints {
    #[njson(skip_serializing_if = "AuthHeader::is_default")]
    pub auth_header: AuthHeader,
    #[njson(skip_serializing_if = "Option::is_none")]
    pub vertex: Option<crate::vertex::Vertex>,
    pub chat: String,
    pub responses: String,
    pub anthropic: String,
    pub gemini: String,
    #[njson(skip_serializing_if = "String::is_empty")]
    pub models_url: String,
}

#[derive(Clone, Debug, NsonSerialize, NsonDeserialize)]
#[njson(deny_unknown_fields)]
pub struct Provider {
    pub id: String,
    /// Ordered models.dev catalogs for matching vendor-specific model IDs.
    #[njson(default, skip_serializing_if = "String::is_empty")]
    pub catalog: String,
    #[njson(default, skip_serializing_if = "Option::is_none")]
    pub web_site: Option<crate::webchat::Site>,
    #[njson(default)]
    pub endpoints: Endpoints,
    #[njson(default)]
    pub key_env: String,
    pub models: Vec<String>,
    /// Backup model and route group IDs in the order to try for direct requests.
    #[njson(default, skip_serializing_if = "Vec::is_empty")]
    pub fallback: Vec<String>,
    #[njson(
        default,
        alias = "noStandIn",
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub no_stand_in: bool,
    /// IDs the provider catalog explicitly marks free.
    #[njson(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub free_models: BTreeSet<String>,
    /// Protocol information advertised by an upstream gateway for individual models.
    #[njson(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub model_protocols: BTreeMap<String, crate::catalog::RelayProtocols>,
}

#[derive(Clone, Debug, NsonSerialize, NsonDeserialize)]
#[njson(default = "Config::default", deny_unknown_fields)]
pub struct Config {
    pub listen: SocketAddr,
    #[njson(skip_serializing_if = "BTreeSet::is_empty")]
    pub cors_origins: BTreeSet<String>,
    pub providers: Vec<Provider>,
    /// Previous provider IDs accepted by saved client conversations.
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub former_providers: BTreeMap<String, String>,
    pub groups: BTreeMap<String, Vec<String>>,
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_match: BTreeMap<String, Vec<MemberPattern>>,
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_placed: BTreeMap<String, Vec<String>>,
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_labels: BTreeMap<String, String>,
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_notes: BTreeMap<String, String>,
    /// Seconds to wait for streamed content, or -1 to override the default with no deadline.
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_first_content: BTreeMap<String, i32>,
    #[njson(skip_serializing_if = "is_zero")]
    pub default_group_first_content: i32,
    /// Seconds to retry temporarily busy members. Zero inherits the enclosing group.
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_patience: BTreeMap<String, i32>,
    #[njson(skip_serializing_if = "patience_is_off")]
    pub default_group_patience: i32,
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_off: BTreeMap<String, BTreeSet<String>>,
    /// Groups whose direct model members are omitted from client catalogs.
    #[njson(skip_serializing_if = "BTreeSet::is_empty")]
    pub group_unlists: BTreeSet<String>,
    #[njson(skip_serializing_if = "BTreeSet::is_empty")]
    pub disabled_groups: BTreeSet<String>,
    /// Group IDs in the user's preferred display order, including temporarily absent auto groups.
    #[njson(skip_serializing_if = "Vec::is_empty")]
    pub group_display_order: Vec<String>,
    /// Former group name to its current name, retained for saved client conversations.
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub former_groups: BTreeMap<String, String>,
    #[njson(skip_serializing_if = "std::ops::Not::not")]
    pub no_auto_groups: bool,
    #[njson(skip_serializing_if = "BTreeSet::is_empty")]
    pub hidden_auto_groups: BTreeSet<String>,
    #[njson(skip_serializing_if = "BTreeSet::is_empty")]
    pub auto_group_skip: BTreeSet<String>,
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub model_same_as: BTreeMap<String, String>,
    #[njson(skip_serializing_if = "BTreeSet::is_empty")]
    pub rotating_groups: BTreeSet<String>,
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_weights: BTreeMap<String, BTreeMap<String, u32>>,
    /// Member weights retained while a group uses another strategy.
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_last_weights: BTreeMap<String, BTreeMap<String, u32>>,
    /// A manual group routes only to its selected member, or its first if the selection is gone.
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    pub group_picks: BTreeMap<String, String>,
    /// Remember a manual choice while a group uses another strategy.
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
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
            former_providers: BTreeMap::new(),
            groups: BTreeMap::new(),
            group_match: BTreeMap::new(),
            group_placed: BTreeMap::new(),
            group_labels: BTreeMap::new(),
            group_notes: BTreeMap::new(),
            group_first_content: BTreeMap::new(),
            default_group_first_content: 0,
            group_patience: BTreeMap::new(),
            default_group_patience: -1,
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
            group_last_weights: BTreeMap::new(),
            group_picks: BTreeMap::new(),
            group_last_picks: BTreeMap::new(),
            peer_listen: String::new(),
            allowed_peers: vec![],
        }
    }
}

fn is_zero(value: &i32) -> bool {
    *value == 0
}

fn patience_is_off(value: &i32) -> bool {
    *value == -1
}

fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 96
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

fn valid_model_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 256
        && id.split('/').all(|part| !matches!(part, "" | "." | ".."))
        && id.trim() == id
        && !id.chars().any(char::is_control)
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
    mut request: RequestBuilder,
    url: &Url,
    protocol: &str,
    key: &str,
    mode: AuthHeader,
) -> RequestBuilder {
    for &name in api_key_headers(url, protocol, mode) {
        request = if name == "authorization" {
            request.header(name, format!("Bearer {key}"))
        } else {
            request.header(name, key)
        };
    }
    request
}

pub(crate) fn api_key_headers(
    url: &Url,
    protocol: &str,
    mode: AuthHeader,
) -> &'static [&'static str] {
    if azure(url) && protocol != "anthropic" {
        return &["api-key"];
    }
    if url.path().starts_with("/inference/") && crate::opencode::Console::request(url).is_some() {
        return &["authorization"];
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
                &["x-api-key"]
            } else {
                match mode {
                    AuthHeader::ApiKey => &["x-api-key"],
                    AuthHeader::Bearer => &["authorization"],
                    AuthHeader::Both => &["x-api-key", "authorization"],
                }
            }
        }
        "gemini" => &["x-goog-api-key"],
        _ => &["authorization"],
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
                ensure!(valid_model_id(m), "invalid model id");
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
        ensure!(
            self.default_group_first_content >= -1
                && self.group_first_content.values().all(|&wait| wait >= -1),
            "first-content timeout must be -1, 0 or positive seconds"
        );
        ensure!(
            self.group_first_content
                .keys()
                .all(|id| self.groups.contains_key(id)),
            "first-content timeout references an unknown group"
        );
        ensure!(
            (-1..=240).contains(&self.default_group_patience)
                && self
                    .group_patience
                    .values()
                    .all(|wait| (-1..=240).contains(wait)),
            "retry patience must be between -1 and 240 seconds"
        );
        ensure!(
            self.group_patience
                .keys()
                .all(|id| self.groups.contains_key(id)),
            "retry patience references an unknown group"
        );
        let resolved = self.resolved_groups();
        ensure!(
            self.group_match
                .keys()
                .all(|id| self.groups.contains_key(id)),
            "patterns reference an unknown route group"
        );
        ensure!(
            self.group_placed
                .keys()
                .all(|id| self.group_match.contains_key(id)),
            "member placement requires a group with patterns"
        );
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
                self.group_match.contains_key(name)
                    || self.group_picks.contains_key(name)
                    || members.iter().any(|m| !off.contains(m)),
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
                valid_group_name(group)
                    && (!members.is_empty()
                        || self.group_match.get(group).is_some_and(|p| !p.is_empty())),
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
        for (group, weights) in self.group_weights.iter().chain(&self.group_last_weights) {
            let members = self
                .groups
                .get(group)
                .context("weights reference an unknown group")?;
            ensure!(
                !self.group_weights.contains_key(group)
                    || (!self.rotating_groups.contains(group)
                        && !self.group_last_weights.contains_key(group)),
                "group {group}: select one routing strategy"
            );
            let members: HashSet<_> = members.iter().collect();
            for (id, weight) in weights {
                ensure!(
                    (members.contains(id)
                        || self
                            .group_match
                            .get(group)
                            .is_some_and(|r| r.iter().any(|r| r.matches(id, true))))
                        && (1..=MAX_MEMBER_WEIGHT).contains(weight),
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
        let mut config: Self = nextjson::from_str(text).map_err(anyhow::Error::msg)?;
        for provider in &mut config.providers {
            let mut seen = HashSet::new();
            provider.fallback = std::mem::take(&mut provider.fallback)
                .into_iter()
                .map(|id| id.trim().to_owned())
                .filter(|id| !id.is_empty() && seen.insert(id.clone()))
                .collect();
        }
        let active: HashSet<_> = config.providers.iter().map(|p| p.id.as_str()).collect();
        config.former_providers.retain(|old, current| {
            valid_id(old)
                && old != "group"
                && !active.contains(old.as_str())
                && active.contains(current.as_str())
        });
        config.cors_origins = origins::clean(&config.cors_origins)?;
        config.group_match.retain(|_, rules| {
            let mut seen = HashSet::new();
            rules.retain(|rule| seen.insert(rule.as_str().to_owned()));
            !rules.is_empty()
        });
        config.group_placed.retain(|_, ids| {
            let mut seen = HashSet::new();
            ids.retain(|id| seen.insert(id.clone()));
            !ids.is_empty()
        });
        config.group_off.retain(|group, off| {
            if let Some(members) = config.groups.get(group) {
                off.retain(|m| {
                    members.contains(m)
                        || config
                            .group_match
                            .get(group)
                            .is_some_and(|r| r.iter().any(|r| r.matches(m, true)))
                });
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
    load(dir)?.formatted()
}

impl Config {
    pub fn to_json(&self) -> Result<String> {
        nextjson::to_string(self).map_err(anyhow::Error::msg)
    }

    fn formatted(&self) -> Result<String> {
        nextjson::to_string_pretty(self).map_err(anyhow::Error::msg)
    }

    fn check_snapshot(&self, previous: &str) -> Result<()> {
        let formatted = self.formatted()?;
        if formatted == previous {
            return Ok(());
        }
        // An already-open client may have a snapshot formatted by an earlier release.
        let same = previous.len() <= MAX_CONFIG
            && nextjson::from_str::<nextjson::Value>(previous).is_ok_and(|old| {
                nextjson::to_value(self)
                    .is_ok_and(|current| crate::json::equivalent(&current, &old))
            });
        ensure!(same, "configuration changed on disk; reload before saving");
        Ok(())
    }
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
    current.check_snapshot(previous)?;
    ensure!(
        !cfg.providers.iter().any(|p| p.id == "auto")
            || current.providers.iter().any(|p| p.id == "auto"),
        "provider ID auto is reserved for automatic model groups"
    );
    write_settings(dir, &cfg)
}

fn write_settings(dir: &Path, cfg: &Config) -> Result<()> {
    let bytes = cfg.formatted()?.into_bytes();
    ensure!(
        bytes.len() <= MAX_CONFIG,
        "formatted configuration exceeds 1 MiB"
    );
    replace_contents(&dir.join("settings.json"), &bytes, None)
}

pub(crate) fn replace_contents(
    destination: &Path,
    bytes: &[u8],
    permissions: Option<fs::Permissions>,
) -> Result<()> {
    let dir = destination.parent().context("missing parent folder")?;
    #[cfg(not(windows))]
    let mut file = NamedTempFile::new_in(dir)?;
    #[cfg(windows)]
    let mut file = tempfile::Builder::new().make_in(dir, |path| {
        File::options().write(true).create_new(true).open(path)
    })?;
    if let Some(permissions) = permissions {
        file.as_file().set_permissions(permissions)?;
    }
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    #[cfg(not(windows))]
    file.persist(destination)?;
    #[cfg(windows)]
    {
        let mut staged = file.into_temp_path();
        // std::fs::rename can replace a file held open by a reader that shares delete access.
        fs::rename(&staged, destination)
            .with_context(|| format!("cannot replace {}", destination.display()))?;
        staged.disable_cleanup(true);
    }
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

    #[cfg(windows)]
    #[test]
    fn settings_remain_readable_during_atomic_replacement() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        let original = r#"{"listen":"127.0.0.1:3425"}"#;
        let updated = r#"{"listen":"127.0.0.1:3426"}"#;
        fs::write(&path, original)?;
        let previous = snapshot(dir.path())?;
        let mut reader = File::open(&path)?;

        save(dir.path(), &previous, updated)?;

        let mut old_contents = String::new();
        reader.read_to_string(&mut old_contents)?;
        assert_eq!(old_contents, original);
        assert_eq!(load(dir.path())?.listen.port(), 3426);
        assert!(save(dir.path(), &previous, original).is_err());
        Ok(())
    }

    #[test]
    fn legacy_settings_keep_defaults_wire_names_and_snapshot_equality() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let legacy = r#"{"listen":"[::1]:3425","providers":[{"id":"relay","models":["m"],"noStandIn":true,"endpoints":{"chat":"http://127.0.0.1:1/v1","auth_header":"both"},"model_protocols":{"m":{"native_endpoints":[],"magpie_prefers":"/v1/messages"}}}],"groups":{"g":[]},"group_match":{"g":["relay/*"]}}"#;
        fs::write(dir.path().join("settings.json"), legacy)?;
        // Serialized by the legacy editor, including its default and omitted fields.
        let previous = r#"{"listen":"[::1]:3425","providers":[{"id":"relay","endpoints":{"chat":"http://127.0.0.1:1/v1","responses":"","anthropic":"","gemini":""},"key_env":"","models":["m"],"no_stand_in":true,"model_protocols":{"m":{"native_endpoints":[],"magpie_prefers":"/v1/messages"}}}],"groups":{"g":[]},"group_match":{"g":["relay/*"]},"peer_listen":"","allowed_peers":[]}"#;
        let before = snapshot(dir.path())?;
        save(dir.path(), previous, legacy)?;
        let reordered: BTreeMap<String, nextjson::Value> = nextjson::from_str(previous).unwrap();
        save(
            dir.path(),
            &nextjson::to_string(&reordered).unwrap(),
            legacy,
        )?;
        assert_eq!(snapshot(dir.path())?, before);
        assert!(save(dir.path(), "{}", legacy).is_err());
        assert_eq!(Config::parse("{}")?.listen, Config::default().listen);
        for invalid in [r#"{"providers":[],"providers":[]}"#, r#"{"unknown":0}"#] {
            assert!(Config::parse(invalid).is_err(), "{invalid}");
        }
        Ok(())
    }

    #[test]
    fn api_key_headers_match_vendor_and_protocol() -> Result<()> {
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
                http::Request::get(url.as_str()),
                &url,
                protocol,
                "example",
                AuthHeader::Both,
            )
            .body(())?;
            assert_eq!(request.headers().len(), 1);
            assert_eq!(request.headers()[header], value);
            if header == "api-key" {
                assert_eq!(request.uri().path(), "/openai/v1");
            }
        }
        let url = api_base("https://api.deepseek.com/anthropic", "anthropic")?;
        let request = authenticate(
            http::Request::get(url.as_str()),
            &url,
            "anthropic",
            "example",
            AuthHeader::Both,
        )
        .body(())?;
        assert_eq!(request.headers()["authorization"], "Bearer example");
        assert_eq!(request.headers()["x-api-key"], "example");
        Ok(())
    }
}
