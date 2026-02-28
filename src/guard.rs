use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;

/// Lifecycle guard for OpenTelemetry providers.
///
/// Holds all active OTel providers and ensures they are flushed and shut down
/// gracefully when dropped. **Must be held for the duration of the application.**
///
/// Dropping the guard:
/// 1. Flushes all pending spans, logs, and metrics
/// 2. Shuts down all providers gracefully
///
/// # Examples
///
/// ```no_run
/// // Hold the guard in main — dropping it triggers shutdown
/// let _guard = raccoon_otel::setup_otel("my-service", None).unwrap();
/// // ... application runs ...
/// // guard dropped here → flush + shutdown
/// ```
#[must_use = "dropping the OtelGuard immediately shuts down all OTel providers — \
              hold it for the lifetime of your application (e.g. `let _guard = ...;`)"]
pub struct OtelGuard {
    tracer_provider: Option<SdkTracerProvider>,
    logger_provider: Option<SdkLoggerProvider>,
    shutdown_called: bool,
}

impl OtelGuard {
    pub(crate) fn new(
        tracer_provider: Option<SdkTracerProvider>,
        logger_provider: Option<SdkLoggerProvider>,
    ) -> Self {
        Self {
            tracer_provider,
            logger_provider,
            shutdown_called: false,
        }
    }

    /// Explicitly flush and shut down all providers.
    ///
    /// Safe to call multiple times; subsequent calls are no-ops.
    /// This is also called automatically when the guard is dropped.
    pub fn shutdown(&mut self) {
        if self.shutdown_called {
            return;
        }
        self.shutdown_called = true;
        self.do_shutdown();
    }

    fn do_shutdown(&self) {
        if let Some(ref tp) = self.tracer_provider {
            if let Err(e) = tp.force_flush() {
                eprintln!("raccoon-otel: error flushing tracer provider: {e}");
            }
            if let Err(e) = tp.shutdown() {
                eprintln!("raccoon-otel: error shutting down tracer provider: {e}");
            }
        }

        if let Some(ref lp) = self.logger_provider {
            if let Err(e) = lp.force_flush() {
                eprintln!("raccoon-otel: error flushing logger provider: {e}");
            }
            if let Err(e) = lp.shutdown() {
                eprintln!("raccoon-otel: error shutting down logger provider: {e}");
            }
        }
    }
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if !self.shutdown_called {
            self.shutdown_called = true;
            self.do_shutdown();
        }
    }
}
