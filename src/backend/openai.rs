//! Backend for a local service speaking the OpenAI chat-completions API.
//!
//! Used for Ubuntu inference snaps and for explicitly configured local
//! providers. The HTTP API is stateless, so the session keeps the transcript
//! and replays it on every generation.

use std::{
    io::{BufRead, BufReader, Read},
    net::TcpStream,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll},
    thread,
    time::Duration,
};

use async_trait::async_trait;
use futures_core::Stream;
use serde_json::{json, Value};
use tokio::sync::mpsc;

use super::{
    http::{self, Endpoint},
    worker::run_blocking,
    Backend, BackendSession, ResponseStream,
};
use crate::{
    AiResponse, Availability, AvailabilityReason, BackendInfo, BackendKind, Capabilities,
    GenerationConfig, LocalAiError, ResponseFormat, Result,
};

/// Timeout for quick metadata requests such as `GET /models`.
const PROBE_TIMEOUT: Duration = Duration::from_secs(10);
/// Longest silence tolerated while a model is generating (prompt processing
/// on a CPU can take minutes before the first token).
const GENERATION_IDLE_TIMEOUT: Duration = Duration::from_secs(600);

/// A resolved local OpenAI-compatible service.
#[derive(Debug, Clone)]
pub(crate) struct Service {
    pub(crate) endpoint: Endpoint,
    pub(crate) url: String,
    /// Model name to request; `None` asks the server via `GET /models`.
    pub(crate) model: Option<String>,
}

impl Service {
    pub(crate) fn new(url: &str, model: Option<String>) -> Result<Self> {
        Ok(Self {
            endpoint: Endpoint::parse(url).map_err(LocalAiError::InvalidConfig)?,
            url: url.trim().to_owned(),
            model,
        })
    }

    /// Checks that the service answers and resolves the model name.
    /// Blocking.
    pub(crate) fn probe(&self) -> std::result::Result<String, String> {
        let response = self
            .endpoint
            .send("GET", "/models", None, PROBE_TIMEOUT, &mut |_| {})
            .map_err(|e| format!("{} is not reachable: {e}", self.url))?;
        let status = response.status;
        let body = response
            .read_body()
            .map_err(|e| format!("reading {}/models failed: {e}", self.url))?;
        if status != 200 {
            return Err(format!("{}/models returned HTTP {status}", self.url));
        }
        if let Some(model) = &self.model {
            return Ok(model.clone());
        }
        serde_json::from_slice::<Value>(&body)
            .ok()
            .and_then(|v| v["data"][0]["id"].as_str().map(str::to_owned))
            .ok_or_else(|| format!("{}/models lists no model", self.url))
    }
}

pub(crate) fn capabilities(system_managed_model: bool) -> Capabilities {
    Capabilities {
        text_generation: true,
        streaming: true,
        cancellation: true,
        concurrent_sessions: true,
        system_managed_model,
        // Not every engine honours `response_format`, token counting has no
        // portable endpoint, and tool calling is not exposed by this crate.
        ..Capabilities::default()
    }
}

/// An explicitly configured local OpenAI-compatible provider.
pub(crate) struct OpenAiBackend {
    pub(crate) service: Service,
}

#[async_trait]
impl Backend for OpenAiBackend {
    fn info(&self) -> BackendInfo {
        BackendInfo {
            kind: BackendKind::OpenAiCompatible,
            name: format!("OpenAI-compatible service at {}", self.service.url),
            system_managed_model: false,
        }
    }

    async fn availability(&self) -> Availability {
        let service = self.service.clone();
        match run_blocking(BackendKind::OpenAiCompatible, move || Ok(service.probe())).await {
            Ok(Ok(model)) => Availability::available().with_detail(format!("model {model}")),
            Ok(Err(detail)) => Availability::unavailable(AvailabilityReason::ProviderNotInstalled)
                .with_detail(detail),
            Err(error) => Availability::unavailable(AvailabilityReason::BackendFailure)
                .with_detail(error.to_string()),
        }
    }

    async fn capabilities(&self) -> Capabilities {
        capabilities(false)
    }

    async fn open_session(&self, instructions: Option<&str>) -> Result<Arc<dyn BackendSession>> {
        open_session(
            self.service.clone(),
            BackendKind::OpenAiCompatible,
            instructions,
        )
        .await
    }
}

