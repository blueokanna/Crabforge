use crate::config::Endpoints;

// backup's presets supply these lists for services without a usable model
// catalog. Account permissions and regional availability still apply.
const BEDROCK: &[&str] = &[
    "global.anthropic.claude-opus-5-5",
    "global.anthropic.claude-sonnet-5-5",
    "global.anthropic.claude-sonnet-5",
    "global.anthropic.claude-opus-5",
    "global.anthropic.claude-fable-5-2",
    "global.anthropic.claude-fable-5-1",
    "global.anthropic.claude-opus-4-8",
    "global.anthropic.claude-opus-4-7",
    "global.anthropic.claude-haiku-4-5-20251001-v1:0",
    "global.openai.gpt-6-astra",
    "global.openai.gpt-6-sol",
    "global.openai.gpt-6-luna",
    "openai.gpt-oss-120b-1:0",
    "openai.gpt-oss-20b-1:0",
    "qwen.qwen3-coder-480b-a35b-v1:0",
    "deepseek.v3.2",
    "moonshotai.kimi-k2.5",
    "zai.glm-5",
    "minimax.minimax-m2.5",
];
const VERTEX_GLOBAL: &[&str] = &[
    "gemini-3.8-flash",
    "gemini-3.7-flash",
    "gemini-3.6-flash",
    "gemini-3.5-flash",
    "gemini-3.5-flash-lite",
    "gemini-4-carbon",
    "gemini-4-hydrogen",
    "gemini-4-argon",
    "gemini-3.1-pro-preview",
    "gemini-3.1-pro-preview-customtools",
    "gemini-3.1-flash-lite",
    "gemini-3-flash-preview",
    "gemini-2.5-pro",
    "gemini-2.5-flash",
    "gemini-2.5-flash-lite",
    "deepseek-ai/deepseek-v4.1-flash-maas",
    "zai-org/glm-5.2-maas",
    "openai/gpt-oss-120b-maas",
    "claude-opus-5-5",
    "claude-sonnet-5-5",
    "claude-haiku-5-5",
    "claude-fable-5-2",
    "claude-fable-5-1",
];
const VOYAGE: &[&str] = &[
    "voyage-4-large",
    "voyage-4",
    "voyage-4-lite",
    "voyage-code-4",
    "voyage-finance-2",
    "voyage-law-2",
    "voyage-3-large",
    "voyage-3.5",
    "voyage-3.5-lite",
    "voyage-code-3",
    "voyage-code-2",
    "rerank-3",
    "rerank-3-lite",
    "rerank-2.5",
    "rerank-2.5-lite",
];
const ARK_PLANS: &[&str] = &[
    "ark-code-latest",
    "doubao-seed-evolving",
    "doubao-seed-2.1-pro",
    "doubao-seed-2.1-lite",
    "doubao-seed-2.0-mini",
    "minimax-m3",
    "glm-5.3",
    "glm-5.3-flash",
    "deepseek-v4.1-flash",
    "deepseek-v4.1-flash-economy",
    "deepseek-v4-flash",
    "deepseek-v4-pro",
    "kimi-k2.8-preview",
    "kimi-k3",
    "kimi-k3.1-pro",
];
const QIANFAN_PERSONAL: &[&str] = &[
    "qianfan-code-latest",
    "deepseek-v4.1-flash",
    "deepseek-v4-pro",
    "deepseek-v4-pro-0813",
    "deepseek-v4-flash-0731",
    "glm-5.3",
    "glm-5.3-flash",
    "glm-5.2",
    "glm-5.1",
];
const QIANFAN_TEAM: &[&str] = &[
    "qianfan-code-latest",
    "deepseek-v4.1-flash",
    "deepseek-v4-pro",
    "deepseek-v4-pro-0813",
    "deepseek-v4-flash",
    "deepseek-v4-flash-0731",
    "deepseek-v3.2",
    "glm-5.3",
    "glm-5.3-flash",
    "glm-5.2",
    "glm-5.1",
    "glm-5",
];

