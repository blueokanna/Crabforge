use anyhow::{Context, Result, ensure};
use crabforge_core::{config, gateway, peer, service::Service};
use nextjson::{NsonDeserialize, NsonSerialize, json};
use std::sync::LazyLock;
use tokio::sync::Mutex;

static SERVER: LazyLock<Mutex<Option<Service>>> = LazyLock::new(|| Mutex::new(None));
static LOGIN: Mutex<Option<crabforge_core::webchat::Browser>> = Mutex::const_new(None);

fn decode<'a, T: NsonDeserialize<'a>>(text: &'a str) -> Result<T> {
    nextjson::from_str(text).map_err(anyhow::Error::msg)
}

fn encode<T: NsonSerialize>(value: &T) -> Result<String> {
    nextjson::to_string(value).map_err(anyhow::Error::msg)
}

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

pub async fn connect_peer(
    remote: String,
    listen: String,
    relay_fallback: Option<String>,
) -> Result<()> {
    let mut tunnel = TUNNEL.lock().await;
    ensure!(
        tunnel.as_ref().is_none_or(|t| t.task.is_finished()),
        "a P2P tunnel is already open"
    );
    let key = peer::identity(&config::data_dir()?)?;
    let (address, task) =
        peer::tunnel_with_relay(key, &remote, relay_fallback.as_deref(), listen.parse()?).await?;
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
    encode(
        &json!({"members":groups.members,"expanded":groups.expanded,"public_refs":groups.public_refs,"order":groups.ordered_names(&cfg.group_display_order)}),
    )
}

pub fn match_group_models(configuration: String, patterns: Vec<String>) -> Result<Vec<String>> {
    let mut cfg = crabforge_core::config::Config::parse(&configuration)?;
    cfg.set_member_patterns("", patterns)?;
    Ok(cfg.expand_patterns("", &[]))
}

pub fn renamed_group(configuration: String, from: String, to: String) -> Result<String> {
    let cfg = config::Config::parse(&configuration)?.renamed_group(&from, &to)?;
    cfg.to_json()
}

pub fn copied_group(configuration: String, from: String) -> Result<String> {
    let (cfg, _) = config::Config::parse(&configuration)?.copied_group(&from, "")?;
    cfg.to_json()
}

pub fn shared_group(configuration: String, id: String) -> Result<String> {
    config::Config::parse(&configuration)?.share_group(&id)
}

pub fn imported_group(configuration: String, text: String, replace: bool) -> Result<String> {
    let (cfg, report) = config::Config::parse(&configuration)?.import_group(&text, replace)?;
    encode(&json!({"configuration": cfg, "report": report}))
}

pub fn ui_language() -> Result<String> {
    read_preference("ui-language")
}

fn read_preference(name: &str) -> Result<String> {
    match std::fs::read_to_string(config::data_dir()?.join(name)) {
        Ok(value) => Ok(value),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e.into()),
    }
}

pub fn set_ui_language(language: String) -> Result<()> {
    ensure!(
        matches!(
            language.as_str(),
            "" | "zh" | "en" | "fr" | "de" | "hi" | "ja" | "ko" | "pt" | "es"
        ),
        "unsupported UI language"
    );
    write_preference("ui-language", &language)
}

pub fn ui_layout() -> Result<String> {
    read_preference("ui-layout")
}

pub fn set_ui_layout(grid: bool) -> Result<()> {
    write_preference("ui-layout", if grid { "grid" } else { "list" })
}

fn write_preference(name: &str, value: &str) -> Result<()> {
    use std::io::Write;
    let dir = config::data_dir()?;
    std::fs::create_dir_all(&dir)?;
    let mut file = tempfile::NamedTempFile::new_in(&dir)?;
    file.write_all(value.as_bytes())?;
    file.as_file().sync_all()?;
    file.persist(dir.join(name))?;
    Ok(())
}
pub fn presets() -> String {
    crabforge_core::catalog::presets().expect("valid embedded provider presets")
}

pub fn agents() -> String {
    json!(crabforge_core::agents::CLIENTS).to_string()
}

pub fn agent_rules() -> Result<String> {
    crabforge_core::rules::library(&config::data_dir()?, crabforge_core::rules::Edit::Read)
}

pub fn project_mcp_servers(project: String, agent: String) -> Result<String> {
    use crabforge_core::project_mcp::{self, Edit};
    project_mcp::edit(&config::data_dir()?, project.as_ref(), &agent, Edit::Read)
}

