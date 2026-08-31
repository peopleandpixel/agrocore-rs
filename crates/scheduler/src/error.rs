use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Cron parsing error: {0}")]
    Cron(#[from] cron::error::Error),

    #[error("Job scheduler error: {0}")]
    JobScheduler(#[from] tokio_cron_scheduler::JobSchedulerError),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

pub type SchedulerResult<T> = Result<T, SchedulerError>;
