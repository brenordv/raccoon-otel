use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, Layer};

use crate::BoxedLayer;

/// Compose and globally register a tracing subscriber with OTel layers.
///
/// Layers added:
/// - [`EnvFilter`] — respects `RUST_LOG` / `OTEL_LOG_LEVEL` env vars (defaults to `info`)
/// - `fmt` — formatted output to stdout
/// - `OpenTelemetryLayer` — bridges tracing spans to OTel traces (if tracer provider given)
/// - `OpenTelemetryTracingBridge` — bridges tracing events to OTel logs (if logger provider given)
/// - any `extra_layers` supplied by the caller (e.g. a file writer)
///
/// # Errors
///
/// Returns an error if the global subscriber has already been set.
pub(crate) fn compose_subscriber(
    tracer_provider: Option<&SdkTracerProvider>,
    logger_provider: Option<&SdkLoggerProvider>,
    extra_layers: Vec<BoxedLayer>,
) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let mut layers: Vec<BoxedLayer> = Vec::new();

    layers.push(tracing_subscriber::fmt::layer().with_target(true).boxed());

    if let Some(tp) = tracer_provider {
        use opentelemetry::trace::TracerProvider as _;
        let otel_trace_layer =
            tracing_opentelemetry::layer().with_tracer(tp.tracer("raccoon-otel"));
        layers.push(otel_trace_layer.boxed());
    }

    if let Some(lp) = logger_provider {
        let otel_log_layer =
            opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(lp);
        layers.push(otel_log_layer.boxed());
    }

    layers.extend(extra_layers);

    let subscriber = tracing_subscriber::registry().with(layers.with_filter(env_filter));

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| anyhow::anyhow!("Failed to set global subscriber: {e}"))?;

    Ok(())
}
