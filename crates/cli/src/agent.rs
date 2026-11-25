use anyhow::{Context, Result, ensure};
use clap::ValueEnum;
use crabforge_core::config;
mod crush;
mod kimi;
mod nanobot;
mod omp;
mod vibe;
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Agent {
    Codex,
    Copilot,
    Crush,
    Claude,
    Deveco,
    Gemini,
    Kimi,
    #[value(alias = "mimo")]
    Mimocode,
    Nanobot,
    Mu,
    Omo,
    Omp,
    Opencode,
    #[value(name = "prime-agent")]
    Prime,
    Pi,
    Goose,
    Qwen,
    Vibe,
}

impl Agent {
    #[cfg(not(windows))]
    fn command(self, binary: &str) -> Result<Command> {
        Ok(Command::new(binary))
    }

    pub(crate) fn omp_profiles(dir: &Path) -> Result<()> {
        omp::profiles::list(dir, &Self::Omp.command("omp")?)
    }
    #[cfg(windows)]
    fn command(self, binary: &str) -> Result<Command> {
        // npm's Windows launchers are batch files. Use the package's bin entry
        // directly so arguments are always passed to the client without a shell.
        let package = match self {
            Self::Codex => "@openai/codex",
            Self::Copilot => "@github/copilot",
            Self::Claude => "@anthropic-ai/claude-code",
            Self::Deveco => "@deveco/deveco-code",
            Self::Mimocode => "@mimo-ai/cli",
            Self::Gemini => "@google/gemini-cli",
            Self::Kimi => "@moonshot-ai/kimi-code",
            Self::Pi => "@earendil-works/pi-coding-agent",
            Self::Mu => "mu-agent",
            Self::Omo => "omo-ai",
            Self::Omp => "@oh-my-pi/pi-coding-agent",
            Self::Opencode => "@opencode/cli",
            Self::Goose | Self::Vibe | Self::Crush | Self::Nanobot | Self::Prime => "",
            Self::Qwen => "@qwen-code/qwen-code",
        };
        let paths = std::env::var_os("PATH").unwrap_or_default();
        for dir in std::env::split_paths(&paths).filter(|dir| dir.is_absolute()) {
            let exe = dir.join(format!("{binary}.exe"));
            if exe.is_file() {
                return Ok(Command::new(exe));
            }
            if package.is_empty() || !dir.join(format!("{binary}.cmd")).is_file() {
                continue;
            }
            let modules = if dir.ends_with(".bin") {
                dir.parent().unwrap().to_owned()
            } else {
                dir.join("node_modules")
            };
            let package_dir = modules.join(package);
            let manifest = package_dir.join("package.json");
            if !manifest.is_file() {
                continue;
            }
            let meta: serde_json::Value = serde_json::from_slice(&std::fs::read(manifest)?)?;
            let entry = meta["bin"][binary]
                .as_str()
                .or_else(|| meta["bin"].as_str())
                .context("missing npm client entry point")?;
            let script = package_dir.join(entry);
            ensure!(
                script.is_file(),
                "missing client entry point {}",
                script.display()
            );
            if script.extension().is_some_and(|e| e == "exe") {
                return Ok(Command::new(script));
            }
            let mut cmd = Command::new(if matches!(self, Self::Omp) {
                "bun"
            } else {
                "node"
            });
            cmd.arg(script);
            return Ok(cmd);
        }
        anyhow::bail!("install {binary} on PATH")
    }