const TOKENHUB_CHINA: &[&str] = &[
    "tc-code-latest",
    "auto",
    "glm-5.3",
    "glm-5.3-flash",
    "glm-5.2",
    "glm-5.1",
    "glm-5",
    "kimi-k3",
    "kimi-k2.7-code",
    "kimi-k2.7-code-highspeed",
    "kimi-k2.6",
    "deepseek-v4-pro-202606",
    "deepseek-v4-flash-202605",
    "deepseek-v4.1-flash",
    "deepseek-v4-pro",
    "deepseek-v4-pro-0813",
    "deepseek-v4-flash-0731",
    "minimax-m3",
    "minimax-m2.7",
    "hy4-preview",
    "hy3",
    "mimo-v2.6-pro",
    "mimo-v2.6-flash",
];
const TOKENHUB_GLOBAL: &[&str] = &[
    "auto",
    "hy4-preview",
    "glm-5.3",
    "glm-5.3-flash",
    "glm-5.2",
    "kimi-k3",
    "kimi-k2.7-code",
    "kimi-k2.7-code-highspeed",
    "minimax-m3",
    "deepseek-v4.1-flash",
    "deepseek-v4-pro",
    "deepseek-v4-pro-0813",
    "deepseek-v4-flash-0731",
    "deepseek-v4-pro-202606",
    "deepseek-v4-flash-202605",
    "mimo-v2.6-pro",
    "mimo-v2.6-flash",
];

pub fn fixed_models(bases: &Endpoints) -> Option<&'static [&'static str]> {
    if !bases.models_url.is_empty() {
        return None;
    }
    if let Some(v) = &bases.vertex {
        return Some(match v.location() {
            _ if v.project.is_empty() => &VERTEX_GLOBAL[..15],
            "global" => VERTEX_GLOBAL,
            "us" | "eu" => &[
                "gemini-3.8-flash",
                "gemini-3.7-flash",
                "gemini-3.6-flash",
                "gemini-3.5-flash",
                "gemini-3.5-flash-lite",
                "gemini-4-carbon",
                "gemini-4-hydrogen",
                "gemini-4-argon",
                "gemini-3.1-flash-lite",
            ],
            _ => &VERTEX_GLOBAL[12..15],
        });
    }
    let chat = bases.chat.trim_end_matches('/');
    match chat {
        "https://api.voyageai.com/v1" => Some(VOYAGE),
        "https://tokenhub.tencentmaas.com/plan/v3" => Some(TOKENHUB_CHINA),
        "https://tokenhub-intl.tencentmaas.com/plan/v3" => Some(TOKENHUB_GLOBAL),
        "https://ark.cn-beijing.volces.com/api/coding/v3"
        | "https://ark.cn-beijing.volces.com/api/plan/v3" => Some(ARK_PLANS),
        "https://qianfan.baidubce.com/v2/tokenplan/personal" => Some(QIANFAN_PERSONAL),
        "https://qianfan.baidubce.com/v2/tokenplan/team" => Some(QIANFAN_TEAM),
        "https://api.modelarts-maas.com/plan/v2" => Some(&[
            "glm-5.3",
            "glm-5.1",
            "kimi-k2.6",
            "deepseek-v4.1-flash",
            "deepseek-v4-flash",
        ]),
        "" if bases.responses.is_empty()
            && bases.anthropic.is_empty()
            && bases.gemini.trim_end_matches('/')
                == "https://aiplatform.googleapis.com/v1/publishers/google" =>
        {
            Some(&VERTEX_GLOBAL[..15])
        }
        _ => {
            let base = [chat, &bases.responses, &bases.anthropic]
                .into_iter()
                .find(|url| !url.is_empty())?;
            let url = url::Url::parse(base).ok()?;
            (crate::bedrock::scope(&url)?.1 == "bedrock"
                && matches!(
                    url.path().trim_end_matches('/'),
                    "/openai/v1" | "/anthropic"
                ))
            .then_some(BEDROCK)
        }
    }
}
