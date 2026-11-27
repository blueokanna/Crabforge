use anyhow::{Result, anyhow};
use async_stream::try_stream;
use futures_util::{Stream, StreamExt};
use http::{HeaderMap, header::CONTENT_TYPE};

pub(crate) fn is_event_stream(headers: &HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split(';').next())
        .is_some_and(|h| h.trim().eq_ignore_ascii_case("text/event-stream"))
}

#[derive(Default)]
pub(crate) struct Parser {
    line: Vec<u8>,
    name: String,
    data: String,
    after_cr: bool,
}

impl Parser {
    /// Split before decoding so a multibyte code point can span HTTP chunks.
    pub(crate) fn push(&mut self, byte: u8) -> Result<Option<(String, String)>> {
        if self.after_cr && byte == b'\n' {
            self.after_cr = false;
            return Ok(None);
        }
        self.after_cr = byte == b'\r';
        if !matches!(byte, b'\r' | b'\n') {
            if self.line.len() + self.data.len() >= 2 * 1024 * 1024 {
                return Err(anyhow!("SSE frame exceeds 2 MiB"));
            }
            self.line.push(byte);
            return Ok(None);
        }
        if self.line.is_empty() {
            let name = std::mem::take(&mut self.name);
            return Ok(self
                .data
                .pop()
                .map(|_| (name, std::mem::take(&mut self.data))));
        }
        if self.line == b"data" {
            self.data.push('\n');
        } else if let Some(value) = self.line.strip_prefix(b"data:") {
            self.data.push_str(std::str::from_utf8(
                value.strip_prefix(b" ").unwrap_or(value),
            )?);
            self.data.push('\n');
        } else if let Some(value) = self.line.strip_prefix(b"event:") {
            self.name =
                String::from_utf8_lossy(value.strip_prefix(b" ").unwrap_or(value)).into_owned();
        }
        self.line.clear();
        Ok(None)
    }

    fn finish(self) -> Result<()> {
        if !self.line.is_empty() || !self.data.is_empty() {
            return Err(anyhow!("incomplete SSE frame"));
        }
        Ok(())
    }
}

pub(crate) fn data<E>(
    mut chunks: impl Stream<Item = Result<bytes::Bytes, E>> + Unpin,
) -> impl Stream<Item = Result<String>>
where
    E: std::error::Error + Send + Sync + 'static,
{
    try_stream! {
        let mut parser = Parser::default();
        while let Some(chunk) = chunks.next().await {
            let chunk = chunk?;
            for &byte in &chunk {
                if let Some((_, data)) = parser.push(byte)? {
                    yield data;
                }
            }
        }
        parser.finish()?;
    }
}
