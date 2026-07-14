//! SSE parser + generic [`EventStream<T>`].
//!
//! The parser is hand-rolled over a byte stream supplied by reqwest on native
//! targets and the browser's Fetch/ReadableStream APIs on WebAssembly. It
//! handles `data:` accumulation, the `data: [DONE]` terminator, comment lines
//! (`:` prefix), `\r\n` and bare `\r` line endings, and events split across
//! chunk boundaries.
//!
//! Reconnection: when the underlying body stream errors on a transient
//! failure, the stream re-opens via the caller-supplied closure with
//! exponential backoff capped at [`MAX_RECONNECT_BACKOFF`]. Reconnects are
//! counted across the lifetime of the stream, so an intermittent connection
//! cannot exceed the configured request-replay budget. Non-transient errors
//! and exhausted budget surface as `Err` and terminate the stream.
//!
//! Cancellation: dropping the `EventStream` drops the native response stream
//! or aborts the browser Fetch request. No explicit `CancellationToken` is
//! required — combine with `tokio::select!` on native targets or an abortable
//! local future in the browser.

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::Bytes;
use futures::{Stream, StreamExt};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Response;
use serde::de::DeserializeOwned;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::error::{Error, Result};
use crate::retry::MAX_RECONNECT_BACKOFF;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type StreamResponse = Response;

#[cfg(target_arch = "wasm32")]
pub(crate) struct StreamResponse {
    response: gloo_net::http::Response,
    abort: web_sys::AbortController,
}

#[cfg(target_arch = "wasm32")]
impl StreamResponse {
    pub(crate) fn new(response: gloo_net::http::Response, abort: web_sys::AbortController) -> Self {
        Self { response, abort }
    }
}

/// Async factory that re-opens the underlying HTTP response after a
/// transient failure. Returned by callers in `crate::client` so the stream
/// can resume the same request body on reconnect.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type Reopen = std::sync::Arc<
    dyn Fn() -> futures::future::BoxFuture<'static, Result<StreamResponse>> + Send + Sync + 'static,
>;
#[cfg(target_arch = "wasm32")]
pub(crate) type Reopen = std::rc::Rc<
    dyn Fn() -> futures::future::LocalBoxFuture<'static, Result<StreamResponse>> + 'static,
>;

#[cfg(not(target_arch = "wasm32"))]
type ByteStream = futures::stream::BoxStream<'static, Result<Bytes>>;
#[cfg(target_arch = "wasm32")]
type ByteStream = futures::stream::LocalBoxStream<'static, Result<Bytes>>;

#[cfg(not(target_arch = "wasm32"))]
type SleepFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
#[cfg(target_arch = "wasm32")]
type SleepFuture = Pin<Box<dyn Future<Output = ()> + 'static>>;

#[cfg(not(target_arch = "wasm32"))]
type ReopenFuture = futures::future::BoxFuture<'static, Result<StreamResponse>>;
#[cfg(target_arch = "wasm32")]
type ReopenFuture = futures::future::LocalBoxFuture<'static, Result<StreamResponse>>;

/// A stream of deserialized SSE events.
///
/// Implements [`futures::Stream`] with `Item = Result<T>`. Yields `None` on
/// the `data: [DONE]` terminator or when the underlying body finishes.
pub struct EventStream<T: DeserializeOwned> {
    state: State,
    buf: SseBuffer,
    reopen: Option<Reopen>,
    reconnect_attempt: u32,
    max_reconnects: u32,
    _marker: PhantomData<fn() -> T>,
}

impl<T: DeserializeOwned> std::fmt::Debug for EventStream<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventStream")
            .field("reconnect_attempt", &self.reconnect_attempt)
            .field("max_reconnects", &self.max_reconnects)
            .field("buffered", &self.buf.pending_len())
            .field("state", &self.state.tag())
            .finish()
    }
}

enum State {
    /// Active body stream — poll it for the next chunk.
    Reading(ByteStream),
    /// Sleeping before the next reconnect attempt.
    Backoff(SleepFuture),
    /// Re-opening the body via the caller's `reopen` closure.
    Reopening(ReopenFuture),
    /// Stream terminated (success or fatal error).
    Done,
}