pub fn save_project_mcp_server(
    project: String,
    agent: String,
    name: String,
    source: String,
    revision: String,
    replace: bool,
) -> Result<String> {
    use crabforge_core::project_mcp::{self, Edit};
    project_mcp::edit(
        &config::data_dir()?,
        project.as_ref(),
        &agent,
        Edit::Save {
            name,
            source,
            expected: Some(revision),
            replace,
        },
    )
}

pub fn remove_project_mcp_server(
    project: String,
    agent: String,
    name: String,
    revision: String,
) -> Result<String> {
    use crabforge_core::project_mcp::{self, Edit};
    project_mcp::edit(
        &config::data_dir()?,
        project.as_ref(),
        &agent,
        Edit::Remove {
            name,
            expected: revision,
        },
    )
}

pub fn agent_skills() -> Result<String> {
    crabforge_core::skills::library(&config::data_dir()?, crabforge_core::skills::Edit::Read)
}

pub fn discover_agent_skills() -> Result<String> {
    crabforge_core::skills::discover(&config::data_dir()?)
}

pub fn check_agent_skill_update(name: String, revision: String) -> Result<String> {
    crabforge_core::skills::check_update(&config::data_dir()?, &name, &revision)
}

pub fn repository_skill_catalog(source: String) -> Result<String> {
    crabforge_core::skills::source_catalog(&config::data_dir()?, &source)
}

pub fn dismiss_skill_candidate(source: String) -> Result<()> {
    crabforge_core::skills::dismiss_candidate(&config::data_dir()?, &source)
}

pub fn agent_environment() -> Result<String> {
    use crabforge_core::environment::{self, Edit};
    environment::edit(&config::data_dir()?, Edit::Read, true)
}

pub fn save_agent_variable(
    name: String,
    value: String,
    agents: Vec<String>,
    old_name: Option<String>,
    revision: Option<String>,
) -> Result<String> {
    use crabforge_core::environment::{self, Edit};
    let previous = match (old_name, revision) {
        (Some(name), Some(revision)) => Some((name, revision)),
        (None, None) => None,
        _ => anyhow::bail!("an edited variable needs its original name and revision"),
    };
    environment::edit(
        &config::data_dir()?,
        Edit::Save {
            name,
            value,
            agents,
            previous,
        },
        true,
    )
}

pub fn remove_agent_variable(name: String, revision: String) -> Result<String> {
    use crabforge_core::environment::{self, Edit};
    environment::edit(&config::data_dir()?, Edit::Remove { name, revision }, true)
}

pub fn sync_agent_environment() -> Result<String> {
    use crabforge_core::environment::{self, Edit};
    environment::edit(&config::data_dir()?, Edit::Sync, true)
}

pub fn agent_hooks() -> Result<String> {
    use crabforge_core::hooks::{self, Edit};
    hooks::edit(&config::data_dir()?, Edit::Read)
}

pub fn save_agent_hook(
    source: String,
    old: Option<String>,
    revision: Option<String>,
) -> Result<String> {
    use crabforge_core::hooks::{self, Edit};
    anyhow::ensure!(
        old.is_some() == revision.is_some(),
        "an edited hook needs its original name and revision"
    );
    hooks::edit(
        &config::data_dir()?,
        Edit::Save {
            source,
            previous: old.zip(revision),
        },
    )
}

pub fn adopt_agent_hook(key: String, name: String) -> Result<String> {
    use crabforge_core::hooks::{self, Edit};
    hooks::edit(&config::data_dir()?, Edit::Adopt { key, name })
}

pub fn remove_agent_hook(name: String, revision: String) -> Result<String> {
    use crabforge_core::hooks::{self, Edit};
    hooks::edit(&config::data_dir()?, Edit::Remove { name, revision })
}

pub fn sync_agent_hooks() -> Result<String> {
    use crabforge_core::hooks::{self, Edit};
    hooks::edit(&config::data_dir()?, Edit::Sync)
}

pub fn adopt_agent_skill(
    agent: String,
    name: String,
    path: String,
    content: String,
) -> Result<String> {
    crabforge_core::skills::library(
        &config::data_dir()?,
        crabforge_core::skills::Edit::Adopt {
            agent,
            name,
            path: path.into(),
            content,
        },
    )
}

pub fn project_mcp_library() -> Result<String> {
    use crabforge_core::project_mcp::library::{self, Edit};
    library::edit(&config::data_dir()?, Edit::Read)
}

pub fn save_mcp_library_server(
    name: String,
    source: String,
    revision: Option<String>,
) -> Result<String> {
    use crabforge_core::project_mcp::library::{self, Edit};
    library::edit(
        &config::data_dir()?,
        Edit::Save {
            name,
            source,
            expected: revision,
        },
    )
}

