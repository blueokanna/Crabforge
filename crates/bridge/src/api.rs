use anyhow::{Result, ensure};
use crabforge_core::{config, gateway, peer, service::Service};
use serde_json::json;
use std::sync::LazyLock;
use tokio::sync::Mutex;

static SERVER: LazyLock<Mutex<Option<Service>>> = LazyLock::new(|| Mutex::new(None));
static LOGIN: Mutex<Option<crabforge_core::webchat::Browser>> = Mutex::const_new(None);

pub async fn web_login(site: String) -> Result<()> {
    let mut login = LOGIN.lock().await;
    ensure!(login.is_none(), "a sign-in window is already open");
    let site = match site.as_str() {
        "deepseek" => crabforge_core::webchat::Site::DeepSeek,
        "chatglm" => crabforge_core::webchat::Site::ChatGLM,
        _ => anyhow::bail!("unknown web chat site"),
    };
    let browser = crabforge_core::webchat::Browser::open(&config::data_dir()?, site).await?;
    if let Err(e) = browser.visit().await {
        browser.close().await;
        return Err(e);
    }
    *login = Some(browser);
    Ok(())
}

pub async fn finish_web_login() {
    if let Some(browser) = LOGIN.lock().await.take() {
        browser.close().await;
    }
}
struct Tunnel {
    address: std::net::SocketAddr,
    task: tokio::task::JoinHandle<Result<()>>,
}
static TUNNEL: Mutex<Option<Tunnel>> = Mutex::const_new(None);

pub async fn connect_peer(remote: String, listen: String) -> Result<()> {
    let mut tunnel = TUNNEL.lock().await;
    ensure!(
        tunnel.as_ref().is_none_or(|t| t.task.is_finished()),
        "a P2P tunnel is already open"
    );
    let key = peer::identity(&config::data_dir()?)?;
    let (address, task) = peer::tunnel(key, &remote, listen.parse()?).await?;
    *tunnel = Some(Tunnel { address, task });
    Ok(())
}

pub async fn disconnect_peer() {
    let mut guard = TUNNEL.lock().await;
    if let Some(tunnel) = guard.take() {
        tunnel.task.abort();
        let _ = tunnel.task.await;
    }
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn configuration() -> Result<String> {
    config::snapshot(&config::data_dir()?)
}

pub fn rename_provider(previous: String, from: String, to: String) -> Result<()> {
    config::rename_provider(&config::data_dir()?, &previous, &from, &to)
}

pub fn route_groups(configuration: String) -> Result<String> {
    let cfg = config::Config::parse(&configuration)?;
    let groups = cfg.resolved_groups();
    Ok(serde_json::to_string(
        &serde_json::json!({"members":groups.members,"expanded":groups.expanded,"public_refs":groups.public_refs,"order":groups.ordered_names(&cfg.group_display_order)}),
    )?)
}

pub fn match_group_models(configuration: String, patterns: Vec<String>) -> Result<Vec<String>> {
    let mut cfg = crabforge_core::config::Config::parse(&configuration)?;
    cfg.set_member_patterns("", patterns)?;
    Ok(cfg.expand_patterns("", &[]))
}

pub fn renamed_group(configuration: String, from: String, to: String) -> Result<String> {
    let cfg = config::Config::parse(&configuration)?.renamed_group(&from, &to)?;
    Ok(serde_json::to_string(&cfg)?)
}

pub fn copied_group(configuration: String, from: String) -> Result<String> {
    let (cfg, _) = config::Config::parse(&configuration)?.copied_group(&from, "")?;
    Ok(serde_json::to_string(&cfg)?)
}

pub fn shared_group(configuration: String, id: String) -> Result<String> {
    config::Config::parse(&configuration)?.share_group(&id)
}

pub fn imported_group(configuration: String, text: String, replace: bool) -> Result<String> {
    let (cfg, report) = config::Config::parse(&configuration)?.import_group(&text, replace)?;
    Ok(serde_json::to_string(
        &serde_json::json!({"configuration": cfg, "report": report}),
    )?)
}

pub fn ui_language() -> Result<String> {
    let path = config::data_dir()?.join("ui-language");
    match std::fs::read_to_string(path) {
        Ok(language) => Ok(language),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e.into()),
    }
}

pub fn set_ui_language(language: String) -> Result<()> {
    use std::io::Write;
    ensure!(
        matches!(
            language.as_str(),
            "" | "zh" | "en" | "fr" | "de" | "hi" | "ja" | "ko" | "pt" | "es"
        ),
        "unsupported UI language"
    );
    let dir = config::data_dir()?;
    std::fs::create_dir_all(&dir)?;
    let mut file = tempfile::NamedTempFile::new_in(&dir)?;
    file.write_all(language.as_bytes())?;
    file.as_file().sync_all()?;
    file.persist(dir.join("ui-language"))?;
    Ok(())
}
pub fn presets() -> String {
    crabforge_core::catalog::presets().expect("valid embedded provider presets")
}

pub fn agents() -> String {
    json!(crabforge_core::agents::CLIENTS).to_string()
}

pub fn omp_profiles() -> Result<String> {
    use crabforge_core::agents::omp_profiles;
    let profiles = omp_profiles::list(&config::data_dir()?, &omp_profiles::installed()?)?;
    Ok(serde_json::to_string(&profiles)?)
}

pub fn omp_advisor(profile: String) -> Result<String> {
    use crabforge_core::agents::{omp_profiles, omp_settings};
    let home = omp_profiles::settings_directory(&config::data_dir()?, Some(&profile))?;
    Ok(serde_json::to_string(&omp_settings::read_advisor(&home)?)?)
}

