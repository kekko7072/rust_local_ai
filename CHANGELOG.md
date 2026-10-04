# Changelog

## 0.2.1

### Fixed
- Fix `cancel()` not stopping a generation on Windows for the
  OpenAI-compatible and inference-snap backends. Shutting a socket down from
  another thread does not wake a blocked read on Windows, so the HTTP client
  now reads in short intervals, checks an abort flag in between, and still
  honours the full idle timeout.

## 0.2.0

### Added
- **Windows:** Phi Silica through the Windows App SDK `LanguageModel` API, with
  generated bindings. The only new dependency is `windows-core`, and only on
  Windows. Ready states map to availability reasons.
- **Ubuntu:** Canonical inference snaps, discovered with
  `<snap> status --format=json`. `rust_local_ai::inference_snap(name)` and the
  `RUST_LOCAL_AI_INFERENCE_SNAP` variable select one.
- `rust_local_ai::openai_compatible(url, model)` for local llama.cpp, Ollama,
  LM Studio or Foundry Local servers, on any OS. It uses a built-in HTTP/1.1
  client limited to loopback addresses, with streaming and cancellation.
- `LocalAiModel::prepare()`, the only call that may trigger a model download.
- `genui` feature: a port of flutter_local_ai's generative UI
  (`LocalAiUiGenerator`, `GenUiModuleSpec`, `parse_model_output`).
- `a2ui` feature: modules as A2UI v0.9 messages via `a2ui-types`.
- `LocalAiError::InvalidModelOutput` and `BackendKind::OpenAiCompatible`.
- The crate no longer needs a tokio runtime and works with any async executor.

### Fixed
- Apple: a use-after-free when a generation future was dropped while Swift
  was still running. Dropping the future now also cancels the generation.
- macOS never built on the declared 1.75 MSRV (`unsafe extern` needs 1.82).

### Changed
- Fewer dependencies: `thiserror`, `pin-project-lite` and `futures-util` were
  removed from normal dependencies, and tokio now uses only `sync`.

## 0.1.0

- Initial release: portable session API, fake test backend, and the Apple
  Foundation Models adapter.