pub fn assign_mcp_library_server(
    name: String,
    project: String,
    agent: String,
    enabled: bool,
    revision: String,
) -> Result<String> {
    use crabforge_core::project_mcp::library::{self, Edit};
    library::edit(
        &config::data_dir()?,
        Edit::Assign {
            name,
            project: project.into(),
            agent,
            enabled,
            expected: revision,
        },
    )
}

pub fn remove_mcp_library_server(name: String, revision: String) -> Result<String> {
    use crabforge_core::project_mcp::library::{self, Edit};
    library::edit(
        &config::data_dir()?,
        Edit::Remove {
            name,
            expected: revision,
        },
    )
}

pub fn sync_mcp_library() -> Result<String> {
    use crabforge_core::project_mcp::library::{self, Edit};
    library::edit(&config::data_dir()?, Edit::Sync)
}

pub fn import_agent_skill(
    source: String,
    agents: Vec<String>,
    revision: Option<String>,
) -> Result<String> {
    crabforge_core::skills::library(
        &config::data_dir()?,
        crabforge_core::skills::Edit::Import {
            source: source.into(),
            agents,
            expected: revision,
        },
    )
}

pub fn assign_agent_skill(name: String, agents: Vec<String>, revision: String) -> Result<String> {
    crabforge_core::skills::library(
        &config::data_dir()?,
        crabforge_core::skills::Edit::Assign {
            name,
            agents,
            expected: Some(revision),
        },
    )
}

pub fn remove_agent_skill(name: String, revision: String) -> Result<String> {
    crabforge_core::skills::library(
        &config::data_dir()?,
        crabforge_core::skills::Edit::Remove {
            name,
            expected: Some(revision),
        },
    )
}

pub fn sync_agent_skills() -> Result<String> {
    crabforge_core::skills::library(&config::data_dir()?, crabforge_core::skills::Edit::Sync)
}

pub fn save_agent_rule(
    old: Option<String>,
    revision: Option<String>,
    rule: String,
) -> Result<String> {
    crabforge_core::rules::library(
        &config::data_dir()?,
        crabforge_core::rules::Edit::Save {
            old,
            expected: revision,
            rule: decode(&rule)?,
        },
    )
}

pub fn remove_agent_rule(name: String, revision: String) -> Result<String> {
    crabforge_core::rules::library(
        &config::data_dir()?,
        crabforge_core::rules::Edit::Remove {
            name,
            expected: Some(revision),
        },
    )
}

pub fn sync_agent_rules() -> Result<String> {
    crabforge_core::rules::library(&config::data_dir()?, crabforge_core::rules::Edit::Sync)
}

pub fn agent_sessions(agent: Option<String>) -> Result<String> {
    use crabforge_core::agents::NativeAgent;

    let agent = agent
        .map(|id| NativeAgent::by_id(&id).context("unsupported session agent"))
        .transpose()?;
    encode(&crabforge_core::sessions::list(agent, 50)?)
}

pub fn preview_session_memory(agent: String, profile: String, session: String) -> Result<String> {
    let agent =
        crabforge_core::agents::NativeAgent::by_id(&agent).context("unsupported session agent")?;
    encode(&crabforge_core::memory::distill::preview(
        agent,
        Some(profile.as_ref()),
        &session,
    )?)
}

pub async fn propose_session_memory(
    agent: String,
    profile: String,
    session: String,
    model: String,
    revision: String,
) -> Result<String> {
    let agent =
        crabforge_core::agents::NativeAgent::by_id(&agent).context("unsupported session agent")?;
    encode(
        &crabforge_core::memory::distill::propose(
            &config::data_dir()?,
            agent,
            Some(profile.as_ref()),
            &session,
            &model,
            &revision,
        )
        .await?,
    )
}

pub fn search_memories(query: String) -> Result<String> {
    encode(&crabforge_core::memory::search(
        &config::data_dir()?,
        decode(&query)?,
    )?)
}

pub fn memory_project_key(project: String) -> Result<String> {
    crabforge_core::memory::project_key(project.as_ref())
}

pub fn save_memory(
    id: Option<String>,
    revision: Option<String>,
    changes: String,
) -> Result<String> {
    encode(&crabforge_core::memory::save(
        &config::data_dir()?,
        id.as_deref(),
        revision.as_deref(),
        decode(&changes)?,
    )?)
}

pub fn project_instruction_agents() -> Vec<String> {
    crabforge_core::project_instructions::agents()
        .map(str::to_owned)
        .collect()
}

pub fn project_instructions(project: String, agent: String) -> Result<String> {
    encode(&crabforge_core::project_instructions::read(
        &config::data_dir()?,
        project.as_ref(),
        &agent,
    )?)
}

