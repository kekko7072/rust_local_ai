//! Small safe wrapper around the Swift Foundation Models bridge.
//!
//! All unsafe Rust in the crate is confined to this module. The C ABI owns
//! returned strings until `rla_string_free`, and session handles are retained
//! by Swift from create until exactly one destroy call in `Drop`.

use std::{
    ffi::{c_char, c_void, CStr, CString},
    ptr,
    sync::Arc,
};

use async_trait::async_trait;

use super::{Backend, BackendSession, ResponseStream};
use crate::{
    AiResponse, Availability, AvailabilityReason, BackendInfo, BackendKind, Capabilities,
    GenerationConfig, LocalAiError, Result,
};

unsafe extern "C" {
    fn rla_apple_availability(detail: *mut *mut c_char) -> i32;
    fn rla_apple_capabilities() -> u32;
    fn rla_apple_session_create(
        instructions: *const c_char,
        error: *mut *mut c_char,
    ) -> *mut c_void;
    fn rla_apple_session_add(session: *mut c_void, text: *const c_char) -> i32;
    fn rla_apple_session_generate(
        session: *mut c_void,
        temperature: f64,
        top_p: f64,
        maximum_tokens: i32,
        output: *mut *mut c_char,
        error: *mut *mut c_char,
    ) -> i32;
    fn rla_apple_session_cancel(session: *mut c_void);
    fn rla_apple_session_destroy(session: *mut c_void);
    fn rla_apple_count_tokens(
        text: *const c_char,
        output: *mut u64,
        error: *mut *mut c_char,
    ) -> i32;
    fn rla_string_free(value: *mut c_char);
}

pub(crate) fn platform_backend() -> Arc<dyn Backend> {
    Arc::new(AppleBackend)
}

struct AppleBackend;

#[async_trait]
impl Backend for AppleBackend {
    fn info(&self) -> BackendInfo {
        BackendInfo {
            kind: BackendKind::AppleFoundationModels,
            name: "Apple Foundation Models".into(),
            system_managed_model: true,
        }
    }

    async fn availability(&self) -> Availability {
        let mut detail = ptr::null_mut();
        // SAFETY: Swift writes either null or one owned C string to `detail`.
        let code = unsafe { rla_apple_availability(&mut detail) };
        let detail = take_string(detail);
        if code == 0 {
            Availability::available().with_detail(detail.unwrap_or_else(|| "ready".into()))
        } else {
            Availability::unavailable(match code {
                1 => AvailabilityReason::UnsupportedOsVersion,
                2 => AvailabilityReason::UnsupportedHardware,
                3 => AvailabilityReason::SystemFeatureDisabled,
                4 => AvailabilityReason::ModelNotReady,
                _ => AvailabilityReason::BackendFailure,
            })
            .with_detail(detail.unwrap_or_else(|| "Apple Foundation Models is unavailable".into()))
        }
    }

    async fn capabilities(&self) -> Capabilities {
        // SAFETY: Pure Swift function with no arguments or borrowed memory.
        let bits = unsafe { rla_apple_capabilities() };
        Capabilities {
            text_generation: bits & (1 << 0) != 0,
            streaming: bits & (1 << 1) != 0,
            structured_output: bits & (1 << 2) != 0,
            tool_calling: bits & (1 << 3) != 0,
            token_counting: bits & (1 << 4) != 0,
            cancellation: bits & (1 << 5) != 0,
            concurrent_sessions: bits & (1 << 6) != 0,
            system_managed_model: bits & (1 << 7) != 0,
            vision: false,
        }
    }

    async fn open_session(&self, instructions: Option<&str>) -> Result<Arc<dyn BackendSession>> {
        let instructions = optional_c_string(instructions)?;
        let mut error = ptr::null_mut();
        // SAFETY: pointers remain valid for the call; Swift retains the handle.
        let handle = unsafe {
            rla_apple_session_create(
                instructions.as_ref().map_or(ptr::null(), |v| v.as_ptr()),
                &mut error,
            )
        };
        if handle.is_null() {
            return Err(backend_error(take_string(error)));
        }
        Ok(Arc::new(AppleSession { handle }))
    }
}

struct AppleSession {
    handle: *mut c_void,
}

