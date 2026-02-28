use std::collections::HashMap;
use std::time::Duration;

/// OTLP transport protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// gRPC transport (port 4317).
    Grpc,
    /// HTTP with Protobuf encoding (default, port 4318).
    HttpProtobuf,
    /// HTTP with JSON encoding (port 4318).
    HttpJson,
}

/// Configuration options for OpenTelemetry setup.
///
/// Use [`OtelOptions::builder()`] to construct an instance.
/// All fields are optional; unset values fall back to environment variables, then defaults.
#[derive(Debug, Clone, Default)]
pub struct OtelOptions {
    pub(crate) endpoint: Option<String>,
    pub(crate) protocol: Option<Protocol>,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) resource_attributes: HashMap<String, String>,
    pub(crate) export_timeout: Option<Duration>,
}

impl OtelOptions {
    /// Create a new builder for `OtelOptions`.
    pub fn builder() -> OtelOptionsBuilder {
        OtelOptionsBuilder::default()
    }
}

/// Builder for [`OtelOptions`].
#[derive(Debug, Default)]
pub struct OtelOptionsBuilder {
    endpoint: Option<String>,
    protocol: Option<Protocol>,
    headers: HashMap<String, String>,
    resource_attributes: HashMap<String, String>,
    export_timeout: Option<Duration>,
}

impl OtelOptionsBuilder {
    /// Set the OTLP endpoint (e.g. `"http://collector:4317"`).
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Set the OTLP transport protocol.
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    /// Set headers to include in OTLP export requests (e.g. authorization tokens).
    pub fn headers(
        mut self,
        headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.headers = headers
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }

    /// Set additional resource attributes (e.g. `("deployment.environment", "production")`).
    pub fn resource_attributes(
        mut self,
        attrs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.resource_attributes = attrs
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }

    /// Set the export timeout for OTLP requests.
    pub fn export_timeout(mut self, timeout: Duration) -> Self {
        self.export_timeout = Some(timeout);
        self
    }

    /// Build the [`OtelOptions`].
    pub fn build(self) -> OtelOptions {
        OtelOptions {
            endpoint: self.endpoint,
            protocol: self.protocol,
            headers: self.headers,
            resource_attributes: self.resource_attributes,
            export_timeout: self.export_timeout,
        }
    }
}
