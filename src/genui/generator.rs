use std::{future::poll_fn, pin::Pin};

use futures_core::Stream;

use serde_json::{json, Value};

use super::{parse_model_output, GenUiModuleSpec, GENUI_BLOCK_TYPES, GENUI_INSTRUCTIONS};
use crate::{GenerationConfig, LocalAiError, LocalAiModel, LocalAiSession, ResponseFormat, Result};

/// Output budget for one module: enough for 2-4 blocks on small on-device
/// models.
pub const GENUI_MAX_OUTPUT_TOKENS: u32 = 900;

/// Steering for one [`LocalAiUiGenerator::generate_module`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenUiOptions {
    /// Design principles the module should respect, e.g.
    /// `"Keep it simple and low-pressure"`.
    pub principles: Option<String>,
    /// Language name (e.g. `"Italian"`) for all user-facing copy. `None` leaves
    /// the model to answer in the goal's language.
    pub language: Option<String>,
    /// Ask for at most 3 short blocks as minified JSON at a lower temperature.
    /// Improves reliability on the smallest on-device models.
    pub compact: bool,
    /// Total generations to try when the model's output is not a usable
    /// module (default 2; 0 is treated as 1). Retries use the compact prompt.
    /// Errors other than unusable output are returned immediately.
    pub max_attempts: u8,
}

impl Default for GenUiOptions {
    fn default() -> Self {
        Self {
            principles: None,
            language: None,
            compact: false,
            max_attempts: 2,
        }
    }
}

/// Turns a natural-language goal into a [`GenUiModuleSpec`] using a local
/// model.
///
/// Every module is generated in its own short-lived session carrying the genUI
/// instructions, so ongoing chats on the same model are not disturbed and the
/// model's context window is not consumed across modules.
#[derive(Debug, Clone)]
pub struct LocalAiUiGenerator {
    model: LocalAiModel,
}

impl LocalAiUiGenerator {
    pub fn new(model: LocalAiModel) -> Self {
        Self { model }
    }

    pub fn model(&self) -> &LocalAiModel {
        &self.model
    }

    /// Generates a module for `goal`.
    ///
    /// Fails with [`LocalAiError::InvalidConfig`] for an empty goal, with the
    /// usual availability and capability errors from the model, and with
    /// [`LocalAiError::InvalidModelOutput`] when no valid module can be
    /// recovered from the model's text, so callers can fall back to a
    /// deterministic UI.
    pub async fn generate_module(
        &self,
        goal: &str,
        options: &GenUiOptions,
    ) -> Result<GenUiModuleSpec> {
        self.generate(goal, options, None).await
    }

    /// Like [`generate_module`](Self::generate_module), streaming the
    /// cumulative raw model output to `on_text` as it decodes, for live
    /// progress or preview UI.
    ///
    /// Falls back to a single blocking generation when the backend cannot
    /// stream or the stream fails before producing any text. When an attempt
    /// is retried, the text starts again from empty.
    pub async fn generate_module_with_progress(
        &self,
        goal: &str,
        options: &GenUiOptions,
        mut on_text: impl FnMut(&str) + Send,
    ) -> Result<GenUiModuleSpec> {
        self.generate(goal, options, Some(&mut on_text)).await
    }

