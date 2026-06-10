# AGENTS.md — `phenotype-otel`

## Project Overview

Phenotype OpenTelemetry bridge crate. Wraps `opentelemetry`, `opentelemetry-otlp`,
and `tracing-opentelemetry` into a single `pheno_otel::init(service_name)` call
that installs the global tracer + tracing pipeline.

## Stack

- Language: Rust (edition 2021)
- Build: Cargo
- CI: GitHub Actions (`.github/workflows/ci.yml`)

## Key Commands

```bash
# Build & test
cargo check --all-features
cargo test --workspace --all-features
cargo build --release --workspace

# Lint
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings

# Security
cargo deny check licenses
cargo audit
```

## Quality Gate (run before PR)

```bash
task quality
```

## Architecture (Hexagonal)

- **Domain:** `pheno_otel` — single-crate library.
- **Port:** `init(service_name: &str)` is the public port (one-shot setup).
- **Adapters:**
  - `opentelemetry_otlp` HTTP exporter (default `http://localhost:4318`)
  - `tracing-subscriber` `EnvFilter` + `fmt` layer
  - `tracing-opentelemetry` bridge layer
- **Resources:** `OTEL_EXPORTER_OTLP_ENDPOINT`, `RUST_LOG`.

## Governance

- **DAG Stage 0 — State Unification:** synced with `main`, all branches
  cleaned.
- **DAG Stage 1 — Tooling Standardization:** Taskfile.yml + deny.toml
  enforce SSOT recipes.
- **DAG Stage 2 — Hexagonal / Layer Refactor:** port/adapter pattern
  already in place (single-crate library).
- **DAG Stage 3 — QA Hardening:** CI cargo test, clippy, fmt, deny.

## License

MIT OR Apache-2.0
