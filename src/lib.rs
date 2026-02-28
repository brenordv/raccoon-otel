//! # raccoon-otel
//!
//! Drop-in OpenTelemetry bridge for Rust applications using the [`tracing`] crate.
//!
//! One function call to enable OTel export for traces and logs — existing
//! `#[instrument]`, `tracing::info!()`, and span macros work unchanged.
//!
//! ## Quick Start
//!
//! ```no_run
//! # fn main() -> anyhow::Result<()> {
//! let _guard = raccoon_otel::setup_otel("my-service", None)?;
//!
//! tracing::info!("This goes to stdout AND the OTel backend");
//! # Ok(())
//! # }
//! ```
//!
//! ## Configured Usage
//!
//! ```no_run
//! use std::time::Duration;
//! use raccoon_otel::{OtelOptions, Protocol};
//!
//! # fn main() -> anyhow::Result<()> {
//! let _guard = raccoon_otel::setup_otel("my-service", Some(
//!     OtelOptions::builder()
//!         .endpoint("http://collector:4318")
//!         .protocol(Protocol::HttpProtobuf)
//!         .resource_attributes([("deployment.environment", "production")])
//!         .headers([("Authorization", "Bearer token123")])
//!         .export_timeout(Duration::from_secs(30))
//!         .build()
//! ))?;
//! # Ok(())
//! # }
//! ```

mod env;
mod guard;
mod options;
mod providers;
mod resource;
mod subscriber;

pub mod re_exports;

pub use guard::OtelGuard;
pub use options::{OtelOptions, OtelOptionsBuilder, Protocol};

use anyhow::Context;

/// Initialize OpenTelemetry with the given service name and optional configuration.
///
/// Sets up trace and log export pipelines, composes a global tracing subscriber
/// with OTel layers, and returns an [`OtelGuard`] that manages provider lifecycles.
///
/// The guard **must** be held for the duration of the application. Dropping it
/// triggers a graceful flush and shutdown of all providers.
///
/// # Configuration Priority
///
/// 1. **Programmatic** — values set in [`OtelOptions`]
/// 2. **Environment variables** — `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_SERVICE_NAME`, etc.
/// 3. **Defaults** — `http://localhost:4318` (HTTP+protobuf), 30s timeout, `info` log level
///
/// # Errors
///
/// Returns an error if:
/// - A required transport feature is not enabled (e.g. `grpc` or `http`)
/// - Provider or exporter initialization fails
/// - The global tracing subscriber has already been set
pub fn setup_otel(service_name: &str, options: Option<OtelOptions>) -> anyhow::Result<OtelGuard> {
    let opts = options.unwrap_or_default();
    let resolved = env::resolve_config(service_name, &opts);

    // Set up W3C trace context propagation for distributed tracing
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );

    let resource = resource::build_resource(&resolved.service_name, &resolved.resource_attributes);

    let tracer_provider = if cfg!(feature = "traces") {
        Some(
            providers::tracer::build_tracer_provider(resource.clone(), &resolved)
                .context("Failed to initialize tracer provider")?,
        )
    } else {
        None
    };

    let logger_provider = if cfg!(feature = "logs") {
        Some(
            providers::logger::build_logger_provider(resource, &resolved)
                .context("Failed to initialize logger provider")?,
        )
    } else {
        None
    };

    subscriber::compose_subscriber(tracer_provider.as_ref(), logger_provider.as_ref())
        .context("Failed to compose and set global subscriber")?;

    Ok(OtelGuard::new(tracer_provider, logger_provider))
}
