use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

/// Compose and globally register a tracing subscriber with OTel layers.
///
/// Layers added:
/// - [`EnvFilter`] — respects `RUST_LOG` / `OTEL_LOG_LEVEL` env vars (defaults to `info`)
/// - `fmt` — formatted output to stdout
/// - `OpenTelemetryLayer` — bridges tracing spans to OTel traces (if tracer provider given)
/// - `OpenTelemetryTracingBridge` — bridges tracing events to OTel logs (if logger provider given)
///
/// # Errors
///
/// Returns an error if the global subscriber has already been set.
pub(crate) fn compose_subscriber(
    tracer_provider: Option<&SdkTracerProvider>,
    logger_provider: Option<&SdkLoggerProvider>,
) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true);

    let otel_trace_layer = tracer_provider.map(|tp| {
        use opentelemetry::trace::TracerProvider as _;
        tracing_opentelemetry::layer().with_tracer(tp.tracer("raccoon-otel"))
    });

    let otel_log_layer =
        logger_provider.map(opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new);

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(otel_trace_layer)
        .with(otel_log_layer);

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| anyhow::anyhow!("Failed to set global subscriber: {e}"))?;

    Ok(())
}
