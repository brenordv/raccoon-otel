//! Curated re-exports of key OpenTelemetry and tracing types.
//!
//! These re-exports let users access commonly needed types without adding
//! direct dependencies on `opentelemetry`, `opentelemetry_sdk`,
//! `tracing-opentelemetry`, or `tracing-subscriber` to their own `Cargo.toml`.

/// Re-export of the `tracing` crate for convenient access.
pub use tracing;

/// Re-export of `tracing_subscriber`, for building [`BoxedLayer`] values to
/// pass to [`setup_otel_with_layers`] without depending on a matching
/// `tracing-subscriber` version directly.
///
/// [`BoxedLayer`]: crate::BoxedLayer
/// [`setup_otel_with_layers`]: crate::setup_otel_with_layers
pub use tracing_subscriber;

/// Re-export of the `opentelemetry` API crate.
pub use opentelemetry;

/// Re-export of the `opentelemetry_sdk` crate.
pub use opentelemetry_sdk;

/// Re-export of `tracing_opentelemetry` for span context extensions.
pub use tracing_opentelemetry;
