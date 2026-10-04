use std::{pin::Pin, sync::Arc};

use async_trait::async_trait;
use futures_core::Stream;

use crate::{AiResponse, Availability, BackendInfo, Capabilities, GenerationConfig, Result};

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

mod unsupported;
pub(crate) use unsupported::platform_backend;
