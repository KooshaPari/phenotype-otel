//! `pheno-otel` basic example.
//!
//! Initialises the global OTLP + tracing pipeline with
//! `service_name="example"`, emits a few spans / events, and shuts the
//! provider down cleanly on exit.
//!
//! Run with:
//! ```text
//! cargo run --example basic
//! # or, with a local OTel collector on :4318:
//! OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo run --example basic
//! ```

use tracing::{info, info_span};

fn main() -> Result<(), opentelemetry::global::Error> {
    pheno_otel::init("example")?;

    let root = info_span!("example.root", run_id = 42);
    let _enter = root.enter();

    info!("starting example");
    do_work("alpha");
    do_work("beta");
    info!("done");

    pheno_otel::shutdown();
    Ok(())
}

#[tracing::instrument]
fn do_work(label: &str) {
    info!(label, "working");
}
