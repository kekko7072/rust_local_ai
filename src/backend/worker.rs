//! Runs blocking work (native calls, local HTTP) off the async executor.

use std::thread;

use tokio::sync::oneshot;

use crate::{BackendKind, LocalAiError, Result};

/// Runs `work` on a dedicated thread and awaits its result.
///
/// Uses a plain thread and a oneshot channel rather than a runtime's blocking
/// pool, so the crate works under any async executor. If the awaiting future
/// is dropped, the thread still runs to completion; callers own any state it
/// needs (for example through an `Arc`).
pub(crate) async fn run_blocking<T: Send + 'static>(
    backend: BackendKind,
    work: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T> {
    let (sender, receiver) = oneshot::channel();
    thread::Builder::new()
        .name("rust_local_ai-worker".into())
        .spawn(move || {
            let _ = sender.send(work());
        })
        .map_err(|error| LocalAiError::Backend {
            backend,
            message: format!("failed to start worker thread: {error}"),
        })?;
    receiver.await.map_err(|_| LocalAiError::Backend {
        backend,
        message: "worker thread panicked".into(),
    })?
}
