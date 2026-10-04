//! Runs against a real local OpenAI-compatible server when
//! `RUST_LOCAL_AI_TEST_OPENAI_URL` is set (for example an inference snap's
//! `http://localhost:8338/v1`, or `llama-server`'s `http://127.0.0.1:8080/v1`).
//! Skipped otherwise.

use std::{env, time::Duration};

use futures_util::StreamExt;
use rust_local_ai::{openai_compatible, GenerationConfig, LocalAiError, LocalAiModel};

fn live_model() -> Option<LocalAiModel> {
    let url = env::var("RUST_LOCAL_AI_TEST_OPENAI_URL").ok()?;
    Some(openai_compatible(&url, None).expect("valid local URL"))
}

fn short() -> GenerationConfig {
    GenerationConfig {
        max_output_tokens: Some(24),
        temperature: Some(0.0),
        ..GenerationConfig::default()
    }
}

#[tokio::test]
async fn live_generate_stream_and_cancel() {
    let Some(model) = live_model() else {
        eprintln!("RUST_LOCAL_AI_TEST_OPENAI_URL not set; skipping");
        return;
    };
    let availability = model.availability().await;
    assert!(availability.available, "{availability:?}");

    let session = model.open_session(Some("You are terse.")).await.unwrap();
    session.add_query_chunk("Name a colour.").await.unwrap();
    let response = session.generate(short()).await.unwrap();
    println!("generate: {response:?}");
    assert!(response.output_tokens.unwrap_or(1) > 0);

    session.add_query_chunk("Name another.").await.unwrap();
    let mut stream = session.generate_stream(short()).await.unwrap();
    let mut streamed = String::new();
    while let Some(chunk) = stream.next().await {
        streamed.push_str(&chunk.unwrap());
    }
    drop(stream);
    println!("stream: {streamed:?}");
    assert!(!streamed.is_empty());

    session
        .add_query_chunk("Write a very long story.")
        .await
        .unwrap();
    let long = GenerationConfig {
        max_output_tokens: Some(4096),
        ..GenerationConfig::default()
    };
    let generating = {
        let session = session.clone();
        tokio::spawn(async move { session.generate(long).await })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    session.cancel().await.unwrap();
    let result = tokio::time::timeout(Duration::from_secs(30), generating)
        .await
        .expect("cancel must unblock generation")
        .unwrap();
    println!("cancel: {:?}", result.as_ref().map(|r| r.text.len()));
    // A fast model may already have finished; otherwise it must be cancelled.
    assert!(
        matches!(result, Ok(_) | Err(LocalAiError::Cancelled)),
        "{result:?}"
    );

    // The session is still usable after a cancellation.
    session.add_query_chunk("Say OK.").await.unwrap();
    session.generate(short()).await.unwrap();
}

#[cfg(feature = "genui")]
#[tokio::test]
async fn live_genui() {
    use rust_local_ai::genui::{GenUiOptions, LocalAiUiGenerator};

    let Some(model) = live_model() else {
        return;
    };
    let result = LocalAiUiGenerator::new(model)
        .generate_module("Save $500 for a weekend trip", &GenUiOptions::default())
        .await;
    println!("genui: {result:?}");
    // Tiny test models may not manage the schema; that must surface as an
    // ordinary error, never a panic or a hang.
    assert!(matches!(
        result,
        Ok(_) | Err(LocalAiError::InvalidModelOutput(_))
    ));
}
