# rust_local_ai

`rust_local_ai` is a unified Rust interface to AI capabilities provided or
managed by the operating system.

It is **not primarily another model runner or model-download framework**. The
library prefers OS-native AI, then distribution-managed AI, then an explicitly
configured installed provider. It never silently downloads a model, installs a
provider, or starts a heavyweight inference service.

> This crate is early `0.x` software. The portable API and fake backend are
> implemented and tested. The macOS adapter currently implements availability,
> sessions, text generation, cancellation, and token counting (macOS 26.4+).
> Remaining native adapters are under active development.

## Installation

```toml
[dependencies]
rust_local_ai = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

## Basic usage

```no_run
use rust_local_ai::{detect, GenerationConfig};

#[tokio::main]
async fn main() -> rust_local_ai::Result<()> {
    let model = detect().await?;
    let availability = model.availability().await;
    if !availability.available {
        eprintln!("Local AI unavailable: {:?}", availability.reason);
        return Ok(());
    }

    let session = model.open_session(Some("You are a helpful assistant.")).await?;
    session.add_query_chunk("Explain reinforcement learning simply.").await?;
    let response = session.generate(GenerationConfig::default()).await?;
    println!("{}", response.text);
    session.close().await?;
    Ok(())
}
```

See [`examples/basic.rs`](examples/basic.rs),
[`examples/streaming.rs`](examples/streaming.rs), and
[`examples/capabilities.rs`](examples/capabilities.rs).

## Capabilities and errors

Call `model.capabilities().await` before depending on optional functionality.
Calls are checked again and return `LocalAiError::UnsupportedCapability` when a
capability is absent. Other stable variants include `Unavailable`,
`SessionBusy`, `SessionClosed`, `ModelClosed`, `InvalidConfig`, `Cancelled`, and
`Backend`.

Only one generation may run on a session. Independent sessions can run in
parallel when `concurrent_sessions` is reported. A successful `close()` is
terminal and idempotent. If backend teardown fails, close returns that error and
the session remains open so teardown can be retried.

Structured JSON and JSON-schema requests use `ResponseFormat`. They are only
accepted when `structured_output` is advertised. Tool calling types are not yet
public because no shipping adapter implements them; the crate will not claim a
capability merely because an API shape exists.

## Support matrix

This table describes functionality in this repository today, not upstream OS
potential.

| Capability | Apple | Windows | Ubuntu | Other Linux providers | Fake/test backend |
|---|---:|---:|---:|---:|---:|
| Availability detection | ✓ | planned | planned | planned | ✓ |
| Text generation | beta | — | — | — | ✓ |
| Streaming | — | — | — | — | ✓ |
| Structured output | — | — | — | — | ✓ |
| Tool calling | — | — | — | — | — |
| Generative UI (`genui`) | beta | — | — | — | ✓ |
| Token counting | beta (26.4+) | — | — | — | ✓ |
| Cancellation | beta | — | — | — | ✓ |
| Concurrent sessions | beta | — | — | — | ✓ |
| System-managed model | ✓ | intended | intended | varies | no |

The Apple adapter is a thin Swift C-ABI bridge to `FoundationModels`; it does
not bundle a model. It compiles to an unavailable fallback with older Apple
SDKs, and runtime availability distinguishes old OS versions, ineligible
hardware, disabled Apple Intelligence, and a model that is still preparing.
The hardware integration test is ignored in ordinary CI and can be run with
`cargo test --test apple_integration -- --ignored` on a configured Mac.

On Linux there is deliberately no fictional universal backend. Provider
selection will prefer distribution-managed infrastructure such as Canonical
Inference Snaps, then supported providers already installed by the user.

## Testing adapters

Enable the `testing` feature to use `rust_local_ai::testing::FakeBackend` in
downstream contract tests. It supports deterministic responses, streams,
failures, cancellation, and close-retry testing without OS AI hardware.

## Generative UI

Enable the `genui` feature to turn a natural-language goal into a small,
renderable UI module on-device. It is a port of the genUI engine in
[`flutter_local_ai`](https://pub.dev/packages/flutter_local_ai), with the same
typed-block schema, system instructions and tolerant parser. A module made by
either package renders the same way in both.

```toml
[dependencies]
rust_local_ai = { version = "0.1", features = ["genui"] }  # or "a2ui"
```

```rust,ignore
use rust_local_ai::{detect, genui::{GenUiOptions, LocalAiUiGenerator}};

let generator = LocalAiUiGenerator::new(detect().await?);
let module = generator
    .generate_module(
        "Save $500 for a weekend trip",
        &GenUiOptions {
            principles: Some("Keep it simple and low-pressure".into()),
            language: Some("Italian".into()),
            ..GenUiOptions::default()
        },
    )
    .await?; // LocalAiError::InvalidModelOutput lets you fall back to a fixed UI

println!("{}", module.title);            // e.g. "Weekend trip fund"
let json = module.to_module_json();      // flutter_local_ai module shape
let tree = module.to_component_maps();   // renderer-neutral component tree
```

A `GenUiModuleSpec` is a stack of typed blocks (`amount`, `progress`,
`checklist`, `week`, `stat`, `list`, `lessons`, `reminder`, `calc`, `docs`,
`note`). Each module runs in its own short-lived session, so it does not touch
ongoing chats. Output goes through a 900-token budget and is validated before
it is returned. When the model's output is cut off, the parser repairs it where
it can so a partial module still renders.
`generate_module_with_progress` streams the raw text as it decodes, for live
previews.

To drive the same generation with any other model, send
`genui::GENUI_INSTRUCTIONS` as its system prompt and pass its raw text to
`genui::parse_model_output`.

### A2UI interoperability

The `a2ui` feature (it also enables `genui`) turns a module into
[A2UI](https://a2ui.org) v0.9 protocol messages, typed with the
[`a2ui-types`](https://crates.io/crates/a2ui-types) crate. Any A2UI renderer
can display them, including Flutter's `genui`, the web renderers, or Rust
renderers such as the [`a2ui`](https://crates.io/crates/a2ui) crate
(ratatui, egui, Iced, Bevy, and more):

```rust,ignore
use rust_local_ai::genui::A2UI_V09_BASIC_CATALOG_ID;

// createSurface, updateComponents, updateDataModel (the full module)
for message in module.to_a2ui_v09_messages("trip", A2UI_V09_BASIC_CATALOG_ID) {
    println!("{}", serde_json::to_string(&message)?); // JSONL stream
}
```

See [`examples/genui.rs`](examples/genui.rs)
(`cargo run --example genui --features a2ui`).

The inference core has no UI-framework dependency. A future N-API crate can
wrap this same core for Electron without reimplementing native adapters in
TypeScript.

## License

Licensed under either Apache-2.0 or MIT, at your option.

Created by [Francesco Vezzani](https://vezz.io).
