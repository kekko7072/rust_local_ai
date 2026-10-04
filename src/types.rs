use serde::{Deserialize, Serialize};

/// Stable identity for the selected implementation family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BackendKind {
    AppleFoundationModels,
    WindowsAi,
    UbuntuInferenceSnap,
    LinuxProvider,
    /// An explicitly configured local service speaking the OpenAI API.
    OpenAiCompatible,
    Fake,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendInfo {
    pub kind: BackendKind,
    pub name: String,
    pub system_managed_model: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AvailabilityReason {
    UnsupportedOperatingSystem,
    UnsupportedOsVersion,
    UnsupportedHardware,
    SystemFeatureDisabled,
    ModelNotReady,
    ProviderNotInstalled,
    BackendFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Availability {
    pub available: bool,
    pub reason: Option<AvailabilityReason>,
    pub detail: Option<String>,
}

impl Availability {
    pub fn available() -> Self {
        Self {
            available: true,
            reason: None,
            detail: None,
        }
    }

    pub fn unavailable(reason: AvailabilityReason) -> Self {
        Self {
            available: false,
            reason: Some(reason),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    pub text_generation: bool,
    pub streaming: bool,
    pub structured_output: bool,
    pub tool_calling: bool,
    pub vision: bool,
    pub token_counting: bool,
    pub cancellation: bool,
    pub concurrent_sessions: bool,
    pub system_managed_model: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ModelStatus {
    Ready,
    Unavailable,
    Preparing,
    InUse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ResponseFormat {
    Text,
    Json,
    JsonSchema(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub max_output_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub seed: Option<u64>,
    pub response_format: ResponseFormat,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_output_tokens: None,
            temperature: None,
            top_p: None,
            seed: None,
            response_format: ResponseFormat::Text,
        }
    }
}

impl GenerationConfig {
    pub(crate) fn validate(&self) -> crate::Result<()> {
        if self.max_output_tokens == Some(0) {
            return Err(crate::LocalAiError::InvalidConfig(
                "max_output_tokens must be greater than zero".into(),
            ));
        }
        if self.temperature.is_some_and(|v| !v.is_finite() || v < 0.0) {
            return Err(crate::LocalAiError::InvalidConfig(
                "temperature must be finite and non-negative".into(),
            ));
        }
        if self
            .top_p
            .is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
        {
            return Err(crate::LocalAiError::InvalidConfig(
                "top_p must be between 0 and 1".into(),
            ));
        }
        if let ResponseFormat::JsonSchema(schema) = &self.response_format {
            if !schema.is_object() {
                return Err(crate::LocalAiError::InvalidConfig(
                    "JSON schema must be an object".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiResponse {
    pub text: String,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub finish_reason: Option<String>,
}

impl AiResponse {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            input_tokens: None,
            output_tokens: None,
            finish_reason: None,
        }
    }
}