impl State {
    fn tag(&self) -> &'static str {
        match self {
            State::Reading(_) => "reading",
            State::Backoff(_) => "backoff",
            State::Reopening(_) => "reopening",
            State::Done => "done",
        }
    }
}

impl<T: DeserializeOwned> EventStream<T> {
    /// Build a new event stream from an already-opened `Response` and a
    /// reconnect closure. The closure is invoked on transient mid-stream
    /// failures with exponential backoff.
    #[allow(dead_code)] // Consumed by the streaming endpoints (HRA-123).
    pub(crate) fn new(initial: StreamResponse, reopen: Reopen, max_reconnects: u32) -> Self {
        Self {
            state: State::Reading(box_byte_stream(initial)),
            buf: SseBuffer::default(),
            reopen: Some(reopen),
            reconnect_attempt: 0,
            max_reconnects,
            _marker: PhantomData,
        }
    }

    /// Build a non-reconnecting stream (used by tests and any caller that
    /// doesn't want resume semantics).
    #[cfg(test)]
    pub(crate) fn from_bytes_stream(bytes: ByteStream) -> Self {
        Self {
            state: State::Reading(bytes),
            buf: SseBuffer::default(),
            reopen: None,
            reconnect_attempt: 0,
            max_reconnects: 0,
            _marker: PhantomData,
        }
    }

    fn reconnect_delay(&self) -> Duration {
        // Exponential: 100ms, 200ms, 400ms, …, capped at MAX_RECONNECT_BACKOFF.
        let base_ms = 100u64.saturating_mul(1u64 << self.reconnect_attempt.min(8));
        let computed = Duration::from_millis(base_ms);
        computed.min(MAX_RECONNECT_BACKOFF)
    }
}

impl<T: DeserializeOwned> Stream for EventStream<T> {
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // First drain any complete events already buffered.
            match self.buf.next_event() {
                Some(SseEvent::Data(payload)) => {
                    let decoded: Result<T> = serde_json::from_slice(&payload)
                        .map_err(|e| Error::Stream(format!("malformed SSE payload: {e}")));
                    return Poll::Ready(Some(decoded));
                }
                Some(SseEvent::Done) => {
                    self.state = State::Done;
                    return Poll::Ready(None);
                }
                None => {}
            }

