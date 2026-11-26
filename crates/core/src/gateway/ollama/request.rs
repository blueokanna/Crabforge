use anyhow::{Context, Result, bail, ensure};
use nextjson::{Map, NsonDeserialize, Value, json};

#[derive(Default, NsonDeserialize)]
#[njson(default)]
struct Message {
    role: String,
    content: String,
    images: Vec<String>,
    thinking: String,
    tool_calls: Vec<Value>,
    tool_name: String,
    tool_call_id: String,
}

#[derive(Default, NsonDeserialize)]
#[njson(default)]
pub(super) struct Input {
    pub model: String,
    pub name: String,
    messages: Vec<Message>,
    prompt: String,
    system: String,
    images: Vec<String>,
    stream: Option<bool>,
    think: Option<Value>,
    options: Option<Value>,
    tools: Option<Value>,
    format: Option<Value>,
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
            thinking: !matches!(input.think, Some(Value::Bool(false))),
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
        let mut out = Map::new();
        out.insert("model".into(), model.into());
        out.insert("messages".into(), Value::Array(messages));
        out.insert("stream".into(), stream.into());
        if stream {
            out.insert("stream_options".into(), json!({"include_usage":true}));
        }
        if !generate && let Some(tools) = input.tools {
            out.insert("tools".into(), tools);
        }
        match input.format.unwrap_or(Value::Null) {
            Value::Null => (),
            Value::String(ref s) if s.is_empty() => (),
            Value::String(ref s) if s == "json" => {
                out.insert("response_format".into(), json!({"type":"json_object"}));
            }
            schema @ Value::Object(_) => {
                out.insert(
                    "response_format".into(),
                    json!({"type":"json_schema","json_schema":{"name":"response","schema":schema}}),
                );
            }
            _ => bail!("expected format 'json' or a JSON schema"),
        }
        match input.think.unwrap_or(Value::Null) {
            Value::String(effort) if !effort.is_empty() => {
                out.insert("reasoning_effort".into(), effort.into());
            }
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
            if let Some(v) = input
                .options
                .as_ref()
                .and_then(|o| o.get(key))
                .filter(|v| !v.is_null())
            {
                out.insert(key.into(), v.clone());
            }
        }
        if let Some(n) = input
            .options
            .as_ref()
            .and_then(|o| o.get("num_predict"))
            .and_then(crate::json::unsigned)
            .filter(|n| *n > 0)
        {
            out.insert("max_tokens".into(), n.into());
        }
        req.body = Some(nextjson::to_vec(&out).map_err(anyhow::Error::msg)?);
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
        let mut msg = Map::new();
        msg.insert("role".into(), role.as_str().into());
        match role.as_str() {
            "user" => {
                msg.insert("content".into(), content(m.content, m.images));
            }
            "system" | "developer" => {
                msg.insert("content".into(), m.content.into());
            }
            "assistant" => {
                if !m.thinking.is_empty() {
                    msg.insert("reasoning_content".into(), m.thinking.into());
                }
                let content = if m.content.is_empty() && !m.tool_calls.is_empty() {
                    Value::Null
                } else {
                    m.content.into()
                };
                msg.insert("content".into(), content);
                if !m.tool_calls.is_empty() {
                    let mut calls = Vec::with_capacity(m.tool_calls.len());
                    open.clear();
                    for (j, call) in m.tool_calls.into_iter().enumerate() {
                        let id = call
                            .get("id")
                            .and_then(Value::as_str)
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("call_{i}_{j}"));
                        let f = call.get("function").context("missing tool function")?;
                        let name = f
                            .get("name")
                            .and_then(Value::as_str)
                            .context("expected string name")?;
                        let args = match f.get("arguments").unwrap_or(&Value::Null) {
                            Value::String(s) => s.clone(),
                            args @ Value::Object(_) => args.to_string(),
                            Value::Null => "{}".into(),
                            _ => bail!("invalid tool arguments"),
                        };
                        calls.push(json!({"id":id,"type":"function","function":{"name":name,"arguments":args}}));
                        open.push((id, name.into()));
                    }
                    msg.insert("tool_calls".into(), Value::Array(calls));
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
                msg.insert("tool_call_id".into(), id.into());
                msg.insert("content".into(), m.content.into());
                if !m.images.is_empty() {
                    out.push(Value::Object(msg));
                    out.push(json!({"role":"user","content":content(String::new(),m.images)}));
                    continue;
                }
            }
            _ => bail!("unknown message role {role}"),
        }
        out.push(Value::Object(msg));
    }
    Ok(out)
}
