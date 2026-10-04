use std::sync::Arc;

use async_trait::async_trait;

use super::{Backend, BackendSession};
use crate::{
    Availability, AvailabilityReason, BackendInfo, BackendKind, Capabilities, LocalAiError, Result,
};

pub(crate) fn platform_backend() -> Arc<dyn Backend> {
    Arc::new(UnsupportedBackend)
}

struct UnsupportedBackend;

#[async_trait]
impl Backend for UnsupportedBackend {
    fn info(&self) -> BackendInfo {
        BackendInfo {
            kind: BackendKind::Unsupported,
            name: platform_name().into(),
            system_managed_model: false,
        }
    }

    async fn availability(&self) -> Availability {
        Availability::unavailable(reason()).with_detail(detail())
    }

    async fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }

    async fn open_session(&self, _: Option<&str>) -> Result<Arc<dyn BackendSession>> {
        Err(LocalAiError::Unavailable { reason: reason() })
    }
}

fn platform_name() -> &'static str {
    "Unsupported operating system"
}

fn reason() -> AvailabilityReason {
    AvailabilityReason::UnsupportedOperatingSystem
}

fn detail() -> &'static str {
    "rust_local_ai has adapters for macOS, Windows and Linux; on other systems use \
     rust_local_ai::openai_compatible with a local provider"
}
