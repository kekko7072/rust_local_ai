//! End-to-end tests of the OpenAI-compatible backend (used by Ubuntu inference
//! snaps) against a scripted local server.

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use futures_util::StreamExt;
use rust_local_ai::{
    openai_compatible, AvailabilityReason, BackendKind, GenerationConfig, LocalAiError,
};
use serde_json::{json, Value};

type Handler = dyn Fn(&str, &str, Value, &mut TcpStream) + Send + Sync;

/// A tiny OpenAI-style server. Every request body is recorded.
struct Server {
    url: String,
    requests: Arc<Mutex<Vec<(String, Value)>>>,
}

impl Server {
    fn start(handler: impl Fn(&str, &str, Value, &mut TcpStream) + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v1", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = requests.clone();
        let handler: Arc<Handler> = Arc::new(handler);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let recorded = recorded.clone();
                let handler = handler.clone();
                thread::spawn(move || serve(stream, &*handler, &recorded));
            }
        });
        Self { url, requests }
    }

    fn chat_requests(&self) -> Vec<Value> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|(path, _)| path.ends_with("/chat/completions"))
            .map(|(_, body)| body.clone())
            .collect()
    }
}

fn serve(mut stream: TcpStream, handler: &Handler, recorded: &Mutex<Vec<(String, Value)>>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).unwrap_or(0) == 0 {
        return;
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_owned();
    let path = parts.next().unwrap_or_default().to_owned();
    let mut length = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse().unwrap();
        }
        if line == "\r\n" {
            break;
        }
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    recorded.lock().unwrap().push((path.clone(), body.clone()));
    if path.ends_with("/models") {
        respond_json(
            &mut stream,
            200,
            &json!({"object": "list", "data": [{"id": "qwen3-4b"}]}),
        );
    } else {
        handler(&method, &path, body, &mut stream);
    }
}

fn respond_json(stream: &mut TcpStream, status: u16, body: &Value) {
    let body = body.to_string();
    let _ = write!(
        stream,
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
}

fn completion(text: &str) -> Value {
    json!({
        "choices": [{"index": 0, "message": {"role": "assistant", "content": text}, "finish_reason": "stop"}],
        "usage": {"prompt_tokens": 7, "completion_tokens": 3}
    })
}

/// Writes `events` as a chunked server-sent-events response.
fn respond_sse(stream: &mut TcpStream, events: &[String]) {
    let _ = write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n"
    );
    for event in events {
        let frame = format!("data: {event}\n\n");
        let _ = write!(stream, "{:x}\r\n{frame}\r\n", frame.len());
        let _ = stream.flush();
    }
    let _ = write!(stream, "0\r\n\r\n");
}

fn delta(text: &str) -> String {
    json!({"choices": [{"index": 0, "delta": {"content": text}}]}).to_string()
}

#[tokio::test]
async fn generates_and_replays_the_transcript() {
    let server = Server::start(|_, _, body, stream| {
        let turns = body["messages"].as_array().unwrap().len();
        respond_json(stream, 200, &completion(&format!("answer {turns}")));
    });
    let model = openai_compatible(&server.url, None).unwrap();
    assert_eq!(model.backend().kind, BackendKind::OpenAiCompatible);
    let availability = model.availability().await;
    assert!(availability.available, "{availability:?}");
    assert_eq!(availability.detail.as_deref(), Some("model qwen3-4b"));

    let session = model.open_session(Some("Be brief.")).await.unwrap();
    session.add_query_chunk("Hello ").await.unwrap();
    session.add_query_chunk("there").await.unwrap();
    let config = GenerationConfig {
        max_output_tokens: Some(64),
        temperature: Some(0.2),
        seed: Some(7),
        ..GenerationConfig::default()
    };
    let first = session.generate(config.clone()).await.unwrap();
    assert_eq!(first.text, "answer 2");
    assert_eq!(first.input_tokens, Some(7));
    assert_eq!(first.finish_reason.as_deref(), Some("stop"));

    session.add_query_chunk("And again?").await.unwrap();
    assert_eq!(session.generate(config).await.unwrap().text, "answer 4");

    let requests = server.chat_requests();
    assert_eq!(requests[0]["model"], "qwen3-4b");
    assert_eq!(requests[0]["max_tokens"], 64);
    assert_eq!(requests[0]["seed"], 7);
    assert_eq!(requests[0]["stream"], false);
    assert_eq!(
        requests[1]["messages"],
        json!([
            {"role": "system", "content": "Be brief."},
            {"role": "user", "content": "Hello there"},
            {"role": "assistant", "content": "answer 2"},
            {"role": "user", "content": "And again?"},
        ])
    );
    session.close().await.unwrap();
}