            // Drive the state machine to produce more bytes/events.
            // Take the current state out so we can replace it after polling.
            let cur = std::mem::replace(&mut self.state, State::Done);
            match cur {
                State::Reading(mut s) => match s.poll_next_unpin(cx) {
                    Poll::Ready(Some(Ok(chunk))) => {
                        self.buf.push(&chunk);
                        self.state = State::Reading(s);
                        continue;
                    }
                    Poll::Ready(Some(Err(err))) => {
                        if err.is_transient()
                            && self.reopen.is_some()
                            && self.reconnect_attempt < self.max_reconnects
                        {
                            // Schedule a reconnect.
                            let delay = self.reconnect_delay();
                            self.reconnect_attempt = self.reconnect_attempt.saturating_add(1);
                            self.state = State::Backoff(Box::pin(crate::timer::sleep(delay)));
                            continue;
                        }
                        self.state = State::Done;
                        return Poll::Ready(Some(Err(err)));
                    }
                    Poll::Ready(None) => {
                        // Body finished without [DONE]. Flush any final event,
                        // then complete.
                        if let Some(ev) = self.buf.finish() {
                            self.state = State::Done;
                            return match ev {
                                SseEvent::Data(payload) => {
                                    let decoded: Result<T> = serde_json::from_slice(&payload)
                                        .map_err(|e| {
                                            Error::Stream(format!("malformed SSE payload: {e}"))
                                        });
                                    Poll::Ready(Some(decoded))
                                }
                                SseEvent::Done => Poll::Ready(None),
                            };
                        }
                        self.state = State::Done;
                        return Poll::Ready(None);
                    }
                    Poll::Pending => {
                        self.state = State::Reading(s);
                        return Poll::Pending;
                    }
                },
                State::Backoff(mut fut) => match fut.as_mut().poll(cx) {
                    Poll::Ready(()) => {
                        let reopen = self
                            .reopen
                            .clone()
                            .expect("Backoff state requires a reopen closure");
                        let f = (reopen)();
                        self.state = State::Reopening(f);
                        continue;
                    }
                    Poll::Pending => {
                        self.state = State::Backoff(fut);
                        return Poll::Pending;
                    }
                },
                State::Reopening(mut fut) => match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok(resp)) => {
                        self.state = State::Reading(box_byte_stream(resp));
                        continue;
                    }
                    Poll::Ready(Err(err)) => {
                        if err.is_transient() && self.reconnect_attempt < self.max_reconnects {
                            // Try again, subject to the cap.
                            let delay = self.reconnect_delay();
                            self.reconnect_attempt = self.reconnect_attempt.saturating_add(1);
                            self.state = State::Backoff(Box::pin(crate::timer::sleep(delay)));
                            continue;
                        }
                        self.state = State::Done;
                        return Poll::Ready(Some(Err(err)));
                    }
                    Poll::Pending => {
                        self.state = State::Reopening(fut);
                        return Poll::Pending;
                    }
                },
                State::Done => {
                    self.state = State::Done;
                    return Poll::Ready(None);
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn box_byte_stream(response: StreamResponse) -> ByteStream {
    response
        .bytes_stream()
        .map(|result| result.map_err(Error::from))
        .boxed()
}

#[cfg(target_arch = "wasm32")]
fn box_byte_stream(response: StreamResponse) -> ByteStream {
    let StreamResponse { response, abort } = response;
    let Some(body) = response.body() else {
        return futures::stream::empty().boxed_local();
    };
    let abort = AbortOnDrop(abort);
    wasm_streams::ReadableStream::from_raw(body.unchecked_into())
        .into_stream()
        .map(move |item| {
            let _abort = &abort;
            let value = item.map_err(|error| Error::BrowserTransport(format!("{error:?}")))?;
            let array = js_sys::Uint8Array::new(&value);
            let mut bytes = vec![0; array.length() as usize];
            array.copy_to(&mut bytes);
            Ok(Bytes::from(bytes))
        })
        .boxed_local()
}

#[cfg(target_arch = "wasm32")]
struct AbortOnDrop(web_sys::AbortController);

#[cfg(target_arch = "wasm32")]
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// A parsed SSE event.
#[derive(Debug, PartialEq, Eq)]
enum SseEvent {
    /// Concatenated `data:` payload of a single event.
    Data(Vec<u8>),
    /// `data: [DONE]` terminator.
    Done,
}

/// Incremental SSE parser. Accumulates bytes and yields complete events as
/// `data:` lines are joined and blank lines flush them.
#[derive(Default)]
struct SseBuffer {
    /// Raw bytes not yet consumed as full lines (no trailing `\n` seen).
    pending: Vec<u8>,
    /// Lines that belong to the in-progress event (each is one `data:` payload
    /// without the `data:` prefix or trailing newline).
    current_data: Vec<Vec<u8>>,
    /// Whether the current event contained at least one `data:` line.
    has_data: bool,
}

impl SseBuffer {
    fn pending_len(&self) -> usize {
        self.pending.len()
    }

    fn push(&mut self, chunk: &[u8]) {
        self.pending.extend_from_slice(chunk);
    }

    /// Pop the next complete event from the buffer, if one is available.
    fn next_event(&mut self) -> Option<SseEvent> {
        loop {
            let idx = self.pending.iter().position(|&b| b == b'\n')?;
            // Take the line (excluding the `\n`); trim any trailing `\r`.
            let mut line: Vec<u8> = self.pending.drain(..=idx).collect();
            line.pop(); // remove the `\n`
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            if let Some(ev) = self.process_line(line) {
                return Some(ev);
            }
        }
    }

    /// Called when the upstream byte stream is exhausted. Flushes any
    /// pending bytes as a final line.
    fn finish(&mut self) -> Option<SseEvent> {
        if !self.pending.is_empty() {
            let mut line = std::mem::take(&mut self.pending);
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            if let Some(ev) = self.process_line(line) {
                return Some(ev);
            }
        }
        self.flush_event()
    }

    fn process_line(&mut self, line: Vec<u8>) -> Option<SseEvent> {
        if line.is_empty() {
            return self.flush_event();
        }
        // Comment line.
        if line.first() == Some(&b':') {
            return None;
        }
        // Strip the field name. SSE allows arbitrary fields; we only care about `data`.
        if let Some(rest) = strip_field(&line, b"data") {
            self.current_data.push(rest);
            self.has_data = true;
        }
        // Other fields (event:, id:, retry:) are intentionally ignored — the
        // OpenRouter SSE stream uses only `data:`.
        None
    }

    fn flush_event(&mut self) -> Option<SseEvent> {
        if !self.has_data {
            return None;
        }
        self.has_data = false;
        let lines = std::mem::take(&mut self.current_data);
        // Per the SSE spec, multi-line `data:` payloads are joined by `\n`.
        let mut payload: Vec<u8> = Vec::new();
        for (i, l) in lines.iter().enumerate() {
            if i > 0 {
                payload.push(b'\n');
            }
            payload.extend_from_slice(l);
        }
        // `[DONE]` terminator is treated specially.
        if payload == b"[DONE]" {
            return Some(SseEvent::Done);
        }
        Some(SseEvent::Data(payload))
    }
}

/// If `line` starts with `field:`, return the value (with at most one leading
/// space trimmed, per the SSE spec).
fn strip_field(line: &[u8], field: &[u8]) -> Option<Vec<u8>> {
    if line.len() < field.len() + 1 {
        return None;
    }
    if &line[..field.len()] != field {
        return None;
    }
    if line[field.len()] != b':' {
        return None;
    }
    let mut rest = &line[field.len() + 1..];
    if rest.first() == Some(&b' ') {
        rest = &rest[1..];
    }
    Some(rest.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use pretty_assertions::assert_eq;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn drain_buffer(buf: &mut SseBuffer) -> Vec<SseEvent> {
        let mut out = Vec::new();
        while let Some(ev) = buf.next_event() {
            out.push(ev);
        }
        if let Some(ev) = buf.finish() {
            out.push(ev);
        }
        out
    }

    #[test]
    fn parses_single_event() {
        let mut b = SseBuffer::default();
        b.push(b"data: {\"x\":1}\n\n");
        let events = drain_buffer(&mut b);
        assert_eq!(events, vec![SseEvent::Data(b"{\"x\":1}".to_vec())]);
    }

    #[test]
    fn parses_done_terminator() {
        let mut b = SseBuffer::default();
        b.push(b"data: [DONE]\n\n");
        let events = drain_buffer(&mut b);
        assert_eq!(events, vec![SseEvent::Done]);
    }

    #[test]
    fn ignores_comment_lines() {
        let mut b = SseBuffer::default();
        b.push(b": heartbeat\ndata: {\"a\":1}\n\n");
        let events = drain_buffer(&mut b);
        assert_eq!(events, vec![SseEvent::Data(b"{\"a\":1}".to_vec())]);
    }

    #[test]
    fn joins_multi_line_data() {
        let mut b = SseBuffer::default();
        b.push(b"data: line1\ndata: line2\n\n");
        let events = drain_buffer(&mut b);
        assert_eq!(events, vec![SseEvent::Data(b"line1\nline2".to_vec())]);
    }

    #[test]
    fn handles_crlf_line_endings() {
        let mut b = SseBuffer::default();
        b.push(b"data: {\"x\":1}\r\n\r\n");
        let events = drain_buffer(&mut b);
        assert_eq!(events, vec![SseEvent::Data(b"{\"x\":1}".to_vec())]);
    }

    #[test]
    fn handles_chunk_boundaries() {
        let mut b = SseBuffer::default();
        b.push(b"data: {\"x");
        assert!(b.next_event().is_none());
        b.push(b"\":1}\n");
        // No terminating blank line yet — event not flushed.
        assert!(b.next_event().is_none());
        b.push(b"\n");
        let events = drain_buffer(&mut b);
        assert_eq!(events, vec![SseEvent::Data(b"{\"x\":1}".to_vec())]);
    }

    #[test]
    fn ignores_non_data_fields() {
        let mut b = SseBuffer::default();
        b.push(b"event: ping\nid: 42\nretry: 1000\ndata: {\"x\":1}\n\n");
        let events = drain_buffer(&mut b);
        assert_eq!(events, vec![SseEvent::Data(b"{\"x\":1}".to_vec())]);
    }

    #[test]
    fn flushes_trailing_event_without_blank_line() {
        let mut b = SseBuffer::default();
        b.push(b"data: {\"x\":1}\n");
        // No second \n; finish() flushes.
        let events = drain_buffer(&mut b);
        assert_eq!(events, vec![SseEvent::Data(b"{\"x\":1}".to_vec())]);
    }

    #[test]
    fn handles_empty_data_payload() {
        let mut b = SseBuffer::default();
        b.push(b"data: \n\n");
        let events = drain_buffer(&mut b);
        assert_eq!(events, vec![SseEvent::Data(Vec::new())]);
    }

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Sample {
        x: i32,
    }

    #[tokio::test]
    async fn event_stream_yields_decoded_events_then_done() {
        let chunks: Vec<Result<Bytes>> = vec![
            Ok(Bytes::from_static(b"data: {\"x\":1}\n\n")),
            Ok(Bytes::from_static(b"data: {\"x\":2}\n\n")),
            Ok(Bytes::from_static(b"data: [DONE]\n\n")),
        ];
        let body: ByteStream = stream::iter(chunks).boxed();
        let mut s: EventStream<Sample> = EventStream::from_bytes_stream(body);
        let a = s.next().await.unwrap().unwrap();
        let b = s.next().await.unwrap().unwrap();
        assert_eq!(a, Sample { x: 1 });
        assert_eq!(b, Sample { x: 2 });
        assert!(s.next().await.is_none());
    }

    #[tokio::test]
    async fn event_stream_surfaces_malformed_payload_as_error() {
        let chunks: Vec<Result<Bytes>> = vec![Ok(Bytes::from_static(b"data: not-json\n\n"))];
        let body: ByteStream = stream::iter(chunks).boxed();
        let mut s: EventStream<Sample> = EventStream::from_bytes_stream(body);
        let item = s.next().await.unwrap();
        assert!(matches!(item, Err(Error::Stream(_))));
    }

    #[tokio::test]
    async fn event_stream_handles_split_event_across_chunks() {
        let chunks: Vec<Result<Bytes>> = vec![
            Ok(Bytes::from_static(b"data: {\"x")),
            Ok(Bytes::from_static(b"\":7}\n\n")),
            Ok(Bytes::from_static(b"data: [DONE]\n\n")),
        ];
        let body: ByteStream = stream::iter(chunks).boxed();
        let mut s: EventStream<Sample> = EventStream::from_bytes_stream(body);
        let a = s.next().await.unwrap().unwrap();
        assert_eq!(a, Sample { x: 7 });
        assert!(s.next().await.is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn reconnect_budget_is_lifetime_bounded() {
        let body: ByteStream =
            stream::iter(vec![Err(Error::BrowserTransport("lost".into()))]).boxed();
        let calls = Arc::new(AtomicUsize::new(0));
        let reopen_calls = Arc::clone(&calls);
        let reopen: Reopen = Arc::new(move || {
            reopen_calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Err(Error::BrowserTransport("still lost".into())) })
        });
        let mut stream: EventStream<Sample> = EventStream {
            state: State::Reading(body),
            buf: SseBuffer::default(),
            reopen: Some(reopen),
            reconnect_attempt: 0,
            max_reconnects: 2,
            _marker: PhantomData,
        };

        let error = stream.next().await.unwrap().unwrap_err();
        assert!(matches!(error, Error::BrowserTransport(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(stream.reconnect_attempt, 2);
    }
}
