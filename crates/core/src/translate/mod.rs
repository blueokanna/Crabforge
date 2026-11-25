mod reply;
mod request;
pub(crate) mod sse;
mod stream_input;
mod stream_output;

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::collections::HashMap;

pub use reply::convert_reply;
pub use request::convert_request;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Chat,
    Responses,
    Messages,
    Gemini,
}

#[derive(Debug)]
enum Part {
    Text(String),
    Call {
        id: String,
        name: String,
        args: Value,
    },
    Result {
        id: String,
        output: String,
    },
}

struct Turn {
    role: String,
    parts: Vec<Part>,
}

fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key]
        .as_str()
        .with_context(|| format!("expected string {key}"))
}
fn array<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    v[key]
        .as_array()
        .with_context(|| format!("expected array {key}"))
}
fn keys(v: &Value, allowed: &[&str]) -> Result<()> {
    for key in v.as_object().context("expected a JSON object")?.keys() {
        ensure!(
            allowed.contains(&key.as_str()),
            "cannot translate field {key}"
        );
    }
    Ok(())
}
fn strings(parts: &[Part]) -> Result<String> {
    parts
        .iter()
        .map(|p| match p {
            Part::Text(s) => Ok(s.as_str()),
            _ => bail!("expected text content"),
        })
        .collect()
}

fn read_parts(format: Format, content: &Value) -> Result<Vec<Part>> {
    if let Some(s) = content.as_str() {
        return Ok(vec![Part::Text(s.into())]);
    }
    let mut parts = vec![];
    for p in content
        .as_array()
        .context("expected text or content blocks")?
    {
        let part = match format {
            Format::Gemini => {
                if let Some(s) = p["text"].as_str() {
                    keys(p, &["text"])?;
                    Part::Text(s.into())
                } else if let Some(call) = p.get("functionCall") {
                    keys(p, &["functionCall"])?;
                    let name = text(call, "name")?;
                    Part::Call {
                        id: call["id"].as_str().unwrap_or(name).into(),
                        name: name.into(),
                        args: call["args"].clone(),
                    }
                } else if let Some(result) = p.get("functionResponse") {
                    keys(p, &["functionResponse"])?;
                    let name = text(result, "name")?;
                    Part::Result {
                        id: result["id"].as_str().unwrap_or(name).into(),
                        output: result["response"].to_string(),
                    }
                } else {
                    bail!("unsupported Gemini part")
                }
            }
            _ => match text(p, "type")? {
                "text" | "input_text" | "output_text" => Part::Text(text(p, "text")?.into()),
                "tool_use" if format == Format::Messages => Part::Call {
                    id: text(p, "id")?.into(),
                    name: text(p, "name")?.into(),
                    args: p["input"].clone(),
                },
                "tool_result" if format == Format::Messages => {
                    ensure!(
                        p["is_error"] != true,
                        "cannot preserve an error tool result on this protocol"
                    );
                    Part::Result {
                        id: text(p, "tool_use_id")?.into(),
                        output: strings(&read_parts(format, &p["content"])?)?,
                    }
                }
                other => bail!("cannot translate content block {other}"),
            },
        };
        parts.push(part);
    }
    Ok(parts)
}

