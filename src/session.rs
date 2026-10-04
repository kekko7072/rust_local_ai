use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use futures_core::Stream;
use pin_project_lite::pin_project;
use tokio::sync::Mutex;

use crate::{
    backend::{BackendSession, ResponseStream},
    AiResponse, Capabilities, GenerationConfig, LocalAiError, ResponseFormat, Result,
};

#[derive(Clone)]
pub struct LocalAiSession {
    inner: Arc<SessionInner>,
}

struct SessionInner {
    backend: Arc<dyn BackendSession>,
    capabilities: Capabilities,
    closed: AtomicBool,
    generating: Arc<AtomicBool>,
    close_lock: Mutex<()>,
}

impl std::fmt::Debug for LocalAiSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalAiSession")
            .field("closed", &self.is_closed())
            .field("capabilities", &self.inner.capabilities)
            .finish()
    }
}

impl LocalAiSession {
    pub(crate) fn new(backend: Arc<dyn BackendSession>, capabilities: Capabilities) -> Self {
        Self {
            inner: Arc::new(SessionInner {
                backend,
                capabilities,
                closed: AtomicBool::new(false),
                generating: Arc::new(AtomicBool::new(false)),
                close_lock: Mutex::new(()),
            }),
        }
    }

    fn ensure_open(&self) -> Result<()> {
        if self.is_closed() {
            Err(LocalAiError::SessionClosed)
        } else {
            Ok(())
        }
    }

    fn require(&self, supported: bool, capability: &'static str) -> Result<()> {
        if supported {
            Ok(())
        } else {
            Err(LocalAiError::UnsupportedCapability { capability })
        }
    }

    fn acquire_generation(&self) -> Result<GenerationLease> {
        self.ensure_open()?;
        self.inner
            .generating
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| LocalAiError::SessionBusy)?;
        Ok(GenerationLease {
            busy: self.inner.generating.clone(),
        })
    }

    pub fn capabilities(&self) -> Capabilities {
        self.inner.capabilities
    }
    pub fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::Acquire)
    }

    pub async fn add_query_chunk(&self, chunk: &str) -> Result<()> {
        self.ensure_open()?;
        self.inner.backend.add_query_chunk(chunk).await
    }

    pub async fn generate(&self, config: GenerationConfig) -> Result<AiResponse> {
        self.require(self.inner.capabilities.text_generation, "text_generation")?;
        require_format(self.inner.capabilities, &config.response_format)?;
        config.validate()?;
        let _lease = self.acquire_generation()?;
        self.inner.backend.generate(config).await
    }

    pub async fn generate_stream(&self, config: GenerationConfig) -> Result<ResponseStream> {
        self.require(self.inner.capabilities.streaming, "streaming")?;
        require_format(self.inner.capabilities, &config.response_format)?;
        config.validate()?;
        let lease = self.acquire_generation()?;
        let stream = self.inner.backend.generate_stream(config).await?;
        Ok(Box::pin(LeaseStream {
            stream,
            _lease: lease,
        }))
    }

    pub async fn cancel(&self) -> Result<()> {
        self.ensure_open()?;
        self.require(self.inner.capabilities.cancellation, "cancellation")?;
        self.inner.backend.cancel().await
    }

    pub async fn count_tokens(&self, text: &str) -> Result<u64> {
        self.ensure_open()?;
        self.require(self.inner.capabilities.token_counting, "token_counting")?;
        self.inner.backend.count_tokens(text).await
    }

    pub async fn close(&self) -> Result<()> {
        let _guard = self.inner.close_lock.lock().await;
        if self.is_closed() {
            return Ok(());
        }
        if self.inner.generating.load(Ordering::Acquire) {
            return Err(LocalAiError::SessionBusy);
        }
        self.inner.backend.close().await?;
        self.inner.closed.store(true, Ordering::Release);
        Ok(())
    }
}

fn require_format(capabilities: Capabilities, format: &ResponseFormat) -> Result<()> {
    if !matches!(format, ResponseFormat::Text) && !capabilities.structured_output {
        Err(LocalAiError::UnsupportedCapability {
            capability: "structured_output",
        })
    } else {
        Ok(())
    }
}

struct GenerationLease {
    busy: Arc<AtomicBool>,
}
impl Drop for GenerationLease {
    fn drop(&mut self) {
        self.busy.store(false, Ordering::Release);
    }
}

pin_project! {
    struct LeaseStream {
        #[pin]
        stream: ResponseStream,
        _lease: GenerationLease,
    }
}

impl Stream for LeaseStream {
    type Item = Result<String>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }
}
