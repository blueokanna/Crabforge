use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use crabforge_core::webchat::{Browser, Site};
use crabforge_core::{config, peer, service::Service};
use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf};
mod agent;
mod agent_files;
mod claude_plugins;
mod environment;
mod groups;
mod hooks;
mod memory;
mod plugin_bundles;
mod project_mcp;
mod rules;
mod skills;

#[derive(Parser)]
#[command(
    version,
    about = "Crabforge model gateway and authenticated QUIC tunnels"
)]
struct Args {
    /// Configuration directory (defaults to the user's com.blue.crabforge folder).
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage configured upstream providers.
    Provider {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    /// Manage named gateway route groups.
    #[command(visible_alias = "groups")]
    Group {
        #[command(subcommand)]
        command: Option<GroupCommand>,
    },
    /// Preview or atomically import supported settings from a sealed Magpie backup.
    ImportMagpie {
        file: PathBuf,
        /// Select a provider by ID. Repeat to import several. Defaults to all providers.
        #[arg(long = "provider")]
        providers: Vec<String>,
        /// Select a group by ID. Repeat to include nested groups. Defaults to all groups.
        #[arg(long = "group")]
        groups: Vec<String>,
        /// Read this provider's API key from the specified environment variable.
        #[arg(long = "key-env", value_parser = parse_key_env, value_name = "PROVIDER=VARIABLE")]
        key_envs: Vec<(String, String)>,
        /// Include the gateway port, browser origins and equivalent model names from the backup.
        #[arg(long)]
        settings: bool,
        /// Import those settings without loading the backup's providers or groups.
        #[arg(long, conflicts_with_all = ["settings", "providers", "groups", "key_envs"])]
        settings_only: bool,
        /// Save the selected settings to Crabforge's data directory in one write.
        #[arg(long)]
        apply: bool,
    },
    /// Print the API endpoint presets as JSON.
    Catalog,
    /// List the agent-specific API roots accepted by this gateway.
    Agents,
    /// Read the managed block in an agent's project file, edit it or undo the last edit.
    ProjectInstructions {
        project: PathBuf,
        agent: String,
        /// UTF-8 Markdown to put inside the managed block.
        #[arg(long, group = "change")]
        from: Option<PathBuf>,
        #[arg(long, group = "change")]
        clear: bool,
        #[arg(long, group = "change")]
        restore: bool,
        /// Reject the edit if the file has changed since this revision was read.
        #[arg(long)]
        revision: Option<String>,
    },
    /// Search, create and edit local Agent memories.
    Memory {
        #[command(subcommand)]
        command: memory::Command,
    },
    /// Manage shared Markdown rules for Claude Code, Cline and Kiro CLI.
    Rules {
        #[command(subcommand)]
        command: rules::Command,
    },
    /// Import local skill folders and assign them to native agent profiles.
    Skills {
        #[command(subcommand)]
        command: skills::Command,
    },
    /// Manage shared variables for Claude Code, Codex and Pi.
    #[command(visible_alias = "env")]
    Environment {
        #[command(subcommand)]
        command: environment::Command,
    },
    /// Manage hook commands and their client assignments.
    Hooks {
        /// Manage assignments or native hooks in the selected project.
        #[arg(long, global = true)]
        project: Option<PathBuf>,
        #[command(subcommand)]
        command: hooks::Command,
    },
    /// Manage subagent definitions for native clients.
    Subagents {
        /// Publish in the selected project instead of the user profile.
        #[arg(long, global = true)]
        project: Option<PathBuf>,
        #[command(subcommand)]
        command: agent_files::Command,
    },
    /// Manage slash commands and prompt templates for native clients.
    Commands {
        #[command(subcommand)]
        command: agent_files::Command,
    },
    /// List installed Claude Code plugins and manage user or project enablement.
    ClaudePlugins {
        /// Manage plugin assignments in this project.
        #[arg(long, global = true)]
        project: Option<PathBuf>,
        #[command(subcommand)]
        command: claude_plugins::Command,
    },
    /// Preview and save plugin bundles in the shared library.
    PluginBundles {
        #[command(subcommand)]
        command: plugin_bundles::Command,
    },
    /// Discover recent local Claude Code, Codex, Gemini CLI and Pi sessions.
    Sessions {
        #[arg(long, value_enum)]
        agent: Option<SessionAgent>,
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u16).range(1..=500))]
        limit: u16,
    },
    /// Run a local MCP server over standard input and output.
    Mcp {
        #[command(subcommand)]
        command: memory::McpCommand,
    },
    /// Manage MCP servers in a project's native agent configuration.
    ProjectMcp {
        #[command(subcommand)]
        command: project_mcp::Command,
    },
    /// List Oh My Pi profiles, the active profile and each agent directory.
    OmpProfiles,
    /// Show role assignments and available models, set one role, or restore its previous value.
    OmpRoles {
        #[arg(long)]
        profile: Option<String>,
        #[arg(long, requires = "action")]
        role: Option<String>,
        #[arg(long, requires = "role", group = "action")]
        model: Option<String>,
        #[arg(long, requires = "role", group = "action")]
        reset: bool,
    },
    /// Refresh the selected Oh My Pi profile's model catalog and enabledModels.
    OmpSync {
        #[arg(long)]
        profile: Option<String>,
    },
    /// Show or select the gateway protocol for every chat model in an Oh My Pi profile.
    OmpProtocol {
        #[arg(long)]
        profile: Option<String>,
        #[arg(value_parser = ["auto", "openai-completions", "openai-responses", "anthropic-messages", "google-generative-ai"])]
        protocol: Option<String>,
    },
    /// Read or set the thinking level for sessions and single-model roles.
    OmpThinking {
        #[arg(long)]
        profile: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long, requires = "level")]
        role: Option<String>,
        #[arg(long, requires = "role")]
        level: Option<String>,
    },
    /// Read or change a profile's Oh My Pi advisor. Use inherit to reset a setting.
    OmpAdvisor {
        #[arg(long)]
        profile: Option<String>,
        #[arg(long, value_parser=["inherit", "off", "turn", "agent-end"])]
        review: Option<String>,
        #[arg(long, value_parser=["inherit", "off", "1", "3", "5", "strict"])]
        wait: Option<String>,
        #[arg(long, value_parser=["inherit", "nit", "concern", "blocker"])]
        wake: Option<String>,
    },
    /// Launch an installed client with a model from the running gateway.
    Agent {
        #[arg(value_enum)]
        client: agent::Agent,
        model: String,
        /// Choose an Oh My Pi profile. Defaults to OMP_PROFILE, then PI_PROFILE.
        #[arg(long)]
        profile: Option<String>,
        #[arg(last = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// List model IDs from a provider's live catalog or its built-in list.
    Models {
        provider: String,
        /// Replace this provider's configured models and free flags with the current upstream catalog.
        #[arg(long)]
        sync: bool,
    },
    /// Show the active settings, or validate and import a JSON file.
    Config { file: Option<PathBuf> },
    /// Print the gateway API key for configuring an agent.
    Key,
    /// Print this installation's stable, public libp2p identity.
    PeerId,
    /// Serve the configured HTTP and P2P endpoints until interrupted.
    Serve,
    /// Bind a local HTTP endpoint to a trusted remote peer over QUIC.
    Tunnel {
        remote: String,
        /// Relay circuit address for the same peer, tried if the direct QUIC dial fails.
        #[arg(long)]
        relay_fallback: Option<String>,
        #[arg(long, default_value = "127.0.0.1:3426")]
        listen: SocketAddr,
    },
    /// Open the official DeepSeek or ChatGLM chat in the default browser.
    Web {
        #[arg(value_parser=["deepseek","chatglm"])]
        site: String,
        /// Sign in to the browser profile used by the site's gateway provider.
        #[arg(long)]
        login: bool,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum SessionAgent {
    Claude,
    Codex,
    Gemini,
    Pi,
}

impl From<SessionAgent> for crabforge_core::agents::NativeAgent {
    fn from(agent: SessionAgent) -> Self {
        match agent {
            SessionAgent::Claude => Self::Claude,
            SessionAgent::Codex => Self::Codex,
            SessionAgent::Gemini => Self::Gemini,
            SessionAgent::Pi => Self::Pi,
        }
    }
}

#[derive(Subcommand)]
enum ProviderCommand {
    /// Preview provider settings from a sealed Magpie backup, or add them with --apply.
    ImportMagpie {
        file: PathBuf,
        /// Select a provider by ID. Repeat to import several. Defaults to all providers.
        #[arg(long = "provider")]
        providers: Vec<String>,
        /// Read this provider's API key from the specified environment variable.
        #[arg(long = "key-env", value_parser = parse_key_env, value_name = "PROVIDER=VARIABLE")]
        key_envs: Vec<(String, String)>,
        /// Save the selected settings to Crabforge's data directory.
        #[arg(long)]
        apply: bool,
    },
    /// Rename a provider and migrate its saved model references.
    Rename { from: String, to: String },
    /// Show or replace the ordered fallback models and groups. Use none to clear.
    Fallback { id: String, targets: Vec<String> },
    /// Set fallback-for-others=yes|no (also accepts on|off).
    Set {
        id: String,
        #[arg(action = clap::ArgAction::Set, value_parser = parse_stand_in)]
        setting: bool,
    },
}

fn parse_key_env(input: &str) -> Result<(String, String), String> {
    input
        .split_once('=')
        .filter(|(provider, variable)| !provider.is_empty() && !variable.is_empty())
        .map(|(provider, variable)| (provider.to_owned(), variable.to_owned()))
        .ok_or_else(|| "expected PROVIDER=VARIABLE".into())
}

fn key_env_map(entries: Vec<(String, String)>) -> Result<BTreeMap<String, String>> {
    let count = entries.len();
    let map: BTreeMap<_, _> = entries.into_iter().collect();
    ensure!(
        map.len() == count,
        "specify one key environment variable per provider"
    );
    Ok(map)
}

fn backup_passphrase() -> Result<zeroize::Zeroizing<String>> {
    use std::io::{BufRead, IsTerminal, Read, stdin};
    const MAX_PASSPHRASE: u64 = 16 * 1024;
    if stdin().is_terminal() {
        return Ok(zeroize::Zeroizing::new(rpassword::prompt_password(
            "Magpie backup passphrase: ",
        )?));
    }
    let mut line = zeroize::Zeroizing::new(String::new());
    stdin()
        .lock()
        .take(MAX_PASSPHRASE + 1)
        .read_line(&mut line)?;
    ensure!(
        line.len() as u64 <= MAX_PASSPHRASE,
        "Magpie backup passphrase is too long"
    );
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(line)
}

fn parse_stand_in(input: &str) -> Result<bool, String> {
    match input
        .strip_prefix("fallback-for-others=")
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("yes" | "on") => Ok(true),
        Some("no" | "off") => Ok(false),
        _ => Err("expected fallback-for-others=yes|no".into()),
    }
}

#[derive(Subcommand)]
enum GroupCommand {
    List,
    /// Print this group's portable Magpie routing-group JSON.
    Share {
        id: String,
    },
    /// Import a shared group from a UTF-8 file or from standard input.
    Import {
        file: Option<PathBuf>,
        #[arg(long)]
        replace: bool,
    },
    /// Preview routing groups from a sealed Magpie backup, or add them with --apply.
    ImportMagpie {
        file: PathBuf,
        /// Select a group by ID. Repeat to include nested groups. Defaults to all groups.
        #[arg(long = "group")]
        groups: Vec<String>,
        /// Save the selected groups to Crabforge's data directory.
        #[arg(long)]
        apply: bool,
    },
    /// Show one group's address, strategy, members, note and parent groups.
    Show {
        id: String,
    },
    #[command(external_subcommand)]
    ByName(Vec<String>),
    /// Switch a saved group on or off across the gateway and agent catalogs.
    Switch {
        id: String,
        #[arg(value_parser=["on","off"])]
        mode: String,
    },
    /// Switch a member on or off while preserving its position and weight.
    Member {
        id: String,
        member: String,
        #[arg(value_parser=["on","off"])]
        mode: String,
    },
    /// Duplicate a route group, including its routing strategy and selected member.
    #[command(visible_alias = "duplicate")]
    Copy {
        from: String,
        name: Vec<String>,
    },
    /// List these groups first, preserving the relative order of other visible groups.
    Order {
        #[arg(required = true)]
        ids: Vec<String>,
    },
    /// Put these members first in a pattern group. Omit members to restore catalog order.
    Place {
        id: String,
        members: Vec<String>,
    },
    /// Switch the selected member of an existing manual group, including while serving.
    Pick {
        id: String,
        member: String,
    },
    /// Rename a group, preserving its previous address and updating parent groups.
    Rename {
        from: String,
        to: String,
    },
    /// Enable or disable groups discovered across configured providers.
    Auto {
        #[arg(value_parser=["on","off"])]
        mode: String,
    },
    /// Restore a removed automatic group when its providers are available.
    Restore {
        id: String,
    },
    /// Create or update a group with explicit IDs, glob patterns, re: expressions or free: rules.
    Set {
        id: String,
        #[arg(required = true)]
        members: Vec<String>,
        /// Set the group's one-line note (up to 200 characters). Use --note= to clear it.
        #[arg(long)]
        note: Option<String>,
        /// List the group's direct models in client catalogs (yes or no).
        #[arg(long, value_parser=["yes", "no"])]
        listed: Option<String>,
        #[arg(long)]
        rotate: bool,
        /// Resume weighted routing with the saved weights, defaulting to one per member.
        #[arg(long, conflicts_with_all=["rotate", "manual", "pick"])]
        weighted: bool,
        /// Resume manual selection, using the last pick if it is still a member.
        #[arg(long, conflicts_with_all=["rotate", "weight", "pick"])]
        manual: bool,
        /// Route only to this member, including when it fails or lacks the requested protocol.
        #[arg(long, conflicts_with_all=["rotate", "weight"], value_name="MEMBER")]
        pick: Option<String>,
        /// Use smooth weighted round robin. Repeat for each override; unlisted members weigh one.
        #[arg(long, conflicts_with="rotate", value_name="PROVIDER/MODEL=WEIGHT", value_parser=parse_weight)]
        weight: Vec<(String, u32)>,
    },
    Remove {
        id: String,
    },
}

fn parse_weight(input: &str) -> Result<(String, u32), String> {
    let (id, value) = input
        .rsplit_once('=')
        .ok_or("expected provider/model=weight")?;
    let weight = value
        .parse::<u32>()
        .ok()
        .filter(|w| (1..=config::MAX_MEMBER_WEIGHT).contains(w))
        .ok_or_else(|| format!("weight must be between 1 and {}", config::MAX_MEMBER_WEIGHT))?;
    Ok((id.to_owned(), weight))
}

pub fn run() -> Result<()> {
    dispatch(Args::parse())
}

#[tokio::main(flavor = "current_thread")]
async fn dispatch(args: Args) -> Result<()> {
    let dir = match args.data_dir {
        Some(p) => p,
        None => config::data_dir()?,
    };
    match args.command {
        Command::ImportMagpie {
            file,
            providers,
            groups,
            key_envs,
            settings,
            settings_only,
            apply,
        } => {
            let key_envs = key_env_map(key_envs)?;
            let passphrase = backup_passphrase()?;
            let settings = match (settings, settings_only) {
                (_, true) => config::MagpieSettingsMode::Only,
                (true, false) => config::MagpieSettingsMode::Include,
                (false, false) => config::MagpieSettingsMode::Skip,
            };
            let report = config::import_magpie_backup(
                &dir,
                &file,
                &passphrase,
                config::MagpieImportOptions {
                    providers: &providers,
                    groups: &groups,
                    key_envs: &key_envs,
                    settings,
                    apply,
                },
            )?;
            println!(
                "{}",
                nextjson::to_string_pretty(&report).map_err(anyhow::Error::msg)?
            );
        }
        Command::Provider {
            command:
                ProviderCommand::ImportMagpie {
                    file,
                    providers,
                    key_envs,
                    apply,
                },
        } => {
            let key_envs = key_env_map(key_envs)?;
            let passphrase = backup_passphrase()?;
            let report = config::import_magpie_providers(
                &dir,
                &file,
                &passphrase,
                &providers,
                &key_envs,
                apply,
            )?;
            println!(
                "{}",
                nextjson::to_string_pretty(&report).map_err(anyhow::Error::msg)?
            );
        }
        Command::Provider {
            command: ProviderCommand::Rename { from, to },
        } => {
            config::rename_provider(&dir, &config::snapshot(&dir)?, &from, &to)?;
        }
        Command::Provider {
            command: ProviderCommand::Fallback { id, targets },
        } => {
            let previous = config::snapshot(&dir)?;
            let mut cfg = config::Config::parse(&previous)?;
            let index = cfg
                .providers
                .iter()
                .position(|p| p.id == id)
                .context("unknown provider")?;
            if !targets.is_empty() {
                let resolved = cfg.resolved_groups();
                let known: std::collections::HashSet<_> =
                    cfg.agent_model_ids(&resolved).into_iter().collect();
                let mut fallback = Vec::new();
                if targets != ["none"] {
                    for target in targets {
                        let target = target.trim();
                        ensure!(
                            known.contains(target)
                                || resolved.group_reference(&cfg, target).is_some()
                                || cfg.renamed_model(target).is_some()
                                || cfg.auto_stand_in(target).is_some(),
                            "unknown fallback model or group {target:?}"
                        );
                        if !fallback.iter().any(|id| id == target) {
                            fallback.push(target.to_owned());
                        }
                    }
                }
                cfg.providers[index].fallback = fallback;
                config::save(&dir, &previous, &cfg.to_json()?)?;
            }
            println!(
                "{}",
                nextjson::to_string(&cfg.providers[index].fallback).map_err(anyhow::Error::msg)?
            );
        }
        Command::Provider {
            command: ProviderCommand::Set { id, setting },
        } => {
            let previous = config::snapshot(&dir)?;
            let mut cfg = config::Config::parse(&previous)?;
            let provider = cfg
                .providers
                .iter_mut()
                .find(|p| p.id == id)
                .context("unknown provider")?;
            provider.no_stand_in = !setting;
            config::save(&dir, &previous, &cfg.to_json()?)?;
        }
        Command::Group { command } => {
            let previous = config::snapshot(&dir)?;
            let mut cfg = config::Config::parse(&previous)?;
            match command.unwrap_or(GroupCommand::List) {
                GroupCommand::ImportMagpie {
                    file,
                    groups,
                    apply,
                } => {
                    let passphrase = backup_passphrase()?;
                    let report = config::import_magpie_groups(
                        &dir,
                        &previous,
                        &file,
                        &passphrase,
                        &groups,
                        apply,
                    )?;
                    println!(
                        "{}",
                        nextjson::to_string_pretty(&report).map_err(anyhow::Error::msg)?
                    );
                }
                GroupCommand::Share { id } => println!("{}", cfg.share_group(&id)?),
                GroupCommand::Import { file, replace } => {
                    use std::io::Read;
                    let reader: Box<dyn Read> = match file {
                        Some(path) if path.as_os_str() != "-" => {
                            Box::new(std::fs::File::open(path)?)
                        }
                        _ => Box::new(std::io::stdin().lock()),
                    };
                    let mut text = String::new();
                    reader
                        .take(config::MAX_CONFIG as u64 + 1)
                        .read_to_string(&mut text)?;
                    let (modified, report) = cfg.import_group(&text, replace)?;
                    config::save(&dir, &previous, &modified.to_json()?)?;
                    println!(
                        "{}",
                        nextjson::to_string(&report).map_err(anyhow::Error::msg)?
                    );
                }
                GroupCommand::Show { id } => groups::show(&cfg, &id)?,
                GroupCommand::ByName(args) => {
                    ensure!(args.len() == 1, "expected group show <id>");
                    groups::show(&cfg, &args[0])?;
                }
                GroupCommand::Member { id, member, mode } => {
                    let name = cfg.group_name(&id);
                    let resolved = cfg.resolved_groups();
                    let effective = resolved.members.get(&name).context("unknown route group")?;
                    ensure!(
                        resolved.display_members(&cfg, &name).contains(&member),
                        "unknown member {member} in group {name}"
                    );
                    cfg.groups
                        .entry(name.clone())
                        .or_insert_with(|| effective.clone());
                    let off = cfg.group_off.entry(name).or_default();
                    if mode == "off" {
                        off.insert(member);
                    } else {
                        off.remove(&member);
                    }
                    config::save(&dir, &previous, &cfg.to_json()?)?;
                }
                GroupCommand::Switch { id, mode } => {
                    let name = cfg.group_name(&id);
                    ensure!(
                        cfg.groups.contains_key(&name),
                        "only saved route groups can be switched on or off: {id}"
                    );
                    if mode == "off" {
                        cfg.disabled_groups.insert(name);
                    } else {
                        cfg.disabled_groups.remove(&name);
                    }
                    config::save(&dir, &previous, &cfg.to_json()?)?;
                }
                GroupCommand::Copy { from, name } => {
                    let (updated, id) = cfg.copied_group(&from, &name.join(" "))?;
                    config::save(&dir, &previous, &updated.to_json()?)?;
                    println!("{} (group/{id})", updated.group_labels[&id]);
                }
                GroupCommand::Pick { id, member } => {
                    let id = cfg.group_name(&id);
                    cfg.pick_member(&id, &member)?;
                    config::save(&dir, &previous, &cfg.to_json()?)?;
                }
                GroupCommand::Rename { from, to } => {
                    let updated = cfg.renamed_group(&from, &to)?;
                    config::save(&dir, &previous, &updated.to_json()?)?;
                }
                GroupCommand::List => {
                    let groups = cfg.resolved_groups();
                    for name in groups.ordered_names(&cfg.group_display_order) {
                        let members = groups.display_members(&cfg, name);
                        let legacy = format!("group/{name}");
                        let id = groups.public_refs.get(&legacy).unwrap_or(&legacy);
                        let state = if cfg.disabled_groups.contains(name) {
                            "\tstate=off"
                        } else {
                            ""
                        };
                        let off = cfg
                            .group_off
                            .get(name)
                            .map(|off| {
                                format!(
                                    "\toff={}",
                                    members
                                        .iter()
                                        .filter(|m| off.contains(*m))
                                        .cloned()
                                        .collect::<Vec<_>>()
                                        .join(",")
                                )
                            })
                            .unwrap_or_default();
                        let strategy = if let Some(pick) = cfg.group_picks.get(name) {
                            let selected = members
                                .iter()
                                .find(|m| *m == pick)
                                .or(members.first())
                                .map_or("", String::as_str);
                            println!(
                                "{id}\tmanual\t{selected}\t{}{off}{state}",
                                members.join(" ")
                            );
                            continue;
                        } else if let Some(weights) = cfg.group_weights.get(name) {
                            let weights = members
                                .iter()
                                .map(|m| format!("{m}={}", weights.get(m).unwrap_or(&1)))
                                .collect::<Vec<_>>();
                            println!("{id}\tweight\t{}{off}{state}", weights.join(" "));
                            continue;
                        } else if cfg.rotating_groups.contains(name) {
                            "rotate"
                        } else {
                            "order"
                        };
                        println!("{id}\t{strategy}\t{}{off}{state}", members.join(" "));
                    }
                }
                GroupCommand::Order { ids } => {
                    let groups = cfg.resolved_groups();
                    let mut seen = std::collections::BTreeSet::new();
                    let mut order = Vec::new();
                    for id in ids {
                        let name = cfg.group_name(&id);
                        ensure!(groups.members.contains_key(&name), "unknown group {id}");
                        ensure!(seen.insert(name.clone()), "group {id} is listed twice");
                        order.push(name);
                    }
                    order.extend(
                        groups
                            .ordered_names(&cfg.group_display_order)
                            .into_iter()
                            .filter(|name| !seen.contains(*name))
                            .cloned(),
                    );
                    cfg.group_display_order = order;
                    config::save(&dir, &previous, &cfg.to_json()?)?;
                }
                GroupCommand::Auto { mode } => {
                    cfg.no_auto_groups = mode == "off";
                    config::save(&dir, &previous, &cfg.to_json()?)?;
                }
                GroupCommand::Place { id, members } => {
                    let id = cfg.group_name(&id);
                    ensure!(
                        cfg.group_match.contains_key(&id),
                        "placement requires a pattern group"
                    );
                    cfg.group_placed.remove(&id);
                    let current = cfg.expand_patterns(&id, &cfg.groups[&id]);
                    let mut seen = std::collections::BTreeSet::new();
                    for member in &members {
                        ensure!(
                            current.contains(member) && seen.insert(member),
                            "unknown or repeated member {member}"
                        );
                    }
                    if !members.is_empty() {
                        cfg.group_placed.insert(id, members);
                    }
                    config::save(&dir, &previous, &cfg.to_json()?)?;
                }
                GroupCommand::Restore { id } => {
                    let id = cfg.group_name(&id);
                    ensure!(cfg.hidden_auto_groups.remove(&id), "group is not hidden");
                    config::save(&dir, &previous, &cfg.to_json()?)?;
                }
                GroupCommand::Set {
                    id,
                    members,
                    note,
                    listed,
                    rotate,
                    weighted,
                    manual,
                    pick,
                    weight,
                } => {
                    let id = cfg.group_name(&id);
                    let (rules, named): (Vec<_>, Vec<_>) = members
                        .into_iter()
                        .partition(|m| config::MemberPattern::is_pattern(m));
                    cfg.set_member_patterns(&id, rules)?;
                    cfg.reorder_named_members(&id, &named);
                    let members = cfg.expand_patterns(&id, &named);
                    let previous_pick = cfg
                        .group_picks
                        .remove(&id)
                        .or_else(|| cfg.group_last_picks.remove(&id))
                        .filter(|m| members.contains(m));
                    if let Some(pick) = pick {
                        ensure!(
                            members.contains(&pick),
                            "manual selection must name a member of {id}"
                        );
                        cfg.group_picks.insert(id.clone(), pick);
                    } else if manual {
                        cfg.group_picks.insert(
                            id.clone(),
                            previous_pick
                                .or_else(|| members.first().cloned())
                                .unwrap_or_default(),
                        );
                    } else if let Some(pick) = previous_pick {
                        cfg.group_last_picks.insert(id.clone(), pick);
                    }
                    let mut previous_weights = cfg
                        .group_weights
                        .remove(&id)
                        .or_else(|| cfg.group_last_weights.remove(&id))
                        .unwrap_or_default();
                    previous_weights.retain(|member, _| members.contains(member));
                    if !weight.is_empty() {
                        let count = weight.len();
                        let weights = weight
                            .into_iter()
                            .collect::<std::collections::BTreeMap<_, _>>();
                        ensure!(weights.len() == count, "duplicate member weight");
                        cfg.group_weights.insert(id.clone(), weights);
                    } else if weighted {
                        cfg.group_weights.insert(id.clone(), previous_weights);
                    } else if !previous_weights.is_empty() {
                        cfg.group_last_weights.insert(id.clone(), previous_weights);
                    }
                    if rotate {
                        cfg.rotating_groups.insert(id.clone());
                    } else {
                        cfg.rotating_groups.remove(&id);
                    }
                    cfg.hidden_auto_groups.remove(&id);
                    match listed.as_deref() {
                        Some("no") => {
                            cfg.group_unlists.insert(id.clone());
                        }
                        Some("yes") => {
                            cfg.group_unlists.remove(&id);
                        }
                        _ => (),
                    }
                    if let Some(note) = note {
                        cfg.group_notes.insert(id.clone(), note);
                    }
                    cfg.groups.insert(id, named);
                    cfg.former_groups
                        .retain(|old, _| !cfg.groups.contains_key(old));
                    config::save(&dir, &previous, &cfg.to_json()?)?;
                }
                GroupCommand::Remove { id } => {
                    let id = cfg.group_name(&id);
                    ensure!(
                        cfg.resolved_groups().members.contains_key(&id),
                        "unknown route group"
                    );
                    cfg.groups.remove(&id);
                    cfg.group_match.remove(&id);
                    cfg.group_placed.remove(&id);
                    cfg.group_labels.remove(&id);
                    cfg.group_notes.remove(&id);
                    cfg.group_off.remove(&id);
                    cfg.group_unlists.remove(&id);
                    cfg.group_display_order.retain(|name| name != &id);
                    cfg.disabled_groups.remove(&id);
                    cfg.former_groups.retain(|_, target| target != &id);
                    if id.starts_with("auto-") {
                        cfg.hidden_auto_groups.insert(id.clone());
                    }
                    cfg.rotating_groups.remove(&id);
                    cfg.group_weights.remove(&id);
                    cfg.group_last_weights.remove(&id);
                    cfg.group_picks.remove(&id);
                    cfg.group_last_picks.remove(&id);
                    config::save(&dir, &previous, &cfg.to_json()?)?;
                }
            }
        }
        Command::Agent {
            client,
            model,
            profile,
            args,
        } => client.launch(&dir, &model, profile.as_deref(), args)?,
        Command::Models { provider, sync } => {
            let previous = config::snapshot(&dir)?;
            let mut cfg = config::Config::parse(&previous)?;
            let p = cfg
                .providers
                .iter_mut()
                .find(|p| p.id == provider)
                .context("provider not found")?;
            let models = if let Some(site) = p.web_site {
                vec![site.model().to_owned()]
            } else {
                if crabforge_core::catalog::fixed_models(&p.endpoints).is_some() {
                    eprintln!(
                        "Using built-in model IDs. Availability depends on the upstream account."
                    );
                }
                if sync {
                    let catalog =
                        crabforge_core::catalog::discover_catalog(&p.endpoints, &p.key_env).await?;
                    p.models.clone_from(&catalog.models);
                    p.free_models = catalog.free_models;
                    p.model_protocols = catalog.model_protocols;
                    if let Some(mode) = catalog.learned_auth {
                        p.endpoints.auth_header = mode;
                    }
                    config::save(&dir, &previous, &cfg.to_json()?)?;
                    catalog.models
                } else {
                    crabforge_core::catalog::discover_provider(&dir, p).await?
                }
            };
            for model in models {
                println!("{model}");
            }
        }
        Command::Agents => {
            let base = config::load(&dir)?.listen;
            for (id, name) in crabforge_core::agents::CLIENTS {
                println!("{name}\thttp://{base}/agent/{id}");
            }
        }
        Command::OmpProfiles => agent::Agent::omp_profiles(&dir)?,
        Command::OmpRoles {
            profile,
            role,
            model,
            ..
        } => {
            use crabforge_core::agents::{omp_profiles, omp_settings::roles};
            let cfg = config::load(&dir)?;
            let home = omp_profiles::settings_directory(&dir, profile.as_deref())?;
            if let Some(name) = role {
                roles::save_role(&home, &cfg, &name, model.as_deref(), None)?;
            }
            println!(
                "{}",
                nextjson::to_string_pretty(&roles::read_roles(&home, &cfg)?)
                    .map_err(anyhow::Error::msg)?
            );
        }
        Command::OmpSync { profile } => println!(
            "{}",
            crabforge_core::agents::omp_catalog::sync(&dir, profile.as_deref())?
        ),
        Command::OmpProtocol { profile, protocol } => {
            use crabforge_core::agents::omp_catalog;
            if let Some(choice) = protocol {
                omp_catalog::set_protocol(&dir, profile.as_deref(), &choice, None)?;
            }
            println!("{}", omp_catalog::read_protocol(&dir, profile.as_deref())?);
        }
        Command::OmpThinking {
            profile,
            session,
            role,
            level,
        } => {
            use crabforge_core::agents::{omp_profiles, omp_settings};
            let home = omp_profiles::settings_directory(&dir, profile.as_deref())?;
            let changes = omp_settings::ThinkingChanges {
                session,
                roles: role.zip(level).into_iter().collect(),
            };
            println!(
                "{}",
                nextjson::to_string_pretty(&omp_settings::save_thinking(&home, None, &changes)?)
                    .map_err(anyhow::Error::msg)?
            );
        }
        Command::OmpAdvisor {
            profile,
            review,
            wait,
            wake,
        } => {
            use crabforge_core::agents::{omp_profiles, omp_settings};
            let home = omp_profiles::settings_directory(&dir, profile.as_deref())?;
            let changes = omp_settings::AdvisorChanges { review, wait, wake };
            println!(
                "{}",
                nextjson::to_string_pretty(&omp_settings::save_advisor(&home, None, &changes)?)
                    .map_err(anyhow::Error::msg)?
            );
        }
        Command::Catalog => println!("{}", crabforge_core::catalog::presets()?),
        Command::ProjectInstructions {
            project,
            agent,
            from,
            clear,
            restore,
            revision,
        } => {
            use crabforge_core::project_instructions::{edit, read, read_markdown};
            let mut view = read(&dir, &project, &agent)?;
            if from.is_some() || clear || restore {
                let text = from
                    .as_deref()
                    .map(read_markdown)
                    .transpose()?
                    .unwrap_or_default();
                edit(
                    &dir,
                    &project,
                    &agent,
                    revision.as_deref().unwrap_or(&view.revision),
                    (!restore).then_some(text.as_str()),
                )?;
                view = read(&dir, &project, &agent)?;
            }
            println!(
                "{}",
                nextjson::to_string(&view).map_err(anyhow::Error::msg)?
            );
        }
        Command::Memory { command } => memory::run(&dir, command).await?,
        Command::Rules { command } => rules::run(&dir, command)?,
        Command::Skills { command } => skills::run(&dir, command)?,
        Command::Environment { command } => environment::run(&dir, command)?,
        Command::Hooks { project, command } => hooks::run(&dir, project.as_deref(), command)?,
        Command::ClaudePlugins { project, command } => {
            claude_plugins::run(&dir, project.as_deref(), command)?
        }
        Command::PluginBundles { command } => plugin_bundles::run(&dir, command)?,
        Command::Subagents { project, command } => agent_files::run(
            &dir,
            crabforge_core::agent_files::Kind::Subagent,
            project.as_deref(),
            command,
        )?,
        Command::Commands { command } => agent_files::run(
            &dir,
            crabforge_core::agent_files::Kind::Command,
            None,
            command,
        )?,
        Command::ProjectMcp { command } => project_mcp::run(&dir, command)?,
        Command::Sessions { agent, limit } => println!(
            "{}",
            nextjson::to_string_pretty(&crabforge_core::sessions::list(
                &dir,
                agent.map(Into::into),
                usize::from(limit),
            )?)
            .map_err(anyhow::Error::msg)?
        ),
        Command::Mcp { command } => memory::serve(&dir, command)?,
        Command::Config { file: None } => println!("{}", config::snapshot(&dir)?),
        Command::Config { file: Some(path) } => {
            use std::io::Read;
            let previous = config::snapshot(&dir)?;
            let mut text = String::new();
            std::fs::File::open(path)?
                .take(config::MAX_CONFIG as u64 + 1)
                .read_to_string(&mut text)?;
            ensure!(
                text.len() <= config::MAX_CONFIG,
                "configuration exceeds 1 MiB"
            );
            config::save(&dir, &previous, &text)?;
        }
        Command::Key => println!("{}", config::token(&dir)?),
        Command::PeerId => println!("{}", peer::identity(&dir)?.public().to_peer_id()),
        Command::Serve => {
            let mut service = Box::pin(Service::start(&dir)).await?;
            eprintln!("HTTP http://{}/v1", service.addr);
            if let Some(peer) = &service.peer {
                eprintln!("P2P {}", peer.address);
            }
            tokio::select! { r=service.wait()=>r?, r=tokio::signal::ctrl_c()=>r? }
        }
        Command::Tunnel {
            remote,
            relay_fallback,
            listen,
        } => {
            let (addr, mut task) = Box::pin(peer::tunnel_with_relay(
                peer::identity(&dir)?,
                &remote,
                relay_fallback.as_deref(),
                listen,
            ))
            .await?;
            eprintln!("Tunnel http://{addr}/v1");
            tokio::select! { r=&mut task=>r.context("tunnel task")??, r=tokio::signal::ctrl_c()=>{r?;task.abort();} }
        }
        Command::Web { site, login: false } => config::open_web(&site)?,
        Command::Web { site, login: true } => {
            let site = if site == "deepseek" {
                Site::DeepSeek
            } else {
                Site::ChatGLM
            };
            let browser = Browser::open(&dir, site).await?;
            let visit = browser.visit().await;
            if visit.is_ok() {
                eprintln!("Complete sign-in in the browser, then press Enter here.");
                let mut line = String::new();
                let _ = std::io::stdin().read_line(&mut line);
            }
            browser.close().await;
            visit?;
        }
    }
    Ok(())
}
