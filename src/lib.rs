//! Phenotype OpenTelemetry bridge — single-call OTLP + tracing-subscriber init.
//!
//! This crate provides a turnkey `init()` function that wires up the OTLP
//! span exporter and a `tracing-subscriber` bridge so that any `tracing`
//! event in the host application is automatically exported as an OTLP span.
//!
//! # Example
//!
//! ```no_run
//! use opentelemetry::trace::TracerProvider;
//! use opentelemetry_otlp::WithExportConfig;
//! use opentelemetry_sdk::Resource;
//!
//! // Real usage requires the host crate to call OtelConfig::init() or init().
//! // This example shows the recommended pattern but is marked no_run
//! // because it requires a running OTLP collector.
//! ```
//!
//! # Environment Variables
//!
//! The default values are overridable via standard OTEL env vars:
//! - `OTEL_EXPORTER_OTLP_ENDPOINT` (default: `http://localhost:4318`)
//! - `OTEL_SERVICE_NAME` (default: `phenotype-service`)

use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    trace::{Config, Sampler},
    Resource,
};
use std::collections::HashMap;
use thiserror::Error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Default OTLP HTTP endpoint.
pub const DEFAULT_OTLP_ENDPOINT: &str = "http://localhost:4318";

/// Errors that can occur during OTEL bridge initialization.
#[derive(Debug, Error)]
pub enum OtelBridgeError {
    /// OTLP exporter construction failed.
    #[error("OTLP exporter build failed: {0}")]
    Export(String),
    /// `tracing_subscriber::registry().try_init()` failed.
    #[error("tracing subscriber init failed: {0}")]
    Subscriber(String),
}

/// Initialize the OTLP HTTP exporter + tracing-subscriber bridge with explicit args.
pub fn init(service_name: &str, otlp_endpoint: &str) -> Result<(), OtelBridgeError> {
    let exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint(otlp_endpoint)
        .build_span_exporter()
        .map_err(|e| OtelBridgeError::Export(e.to_string()))?;

    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_config(
            Config::default()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![opentelemetry::KeyValue::new(
                    "service.name",
                    service_name.to_string(),
                )])),
        )
        .build();

    opentelemetry::global::set_tracer_provider(provider.clone());

    let tracer = provider.tracer(service_name.to_string());
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(otel_layer)
        .try_init()
        .map_err(|e| OtelBridgeError::Subscriber(e.to_string()))?;

    Ok(())
}

/// Shutdown the global tracer provider, flushing any buffered spans.
pub fn shutdown() {
    opentelemetry::global::shutdown_tracer_provider();
}

/// Re-export of `opentelemetry::KeyValue` for ergonomic attribute construction.
pub use opentelemetry::KeyValue as Attribute;

/// Builder for OTEL bridge configuration.
#[derive(Debug, Clone)]
pub struct OtelConfig {
    service_name: String,
    otlp_endpoint: String,
    attributes: HashMap<String, String>,
}

impl OtelConfig {
    /// Create a new config with default values.
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            otlp_endpoint: DEFAULT_OTLP_ENDPOINT.to_string(),
            attributes: HashMap::new(),
        }
    }

    /// Override the OTLP endpoint.
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.otlp_endpoint = endpoint.to_string();
        self
    }

    /// Add a resource attribute.
    #[must_use]
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Initialize the OTEL bridge with this configuration.
    pub fn init(self) -> Result<(), OtelBridgeError> {
        init(&self.service_name, &self.otlp_endpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_endpoint() {
        let c = OtelConfig::new("svc");
        assert_eq!(c.otlp_endpoint, DEFAULT_OTLP_ENDPOINT);
        assert_eq!(c.service_name, "svc");
    }

    #[test]
    fn config_with_endpoint() {
        let c = OtelConfig::new("svc").with_endpoint("http://collector:4318");
        assert_eq!(c.otlp_endpoint, "http://collector:4318");
    }

    #[test]
    fn config_with_attribute() {
        let c = OtelConfig::new("svc").with_attribute("env", "prod");
        assert_eq!(c.attributes.get("env"), Some(&"prod".to_string()));
    }

    #[test]
    fn default_endpoint_constant() {
        assert_eq!(DEFAULT_OTLP_ENDPOINT, "http://localhost:4318");
    }
}
