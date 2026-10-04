//! Deterministic backend for contract tests and downstream integration tests.

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use async_trait::async_trait;
use futures_util::stream;

use crate::{
    AiResponse, Availability, Backend, BackendInfo, BackendKind, BackendSession, Capabilities,
    GenerationConfig, LocalAiError, LocalAiModel, ResponseStream, Result,
};

#[derive(Clone)]
pub struct FakeBackend {
    state: Arc<State>,
}

struct State {
    availability: Mutex<Availability>,
    capabilities: Mutex<Capabilities>,
    responses: Mutex<VecDeque<Result<AiResponse>>>,
    stream_chunks: Mutex<Vec<Result<String>>>,
    close_failures: AtomicUsize,
    generation_blocked: AtomicBool,
}

impl Default for FakeBackend {
    fn default() -> Self {
        Self::new(Capabilities {
            text_generation: true,
            streaming: true,
            structured_output: true,
            token_counting: true,
            cancellation: true,
            concurrent_sessions: true,
            ..Capabilities::default()
        })
    }
}

impl FakeBackend {
    pub fn new(capabilities: Capabilities) -> Self {
        Self {
            state: Arc::new(State {
                availability: Mutex::new(Availability::available()),
                capabilities: Mutex::new(capabilities),
                responses: Mutex::new(VecDeque::new()),
                stream_chunks: Mutex::new(vec![]),
                close_failures: AtomicUsize::new(0),
                generation_blocked: AtomicBool::new(false),
            }),
        }
    }

    pub fn model(&self) -> LocalAiModel {
        LocalAiModel::from_backend(Arc::new(self.clone()))
    }
    pub fn set_availability(&self, value: Availability) {
        *self.state.availability.lock().expect("fake lock") = value;
    }
    pub fn push_response(&self, value: Result<AiResponse>) {
        self.state
            .responses
            .lock()
            .expect("fake lock")
            .push_back(value);
    }
    pub fn set_stream_chunks(&self, chunks: Vec<Result<String>>) {
        *self.state.stream_chunks.lock().expect("fake lock") = chunks;
    }
    pub fn fail_next_closes(&self, count: usize) {
        self.state.close_failures.store(count, Ordering::Release);
    }
    pub fn set_generation_blocked(&self, value: bool) {
        self.state
            .generation_blocked
            .store(value, Ordering::Release);
    }
}

#[async_trait]
impl Backend for FakeBackend {
    fn info(&self) -> BackendInfo {
        BackendInfo {
            kind: BackendKind::Fake,
            name: "deterministic fake".into(),
            system_managed_model: false,
        }
    }
    async fn availability(&self) -> Availability {
        self.state.availability.lock().expect("fake lock").clone()
    }
    async fn capabilities(&self) -> Capabilities {
        *self.state.capabilities.lock().expect("fake lock")
    }
    async fn open_session(&self, _: Option<&str>) -> Result<Arc<dyn BackendSession>> {
        Ok(Arc::new(FakeSession {
            state: self.state.clone(),
            prompt: Mutex::new(String::new()),
            cancelled: AtomicBool::new(false),
        }))
    }
}

struct FakeSession {
    state: Arc<State>,
    prompt: Mutex<String>,
    cancelled: AtomicBool,
}

#[async_trait]
impl BackendSession for FakeSession {
    async fn add_query_chunk(&self, chunk: &str) -> Result<()> {
        self.prompt.lock().expect("fake lock").push_str(chunk);
        Ok(())
    }

    async fn generate(&self, _: GenerationConfig) -> Result<AiResponse> {
        while self.state.generation_blocked.load(Ordering::Acquire) {
            tokio::task::yield_now().await;
        }
        if self.cancelled.swap(false, Ordering::AcqRel) {
            return Err(LocalAiError::Cancelled);
        }
        if let Some(response) = self.state.responses.lock().expect("fake lock").pop_front() {
            response
        } else {
            Ok(AiResponse::text(
                self.prompt.lock().expect("fake lock").clone(),
            ))
        }
    }

    async fn generate_stream(&self, _: GenerationConfig) -> Result<ResponseStream> {
        let chunks = std::mem::take(&mut *self.state.stream_chunks.lock().expect("fake lock"));
        Ok(Box::pin(stream::iter(chunks)))
    }

    async fn cancel(&self) -> Result<()> {
        self.cancelled.store(true, Ordering::Release);
        Ok(())
    }
    async fn count_tokens(&self, text: &str) -> Result<u64> {
        Ok(text.split_whitespace().count() as u64)
    }
    async fn close(&self) -> Result<()> {
        let remaining = self.state.close_failures.load(Ordering::Acquire);
        if remaining > 0 {
            self.state.close_failures.fetch_sub(1, Ordering::AcqRel);
            Err(LocalAiError::Backend {
                backend: BackendKind::Fake,
                message: "injected close failure".into(),
            })
        } else {
            Ok(())
        }
    }
}
