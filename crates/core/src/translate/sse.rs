use anyhow::{Result, anyhow};
use async_stream::try_stream;
use futures_util::{Stream, StreamExt};

/// The frame budget also bounds an unterminated line. Split before decoding so
/// a multibyte code point can span two HTTP chunks.
pub(crate) fn data<E>(
    mut chunks: impl Stream<Item = Result<axum::body::Bytes, E>> + Unpin,
) -> impl Stream<Item = Result<String>>
where
    E: std::error::Error + Send + Sync + 'static,
{
    try_stream! {
        let mut line = Vec::new();
        let mut data = String::new();
        let mut after_cr = false;
        while let Some(chunk) = chunks.next().await {
            let chunk = chunk?;
            for &byte in &chunk {
                if after_cr && byte == b'\n' { after_cr = false; continue; }
                after_cr = byte == b'\r';
                if !matches!(byte, b'\r' | b'\n') {
                    if line.len() + data.len() >= 2 * 1024 * 1024 { Err(anyhow!("SSE frame exceeds 2 MiB"))?; }
                    line.push(byte);
                    continue;
                }
                if line.is_empty() && !data.is_empty() {
                    data.pop();
                    yield std::mem::take(&mut data);
                } else if line == b"data" {
                    data.push('\n');
                } else if let Some(value) = line.strip_prefix(b"data:") {
                    data.push_str(std::str::from_utf8(value.strip_prefix(b" ").unwrap_or(value))?);
                    data.push('\n');
                }
                line.clear();
            }
        }
        if !line.is_empty() || !data.is_empty() { Err(anyhow!("incomplete SSE frame"))?; }
    }
}
