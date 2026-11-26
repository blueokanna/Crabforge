use crate::{config::Endpoints, translate::Format};
use axum::http::{header::USER_AGENT, request::Builder as RequestBuilder};
use nextjson::Value;
use std::collections::BTreeMap;
use url::Url;

include!(concat!(env!("OUT_DIR"), "/opencode_models.rs"));

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Console {
    Zen,
    Go,
}

impl Console {
    fn split(url: &Url) -> Option<(Self, &str)> {
        let host = url.host_str()?.trim_end_matches('.');
        if host != "opencode.ai" && !host.ends_with(".opencode.ai") {
            return None;
        }
        let path = url.path().trim_matches('/');
        let (service, suffix) = path.split_once('/').unwrap_or((path, ""));
        if !matches!(service, "zen" | "inference") {
            return None;
        }
        Some(match suffix {
            "go" => (Self::Go, ""),
            rest if rest.starts_with("go/") => (Self::Go, &rest[3..]),
            rest => (Self::Zen, rest),
        })
    }

    /// A custom path on the same host can have a different catalog and protocol.
    pub fn at(url: &Url) -> Option<Self> {
        let (plan, suffix) = Self::split(url)?;
        matches!(
            suffix,
            "" | "v1"
                | "openai"
                | "anthropic"
                | "google"
                | "openai/v1"
                | "anthropic/v1"
                | "google/v1beta"
        )
        .then_some(plan)
    }

    pub fn request(url: &Url) -> Option<Self> {
        let (plan, suffix) = Self::split(url)?;
        let (family, path) = suffix.split_once('/')?;
        let path = match family {
            "openai" | "anthropic" => path.strip_prefix("v1/")?,
            "google" => path.strip_prefix("v1beta/")?,
            "v1" => path,
            _ => return None,
        };
        (matches!(
            path,
            "chat/completions"
                | "responses"
                | "responses/compact"
                | "messages"
                | "messages/count_tokens"
                | "embeddings"
                | "models"
                | "decisions"
        ) || path.starts_with("models/"))
        .then_some(plan)
    }

    fn base(self) -> &'static str {
        match self {
            Self::Go => "/inference/go/",
            Self::Zen => "/inference/",
        }
    }

    pub fn models_url(self, url: &Url) -> Url {
        url.join(&format!("{}v1/models", self.base()))
            .expect("absolute Console URL")
    }

    pub fn format(self, model: &str) -> Option<Format> {
        let plan = match self {
            Self::Go => "opencode-go",
            Self::Zen => "opencode",
        };
        let index = MODELS
            .binary_search_by_key(&(plan, model), |entry| entry.0)
            .ok()?;
        Some(MODELS[index].1)
    }

    pub fn fill_missing(bases: &mut Endpoints) {
        let console = [
            &bases.chat,
            &bases.responses,
            &bases.anthropic,
            &bases.gemini,
        ]
        .into_iter()
        .find_map(|base| {
            let url = Url::parse(base).ok()?;
            Some((Self::at(&url)?, url))
        });
        let Some((console, url)) = console else {
            return;
        };
        let root = url.join(console.base()).expect("absolute Console URL");
        for (base, path) in [
            (&mut bases.chat, "openai/v1"),
            (&mut bases.responses, "openai/v1"),
            (&mut bases.anthropic, "anthropic"),
            (&mut bases.gemini, "google/v1beta"),
        ] {
            if base.is_empty() {
                *base = root.join(path).expect("absolute Console URL").to_string();
            }
        }
    }
}

pub(crate) const USER_AGENT_VALUE: &str = concat!("crabforge/", env!("CARGO_PKG_VERSION"));

// Existing conversation IDs hash the opening turn with its object keys sorted.
fn ordered_turn(value: &Value) -> Value {
    match value {
        Value::Object(obj) => {
            let fields: BTreeMap<_, _> = obj.iter().map(|(k, v)| (k, ordered_turn(v))).collect();
            Value::Object(fields.into_iter().collect())
        }
        Value::Array(items) => Value::Array(items.iter().map(ordered_turn).collect()),
        other => other.clone(),
    }
}

