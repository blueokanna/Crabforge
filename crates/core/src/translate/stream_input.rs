use super::{Format, Part, array, read_parts, read_turn, text};
use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

#[derive(Default)]
struct Function {
    id: String,
    name: String,
    arguments: String,
    input: Option<Value>,
}

impl Function {
    fn finish(self) -> Result<Part> {
        ensure!(
            !self.id.is_empty() && !self.name.is_empty(),
            "incomplete streamed function call"
        );
        let args = if self.arguments.is_empty() {
            self.input.context("missing function arguments")?
        } else {
            serde_json::from_str(&self.arguments)?
        };
        ensure!(args.is_object(), "expected function arguments object");
        Ok(Part::Call {
            id: self.id,
            name: self.name,
            args,
        })
    }
}

#[derive(Default)]
pub(super) struct Decoder {
    functions: BTreeMap<u64, Function>,
    response_text: HashSet<(u64, u64)>,
    response_items: HashSet<u64>,
    pub input: u64,
    pub output: u64,
    pub reason: Option<String>,
    pub done: bool,
}

fn index(v: &Value, key: &str) -> Result<u64> {
    v[key].as_u64().context("missing stream index")
}

impl Decoder {
    fn usage(&mut self, v: &Value, input: &str, output: &str) {
        if let Some(n) = v[input].as_u64() {
            self.input = n;
        }
        if let Some(n) = v[output].as_u64() {
            self.output = n;
        }
    }

    pub fn read(&mut self, from: Format, data: &str) -> Result<Vec<Part>> {
        if from == Format::Chat && data == "[DONE]" {
            self.done = true;
            return Ok(vec![]);
        }
        let v: Value = serde_json::from_str(data)?;
        ensure!(v.get("error").is_none(), "upstream returned a stream error");
        match from {
            Format::Chat => self.chat(&v),
            Format::Messages => self.messages(&v),
            Format::Responses => self.responses(&v),
            Format::Gemini => self.gemini(&v),
        }
    }

    fn chat(&mut self, v: &Value) -> Result<Vec<Part>> {
        self.usage(&v["usage"], "prompt_tokens", "completion_tokens");
        let mut parts = vec![];
        for choice in array(v, "choices")? {
            ensure!(
                choice["index"].as_u64().unwrap_or(0) == 0,
                "multiple stream choices"
            );
            let delta = &choice["delta"];
            ensure!(delta["refusal"].is_null(), "upstream refused the request");
            if let Some(t) = delta["content"].as_str() {
                parts.push(Part::Text(t.into()));
            }
            if let Some(calls) = delta["tool_calls"].as_array() {
                for call in calls {
                    let f = self.functions.entry(index(call, "index")?).or_default();
                    if let Some(s) = call["id"].as_str() {
                        f.id.push_str(s);
                    }
                    if let Some(s) = call["function"]["name"].as_str() {
                        f.name.push_str(s);
                    }
                    if let Some(s) = call["function"]["arguments"].as_str() {
                        f.arguments.push_str(s);
                    }
                }
            }
            if let Some(reason) = choice["finish_reason"].as_str() {
                self.reason = Some(reason.into());
                for f in std::mem::take(&mut self.functions).into_values() {
                    parts.push(f.finish()?);
                }
            }
        }
        Ok(parts)
    }

