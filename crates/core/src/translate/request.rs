use super::*;

pub struct Converted {
    pub body: Vec<u8>,
    pub stream: bool,
}

fn read_tools(format: Format, request: &Value) -> Result<Vec<Map>> {
    let Some(tools) = request.get("tools") else {
        return Ok(vec![]);
    };
    let mut definitions = vec![];
    for tool in tools.as_array().context("expected tools array")? {
        let schema = match format {
            Format::Chat => {
                ensure!(text(tool, "type")? == "function", "unsupported hosted tool");
                field(tool, "function")
            }
            Format::Responses => {
                ensure!(text(tool, "type")? == "function", "unsupported hosted tool");
                tool
            }
            Format::Messages => {
                ensure!(
                    tool.get("type").is_none() || field(tool, "type").as_str() == Some("custom"),
                    "unsupported hosted tool"
                );
                tool
            }
            Format::Gemini => {
                keys(tool, &["functionDeclarations"])?;
                for declaration in array(tool, "functionDeclarations")? {
                    definitions.push(
                        declaration
                            .as_object()
                            .context("expected a function declaration object")?
                            .clone(),
                    );
                }
                continue;
            }
        };
        ensure!(
            field(schema, "strict").as_bool() != Some(true),
            "strict tool validation requires a native endpoint"
        );
        let mut def = Map::new();
        def.insert("name".into(), text(schema, "name")?.into());
        def.insert(
            "parameters".into(),
            schema
                .get("input_schema")
                .or_else(|| schema.get("parameters"))
                .cloned()
                .unwrap_or_else(|| json!({"type":"object","properties":{}})),
        );
        if let Some(d) = schema.get("description") {
            def.insert("description".into(), d.clone());
        }
        definitions.push(def);
    }
    Ok(definitions)
}

fn write_tools(format: Format, defs: Vec<Map>) -> Value {
    if format == Format::Gemini {
        return json!([{"functionDeclarations":defs}]);
    }
    Value::Array(
        defs.into_iter()
            .map(|mut d| match format {
                Format::Chat => json!({"type":"function","function":d}),
                Format::Responses => {
                    d.insert("type".into(), "function".into());
                    d.into()
                }
                Format::Messages => {
                    let parameters = d
                        .remove("parameters")
                        .unwrap_or_else(|| json!({"type":"object"}));
                    d.insert("input_schema".into(), parameters);
                    d.into()
                }
                Format::Gemini => unreachable!(),
            })
            .collect(),
    )
}

fn choice(from: Format, to: Format, v: &Value) -> Result<Value> {
    let mode = match from {
        Format::Messages => text(v, "type")?,
        Format::Gemini => text(field(v, "functionCallingConfig"), "mode")?,
        _ => v
            .as_str()
            .or_else(|| field(v, "type").as_str())
            .context("invalid tool choice")?,
    };
    let name = match from {
        Format::Messages | Format::Responses => field(v, "name").as_str(),
        Format::Chat => v.pointer("/function/name").and_then(Value::as_str),
        Format::Gemini => {
            let names = v
                .pointer("/functionCallingConfig/allowedFunctionNames")
                .unwrap_or(&Value::Null);
            ensure!(
                names.is_null() || names.as_array().is_some_and(|n| n.len() == 1),
                "cannot translate multiple allowed functions"
            );
            names
                .as_array()
                .and_then(|v| v.first())
                .and_then(Value::as_str)
        }
    };
    let mode = match mode {
        "auto" | "AUTO" => "auto",
        "any" | "required" | "ANY" => "required",
        "none" | "NONE" => "none",
        "function" | "tool" if name.is_some() => "required",
        _ => bail!("unsupported tool choice"),
    };
    Ok(match (to, name) {
        (Format::Chat, Some(n)) => json!({"type":"function","function":{"name":n}}),
        (Format::Responses, Some(n)) => json!({"type":"function","name":n}),
        (Format::Messages, Some(n)) => json!({"type":"tool","name":n}),
        (Format::Gemini, Some(n)) => {
            json!({"functionCallingConfig":{"mode":"ANY","allowedFunctionNames":[n]}})
        }
        (Format::Messages, None) => json!({"type":if mode=="required"{"any"}else{mode}}),
        (Format::Gemini, None) => {
            json!({"functionCallingConfig":{"mode":if mode=="required"{"ANY".into()}else{mode.to_uppercase()}}})
        }
        (_, None) => json!(mode),
    })
}

