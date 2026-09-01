pub mod config;
pub mod context;
pub mod error;
pub mod layers;

pub use config::{EnvironmentType, LoggingConfig, RotationConfig, RotationType};
pub use context::{
    RequestContext, ServiceContext, ServiceContextLayer, SpanExt, create_span, set_service_context,
};
pub use error::{LoggingError, LoggingResult};
pub use layers::{LoggingHandle, init_logging};

// Public API uses log crate (no tracing coupling for consumers)
pub use log::{Level, LevelFilter, Metadata, Record, debug, error, info, trace, warn};

// Our own span macros (no tracing dependency)
#[macro_export]
macro_rules! agrocore_span {
    ($name:expr) => {
        log::trace!(target: "agrocore_span", "{}", $name)
    };
    ($name:expr, $($field:tt)*) => {
        log::trace!(target: "agrocore_span", "{} {}", $name, format_args!($($field)*))
    };
}

#[macro_export]
macro_rules! agrocore_info {
    ($($arg:tt)*) => { log::info!($($arg)*) };
}

#[macro_export]
macro_rules! agrocore_error {
    ($($arg:tt)*) => { log::error!($($arg)*) };
}

#[macro_export]
macro_rules! agrocore_warn {
    ($($arg:tt)*) => { log::warn!($($arg)*) };
}

#[macro_export]
macro_rules! agrocore_debug {
    ($($arg:tt)*) => { log::debug!($($arg)*) };
}
