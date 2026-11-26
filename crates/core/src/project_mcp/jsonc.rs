use anyhow::{Context, Result, ensure};
use nextjson::{
    Value,
    formats::{Format, Json5},
};
use std::{collections::HashSet, ops::Range};

const MAX_DEPTH: usize = 64;

pub(crate) struct Object {
    members: Vec<Member>,
    close: usize,
}

struct Member {
    key: String,
    start: usize,
    value: Range<usize>,
    comma: Option<usize>,
}

struct Cursor<'a> {
    source: &'a str,
    index: usize,
}

impl Cursor<'_> {
    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.index).copied()
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, byte: u8) -> Result<()> {
        ensure!(
            self.consume(byte),
            "expected {} in JSON configuration near byte {}",
            char::from(byte),
            self.index
        );
        Ok(())
    }

    fn skip(&mut self) -> Result<()> {
        loop {
            let rest = &self.source[self.index..];
            if let Some(comment) = rest.strip_prefix("//") {
                self.index += 2 + comment.find(['\r', '\n']).unwrap_or(comment.len());
            } else if let Some(comment) = rest.strip_prefix("/*") {
                self.index += 4 + comment.find("*/").context("unclosed JSON comment")?;
            } else {
                let spaces = rest.len() - rest.trim_start().len();
                if spaces == 0 {
                    return Ok(());
                }
                self.index += spaces;
            }
        }
    }

    fn string(&mut self, quote: u8) -> Result<()> {
        self.index += 1;
        while let Some(byte) = self.peek() {
            self.index += 1;
            if byte == quote {
                return Ok(());
            }
            if byte == b'\\' && self.peek().is_some() {
                self.index += 1;
            }
        }
        anyhow::bail!("unclosed JSON string")
    }

    fn atom(&mut self) -> Result<()> {
        let start = self.index;
        self.index += self.source[start..]
            .find(|ch: char| ch.is_whitespace() || ":,]}/".contains(ch))
            .unwrap_or(self.source.len() - start);
        ensure!(self.index > start, "missing JSON value");
        Ok(())
    }

    fn key(&mut self) -> Result<String> {
        let start = self.index;
        if let Some(quote @ (b'"' | b'\'')) = self.peek() {
            self.string(quote)?;
            let decoded: Value = Json5
                .decode(&self.source.as_bytes()[start..self.index])
                .map_err(anyhow::Error::msg)?;
            Ok(decoded
                .as_str()
                .context("expected JSON object key")?
                .to_owned())
        } else {
            self.atom()?;
            let property = &self.source[start..self.index];
            let decoded: Value = Json5
                .decode(format!("{{{property}:null}}").as_bytes())
                .map_err(anyhow::Error::msg)?;
            let (key, _) = decoded
                .as_object()
                .and_then(|object| object.iter().next())
                .context("invalid JSON object key")?;
            Ok(key.to_owned())
        }
    }

    fn value(&mut self, depth: usize) -> Result<()> {
        ensure!(
            depth <= MAX_DEPTH,
            "JSON configuration has more than {MAX_DEPTH} nested levels"
        );
        match self.peek() {
            Some(b'{') => {
                self.object(depth, false)?;
                Ok(())
            }
            Some(b'[') => {
                self.index += 1;
                self.skip()?;
                while !self.consume(b']') {
                    self.value(depth + 1)?;
                    self.skip()?;
                    if !self.consume(b',') {
                        return self.expect(b']');
                    }
                    self.skip()?;
                }
                Ok(())
            }
            Some(quote @ (b'"' | b'\'')) => self.string(quote),
            _ => self.atom(),
        }
    }

    fn object(&mut self, depth: usize, collect: bool) -> Result<Object> {
        self.expect(b'{')?;
        self.skip()?;
        let mut members: Vec<Member> = Vec::new();
        let mut keys = HashSet::new();
        while self.peek() != Some(b'}') {
            let start = self.index;
            let key = self.key()?;
            self.skip()?;
            self.expect(b':')?;
            self.skip()?;
            let value_start = self.index;
            self.value(depth + 1)?;
            let value = value_start..self.index;
            self.skip()?;
            let comma = self.peek().filter(|&byte| byte == b',').map(|_| self.index);
            if collect {
                ensure!(
                    keys.insert(key.clone()),
                    "duplicate JSON configuration key {key:?}"
                );
                members.push(Member {
                    key,
                    start,
                    value,
                    comma,
                });
            }
            if comma.is_none() {
                break;
            }
            self.index += 1;
            self.skip()?;
        }
        let close = self.index;
        self.expect(b'}')?;
        Ok(Object { members, close })
    }
}

pub(crate) fn parse(source: &str, path: &[&str]) -> Result<Option<Object>> {
    let mut cursor = Cursor {
        source,
        index: if source.starts_with('\u{feff}') { 3 } else { 0 },
    };
    cursor.skip()?;
    let mut object = cursor.object(0, true)?;
    cursor.skip()?;
    ensure!(
        cursor.index == source.len(),
        "unexpected content after JSON configuration object"
    );
    for key in path {
        let Some(member) = object
            .members
            .iter()
            .find(|candidate| candidate.key == *key)
        else {
            return Ok(None);
        };
        cursor.index = member.value.start;
        object = cursor
            .object(0, true)
            .with_context(|| format!("expected {key} to contain an object"))?;
    }
    Ok(Some(object))
}

impl Object {
    pub(crate) fn entries(self, source: &str) -> impl Iterator<Item = (String, &str)> {
        self.members
            .into_iter()
            .map(|member| (member.key, &source[member.value]))
    }

    fn encode(value: &Value) -> Result<String> {
        nextjson::to_string(value).map_err(anyhow::Error::msg)
    }

    pub(crate) fn edit(
        self,
        source: &str,
        key: &str,
        replacement: Option<&Value>,
    ) -> Result<String> {
        let mut text = source.to_owned();
        let position = self.members.iter().position(|member| member.key == key);
        match (position, replacement) {
            (Some(index), Some(value)) => {
                text.replace_range(self.members[index].value.clone(), &Self::encode(value)?);
            }
            (Some(index), None) => {
                let member = &self.members[index];
                if let Some(comma) = member.comma {
                    text.remove(comma);
                    text.replace_range(member.start..member.value.end, "");
                } else {
                    text.replace_range(member.start..member.value.end, "");
                    if index > 0 {
                        text.remove(
                            self.members[index - 1]
                                .comma
                                .context("missing object separator")?,
                        );
                    }
                }
            }
            (None, Some(value)) => {
                let key = Self::encode(&Value::from(key))?;
                let end = self.close;
                let line = &source[source[..end].rfind('\n').map_or(0, |at| at + 1)..end];
                let newline = if source.contains("\r\n") {
                    "\r\n"
                } else {
                    "\n"
                };
                let indentation = if line.trim().is_empty() { line } else { "" };
                let addition = format!("  {key}: {}{newline}{indentation}", Self::encode(value)?);
                if line.trim().is_empty() {
                    text.insert_str(end, &addition);
                } else {
                    text.insert_str(end, &format!("{newline}{addition}"));
                }
                if let Some(last) = self.members.last().filter(|last| last.comma.is_none()) {
                    text.insert(last.value.end, ',');
                }
            }
            (None, None) => {}
        }
        Ok(text)
    }
}
