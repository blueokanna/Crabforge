/// IDs used in the agent-specific gateway address, matching the reference clients.
pub const CLIENTS: &[(&str, &str)] = &[
    ("claude", "Claude Code"),
    ("codex", "Codex"),
    ("gemini", "Gemini CLI"),
    ("opencode", "OpenCode"),
    ("mimocode", "MiMo Code"),
    ("deveco", "DevEco Code"),
    ("pi", "Pi"),
    ("goose", "Goose"),
    ("cursor", "Cursor CLI"),
    ("copilot", "Copilot CLI"),
    ("vscode", "VS Code"),
    ("crush", "Crush"),
    ("dsh", "DeepSeek Harness"),
    ("commandcode", "Command Code"),
    ("fx", "fx"),
    ("omp", "omp"),
    ("omo", "OmO"),
    ("devin", "Devin"),
    ("hermes", "Hermes Agent"),
    ("morph", "Mister Morph"),
    ("kimi", "Kimi Code"),
    ("muse", "Muse Code"),
    ("empryo", "Empryo"),
    ("nanobot", "nanobot"),
    ("miniq", "miniQ"),
    ("paimon", "Paimon"),
    ("minimax-code", "MiniMax Code"),
    ("craft-agents", "Craft Agents"),
    ("tuanjie-cowork", "Tuanjie Cowork"),
    ("droid", "Droid"),
    ("grok", "Grok Build"),
    ("kigi", "Kigi"),
    ("zcode", "ZCode"),
    ("hanako", "OpenHanako"),
    ("atomcode", "AtomCode"),
    ("ocr", "Open Code Review"),
    ("snow", "Snow CLI"),
    ("qwenpaw", "QwenPaw"),
    ("alma", "Alma"),
    ("shelley", "Shelley"),
    ("claude-desktop", "Claude Desktop"),
    ("claude-science", "Claude Science"),
    ("agy", "Antigravity CLI"),
    ("dim", "Dim"),
    ("antigravity", "Antigravity"),
    ("openchamber", "OpenChamber"),
    ("stepcode", "StepCode"),
    ("mothx", "MothX"),
    ("penguin", "PenguinHarness"),
    ("aside", "Aside"),
    ("prime-agent", "Prime Agent"),
    ("mu", "mu"),
    ("cursor-local", "Cursor Private Inference"),
    ("zed", "Zed"),
    ("vscode-insiders", "VS Code Insiders"),
    ("vscodium", "VSCodium"),
    ("zoo-code", "Zoo Code"),
    ("copilot-jetbrains", "Copilot (JetBrains)"),
    ("air", "JetBrains Air"),
    ("air-ide", "Air (JetBrains IDEs)"),
    ("ds-bot", "DS Bot"),
    ("reasonix", "Reasonix Studio"),
    ("qwen", "Qwen Code"),
    ("ante", "Ante"),
    ("astrbot", "AstrBot"),
    ("openclaw", "OpenClaw"),
    ("jcode", "jcode"),
    ("vibe", "Mistral Vibe"),
    ("cline", "Cline"),
    ("qoder", "Qoder"),
    ("qoder-cn", "Qoder CN"),
    ("grok-app", "Grok App"),
    ("workbuddy", "WorkBuddy"),
    ("workbuddy-ai", "WorkBuddy AI"),
    ("codebuddy", "CodeBuddy"),
    ("pencil", "Pencil"),
    ("paseo", "Paseo"),
    ("openpencil", "OpenPencil"),
    ("openbot", "OpenBot"),
    ("obsidian", "Obsidian"),
    ("newmax", "NewMax"),
    ("liveagent", "LiveAgent"),
    ("raycast", "Raycast"),
    ("kirocrew", "Kiro Crew"),
    ("t3code", "T3 Code"),
    ("openmausbot", "OpenMausBot"),
    ("avibe", "Avibe"),
    ("kun", "Kun"),
    ("nowledge-mem", "Nowledge Mem"),
    ("cindy", "Cindy"),
    ("aionui", "AionUi"),
    ("cherrystudio", "Cherry Studio"),
    ("lody", "Lody"),
];

pub fn by_path(path: &str) -> (&str, Option<&'static str>) {
    if let Some((id, rest)) = path.strip_prefix("agent/").and_then(|s| s.split_once('/'))
        && let Some((_, name)) = CLIENTS.iter().find(|(known, _)| *known == id)
    {
        return (rest, Some(name));
    }
    (path, None)
}

mod command;
pub mod omp_catalog;
pub mod omp_profiles;
pub mod omp_settings;
mod user_agent;
mod yaml;
pub use command::installed_command;
pub(crate) use user_agent::client_name;

/// Model configuration shared by the Pi clients and Oh My Pi.
#[derive(nextjson::NsonSerialize)]
pub struct PiModel<'a> {
    id: &'a str,
    #[njson(skip_serializing_if = "Option::is_none")]
    api: Option<&'a str>,
    #[njson(rename = "baseUrl", skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[njson(skip_serializing_if = "Option::is_none")]
    compat: Option<PiCompatibility>,
}

#[derive(nextjson::NsonSerialize)]
struct PiCompatibility {
    #[njson(rename = "supportsStrictMode")]
    supports_strict_mode: bool,
}

pub fn pi_model<'a>(
    id: &'a str,
    root: &str,
    api: &'a str,
    protocols: Option<&crate::gateway::NativeProtocols>,
    binary: &str,
) -> PiModel<'a> {
    let mut model = PiModel {
        id,
        api: None,
        base_url: None,
        compat: None,
    };
    let selected = if api == "openai-completions" {
        protocols.copied().unwrap_or_default().pi_api()
    } else {
        api
    };
    // Pi and mu's Google adapter cannot use the custom fetch supplied by SDK hosts.
    if selected == "google-generative-ai"
        && matches!(binary, "pi" | "mu")
        && api == "openai-completions"
    {
        return model;
    }
    if selected != api {
        model.api = Some(selected);
        model.base_url = match selected {
            "anthropic-messages" => Some(root.into()),
            "google-generative-ai" => Some(format!("{root}/v1beta")),
            _ => None,
        };
    }
    if selected == "openai-responses" && binary != "omp" {
        model.compat = Some(PiCompatibility {
            supports_strict_mode: true,
        });
    }
    model
}

/// Gemini CLI selects its gateway backend from the base URL alone. The
/// explicit API-key selection enables its Gemini protocol for this profile.
pub fn gemini_home(dir: &std::path::Path) -> anyhow::Result<std::path::PathBuf> {
    let home = std::path::absolute(dir.join("agents/gemini"))?;
    let settings = crate::config::private_file(&home.join(".gemini"), "settings.json", || {
        Ok(br#"{"security":{"auth":{"selectedType":"gemini-api-key"}}}"#.to_vec())
    })?;
    let settings: nextjson::Value = nextjson::from_slice(&settings).map_err(anyhow::Error::msg)?;
    anyhow::ensure!(
        settings
            .pointer("/security/auth/selectedType")
            .and_then(nextjson::Value::as_str)
            == Some("gemini-api-key"),
        "select gemini-api-key in the Crabforge Gemini profile"
    );
    Ok(home)
}