pub(crate) async fn open_session(
    service: Service,
    kind: BackendKind,
    instructions: Option<&str>,
) -> Result<Arc<dyn BackendSession>> {
    let probe = service.clone();
    let model = run_blocking(kind, move || Ok(probe.probe()))
        .await?
        .map_err(|message| LocalAiError::Backend {
            backend: kind,
            message,
        })?;
    let mut messages = Vec::new();
    if let Some(instructions) = instructions.filter(|i| !i.is_empty()) {
        messages.push(json!({"role": "system", "content": instructions}));
    }
    Ok(Arc::new(OpenAiSession {
        shared: Arc::new(Shared {
            endpoint: service.endpoint,
            model,
            kind,
            transcript: Mutex::new(Transcript {
                messages,
                pending: String::new(),
            }),
            active: Mutex::new(None),
            cancelled: AtomicBool::new(false),
        }),
    }))
}

struct OpenAiSession {
    shared: Arc<Shared>,
}

struct Shared {
    endpoint: Endpoint,
    model: String,
    kind: BackendKind,
    transcript: Mutex<Transcript>,
    /// The socket of the generation in flight, so `cancel` can abort it.
    active: Mutex<Option<TcpStream>>,
    cancelled: AtomicBool,
}

struct Transcript {
    messages: Vec<Value>,
    pending: String,
}

impl Shared {
    fn error(&self, message: impl Into<String>) -> LocalAiError {
        LocalAiError::Backend {
            backend: self.kind,
            message: message.into(),
        }
    }

    /// Takes the pending turn and builds the request body.
    fn start_turn(&self, config: &GenerationConfig, stream: bool) -> (String, Vec<u8>) {
        let mut transcript = self.transcript.lock().expect("transcript lock");
        let user = std::mem::take(&mut transcript.pending);
        let mut messages = transcript.messages.clone();
        messages.push(json!({"role": "user", "content": user}));
        drop(transcript);

        let mut body = json!({"model": self.model, "messages": messages, "stream": stream});
        if let Some(max) = config.max_output_tokens {
            body["max_tokens"] = max.into();
        }
        if let Some(temperature) = config.temperature {
            body["temperature"] = temperature.into();
        }
        if let Some(top_p) = config.top_p {
            body["top_p"] = top_p.into();
        }
        if let Some(seed) = config.seed {
            body["seed"] = seed.into();
        }
        if stream {
            body["stream_options"] = json!({"include_usage": true});
        }
        // Structured output is not advertised, so the session layer only lets
        // `ResponseFormat::Text` through.
        debug_assert!(matches!(config.response_format, ResponseFormat::Text));
        (user, serde_json::to_vec(&body).expect("request serializes"))
    }

    /// Records a completed turn, or returns the user text to the pending
    /// buffer so a failed generation can be retried.
    fn finish_turn(&self, user: String, answer: Option<&str>) {
        let mut transcript = self.transcript.lock().expect("transcript lock");
        match answer {
            Some(answer) => {
                transcript
                    .messages
                    .push(json!({"role": "user", "content": user}));
                transcript
                    .messages
                    .push(json!({"role": "assistant", "content": answer}));
            }
            None => {
                let rest = std::mem::take(&mut transcript.pending);
                transcript.pending = user + &rest;
            }
        }
    }

    /// Sends a chat-completions request, registering the socket for `cancel`
    /// as soon as it is connected (non-streaming servers reply only once the
    /// whole answer is generated).
    fn send(&self, body: &[u8]) -> Result<http::Response> {
        let response = self
            .endpoint
            .send(
                "POST",
                "/chat/completions",
                Some(body),
                GENERATION_IDLE_TIMEOUT,
                &mut |stream| {
                    *self.active.lock().expect("active lock") = stream.try_clone().ok();
                    // A cancel that arrived before the socket was registered.
                    if self.cancelled.load(Ordering::Acquire) {
                        http::abort(stream);
                    }
                },
            )
            .map_err(|e| self.io_error(e))?;
        if response.status != 200 {
            let status = response.status;
            let body = response.read_body().unwrap_or_default();
            return Err(self.error(format!(
                "HTTP {status}: {}",
                error_message(&body).unwrap_or_else(|| String::from_utf8_lossy(&body).into_owned())
            )));
        }
        Ok(response)
    }

    /// Marks the generation in flight as cancelled and aborts its socket.
    fn abort(&self) {
        self.cancelled.store(true, Ordering::Release);
        if let Some(stream) = self.active.lock().expect("active lock").as_ref() {
            http::abort(stream);
        }
    }

    fn io_error(&self, error: std::io::Error) -> LocalAiError {
        if self.cancelled.load(Ordering::Acquire) {
            LocalAiError::Cancelled
        } else {
            self.error(format!("request failed: {error}"))
        }
    }

