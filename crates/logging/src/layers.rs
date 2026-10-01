use crate::config::{LoggingConfig, RotationType};
use crate::error::{LoggingError, LoggingResult};

#[cfg(any(feature = "dev-console", feature = "otlp"))]
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt as SubscriberExt;

#[cfg(feature = "otlp")]
use opentelemetry::KeyValue;
#[cfg(feature = "otlp")]
use opentelemetry::trace::TracerProvider as _;
#[cfg(feature = "otlp")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "otlp")]
use tracing_opentelemetry::OpenTelemetryLayer;

#[cfg(any(feature = "dev-console", feature = "otlp"))]
use std::fs;
#[cfg(any(feature = "dev-console", feature = "otlp"))]
use std::io::{self, Write};

#[cfg(feature = "dev-console")]
use tracing_appender::rolling::{RollingFileAppender, Rotation};
#[cfg(feature = "dev-console")]
use tracing_appender::{
    non_blocking,
    non_blocking::{NonBlocking, WorkerGuard},
};

/// A writer that discards all data (like /dev/null) - always available
#[cfg(any(feature = "dev-console", feature = "otlp"))]
struct NoOpWriter;

#[cfg(any(feature = "dev-console", feature = "otlp"))]
impl Write for NoOpWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(any(feature = "dev-console", feature = "otlp"))]
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for NoOpWriter {
    type Writer = NoOpWriter;

    fn make_writer(&'a self) -> Self::Writer {
        NoOpWriter
    }
}

/// Unified file writer that works whether file logging is enabled or not
#[cfg(feature = "dev-console")]
enum FileWriter {
    Enabled(NonBlocking),
    Disabled,
}

#[cfg(feature = "dev-console")]
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for FileWriter {
    type Writer = Box<dyn Write + Send + Sync>;

    fn make_writer(&'a self) -> Self::Writer {
        match self {
            FileWriter::Enabled(nb) => Box::new(nb.make_writer()),
            FileWriter::Disabled => Box::new(NoOpWriter),
        }
    }
}

/// Unified console writer that works whether console logging is enabled or not
#[cfg(feature = "dev-console")]
enum ConsoleWriter {
    Enabled,
    Disabled,
}

#[cfg(feature = "dev-console")]
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for ConsoleWriter {
    type Writer = Box<dyn Write + Send + Sync>;

    fn make_writer(&'a self) -> Self::Writer {
        match self {
            ConsoleWriter::Enabled => Box::new(std::io::stdout()),
            ConsoleWriter::Disabled => Box::new(NoOpWriter),
        }
    }
}

#[cfg(feature = "dev-console")]
pub type FileGuard = Option<WorkerGuard>;

#[cfg(not(feature = "dev-console"))]
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
        // Drop the guard to flush file buffers
        #[cfg(feature = "dev-console")]
        if let Some(guard) = self.guard {
            drop(guard);
        }
    }
}