    async fn generate(
        &self,
        goal: &str,
        options: &GenUiOptions,
        on_text: Option<&mut (dyn FnMut(&str) + Send + '_)>,
    ) -> Result<GenUiModuleSpec> {
        let goal = goal.trim();
        if goal.is_empty() {
            return Err(LocalAiError::InvalidConfig(
                "genUI goal must not be empty".into(),
            ));
        }
        let mut on_text = on_text;
        let mut last_error = None;
        for attempt in 0..options.max_attempts.max(1) {
            // Retries switch to the stricter compact prompt, which small models
            // follow more reliably.
            let compact = options.compact || attempt > 0;
            match self
                .attempt(goal, options, compact, on_text.as_deref_mut())
                .await
            {
                Err(LocalAiError::InvalidModelOutput(message)) => {
                    last_error = Some(LocalAiError::InvalidModelOutput(message));
                }
                result => return result,
            }
        }
        Err(last_error.expect("at least one attempt ran"))
    }

    async fn attempt(
        &self,
        goal: &str,
        options: &GenUiOptions,
        compact: bool,
        on_text: Option<&mut (dyn FnMut(&str) + Send + '_)>,
    ) -> Result<GenUiModuleSpec> {
        let session = self.model.open_session(Some(GENUI_INSTRUCTIONS)).await?;
        let config = GenerationConfig {
            max_output_tokens: Some(GENUI_MAX_OUTPUT_TOKENS),
            // Lower temperature on small models gives more reliable JSON.
            temperature: Some(if compact { 0.2 } else { 0.5 }),
            // Constrained decoding, where the backend offers it, guarantees
            // well-formed JSON; the parser below still validates it.
            response_format: if session.capabilities().structured_output {
                ResponseFormat::JsonSchema(module_json_schema())
            } else {
                ResponseFormat::Text
            },
            ..GenerationConfig::default()
        };
        let prompt = build_prompt(goal, options, compact);
        let raw = read_text(&session, &prompt, config, on_text).await;
        // The session is throwaway; a failed teardown must not discard a
        // module that was generated successfully.
        let _ = session.close().await;

        parse_model_output(&raw?).ok_or_else(|| {
            LocalAiError::InvalidModelOutput("no valid genUI module in the model output".into())
        })
    }
}

/// JSON Schema for a module, used when the backend supports structured
/// output. Block fields beyond `type` are left open so every block type fits.
pub(crate) fn module_json_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "title": {"type": "string"},
            "icon": {"type": "string"},
            "tone": {"type": "string", "enum": ["fern", "apricot", "sky", "lilac"]},
            "blurb": {"type": "string"},
            "blocks": {
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "properties": {"type": {"type": "string", "enum": GENUI_BLOCK_TYPES}},
                    "required": ["type"],
                },
            },
        },
        "required": ["title", "icon", "tone", "blurb", "blocks"],
    })
}

fn build_prompt(goal: &str, options: &GenUiOptions, compact: bool) -> String {
    let mut prompt = format!("Design the Fledge module for this goal: \"{goal}\".");
    if let Some(principles) = options.principles.as_deref().filter(|p| !p.is_empty()) {
        prompt.push_str(&format!(
            "\nThe user's guiding principles: {principles}. Respect them."
        ));
    }
    if let Some(language) = options.language.as_deref().filter(|l| !l.is_empty()) {
        prompt.push_str(&format!(
            "\nWrite ALL user-facing text (title, blurb, and every label, item and note) in {language}."
        ));
    }
    prompt.push_str("\nReturn ONLY the JSON object.");
    if compact {
        prompt.push_str(
            " Use at most 3 blocks. Keep every string under 8 words. Output minified JSON \
             on a single line with no spaces after colons or commas, and no trailing commas.",
        );
    }
    prompt
}

async fn read_text(
    session: &LocalAiSession,
    prompt: &str,
    config: GenerationConfig,
    on_text: Option<&mut (dyn FnMut(&str) + Send + '_)>,
) -> Result<String> {
    session.add_query_chunk(prompt).await?;
    if let Some(on_text) = on_text {
        if session.capabilities().streaming {
            let mut text = String::new();
            match session.generate_stream(config.clone()).await {
                Ok(mut stream) => {
                    while let Some(chunk) = poll_fn(|cx| Pin::new(&mut stream).poll_next(cx)).await
                    {
                        match chunk {
                            Ok(chunk) => {
                                text.push_str(&chunk);
                                on_text(&text);
                            }
                            // Text already arrived (or the caller cancelled):
                            // a real failure, not a missing streaming
                            // implementation.
                            Err(err)
                                if !text.is_empty() || matches!(err, LocalAiError::Cancelled) =>
                            {
                                return Err(err)
                            }
                            Err(_) => break,
                        }
                    }
                    if !text.is_empty() {
                        return Ok(text);
                    }
                }
                Err(LocalAiError::Cancelled) => return Err(LocalAiError::Cancelled),
                Err(_) => {}
            }
        }
    }
    Ok(session.generate(config).await?.text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_includes_optional_steering() {
        let plain = build_prompt("Save $500", &GenUiOptions::default(), false);
        assert_eq!(
            plain,
            "Design the Fledge module for this goal: \"Save $500\".\nReturn ONLY the JSON object."
        );
        let steered = build_prompt(
            "Save $500",
            &GenUiOptions {
                principles: Some("Keep it simple".into()),
                language: Some("Italian".into()),
                ..GenUiOptions::default()
            },
            true,
        );
        assert!(steered.contains("guiding principles: Keep it simple."));
        assert!(steered.contains("in Italian."));
        assert!(steered.ends_with("and no trailing commas."));
    }

    #[test]
    fn json_schema_lists_every_block_type() {
        let schema = module_json_schema();
        assert_eq!(schema["type"], "object");
        let types = schema["properties"]["blocks"]["items"]["properties"]["type"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(types.len(), GENUI_BLOCK_TYPES.len());
    }
}
