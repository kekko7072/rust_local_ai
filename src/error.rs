use std::fmt;

use crate::{AvailabilityReason, BackendKind};

pub type Result<T> = std::result::Result<T, LocalAiError>;

#[derive(Debug)]
#[non_exhaustive]
pub enum LocalAiError {
    Unavailable {
        reason: AvailabilityReason,
    },
    SessionBusy,
    SessionClosed,
    ModelClosed,
    UnsupportedCapability {
        capability: &'static str,
    },
    InvalidConfig(String),
    Backend {
        backend: BackendKind,
        message: String,
    },
    Cancelled,
    InvalidModelOutput(String),
}

impl fmt::Display for LocalAiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable { reason } => write!(f, "local AI is unavailable: {reason:?}"),
            Self::SessionBusy => f.write_str("the session already has a generation in progress"),
            Self::SessionClosed => f.write_str("the session is closed"),
            Self::ModelClosed => f.write_str("the model is closed"),
            Self::UnsupportedCapability { capability } => {
                write!(f, "the backend does not support {capability}")
            }
            Self::InvalidConfig(message) => {
                write!(f, "invalid generation configuration: {message}")
            }
            Self::Backend { backend, message } => write!(f, "{backend:?} backend error: {message}"),
            Self::Cancelled => f.write_str("generation was cancelled"),
            Self::InvalidModelOutput(message) => {
                write!(f, "the model output could not be used: {message}")
            }
        }
    }
}

impl std::error::Error for LocalAiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_stable() {
        assert_eq!(
            LocalAiError::Unavailable {
                reason: AvailabilityReason::ModelNotReady
            }
            .to_string(),
            "local AI is unavailable: ModelNotReady"
        );
        assert_eq!(
            LocalAiError::UnsupportedCapability {
                capability: "streaming"
            }
            .to_string(),
            "the backend does not support streaming"
        );
        assert_eq!(
            LocalAiError::Backend {
                backend: BackendKind::Fake,
                message: "boom".into()
            }
            .to_string(),
            "Fake backend error: boom"
        );
        assert_eq!(
            LocalAiError::InvalidModelOutput("no JSON".into()).to_string(),
            "the model output could not be used: no JSON"
        );
    }
}
