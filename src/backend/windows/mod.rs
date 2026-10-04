//! Windows: Phi Silica through the Windows App SDK `LanguageModel` API.
//!
//! Microsoft gates this API at runtime: it needs a Copilot+ PC (NPU) or a
//! supported GPU, Windows 11 25H2 or later, and a process with package
//! identity that declares the `systemAIModels` capability (the stable Windows
//! App SDK channel also requires a Limited Access Feature unlock). Every gate
//! is reported through [`Availability`]; nothing here assumes readiness from
//! the OS version.
//!
//! `LanguageModel` is stateless per call, so the session keeps the transcript
//! and replays it, like flutter_local_ai's Windows host.

#[allow(clippy::all, unused_qualifications, missing_docs)]
#[rustfmt::skip]
mod bindings;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use async_trait::async_trait;
use windows_core::{RuntimeType, HRESULT, HSTRING};

use self::bindings::{
    Microsoft::Windows::AI::{
        AIFeatureReadyResultState, AIFeatureReadyState,
        Text::{LanguageModel, LanguageModelOptions, LanguageModelResponseStatus},
    },
    Windows::Foundation::{AsyncStatus, IAsyncOperation, IAsyncOperationWithProgress},
};
use super::{worker, Backend, BackendSession, ResponseStream};
use crate::{
    AiResponse, Availability, AvailabilityReason, BackendInfo, BackendKind, Capabilities,
    GenerationConfig, LocalAiError, Result,
};

const KIND: BackendKind = BackendKind::WindowsAi;
const REGDB_E_CLASSNOTREG: HRESULT = HRESULT(0x8004_0154_u32 as i32);
const E_NOINTERFACE: HRESULT = HRESULT(0x8000_4002_u32 as i32);

pub(crate) fn platform_backend() -> Arc<dyn Backend> {
    Arc::new(WindowsAiBackend)
}

struct WindowsAiBackend;

#[async_trait]
impl Backend for WindowsAiBackend {
    fn info(&self) -> BackendInfo {
        BackendInfo {
            kind: KIND,
            name: "Windows AI (Phi Silica)".into(),
            system_managed_model: true,
        }
    }

    async fn availability(&self) -> Availability {
        worker::run_blocking(KIND, || Ok(availability_now()))
            .await
            .unwrap_or_else(|error| {
                Availability::unavailable(AvailabilityReason::BackendFailure)
                    .with_detail(error.to_string())
            })
    }

    async fn capabilities(&self) -> Capabilities {
        Capabilities {
            text_generation: true,
            cancellation: true,
            concurrent_sessions: true,
            system_managed_model: true,
            // Streaming, structured output and exact token counts are not
            // verified on hardware yet, so they are not claimed.
            ..Capabilities::default()
        }
    }

    async fn prepare(&self) -> Result<Availability> {
        worker::run_blocking(KIND, || {
            let operation = LanguageModel::EnsureReadyAsync().map_err(win_error)?;
            let result = wait(&operation, &AtomicBool::new(false))?;
            match result.Status().map_err(win_error)? {
                AIFeatureReadyResultState::Success => Ok(()),
                _ => Err(backend_error(format!(
                    "Windows AI preparation failed: {} ({})",
                    result
                        .ErrorDisplayText()
                        .map(|t| t.to_string())
                        .unwrap_or_default(),
                    result
                        .ExtendedError()
                        .map(|code| format!("{:#010X}", code.0))
                        .unwrap_or_default()
                ))),
            }
        })
        .await?;
        Ok(self.availability().await)
    }

    async fn open_session(&self, instructions: Option<&str>) -> Result<Arc<dyn BackendSession>> {
        let transcript = match instructions.filter(|i| !i.is_empty()) {
            Some(instructions) => format!("{instructions}\n\n"),
            None => String::new(),
        };
        Ok(Arc::new(WindowsAiSession {
            shared: Arc::new(Shared {
                transcript: Mutex::new(transcript),
                cancelled: AtomicBool::new(false),
            }),
        }))
    }
}

