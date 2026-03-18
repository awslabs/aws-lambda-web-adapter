# AGENTS.md

## Build & Test

- Build: `cargo build`
- Run all tests: `cargo nextest run`
- Run a single test: `cargo nextest run <test_name>`
- Lint: `cargo clippy --all-targets`
- Docs: `cargo doc --no-deps`

## Project Structure

- `src/lib.rs` — Core adapter logic, public API (`Adapter`, `AdapterOptions`, `Protocol`, `LambdaInvokeMode`)
- `src/main.rs` — Binary entrypoint
- `src/readiness.rs` — Readiness check checkpoint timing
- `tests/integ_tests/` — Integration tests
- `tests/e2e_tests/` — End-to-end tests (marked `#[ignore]`, require external setup)

## Key Conventions

- Environment variables use `AWS_LWA_` prefix
- Tests that mutate env vars rely on nextest's per-test process isolation
