use crate::config::LoggingConfig;
use crate::error::{LoggingError, LoggingResult};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;

#[cfg(feature = "otlp")]
use opentelemetry::KeyValue;
#[cfg(feature = "otlp")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "otlp")]
use opentelemetry_sdk::trace::TracerProvider;
#[cfg(feature = "otlp")]
use tracing_opentelemetry::OpenTelemetryLayer;

/// Guard for file writer - must be kept alive
pub type FileGuard = Option<()>;

/// Result of logging initialization
pub struct LoggingHandle {
    pub guard: FileGuard,
    #[cfg(feature = "otlp")]
    pub otlp_shutdown: Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl LoggingHandle {
    pub fn new(guard: FileGuard) -> Self {
        Self {
            guard,
            #[cfg(feature = "otlp")]
            otlp_shutdown: None,
        }
    }

    #[cfg(feature = "otlp")]
    pub fn with_otlp_shutdown(mut self, shutdown: Box<dyn FnOnce() + Send + Sync>) -> Self {
        self.otlp_shutdown = Some(shutdown);
        self
    }

    /// Call on application shutdown
    pub fn shutdown(self) {
        #[cfg(feature = "otlp")]
        if let Some(shutdown) = self.otlp_shutdown {
            shutdown();
        }
    }
}

/// Initialize logging based on config
pub fn init_logging(config: LoggingConfig) -> LoggingResult<LoggingHandle> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| config.level.clone().into());

    let subscriber = tracing_subscriber::registry().with(env_filter);

    // 1. Console layer (dev)
    if config.console_enabled {
        let console_layer = build_console_layer(&config);
        let subscriber = subscriber.with(console_layer);

        // 2. File layer - not implemented for simplicity

        // 3. OTLP layer (distributed tracing)
        #[cfg(feature = "otlp")]
        if config.otlp_enabled {
            let otlp_layer = build_otlp_layer(&config)?;
            let subscriber = subscriber.with(otlp_layer);

            let _ = tracing::subscriber::set_global_default(subscriber)
                .map_err(|e| LoggingError::InvalidConfig(e.to_string()))?;

            let handle = LoggingHandle::new(None);
            return Ok(handle);
        }

        let _ = tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| LoggingError::InvalidConfig(e.to_string()))?;

        let handle = LoggingHandle::new(None);
        return Ok(handle);
    }

    // No console, just try to set minimal logging
    let _ = tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| LoggingError::InvalidConfig(e.to_string()))?;

    let handle = LoggingHandle::new(None);
    Ok(handle)
}

/// Console layer builder - uses conditional compilation to handle pretty
fn build_console_layer<S>(config: &LoggingConfig) -> impl Layer<S> + Send + Sync
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    if config.console_pretty {
        let builder = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
            .pretty();

        let builder = if config.console_thread_ids {
            builder.with_thread_ids(true)
        } else {
            builder
        };

        let builder = if config.console_thread_names {
            builder.with_thread_names(true)
        } else {
            builder
        };

        builder.boxed()
    } else {
        let builder = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

        let builder = if config.console_thread_ids {
            builder.with_thread_ids(true)
        } else {
            builder
        };

        let builder = if config.console_thread_names {
            builder.with_thread_names(true)
        } else {
            builder
        };

        builder.boxed()
    }
}

/// OTLP layer builder
#[cfg(feature = "otlp")]
fn build_otlp_layer(config: &LoggingConfig) -> LoggingResult<impl Layer<Registry> + Send + Sync>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    let endpoint = config.otlp_endpoint.clone();
    let service_name = config.otlp_service_name.clone();

    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint)
                .with_timeout(std::time::Duration::from_millis(
                    config.otlp_batch_timeout_ms,
                ))
                .with_max_export_batch_size(config.otlp_max_export_batch_size),
        )
        .with_resource(opentelemetry_sdk::Resource::new(vec![
            KeyValue::new("service.name", service_name),
            KeyValue::new(
                "deployment.environment",
                format!("{:?}", config.environment),
            ),
        ]))
        .build();

    let tracer = tracer_provider.tracer("agrocore");
    let otel_layer = OpenTelemetryLayer::new(tracer);

    Ok(otel_layer)
}