fn availability_now() -> Availability {
    match LanguageModel::GetReadyState() {
        Ok(AIFeatureReadyState::Ready) => Availability::available(),
        Ok(AIFeatureReadyState::NotReady) => {
            Availability::unavailable(AvailabilityReason::ModelNotReady).with_detail(
                "the Phi Silica model is not installed yet; call LocalAiModel::prepare() \
             (with the user's consent: Windows may download it)",
            )
        }
        Ok(AIFeatureReadyState::NotSupportedOnCurrentSystem)
        | Ok(AIFeatureReadyState::NotCompatibleWithSystemHardware) => Availability::unavailable(
            AvailabilityReason::UnsupportedHardware,
        )
        .with_detail("Phi Silica needs a Copilot+ PC or a supported GPU with current drivers"),
        Ok(AIFeatureReadyState::DisabledByUser) => {
            Availability::unavailable(AvailabilityReason::SystemFeatureDisabled)
                .with_detail("generative AI is turned off in Windows settings or by policy")
        }
        Ok(AIFeatureReadyState::CapabilityMissing) => {
            Availability::unavailable(AvailabilityReason::SystemFeatureDisabled)
                .with_detail("the app package must declare the systemAIModels capability")
        }
        Ok(AIFeatureReadyState::OSUpdateNeeded) => {
            Availability::unavailable(AvailabilityReason::UnsupportedOsVersion)
                .with_detail("Windows needs an update before Phi Silica can run")
        }
        Ok(other) => Availability::unavailable(AvailabilityReason::BackendFailure)
            .with_detail(format!("unknown Windows AI ready state {}", other.0)),
        Err(error) if error.code() == REGDB_E_CLASSNOTREG => {
            Availability::unavailable(AvailabilityReason::ProviderNotInstalled).with_detail(
                "the Windows App SDK AI runtime is not available to this process; it must \
                 run with package identity and the Windows App Runtime installed",
            )
        }
        Err(error) => Availability::unavailable(AvailabilityReason::BackendFailure)
            .with_detail(describe(&error)),
    }
}

struct WindowsAiSession {
    shared: Arc<Shared>,
}

struct Shared {
    transcript: Mutex<String>,
    cancelled: AtomicBool,
}

impl Shared {
    fn generate_blocking(&self, config: GenerationConfig) -> Result<AiResponse> {
        let prompt = HSTRING::from(self.transcript.lock().expect("transcript lock").as_str());

        let model = wait(
            &LanguageModel::CreateAsync().map_err(win_error)?,
            &self.cancelled,
        )?;
        let result = (|| {
            let options = LanguageModelOptions::new().map_err(win_error)?;
            if let Some(temperature) = config.temperature {
                options.SetTemperature(temperature).map_err(win_error)?;
            }
            if let Some(top_p) = config.top_p {
                options.SetTopP(top_p).map_err(win_error)?;
            }
            // `LanguageModelOptions` has no output-token limit or seed, so
            // `max_output_tokens` and `seed` cannot be applied here.
            let operation = model
                .GenerateResponseAsync2(&prompt, &options)
                .map_err(win_error)?;
            let response = wait(&operation, &self.cancelled)?;
            let status = response.Status().map_err(win_error)?;
            if let Some(error) = status_error(status) {
                return Err(error);
            }
            Ok(response.Text().map_err(win_error)?.to_string())
        })();
        let _ = model.Close();

        let text = result?;
        self.transcript
            .lock()
            .expect("transcript lock")
            .push_str(&format!("\n\n{text}\n\n"));
        Ok(AiResponse::text(text))
    }
}

#[async_trait]
impl BackendSession for WindowsAiSession {
    async fn add_query_chunk(&self, chunk: &str) -> Result<()> {
        self.shared
            .transcript
            .lock()
            .expect("transcript lock")
            .push_str(chunk);
        Ok(())
    }

    async fn generate(&self, config: GenerationConfig) -> Result<AiResponse> {
        self.shared.cancelled.store(false, Ordering::Release);
        let shared = self.shared.clone();
        let guard = CancelOnDrop(Some(self.shared.clone()));
        let result = worker::run_blocking(KIND, move || shared.generate_blocking(config)).await;
        guard.disarm();
        result
    }

    async fn generate_stream(&self, _: GenerationConfig) -> Result<ResponseStream> {
        Err(LocalAiError::UnsupportedCapability {
            capability: "streaming",
        })
    }

    async fn cancel(&self) -> Result<()> {
        self.shared.cancelled.store(true, Ordering::Release);
        Ok(())
    }

    async fn count_tokens(&self, _: &str) -> Result<u64> {
        Err(LocalAiError::UnsupportedCapability {
            capability: "token_counting",
        })
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

/// Cancels the generation if the awaiting future is dropped early.
struct CancelOnDrop(Option<Arc<Shared>>);

impl CancelOnDrop {
    fn disarm(mut self) {
        self.0 = None;
    }
}

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if let Some(shared) = self.0.take() {
            shared.cancelled.store(true, Ordering::Release);
        }
    }
}

