use super::routing::Candidate;
use axum_core::{body::Body, response::Response};
use bytes::BytesMut;
use futures_util::{Stream, StreamExt, stream};
use http::{HeaderMap, StatusCode, header};
use std::{collections::VecDeque, pin::Pin, time::Duration};
use time::{OffsetDateTime, format_description::well_known::Rfc2822};
use tokio::time::Instant;

const MAX_ROUNDS: u32 = 12;
const MAX_DELAY: Duration = Duration::from_secs(30);
const ERROR_PREFIX: usize = 8192;

pub(super) fn retryable(status: StatusCode) -> bool {
    matches!(
        status.as_u16(),
        408 | 429 | 500 | 502 | 503 | 504 | 520..=524 | 529
    )
}

fn retry_after(headers: &HeaderMap) -> Duration {
    let Some(value) = headers
        .get(header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
    else {
        return Duration::ZERO;
    };
    let value = value.trim();
    let delay = value.parse::<u64>().map_or_else(
        |_| {
            OffsetDateTime::parse(value, &Rfc2822)
                .ok()
                .and_then(|date| Duration::try_from(date - OffsetDateTime::now_utc()).ok())
                .unwrap_or_default()
        },
        Duration::from_secs,
    );
    delay.min(Duration::from_secs(241))
}

// Keep the original response stream available if no retry fits the remaining budget.
async fn inspect(response: Response) -> (Response, bool) {
    let (head, body) = response.into_parts();
    let mut stream = body.into_data_stream();
    let mut replay = Vec::new();
    let mut prefix = BytesMut::new();
    let finish = tokio::time::timeout(Duration::from_secs(1), async {
        while let Some(chunk) = stream.next().await {
            let Ok(bytes) = &chunk else {
                replay.push(chunk);
                return false;
            };
            prefix.extend_from_slice(&bytes[..bytes.len().min(ERROR_PREFIX - prefix.len())]);
            replay.push(chunk);
            if prefix.len() == ERROR_PREFIX {
                break;
            }
        }
        true
    })
    .await
    .unwrap_or(false);
    let text = String::from_utf8_lossy(&prefix).to_ascii_lowercase();
    let busy = finish
        && ![
            "insufficient_quota",
            "billing_hard_limit",
            "billing_not_active",
            "insufficient credits",
            "insufficient balance",
            "quota exhausted",
            "quota has been exhausted",
            "queued the request",
            "this plan's queue:",
        ]
        .iter()
        .any(|word| text.contains(word));
    let body = Body::from_stream(stream::iter(replay).chain(stream));
    (Response::from_parts(head, body), busy)
}

struct Waiting<'a> {
    candidate: Candidate<'a>,
    ready: Instant,
}

pub(super) struct Attempts<'a, S: Stream<Item = Candidate<'a>>> {
    initial: Pin<Box<S>>,
    next: Option<Candidate<'a>>,
    current: VecDeque<Candidate<'a>>,
    waiting: Vec<Waiting<'a>>,
    started: Option<Instant>,
    rounds: u32,
}

impl<'a, S: Stream<Item = Candidate<'a>>> Attempts<'a, S> {
    pub(super) fn new(candidates: S) -> Self {
        Self {
            initial: Box::pin(candidates),
            next: None,
            current: VecDeque::new(),
            waiting: Vec::new(),
            started: None,
            rounds: 0,
        }
    }

    pub(super) async fn has_more(&mut self) -> bool {
        if self.current.is_empty() && self.next.is_none() {
            self.next = self.initial.next().await;
        }
        !self.current.is_empty() || self.next.is_some()
    }

    pub(super) async fn next(&mut self) -> Option<Candidate<'a>> {
        match self.current.pop_front().or_else(|| self.next.take()) {
            Some(candidate) => Some(candidate),
            None => self.initial.next().await,
        }
    }

    pub(super) async fn advance(&mut self) -> bool {
        if self.has_more().await {
            return true;
        }
        let Some(retry_at) = self.replan() else {
            return false;
        };
        tokio::time::sleep_until(retry_at).await;
        true
    }

    pub(super) async fn observe(
        &mut self,
        candidate: Option<Candidate<'a>>,
        response: Response,
    ) -> Response {
        let Some(candidate) = candidate.filter(|_| self.rounds < MAX_ROUNDS) else {
            return response;
        };
        let now = Instant::now();
        let ready = now + retry_after(response.headers());
        let (response, busy) = inspect(response).await;
        if busy {
            self.waiting.push(Waiting { candidate, ready });
        }
        response
    }

    pub(super) fn replan(&mut self) -> Option<Instant> {
        if self.waiting.is_empty() || self.rounds == MAX_ROUNDS {
            return None;
        }
        let now = Instant::now();
        let started = *self.started.get_or_insert(now);
        let next = now + Duration::from_secs(2 << self.rounds).min(MAX_DELAY);
        self.waiting
            .retain(|entry| next.max(entry.ready) <= started + entry.candidate.patience);
        let ready = self.waiting.iter().map(|entry| entry.ready).min()?;
        let retry_at = next.max(ready);
        let mut deferred = Vec::new();
        for entry in self.waiting.drain(..) {
            if entry.ready <= retry_at {
                self.current.push_back(entry.candidate);
            } else {
                deferred.push(entry);
            }
        }
        self.waiting = deferred;
        self.rounds += 1;
        Some(retry_at)
    }
}
