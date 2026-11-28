use super::CLIENTS;
use std::borrow::Cow;

// These clients have a known request signature without a configurable agent root.
const OTHER_CLIENTS: &[(&str, &str)] = &[
    ("curl", "curl"),
    ("pi-desktop", "PI-Desktop"),
    ("flowdown", "FlowDown"),
    ("bob", "Bob"),
    ("dimcode", "DimCode"),
    ("kimi-code-desktop", "Kimi Code Desktop"),
    ("raft", "Raft"),
    ("catpaw", "CatPaw"),
    ("memory", "Memory"),
];

// Agent id, exact aliases, then the product prefixes used by versioned clients.
const SIGNATURES: &[(&str, &[&str], &[&str])] = &[
    (
        "claude",
        &["cc", "claude-code"],
        &["claude-cli", "claude-code"],
    ),
    ("claude-desktop", &["claude-app"], &[]),
    ("claude-science", &["operon", "science"], &[]),
    ("codex", &[], &["codex", "acp-extension-codex"]),
    ("agy", &["antigravity-cli"], &[]),
    ("dim", &["dima", "dimcode-alpha"], &[]),
    ("antigravity", &["antigravity-app", "agy-app"], &[]),
    ("opencode", &["oc"], &["opencode"]),
    ("openchamber", &["chamber"], &[]),
    ("mimocode", &["mimo"], &["mimocode"]),
    ("deveco", &[], &["deveco"]),
    ("pi", &[], &["pi-"]),
    ("goose", &[], &["goose"]),
    ("cursor", &["cursor-agent"], &["cursor"]),
    ("cursor-local", &["cursor-private-inference"], &[]),
    ("copilot", &["gh-copilot"], &["copilot", "github-copilot"]),
    (
        "vscode",
        &["vs-code", "copilot-chat", "vscode-chat"],
        &["githubcopilotchat", "vscode_agent_host"],
    ),
    (
        "vscode-insiders",
        &["vs-code-insiders", "code-insiders"],
        &[],
    ),
    ("vscodium", &["codium", "vscodium-chat"], &[]),
    (
        "copilot-jetbrains",
        &[
            "jetbrains",
            "jetbrains-copilot",
            "copilot-idea",
            "copilot-intellij",
            "intellij",
            "idea",
        ],
        &[],
    ),
    ("crush", &[], &["crush"]),
    ("dsh", &[], &["deepseek-harness"]),
    (
        "commandcode",
        &["command-code", "cmd"],
        &["command-code", "commandcode"],
    ),
    ("omp", &["oh-my-pi"], &["oh-my-pi"]),
    ("omo", &["oh-my-openagent", "omo-ai"], &[]),
    ("ds-bot", &["dsbot", "deepseek-bot"], &[]),
    ("mu", &["mu-agent"], &[]),
    ("hermes", &["hermes-agent"], &["hermes-agent"]),
    ("morph", &["mistermorph", "mister-morph"], &["mistermorph"]),
    (
        "kimi",
        &["kimi-code", "kimi-cli"],
        &["kimicli", "kimi-code-cli"],
    ),
    (
        "muse",
        &["muse-code", "musecode"],
        &["muse-build", "muse-code"],
    ),
    ("empryo", &["soulforge"], &[]),
    ("nanobot", &["nanobot-ai"], &["nanobot"]),
    ("minimax-code", &["mcode"], &["minimax-code", "minimaxcode"]),
    (
        "craft-agents",
        &["craft", "craftagents", "craft-agent"],
        &[],
    ),
    (
        "tuanjie-cowork",
        &["tuanjie", "tjcowork", "codely-cowork", "tuanjiecowork"],
        &[],
    ),
    ("droid", &["factory", "factory-droid"], &["factory-cli"]),
    (
        "grok",
        &["grok-build", "grok-cli"],
        &["grok-shell", "grok-pager", "xai-grok-build"],
    ),
    ("kigi", &["kigi-cli"], &["kigi"]),
    ("zcode", &["z-code"], &["zcode"]),
    (
        "hanako",
        &["openhanako", "hana", "hanaagent"],
        &["hanaagent"],
    ),
    ("atomcode", &[], &["atomcode"]),
    ("ocr", &[], &["open-code-review"]),
    ("snow", &["snow-cli", "snow-ai", "snow-app"], &[]),
    ("alma", &[], &["alma"]),
    ("shelley", &[], &["shelley"]),
    ("stepcode", &["step"], &["stepcode"]),
    ("mothx", &["vibecoding"], &[]),
    ("penguin", &["penguin-harness", "penguinharness"], &[]),
    ("zed", &[], &["zed"]),
    ("zoo-code", &["zoocode", "zoo"], &[]),
    ("air", &["jetbrains-air"], &[]),
    (
        "air-ide",
        &["air-plugin", "air-in-ides", "jetbrains-acp"],
        &[],
    ),
    ("reasonix", &["reasonix-studio"], &["reasonix"]),
    ("qwen", &["qwen-code", "qwen-cli"], &["qwencode"]),
    ("vibe", &["mistral-vibe"], &["mistral-vibe"]),
    ("cline", &["cline-cli"], &["cline"]),
    ("qoder", &["qodercli", "qoder-cli", "qoder"], &[]),
    ("qoder-cn", &["qoderclicn", "qodercn", "qodercn-cli"], &[]),
    ("grok-app", &["grokapp"], &[]),
    ("workbuddy", &["work-buddy"], &["workbuddy"]),
    (
        "workbuddy-ai",
        &["workbuddy-intl", "workbuddyai"],
        &["workbuddy-ai"],
    ),
    ("codebuddy", &["codebuddy-code", "cbc"], &["codebuddy"]),
    ("pencil", &["pen.dev", "pen-dev", "pendev"], &[]),
    ("paseo", &["getpaseo"], &[]),
    ("openpencil", &["open-pencil", "openpencil-desktop"], &[]),
    ("openbot", &["open-bot"], &[]),
    ("obsidian", &["obsidian-copilot"], &[]),
    ("newmax", &["newmax-ai", "newmaxai"], &[]),
    ("liveagent", &["live-agent", "k-brain", "kbrain"], &[]),
    ("raycast", &["raycast-ai"], &[]),
    ("kirocrew", &["kiro-crew", "crew"], &[]),
    ("t3code", &["t3", "t3-code"], &[]),
    ("openmausbot", &["omb", "openmaus", "open-maus-bot"], &[]),
    ("avibe", &["vibey", "avibe-os"], &[]),
    ("cindy", &[], &["cindy"]),
    ("cherrystudio", &["cherry-studio"], &[]),
    ("lody", &[], &["lody"]),
    ("curl", &[], &["curl"]),
    ("flowdown", &[], &["flowdown"]),
    ("dimcode", &[], &["dimagent", "dimcode"]),
    ("kimi-code-desktop", &[], &["kimi-code-desktop"]),
];

pub(crate) fn client_name(header: &str) -> Option<Cow<'static, str>> {
    let product = header
        .trim()
        .split(['/', ' '])
        .next()
        .filter(|p| !p.is_empty())?;
    let clients = || CLIENTS.iter().chain(OTHER_CLIENTS);
    if let Some((_, name)) = clients().find(|(id, _)| id.eq_ignore_ascii_case(product)) {
        return Some(Cow::Borrowed(name));
    }
    let lower = product.to_ascii_lowercase();
    let signature = SIGNATURES
        .iter()
        .find(|(_, names, _)| names.contains(&lower.as_str()))
        .or_else(|| {
            SIGNATURES
                .iter()
                .find(|(_, _, prefixes)| prefixes.iter().any(|prefix| lower.starts_with(prefix)))
        });
    if let Some((id, _, _)) = signature
        && let Some((_, name)) = clients().find(|(k, _)| k == id)
    {
        return Some(Cow::Borrowed(name));
    }
    Some(Cow::Owned(product.to_owned()))
}
