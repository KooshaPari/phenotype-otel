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
