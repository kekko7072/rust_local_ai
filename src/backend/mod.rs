use std::{pin::Pin, sync::Arc};

use async_trait::async_trait;
use futures_core::Stream;

use crate::{AiResponse, Availability, BackendInfo, Capabilities, GenerationConfig, Result};

pub(crate) mod http;
pub(crate) mod openai;
mod worker;

pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<String>> + Send + 'static>>;

/// Adapter contract for an OS or distribution managed AI runtime.
///
/// Implementations must not download models, install providers, or launch a
/// heavyweight service as a side effect of these methods.
#[async_trait]
pub trait Backend: Send + Sync {
    fn info(&self) -> BackendInfo;
    async fn availability(&self) -> Availability;
    async fn capabilities(&self) -> Capabilities;
    async fn open_session(&self, instructions: Option<&str>) -> Result<Arc<dyn BackendSession>>;

    /// Explicitly prepares the model, which may make the OS download it.
    ///
    /// This is the only method allowed to trigger a download, and only because
    /// the application asked for it. The default does nothing and reports the
    /// current availability.
    async fn prepare(&self) -> Result<Availability> {
        Ok(self.availability().await)
    }
}

#[async_trait]
pub trait BackendSession: Send + Sync {
    async fn add_query_chunk(&self, chunk: &str) -> Result<()>;
    async fn generate(&self, config: GenerationConfig) -> Result<AiResponse>;
    async fn generate_stream(&self, config: GenerationConfig) -> Result<ResponseStream>;
    async fn cancel(&self) -> Result<()>;
    async fn count_tokens(&self, text: &str) -> Result<u64>;
    async fn close(&self) -> Result<()>;
}

#[cfg(target_os = "macos")]
mod apple;
#[cfg(target_os = "linux")]
pub(crate) mod linux;
#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
mod unsupported;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
pub(crate) use apple::platform_backend;
#[cfg(target_os = "linux")]
pub(crate) use linux::platform_backend;
#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
pub(crate) use unsupported::platform_backend;
#[cfg(windows)]
pub(crate) use windows::platform_backend;
