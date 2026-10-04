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
    if cfg!(target_os = "macos") {
        "Apple Foundation Models (adapter unavailable in this build)"
    } else if cfg!(target_os = "windows") {
        "Windows AI (adapter unavailable in this build)"
    } else if cfg!(target_os = "linux") {
        "Linux (no managed provider detected)"
    } else {
        "Unsupported operating system"
    }
}

fn reason() -> AvailabilityReason {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        AvailabilityReason::UnsupportedOsVersion
    } else if cfg!(target_os = "linux") {
        AvailabilityReason::ProviderNotInstalled
    } else {
        AvailabilityReason::UnsupportedOperatingSystem
    }
}

fn detail() -> &'static str {
    "No working OS-managed adapter is compiled; rust_local_ai never downloads a model automatically"
}
