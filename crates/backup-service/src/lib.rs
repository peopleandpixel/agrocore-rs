//! agrocore-backup library crate

pub mod config;
pub mod encryption;
pub mod error;
pub mod manifest;
pub mod metrics;
pub mod nats_client;
pub mod pg_dump;
pub mod retention;
pub mod service;
pub mod sftp_backend;
pub mod storage;
pub mod verification;
pub mod webdav_backend;

pub use crate::config::{
    BackupConfig, BackupMetadataConfig, BackupTarget, ConfigPathsConfig, EncryptionConfig,
    EncryptionMethod, PgDumpConfig, RetentionConfig, S3Credentials, TargetEncryption,
    VerificationConfig,
};
pub use crate::error::{BackupError, BackupResult};
pub use crate::service::{BackupJob, BackupService, BackupStatus, BackupSummary, BackupType};