    fn clear_active(&self) {
        *self.active.lock().expect("active lock") = None;
    }

    fn generate_blocking(&self, config: GenerationConfig) -> Result<AiResponse> {
        let (user, body) = self.start_turn(&config, false);
        let result = self.send(&body).and_then(|response| {
            let body = response.read_body().map_err(|e| self.io_error(e))?;
            parse_completion(&body).map_err(|m| self.error(m))
        });
        self.clear_active();
        self.finish_turn(user, result.as_ref().ok().map(|r| r.text.as_str()));
        result
    }

    fn stream_blocking(
        self: Arc<Self>,
        config: GenerationConfig,
        sender: mpsc::Sender<Result<String>>,
    ) {
        let (user, body) = self.start_turn(&config, true);
        let response = match self.send(&body) {
            Ok(response) => response,
            Err(error) => {
                self.clear_active();
                self.finish_turn(user, None);
                let _ = sender.blocking_send(Err(error));
                return;
            }
        };
        let mut text = String::new();
        let outcome = read_events(BufReader::new(response.body), |data| {
            let delta = parse_delta(data).map_err(|m| self.error(m))?;
            if !delta.is_empty() {
                text.push_str(&delta);
                if sender.blocking_send(Ok(delta)).is_err() {
                    // The consumer dropped the stream.
                    return Err(LocalAiError::Cancelled);
                }
            }
            Ok(())
        })
        .map_err(|error| match error {
            EventError::Io(e) => self.io_error(e),
            EventError::Handler(e) => e,
        });
        self.clear_active();
        match outcome {
            Ok(()) => self.finish_turn(user, Some(&text)),
            Err(error) => {
                self.finish_turn(user, None);
                let _ = sender.blocking_send(Err(error));
            }
        }
    }
}

#[async_trait]
impl BackendSession for OpenAiSession {
    async fn add_query_chunk(&self, chunk: &str) -> Result<()> {
        self.shared
            .transcript
            .lock()
            .expect("transcript lock")
            .pending
            .push_str(chunk);
        Ok(())
    }

    async fn generate(&self, config: GenerationConfig) -> Result<AiResponse> {
        self.shared.cancelled.store(false, Ordering::Release);
        let shared = self.shared.clone();
        let guard = AbortOnDrop(Some(self.shared.clone()));
        let result = run_blocking(self.shared.kind, move || shared.generate_blocking(config)).await;
        guard.disarm();
        result
    }

    async fn generate_stream(&self, config: GenerationConfig) -> Result<ResponseStream> {
        self.shared.cancelled.store(false, Ordering::Release);
        let (sender, receiver) = mpsc::channel(64);
        let shared = self.shared.clone();
        thread::Builder::new()
            .name("rust_local_ai-stream".into())
            .spawn(move || shared.stream_blocking(config, sender))
            .map_err(|e| {
                self.shared
                    .error(format!("failed to start stream thread: {e}"))
            })?;
        Ok(Box::pin(ReceiverStream {
            receiver,
            abort: AbortOnDrop(Some(self.shared.clone())),
        }))
    }

    async fn cancel(&self) -> Result<()> {
        self.shared.abort();
        Ok(())
    }

    async fn count_tokens(&self, _: &str) -> Result<u64> {
        Err(LocalAiError::UnsupportedCapability {
            capability: "token_counting",
        })
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

/// Aborts the request in flight if the awaiting future (or the response
/// stream) is dropped before the generation finished.
struct AbortOnDrop(Option<Arc<Shared>>);

impl AbortOnDrop {
    fn disarm(mut self) {
        self.0 = None;
    }
}

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        if let Some(shared) = self.0.take() {
            shared.abort();
        }
    }
}

struct ReceiverStream {
    receiver: mpsc::Receiver<Result<String>>,
    abort: AbortOnDrop,
}

impl Stream for ReceiverStream {
    type Item = Result<String>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let item = self.receiver.poll_recv(cx);
        if matches!(item, Poll::Ready(None)) {
            // Fully consumed: dropping the stream must not cancel anything.
            self.abort.0 = None;
        }
        item
    }
}

fn error_message(body: &[u8]) -> Option<String> {
    let value: Value = serde_json::from_slice(body).ok()?;
    let error = &value["error"];
    error["message"]
        .as_str()
        .or_else(|| error.as_str())
        .map(str::to_owned)
}

