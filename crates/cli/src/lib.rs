use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use crabforge_core::webchat::{Browser, Site};
use crabforge_core::{config, peer, service::Service};
use std::{net::SocketAddr, path::PathBuf};
mod agent;
mod groups;

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
    /// Manage named gateway route groups.
    #[command(visible_alias = "groups")]
    Group {
        #[command(subcommand)]
        command: Option<GroupCommand>,
    },
    /// Print the API endpoint presets as JSON.
    Catalog,
    /// List the agent-specific API roots accepted by this gateway.
    Agents,
    /// List Oh My Pi profiles, the active profile and each agent directory.
    OmpProfiles,
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
    Models { provider: String },
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

#[derive(Subcommand)]
enum GroupCommand {
    List,
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
    /// Create or update a group. Members are provider/model or group/name IDs in fallback order.
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

#[tokio::main(flavor = "current_thread")]
pub async fn run() -> Result<()> {
    let args = Args::parse();
    let dir = match args.data_dir {
        Some(p) => p,
        None => config::data_dir()?,
    };
    match args.command {
        Command::Group { command } => {
            let previous = config::snapshot(&dir)?;
            let mut cfg = config::Config::parse(&previous)?;
            match command.unwrap_or(GroupCommand::List) {
                GroupCommand::Show { id } => groups::show(&cfg, &id)?,
                GroupCommand::ByName(args) => {
                    ensure!(args.len() == 1, "expected group show <id>");
                    groups::show(&cfg, &args[0])?;
                }
                GroupCommand::Member { id, member, mode } => {
                    let name = cfg.group_name(&id);
                    let resolved = cfg.resolved_groups();
                    let effective = resolved.members.get(&name).context("unknown route group")?;
                    let members = cfg
                        .groups
                        .entry(name.clone())
                        .or_insert_with(|| effective.clone());
                    ensure!(
                        members.contains(&member),
                        "unknown member {member} in group {name}"
                    );
                    let off = cfg.group_off.entry(name).or_default();
                    if mode == "off" {
                        off.insert(member);
                    } else {
                        off.remove(&member);
                    }
                    config::save(&dir, &previous, &serde_json::to_string(&cfg)?)?;
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
                    config::save(&dir, &previous, &serde_json::to_string(&cfg)?)?;
                }
                GroupCommand::Copy { from, name } => {
                    let (updated, id) = cfg.copied_group(&from, &name.join(" "))?;
                    config::save(&dir, &previous, &serde_json::to_string(&updated)?)?;
                    println!("{} (group/{id})", updated.group_labels[&id]);
                }
                GroupCommand::Pick { id, member } => {
                    let id = cfg.group_name(&id);
                    cfg.pick_member(&id, &member)?;
                    config::save(&dir, &previous, &serde_json::to_string(&cfg)?)?;
                }
                GroupCommand::Rename { from, to } => {
                    let updated = cfg.renamed_group(&from, &to)?;
                    config::save(&dir, &previous, &serde_json::to_string(&updated)?)?;
                }
                GroupCommand::List => {
                    let groups = cfg.resolved_groups();
                    for name in groups.ordered_names(&cfg.group_display_order) {
                        let members = cfg.groups.get(name).unwrap_or(&groups.members[name]);
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
                            let selected =
                                members.iter().find(|m| *m == pick).unwrap_or(&members[0]);
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
                    config::save(&dir, &previous, &serde_json::to_string(&cfg)?)?;
                }
                GroupCommand::Auto { mode } => {
                    cfg.no_auto_groups = mode == "off";
                    config::save(&dir, &previous, &serde_json::to_string(&cfg)?)?;
                }
                GroupCommand::Restore { id } => {
                    let id = cfg.group_name(&id);
                    ensure!(cfg.hidden_auto_groups.remove(&id), "group is not hidden");
                    config::save(&dir, &previous, &serde_json::to_string(&cfg)?)?;
                }
                GroupCommand::Set {
                    id,
                    members,
                    note,
                    listed,
                    rotate,
                    manual,
                    pick,
                    weight,
                } => {
                    let id = cfg.group_name(&id);
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
                            previous_pick.unwrap_or_else(|| members[0].clone()),
                        );
                    } else if let Some(pick) = previous_pick {
                        cfg.group_last_picks.insert(id.clone(), pick);
                    }
                    cfg.group_weights.remove(&id);
                    if !weight.is_empty() {
                        let count = weight.len();
                        let weights = weight
                            .into_iter()
                            .collect::<std::collections::BTreeMap<_, _>>();
                        ensure!(weights.len() == count, "duplicate member weight");
                        cfg.group_weights.insert(id.clone(), weights);
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
                    cfg.groups.insert(id, members);
                    cfg.former_groups
                        .retain(|old, _| !cfg.groups.contains_key(old));
                    config::save(&dir, &previous, &serde_json::to_string(&cfg)?)?;
                }
                GroupCommand::Remove { id } => {
                    let id = cfg.group_name(&id);
                    ensure!(
                        cfg.resolved_groups().members.contains_key(&id),
                        "unknown route group"
                    );
                    cfg.groups.remove(&id);
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
                    cfg.group_picks.remove(&id);
                    cfg.group_last_picks.remove(&id);
                    config::save(&dir, &previous, &serde_json::to_string(&cfg)?)?;
                }
            }
        }
        Command::Agent {
            client,
            model,
            profile,
            args,
        } => client.launch(&dir, &model, profile.as_deref(), args)?,
        Command::Models { provider } => {
            let config = config::load(&dir)?;
            let p = config
                .providers
                .iter()
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
                crabforge_core::catalog::discover_provider(&dir, p).await?
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
        Command::Catalog => println!("{}", crabforge_core::catalog::presets()?),
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
            let mut service = Service::start(&dir).await?;
            eprintln!("HTTP http://{}/v1", service.addr);
            if let Some(peer) = &service.peer {
                eprintln!("P2P {}", peer.address);
            }
            tokio::select! { r=service.wait()=>r?, r=tokio::signal::ctrl_c()=>r? }
        }
        Command::Tunnel { remote, listen } => {
            let (addr, mut task) = peer::tunnel(peer::identity(&dir)?, &remote, listen).await?;
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