/// The parts of `IAsyncOperation` / `IAsyncOperationWithProgress` needed to
/// wait for a result.
trait AsyncOperation {
    type Output;
    fn status(&self) -> windows_core::Result<AsyncStatus>;
    fn error_code(&self) -> windows_core::Result<HRESULT>;
    fn cancel(&self) -> windows_core::Result<()>;
    fn results(&self) -> windows_core::Result<Self::Output>;
}

impl<T: RuntimeType + 'static> AsyncOperation for IAsyncOperation<T> {
    type Output = T;
    fn status(&self) -> windows_core::Result<AsyncStatus> {
        self.Status()
    }
    fn error_code(&self) -> windows_core::Result<HRESULT> {
        self.ErrorCode()
    }
    fn cancel(&self) -> windows_core::Result<()> {
        self.Cancel()
    }
    fn results(&self) -> windows_core::Result<T> {
        self.GetResults()
    }
}

impl<T: RuntimeType + 'static, P: RuntimeType + 'static> AsyncOperation
    for IAsyncOperationWithProgress<T, P>
{
    type Output = T;
    fn status(&self) -> windows_core::Result<AsyncStatus> {
        self.Status()
    }
    fn error_code(&self) -> windows_core::Result<HRESULT> {
        self.ErrorCode()
    }
    fn cancel(&self) -> windows_core::Result<()> {
        self.Cancel()
    }
    fn results(&self) -> windows_core::Result<T> {
        self.GetResults()
    }
}

/// Blocks the worker thread until `operation` finishes, cancelling it when
/// `cancelled` is set. Polling keeps this free of hand-written COM delegates.
fn wait<O: AsyncOperation>(operation: &O, cancelled: &AtomicBool) -> Result<O::Output> {
    let mut delay = Duration::from_millis(2);
    let mut cancel_sent = false;
    loop {
        match operation.status().map_err(win_error)? {
            AsyncStatus::Completed => return operation.results().map_err(win_error),
            AsyncStatus::Canceled => return Err(LocalAiError::Cancelled),
            AsyncStatus::Error => {
                let code = operation.error_code().map_err(win_error)?;
                return Err(win_error(windows_core::Error::from_hresult(code)));
            }
            _ => {
                if cancelled.load(Ordering::Acquire) && !cancel_sent {
                    // Best effort: the status loop observes the outcome.
                    let _ = operation.cancel();
                    cancel_sent = true;
                }
                thread::sleep(delay);
                delay = (delay * 2).min(Duration::from_millis(25));
            }
        }
    }
}

fn status_error(status: LanguageModelResponseStatus) -> Option<LocalAiError> {
    let message = match status {
        LanguageModelResponseStatus::Complete => return None,
        LanguageModelResponseStatus::BlockedByPolicy => {
            "generative AI is blocked by system or user policy on this device"
        }
        LanguageModelResponseStatus::PromptLargerThanContext => {
            "the conversation no longer fits the Windows AI context window; \
             close the session and start a new one"
        }
        LanguageModelResponseStatus::PromptBlockedByContentModeration => {
            "the prompt was blocked by Windows content moderation"
        }
        LanguageModelResponseStatus::ResponseBlockedByContentModeration => {
            "the response was blocked by Windows content moderation"
        }
        LanguageModelResponseStatus::UnsupportedLanguage
        | LanguageModelResponseStatus::LanguageMismatch => {
            "Windows AI does not support the prompt's language"
        }
        other => {
            return Some(backend_error(format!(
                "Windows AI did not complete the response (status {})",
                other.0
            )))
        }
    };
    Some(backend_error(message))
}

fn describe(error: &windows_core::Error) -> String {
    format!("{} ({:#010X})", error.message(), error.code().0)
}

fn win_error(error: windows_core::Error) -> LocalAiError {
    if error.code() == E_NOINTERFACE {
        return backend_error(
            "the installed Windows App Runtime is older than this crate's Windows AI API; \
             deploy Windows App SDK 2.0 or newer",
        );
    }
    backend_error(describe(&error))
}

fn backend_error(message: impl Into<String>) -> LocalAiError {
    LocalAiError::Backend {
        backend: KIND,
        message: message.into(),
    }
}
