#![deny(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]

use std::sync::Arc;

mod backend;
mod error;
mod model;
mod session;
mod types;

pub use backend::{Backend, BackendSession, ResponseStream};
pub use error::{LocalAiError, Result};
pub use model::LocalAiModel;
pub use session::LocalAiSession;
pub use types::*;

/// Detects the best OS/distribution-managed backend available in this build:
/// Apple Foundation Models on macOS, Windows AI (Phi Silica) on Windows, and
/// Ubuntu inference snaps on Linux.
///
/// Detection never installs a provider, starts a service, or downloads a model.
/// Check [`LocalAiModel::availability`] before use.
pub async fn detect() -> Result<LocalAiModel> {
    Ok(LocalAiModel::from_backend(backend::platform_backend()))
}

/// Uses a local service that speaks the OpenAI chat-completions API, such as
/// llama.cpp's server, Ollama, LM Studio or Foundry Local.
///
/// `base_url` is the API root, e.g. `http://localhost:11434/v1`. Only plain
/// `http://` URLs on this machine (`localhost`, `127.0.0.0/8`, `::1`) are
/// accepted, so prompts never leave it. When `model` is `None`, the first model
/// listed by the service's `/models` endpoint is used. Nothing is contacted
/// until the model is used.
pub fn openai_compatible(base_url: &str, model: Option<&str>) -> Result<LocalAiModel> {
    let service = backend::openai::Service::new(base_url, model.map(str::to_owned))?;
    Ok(LocalAiModel::from_backend(Arc::new(
        backend::openai::OpenAiBackend { service },
    )))
}

/// Uses a specific Ubuntu inference snap (for example `"qwen3"`) instead of the
/// first suitable one [`detect`] would pick.
///
/// Setting the `RUST_LOCAL_AI_INFERENCE_SNAP` environment variable has the
/// same effect on [`detect`].
#[cfg(target_os = "linux")]
pub fn inference_snap(name: &str) -> LocalAiModel {
    LocalAiModel::from_backend(backend::linux::backend(Some(name.to_owned())))
}

#[cfg(feature = "genui")]
pub mod genui;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
