# raccoon-otel

Drop-in OpenTelemetry bridge for Rust applications using the [`tracing`](https://crates.io/crates/tracing) crate.

**One function call** to export traces and logs to any OTLP-compatible backend. Existing `#[instrument]`, `tracing::info!()`, and span macros work unchanged.

```rust
let _guard = raccoon_otel::setup_otel("my-service", None)?;

tracing::info!("This goes to stdout AND the OTel backend");
```

## Why this crate?

Setting up OpenTelemetry in Rust today might require 5+ carefully version-aligned crate dependencies, 60+ lines of boilerplate,
and a solid understanding of how `tracing-opentelemetry`, `opentelemetry-appender-tracing`, provider lifecycles, and 
subscriber composition all fit together. 

If providers get dropped too early, spans silently vanish. If crate versions drift, compilation fails with inscrutable errors.

`raccoon-otel` solves this with a single dependency and a single function call to set everything up.

I've created other packages that follow the same spirit:
- [Python](https://github.com/nicholasgasior/log-factory-package-ext-otel)
- [C#](https://github.com/nicholasgasior/raccoon-ninja-otel)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
raccoon-otel = "1"
```

Default features include HTTP+protobuf transport (reqwest), traces, logs, and the tokio runtime. See [Feature Flags](#feature-flags) for customization.

### Using gRPC transport instead of HTTP

```toml
[dependencies]
raccoon-otel = { version = "1", default-features = false, features = ["grpc", "traces", "logs", "rt-tokio"] }
```

## Quick Start

### Minimal (zero-config)

Works immediately with an OTLP collector at `localhost:4318`:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = raccoon_otel::setup_otel("my-service", None)?;

    tracing::info!("Hello from raccoon-otel!");

    // _guard is dropped here -> flush + graceful shutdown
    Ok(())
}
```

### With configuration

```rust
use std::time::Duration;
use raccoon_otel::{OtelOptions, Protocol};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = raccoon_otel::setup_otel("my-service", Some(
        OtelOptions::builder()
            .endpoint("http://collector:4318")
            .protocol(Protocol::HttpProtobuf)
            .resource_attributes([
                ("deployment.environment", "production"),
                ("service.version", "1.2.3"),
            ])
            .headers([("Authorization", "Bearer token123")])
            .export_timeout(Duration::from_secs(30))
            .build()
    ))?;

    tracing::info!("Configured and exporting!");
    Ok(())
}
```

### Environment variable configuration

No code changes needed. Set standard OTel env vars and call `setup_otel` with no options:

```bash
export OTEL_SERVICE_NAME=my-service
export OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4318
export OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
export RUST_LOG=info,my_crate=debug
```

```rust
let _guard = raccoon_otel::setup_otel("fallback-name", None)?;
```

## Configuration

### Priority order

Configuration is resolved in order of precedence (highest first):

1. **Programmatic** -- values set via `OtelOptions::builder()`
2. **Environment variables** -- standard OTel env vars
3. **Defaults** -- localhost endpoints, 30s timeout, `info` log level

### Supported environment variables

| Variable                      | Description                                              | Default                                          |
|-------------------------------|----------------------------------------------------------|--------------------------------------------------|
| `OTEL_SERVICE_NAME`           | Service name for the resource                            | Value passed to `setup_otel()`                   |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Base OTLP endpoint                                       | `http://localhost:4318` (HTTP) or `:4317` (gRPC) |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | Transport protocol: `http/protobuf`, `http/json`, `grpc` | `http/protobuf`                                  |
| `OTEL_EXPORTER_OTLP_HEADERS`  | Comma-separated `key=value` pairs                        | (none)                                           |
| `OTEL_EXPORTER_OTLP_TIMEOUT`  | Export timeout in milliseconds                           | `30000`                                          |
| `RUST_LOG`                    | Log level filter directives                              | `info`                                           |

### Builder API

```rust
OtelOptions::builder()
    .endpoint("http://collector:4318")    // OTLP receiver URL
    .protocol(Protocol::HttpProtobuf)     // HttpProtobuf | HttpJson | Grpc
    .resource_attributes([                // Additional OTel resource attributes
        ("deployment.environment", "staging"),
    ])
    .headers([                            // Auth headers for OTLP requests
        ("Authorization", "Bearer token"),
    ])
    .export_timeout(Duration::from_secs(10))  // Per-request timeout
    .build()
```

All builder methods are optional. Unset values fall through to env vars, then defaults.

## The OtelGuard

`setup_otel()` returns an `OtelGuard` that owns all provider lifecycles. This is the most critical part of the API:

```rust
// CORRECT: guard lives for the whole program
let _guard = raccoon_otel::setup_otel("my-service", None)?;

// WRONG: guard is dropped immediately, providers shut down, nothing exports
raccoon_otel::setup_otel("my-service", None)?;
```

The `#[must_use]` attribute on `OtelGuard` produces a compiler warning if the return value is not bound to a variable.

When the guard is dropped (either by going out of scope or by calling `.shutdown()` explicitly):

1. All pending spans and logs are **flushed** to the backend
2. All providers are **shut down** gracefully
3. Shutdown errors are printed to stderr (never panics)

Calling `.shutdown()` multiple times is safe -- subsequent calls are no-ops.

## Feature Flags

### Transport (pick at least one)

| Feature | Description                           | Default |
|---------|---------------------------------------|---------|
| `http`  | HTTP+protobuf via reqwest (port 4318) | Yes     |
| `grpc`  | gRPC via tonic (port 4317)            | No      |

### Signals

| Feature   | Description                         | Default      |
|-----------|-------------------------------------|--------------|
| `traces`  | Export tracing spans as OTel traces | Yes          |
| `logs`    | Export tracing events as OTel logs  | Yes          |
| `metrics` | Export metrics via `MetricsLayer`   | No (planned) |

### Compression

| Feature | Description                       | Default |
|---------|-----------------------------------|---------|
| `gzip`  | gzip compression for gRPC exports | No      |
| `zstd`  | zstd compression for gRPC exports | No      |

### TLS

| Feature      | Description                                      | Default |
|--------------|--------------------------------------------------|---------|
| `tls`        | System-native TLS root certificates              | No      |
| `tls-webpki` | Embedded Mozilla CA roots (no system dependency) | No      |

### Async Runtime

| Feature                   | Description                  | Default |
|---------------------------|------------------------------|---------|
| `rt-tokio`                | tokio multi-threaded runtime | Yes     |
| `rt-tokio-current-thread` | tokio current-thread runtime | No      |

### Example: gRPC with TLS

```toml
[dependencies]
raccoon-otel = { version = "1", default-features = false, features = [
    "grpc", "traces", "logs", "rt-tokio", "tls"
] }
```

## Re-exports

`raccoon-otel` re-exports key crates so you don't need to add direct dependencies or worry about version alignment:

```rust
use raccoon_otel::re_exports::tracing;
use raccoon_otel::re_exports::opentelemetry;
use raccoon_otel::re_exports::opentelemetry_sdk;
use raccoon_otel::re_exports::tracing_opentelemetry;
```

This is particularly useful for accessing span context extensions or the `opentelemetry::global` module without managing separate dependency entries.

## What gets exported

### Traces

Every `tracing` span becomes an OTel span. This includes:

- Spans created with `#[tracing::instrument]`
- Spans created with `tracing::info_span!()`, `tracing::debug_span!()`, etc.
- Spans from libraries that use `tracing` (e.g., `hyper`, `tower`, `axum`, `sqlx`, `sea-orm`, `reqwest`)

### Logs

Every `tracing` event becomes an OTel log record. This includes:

- Events from `tracing::info!()`, `tracing::error!()`, `tracing::debug!()`, etc.
- Events are automatically correlated with their parent span for log-trace correlation

### Console output

`raccoon-otel` always adds a `fmt` layer to the subscriber, so all events also print to stdout with the standard `tracing_subscriber::fmt` format. You get both local console output and remote OTel export simultaneously.

### Log level filtering

The `EnvFilter` layer respects the `RUST_LOG` environment variable. Default level is `info`.

```bash
# Show debug logs from your crate, info from everything else
RUST_LOG=info,my_crate=debug cargo run

# Show all trace-level logs
RUST_LOG=trace cargo run
```

## Database instrumentation

Unlike Python and C#, Rust's database ecosystem already uses `tracing` natively. When `raccoon-otel` is initialized, database spans are exported automatically with **zero additional configuration**:

- **sqlx** -- emits tracing spans for all queries
- **sea-orm** -- built on sqlx, inherits its tracing spans
- **diesel** -- has optional tracing support via `diesel-tracing`

No `with_database()` feature or call is needed (like we have on the C# and Python packages).

## Distributed tracing

`raccoon-otel` automatically configures the W3C TraceContext propagator. This means trace context is propagated across
service boundaries when using HTTP clients and servers that support `opentelemetry::global::get_text_map_propagator()`.

Libraries like `reqwest-tracing` and `tower-http` can inject and extract the `traceparent` header automatically.

## Architecture

```
Your Application
    │
    │  tracing::info!(), #[instrument], spans
    │
    ▼
┌─────────────────────────────────────┐
│  raccoon-otel                       │
│                                     │
│  ┌──────────┐  ┌─────────────────┐  │
│  │ EnvFilter│  │   fmt (stdout)  │  │
│  └──────────┘  └─────────────────┘  │
│  ┌───────────────────┐              │
│  │ OpenTelemetryLayer│──► TracerProvider ──► OTLP SpanExporter
│  │  (traces)         │              │
│  └───────────────────┘              │
│  ┌───────────────────┐              │
│  │ TracingBridge     │──► LoggerProvider ──► OTLP LogExporter
│  │  (logs)           │              │
│  └───────────────────┘              │
│                                     │
│  OtelGuard (flush + shutdown)       │
└─────────────────────────────────────┘
    │
    ▼
  OTLP Collector / Backend
  (Jaeger, Grafana, Datadog, etc.)
```

## Version alignment

One of the main reasons this crate exists is to shield users from the notoriously fragile version coupling in the Rust OTel ecosystem. For reference, this crate internally aligns:

| Internal dependency              | Version                          |
|----------------------------------|----------------------------------|
| `opentelemetry`                  | 0.31                             |
| `opentelemetry_sdk`              | 0.31                             |
| `opentelemetry-otlp`             | 0.31                             |
| `tracing-opentelemetry`          | 0.32 (off-by-one is intentional) |
| `opentelemetry-appender-tracing` | 0.31                             |
| `tracing`                        | 0.1                              |
| `tracing-subscriber`             | 0.3                              |

You should not need to depend on any of these directly. If you do need access to their types, use the [`re_exports`](#re-exports) module.

## Limitations

- **Single initialization only.** `setup_otel()` sets the global tracing subscriber. Calling it twice will return an error. This is a limitation of `tracing`'s global subscriber model.

- **Metrics not yet implemented.** The `metrics` feature flag is defined and wires up the correct dependencies, but the `MetricsLayer` integration is not yet built. This is planned for a future release. (Maybe)

- **Programmatic headers not yet passed to exporters.** Headers set via `OtelOptions::builder().headers(...)` are parsed and resolved, but not yet forwarded to the tonic/reqwest exporters. Headers set via the `OTEL_EXPORTER_OTLP_HEADERS` environment variable work natively (the OTLP SDK reads them directly).

- **No custom sampler configuration.** The default sampler (always-on) is used. Custom sampler support (`OTEL_TRACES_SAMPLER` / `OTEL_TRACES_SAMPLER_ARG`) is planned for a future release. (Also maybe)

- **Requires a tokio runtime.** The batch exporters use tokio for async processing. The `rt-tokio` (default) or `rt-tokio-current-thread` feature must be enabled.

- **Pre-1.0 upstream dependencies.** The `opentelemetry` Rust crates are pre-1.0, meaning minor version bumps can contain breaking changes. `raccoon-otel` pins to `0.31` to shield users, but upstream API changes may require a new major version of this crate.

## Advantages

- **One function, one dependency.** Replace 6-8 dependencies and 60-80 lines of boilerplate with a single `setup_otel()` call.

- **Zero code changes.** Existing `tracing` instrumentation (`#[instrument]`, `info!()`, spans) exports to OTel automatically. No macro changes, no API migrations.

- **Safe lifecycle management.** The `#[must_use]` `OtelGuard` prevents the silent-drop footgun that is the #1 source of "my spans aren't showing up" issues.

- **Version shield.** You don't need to track which `tracing-opentelemetry` version works with which `opentelemetry` version. This crate manages it for you.

- **Dual output.** Events go to both stdout (for local development) and the OTel backend (for production). No need to choose.

- **Environment-variable-driven.** Deployments can configure endpoints, protocols, and headers entirely via env vars without code changes.

- **Feature-gated.** Only compile what you use. HTTP-only? gRPC-only? No TLS? Feature flags keep the dependency tree minimal.

- **W3C context propagation.** Distributed tracing across services works out of the box.

## Roadmap

- **Metrics**: `MetricsLayer` integration for exporting `monotonic_counter.*` and `histogram.*` events as OTel metrics
- **Auto-instrumentation**: `with_axum()` for tower-http TraceLayer, `with_reqwest()` for reqwest-tracing middleware
- **Compression**: HTTP transport compression (gzip, zstd)
- **Custom samplers**: Support for `OTEL_TRACES_SAMPLER` and programmatic sampler configuration
- **Programmatic headers**: Forward `OtelOptions` headers to tonic metadata / reqwest headers

## License

MIT