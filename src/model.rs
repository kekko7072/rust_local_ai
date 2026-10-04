use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{
    backend::Backend, Availability, BackendInfo, Capabilities, LocalAiError, LocalAiSession, Result,
};

#[derive(Clone)]
pub struct LocalAiModel {
    backend: Arc<dyn Backend>,
    closed: Arc<AtomicBool>,
}

impl std::fmt::Debug for LocalAiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalAiModel")
            .field("backend", &self.backend.info())
            .field("closed", &self.closed.load(Ordering::Acquire))
            .finish()
    }
}

impl LocalAiModel {
    pub fn from_backend(backend: Arc<dyn Backend>) -> Self {
        Self {
            backend,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn backend(&self) -> BackendInfo {
        self.backend.info()
    }
    pub async fn availability(&self) -> Availability {
        self.backend.availability().await
    }
    pub async fn capabilities(&self) -> Capabilities {
        self.backend.capabilities().await
    }

    /// Explicitly prepares the model and returns the resulting availability.
    ///
    /// On Windows this asks the OS to install Phi Silica, which can be a large
    /// download: obtain the user's consent first. Backends whose model is
    /// prepared by the system (Apple) or by the provider (inference snaps)
    /// just report their availability. No other call ever triggers a download.
    pub async fn prepare(&self) -> Result<Availability> {
        if self.closed.load(Ordering::Acquire) {
            return Err(LocalAiError::ModelClosed);
        }
        self.backend.prepare().await
    }

    pub async fn open_session(&self, instructions: Option<&str>) -> Result<LocalAiSession> {
        if self.closed.load(Ordering::Acquire) {
            return Err(LocalAiError::ModelClosed);
        }
        let availability = self.availability().await;
        if !availability.available {
            return Err(LocalAiError::Unavailable {
                reason: availability
                    .reason
                    .unwrap_or(crate::AvailabilityReason::BackendFailure),
            });
        }
        let capabilities = self.capabilities().await;
        let session = self.backend.open_session(instructions).await?;
        Ok(LocalAiSession::new(session, capabilities))
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}
