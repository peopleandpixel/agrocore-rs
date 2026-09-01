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

#[cfg(feature = "dev-console")]
pub use tracing::{debug, error, info, warn};

#[cfg(feature = "otlp")]
pub use tracing::{debug, error, info, warn};
