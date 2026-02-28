//! Curated re-exports of key OpenTelemetry and tracing types.
//!
//! These re-exports let users access commonly needed types without adding
//! direct dependencies on `opentelemetry`, `opentelemetry_sdk`, or
//! `tracing-opentelemetry` to their own `Cargo.toml`.

/// Re-export of the `tracing` crate for convenient access.
pub use tracing;

/// Re-export of the `opentelemetry` API crate.
pub use opentelemetry;

/// Re-export of the `opentelemetry_sdk` crate.
pub use opentelemetry_sdk;

/// Re-export of `tracing_opentelemetry` for span context extensions.
pub use tracing_opentelemetry;
