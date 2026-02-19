# AGENTS.md
Guidance for coding agents working in this repository.
Follow existing code patterns first.

## Stack and Layout
- Language: Rust 2021
- Server/runtime: `axum` + `tokio`
- Crate: single package (`kokoro-openai-server`)
- Main modules: `api`, `backend`, `config`, `error`, `streaming`, `validation`
- Tests: inline unit tests (`#[cfg(test)]`) in source files

## Prerequisites
- Rust toolchain installed (`cargo` available)
- Optional feature environments:
  - `coreml` for macOS
  - `directml` for Windows
  - `cuda` for CUDA systems

## Useful Environment Variables
- `RUST_LOG`
- `HOST`, `PORT`
- `API_KEY`
- `KOKORO_MODEL_PATH`
- `KOKORO_ACCELERATION`
- `KOKORO_WORKERS`
- `KOKORO_MAX_INPUT_CHARS`

## Build Commands
Run from repo root.

```bash
cargo build
cargo build --release
cargo build --release --no-default-features --features cpu
cargo build --release --no-default-features --features cuda
cargo build --release --no-default-features --features directml
```

## Run Commands
```bash
cargo run --release
cargo run --release -- --host 0.0.0.0 --port 8000
./run.sh
```

## Format and Lint
```bash
cargo fmt
cargo fmt --all -- --check
cargo clippy
cargo clippy --all-targets --all-features -- -D warnings
```

Notes:
- No `rustfmt.toml` found -> use default rustfmt behavior.
- No `clippy.toml` found -> use default clippy behavior.

## Test Commands
```bash
cargo test
cargo test --release
cargo test -- --nocapture
cargo test validation::tests
cargo test test_validate_model
cargo test validation::tests::test_validate_model -- --exact
```

Known test locations:
- `src/config.rs`
- `src/validation.rs`
- `src/streaming.rs`

Recommended validation pass before finishing non-trivial changes:
1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test`

## Code Style Guidelines

### Imports
- Prefer explicit imports, avoid glob imports.
- Group imports logically and match local file conventions.
- Let rustfmt handle wrapping and ordering.

### Formatting and Structure
- Run `cargo fmt` after edits.
- Keep functions focused; extract helpers for repeated logic.
- Prefer trailing commas in multiline expressions.
- Keep comments minimal; add them only for non-obvious intent.

### Types
- Prefer explicit domain types (`struct`, `enum`) over loosely typed data.
- Derive only necessary traits (`Debug`, `Clone`, `Serialize`, `Deserialize`, etc.).
- Keep visibility narrow; avoid unnecessary `pub` surface area.
- Prefer `&str` for borrowed strings and `String` for owned strings.
- Use `Arc<T>` for shared async state.

### Naming
- Types/traits/enums: `PascalCase`
- Functions/modules/files/variables: `snake_case`
- Constants/statics: `UPPER_SNAKE_CASE`
- Tests: descriptive behavior names (example: `test_validate_speed`)

### Error Handling
- API handlers should use `ApiResult<T>` and return `AppError` variants.
- Keep client errors stable and safe; do not leak internal details.
- Log internal failures with `tracing` (especially `error!`).
- Use `anyhow::{Result, Context}` for app/bootstrap/backend internals.
- Add `.context("...")` at fallible boundaries.
- Avoid `.unwrap()` and `.expect()` in production code.

### Validation Rules
- Validate inputs before expensive synthesis work.
- Keep validation centralized in `validation.rs` when possible.
- Validate at least: model, input, voice, speed, response format.
- Preserve strict response-format support (`wav`, `pcm`) unless requirements change.

### Async and Concurrency
- Use `tokio::task::spawn_blocking` for CPU-heavy inference/synthesis.
- Respect existing worker/concurrency limits (semaphore pattern).
- For streaming, propagate errors and terminate cleanly on receiver drop.
- Preserve graceful shutdown behavior in server code.

### Logging and Request Tracing
- Use structured `tracing` logs.
- Include request identifiers when available.
- Avoid extremely noisy logs in hot loops or per-sample paths.

### Testing Expectations
- Prefer tests near implementation (`#[cfg(test)] mod tests`).
- Cover both success and failure cases.
- Include edge values for numeric/audio conversions (`-1.0`, `0.0`, `1.0`).
- Keep tests deterministic and local (no network dependency).

## Security and API Compatibility
- Do not weaken bearer-token authentication behavior.
- Do not bypass auth middleware for protected routes.
- Keep OpenAI-compatible error response structure.
- Preserve expected headers like `X-Request-Id` and correct content type.

## Cursor/Copilot Rules
Checked for additional instructions in:
- `.cursorrules`
- `.cursor/rules/`
- `.github/copilot-instructions.md`

No Cursor or Copilot rule files were found when this file was generated.
If those files are added later, merge their guidance into this document.
