use anyhow::Context;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;

use crate::env::ResolvedConfig;
use crate::options::Protocol;

/// Build and globally register a [`SdkTracerProvider`] with an OTLP exporter.
///
/// # Errors
///
/// Returns an error if the OTLP exporter or provider fails to initialize.
pub(crate) fn build_tracer_provider(
    resource: Resource,
    config: &ResolvedConfig,
) -> anyhow::Result<SdkTracerProvider> {
    let exporter = build_span_exporter(config).context("Failed to build OTLP span exporter")?;

    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    // Register globally so auto-instrumentation and context propagation work
    opentelemetry::global::set_tracer_provider(provider.clone());

    Ok(provider)
}

fn build_span_exporter(
    config: &ResolvedConfig,
) -> anyhow::Result<opentelemetry_otlp::SpanExporter> {
    match config.protocol {
        Protocol::Grpc => {
            #[cfg(feature = "grpc")]
            {
                let exporter = opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_endpoint(&config.endpoint)
                    .with_timeout(config.export_timeout)
                    .build()
                    .context("Failed to build gRPC span exporter")?;
                Ok(exporter)
            }
            #[cfg(not(feature = "grpc"))]
            {
                anyhow::bail!(
                    "gRPC transport requested but the `grpc` feature is not enabled. \
                     Enable it in Cargo.toml: raccoon-otel = {{ features = [\"grpc\"] }}"
                );
            }
        }
        Protocol::HttpProtobuf | Protocol::HttpJson => {
            #[cfg(feature = "http")]
            {
                let endpoint = format!("{}/v1/traces", config.endpoint.trim_end_matches('/'));
                let exporter = opentelemetry_otlp::SpanExporter::builder()
                    .with_http()
                    .with_endpoint(endpoint)
                    .with_timeout(config.export_timeout)
                    .build()
                    .context("Failed to build HTTP span exporter")?;
                Ok(exporter)
            }
            #[cfg(not(feature = "http"))]
            {
                anyhow::bail!(
                    "HTTP transport requested but the `http` feature is not enabled. \
                     Enable it in Cargo.toml: raccoon-otel = {{ features = [\"http\"] }}"
                );
            }
        }
    }
}
