use super::{Format, Part, array, field, read_reply_parts, read_turn, text};
use crate::json::unsigned;
use anyhow::{Context, Result, bail, ensure};
use nextjson::Value;
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
            nextjson::from_str(&self.arguments).map_err(anyhow::Error::msg)?
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
    unsigned(field(v, key)).context("missing stream index")
}

impl Decoder {
    fn usage(&mut self, v: &Value, input: &str, output: &str) {
        if let Some(n) = unsigned(field(v, input)) {
            self.input = n;
        }
        if let Some(n) = unsigned(field(v, output)) {
            self.output = n;
        }
    }

    pub fn read(&mut self, from: Format, data: &str) -> Result<Vec<Part>> {
        if from == Format::Chat && data == "[DONE]" {
            self.done = true;
            return Ok(vec![]);
        }
        let v: Value = nextjson::from_str(data).map_err(anyhow::Error::msg)?;
        ensure!(v.get("error").is_none(), "upstream returned a stream error");
        match from {
            Format::Chat => self.chat(&v),
            Format::Messages => self.messages(&v),
            Format::Responses => self.responses(&v),
            Format::Gemini => self.gemini(&v),
        }
    }

    fn chat(&mut self, v: &Value) -> Result<Vec<Part>> {
        self.usage(field(v, "usage"), "prompt_tokens", "completion_tokens");
        let mut parts = vec![];
        for choice in array(v, "choices")? {
            ensure!(
                unsigned(field(choice, "index")).unwrap_or(0) == 0,
                "multiple stream choices"
            );
            let delta = field(choice, "delta");
            ensure!(
                field(delta, "refusal").is_null(),
                "upstream refused the request"
            );
            if let Some(t) = field(delta, "content").as_str() {
                parts.push(Part::Text(t.into()));
            }
            if let Some(calls) = field(delta, "tool_calls").as_array() {
                for call in calls {
                    let f = self.functions.entry(index(call, "index")?).or_default();
                    if let Some(s) = field(call, "id").as_str() {
                        f.id.push_str(s);
                    }
                    if let Some(s) = call.pointer("/function/name").and_then(Value::as_str) {
                        f.name.push_str(s);
                    }
                    if let Some(s) = call.pointer("/function/arguments").and_then(Value::as_str) {
                        f.arguments.push_str(s);
                    }
                }
            }
            if let Some(reason) = field(choice, "finish_reason").as_str() {
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
            "message_start" => self.usage(
                v.pointer("/message/usage").unwrap_or(&Value::Null),
                "input_tokens",
                "output_tokens",
            ),
            "content_block_start" => {
                let block = field(v, "content_block");
                match text(block, "type")? {
                    "text" => parts.push(Part::Text(text(block, "text")?.into())),
                    "thinking" | "redacted_thinking" => (),
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
                let delta = field(v, "delta");
                match text(delta, "type")? {
                    "text_delta" => parts.push(Part::Text(text(delta, "text")?.into())),
                    "thinking_delta" | "signature_delta" => (),
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
                self.usage(field(v, "usage"), "input_tokens", "output_tokens");
                self.reason = Some(text(field(v, "delta"), "stop_reason")?.into());
            }
            "message_stop" => self.done = true,
            "ping" => (),
            other => bail!("unsupported Messages event {other}"),
        }
        Ok(parts)
    }

    fn completed_item(&mut self, i: u64, item: &Value, parts: &mut Vec<Part>) -> Result<()> {
        if !self.response_items.insert(i) || field(item, "type").as_str() == Some("reasoning") {
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
                self.completed_item(index(v, "output_index")?, field(v, "item"), &mut parts)?
            }
            "response.completed" | "response.incomplete" => {
                let reply = field(v, "response");
                for (i, item) in array(reply, "output")?.iter().enumerate() {
                    self.completed_item(i as u64, item, &mut parts)?;
                }
                self.usage(field(reply, "usage"), "input_tokens", "output_tokens");
                self.reason = Some(
                    match text(reply, "status")? {
                        "completed" => "stop",
                        "incomplete" => text(field(reply, "incomplete_details"), "reason")?,
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
            | "response.function_call_arguments.done"
            | "response.reasoning_summary_part.added"
            | "response.reasoning_summary_part.done"
            | "response.reasoning_summary_text.delta"
            | "response.reasoning_summary_text.done"
            | "response.reasoning_text.delta"
            | "response.reasoning_text.done" => (),
            other => bail!("unsupported Responses event {other}"),
        }
        Ok(parts)
    }

    fn gemini(&mut self, v: &Value) -> Result<Vec<Part>> {
        self.usage(
            field(v, "usageMetadata"),
            "promptTokenCount",
            "candidatesTokenCount",
        );
        let mut parts = vec![];
        if let Some(candidates) = field(v, "candidates").as_array() {
            for candidate in candidates {
                ensure!(
                    unsigned(field(candidate, "index")).unwrap_or(0) == 0,
                    "multiple Gemini candidates"
                );
                if let Some(content) = candidate.pointer("/content/parts") {
                    parts.extend(read_reply_parts(Format::Gemini, content)?);
                }
                if let Some(reason) = field(candidate, "finishReason").as_str() {
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
