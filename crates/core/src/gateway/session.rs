use http::{HeaderMap, HeaderValue, header::USER_AGENT};
use std::borrow::Cow;

pub(super) struct Caller<'a> {
    pub agent: Option<Cow<'static, str>>,
    pub session: Option<&'a str>,
    pub native_session: Option<&'a str>,
}

impl<'a> Caller<'a> {
    pub fn new(agent: Option<&'static str>, headers: &'a HeaderMap) -> Self {
        let id = |name: &str| {
            let value = headers.get(name)?.to_str().ok()?.trim();
            // HTTP text header values are ASCII, so the byte bound is a character boundary.
            (!value.is_empty()).then(|| &value[..value.len().min(128)])
        };
        let native_session = [
            "x-opencode-session",
            "x-session-affinity",
            "x-session-id",
            "session_id",
            "session-id",
            "x-claude-code-session-id",
            "x-conversation-id",
            "x-deepseek-harness-session-id",
        ]
        .into_iter()
        .find_map(id);
        Self {
            agent: agent
                .map(Cow::Borrowed)
                .or_else(|| crate::agents::client_name(headers.get(USER_AGENT)?.to_str().ok()?)),
            session: id("x-crabforge-session")
                .or_else(|| id(concat!("x-mag", "pie-session")))
                .or(native_session),
            native_session,
        }
    }

    pub fn set_response_headers(&self, headers: &mut HeaderMap) {
        if let Some(id) = self.session {
            let value = HeaderValue::from_str(id).expect("validated session header");
            headers.insert("x-crabforge-session", value);
        }
    }
}
