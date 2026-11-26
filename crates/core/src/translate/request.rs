use super::*;

pub struct Converted {
    pub body: Vec<u8>,
    pub stream: bool,
}

fn read_tools(format: Format, request: &Value) -> Result<Vec<Value>> {
    let Some(tools) = request.get("tools") else {
        return Ok(vec![]);
    };
    let mut definitions = vec![];
    for tool in tools.as_array().context("expected tools array")? {
        let schema = match format {
            Format::Chat => {
                ensure!(tool["type"] == "function", "unsupported hosted tool");
                &tool["function"]
            }
            Format::Responses => {
                ensure!(tool["type"] == "function", "unsupported hosted tool");
                tool
            }
            Format::Messages => {
                ensure!(
                    tool.get("type").is_none() || tool["type"] == "custom",
                    "unsupported hosted tool"
                );
                tool
            }
            Format::Gemini => {
                keys(tool, &["functionDeclarations"])?;
                definitions.extend(array(tool, "functionDeclarations")?.iter().cloned());
                continue;
            }
        };
        ensure!(
            schema["strict"] != true,
            "strict tool validation requires a native endpoint"
        );
        let mut def = json!({"name":text(schema,"name")?,"parameters":schema.get("input_schema").or_else(||schema.get("parameters")).cloned().unwrap_or_else(||json!({"type":"object","properties":{}}))});
        if let Some(d) = schema.get("description") {
            def["description"] = d.clone();
        }
        definitions.push(def);
    }
    Ok(definitions)
}

fn write_tools(format: Format, defs: Vec<Value>) -> Value {
    if format == Format::Gemini {
        return json!([{"functionDeclarations":defs}]);
    }
    Value::Array(
        defs.into_iter()
            .map(|mut d| match format {
                Format::Chat => json!({"type":"function","function":d}),
                Format::Responses => {
                    d["type"] = json!("function");
                    d
                }
                Format::Messages => {
                    d["input_schema"] = d
                        .as_object_mut()
                        .unwrap()
                        .remove("parameters")
                        .unwrap_or_else(|| json!({"type":"object"}));
                    d
                }
                Format::Gemini => unreachable!(),
            })
            .collect(),
    )
}

fn choice(from: Format, to: Format, v: &Value) -> Result<Value> {
    let mode = match from {
        Format::Messages => text(v, "type")?,
        Format::Gemini => text(&v["functionCallingConfig"], "mode")?,
        _ => v
            .as_str()
            .or_else(|| v["type"].as_str())
            .context("invalid tool choice")?,
    };
    let name = match from {
        Format::Messages | Format::Responses => v["name"].as_str(),
        Format::Chat => v["function"]["name"].as_str(),
        Format::Gemini => {
            let names = &v["functionCallingConfig"]["allowedFunctionNames"];
            ensure!(
                names.is_null() || names.as_array().is_some_and(|n| n.len() == 1),
                "cannot translate multiple allowed functions"
            );
            names[0].as_str()
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
    let v: Value = serde_json::from_slice(bytes)?;
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
        v["store"].is_null() || v["store"] == false,
        "translated requests only support store: false"
    );
    ensure!(
        v["n"].is_null() || v["n"] == 1,
        "only one translated completion is supported"
    );
    if !v["stream_options"].is_null() {
        keys(&v["stream_options"], &["include_usage"])?;
    }
    let stream = stream_in_path || v["stream"] == true;
    let mut turns = vec![];
    let system = match from {
        Format::Messages => v.get("system"),
        Format::Responses => v.get("instructions"),
        Format::Gemini => v["systemInstruction"].get("parts"),
        _ => None,
    };
    if let Some(s) = system.filter(|s| !s.is_null()) {
        turns.push(Turn {
            role: "system".into(),
            parts: read_parts(from, s)?,
        });
    }
    let input = &v[match from {
        Format::Responses => "input",
        Format::Gemini => "contents",
        _ => "messages",
    }];
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
    let mut out = json!({});
    if stream && to != Format::Gemini {
        out["stream"] = json!(true);
        if to == Format::Chat {
            out["stream_options"] = json!({"include_usage": true});
        }
    }
    if to != Format::Gemini {
        out["model"] = json!(model);
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
                out["systemInstruction"] = json!({"parts":[{"text":instructions.join("\n\n")}]});
            } else {
                out["system"] = json!(instructions.join("\n\n"));
            }
        }
    }
    let messages = write_turns(to, &turns)?;
    ensure!(!messages.is_empty(), "empty conversation");
    out[match to {
        Format::Responses => "input",
        Format::Gemini => "contents",
        _ => "messages",
    }] = json!(messages);
    let src = if from == Format::Gemini {
        &v["generationConfig"]
    } else {
        &v
    };
    if from == Format::Gemini && !src.is_null() {
        keys(
            src,
            &["temperature", "topP", "maxOutputTokens", "stopSequences"],
        )?;
    }
    let mut options = json!({});
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
            .find_map(|k| src.get(*k))
            .filter(|v| !v.is_null())
        {
            ensure!(
                dest != "stop" || to != Format::Responses,
                "Responses cannot preserve stop sequences"
            );
            options[dest] = if dest.starts_with("stop") && to != Format::Chat && val.is_string() {
                json!([val])
            } else {
                val.clone()
            };
        }
    }
    if to == Format::Messages && options.get("max_tokens").is_none() {
        options["max_tokens"] = json!(4096);
    }
    if to == Format::Gemini {
        if !options.as_object().unwrap().is_empty() {
            out["generationConfig"] = options;
        }
    } else {
        out.as_object_mut()
            .unwrap()
            .append(options.as_object_mut().unwrap());
    }
    if to == Format::Responses {
        out["store"] = json!(false);
    }
    let defs = read_tools(from, &v)?;
    if !defs.is_empty() {
        out["tools"] = write_tools(to, defs);
    }
    if let Some(c) = v
        .get(if from == Format::Gemini {
            "toolConfig"
        } else {
            "tool_choice"
        })
        .filter(|v| !v.is_null())
    {
        out[if to == Format::Gemini {
            "toolConfig"
        } else {
            "tool_choice"
        }] = choice(from, to, c)?;
    }
    Ok(Converted {
        body: serde_json::to_vec(&out)?,
        stream,
    })
}
