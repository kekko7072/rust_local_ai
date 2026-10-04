#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

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

/// Detects the best OS/distribution-managed backend available in this build.
///
/// Detection never installs a provider, starts a service, or downloads a model.
pub async fn detect() -> Result<LocalAiModel> {
    Ok(LocalAiModel::from_backend(backend::platform_backend()))
}

#[cfg(any(test, feature = "testing"))]
pub mod testing;
