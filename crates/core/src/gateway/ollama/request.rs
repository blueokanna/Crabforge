use super::string;
use anyhow::{Result, bail, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Default, Deserialize)]
#[serde(default)]
struct Message {
    role: String,
    content: String,
    images: Vec<String>,
    thinking: String,
    tool_calls: Vec<Value>,
    tool_name: String,
    tool_call_id: String,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub(super) struct Input {
    pub model: String,
    pub name: String,
    messages: Vec<Message>,
    prompt: String,
    system: String,
    images: Vec<String>,
    stream: Option<bool>,
    think: Value,
    options: Value,
    tools: Value,
    format: Value,
}

pub(super) struct Chat {
    pub body: Option<Vec<u8>>,
    pub stream: bool,
    pub thinking: bool,
}

impl Input {
    pub fn into_chat(self, model: &str, generate: bool) -> Result<Chat> {
        let input = self;
        let stream = input.stream.unwrap_or(true);
        let mut req = Chat {
            body: None,
            stream,
            thinking: input.think != false,
        };
        let mut messages = Vec::new();
        if generate {
            if input.prompt.is_empty() && input.images.is_empty() {
                return Ok(req);
            }
            if !input.system.is_empty() {
                messages.push(json!({"role":"system","content":input.system}));
            }
            messages.push(json!({"role":"user","content":content(input.prompt,input.images)}));
        } else {
            if input.messages.is_empty() {
                return Ok(req);
            }
            messages = chat_messages(input.messages)?;
        }
        let mut out = json!({"model": model, "messages": messages, "stream":stream});
        if stream {
            out["stream_options"] = json!({"include_usage":true});
        }
        if !generate && !input.tools.is_null() {
            out["tools"] = input.tools;
        }
        match input.format {
            Value::Null => (),
            Value::String(ref s) if s.is_empty() => (),
            Value::String(ref s) if s == "json" => {
                out["response_format"] = json!({"type":"json_object"})
            }
            schema @ Value::Object(_) => {
                out["response_format"] =
                    json!({"type":"json_schema","json_schema":{"name":"response","schema":schema}})
            }
            _ => bail!("expected format 'json' or a JSON schema"),
        }
        match input.think {
            Value::String(effort) if !effort.is_empty() => out["reasoning_effort"] = json!(effort),
            Value::String(_) | Value::Bool(_) | Value::Null => (),
            _ => bail!("invalid think value"),
        }
        for key in [
            "temperature",
            "top_p",
            "seed",
            "frequency_penalty",
            "presence_penalty",
            "stop",
        ] {
            if let Some(v) = input.options.get(key).filter(|v| !v.is_null()) {
                out[key] = v.clone();
            }
        }
        if let Some(n) = input.options["num_predict"].as_u64().filter(|n| *n > 0) {
            out["max_tokens"] = json!(n);
        }
        req.body = Some(serde_json::to_vec(&out)?);
        Ok(req)
    }
}

fn content(text: String, images: Vec<String>) -> Value {
    if images.is_empty() {
        return Value::String(text);
    }
    let mut parts = Vec::with_capacity(images.len() + 1);
    if !text.is_empty() {
        parts.push(json!({"type":"text","text":text}));
    }
    for image in images {
        let url = if image.starts_with("data:") {
            image
        } else {
            let mime = if image.starts_with("/9j/") {
                "jpeg"
            } else if image.starts_with("R0lGOD") {
                "gif"
            } else if image.starts_with("UklGR") {
                "webp"
            } else {
                "png"
            };
            format!("data:image/{mime};base64,{image}")
        };
        parts.push(json!({"type":"image_url","image_url":{"url":url}}));
    }
    Value::Array(parts)
}

fn chat_messages(messages: Vec<Message>) -> Result<Vec<Value>> {
    let mut open: Vec<(String, String)> = Vec::new();
    let mut out = Vec::with_capacity(messages.len());
    for (i, m) in messages.into_iter().enumerate() {
        let role = m.role.to_ascii_lowercase();
        let mut msg = json!({"role":role});
        match role.as_str() {
            "user" => msg["content"] = content(m.content, m.images),
            "system" | "developer" => msg["content"] = Value::String(m.content),
            "assistant" => {
                if !m.thinking.is_empty() {
                    msg["reasoning_content"] = Value::String(m.thinking);
                }
                msg["content"] = Value::String(m.content);
                if !m.tool_calls.is_empty() {
                    let mut calls = Vec::with_capacity(m.tool_calls.len());
                    open.clear();
                    for (j, call) in m.tool_calls.into_iter().enumerate() {
                        let id = call["id"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("call_{i}_{j}"));
                        let f = &call["function"];
                        let name = string(f, "name")?;
                        let args = match &f["arguments"] {
                            Value::String(s) => s.clone(),
                            Value::Object(_) => f["arguments"].to_string(),
                            Value::Null => "{}".into(),
                            _ => bail!("invalid tool arguments"),
                        };
                        calls.push(json!({"id":id,"type":"function","function":{"name":name,"arguments":args}}));
                        open.push((id, name.into()));
                    }
                    msg["tool_calls"] = Value::Array(calls);
                    if msg["content"] == "" {
                        msg["content"] = Value::Null;
                    }
                }
            }
            "tool" => {
                let mut id = m.tool_call_id;
                let index = if id.is_empty() {
                    open.iter()
                        .position(|(_, name)| *name == m.tool_name)
                        .or((!open.is_empty()).then_some(0))
                } else {
                    open.iter().position(|(call, _)| *call == id)
                };
                if let Some(index) = index {
                    id = open.remove(index).0;
                }
                ensure!(!id.is_empty(), "tool result has no matching call");
                msg["tool_call_id"] = Value::String(id);
                msg["content"] = Value::String(m.content);
                if !m.images.is_empty() {
                    out.push(msg);
                    msg = json!({"role":"user","content":content(String::new(),m.images)});
                }
            }
            _ => bail!("unknown message role {role}"),
        }
        out.push(msg);
    }
    Ok(out)
}
