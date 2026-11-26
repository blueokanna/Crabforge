use crate::gateway::{self, BODY_LIMIT, Gateway};
use anyhow::{Context, Result, ensure};
use axum_core::response::{IntoResponse, Response};
use bytes::{Buf, Bytes, BytesMut};
use courierust::{
    courierust_h1::{encode_chunk, write_response_head},
    courierust_http as wire,
};
use futures_util::StreamExt;
use http::{HeaderName, HeaderValue, Method, Request, StatusCode, Version, header};
use http_body::Body as _;
use simoom_http1::{
    BodyDecoder, BodyEvent, EMPTY_HEADER, Framing, Limits, RequestParser, Status, write::http_date,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::mpsc,
    time::timeout,
};

const HEAD_LIMIT: usize = 32 * 1024;

struct Head {
    request: Request<()>,
    framing: Framing,
    keep_alive: bool,
    expect_continue: bool,
}

struct Input {
    chunks: mpsc::Receiver<Bytes>,
    buffer: BytesMut,
}

impl Input {
    async fn more(&mut self) -> Result<()> {
        let chunk = self.chunks.recv().await.context("client disconnected")?;
        self.buffer.extend_from_slice(&chunk);
        Ok(())
    }

    async fn head(&mut self) -> Result<Head> {
        let mut parser = RequestParser::new();
        let limits = Limits {
            max_head: HEAD_LIMIT,
            max_header_line: HEAD_LIMIT,
            max_request_line: HEAD_LIMIT,
            // The gateway authenticates first and checks the body limit when it reads.
            max_body: u64::MAX,
            ..Default::default()
        };
        loop {
            let mut headers = [EMPTY_HEADER; 100];
            if let Status::Complete(head) = parser.parse(&self.buffer, &mut headers, &limits)? {
                let version = match head.version {
                    simoom_http1::Version::Http11 => Version::HTTP_11,
                    simoom_http1::Version::Http10 => Version::HTTP_10,
                };
                let mut builder = Request::builder()
                    .method(head.method)
                    .uri(head.target)
                    .version(version);
                for h in head.headers {
                    builder = builder.header(
                        HeaderName::from_bytes(h.name)?,
                        HeaderValue::from_bytes(h.value)?,
                    );
                }
                let request = Head {
                    request: builder.body(())?,
                    framing: head.framing,
                    keep_alive: head.keep_alive,
                    expect_continue: head.expect_continue,
                };
                self.buffer.advance(head.head_len);
                return Ok(request);
            }
            self.more().await?;
        }
    }

    async fn body(
        &mut self,
        framing: Framing,
        decoder: &mut BodyDecoder,
        expect: bool,
        write: &mut (impl AsyncWrite + Unpin),
    ) -> Result<Bytes> {
        ensure!(
            !matches!(framing,Framing::Length(n) if n > BODY_LIMIT as u64),
            "request body too large"
        );
        if expect && !decoder.is_done() && self.buffer.is_empty() {
            write.write_all(b"HTTP/1.1 100 Continue\r\n\r\n").await?;
            write.flush().await?;
        }
        let mut body = BytesMut::new();
        while !decoder.is_done() {
            let (used, event) = decoder.decode(&self.buffer)?;
            if let BodyEvent::Data(data) = event {
                body.extend_from_slice(data);
            }
            let need_more = matches!(event, BodyEvent::NeedMore);
            self.buffer.advance(used);
            if need_more {
                self.more().await?;
            }
        }
        Ok(body.freeze())
    }
}

pub(super) async fn serve(io: impl AsyncRead + AsyncWrite + Unpin, app: &Gateway) -> Result<()> {
    let (mut read, mut write) = tokio::io::split(io);
    let (tx, chunks) = mpsc::channel(1);
    let mut input = Input {
        chunks,
        buffer: BytesMut::new(),
    };
    // Reading continues during inference so closing the client cancels a pending
    // response. A pipelined client can queue at most one extra chunk here.
    let receive = async move {
        let mut buffer = [0; 8192];
        loop {
            let n = read.read(&mut buffer).await?;
            if n == 0 || tx.send(Bytes::copy_from_slice(&buffer[..n])).await.is_err() {
                return Ok(());
            }
        }
    };
    tokio::select! {
        result = receive => result,
        result = exchanges(&mut input, &mut write, app) => result,
    }
}

