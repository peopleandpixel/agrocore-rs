// `LoggingConfig` and `LoggingResult` are used by every configuration; the rest are
// only reached when a tracing feature is on. The admin UI depends on this crate
// with `default-features = false` for its WASM build, and an unconditional import
// fails `-D warnings` there even though the file is never executed.
use crate::config::{LoggingConfig, RotationType};
#[cfg(any(feature = "dev-console", feature = "otlp"))]
use crate::error::LoggingError;
use crate::error::LoggingResult;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Type alias for per-crate writer map to reduce type complexity
#[cfg(feature = "dev-console")]
type CrateWriterMap = Arc<RwLock<HashMap<String, NonBlocking>>>;

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

/// Per-crate file writer that routes to the appropriate crate's log file
#[cfg(feature = "dev-console")]
struct CrateFileWriter {
    writers: CrateWriterMap,
}

#[cfg(feature = "dev-console")]
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CrateFileWriter {
    type Writer = Box<dyn Write + Send + Sync>;

    fn make_writer(&'a self) -> Self::Writer {
        Box::new(CrateLogWriter {
            writers: self.writers.clone(),
        })
    }
}

/// Writer that routes log lines to the correct crate's file based on the target
#[cfg(feature = "dev-console")]
struct CrateLogWriter {
    writers: CrateWriterMap,
}

#[cfg(feature = "dev-console")]
impl Write for CrateLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Extract crate name from target (e.g., "agrocore_api::handlers::tasks" -> "agrocore_api")
        let crate_name = match std::str::from_utf8(buf) {
            Ok(s) => extract_crate_name(s),
            Err(_) => None,
        };

        // Try to find the specific crate writer
        if let Some(crate_name) = crate_name {
            let writers = match self.writers.read() {
                Ok(w) => w,
                Err(_) => return Ok(buf.len()),
            };
            if let Some(writer) = writers.get(&crate_name) {
                let mut writer = writer.clone();
                return writer.write(buf);
            }
        }

        // Fallback: write to first available writer
        let writers = match self.writers.read() {
            Ok(w) => w,
            Err(_) => return Ok(buf.len()),
        };
        if let Some(writer) = writers.values().next() {
            let mut writer = writer.clone();
            return writer.write(buf);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let writers = match self.writers.read() {
            Ok(w) => w,
            Err(_) => return Ok(()),
        };
        if let Some(writer) = writers.values().next() {
            let mut writer = writer.clone();
            writer.flush()?;
        }
        Ok(())
    }
}

#[cfg(feature = "dev-console")]
fn extract_crate_name(line: &str) -> Option<String> {
    // Try to extract crate name from log line
    // Format typically: "TIMESTAMP LEVEL crate_name::module::function message"
    let start = line.find(' ')?;
    let level_end = line[start + 1..].find(' ')?;
    let target_start = start + 1 + level_end + 1;
    if target_start < line.len() {
        let target = &line[target_start..];
        target
            .find("::")
            .map(|crate_end| target[..crate_end].to_string())
    } else {
        None
    }
}

#[cfg(feature = "dev-console")]
pub type FileGuard = Option<Vec<WorkerGuard>>;

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
    // No tracing features enabled - just set up basic log crate.
    // The explicit `return` was needed while every other branch was also compiled;
    // with neither feature on this is the only block left, and clippy's
    // `needless_return` fires under `-D warnings` in the WASM build.
    #[cfg(not(any(feature = "dev-console", feature = "otlp")))]
    {
        log::set_max_level(config.level.parse().unwrap_or(log::LevelFilter::Info));
        Ok(LoggingHandle::new(None))
    }

    #[cfg(all(feature = "dev-console", not(feature = "otlp")))]
    {
        // Only dev-console enabled - use registry with all layers in single chain
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| config.level.clone().into());

        tracing_log::LogTracer::init().ok();

        // Build per-crate file layers
        let (crate_guards, crate_writers) = build_crate_file_layers(&config)?;

        // Build main file layer (for backwards compatibility / general logs)
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

        // Build per-crate file writer that routes to appropriate file
        let crate_writer = CrateFileWriter {
            writers: crate_writers,
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
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(crate_writer)
                    .with_ansi(false)
                    .json(),
            );

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| LoggingError::InvalidConfig(e.to_string()))?;

        // Combine guards
        let mut guards = Vec::new();
        if let Some(g) = file_guard {
            guards.push(g);
        }
        guards.extend(crate_guards);

        let handle = LoggingHandle::new(Some(guards));
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

        // Build per-crate file layers
        let (crate_guards, crate_writers) = build_crate_file_layers(&config)?;

        // Build main file layer
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

        // Build per-crate file writer that routes to appropriate file
        let crate_writer = CrateFileWriter {
            writers: crate_writers,
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
            let subscriber = subscriber.with(
                tracing_subscriber::fmt::layer()
                    .with_writer(crate_writer)
                    .with_ansi(false)
                    .json(),
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

        // Combine guards
        let mut guards = Vec::new();
        if let Some(g) = file_guard {
            guards.push(g);
        }
        guards.extend(crate_guards);

        let handle = LoggingHandle::new(Some(guards));
        Ok(handle)
    }
}

/// Build per-crate file layers from config
#[cfg(feature = "dev-console")]
fn build_crate_file_layers(
    config: &LoggingConfig,
) -> LoggingResult<(Vec<WorkerGuard>, CrateWriterMap)> {
    let mut guards = Vec::new();
    let mut writers = HashMap::new();

    let log_dir = config
        .log_dir
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new("logs"));

    // Ensure log directory exists
    fs::create_dir_all(log_dir).map_err(LoggingError::Io)?;

    for (crate_name, crate_config) in &config.crate_logs {
        if !crate_config.enabled {
            continue;
        }

        let file_name = crate_config
            .file_name
            .clone()
            .unwrap_or_else(|| format!("{}.jsonl", crate_name));
        let log_path = log_dir.join(&file_name);

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
            log_path.file_name().unwrap_or_else(|| file_name.as_ref()),
        );

        let (non_blocking, guard) = non_blocking(file_appender);
        guards.push(guard);
        writers.insert(crate_name.clone(), non_blocking);
    }

    Ok((guards, Arc::new(RwLock::new(writers))))
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
