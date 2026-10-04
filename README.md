# rust_local_ai

`rust_local_ai` is a unified Rust interface to AI capabilities provided or
managed by the operating system.

It is **not primarily another model runner or model-download framework**. The
library prefers OS-native AI, then distribution-managed AI, then an explicitly
configured installed provider. It never silently downloads a model, installs a
provider, or starts a heavyweight inference service.

> This crate is early `0.x` software. The portable API and fake backend are
> implemented and tested. Native adapters are under active development; the
> current release reports them unavailable instead of advertising placeholders.

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
| Availability detection | planned | planned | planned | planned | ✓ |
| Text generation | — | — | — | — | ✓ |
| Streaming | — | — | — | — | ✓ |
| Structured output | — | — | — | — | ✓ |
| Tool calling | — | — | — | — | — |
| Token counting | — | — | — | — | ✓ |
| Cancellation | — | — | — | — | ✓ |
| System-managed model | intended | intended | intended | varies | no |

On Linux there is deliberately no fictional universal backend. Provider
selection will prefer distribution-managed infrastructure such as Canonical
Inference Snaps, then supported providers already installed by the user.

## Testing adapters

Enable the `testing` feature to use `rust_local_ai::testing::FakeBackend` in
downstream contract tests. It supports deterministic responses, streams,
failures, cancellation, and close-retry testing without OS AI hardware.

## Generative UI and Node

The inference core has no UI-framework dependency. Future optional GenUI
interoperability will carry structured representations to A2UI, AG-UI, MCP Apps,
or Rust projects such as `adk-ui`. A future N-API crate can wrap this same core
for Electron without reimplementing native adapters in TypeScript.

## License

Licensed under either Apache-2.0 or MIT, at your option.

Created by [Francesco Vezzani](https://vezz.io).