pub fn save_omp_advisor(profile: String, previous: String, changes: String) -> Result<()> {
    use crabforge_core::agents::{omp_profiles, omp_settings};
    let home = omp_profiles::settings_directory(&config::data_dir()?, Some(&profile))?;
    omp_settings::save_advisor(
        &home,
        Some(&serde_json::from_str(&previous)?),
        &serde_json::from_str(&changes)?,
    )?;
    Ok(())
}

pub fn omp_thinking(profile: String) -> Result<String> {
    use crabforge_core::agents::{omp_profiles, omp_settings};
    let home = omp_profiles::settings_directory(&config::data_dir()?, Some(&profile))?;
    Ok(serde_json::to_string(&omp_settings::read_thinking(&home)?)?)
}

pub fn sync_omp_catalog(profile: String) -> Result<u32> {
    let count = crabforge_core::agents::omp_catalog::sync(&config::data_dir()?, Some(&profile))?;
    Ok(count.try_into()?)
}

pub fn omp_protocol(profile: String) -> Result<String> {
    crabforge_core::agents::omp_catalog::read_protocol(&config::data_dir()?, Some(&profile))
}

pub fn save_omp_protocol(profile: String, previous: String, protocol: String) -> Result<()> {
    crabforge_core::agents::omp_catalog::set_protocol(
        &config::data_dir()?,
        Some(&profile),
        &protocol,
        Some(&previous),
    )
}

pub fn omp_roles(profile: String) -> Result<String> {
    use crabforge_core::agents::{omp_profiles, omp_settings::roles};
    let dir = config::data_dir()?;
    let home = omp_profiles::settings_directory(&dir, Some(&profile))?;
    Ok(serde_json::to_string(&roles::read_roles(
        &home,
        &config::load(&dir)?,
    )?)?)
}

pub fn save_omp_role(
    profile: String,
    previous: String,
    role: String,
    model: Option<String>,
) -> Result<()> {
    use crabforge_core::agents::{omp_profiles, omp_settings::roles};
    let dir = config::data_dir()?;
    let home = omp_profiles::settings_directory(&dir, Some(&profile))?;
    roles::save_role(
        &home,
        &config::load(&dir)?,
        &role,
        model.as_deref(),
        Some(&serde_json::from_str(&previous)?),
    )
}

pub fn save_omp_thinking(profile: String, previous: String, changes: String) -> Result<()> {
    use crabforge_core::agents::{omp_profiles, omp_settings};
    let home = omp_profiles::settings_directory(&config::data_dir()?, Some(&profile))?;
    omp_settings::save_thinking(
        &home,
        Some(&serde_json::from_str(&previous)?),
        &serde_json::from_str(&changes)?,
    )?;
    Ok(())
}

pub async fn discover_models(endpoints: String, key_env: String) -> Result<Vec<String>> {
    crabforge_core::catalog::discover(&serde_json::from_str(&endpoints)?, &key_env).await
}

pub async fn discover_model_catalog(endpoints: String, key_env: String) -> Result<String> {
    let catalog =
        crabforge_core::catalog::discover_catalog(&serde_json::from_str(&endpoints)?, &key_env)
            .await?;
    Ok(serde_json::to_string(&catalog)?)
}
pub fn gateway_key() -> Result<String> {
    config::token(&config::data_dir()?)
}
pub fn visit_web(site: String) -> Result<()> {
    config::open_web(&site)
}
pub fn node_id() -> Result<String> {
    Ok(peer::identity(&config::data_dir()?)?
        .public()
        .to_peer_id()
        .to_string())
}

pub async fn save_configuration(previous: String, text: String) -> Result<()> {
    let guard = SERVER.lock().await;
    if guard.as_ref().is_some_and(|s| s.running()) {
        let mut allowed = config::Config::parse(&previous)?;
        let updated = config::Config::parse(&text)?;
        for (group, pick) in &updated.group_picks {
            if allowed.group_picks.get(group) != Some(pick) {
                allowed.pick_member(group, pick)?;
            }
        }
        ensure!(
            serde_json::to_vec(&allowed)? == serde_json::to_vec(&updated)?,
            "stop the gateway before editing settings other than manual selections"
        );
    }
    config::save(&config::data_dir()?, &previous, &text)
}

pub async fn set_running(on: bool) -> Result<()> {
    let mut server = SERVER.lock().await;
    if on && server.as_ref().is_none_or(|s| !s.running()) {
        if let Some(previous) = server.take() {
            previous.stop().await;
        }
        *server = Some(Service::start(&config::data_dir()?).await?);
    }
    if !on && let Some(previous) = server.take() {
        previous.stop().await;
    }
    Ok(())
}

pub async fn status() -> Result<String> {
    let guard = SERVER.lock().await;
    let server = guard.as_ref();
    let attempts=server.map(|s| {
        let history=s.history.lock().unwrap_or_else(|p|p.into_inner());
        history.iter().rev().map(|a|json!({"at":a.at,"agent":a.agent,"session":a.session,"native_session":a.native_session,"model":a.model,"provider":a.provider,"via":a.via,"status":a.status,"reason":gateway::status_text(a.status)})).collect::<Vec<_>>()
    }).unwrap_or_default();
    let mut state = json!({"running":server.is_some_and(Service::running),"address":server.map(|s|s.addr),"peer":server.and_then(|s|s.peer.as_ref()).map(|p|p.address.to_string()),"attempts":attempts,"version":crabforge_core::VERSION});
    drop(guard);
    let tunnel = TUNNEL.lock().await;
    state["tunnel"] = json!(
        tunnel
            .as_ref()
            .filter(|t| !t.task.is_finished())
            .map(|t| t.address)
    );
    Ok(state.to_string())
}
