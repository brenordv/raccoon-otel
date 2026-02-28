use std::collections::HashMap;
use std::time::Duration;

use crate::options::{OtelOptions, Protocol};

const DEFAULT_GRPC_ENDPOINT: &str = "http://localhost:4317";
const DEFAULT_HTTP_ENDPOINT: &str = "http://localhost:4318";
const DEFAULT_EXPORT_TIMEOUT: Duration = Duration::from_secs(30);

/// Fully resolved configuration after merging programmatic options, env vars, and defaults.
///
/// Priority (highest to lowest):
/// 1. Programmatic — values set in [`OtelOptions`]
/// 2. Environment variables — `OTEL_EXPORTER_OTLP_*`
/// 3. Defaults — localhost endpoints, 30s timeout
#[derive(Debug, Clone)]
pub(crate) struct ResolvedConfig {
    pub service_name: String,
    pub endpoint: String,
    pub protocol: Protocol,
    // TODO: pass programmatic headers to exporter builders (tonic MetadataMap / reqwest headers).
    // The OTLP SDK already reads OTEL_EXPORTER_OTLP_HEADERS natively for env-var-based headers.
    #[allow(dead_code)]
    pub headers: HashMap<String, String>,
    pub resource_attributes: HashMap<String, String>,
    pub export_timeout: Duration,
}

/// Resolve configuration by merging programmatic options, env vars, and defaults.
pub(crate) fn resolve_config(service_name: &str, opts: &OtelOptions) -> ResolvedConfig {
    let service_name =
        env_var_non_empty("OTEL_SERVICE_NAME").unwrap_or_else(|| service_name.to_owned());

    let protocol = opts
        .protocol
        .or_else(parse_protocol_env)
        .unwrap_or(Protocol::HttpProtobuf);

    let default_endpoint = match protocol {
        Protocol::Grpc => DEFAULT_GRPC_ENDPOINT,
        Protocol::HttpProtobuf | Protocol::HttpJson => DEFAULT_HTTP_ENDPOINT,
    };

    let endpoint = opts
        .endpoint
        .clone()
        .or_else(|| env_var_non_empty("OTEL_EXPORTER_OTLP_ENDPOINT"))
        .unwrap_or_else(|| default_endpoint.to_owned());

    let mut headers = parse_headers_env();
    // Programmatic headers take precedence over env var headers
    headers.extend(opts.headers.clone());

    let export_timeout = opts
        .export_timeout
        .or_else(parse_timeout_env)
        .unwrap_or(DEFAULT_EXPORT_TIMEOUT);

    ResolvedConfig {
        service_name,
        endpoint,
        protocol,
        headers,
        resource_attributes: opts.resource_attributes.clone(),
        export_timeout,
    }
}

fn env_var_non_empty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

fn parse_protocol_env() -> Option<Protocol> {
    env_var_non_empty("OTEL_EXPORTER_OTLP_PROTOCOL").and_then(|v| match v.as_str() {
        "grpc" => Some(Protocol::Grpc),
        "http/protobuf" => Some(Protocol::HttpProtobuf),
        "http/json" => Some(Protocol::HttpJson),
        _ => None,
    })
}

fn parse_headers_env() -> HashMap<String, String> {
    env_var_non_empty("OTEL_EXPORTER_OTLP_HEADERS")
        .map(|val| {
            val.split(',')
                .filter_map(|pair| {
                    let (key, value) = pair.split_once('=')?;
                    let key = key.trim();
                    let value = value.trim();
                    if key.is_empty() {
                        return None;
                    }
                    Some((key.to_owned(), value.to_owned()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_timeout_env() -> Option<Duration> {
    env_var_non_empty("OTEL_EXPORTER_OTLP_TIMEOUT")
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_millis)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    // Env vars are process-global; serialize tests that mutate them.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_otel_env() {
        std::env::remove_var("OTEL_SERVICE_NAME");
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        std::env::remove_var("OTEL_EXPORTER_OTLP_PROTOCOL");
        std::env::remove_var("OTEL_EXPORTER_OTLP_HEADERS");
        std::env::remove_var("OTEL_EXPORTER_OTLP_TIMEOUT");
    }

    #[test]
    fn resolve_defaults_with_no_options_or_env() {
        let _lock = ENV_LOCK.lock();
        clear_otel_env();

        let opts = OtelOptions::default();
        let resolved = resolve_config("test-service", &opts);

        assert_eq!(resolved.service_name, "test-service");
        assert_eq!(resolved.endpoint, "http://localhost:4318");
        assert_eq!(resolved.protocol, Protocol::HttpProtobuf);
        assert!(resolved.headers.is_empty());
        assert!(resolved.resource_attributes.is_empty());
        assert_eq!(resolved.export_timeout, Duration::from_secs(30));
    }

    #[test]
    fn programmatic_options_take_precedence() {
        let _lock = ENV_LOCK.lock();
        clear_otel_env();
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://env:4317");

        let opts = OtelOptions::builder()
            .endpoint("http://programmatic:4317")
            .protocol(Protocol::HttpProtobuf)
            .export_timeout(Duration::from_secs(60))
            .build();

        let resolved = resolve_config("test-service", &opts);

        assert_eq!(resolved.endpoint, "http://programmatic:4317");
        assert_eq!(resolved.protocol, Protocol::HttpProtobuf);
        assert_eq!(resolved.export_timeout, Duration::from_secs(60));

        clear_otel_env();
    }

    #[test]
    fn parse_headers_from_env() {
        let _lock = ENV_LOCK.lock();
        clear_otel_env();
        std::env::set_var("OTEL_EXPORTER_OTLP_HEADERS", "key1=val1,key2=val2");

        let headers = parse_headers_env();

        assert_eq!(headers.get("key1"), Some(&"val1".to_owned()));
        assert_eq!(headers.get("key2"), Some(&"val2".to_owned()));

        clear_otel_env();
    }

    #[test]
    fn http_protocol_uses_port_4318_default() {
        let _lock = ENV_LOCK.lock();
        clear_otel_env();

        let opts = OtelOptions::builder()
            .protocol(Protocol::HttpProtobuf)
            .build();

        let resolved = resolve_config("test-service", &opts);

        assert_eq!(resolved.endpoint, "http://localhost:4318");
    }
}
