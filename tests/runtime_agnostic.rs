//! The library must not require a specific async runtime: these tests drive it
//! with a minimal std-only executor and no tokio runtime.
#![cfg(feature = "testing")]

use std::{
    future::Future,
    pin::pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread::{self, Thread},
};

use rust_local_ai::{testing::FakeBackend, GenerationConfig};

struct ThreadWaker(Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return output;
        }
        thread::park();
    }
}

#[test]
fn sessions_work_without_a_tokio_runtime() {
    block_on(async {
        let session = FakeBackend::default()
            .model()
            .open_session(None)
            .await
            .unwrap();
        session.add_query_chunk("hello").await.unwrap();
        let response = session.generate(GenerationConfig::default()).await.unwrap();
        assert_eq!(response.text, "hello");
        session.close().await.unwrap();
    });
}

#[cfg(feature = "genui")]
#[test]
fn genui_works_without_a_tokio_runtime() {
    use rust_local_ai::{
        genui::{GenUiOptions, LocalAiUiGenerator},
        AiResponse,
    };

    let backend = FakeBackend::default();
    backend.push_response(Ok(AiResponse::text(
        r#"{"title":"Plan","blocks":[{"type":"note","text":"Start small"}]}"#,
    )));
    let module = block_on(
        LocalAiUiGenerator::new(backend.model()).generate_module("Plan", &GenUiOptions::default()),
    );
    assert_eq!(module.unwrap().title, "Plan");
}