/// Keep the client's conversation ID across protocol conversion. When it has
/// none, derive an opaque local ID from the cache key or first user turn.
pub(crate) fn conversation(
    request: RequestBuilder,
    session: Option<&str>,
    body: &[u8],
    key: &str,
) -> RequestBuilder {
    let request = request.header(USER_AGENT, USER_AGENT_VALUE);
    if let Some(id) = session {
        return request.header("x-opencode-session", id);
    }
    let json: Value = nextjson::from_slice(body).unwrap_or(Value::Null);
    let opening = ["input", "messages", "contents"]
        .iter()
        .filter_map(|k| json.get(k))
        .find_map(|input| {
            if input.is_string() {
                return Some(input);
            }
            input.as_array()?.iter().find(|turn| {
                turn.get("role").and_then(Value::as_str) == Some("user")
                    || (turn.get("role").is_none() && turn.get("parts").is_some())
            })
        });
    let hash_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key.as_bytes());
    let mut hash = ring::hmac::Context::with_key(&hash_key);
    if let Some(cache_key) = json
        .get("prompt_cache_key")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    {
        hash.update(b"cache\0");
        hash.update(cache_key.as_bytes());
    } else if let Some(turn) = opening {
        hash.update(b"opening\0");
        hash.update(ordered_turn(turn).to_string().as_bytes());
    } else {
        hash.update(b"request\0");
        hash.update(body);
    }
    request.header(
        "x-opencode-session",
        hex::encode(&hash.sign().as_ref()[..16]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AuthHeader, authenticate};

    #[test]
    fn console_session_survives_turns_and_auth_uses_the_actual_destination() -> anyhow::Result<()> {
        let url = Url::parse(
            "https://opencode.ai/inference/go/google/v1beta/models/gemini-4-argon:generateContent",
        )?;
        let build = |session: Option<&str>, body: &[u8]| -> anyhow::Result<axum::http::HeaderMap> {
            let request = authenticate(
                axum::http::Request::post(url.as_str()),
                &url,
                "gemini",
                "provider-key",
                AuthHeader::Both,
            );
            Ok(conversation(request, session, body, "local-key")
                .body(())?
                .headers()
                .clone())
        };
        let first = build(None, br#"{"messages":[{"role":"user","content":"hello"}]}"#)?;
        assert_eq!(
            first["x-opencode-session"],
            "2929c1e92880a27ab1c601be63cc6f3e"
        );
        let later = build(None, br#"{"messages":[{"role":"user","content":"hello"},{"role":"assistant","content":"hi"},{"role":"user","content":"again"}]}"#)?;
        assert_eq!(first["x-opencode-session"], later["x-opencode-session"]);
        assert_eq!(first["authorization"], "Bearer provider-key");
        assert_eq!(first[USER_AGENT], USER_AGENT_VALUE);
        assert!(!first.contains_key("x-goog-api-key"));
        let other = build(None, br#"{"messages":[{"role":"user","content":"else"}]}"#)?;
        assert_ne!(first["x-opencode-session"], other["x-opencode-session"]);
        assert_eq!(
            build(Some("codex-thread"), b"{}")?["x-opencode-session"],
            "codex-thread"
        );
        assert_eq!(
            build(Some("client-session"), b"{}")?["x-opencode-session"],
            "client-session"
        );
        for unrelated in [
            "https://relay.example/inference/go",
            "https://opencode.ai/inference/custom",
            "https://otheropencode.ai/zen",
            "https://opencode.ai/inference/go/vendor/v1",
            "https://opencode.ai/inference/custom/openai/v1/chat/completions",
        ] {
            let url = Url::parse(unrelated)?;
            assert!(Console::at(&url).is_none() && Console::request(&url).is_none());
        }
        Ok(())
    }
}