fn parse_completion(body: &[u8]) -> std::result::Result<AiResponse, String> {
    let value: Value =
        serde_json::from_slice(body).map_err(|e| format!("invalid JSON response: {e}"))?;
    if let Some(message) = error_message(body) {
        return Err(message);
    }
    let choice = &value["choices"][0];
    if choice.is_null() {
        return Err("response has no choices".into());
    }
    Ok(AiResponse {
        // Reasoning engines may return a null `content` with the text elsewhere;
        // an empty answer is still a valid completion.
        text: choice["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        input_tokens: value["usage"]["prompt_tokens"].as_u64(),
        output_tokens: value["usage"]["completion_tokens"].as_u64(),
        finish_reason: choice["finish_reason"].as_str().map(str::to_owned),
    })
}

/// Extracts the text delta from one streamed chunk.
fn parse_delta(data: &str) -> std::result::Result<String, String> {
    let value: Value =
        serde_json::from_str(data).map_err(|e| format!("invalid stream event: {e}"))?;
    if let Some(message) = error_message(data.as_bytes()) {
        return Err(message);
    }
    Ok(value["choices"][0]["delta"]["content"]
        .as_str()
        .unwrap_or_default()
        .to_owned())
}

enum EventError {
    Io(std::io::Error),
    Handler(LocalAiError),
}

/// Reads a server-sent-events body, calling `on_data` with each event's data
/// until `[DONE]` or the end of the body.
fn read_events(
    mut reader: impl BufRead,
    mut on_data: impl FnMut(&str) -> Result<()>,
) -> std::result::Result<(), EventError> {
    const MAX_EVENT_BYTES: u64 = 4 * 1024 * 1024;
    let mut data = String::new();
    let mut line = String::new();
    loop {
        line.clear();
        let read = Read::take(&mut reader, MAX_EVENT_BYTES)
            .read_line(&mut line)
            .map_err(EventError::Io)?;
        let at_end = read == 0;
        let field = line.trim_end_matches(['\r', '\n']);
        if at_end || field.is_empty() {
            // A blank line (or the end of the body) dispatches the event.
            if !data.is_empty() {
                if data == "[DONE]" {
                    return Ok(());
                }
                on_data(&data).map_err(EventError::Handler)?;
                data.clear();
            }
            if at_end {
                return Ok(());
            }
        } else if let Some(value) = field.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(value.strip_prefix(' ').unwrap_or(value));
        }
        // Comments (`:`), `event:`, `id:` and `retry:` lines are ignored.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_completions_and_errors() {
        let ok = br#"{"choices":[{"message":{"role":"assistant","content":"Hi"},"finish_reason":"stop"}],
                     "usage":{"prompt_tokens":11,"completion_tokens":2}}"#;
        let response = parse_completion(ok).unwrap();
        assert_eq!(response.text, "Hi");
        assert_eq!(response.input_tokens, Some(11));
        assert_eq!(response.output_tokens, Some(2));
        assert_eq!(response.finish_reason.as_deref(), Some("stop"));

        let null_content = br#"{"choices":[{"message":{"content":null}}]}"#;
        assert_eq!(parse_completion(null_content).unwrap().text, "");

        assert_eq!(
            parse_completion(br#"{"error":{"message":"model not loaded"}}"#).unwrap_err(),
            "model not loaded"
        );
        assert!(parse_completion(br#"{"choices":[]}"#).is_err());
        assert!(parse_completion(b"not json").is_err());
    }

    #[test]
    fn reads_sse_events() {
        let body = ": comment\r\n\
                    data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\r\n\r\n\
                    event: message\ndata: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n\
                    data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n\
                    data: {\"choices\":[],\"usage\":{\"completion_tokens\":2}}\n\n\
                    data: [DONE]\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"ignored\"}}]}\n\n";
        let mut text = String::new();
        read_events(body.as_bytes(), |data| {
            text.push_str(&parse_delta(data).unwrap());
            Ok(())
        })
        .unwrap_or_else(|_| panic!("events parse"));
        assert_eq!(text, "Hello");
    }

    #[test]
    fn multi_line_data_and_missing_done_are_handled() {
        let body = "data: {\"choices\":\ndata: [{\"delta\":{\"content\":\"x\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"y\"}}]}";
        let mut text = String::new();
        read_events(body.as_bytes(), |data| {
            text.push_str(&parse_delta(data).unwrap());
            Ok(())
        })
        .unwrap_or_else(|_| panic!("events parse"));
        assert_eq!(text, "xy");
    }

    #[test]
    fn stream_errors_surface() {
        assert_eq!(
            parse_delta(r#"{"error":{"message":"overloaded"}}"#).unwrap_err(),
            "overloaded"
        );
    }
}
