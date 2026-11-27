use super::{Api, Caller, Gateway, Inbound, Route, forward, json_response};
use anyhow::Result;
use axum_core::response::Response;
use bytes::Bytes;
use http::{Method, StatusCode, request::Parts};
use nextjson::{Value, json};
use sha2::{Digest, Sha256};
use std::time::Instant;

mod reply;
mod request;
pub(super) use reply::Output;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Endpoint {
    Version,
    Tags,
    Show,
    Chat,
    Generate,
}

pub(super) fn error(status: StatusCode, message: &str) -> Response {
    json_response(status, &json!({"error":message}))
}

fn details() -> Value {
    json!({"parent_model":"", "format":"", "family":"crabforge", "families":["crabforge"], "parameter_size":"", "quantization_level":""})
}

async fn group_tools(gw: &Gateway) -> Result<Vec<bool>> {
    let picks = gw.current_picks().await?;
    let mut tools = Vec::new();
    if !picks.is_empty() {
        for group in &gw.group_ids {
            let supported = gw
                .live_members(&group["group/".len()..], &picks)
                .iter()
                .any(|m| match gw.routes[m] {
                    Route::Group { index, .. } => tools[index],
                    route => route.tools(&gw.config),
                });
            tools.push(supported);
        }
    }
    Ok(tools)
}

fn capabilities(gw: &Gateway, model: &str, group_tools: &[bool]) -> Vec<&'static str> {
    let mut caps = vec!["completion"];
    let route = gw.routes[model];
    if match route {
        Route::Group { index, .. } if !group_tools.is_empty() => group_tools[index],
        _ => route.tools(&gw.config),
    } {
        caps.push("tools");
    }
    caps
}

impl Endpoint {
    pub(super) fn parse(path: &str) -> Option<Self> {
        Some(match path {
            "api/version" => Self::Version,
            "api/tags" => Self::Tags,
            "api/show" => Self::Show,
            "api/chat" => Self::Chat,
            "api/generate" => Self::Generate,
            _ => return None,
        })
    }

    pub(super) async fn serve(
        self,
        gw: &Gateway,
        caller: &Caller<'_>,
        parts: &Parts,
        body: impl Future<Output = Result<Bytes>>,
    ) -> Response {
        if matches!(self, Self::Version | Self::Tags) {
            if parts.method != Method::GET {
                return error(StatusCode::METHOD_NOT_ALLOWED, "GET required");
            }
            return match self {
                Self::Version => json_response(StatusCode::OK, &json!({"version":"0.12.6"})),
                _ => {
                    let tools = match group_tools(gw).await {
                        Ok(tools) => tools,
                        Err(e) => return error(StatusCode::SERVICE_UNAVAILABLE, &e.to_string()),
                    };
                    let models: Vec<_> = gw.listed_ids().map(|id| json!({"name":id, "model":id, "size":0,
                        "modified_at":"1970-01-01T00:00:00Z", "digest":hex::encode(Sha256::digest(id.as_bytes())),
                        "details":details(), "capabilities":capabilities(gw, id, &tools)})).collect();
                    json_response(StatusCode::OK, &json!({"models":models}))
                }
            };
        }
        if parts.method != Method::POST {
            return error(StatusCode::METHOD_NOT_ALLOWED, "POST required");
        }
        let (body, permit) = match gw.read_body(body).await {
            Ok(input) => input,
            Err((status, msg)) => return error(status, msg),
        };
        let input: request::Input = match nextjson::from_slice(&body) {
            Ok(value) => value,
            Err(e) => return error(StatusCode::BAD_REQUEST, &e.to_string()),
        };
        drop(body);
        let mut id = input.model.trim();
        if id.is_empty() && self == Self::Show {
            id = input.name.trim();
        }
        if id.is_empty() {
            return error(StatusCode::BAD_REQUEST, "model is required");
        }
        let Some((id, route)) = gw
            .resolve(id)
            .or_else(|| gw.resolve(id.strip_suffix(":latest")?))
        else {
            return error(StatusCode::NOT_FOUND, &format!("model '{id}' not found"));
        };
        if matches!(route, Route::Disabled) {
            return error(StatusCode::NOT_FOUND, "route group is switched off");
        }
        if self == Self::Show {
            let tools = match group_tools(gw).await {
                Ok(tools) => tools,
                Err(e) => return error(StatusCode::SERVICE_UNAVAILABLE, &e.to_string()),
            };
            return json_response(
                StatusCode::OK,
                &json!({"details":details(), "capabilities":capabilities(gw,id,&tools), "model_info":{
                "general.architecture":"crabforge", "general.basename":id, "general.parameter_count":0,
                "crabforge.context_length":32768, "crabforge.embedding_length":0}}),
            );
        }
        let generate = self == Self::Generate;
        let request = match input.into_chat(id, generate) {
            Ok(req) => req,
            Err(e) => return error(StatusCode::BAD_REQUEST, &e.to_string()),
        };
        let output = reply::Output {
            model: id.clone(),
            generate,
            stream: request.stream,
            thinking: request.thinking,
            start: Instant::now(),
        };
        let Some(chat) = request.body else {
            return output.loaded();
        };
        // The translated Chat request uses the common routing and concurrency path.
        let input = Inbound {
            api: Api::Chat("chat/completions"),
            path_model: None,
            headers: &parts.headers,
            bytes: Bytes::from(chat),
            ollama: Some(output),
            classifier: false,
        };
        let response = forward(gw, caller, input, permit).await;
        if response.status().is_success() {
            response
        } else {
            reply::convert_error(response).await
        }
    }
}
