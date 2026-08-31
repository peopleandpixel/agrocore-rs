use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackupError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database connection error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("PostgreSQL error: {0}")]
    Postgres(String),

    #[error("Object store error: {0}")]
    ObjectStore(#[from] object_store::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("Scheduling error: {0}")]
    Scheduling(String),

    #[error("Retention error: {0}")]
    Retention(String),

    #[error("Verification error: {0}")]
    Verification(String),

    #[error("Manifest error: {0}")]
    Manifest(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Age encryption error: {0}")]
    Age(#[from] age::EncryptError),

    #[error("Age decryption error: {0}")]
    AgeDecrypt(#[from] age::DecryptError),

    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),

    #[error("Tempfile error: {0}")]
    Tempfile(#[from] tempfile::PersistError),

    #[error("Chrono parse error: {0}")]
    Chrono(#[from] chrono::ParseError),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Job scheduler error: {0}")]
    JobScheduler(String),
}

pub type BackupResult<T> = Result<T, BackupError>;

impl BackupError {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn encryption(msg: impl Into<String>) -> Self {
        Self::Encryption(msg.into())
    }

    pub fn scheduling(msg: impl Into<String>) -> Self {
        Self::Scheduling(msg.into())
    }

    pub fn retention(msg: impl Into<String>) -> Self {
        Self::Retention(msg.into())
    }

    pub fn verification(msg: impl Into<String>) -> Self {
        Self::Verification(msg.into())
    }

    pub fn manifest(msg: impl Into<String>) -> Self {
        Self::Manifest(msg.into())
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState(msg.into())
    }

    pub fn job_scheduler(msg: impl Into<String>) -> Self {
        Self::JobScheduler(msg.into())
    }

    pub fn postgres(msg: impl Into<String>) -> Self {
        Self::Postgres(msg.into())
    }
}
