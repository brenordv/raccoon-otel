# v1.1.0

## Added
- `setup_otel_with_layers(service_name, options, extra_layers)`: the same setup as `setup_otel`, plus any caller-supplied `tracing` layers installed into the global subscriber. Use it to keep local application logging (for example a rotating file writer) while still exporting to OpenTelemetry.
- `BoxedLayer`: public type alias for a boxed `tracing_subscriber::Layer` over the global `Registry`. This is the element type of the `extra_layers` argument.
- `re_exports::tracing_subscriber`: re-export of `tracing-subscriber`, so callers can build `BoxedLayer` values at the aligned version without adding the crate to their own `Cargo.toml`.

## Changed
- The `RUST_LOG` env filter is now applied across the whole composed layer set, so any extra layers share the same level filtering as the stdout and OTel layers.
- Bumped dependencies to their latest versions.

`setup_otel` keeps the same signature and behavior; it now delegates to `setup_otel_with_layers` with no extra layers, so existing code is unaffected.

# v1.0.0
Initial release