    pub(crate) fn launch(
        self,
        dir: &Path,
        model: &str,
        profile: Option<&str>,
        args: Vec<OsString>,
    ) -> Result<()> {
        ensure!(
            profile.is_none() || matches!(self, Self::Omp),
            "--profile applies to Oh My Pi"
        );
        let cfg = config::load(dir)?;
        ensure!(
            cfg.listen.port() != 0,
            "configure a fixed gateway port to launch an agent from the CLI"
        );
        let groups = cfg.resolved_groups();
        let mut ids = model_ids(&cfg, &groups);
        let canonical = cfg.canonical_auto_ref(model);
        let group = canonical.as_deref().unwrap_or(model);
        let group = groups.aliases.get(group).map_or(group, String::as_str);
        if (model.starts_with("group/") || canonical.is_some())
            && group
                .strip_prefix("group/")
                .is_some_and(|id| cfg.disabled_groups.contains(id))
        {
            anyhow::bail!("route group is switched off");
        }
        let selected = if ids.iter().any(|id| id == model) {
            model.to_owned()
        } else if let Some(public) = groups.public_refs.get(group) {
            public.clone()
        } else if ids.iter().any(|id| id == group) {
            group.to_owned()
        } else {
            cfg.auto_stand_in(model)
                .context("model is missing from the gateway configuration")?
        };
        let model = selected.as_str();
        let unlisted = cfg.unlisted_model_ids(&groups);
        ids.retain(|id| id == model || !unlisted.contains(id));
        let key = config::token(dir)?;
        let binary = match self {
            Self::Codex => "codex",
            Self::Copilot => "copilot",
            Self::Crush => "crush",
            Self::Claude => "claude",
            Self::Deveco => "deveco",
            Self::Mimocode => "mimo",
            Self::Gemini => "gemini",
            Self::Kimi => "kimi",
            Self::Nanobot => "nanobot",
            Self::Mu => "mu",
            Self::Omo => "omo",
            Self::Omp => "omp",
            Self::Opencode => "opencode",
            Self::Prime => "prime-agent",
            Self::Pi => "pi",
            Self::Goose => "goose",
            Self::Qwen => "qwen",
            Self::Vibe => "vibe",
        };
        let agent_id = if matches!(self, Self::Mimocode) {
            "mimocode"
        } else {
            binary
        };
        let root = format!("http://{}/agent/{agent_id}", cfg.listen);
        let mut command = self.command(binary)?;
        let mut args = args.into_iter();
        let mut session_guard = None;
        let nanobot_config;
        let opencode_config;
        let mut crush_server = None;
        match self {
            Self::Opencode | Self::Deveco | Self::Mimocode => {
                let models = ids
                    .into_iter()
                    .map(|id| (id.clone(), serde_json::json!({"name":id})))
                    .collect::<serde_json::Map<_, _>>();
                let selected = format!("crabforge/{model}");
                let (variable, profile) = if matches!(self, Self::Opencode) {
                    command.arg("--standalone");
                    (
                        "OPENCODE_CONFIG",
                        serde_json::json!({
                        "model":selected, "agents":{"title":{"model":selected}},
                        "providers":{"crabforge":{
                            "name":"Crabforge", "env":["CRABFORGE_GATEWAY_KEY"],
                            "package":"@opencode/ai/providers/openai-compatible",
                            "settings":{"baseURL":format!("{root}/v1")}, "models":models
                        }}
                        }),
                    )
                } else {
                    let variable = if matches!(self, Self::Mimocode) {
                        "MIMOCODE_CONFIG"
                    } else {
                        "DEVECO_CONFIG"
                    };
                    (
                        variable,
                        serde_json::json!({
                            "model":selected, "small_model":selected,
                            "provider":{"crabforge":{
                                "npm":"@ai-sdk/openai-compatible", "name":"Crabforge",
                                "options":{"baseURL":format!("{root}/v1"), "apiKey":"{env:CRABFORGE_GATEWAY_KEY}"}, "models":models
                            }}
                        }),
                    )
                };
                let home = std::path::absolute(dir.join("agents").join(agent_id))?;
                fs::create_dir_all(&home)?;
                opencode_config = tempfile::Builder::new()
                    .prefix("gateway-")
                    .suffix(".json")
                    .tempfile_in(home)?;
                serde_json::to_writer(opencode_config.as_file(), &profile)?;
                command
                    .env(variable, opencode_config.path())
                    .env("CRABFORGE_GATEWAY_KEY", key);
            }
            Self::Crush => {
                crush::configure(&mut command, dir, &root, model, ids)?;
                command.env("CRABFORGE_GATEWAY_KEY", key);
                if args
                    .as_slice()
                    .first()
                    .is_some_and(|a| a == "run" || a == "r")
                {
                    crush_server = Some(crush::RunServer::start(&mut command)?);
                    command.arg(args.next().unwrap()).args([
                        "--model",
                        &format!("crabforge/{model}"),
                        "--small-model",
                        &format!("crabforge/{model}"),
                    ]);
                }
            }
            Self::Codex => {
                command.env("CRABFORGE_GATEWAY_KEY", key).args([
                    "-c",
                    "model_provider=\"crabforge\"",
                    "-c",
                    "model_providers.crabforge.name=\"Crabforge\"",
                    "-c",
                    "model_providers.crabforge.wire_api=\"responses\"",
                    "-c",
                    "model_providers.crabforge.env_key=\"CRABFORGE_GATEWAY_KEY\"",
                    "-c",
                    &format!("model_providers.crabforge.base_url=\"{root}/v1\""),
                ]);
            }
            Self::Copilot => {
                command
                    .env(
                        "COPILOT_HOME",
                        std::path::absolute(dir.join("agents/copilot"))?,
                    )
                    .env("COPILOT_PROVIDER_TYPE", "openai")
                    .env("COPILOT_PROVIDER_WIRE_API", "completions")
                    .env("COPILOT_PROVIDER_TRANSPORT", "http")
                    .env("COPILOT_PROVIDER_BASE_URL", format!("{root}/v1"))
                    .env("COPILOT_PROVIDER_API_KEY", key)
                    .env("COPILOT_PROVIDER_WIRE_MODEL", model)
                    .env_remove("COPILOT_PROVIDER_MODEL_ID")
                    .env_remove("COPILOT_PROVIDERS_CONFIG")
                    .env_remove("COPILOT_PROVIDER_API_KEY_COMMAND")
                    .env_remove("COPILOT_PROVIDER_BEARER_TOKEN")
                    .env_remove("COPILOT_PROVIDER_HEADERS");
            }
            Self::Claude => {
                command
                    .env("ANTHROPIC_BASE_URL", root)
                    .env("ANTHROPIC_AUTH_TOKEN", key)
                    .env_remove("ANTHROPIC_API_KEY");
            }
            Self::Gemini => {
                command
                    .env("GEMINI_CLI_HOME", crabforge_core::agents::gemini_home(dir)?)
                    .env("GOOGLE_GEMINI_BASE_URL", root)
                    .env("GEMINI_API_KEY", key)
                    .env_remove("GOOGLE_GENAI_USE_GCA")
                    .env_remove("GOOGLE_API_KEY")
                    .env_remove("GOOGLE_GENAI_USE_VERTEXAI");
            }
            Self::Pi | Self::Mu | Self::Omo | Self::Prime => {
                let profile_var = match self {
                    Self::Mu => "MU_AGENT_DIR",
                    Self::Omo => "OMO_CODING_AGENT_DIR",
                    Self::Prime => "PRIME_AGENT_CODING_AGENT_DIR",
                    _ => "PI_CODING_AGENT_DIR",
                };
                command
                    .env(profile_var, pi_profile(dir, binary, ids, &root)?)
                    .env("CRABFORGE_GATEWAY_KEY", key)
                    .args(["--provider", "crabforge"]);
            }
            Self::Omp => {
                let protocols = crabforge_core::gateway::native_protocols(&cfg, &groups)?;
                let native = omp::profiles::native_profile(args.as_slice())?;
                let home = omp::profiles::agent_directory(dir, &mut command, native.or(profile))?;
                omp::profile(
                    &home,
                    ids,
                    &root,
                    &protocols,
                    omp::supports_session_header(&command),
                )?;
                command
                    .env("CRABFORGE_GATEWAY_KEY", key)
                    .args(["--provider", "crabforge"]);
            }
            Self::Goose => {
                command
                    .env("GOOSE_PATH_ROOT", goose_profile(dir, ids, &root)?)
                    .env("GOOSE_PROVIDER", "crabforge")
                    .env("GOOSE_MODEL", model)
                    .env("CRABFORGE_GATEWAY_KEY", key);
            }
            Self::Qwen => {
                command
                    .env("QWEN_HOME", std::path::absolute(dir.join("agents/qwen"))?)
                    .env("OPENAI_API_KEY", key)
                    .args([
                        "--auth-type",
                        "openai",
                        "--openai-base-url",
                        &format!("{root}/v1"),
                    ]);
            }
            Self::Kimi => {
                let home = std::path::absolute(dir.join("agents/kimi"))?;
                kimi::profile(&home, ids, &root)?;
                command
                    .env("KIMI_CODE_HOME", home)
                    .env("CRABFORGE_GATEWAY_KEY", key)
                    .args(["--model", &format!("crabforge/{model}")]);
            }
            Self::Nanobot => {
                let home = std::path::absolute(dir.join("agents/nanobot"))?;
                nanobot_config = nanobot::profile(&home, &root, model)?;
                command
                    .env("CRABFORGE_GATEWAY_KEY", key)
                    .arg("--home")
                    .arg(home)
                    .args(["agent", "--config"])
                    .arg(nanobot_config.path());
            }
            Self::Vibe => {
                let home = vibe::profile(dir, ids, &root)?;
                session_guard = Some(vibe::session_directory(&mut command, &home)?);
                command
                    .env("VIBE_HOME", home)
                    .env("CRABFORGE_GATEWAY_KEY", key)
                    .args(["--model", &format!("crabforge/{model}")]);
            }
        }
        if !matches!(
            self,
            Self::Goose
                | Self::Vibe
                | Self::Crush
                | Self::Kimi
                | Self::Nanobot
                | Self::Opencode
                | Self::Mimocode
                | Self::Deveco
        ) {
            command.args(["--model", model]);
        }
        let status = command
            .args(args)
            .status()
            .with_context(|| format!("start {binary} from PATH"))?;
        drop(session_guard);
        if let Some(server) = crush_server {
            server.finish()?;
        }
        ensure!(status.success(), "{binary} exited with {status}");
        Ok(())
    }
}