async fn exchanges(
    input: &mut Input,
    out: &mut (impl AsyncWrite + Unpin),
    app: &Gateway,
) -> Result<()> {
    loop {
        let parsed = timeout(Duration::from_secs(15), input.head())
            .await
            .map_err(|_| StatusCode::REQUEST_TIMEOUT)
            .and_then(|result| {
                result.map_err(|err| {
                    err.downcast_ref::<simoom_http1::Error>()
                        .and_then(|e| StatusCode::from_u16(e.status()).ok())
                        .unwrap_or(StatusCode::BAD_REQUEST)
                })
            });
        let head = match parsed {
            Ok(head) => head,
            Err(status) => {
                respond(out, status.into_response(), Version::HTTP_11, false, false).await?;
                break;
            }
        };
        let version = head.request.version();
        let headers_only = head.request.method() == Method::HEAD;
        let mut decoder = BodyDecoder::new(
            head.framing,
            &Limits {
                max_body: BODY_LIMIT as u64,
                ..Default::default()
            },
        );
        let request = head
            .request
            .map(|()| input.body(head.framing, &mut decoder, head.expect_continue, out));
        let response = gateway::dispatch(app, request).await;
        if !respond(
            out,
            response,
            version,
            headers_only,
            head.keep_alive && decoder.is_done(),
        )
        .await?
        {
            break;
        }
    }
    out.shutdown().await?;
    Ok(())
}

async fn respond(
    out: &mut (impl AsyncWrite + Unpin),
    response: Response,
    version: Version,
    head: bool,
    mut keep_alive: bool,
) -> Result<bool> {
    let (parts, body) = response.into_parts();
    let status = parts.status;
    let bodyless = status.is_informational()
        || status == StatusCode::NO_CONTENT
        || status == StatusCode::NOT_MODIFIED;
    let length = body.size_hint().exact();
    let chunked = !head && !bodyless && length.is_none() && version == Version::HTTP_11;
    if !head && !bodyless && length.is_none() && !chunked {
        keep_alive = false;
    }
    let mut headers = parts.headers;
    headers.remove(header::CONTENT_LENGTH);
    headers.remove(header::TRANSFER_ENCODING);
    if !bodyless && let Some(length) = length {
        headers.insert(header::CONTENT_LENGTH, length.into());
    }
    if chunked {
        headers.insert(
            header::TRANSFER_ENCODING,
            HeaderValue::from_static("chunked"),
        );
    }
    if !keep_alive {
        headers.insert(header::CONNECTION, HeaderValue::from_static("close"));
    } else if version == Version::HTTP_10 {
        headers.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
    }
    let date = http_date(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
    headers
        .entry(header::DATE)
        .or_insert(HeaderValue::from_bytes(&date)?);
    let mut wire_headers = wire::HeaderMap::new();
    for (name, value) in &headers {
        wire_headers.append(
            wire::HeaderName::from_bytes(name.as_str().as_bytes())?,
            wire::HeaderValue::from_bytes(value.as_bytes())?,
        );
    }
    let version = if version == Version::HTTP_10 {
        wire::Version::HTTP_10
    } else {
        wire::Version::HTTP_11
    };
    let mut bytes = Vec::new();
    write_response_head(&mut bytes, status.as_u16().into(), version, &wire_headers)?;
    out.write_all(&bytes).await?;
    out.flush().await?;
    if head || bodyless {
        return Ok(keep_alive);
    }
    let mut stream = body.into_data_stream();
    while let Some(data) = stream.next().await {
        let data = data?;
        if data.is_empty() {
            continue;
        }
        if chunked {
            for part in data.chunks(16 * 1024) {
                bytes.clear();
                encode_chunk(part, &mut bytes);
                out.write_all(&bytes).await?;
            }
        } else {
            out.write_all(&data).await?;
        }
        out.flush().await?;
    }
    if chunked {
        out.write_all(b"0\r\n\r\n").await?;
        out.flush().await?;
    }
    Ok(keep_alive)
}