pub fn edit_project_instructions(
    project: String,
    agent: String,
    revision: String,
    text: Option<String>,
) -> Result<()> {
    crabforge_core::project_instructions::edit(
        &config::data_dir()?,
        project.as_ref(),
        &agent,
        &revision,
        text.as_deref(),
    )
}

pub fn project_memory_digest(project: String) -> Result<String> {
    crabforge_core::memory::project_digest(&config::data_dir()?, project.as_ref())
}

pub fn publish_project_memory(
    project: String,
    agent: String,
    revision: String,
    preview: String,
) -> Result<()> {
    crabforge_core::project_instructions::publish_memory(
        &config::data_dir()?,
        project.as_ref(),
        &agent,
        &revision,
        Some(&preview),
    )
}

pub fn agent_instructions(agent: String) -> Result<String> {
    encode(&crabforge_core::project_instructions::read_agent(
        &config::data_dir()?,
        &agent,
    )?)
}

pub fn edit_agent_instructions(
    agent: String,
    revision: String,
    text: Option<String>,
) -> Result<()> {
    crabforge_core::project_instructions::edit_agent(
        &config::data_dir()?,
        &agent,
        &revision,
        text.as_deref(),
    )
}

pub fn agent_memory_digest(agent: String) -> Result<String> {
    crabforge_core::memory::agent_digest(&config::data_dir()?, Some(&agent))
}

pub fn publish_agent_memory(agent: String, revision: String, preview: String) -> Result<()> {
    crabforge_core::project_instructions::publish_agent_memory(
        &config::data_dir()?,
        &agent,
        &revision,
        Some(&preview),
    )
}

pub fn omp_profiles() -> Result<String> {
    use crabforge_core::agents::omp_profiles;
    let profiles = omp_profiles::list(&config::data_dir()?, &omp_profiles::installed()?)?;
    let profiles = profiles
        .iter()
        .map(|p| {
            let directory = p
                .directory
                .to_str()
                .context("profile path must be valid UTF-8")?;
            Ok(json!({"name":p.name,"active":p.active,"directory":directory}))
        })
        .collect::<Result<Vec<_>>>()?;
    encode(&profiles)
}

pub fn omp_advisor(profile: String) -> Result<String> {
    use crabforge_core::agents::{omp_profiles, omp_settings};
    let home = omp_profiles::settings_directory(&config::data_dir()?, Some(&profile))?;
    encode(&omp_settings::read_advisor(&home)?)
}

pub fn save_omp_advisor(profile: String, previous: String, changes: String) -> Result<()> {
    use crabforge_core::agents::{omp_profiles, omp_settings};
    let home = omp_profiles::settings_directory(&config::data_dir()?, Some(&profile))?;
    omp_settings::save_advisor(&home, Some(&decode(&previous)?), &decode(&changes)?)?;
    Ok(())
}

pub fn omp_thinking(profile: String) -> Result<String> {
    use crabforge_core::agents::{omp_profiles, omp_settings};
    let home = omp_profiles::settings_directory(&config::data_dir()?, Some(&profile))?;
    encode(&omp_settings::read_thinking(&home)?)
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
    encode(&roles::read_roles(&home, &config::load(&dir)?)?)
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
        Some(&decode(&previous)?),
    )
}

pub fn save_omp_thinking(profile: String, previous: String, changes: String) -> Result<()> {
    use crabforge_core::agents::{omp_profiles, omp_settings};
    let home = omp_profiles::settings_directory(&config::data_dir()?, Some(&profile))?;
    omp_settings::save_thinking(&home, Some(&decode(&previous)?), &decode(&changes)?)?;
    Ok(())
}

pub async fn discover_models(endpoints: String, key_env: String) -> Result<Vec<String>> {
    crabforge_core::catalog::discover(&decode(&endpoints)?, &key_env).await
}

pub async fn discover_model_catalog(endpoints: String, key_env: String) -> Result<String> {
    let catalog = crabforge_core::catalog::discover_catalog(&decode(&endpoints)?, &key_env).await?;
    encode(&catalog)
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
            allowed.to_json()? == updated.to_json()?,
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
    let running = server.is_some_and(Service::running);
    let address = server.map(|s| s.addr);
    let peer = server
        .and_then(|s| s.peer.as_ref())
        .map(|p| p.address.to_string());
    drop(guard);
    let tunnel = TUNNEL.lock().await;
    let tunnel = tunnel
        .as_ref()
        .filter(|t| !t.task.is_finished())
        .map(|t| t.address);
    encode(
        &json!({"running":running,"address":address,"peer":peer,"attempts":attempts,"version":crabforge_core::VERSION,"tunnel":tunnel}),
    )
}