/// Initialize logging based on config
pub fn init_logging(config: LoggingConfig) -> LoggingResult<LoggingHandle> {
    #[cfg(not(any(feature = "dev-console", feature = "otlp")))]
    {
        // No tracing features enabled - just set up basic log crate
        log::set_max_level(config.level.parse().unwrap_or(log::LevelFilter::Info));
        return Ok(LoggingHandle::new(None));
    }

    #[cfg(all(feature = "dev-console", not(feature = "otlp")))]
    {
        // Only dev-console enabled - use registry with all layers in single chain
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| config.level.clone().into());

        tracing_log::LogTracer::init().ok();

        // Build file layer
        let (file_guard, file_writer) = if config.file_enabled {
            let log_path = if let Some(log_dir) = &config.log_dir {
                log_dir.join(
                    config
                        .file_path
                        .file_name()
                        .unwrap_or_else(|| "agrocore.json".as_ref()),
                )
            } else {
                config.file_path.clone()
            };

            // Ensure log directory exists
            if let Some(parent) = log_path.parent() {
                fs::create_dir_all(parent).map_err(LoggingError::Io)?;
            }

            let rotation = match config.file_rotation.rotation {
                RotationType::Daily => Rotation::DAILY,
                RotationType::Hourly => Rotation::HOURLY,
                RotationType::Never => Rotation::NEVER,
                RotationType::Size => Rotation::DAILY,
            };

            let file_appender = RollingFileAppender::new(
                rotation,
                log_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new(".")),
                log_path
                    .file_name()
                    .unwrap_or_else(|| "agrocore.json".as_ref()),
            );

            let (non_blocking, guard) = non_blocking(file_appender);

            (Some(guard), FileWriter::Enabled(non_blocking))
        } else {
            (None, FileWriter::Disabled)
        };

        // Build console layer
        let console_writer = if config.console_enabled {
            ConsoleWriter::Enabled
        } else {
            ConsoleWriter::Disabled
        };

        // Build subscriber using registry - chain all in single expression
        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file_writer)
                    .with_ansi(false)
                    .json(),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(console_writer)
                    .with_ansi(false)
                    .pretty(),
            );

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| LoggingError::InvalidConfig(e.to_string()))?;

        let handle = LoggingHandle::new(file_guard);
        Ok(handle)
    }

    #[cfg(all(not(feature = "dev-console"), feature = "otlp"))]
    {
        // Only otlp enabled - use registry pattern
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| config.level.clone().into());

        tracing_log::LogTracer::init().ok();

        let otlp_layer = if config.otlp_enabled {
            Some(build_otlp_layer(&config)?)
        } else {
            None
        };

        let subscriber = {
            let base = tracing_subscriber::registry().with(env_filter);

            let subscriber = if let Some(layer) = otlp_layer {
                base.with(layer)
            } else {
                base.with(tracing_subscriber::fmt::layer().with_writer(NoOpWriter))
            };

            subscriber
        };

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| LoggingError::InvalidConfig(e.to_string()))?;

        let handle = LoggingHandle::new(None);
        Ok(handle)
    }

    #[cfg(all(feature = "dev-console", feature = "otlp"))]
    {
        // Both features enabled - use registry with all layers in single chain
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| config.level.clone().into());

        tracing_log::LogTracer::init().ok();

        // Build file layer
        let (file_guard, file_writer) = if config.file_enabled {
            let log_path = if let Some(log_dir) = &config.log_dir {
                log_dir.join(
                    config
                        .file_path
                        .file_name()
                        .unwrap_or_else(|| "agrocore.json".as_ref()),
                )
            } else {
                config.file_path.clone()
            };

            if let Some(parent) = log_path.parent() {
                fs::create_dir_all(parent).map_err(LoggingError::Io)?;
            }

            let rotation = match config.file_rotation.rotation {
                RotationType::Daily => Rotation::DAILY,
                RotationType::Hourly => Rotation::HOURLY,
                RotationType::Never => Rotation::NEVER,
                RotationType::Size => Rotation::DAILY,
            };

            let file_appender = RollingFileAppender::new(
                rotation,
                log_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new(".")),
                log_path
                    .file_name()
                    .unwrap_or_else(|| "agrocore.json".as_ref()),
            );

            let (non_blocking, guard) = non_blocking(file_appender);

            (Some(guard), FileWriter::Enabled(non_blocking))
        } else {
            (None, FileWriter::Disabled)
        };

        // Build console layer
        let console_writer = if config.console_enabled {
            ConsoleWriter::Enabled
        } else {
            ConsoleWriter::Disabled
        };

        // Build OTLP layer
        let otlp_layer = if config.otlp_enabled {
            Some(build_otlp_layer(&config)?)
        } else {
            None
        };

        // Build subscriber - chain all in single expression
        let subscriber = {
            let base = tracing_subscriber::registry().with(env_filter);
            let subscriber = base.with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file_writer)
                    .with_ansi(false)
                    .json(),
            );
            let subscriber = subscriber.with(
                tracing_subscriber::fmt::layer()
                    .with_writer(console_writer)
                    .with_ansi(false)
                    .pretty(),
            );

            let subscriber = if let Some(layer) = otlp_layer {
                subscriber.with(layer)
            } else {
                subscriber.with(tracing_subscriber::fmt::layer().with_writer(NoOpWriter))
            };

            subscriber
        };

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| LoggingError::InvalidConfig(e.to_string()))?;

        let handle = LoggingHandle::new(file_guard);
        Ok(handle)
    }
}

/// OTLP layer builder - returns concrete layer type
#[cfg(feature = "otlp")]
fn build_otlp_layer(config: &LoggingConfig) -> LoggingResult<impl Layer<Registry> + Send + Sync> {
    let endpoint = config.otlp_endpoint.clone();
    let service_name = config.otlp_service_name.clone();

    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint)
                .with_timeout(std::time::Duration::from_millis(
                    config.otlp_batch_timeout_ms,
                )),
            opentelemetry_sdk::runtime::Tokio,
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