// SAFETY: the Swift object serializes mutable prompt/task state with NSLock;
// Foundation Models sessions are Sendable, and the Rust session lease prevents
// concurrent generation calls on one handle.
unsafe impl Send for AppleSession {}
// SAFETY: same synchronization and generation-lease invariant as above.
unsafe impl Sync for AppleSession {}

impl Drop for AppleSession {
    fn drop(&mut self) {
        // SAFETY: this is the sole destroy call for the retained handle.
        unsafe { rla_apple_session_destroy(self.handle) };
    }
}

#[async_trait]
impl BackendSession for AppleSession {
    async fn add_query_chunk(&self, chunk: &str) -> Result<()> {
        let chunk = CString::new(chunk)
            .map_err(|_| LocalAiError::InvalidConfig("query contains a NUL byte".into()))?;
        // SAFETY: the session is live and the string is valid for this call.
        if unsafe { rla_apple_session_add(self.handle, chunk.as_ptr()) } == 0 {
            Ok(())
        } else {
            Err(backend_error(None))
        }
    }

    async fn generate(&self, config: GenerationConfig) -> Result<AiResponse> {
        let handle = self.handle as usize;
        let temperature = config.temperature.map_or(-1.0, f64::from);
        let top_p = config.top_p.map_or(-1.0, f64::from);
        let maximum_tokens = config
            .max_output_tokens
            .map_or(-1, |value| i32::try_from(value).unwrap_or(i32::MAX));
        tokio::task::spawn_blocking(move || {
            let mut output = ptr::null_mut();
            let mut error = ptr::null_mut();
            // SAFETY: the Arc-backed session outlives this awaited blocking task.
            let code = unsafe {
                rla_apple_session_generate(
                    handle as *mut c_void,
                    temperature,
                    top_p,
                    maximum_tokens,
                    &mut output,
                    &mut error,
                )
            };
            if code == 0 {
                Ok(AiResponse::text(take_string(output).unwrap_or_default()))
            } else {
                let message = take_string(error);
                if message.as_deref() == Some("CANCELLED") {
                    Err(LocalAiError::Cancelled)
                } else {
                    Err(backend_error(message))
                }
            }
        })
        .await
        .map_err(|error| backend_error(Some(error.to_string())))?
    }

    async fn generate_stream(&self, _: GenerationConfig) -> Result<ResponseStream> {
        Err(LocalAiError::UnsupportedCapability {
            capability: "streaming",
        })
    }

    async fn cancel(&self) -> Result<()> {
        // SAFETY: session handle remains live through `&self`.
        unsafe { rla_apple_session_cancel(self.handle) };
        Ok(())
    }

    async fn count_tokens(&self, text: &str) -> Result<u64> {
        let text = CString::new(text)
            .map_err(|_| LocalAiError::InvalidConfig("text contains a NUL byte".into()))?;
        tokio::task::spawn_blocking(move || {
            let mut output = 0;
            let mut error = ptr::null_mut();
            // SAFETY: input/output pointers are valid for the duration of the call.
            let code = unsafe { rla_apple_count_tokens(text.as_ptr(), &mut output, &mut error) };
            if code == 0 {
                Ok(output)
            } else {
                Err(backend_error(take_string(error)))
            }
        })
        .await
        .map_err(|error| backend_error(Some(error.to_string())))?
    }

    async fn close(&self) -> Result<()> {
        // Native ownership ends in Drop after the safe session becomes terminal.
        Ok(())
    }
}

fn optional_c_string(value: Option<&str>) -> Result<Option<CString>> {
    value
        .map(|value| {
            CString::new(value)
                .map_err(|_| LocalAiError::InvalidConfig("instructions contain a NUL byte".into()))
        })
        .transpose()
}

fn take_string(value: *mut c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    // SAFETY: bridge strings are NUL-terminated allocations from `strdup`.
    let result = unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned();
    // SAFETY: each bridge allocation is released exactly once here.
    unsafe { rla_string_free(value) };
    Some(result)
}

fn backend_error(message: Option<String>) -> LocalAiError {
    LocalAiError::Backend {
        backend: BackendKind::AppleFoundationModels,
        message: message.unwrap_or_else(|| "Apple Foundation Models call failed".into()),
    }
}
