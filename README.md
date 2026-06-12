<!-- AI-DD-META:START -->
<!-- This repository is planned, maintained, and managed by AI Agents only. -->
<!-- Slop issues are expected and intentionally present as part of an HITL-less -->
<!-- /minimized AI-DD metaproject of learning, refining, and building brute-force -->
<!-- training for both agents and the human operator. -->
![Downloads](https://img.shields.io/github/downloads/KooshaPari/phenotype-otel/total?style=flat-square&label=downloads&color=blue)
![GitHub release](https://img.shields.io/github/v/release/KooshaPari/phenotype-otel?style=flat-square&label=release)
![License](https://img.shields.io/github/license/KooshaPari/phenotype-otel?style=flat-square)
![AI-Slop](https://img.shields.io/badge/AI--DD-Slop%20Expected-orange?style=flat-square)
![AI-Only-Maintained](https://img.shields.io/badge/Planned%20%26%20Maintained%20by-AI%20Agents%20Only-red?style=flat-square)
![HITL-less](https://img.shields.io/badge/HITL--less%20AI--DD-metaproject-yellow?style=flat-square)

> ⚠️ **AI-Agent-Only Repository**
>
> This repo is **planned, maintained, and managed exclusively by AI Agents**.
> Slop issues, rough edges, and AI artifacts are **expected and intentionally
> present** as part of an **HITL-less / minimized AI-DD** metaproject focused
> on learning, refining, and brute-force training both the agents and the
> human operator. Bug reports and contributions are still welcome, but please
> expect AI-generated code, comments, and documentation throughout.
<!-- AI-DD-META:END -->
# pheno-otel

Phenotype OpenTelemetry bridge crate.

Wraps [`opentelemetry`], [`opentelemetry-otlp`], and
[`tracing-opentelemetry`] into a single `pheno_otel::init(service_name)`
call that installs:

- an OTLP HTTP exporter (default `http://localhost:4318`,
  overridable via `OTEL_EXPORTER_OTLP_ENDPOINT`),
- a `tracing-subscriber` registry with the OpenTelemetry bridge layer
  and a `fmt` human-readable layer, and
- a global `TracerProvider` so spans emitted via `tracing` *and* via
  the `opentelemetry` API are exported together.

## Usage

```rust,no_run
fn main() -> Result<(), opentelemetry::global::Error> {
    pheno_otel::init("my-service")?;

    let span = tracing::info_span!("work", kind = "demo");
    let _enter = span.enter();
    tracing::info!("hello from instrumented code");

    pheno_otel::shutdown();
    Ok(())
}
```

## Configuration

| Env var                       | Default                  | Purpose                          |
| ----------------------------- | ------------------------ | -------------------------------- |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `http://localhost:4318`  | OTLP HTTP collector endpoint     |
| `RUST_LOG`                    | `info`                   | `tracing-subscriber` env filter  |

## Layout

- `src/lib.rs` — public API (`init`, `shutdown`, re-exports).
- `examples/basic.rs` — runnable smoke test (init + spans + shutdown).

## License

MIT OR Apache-2.0
