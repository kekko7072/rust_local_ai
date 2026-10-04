#![cfg(all(feature = "testing", feature = "genui"))]

use std::sync::{Arc, Mutex};

use rust_local_ai::{
    genui::{parse_model_output, GenUiOptions, LocalAiUiGenerator, GENUI_INSTRUCTIONS},
    testing::FakeBackend,
    AiResponse, Availability, AvailabilityReason, Capabilities, LocalAiError,
};

const MODULE: &str = r#"Here you go:
```json
{"title":"Weekend trip fund","icon":"piggy-bank","tone":"apricot","blurb":"Save a little each week.",
 "blocks":[{"type":"amount","label":"Saved","value":120,"prefix":"$"},
           {"type":"progress","label":"Goal","value":120,"target":500,"prefix":"$","quickAdd":[10,25]}]}
```"#;

#[tokio::test]
async fn generates_a_module_from_model_output() {
    let backend = FakeBackend::default();
    backend.push_response(Ok(AiResponse::text(MODULE)));
    let module = LocalAiUiGenerator::new(backend.model())
        .generate_module("Save $500 for a weekend trip", &GenUiOptions::default())
        .await
        .unwrap();
    assert_eq!(module.title, "Weekend trip fund");
    assert_eq!(module.blocks.len(), 2);
    assert_eq!(module.blocks[1].describe(), "Goal: $120 of $500");
}

#[tokio::test]
async fn streams_cumulative_text_to_progress_callback() {
    let backend = FakeBackend::default();
    let (head, tail) = MODULE.split_at(MODULE.len() / 2);
    backend.set_stream_chunks(vec![Ok(head.into()), Ok(tail.into())]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let module = LocalAiUiGenerator::new(backend.model())
        .generate_module_with_progress("Save $500", &GenUiOptions::default(), move |text| {
            sink.lock().unwrap().push(text.len())
        })
        .await
        .unwrap();
    assert_eq!(module.title, "Weekend trip fund");
    assert_eq!(*seen.lock().unwrap(), vec![head.len(), MODULE.len()]);
}

#[tokio::test]
async fn falls_back_to_blocking_generation_without_streaming() {
    let backend = FakeBackend::new(Capabilities {
        text_generation: true,
        ..Capabilities::default()
    });
    backend.push_response(Ok(AiResponse::text(MODULE)));
    let mut calls = 0;
    let module = LocalAiUiGenerator::new(backend.model())
        .generate_module_with_progress("Save $500", &GenUiOptions::default(), |_| calls += 1)
        .await
        .unwrap();
    assert_eq!(module.tone.as_str(), "apricot");
    assert_eq!(calls, 0);
}

#[tokio::test]
async fn reports_invalid_output_empty_goal_and_unavailability() {
    let backend = FakeBackend::default();
    let generator = LocalAiUiGenerator::new(backend.model());

    backend.push_response(Ok(AiResponse::text("I cannot help with that.")));
    assert!(matches!(
        generator
            .generate_module("x", &GenUiOptions::default())
            .await,
        Err(LocalAiError::InvalidModelOutput(_))
    ));

    assert!(matches!(
        generator
            .generate_module("   ", &GenUiOptions::default())
            .await,
        Err(LocalAiError::InvalidConfig(_))
    ));

    backend.set_availability(Availability::unavailable(
        AvailabilityReason::SystemFeatureDisabled,
    ));
    assert!(matches!(
        generator
            .generate_module("x", &GenUiOptions::default())
            .await,
        Err(LocalAiError::Unavailable { .. })
    ));
}

#[test]
fn instructions_and_parser_are_reusable_by_other_models() {
    assert!(GENUI_INSTRUCTIONS.contains("\"type\":\"progress\""));
    assert!(parse_model_output(MODULE).is_some());
}
