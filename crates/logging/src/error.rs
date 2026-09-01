use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("OpenTelemetry error: {0}")]
    OpenTelemetry(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type LoggingResult<T> = Result<T, LoggingError>;
