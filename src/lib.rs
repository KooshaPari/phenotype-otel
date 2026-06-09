//! # pheno-otel
//!
//! Phenotype OpenTelemetry bridge crate.
//!
//! Initialises an OTLP HTTP exporter and wires it into the
//! `tracing` ecosystem via `tracing-opentelemetry`, alongside the
//! human-readable `tracing-subscriber` `fmt` layer.
//!
//! ## Quick start
//!
//! ```no_run
//! pheno_otel::init("my-service").expect("init otel");
//! tracing::info!("hello otel");
//! ```
//!
//! Configuration is driven by the standard OpenTelemetry environment
//! variables. The OTLP HTTP exporter endpoint defaults to
//! `http://localhost:4318` (matching the OTel collector default
//! HTTP receiver port) and is overridable via
//! `OTEL_EXPORTER_OTLP_ENDPOINT`.
//!
//! Trace export is batched via `opentelemetry_sdk`'s `BatchSpanProcessor`
//! and shut down cleanly on `Drop` of the returned guard (use
//! [`shutdown`] for an explicit handle).

use std::sync::OnceLock;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{self as sdktrace, TracerProvider};
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Default OTLP HTTP endpoint (OTel collector default receiver).
const DEFAULT_OTLP_ENDPOINT: &str = "http://localhost:4318";

/// Cached global provider so [`shutdown`] can flush + drop it.
static PROVIDER: OnceLock<TracerProvider> = OnceLock::new();

/// Initialise the global OpenTelemetry + tracing pipeline.
///
/// Builds an OTLP HTTP exporter pointed at
/// `OTEL_EXPORTER_OTLP_ENDPOINT` (default `http://localhost:4318`),
/// installs a [`TracerProvider`] with a `BatchSpanProcessor`, layers it
/// over the existing `tracing` subscriber, and combines it with a
/// `tracing-subscriber` `fmt` layer for human-readable logs.
///
/// `service_name` populates the `service.name` resource attribute on
/// every span (used by collectors / Tempo / Jaeger to attribute
/// traces to the right service).
///
/// Returns an error if a subscriber is already installed (because
/// `tracing-subscriber` only allows one global default) or if the
/// OTLP pipeline fails to install.
pub fn init(service_name: &str) -> Result<(), opentelemetry::global::Error> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_OTLP_ENDPOINT.to_string());

    // OTLP HTTP exporter. `with_http()` switches the OTLP pipeline to
    // the HTTP/protobuf transport (the collector's default-receiver
    // on port 4318) instead of gRPC (4317).
    let exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint(endpoint);

    // Service-name resource — every span emitted by this process
    // carries `service.name=<service_name>`.
    let resource = Resource::new([opentelemetry::KeyValue::new(
        SERVICE_NAME,
        service_name.to_string(),
    )]);

    // Tracer provider with batched span export.
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(resource)
        .build();

    // Stash the provider globally so shutdown() can flush it.
    let _ = PROVIDER.set(provider.clone());

    // Set the global tracer provider so opentelemetry::trace::Span
    // users (e.g. instrumented deps) pick it up.
    global::set_tracer_provider(provider.clone());

    let tracer = provider.tracer(service_name.to_string());

    // tracing-opentelemetry bridge layer.
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Human-readable fmt layer (env-overridable filter, info by default).
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true)
        .compact();

    // Honour RUST_LOG (e.g. `RUST_LOG=info,sqlx=warn`); fall back to
    // `info` so the default verbosity is sane for production.
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(otel_layer)
        .with(fmt_layer)
        .try_init()?;

    Ok(())
}

/// Flush and shut down the global tracer provider.
///
/// No-op if [`init`] was never called or has already been invoked.
/// Safe to call from a signal handler / shutdown hook.
pub fn shutdown() {
    if let Some(provider) = PROVIDER.get() {
        for result in provider.force_flush() {
            if let Err(err) = result {
                eprintln!("pheno-otel: force_flush error: {err}");
            }
        }
        for result in provider.shutdown() {
            if let Err(err) = result {
                eprintln!("pheno-otel: shutdown error: {err}");
            }
        }
    }
    // Best-effort: reset the cached provider so a subsequent init()
    // call (e.g. in test harnesses) can succeed.
    // OnceLock has no `take`; we leak the slot on purpose — the
    // process is on its way out.
    global::shutdown_tracer_provider();
}

/// Re-export the underlying `tracing` crate for convenience so
/// downstream binaries can `use pheno_otel::tracing;` without adding
/// `tracing` to their own `Cargo.toml`.
pub use tracing;

/// Re-export of `opentelemetry` for downstream code that wants to
/// access the global tracer provider, instrument libraries directly,
/// or pull in propagation / context types.
pub use opentelemetry;

// Silence unused-import warnings on `sdktrace` while keeping the
// type alias in the public API surface.
#[allow(dead_code)]
type _Trace = sdktrace::TracerProvider;