    fn messages(&mut self, v: &Value) -> Result<Vec<Part>> {
        let mut parts = vec![];
        match text(v, "type")? {
            "message_start" => self.usage(&v["message"]["usage"], "input_tokens", "output_tokens"),
            "content_block_start" => {
                let block = &v["content_block"];
                match text(block, "type")? {
                    "text" => parts.push(Part::Text(text(block, "text")?.into())),
                    "tool_use" => {
                        let f = Function {
                            id: text(block, "id")?.into(),
                            name: text(block, "name")?.into(),
                            input: block.get("input").cloned(),
                            arguments: String::new(),
                        };
                        ensure!(
                            self.functions.insert(index(v, "index")?, f).is_none(),
                            "duplicate function block"
                        );
                    }
                    other => bail!("unsupported streamed block {other}"),
                }
            }
            "content_block_delta" => {
                let delta = &v["delta"];
                match text(delta, "type")? {
                    "text_delta" => parts.push(Part::Text(text(delta, "text")?.into())),
                    "input_json_delta" => self
                        .functions
                        .get_mut(&index(v, "index")?)
                        .context("function delta without a start")?
                        .arguments
                        .push_str(text(delta, "partial_json")?),
                    other => bail!("unsupported streamed delta {other}"),
                }
            }
            "content_block_stop" => {
                if let Some(f) = self.functions.remove(&index(v, "index")?) {
                    parts.push(f.finish()?);
                }
            }
            "message_delta" => {
                self.usage(&v["usage"], "input_tokens", "output_tokens");
                self.reason = Some(text(&v["delta"], "stop_reason")?.into());
            }
            "message_stop" => self.done = true,
            "ping" => (),
            other => bail!("unsupported Messages event {other}"),
        }
        Ok(parts)
    }

    fn completed_item(&mut self, i: u64, item: &Value, parts: &mut Vec<Part>) -> Result<()> {
        if !self.response_items.insert(i) {
            return Ok(());
        }
        let turn = read_turn(Format::Responses, item)?;
        for (j, part) in turn.parts.into_iter().enumerate() {
            if !self.response_text.contains(&(i, j as u64)) {
                parts.push(part);
            }
        }
        Ok(())
    }

    fn responses(&mut self, v: &Value) -> Result<Vec<Part>> {
        let mut parts = vec![];
        match text(v, "type")? {
            "response.output_text.delta" => {
                self.response_text
                    .insert((index(v, "output_index")?, index(v, "content_index")?));
                parts.push(Part::Text(text(v, "delta")?.into()));
            }
            "response.output_item.done" => {
                self.completed_item(index(v, "output_index")?, &v["item"], &mut parts)?
            }
            "response.completed" | "response.incomplete" => {
                let reply = &v["response"];
                for (i, item) in array(reply, "output")?.iter().enumerate() {
                    self.completed_item(i as u64, item, &mut parts)?;
                }
                self.usage(&reply["usage"], "input_tokens", "output_tokens");
                self.reason = Some(
                    match text(reply, "status")? {
                        "completed" => "stop",
                        "incomplete" => text(&reply["incomplete_details"], "reason")?,
                        other => bail!("upstream response status {other}"),
                    }
                    .into(),
                );
                self.done = true;
            }
            "response.created"
            | "response.in_progress"
            | "response.output_item.added"
            | "response.content_part.added"
            | "response.content_part.done"
            | "response.output_text.done"
            | "response.function_call_arguments.delta"
            | "response.function_call_arguments.done" => (),
            other => bail!("unsupported Responses event {other}"),
        }
        Ok(parts)
    }

    fn gemini(&mut self, v: &Value) -> Result<Vec<Part>> {
        self.usage(
            &v["usageMetadata"],
            "promptTokenCount",
            "candidatesTokenCount",
        );
        let mut parts = vec![];
        if let Some(candidates) = v["candidates"].as_array() {
            for candidate in candidates {
                ensure!(
                    candidate["index"].as_u64().unwrap_or(0) == 0,
                    "multiple Gemini candidates"
                );
                if let Some(content) = candidate["content"].get("parts") {
                    parts.extend(read_parts(Format::Gemini, content)?);
                }
                if let Some(reason) = candidate["finishReason"].as_str() {
                    self.reason = Some(reason.into());
                }
            }
        }
        Ok(parts)
    }

    pub fn end(&self, from: Format) -> Result<&str> {
        ensure!(
            self.functions.is_empty()
                && (self.done || matches!(from, Format::Gemini | Format::Chat)),
            "upstream stream interrupted"
        );
        self.reason
            .as_deref()
            .context("stream has no finish reason")
    }
}