/// Independent sessions sharing an agent profile see the same complete catalog.
fn model_ids(cfg: &config::Config, groups: &config::ResolvedGroups) -> Vec<String> {
    let mut ids = BTreeSet::new();
    for p in &cfg.providers {
        for m in &p.models {
            ids.insert(m.clone());
            ids.insert(format!("{}/{m}", p.id));
        }
    }
    let mut group_ids = groups.listed_refs(cfg);
    if cfg.group_display_order.is_empty() {
        ids.extend(group_ids);
        return ids.into_iter().collect();
    }
    for id in &group_ids {
        ids.remove(id);
    }
    group_ids.extend(ids);
    group_ids
}

fn write_catalog(path: &Path, catalog: &serde_json::Value) -> Result<()> {
    let folder = path.parent().context("missing catalog directory")?;
    fs::create_dir_all(folder)?;
    let mut file = tempfile::NamedTempFile::new_in(folder)?;
    serde_json::to_writer_pretty(&mut file, catalog)?;
    file.flush()?;
    file.persist(path)?;
    Ok(())
}

fn edit_toml(
    home: &Path,
    edit: impl FnOnce(&mut toml_edit::DocumentMut) -> Result<()>,
) -> Result<()> {
    fs::create_dir_all(home)?;
    let lock = fs::File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(home.join("crabforge.lock"))?;
    lock.lock()?;
    let path = home.join("config.toml");
    let original = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let mut doc = original
        .parse()
        .with_context(|| format!("invalid {}", path.display()))?;
    edit(&mut doc)?;
    let updated = doc.to_string();
    if updated != original {
        let mut file = tempfile::NamedTempFile::new_in(home)?;
        file.write_all(updated.as_bytes())?;
        file.flush()?;
        file.persist(path)?;
    }
    Ok(())
}

