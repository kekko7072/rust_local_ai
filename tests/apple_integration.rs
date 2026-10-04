#![cfg(target_os = "macos")]

use rust_local_ai::{detect, GenerationConfig};

/// Requires a compatible Apple-silicon Mac with Apple Intelligence enabled
/// and its system model fully prepared. Kept out of ordinary hardware-neutral
/// CI deliberately.
#[tokio::test]
#[ignore = "requires a ready Apple Foundation Models runtime"]
async fn apple_foundation_models_generates_text() -> rust_local_ai::Result<()> {
    let model = detect().await?;
    let availability = model.availability().await;
    assert!(availability.available, "{availability:?}");
    assert!(model.capabilities().await.text_generation);

    let session = model
        .open_session(Some("Reply with exactly: ready"))
        .await?;
    session.add_query_chunk("Are you operational?").await?;
    let response = session.generate(GenerationConfig::default()).await?;
    assert!(!response.text.trim().is_empty());
    session.close().await
}