#[tokio::test]
async fn streams_chunked_server_sent_events() {
    let server = Server::start(|_, _, body, stream| {
        assert_eq!(body["stream"], true);
        respond_sse(
            stream,
            &[delta("Hel"), delta(""), delta("lo"), "[DONE]".into()],
        );
    });
    let model = openai_compatible(&server.url, Some("custom-model")).unwrap();
    assert!(model.capabilities().await.streaming);
    let session = model.open_session(None).await.unwrap();
    session.add_query_chunk("Say hello").await.unwrap();
    let mut stream = session
        .generate_stream(GenerationConfig::default())
        .await
        .unwrap();
    let mut chunks = Vec::new();
    while let Some(chunk) = stream.next().await {
        chunks.push(chunk.unwrap());
    }
    assert_eq!(chunks, vec!["Hel", "lo"]);
    drop(stream);

    // The streamed answer joins the transcript.
    session.add_query_chunk("Again").await.unwrap();
    let mut stream = session
        .generate_stream(GenerationConfig::default())
        .await
        .unwrap();
    while stream.next().await.is_some() {}
    let requests = server.chat_requests();
    assert_eq!(requests[0]["model"], "custom-model");
    assert_eq!(
        requests[1]["messages"][1],
        json!({"role": "assistant", "content": "Hello"})
    );
}

#[tokio::test]
async fn http_errors_keep_the_turn_for_a_retry() {
    let calls = Arc::new(Mutex::new(0));
    let counter = calls.clone();
    let server = Server::start(move |_, _, _, stream| {
        let mut calls = counter.lock().unwrap();
        *calls += 1;
        if *calls == 1 {
            respond_json(
                stream,
                503,
                &json!({"error": {"message": "model is loading"}}),
            );
        } else {
            respond_json(stream, 200, &completion("ok"));
        }
    });
    let session = openai_compatible(&server.url, None)
        .unwrap()
        .open_session(None)
        .await
        .unwrap();
    session.add_query_chunk("Question").await.unwrap();
    match session.generate(GenerationConfig::default()).await {
        Err(LocalAiError::Backend { message, .. }) => {
            assert_eq!(message, "HTTP 503: model is loading")
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(
        session
            .generate(GenerationConfig::default())
            .await
            .unwrap()
            .text,
        "ok"
    );
    let requests = server.chat_requests();
    assert_eq!(
        requests[1]["messages"],
        json!([{"role": "user", "content": "Question"}])
    );
}

#[tokio::test]
async fn cancel_aborts_a_blocking_generation() {
    // The server never answers, like a slow model before its first token.
    let server = Server::start(|_, _, _, stream| {
        let mut buf = [0; 1];
        let _ = stream.read(&mut buf);
    });
    let session = openai_compatible(&server.url, None)
        .unwrap()
        .open_session(None)
        .await
        .unwrap();
    session.add_query_chunk("Think hard").await.unwrap();
    let generating = {
        let session = session.clone();
        tokio::spawn(async move { session.generate(GenerationConfig::default()).await })
    };
    // Wait until the request has reached the server.
    let started = Instant::now();
    while server.chat_requests().is_empty() {
        assert!(started.elapsed() < Duration::from_secs(10));
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    session.cancel().await.unwrap();
    let result = tokio::time::timeout(Duration::from_secs(10), generating)
        .await
        .expect("cancel must unblock the generation")
        .unwrap();
    assert!(matches!(result, Err(LocalAiError::Cancelled)), "{result:?}");
}

#[tokio::test]
async fn unreachable_service_is_reported_not_installed() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let model = openai_compatible(&format!("http://127.0.0.1:{port}/v1"), None).unwrap();
    let availability = model.availability().await;
    assert!(!availability.available);
    assert_eq!(
        availability.reason,
        Some(AvailabilityReason::ProviderNotInstalled)
    );
    assert!(matches!(
        model.open_session(None).await,
        Err(LocalAiError::Unavailable { .. })
    ));
}

#[test]
fn only_local_http_endpoints_are_accepted() {
    for url in [
        "https://localhost:8080/v1",
        "http://api.example.com/v1",
        "http://10.0.0.5:8080/v1",
    ] {
        assert!(
            matches!(
                openai_compatible(url, None),
                Err(LocalAiError::InvalidConfig(_))
            ),
            "{url}"
        );
    }
    assert!(openai_compatible("http://localhost:11434/v1", None).is_ok());
}

#[cfg(feature = "genui")]
#[tokio::test]
async fn genui_runs_over_the_openai_backend() {
    use rust_local_ai::genui::{GenUiOptions, LocalAiUiGenerator, GENUI_INSTRUCTIONS};

    let module =
        r#"{"title":"Trip fund","blocks":[{"type":"amount","label":"Saved","value":"$120"}]}"#;
    let server = Server::start(move |_, _, body, stream| {
        assert_eq!(body["messages"][0]["content"], GENUI_INSTRUCTIONS);
        let (head, tail) = module.split_at(20);
        respond_sse(stream, &[delta(head), delta(tail), "[DONE]".into()]);
    });
    let generator = LocalAiUiGenerator::new(openai_compatible(&server.url, None).unwrap());
    let mut progress = 0;
    let spec = generator
        .generate_module_with_progress("Save for a trip", &GenUiOptions::default(), |_| {
            progress += 1
        })
        .await
        .unwrap();
    assert_eq!(spec.title, "Trip fund");
    assert_eq!(spec.blocks[0].fields()["value"], 120);
    assert_eq!(progress, 2);
    assert_eq!(server.chat_requests()[0]["max_tokens"], 900);
}
