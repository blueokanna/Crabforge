use super::{Api, Config, Format, Provider};
use crate::config::ResolvedGroups;
use std::collections::BTreeMap;

/// The wire protocols the upstream accepts directly, and the one to put in an agent's catalog.
#[derive(Clone, Copy, Default)]
pub struct NativeProtocols {
    chat: bool,
    responses: bool,
    messages: bool,
    preferred: Option<Format>,
}

impl NativeProtocols {
    pub(super) fn model(p: &Provider, model: &str) -> Self {
        if p.web_site.is_some() {
            return Self::default();
        }
        let chat = Api::Chat("chat/completions").url(p, model).is_some();
        let responses_url = Api::Responses("responses").url(p, model);
        let responses = responses_url.is_some();
        let messages = Api::Messages("messages").url(p, model).is_some();
        let gemini = Api::Gemini("generateContent").url(p, model).is_some();
        let leaf = model
            .rsplit('/')
            .next()
            .unwrap_or(model)
            .to_ascii_lowercase();
        let openai = leaf.starts_with("gpt-")
            || leaf.starts_with("codex")
            || leaf
                .strip_prefix('o')
                .is_some_and(|s| s.starts_with(|c: char| c.is_ascii_digit()));
        let prefers_responses = responses_url.as_ref().is_some_and(|url| {
            url.host_str() == Some("api-inference.modelscope.cn")
                || p.id == "modelscope"
                || (openai
                    && (p.id == "copilot"
                        || matches!(url.host_str(), Some("api.openai.com" | "api.pipellm.ai"))
                        || crate::config::azure(url)))
                || (crate::bedrock::scope(url).is_some()
                    && leaf.contains("openai.gpt-")
                    && !leaf.contains("gpt-oss"))
        });
        let preferred = if prefers_responses {
            Some(Format::Responses)
        } else if messages && (leaf.starts_with("claude") || leaf.contains("anthropic.claude")) {
            Some(Format::Messages)
        } else if chat {
            Some(Format::Chat)
        } else if gemini && leaf.starts_with("gemini") {
            Some(Format::Gemini)
        } else if responses {
            Some(Format::Responses)
        } else if messages {
            Some(Format::Messages)
        } else {
            gemini.then_some(Format::Gemini)
        };
        Self {
            chat,
            responses,
            messages,
            preferred,
        }
    }

    pub(super) fn group(mut members: impl Iterator<Item = Self>) -> Self {
        let Some(mut common) = members.next() else {
            return Self::default();
        };
        let mut prefers_responses = common.preferred == Some(Format::Responses);
        for member in members {
            common.chat &= member.chat;
            common.responses &= member.responses;
            common.messages &= member.messages;
            prefers_responses |= member.preferred == Some(Format::Responses);
        }
        common.preferred = (common.responses && prefers_responses).then_some(Format::Responses);
        common
    }

    pub fn omp_api(self) -> &'static str {
        match self.preferred {
            Some(Format::Responses) => "openai-responses",
            Some(Format::Messages) => "anthropic-messages",
            Some(Format::Gemini) => "google-generative-ai",
            _ => "openai-completions",
        }
    }

    pub(super) fn endpoints(self) -> Vec<&'static str> {
        [
            (self.chat, "/v1/chat/completions"),
            (self.responses, "/v1/responses"),
            (self.messages, "/v1/messages"),
        ]
        .into_iter()
        .filter_map(|(served, path)| served.then_some(path))
        .collect()
    }

    pub(super) fn preferred_endpoint(self) -> Option<&'static str> {
        match self.preferred? {
            Format::Chat => Some("/v1/chat/completions"),
            Format::Responses => Some("/v1/responses"),
            Format::Messages => Some("/v1/messages"),
            Format::Gemini => None,
        }
    }
}

pub fn native_protocols(
    config: &Config,
    groups: &ResolvedGroups,
) -> anyhow::Result<BTreeMap<String, NativeProtocols>> {
    let mut catalog = BTreeMap::new();
    for provider in &config.providers {
        let mut p = provider.clone();
        crate::opencode::Console::fill_missing(&mut p.endpoints);
        for model in &p.models {
            catalog.insert(
                format!("{}/{model}", p.id),
                NativeProtocols::model(&p, model),
            );
        }
    }
    for name in Config::group_order(&groups.members)? {
        if config.disabled_groups.contains(name) {
            continue;
        }
        let off = config
            .group_off
            .get(name)
            .filter(|_| !config.group_picks.contains_key(name));
        let members = &groups.members[name];
        let saved = config.groups.get(name).unwrap_or(members);
        let common = NativeProtocols::group(
            saved
                .iter()
                .zip(members)
                .filter(|(id, _)| !off.is_some_and(|off| off.contains(*id)))
                .filter_map(|(_, m)| catalog.get(m).copied()),
        );
        catalog.insert(format!("group/{name}"), common);
    }
    for (id, target) in &groups.aliases {
        if let Some(&info) = catalog.get(target) {
            catalog.entry(id.clone()).or_insert(info);
        }
    }
    for p in &config.providers {
        for model in &p.models {
            let info = catalog[&format!("{}/{model}", p.id)];
            catalog.entry(model.clone()).or_insert(info);
        }
    }
    Ok(catalog)
}
