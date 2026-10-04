use crate::{AvailabilityReason, BackendKind};

pub type Result<T> = std::result::Result<T, LocalAiError>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LocalAiError {
    #[error("local AI is unavailable: {reason:?}")]
    Unavailable { reason: AvailabilityReason },
    #[error("the session already has a generation in progress")]
    SessionBusy,
    #[error("the session is closed")]
    SessionClosed,
    #[error("the model is closed")]
    ModelClosed,
    #[error("the backend does not support {capability}")]
    UnsupportedCapability { capability: &'static str },
    #[error("invalid generation configuration: {0}")]
    InvalidConfig(String),
    #[error("{backend:?} backend error: {message}")]
    Backend {
        backend: BackendKind,
        message: String,
    },
    #[error("generation was cancelled")]
    Cancelled,
}