fn read_turn(format: Format, m: &Value) -> Result<Turn> {
    if format == Format::Responses {
        match m["type"].as_str() {
            Some("function_call") => {
                return Ok(Turn {
                    role: "assistant".into(),
                    parts: vec![Part::Call {
                        id: text(m, "call_id")?.into(),
                        name: text(m, "name")?.into(),
                        args: serde_json::from_str(text(m, "arguments")?)?,
                    }],
                });
            }
            Some("function_call_output") => {
                return Ok(Turn {
                    role: "user".into(),
                    parts: vec![Part::Result {
                        id: text(m, "call_id")?.into(),
                        output: text(m, "output")?.into(),
                    }],
                });
            }
            None | Some("message") => (),
            other => bail!("cannot translate Responses input item {other:?}"),
        }
    }
    let role = m["role"]
        .as_str()
        .or((format == Format::Gemini).then_some("user"))
        .context("message has no role")?;
    let mut parts = match (format, role) {
        (Format::Chat, "tool") => vec![Part::Result {
            id: text(m, "tool_call_id")?.into(),
            output: strings(&read_parts(format, &m["content"])?)?,
        }],
        (Format::Gemini, _) => read_parts(format, &m["parts"])?,
        (Format::Chat, "assistant") if m["content"].is_null() => vec![],
        _ => read_parts(format, &m["content"])?,
    };
    if format == Format::Chat
        && let Some(calls) = m.get("tool_calls").and_then(Value::as_array)
    {
        for call in calls {
            ensure!(call["type"] == "function", "unsupported Chat tool call");
            let f = &call["function"];
            parts.push(Part::Call {
                id: text(call, "id")?.into(),
                name: text(f, "name")?.into(),
                args: serde_json::from_str(text(f, "arguments")?)?,
            });
        }
    }
    let role = match role {
        "system" | "developer" | "user" | "assistant" => role,
        "model" if format == Format::Gemini => "assistant",
        "tool" if format == Format::Chat => "user",
        _ => bail!("unsupported message role {role}"),
    };
    Ok(Turn {
        role: role.into(),
        parts,
    })
}

fn write_turns(format: Format, turns: &[Turn]) -> Result<Vec<Value>> {
    let mut out = vec![];
    let mut call_names = HashMap::new();
    for turn in turns {
        let role = &turn.role;
        let mut parts = vec![];
        let mut calls = vec![];
        for p in &turn.parts {
            match p {
            Part::Text(t)=>parts.push(match format {
                Format::Chat=>json!(t),Format::Responses=>json!({"type":if role=="assistant"{"output_text"}else{"input_text"},"text":t}),Format::Messages=>json!({"type":"text","text":t}),Format::Gemini=>json!({"text":t})}),
            Part::Call{id,name,args}=>{
                ensure!(role=="assistant" && args.is_object(),"invalid function call");call_names.insert(id,name);
                match format {
                    Format::Chat=>calls.push(json!({"id":id,"type":"function","function":{"name":name,"arguments":args.to_string()}})),
                    Format::Messages=>parts.push(json!({"type":"tool_use","id":id,"name":name,"input":args})),
                    Format::Gemini=>parts.push(json!({"functionCall":{"id":id,"name":name,"args":args}})),
                    Format::Responses=>{if !parts.is_empty(){out.push(json!({"type":"message","role":role,"content":std::mem::take(&mut parts)}));}out.push(json!({"type":"function_call","call_id":id,"name":name,"arguments":args.to_string()}));}
                }
            }
            Part::Result{id,output}=>{
                match format {
                    Format::Chat|Format::Responses=>{
                        if !parts.is_empty(){if format==Format::Chat{out.push(json!({"role":role,"content":joined(&parts)}));}else{out.push(json!({"type":"message","role":role,"content":parts}));}parts.clear();}
                        out.push(if format==Format::Chat {json!({"role":"tool","tool_call_id":id,"content":output})}else{json!({"type":"function_call_output","call_id":id,"output":output})});
                    }
                    Format::Messages=>parts.push(json!({"type":"tool_result","tool_use_id":id,"content":output})),
                    Format::Gemini=>{let name=call_names.get(id).context("function response has no matching call")?;let data=serde_json::from_str::<Value>(output).ok().filter(Value::is_object).unwrap_or_else(||json!({"output":output}));parts.push(json!({"functionResponse":{"id":id,"name":name,"response":data}}));}
                }
            }
        }
        }
        if parts.is_empty() && calls.is_empty() {
            continue;
        }
        out.push(match format {
            Format::Chat => {
                let mut message = json!({"role":role,"content":joined(&parts)});
                if !calls.is_empty() {
                    message["tool_calls"] = json!(calls);
                }
                message
            }
            Format::Responses => json!({"type":"message","role":role,"content":parts}),
            Format::Messages => json!({"role":role,"content":parts}),
            Format::Gemini => {
                json!({"role":if role=="assistant"{"model"}else{"user"},"parts":parts})
            }
        });
    }
    Ok(out)
}

fn joined(parts: &[Value]) -> String {
    parts.iter().filter_map(Value::as_str).collect()
}
