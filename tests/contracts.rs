#![cfg(feature = "testing")]

use std::sync::Arc;

use futures_util::StreamExt;
use rust_local_ai::{
    testing::FakeBackend, AiResponse, Availability, AvailabilityReason, Capabilities,
    GenerationConfig, LocalAiError, ResponseFormat,
};
use tokio::sync::Barrier;

#[tokio::test]
async fn chunks_form_a_single_turn() {
    let session = FakeBackend::default()
        .model()
        .open_session(Some("help"))
        .await
        .unwrap();
    session.add_query_chunk("hello ").await.unwrap();
    session.add_query_chunk("world").await.unwrap();
    assert_eq!(
        session
            .generate(GenerationConfig::default())
            .await
            .unwrap()
            .text,
        "hello world"
    );
}

#[tokio::test]
async fn successful_close_is_terminal_and_idempotent() {
    let session = FakeBackend::default()
        .model()
        .open_session(None)
        .await
        .unwrap();
    session.close().await.unwrap();
    session.close().await.unwrap();
    assert!(matches!(
        session.add_query_chunk("no").await,
        Err(LocalAiError::SessionClosed)
    ));
}

#[tokio::test]
async fn failed_close_is_retryable() {
    let fake = FakeBackend::default();
    fake.fail_next_closes(1);
    let session = fake.model().open_session(None).await.unwrap();
    assert!(matches!(
        session.close().await,
        Err(LocalAiError::Backend { .. })
    ));
    assert!(!session.is_closed());
    session.add_query_chunk("still alive").await.unwrap();
    session.close().await.unwrap();
    assert!(session.is_closed());
}

#[tokio::test]
async fn concurrent_generation_returns_session_busy() {
    let fake = FakeBackend::default();
    fake.set_generation_blocked(true);
    let session = fake.model().open_session(None).await.unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let worker_session = session.clone();
    let worker_barrier = barrier.clone();
    let worker = tokio::spawn(async move {
        worker_barrier.wait().await;
        worker_session.generate(GenerationConfig::default()).await
    });
    barrier.wait().await;
    tokio::task::yield_now().await;
    assert!(matches!(
        session.generate(GenerationConfig::default()).await,
        Err(LocalAiError::SessionBusy)
    ));
    fake.set_generation_blocked(false);
    worker.await.unwrap().unwrap();
}

#[tokio::test]
async fn stream_holds_busy_lease_until_dropped() {
    let fake = FakeBackend::default();
    fake.set_stream_chunks(vec![Ok("a".into()), Ok("b".into())]);
    let session = fake.model().open_session(None).await.unwrap();
    let mut stream = session
        .generate_stream(GenerationConfig::default())
        .await
        .unwrap();
    assert!(matches!(
        session.generate(GenerationConfig::default()).await,
        Err(LocalAiError::SessionBusy)
    ));
    assert_eq!(stream.next().await.unwrap().unwrap(), "a");
    drop(stream);
    session.generate(GenerationConfig::default()).await.unwrap();
}

#[tokio::test]
async fn unavailable_model_refuses_sessions_with_typed_reason() {
    let fake = FakeBackend::default();
    fake.set_availability(Availability::unavailable(AvailabilityReason::ModelNotReady));
    assert!(matches!(
        fake.model().open_session(None).await,
        Err(LocalAiError::Unavailable {
            reason: AvailabilityReason::ModelNotReady
        })
    ));
}

#[tokio::test]
async fn unsupported_capability_is_not_forwarded() {
    let fake = FakeBackend::new(Capabilities {
        text_generation: true,
        ..Capabilities::default()
    });
    let session = fake.model().open_session(None).await.unwrap();
    assert!(matches!(
        session.generate_stream(GenerationConfig::default()).await,
        Err(LocalAiError::UnsupportedCapability {
            capability: "streaming"
        })
    ));
}

#[tokio::test]
async fn malformed_configuration_is_rejected() {
    let session = FakeBackend::default()
        .model()
        .open_session(None)
        .await
        .unwrap();
    let config = GenerationConfig {
        top_p: Some(1.5),
        ..GenerationConfig::default()
    };
    assert!(matches!(
        session.generate(config).await,
        Err(LocalAiError::InvalidConfig(_))
    ));
    let schema = GenerationConfig {
        response_format: ResponseFormat::JsonSchema(serde_json::json!([])),
        ..GenerationConfig::default()
    };
    assert!(matches!(
        session.generate(schema).await,
        Err(LocalAiError::InvalidConfig(_))
    ));
}

#[tokio::test]
async fn backend_errors_are_preserved() {
    let fake = FakeBackend::default();
    fake.push_response(Err(LocalAiError::Backend {
        backend: rust_local_ai::BackendKind::Fake,
        message: "boom".into(),
    }));
    let session = fake.model().open_session(None).await.unwrap();
    assert!(
        matches!(session.generate(GenerationConfig::default()).await, Err(LocalAiError::Backend { message, .. }) if message == "boom")
    );
}

#[tokio::test]
async fn token_count_and_cancellation_contracts() {
    let session = FakeBackend::default()
        .model()
        .open_session(None)
        .await
        .unwrap();
    assert_eq!(session.count_tokens("one two three").await.unwrap(), 3);
    session.cancel().await.unwrap();
    assert!(matches!(
        session.generate(GenerationConfig::default()).await,
        Err(LocalAiError::Cancelled)
    ));
}

#[tokio::test]
async fn independent_sessions_generate_concurrently() {
    let model = FakeBackend::default().model();
    let a = model.open_session(None).await.unwrap();
    let b = model.open_session(None).await.unwrap();
    a.add_query_chunk("a").await.unwrap();
    b.add_query_chunk("b").await.unwrap();
    let (a, b) = tokio::join!(
        a.generate(GenerationConfig::default()),
        b.generate(GenerationConfig::default())
    );
    assert_eq!(a.unwrap(), AiResponse::text("a"));
    assert_eq!(b.unwrap(), AiResponse::text("b"));
}