fn pi_profile(dir: &Path, binary: &str, ids: Vec<String>, root: &str) -> Result<PathBuf> {
    let home = std::path::absolute(dir.join("agents").join(binary))?;
    let models: Vec<_> = ids
        .into_iter()
        .map(|id| serde_json::json!({"id":id}))
        .collect();
    let key_ref = if binary == "prime-agent" {
        "CRABFORGE_GATEWAY_KEY"
    } else {
        "${CRABFORGE_GATEWAY_KEY}"
    };
    edit_json_object(&home.join("models.json"), "providers", |providers| {
        let mut provider = serde_json::json!({"baseUrl":format!("{root}/v1"),"api":"openai-completions","apiKey":key_ref,"models":models});
        if binary == "pi" {
            let mut compat = providers
                .get("crabforge")
                .and_then(|p| p["compat"].as_object())
                .cloned()
                .unwrap_or_default();
            compat
                .entry("sendSessionAffinityHeaders")
                .or_insert(true.into());
            provider["compat"] = compat.into();
        }
        providers.insert("crabforge".into(), provider);
    })?;
    Ok(home)
}

fn edit_json_object(
    path: &Path,
    field: &str,
    edit: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>),
) -> Result<()> {
    use serde_json::{Map, Value, json};
    let original = match fs::read(path) {
        Ok(bytes) => serde_json::from_slice::<Map<String, Value>>(&bytes)
            .with_context(|| format!("invalid {}", path.display()))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Map::new(),
        Err(e) => return Err(e.into()),
    };
    let mut doc = original.clone();
    let entries = doc
        .entry(field)
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .with_context(|| format!("invalid {field} in {}", path.display()))?;
    edit(entries);
    if doc != original {
        write_catalog(path, &Value::Object(doc))?;
    }
    Ok(())
}

fn goose_profile(dir: &Path, ids: Vec<String>, root: &str) -> Result<PathBuf> {
    let home = std::path::absolute(dir.join("agents/goose"))?;
    let models: Vec<_> = ids
        .into_iter()
        .map(|name| serde_json::json!({"name":name}))
        .collect();
    let catalog = serde_json::json!({"name":"crabforge", "engine":"openai", "display_name":"Crabforge", "base_url":format!("{root}/v1"), "api_key_env":"CRABFORGE_GATEWAY_KEY", "dynamic_models":false, "skip_canonical_filtering":true, "models":models});
    write_catalog(
        &home.join("config/custom_providers/crabforge.json"),
        &catalog,
    )?;
    Ok(home)
}
