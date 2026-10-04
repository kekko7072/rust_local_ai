# rust_local_ai

`rust_local_ai` is a unified Rust interface to AI capabilities provided or
managed by the operating system.

It is **not primarily another model runner or model-download framework**. The
library prefers OS-native AI, then distribution-managed AI, then an explicitly
configured installed provider. It never silently downloads a model, installs a
provider, or starts a heavyweight inference service.

> This crate is early `0.x` software. The portable API, the fake backend and
> the OpenAI-compatible adapter are implemented and tested. Adapters exist for
> Apple Foundation Models (macOS), Windows AI / Phi Silica (Windows) and Ubuntu
> inference snaps (Linux). The Apple and Windows adapters have not yet been run
> on hardware that has a model.

## Installation

```toml
[dependencies]
rust_local_ai = "0.2"
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

| Capability | Apple | Windows | Ubuntu inference snaps | `openai_compatible` | Fake/test backend |
|---|---:|---:|---:|---:|---:|
| Availability detection | ✓ | ✓ | ✓ | ✓ | ✓ |
| Text generation | beta | beta | ✓ | ✓ | ✓ |
| Streaming | — | — | ✓ | ✓ | ✓ |
| Structured output | — | — | — | — | ✓ |
| Tool calling | — | — | — | — | — |
| Generative UI (`genui`) | beta | beta | ✓ | ✓ | ✓ |
| Token counting | beta (26.4+) | — | — | — | ✓ |
| Cancellation | beta | beta | ✓ | ✓ | ✓ |
| Concurrent sessions | beta | beta | ✓ | ✓ | ✓ |
| Explicit `prepare()` download | — (system) | ✓ | — (snap) | — | — |
| System-managed model | ✓ | ✓ | ✓ | no | no |

"beta" means the code is built and type-checked on CI but hasn't run against a
real model on that platform yet.

### Apple

The Apple adapter is a thin Swift C-ABI bridge to `FoundationModels`; it does
not bundle a model. It compiles to an unavailable fallback with older Apple
SDKs, and runtime availability distinguishes old OS versions, ineligible
hardware, disabled Apple Intelligence, and a model that is still preparing.
The hardware integration test is ignored in ordinary CI and can be run with
`cargo test --test apple_integration -- --ignored` on a configured Mac.

### Windows

The Windows adapter calls Phi Silica through the Windows App SDK's
`Microsoft.Windows.AI.Text.LanguageModel`, the same API flutter_local_ai uses.
The bindings are generated from Microsoft's metadata
(`tools/windows-bindings`) and committed, so building needs only the
`windows-core` crate. No C++ toolchain or NuGet step is needed.

Microsoft gates Phi Silica at runtime, and each gate is reported through
`availability()`:

- a Copilot+ PC (NPU), or a supported NVIDIA/AMD GPU with current drivers;
- Windows 11 25H2 or newer;
- an app with **package identity** that declares the `systemAIModels`
  capability. An unpackaged process reports `ProviderNotInstalled`. The stable
  Windows App SDK channel also requires the app to unlock the Limited Access
  Feature.

When the model isn't installed yet, `availability()` reports `ModelNotReady`.
Call `model.prepare().await` to let Windows download it, after obtaining the
user's consent; nothing else ever triggers a download. The API is stateless,
so the session replays the conversation. `LanguageModelOptions` has no
output-token limit or seed, so `max_output_tokens` and `seed` are not applied
on Windows.

### Ubuntu (inference snaps)

On Linux, `detect()` uses [Canonical inference snaps](https://documentation.ubuntu.com/inference-snaps/)
such as `qwen3`, `gemma3` or `deepseek-r1`. Each one packages a model with an
engine optimized for the machine, and serves an OpenAI-compatible API on
localhost:

```sh
sudo snap install qwen3     # the user installs a model once
```

Discovery finds installed inference snaps, then reads each one's API URL from
`<snap> status --format=json`, the documented way to find it. It uses the first
snap whose service is running, or the one named by `rust_local_ai::inference_snap("qwen3")` or the
`RUST_LOCAL_AI_INFERENCE_SNAP` environment variable. It never installs a snap,
starts a service, switches an engine or downloads anything. A stopped service
is reported as `ModelNotReady`, with the `sudo snap start` command to run.

### Other local providers

`rust_local_ai::openai_compatible(url, model)` uses any local server that
speaks the OpenAI chat-completions API, such as llama.cpp's `llama-server`,
Ollama, LM Studio or Foundry Local:

```rust,ignore
let model = rust_local_ai::openai_compatible("http://localhost:11434/v1", Some("llama3.2"))?;
```

Only plain `http://` URLs on this machine are accepted (`localhost`,
`127.0.0.0/8`, `::1`), so prompts never leave it. The client is a small
built-in HTTP/1.1 implementation with no dependencies. It supports blocking
and SSE-streamed generation, and keeps the transcript client-side.
`cancel()`, or dropping the future or stream, aborts the request on the
socket.

## Dependencies and runtimes

The crate keeps its dependency tree small. At runtime it uses only `serde`,
`serde_json`, `futures-core`, `tokio` (with only its `sync` feature) and the
`async-trait` macro. On Windows, `windows-core` is added for the Phi Silica
bindings. The `a2ui` feature adds `a2ui-types`. Local HTTP uses a built-in
client, not an HTTP crate.

It doesn't need a tokio runtime and works under any async executor. Native
calls run on their own threads rather than a runtime's blocking pool. Dropping
a generation future cancels the native generation, and the native session
stays alive until that call returns.

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
rust_local_ai = { version = "0.2", features = ["genui"] }  # or "a2ui"
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
ongoing chats. Generation uses a 900-token budget, and its output is checked in
several ways before it is returned:

- The parser handles code fences, surrounding prose, trailing commas, stray
  braces and wrapper objects. It repairs output cut off by the token budget,
  so a partial module still renders. It never panics and runs in linear time
  (both are covered by fuzz-style tests).
- Numbers written as strings (`"$1,200"`) are converted to JSON numbers.
  Unknown blocks are dropped, and missing fields get defaults.
- When the output can't be used, generation is retried once with a stricter
  compact prompt (`GenUiOptions::max_attempts`, default 2). Backend errors and
  cancellation are returned immediately and never retried.
- Backends that support structured output are constrained to the module's JSON
  Schema.

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
