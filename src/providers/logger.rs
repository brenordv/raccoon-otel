use anyhow::Context;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::Resource;

use crate::env::ResolvedConfig;
use crate::options::Protocol;

/// Build a [`SdkLoggerProvider`] with an OTLP exporter.
///
/// # Errors
///
/// Returns an error if the OTLP exporter or provider fails to initialize.
pub(crate) fn build_logger_provider(
    resource: Resource,
    config: &ResolvedConfig,
) -> anyhow::Result<SdkLoggerProvider> {
    let exporter = build_log_exporter(config).context("Failed to build OTLP log exporter")?;

    let provider = SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    Ok(provider)
}

fn build_log_exporter(config: &ResolvedConfig) -> anyhow::Result<opentelemetry_otlp::LogExporter> {
    match config.protocol {
        Protocol::Grpc => {
            #[cfg(feature = "grpc")]
            {
                let exporter = opentelemetry_otlp::LogExporter::builder()
                    .with_tonic()
                    .with_endpoint(&config.endpoint)
                    .with_timeout(config.export_timeout)
                    .build()
                    .context("Failed to build gRPC log exporter")?;
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
                let exporter = opentelemetry_otlp::LogExporter::builder()
                    .with_http()
                    .with_endpoint(&config.endpoint)
                    .with_timeout(config.export_timeout)
                    .build()
                    .context("Failed to build HTTP log exporter")?;
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
