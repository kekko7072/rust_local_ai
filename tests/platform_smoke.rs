//! Runs the real platform backend on whatever machine executes the tests.
//!
//! CI machines have no local model, so this mostly checks that detection and
//! availability run the native paths (Windows AI activation, inference-snap
//! discovery, Foundation Models) without panicking or hanging, and that an
//! unavailable backend always says why.

use std::time::Duration;

use rust_local_ai::{detect, LocalAiError};

#[tokio::test]
async fn detection_reports_availability_with_a_reason() {
    let model = detect().await.unwrap();
    let info = model.backend();
    let availability = tokio::time::timeout(Duration::from_secs(60), model.availability())
        .await
        .expect("availability must not hang");
    println!("{info:?}: {availability:?}");
    if availability.available {
        return;
    }
    assert!(availability.reason.is_some(), "{availability:?}");
    assert!(availability.detail.is_some(), "{availability:?}");
    assert!(matches!(
        model.open_session(None).await,
        Err(LocalAiError::Unavailable { .. })
    ));
}
