use super::{Format, Part};
use anyhow::{Result, bail};
use nextjson::{Value, json};
use std::{
    fmt::Write,
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) struct Encoder {
    format: Format,
    model: String,
    id: String,
    created: u64,
    sequence: u64,
    index: usize,
    calls: usize,
    open: bool,
    text: String,
    output: Vec<Value>,
    frames: String,
}

impl Encoder {
    pub fn new(format: Format, model: &str) -> Result<Self> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        Ok(Self {
            format,
            model: model.into(),
            id: format!("cf_{}", now.as_nanos()),
            created: now.as_secs(),
            sequence: 0,
            index: 0,
            calls: 0,
            open: false,
            text: String::new(),
            output: vec![],
            frames: String::new(),
        })
    }

    pub fn drain(&mut self) -> String {
        std::mem::take(&mut self.frames)
    }

    fn data(&mut self, data: Value) {
        writeln!(self.frames, "data: {data}\n").unwrap();
    }

    fn event(&mut self, name: &str, mut data: Value) {
        let object = data.as_object_mut().expect("event payload is an object");
        object.insert("type".into(), name.into());
        if self.format == Format::Responses {
            object.insert("sequence_number".into(), self.sequence.into());
            self.sequence += 1;
        }
        writeln!(self.frames, "event: {name}").unwrap();
        self.data(data);
    }

    fn chat(&mut self, choices: Value, usage: Option<Value>) {
        let mut chunk = json!({"id":format!("chatcmpl-{}",self.id),"object":"chat.completion.chunk","created":self.created,"model":self.model,"choices":choices});
        if let Some(usage) = usage {
            chunk
                .as_object_mut()
                .expect("chat chunk")
                .insert("usage".into(), usage);
        }
        self.data(chunk);
    }

    fn response(&self, status: &str, output: Vec<Value>, usage: Value, incomplete: Value) -> Value {
        json!({"id":format!("resp_{}",self.id),"object":"response","created_at":self.created,"model":self.model,"status":status,"error":null,"incomplete_details":incomplete,"output":output,"usage":usage})
    }

    pub fn start(&mut self, input: u64) {
        match self.format {
            Format::Chat => self.chat(json!([{"index":0,"delta":{"role":"assistant"},"finish_reason":null}]),None),
            Format::Messages => self.event("message_start",json!({"message":{"id":format!("msg_{}",self.id),"type":"message","role":"assistant","model":self.model,"content":[],"stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":input,"output_tokens":0}}})),
            Format::Responses => {
                let response = self.response("in_progress",vec![],Value::Null,Value::Null);
                self.event("response.created",json!({"response":response}));
                self.event("response.in_progress",json!({"response":response}));
            }
            Format::Gemini => (),
        }
    }

    fn item_id(&self, prefix: &str) -> String {
        format!("{prefix}_{}_{}", self.id, self.index)
    }

    fn text_part(text: &str) -> Value {
        json!({"type":"output_text","text":text,"annotations":[]})
    }

    fn message(&self, content: Vec<Value>, status: &str) -> Value {
        json!({"id":self.item_id("msg"),"type":"message","role":"assistant","status":status,"content":content})
    }

    fn gemini_part(&mut self, part: Value) {
        self.data(json!({"responseId":self.id,"modelVersion":self.model,"candidates":[{"index":0,"content":{"role":"model","parts":[part]}}]}));
    }

    fn text_delta(&mut self, value: String) {
        if value.is_empty() {
            return;
        }
        let id = self.item_id("msg");
        if !self.open {
            match self.format {
                Format::Messages => self.event(
                    "content_block_start",
                    json!({"index":self.index,"content_block":{"type":"text","text":""}}),
                ),
                Format::Responses => {
                    self.event("response.output_item.added",json!({"output_index":self.index,"item":self.message(vec![],"in_progress")}));
                    self.event("response.content_part.added",json!({"item_id":id,"output_index":self.index,"content_index":0,"part":Self::text_part("")}));
                }
                _ => (),
            }
            self.open = true;
        }
        match self.format {
            Format::Chat => self.chat(
                json!([{"index":0,"delta":{"content":value},"finish_reason":null}]),
                None,
            ),
            Format::Messages => self.event(
                "content_block_delta",
                json!({"index":self.index,"delta":{"type":"text_delta","text":value}}),
            ),
            Format::Responses => {
                self.text.push_str(&value);
                self.event(
                    "response.output_text.delta",
                    json!({"item_id":id,"output_index":self.index,"content_index":0,"delta":value}),
                );
            }
            Format::Gemini => self.gemini_part(json!({"text":value})),
        }
    }

    fn close_text(&mut self) {
        if !self.open {
            return;
        }
        match self.format {
            Format::Messages => self.event("content_block_stop", json!({"index":self.index})),
            Format::Responses => {
                let text = std::mem::take(&mut self.text);
                let id = self.item_id("msg");
                self.event(
                    "response.output_text.done",
                    json!({"item_id":id,"output_index":self.index,"content_index":0,"text":text}),
                );
                let part = Self::text_part(&text);
                self.event(
                    "response.content_part.done",
                    json!({"item_id":id,"output_index":self.index,"content_index":0,"part":part}),
                );
                let item = self.message(vec![part], "completed");
                self.event(
                    "response.output_item.done",
                    json!({"output_index":self.index,"item":item}),
                );
                self.output.push(item);
            }
            _ => (),
        }
        self.open = false;
        self.index += 1;
    }

    pub fn part(&mut self, part: Part) -> Result<()> {
        let (call, name, args) = match part {
            Part::Text(value) => {
                self.text_delta(value);
                return Ok(());
            }
            Part::Call { id, name, args } => (id, name, args),
            Part::Result { .. } => bail!("unexpected function result from model"),
            Part::Attachment(_) => bail!("unexpected attachment from model"),
        };
        self.close_text();
        let arguments = args.to_string();
        match self.format {
            Format::Chat => self.chat(json!([{"index":0,"delta":{"tool_calls":[{"index":self.calls,"id":call,"type":"function","function":{"name":name,"arguments":arguments}}]},"finish_reason":null}]),None),
            Format::Gemini => self.gemini_part(json!({"functionCall":{"id":call,"name":name,"args":args}})),
            Format::Messages => {
                self.event("content_block_start",json!({"index":self.index,"content_block":{"type":"tool_use","id":call,"name":name,"input":{}}}));
                self.event("content_block_delta",json!({"index":self.index,"delta":{"type":"input_json_delta","partial_json":arguments}}));
                self.event("content_block_stop",json!({"index":self.index}));
            }
            Format::Responses => {
                let id = self.item_id("fc");
                let mut item = json!({"id":id,"type":"function_call","call_id":call,"name":name,"arguments":"","status":"in_progress"});
                self.event("response.output_item.added",json!({"output_index":self.index,"item":item}));
                self.event("response.function_call_arguments.delta",json!({"output_index":self.index,"item_id":id,"delta":arguments}));
                self.event("response.function_call_arguments.done",json!({"output_index":self.index,"item_id":id,"name":name,"arguments":arguments}));
                let object = item.as_object_mut().expect("function call item");
                object.insert("arguments".into(), arguments.into());
                object.insert("status".into(), "completed".into());
                self.event("response.output_item.done",json!({"output_index":self.index,"item":item}));
                self.output.push(item);
            }
        }
        self.calls += 1;
        self.index += 1;
        Ok(())
    }

    pub fn finish(&mut self, reason: &str, input: u64, output: u64) -> Result<()> {
        self.close_text();
        let (chat, messages, gemini, incomplete) = match reason {
            "length" | "max_tokens" | "MAX_TOKENS" | "max_output_tokens" => (
                "length",
                "max_tokens",
                "MAX_TOKENS",
                Some("max_output_tokens"),
            ),
            "content_filter" | "SAFETY" | "refusal" => (
                "content_filter",
                "refusal",
                "SAFETY",
                Some("content_filter"),
            ),
            "stop" | "end_turn" | "STOP" | "tool_calls" | "tool_use" | "stop_sequence" => {
                if self.calls > 0 {
                    ("tool_calls", "tool_use", "STOP", None)
                } else {
                    ("stop", "end_turn", "STOP", None)
                }
            }
            other => bail!("unsupported stream finish reason {other}"),
        };
        let total = input.saturating_add(output);
        match self.format {
            Format::Chat => {
                self.chat(json!([{"index":0,"delta":{},"finish_reason":chat}]), None);
                self.chat(json!([]),Some(json!({"prompt_tokens":input,"completion_tokens":output,"total_tokens":total})));
                self.frames.push_str("data: [DONE]\n\n");
            }
            Format::Messages => {
                self.event("message_delta",json!({"delta":{"stop_reason":messages,"stop_sequence":null},"usage":{"input_tokens":input,"output_tokens":output}}));
                self.event("message_stop",json!({}));
            }
            Format::Responses => {
                let status = if incomplete.is_some() {"incomplete"} else {"completed"};
                let items = std::mem::take(&mut self.output);
                let reply = self.response(status,items,json!({"input_tokens":input,"output_tokens":output,"total_tokens":total}),json!(incomplete.map(|r|json!({"reason":r}))));
                self.event(&format!("response.{status}"),json!({"response":reply}));
            }
            Format::Gemini => self.data(json!({"responseId":self.id,"modelVersion":self.model,"candidates":[{"index":0,"finishReason":gemini}],"usageMetadata":{"promptTokenCount":input,"candidatesTokenCount":output,"totalTokenCount":total}})),
        }
        Ok(())
    }
}