pub fn convert_request(
    from: Format,
    to: Format,
    bytes: &[u8],
    model: &str,
    stream_in_path: bool,
) -> Result<Converted> {
    let v: Value = nextjson::from_slice(bytes).map_err(anyhow::Error::msg)?;
    let allowed: &[&str] = match from {
        Format::Chat => &[
            "model",
            "messages",
            "stream",
            "tools",
            "tool_choice",
            "temperature",
            "top_p",
            "max_tokens",
            "max_completion_tokens",
            "stop",
            "stream_options",
            "store",
            "n",
        ],
        Format::Messages => &[
            "model",
            "system",
            "messages",
            "stream",
            "tools",
            "tool_choice",
            "temperature",
            "top_p",
            "max_tokens",
            "stop_sequences",
            "metadata",
        ],
        Format::Responses => &[
            "model",
            "instructions",
            "input",
            "stream",
            "tools",
            "tool_choice",
            "temperature",
            "top_p",
            "max_output_tokens",
            "store",
        ],
        Format::Gemini => &[
            "contents",
            "systemInstruction",
            "generationConfig",
            "tools",
            "toolConfig",
        ],
    };
    keys(&v, allowed)?;
    ensure!(
        matches!(field(&v, "store"), Value::Null | Value::Bool(false)),
        "translated requests only support store: false"
    );
    ensure!(
        field(&v, "n").is_null() || field(&v, "n") == &Value::from(1_u64),
        "only one translated completion is supported"
    );
    if let Some(options) = v.get("stream_options").filter(|v| !v.is_null()) {
        keys(options, &["include_usage"])?;
    }
    let stream = stream_in_path || field(&v, "stream").as_bool() == Some(true);
    let mut turns = vec![];
    let system = match from {
        Format::Messages => v.get("system"),
        Format::Responses => v.get("instructions"),
        Format::Gemini => v.pointer("/systemInstruction/parts"),
        _ => None,
    };
    if let Some(s) = system.filter(|s| !s.is_null()) {
        turns.push(Turn {
            role: "system".into(),
            parts: read_parts(from, s)?,
        });
    }
    let input = field(
        &v,
        match from {
            Format::Responses => "input",
            Format::Gemini => "contents",
            _ => "messages",
        },
    );
    if from == Format::Responses
        && let Some(t) = input.as_str()
    {
        turns.push(Turn {
            role: "user".into(),
            parts: vec![Part::Text(t.into())],
        });
    } else {
        for m in input.as_array().context("expected message array")? {
            turns.push(read_turn(from, m)?);
        }
    }
    let mut out = Map::new();
    if stream && to != Format::Gemini {
        out.insert("stream".into(), true.into());
        if to == Format::Chat {
            out.insert("stream_options".into(), json!({"include_usage": true}));
        }
    }
    if to != Format::Gemini {
        out.insert("model".into(), model.into());
    }
    if matches!(to, Format::Gemini | Format::Messages) {
        let leading = turns
            .iter()
            .take_while(|t| matches!(t.role.as_str(), "system" | "developer"))
            .count();
        let instructions = turns
            .drain(..leading)
            .map(|t| strings(&t.parts))
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            !turns
                .iter()
                .any(|t| matches!(t.role.as_str(), "system" | "developer")),
            "mid-conversation instructions require a native endpoint"
        );
        if !instructions.is_empty() {
            if to == Format::Gemini {
                out.insert(
                    "systemInstruction".into(),
                    json!({"parts":[{"text":instructions.join("\n\n")}]}),
                );
            } else {
                out.insert("system".into(), instructions.join("\n\n").into());
            }
        }
    }
    let messages = write_turns(to, &turns)?;
    ensure!(!messages.is_empty(), "empty conversation");
    let message_key = match to {
        Format::Responses => "input",
        Format::Gemini => "contents",
        _ => "messages",
    };
    out.insert(message_key.into(), Value::Array(messages));
    let src = if from == Format::Gemini {
        field(&v, "generationConfig")
    } else {
        &v
    };
    if from == Format::Gemini && !src.is_null() {
        keys(
            src,
            &["temperature", "topP", "maxOutputTokens", "stopSequences"],
        )?;
    }
    let mut options = Map::new();
    for (aliases, dest) in [
        (&["temperature"][..], "temperature"),
        (
            &["top_p", "topP"][..],
            if to == Format::Gemini {
                "topP"
            } else {
                "top_p"
            },
        ),
        (
            &[
                "max_tokens",
                "max_output_tokens",
                "max_completion_tokens",
                "maxOutputTokens",
            ][..],
            match to {
                Format::Responses => "max_output_tokens",
                Format::Gemini => "maxOutputTokens",
                _ => "max_tokens",
            },
        ),
        (
            &["stop", "stop_sequences", "stopSequences"][..],
            match to {
                Format::Messages => "stop_sequences",
                Format::Gemini => "stopSequences",
                _ => "stop",
            },
        ),
    ] {
        if let Some(val) = aliases
            .iter()
            .find_map(|k| src.get(k))
            .filter(|v| !v.is_null())
        {
            ensure!(
                dest != "stop" || to != Format::Responses,
                "Responses cannot preserve stop sequences"
            );
            let value = if dest.starts_with("stop") && to != Format::Chat && val.is_string() {
                json!([val])
            } else {
                val.clone()
            };
            options.insert(dest.into(), value);
        }
    }
    if to == Format::Messages && options.get("max_tokens").is_none() {
        options.insert("max_tokens".into(), 4096.into());
    }
    if to == Format::Gemini {
        if !options.is_empty() {
            out.insert("generationConfig".into(), options.into());
        }
    } else {
        out.extend(options);
    }
    if to == Format::Responses {
        out.insert("store".into(), false.into());
    }
    let defs = read_tools(from, &v)?;
    if !defs.is_empty() {
        out.insert("tools".into(), write_tools(to, defs));
    }
    if let Some(c) = v
        .get(if from == Format::Gemini {
            "toolConfig"
        } else {
            "tool_choice"
        })
        .filter(|v| !v.is_null())
    {
        let key = if to == Format::Gemini {
            "toolConfig"
        } else {
            "tool_choice"
        };
        out.insert(key.into(), choice(from, to, c)?);
    }
    Ok(Converted {
        body: nextjson::to_vec(&out).map_err(anyhow::Error::msg)?,
        stream,
    })
}
