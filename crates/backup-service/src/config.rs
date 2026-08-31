use crate::error::{BackupError, BackupResult};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub schedule_db: String,
    pub schedule_config: String,
    pub timezone: String,
    pub targets: Vec<BackupTarget>,
    pub retention: RetentionConfig,
    pub verification: VerificationConfig,
    pub encryption: EncryptionConfig,
    pub pg_dump: PgDumpConfig,
    pub config_paths: ConfigPathsConfig,
    pub metadata: BackupMetadataConfig,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            schedule_db: "0 2 * * *".to_string(),
            schedule_config: "0 3 * * 0".to_string(),
            timezone: "UTC".to_string(),
            targets: vec![],
            retention: RetentionConfig::default(),
            verification: VerificationConfig::default(),
            encryption: EncryptionConfig::default(),
            pg_dump: PgDumpConfig::default(),
            config_paths: ConfigPathsConfig::default(),
            metadata: BackupMetadataConfig::default(),
        }
    }
}

impl BackupConfig {
    pub fn validate(&self) -> BackupResult<()> {
        if self.enabled && self.targets.is_empty() {
            return Err(BackupError::Config(
                "At least one backup target must be configured when backup is enabled".to_string(),
            ));
        }

        for target in &self.targets {
            target.validate()?;
        }

        cron::Schedule::from_str(&self.schedule_db).map_err(|e| {
            BackupError::Config(format!("Invalid schedule_db cron expression: {e}"))
        })?;
        cron::Schedule::from_str(&self.schedule_config).map_err(|e| {
            BackupError::Config(format!("Invalid schedule_config cron expression: {e}"))
        })?;

        self.retention.validate()?;
        self.verification.validate()?;
        self.encryption.validate()?;
        self.pg_dump.validate()?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BackupTarget {
    S3 {
        bucket: String,
        region: String,
        prefix: String,
        endpoint: Option<String>,
        credentials: Option<S3Credentials>,
        encryption: TargetEncryption,
        kms_key_id: Option<String>,
        storage_class: Option<String>,
        max_concurrent_uploads: Option<usize>,
        part_size: Option<usize>,
    },
    MinIO {
        bucket: String,
        endpoint: String,
        region: String,
        prefix: String,
        credentials: Option<S3Credentials>,
        encryption: TargetEncryption,
        storage_class: Option<String>,
        max_concurrent_uploads: Option<usize>,
        part_size: Option<usize>,
    },
    B2 {
        bucket: String,
        endpoint: String,
        region: String,
        prefix: String,
        credentials: Option<S3Credentials>,
        encryption: TargetEncryption,
        storage_class: Option<String>,
        max_concurrent_uploads: Option<usize>,
        part_size: Option<usize>,
    },
    Wasabi {
        bucket: String,
        region: String,
        prefix: String,
        credentials: Option<S3Credentials>,
        encryption: TargetEncryption,
        storage_class: Option<String>,
        max_concurrent_uploads: Option<usize>,
        part_size: Option<usize>,
    },
    Local {
        path: PathBuf,
        encryption: TargetEncryption,
        permissions: Option<u32>,
    },
    Azure {
        account: String,
        container: String,
        prefix: String,
        encryption: TargetEncryption,
        credentials: Option<AzureCredentials>,
    },
    GCS {
        bucket: String,
        prefix: String,
        encryption: TargetEncryption,
        credentials_path: Option<PathBuf>,
    },
    SFTP {
        host: String,
        port: u16,
        username: String,
        path: String,
        encryption: TargetEncryption,
        key_path: Option<PathBuf>,
        password: Option<String>,
    },
    WebDAV {
        url: String,
        username: String,
        password: String,
        prefix: String,
        encryption: TargetEncryption,
        chunk_size: Option<usize>,
    },
}

impl BackupTarget {
    pub fn validate(&self) -> BackupResult<()> {
        match self {
            Self::S3 {
                bucket,
                region,
                prefix,
                ..
            } => {
                if bucket.is_empty() {
                    return Err(BackupError::Config("S3 bucket cannot be empty".to_string()));
                }
                if region.is_empty() {
                    return Err(BackupError::Config("S3 region cannot be empty".to_string()));
                }
                if !prefix.is_empty() && !prefix.ends_with('/') {
                    return Err(BackupError::Config(
                        "S3 prefix must end with '/'".to_string(),
                    ));
                }
            }
            Self::MinIO {
                bucket,
                endpoint,
                region,
                prefix,
                ..
            } => {
                if bucket.is_empty() {
                    return Err(BackupError::Config(
                        "MinIO bucket cannot be empty".to_string(),
                    ));
                }
                if endpoint.is_empty() {
                    return Err(BackupError::Config(
                        "MinIO endpoint cannot be empty".to_string(),
                    ));
                }
                if region.is_empty() {
                    return Err(BackupError::Config(
                        "MinIO region cannot be empty".to_string(),
                    ));
                }
                if !prefix.is_empty() && !prefix.ends_with('/') {
                    return Err(BackupError::Config(
                        "MinIO prefix must end with '/'".to_string(),
                    ));
                }
            }
            Self::B2 {
                bucket,
                endpoint,
                region,
                prefix,
                ..
            } => {
                if bucket.is_empty() {
                    return Err(BackupError::Config("B2 bucket cannot be empty".to_string()));
                }
                if endpoint.is_empty() {
                    return Err(BackupError::Config(
                        "B2 endpoint cannot be empty".to_string(),
                    ));
                }
                if region.is_empty() {
                    return Err(BackupError::Config("B2 region cannot be empty".to_string()));
                }
                if !prefix.is_empty() && !prefix.ends_with('/') {
                    return Err(BackupError::Config(
                        "B2 prefix must end with '/'".to_string(),
                    ));
                }
            }
            Self::Wasabi {
                bucket,
                region,
                prefix,
                ..
            } => {
                if bucket.is_empty() {
                    return Err(BackupError::Config(
                        "Wasabi bucket cannot be empty".to_string(),
                    ));
                }
                if region.is_empty() {
                    return Err(BackupError::Config(
                        "Wasabi region cannot be empty".to_string(),
                    ));
                }
                if !prefix.is_empty() && !prefix.ends_with('/') {
                    return Err(BackupError::Config(
                        "Wasabi prefix must end with '/'".to_string(),
                    ));
                }
            }
            Self::Local { path, .. } => {
                if path.as_os_str().is_empty() {
                    return Err(BackupError::Config(
                        "Local path cannot be empty".to_string(),
                    ));
                }
            }
            Self::Azure {
                account, container, ..
            } => {
                if account.is_empty() {
                    return Err(BackupError::Config(
                        "Azure account cannot be empty".to_string(),
                    ));
                }
                if container.is_empty() {
                    return Err(BackupError::Config(
                        "Azure container cannot be empty".to_string(),
                    ));
                }
            }
            Self::GCS { bucket, .. } => {
                if bucket.is_empty() {
                    return Err(BackupError::Config(
                        "GCS bucket cannot be empty".to_string(),
                    ));
                }
            }
            Self::SFTP {
                host,
                port,
                username,
                ..
            } => {
                if host.is_empty() {
                    return Err(BackupError::Config("SFTP host cannot be empty".to_string()));
                }
                if *port == 0 {
                    return Err(BackupError::Config("SFTP port cannot be 0".to_string()));
                }
                if username.is_empty() {
                    return Err(BackupError::Config(
                        "SFTP username cannot be empty".to_string(),
                    ));
                }
            }
            Self::WebDAV {
                url,
                username,
                password,
                ..
            } => {
                if url.is_empty() {
                    return Err(BackupError::Config(
                        "WebDAV URL cannot be empty".to_string(),
                    ));
                }
                if username.is_empty() {
                    return Err(BackupError::Config(
                        "WebDAV username cannot be empty".to_string(),
                    ));
                }
                if password.is_empty() {
                    return Err(BackupError::Config(
                        "WebDAV password cannot be empty".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn target_id(&self) -> String {
        match self {
            Self::S3 { bucket, prefix, .. } => format!("s3://{bucket}{prefix}"),
            Self::MinIO { bucket, prefix, .. } => format!("minio://{bucket}{prefix}"),
            Self::B2 { bucket, prefix, .. } => format!("b2://{bucket}{prefix}"),
            Self::Wasabi { bucket, prefix, .. } => format!("wasabi://{bucket}{prefix}"),
            Self::Local { path, .. } => format!("local://{}", path.display()),
            Self::Azure {
                account,
                container,
                prefix,
                ..
            } => {
                format!("azure://{account}/{container}{prefix}")
            }
            Self::GCS { bucket, prefix, .. } => format!("gcs://{bucket}{prefix}"),
            Self::SFTP { host, path, .. } => format!("sftp://{host}{path}"),
            Self::WebDAV { url, prefix, .. } => format!("webdav://{url}{prefix}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Credentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureCredentials {
    pub account_key: Option<String>,
    pub sas_token: Option<String>,
    pub tenant_id: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TargetEncryption {
    None,
    Age { recipients: Vec<String> },
    Aes256Gcm { key: String },
    AwsKms { key_id: String },
    AzureKeyVault { key_url: String },
    GcpKms { key_name: String },
}

impl Default for TargetEncryption {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    pub daily: u32,
    pub weekly: u32,
    pub monthly: u32,
    pub yearly: u32,
    pub grace_period_days: u32,
    pub max_total_backups: Option<u32>,
    pub max_total_size_gb: Option<u64>,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            daily: 7,
            weekly: 4,
            monthly: 12,
            yearly: 7,
            grace_period_days: 7,
            max_total_backups: None,
            max_total_size_gb: None,
        }
    }
}

impl RetentionConfig {
    pub fn validate(&self) -> BackupResult<()> {
        if self.daily == 0 || self.weekly == 0 || self.monthly == 0 || self.yearly == 0 {
            return Err(BackupError::Config(
                "All retention periods must be > 0".to_string(),
            ));
        }
        if self.grace_period_days > 90 {
            return Err(BackupError::Config(
                "Grace period should not exceed 90 days".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub enabled: bool,
    pub test_db_name: String,
    pub test_db_template: Option<String>,
    pub verify_checksums: bool,
    pub verify_row_counts: bool,
    pub verify_schema: bool,
    pub timeout_seconds: u64,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            test_db_name: "agrocore_backup_test".to_string(),
            test_db_template: None,
            verify_checksums: true,
            verify_row_counts: true,
            verify_schema: true,
            timeout_seconds: 300,
        }
    }
}

impl VerificationConfig {
    pub fn validate(&self) -> BackupResult<()> {
        if self.test_db_name.is_empty() {
            return Err(BackupError::Config(
                "Verification test_db_name cannot be empty".to_string(),
            ));
        }
        if self.timeout_seconds == 0 {
            return Err(BackupError::Config(
                "Verification timeout must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub default: EncryptionMethod,
    pub age_recipients: Vec<String>,
    pub kms_key_id: Option<String>,
    pub key_rotation_days: Option<u32>,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            default: EncryptionMethod::Age,
            age_recipients: vec![],
            kms_key_id: None,
            key_rotation_days: Some(90),
        }
    }
}

impl EncryptionConfig {
    pub fn validate(&self) -> BackupResult<()> {
        if matches!(self.default, EncryptionMethod::Age) && self.age_recipients.is_empty() {
            return Err(BackupError::Config(
                "Age encryption requires at least one recipient".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EncryptionMethod {
    None,
    Age,
    Aes256Gcm,
    AwsKms,
    AzureKeyVault,
    GcpKms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgDumpConfig {
    pub compression_level: i32,
    pub no_owner: bool,
    pub no_privileges: bool,
    pub no_comments: bool,
    pub no_security_labels: bool,
    pub no_tablespaces: bool,
    pub exclude_tables: Vec<String>,
    pub include_tables: Vec<String>,
    pub format: DumpFormat,
    pub jobs: Option<u32>,
    pub timeout_seconds: u64,
}

impl Default for PgDumpConfig {
    fn default() -> Self {
        Self {
            compression_level: 6,
            no_owner: true,
            no_privileges: true,
            no_comments: true,
            no_security_labels: true,
            no_tablespaces: true,
            exclude_tables: vec![],
            include_tables: vec![],
            format: DumpFormat::Custom,
            jobs: None,
            timeout_seconds: 3600,
        }
    }
}

impl PgDumpConfig {
    pub fn validate(&self) -> BackupResult<()> {
        if !(1..=9).contains(&self.compression_level) {
            return Err(BackupError::Config(
                "Compression level must be between 1 and 9".to_string(),
            ));
        }
        if self.timeout_seconds == 0 {
            return Err(BackupError::Config(
                "pg_dump timeout must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DumpFormat {
    Custom,
    Directory,
    Tar,
    Plain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPathsConfig {
    pub env_files: Vec<PathBuf>,
    pub config_dirs: Vec<PathBuf>,
    pub docker_compose_files: Vec<PathBuf>,
    pub dockerfiles: Vec<PathBuf>,
    pub tls_certs: Vec<PathBuf>,
    pub secrets: Vec<PathBuf>,
    pub exclude_patterns: Vec<String>,
}

impl Default for ConfigPathsConfig {
    fn default() -> Self {
        Self {
            env_files: vec![
                PathBuf::from(".env"),
                PathBuf::from(".env.production"),
                PathBuf::from(".env.local"),
            ],
            config_dirs: vec![PathBuf::from("config")],
            docker_compose_files: vec![
                PathBuf::from("docker-compose.yml"),
                PathBuf::from("docker-compose.prod.yml"),
                PathBuf::from("docker-compose.dev.yml"),
            ],
            dockerfiles: vec![
                PathBuf::from("Dockerfile"),
                PathBuf::from("Dockerfile.api"),
                PathBuf::from("Dockerfile.service"),
                PathBuf::from("Dockerfile.migration"),
            ],
            tls_certs: vec![
                PathBuf::from("certs"),
                PathBuf::from("ssl"),
                PathBuf::from("tls"),
            ],
            secrets: vec![PathBuf::from("secrets"), PathBuf::from(".secrets")],
            exclude_patterns: vec![
                "*.log".to_string(),
                "*.tmp".to_string(),
                "*.bak".to_string(),
                "node_modules/**".to_string(),
                "target/**".to_string(),
                ".git/**".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadataConfig {
    pub include_git_info: bool,
    pub include_schema_version: bool,
    pub include_manifest_checksums: bool,
    pub custom_fields: HashMap<String, String>,
}

impl Default for BackupMetadataConfig {
    fn default() -> Self {
        Self {
            include_git_info: true,
            include_schema_version: true,
            include_manifest_checksums: true,
            custom_fields: HashMap::new(),
        }
    }
}

pub fn load_config() -> BackupResult<BackupConfig> {
    let mut builder = config::Config::builder()
        .add_source(config::Environment::with_prefix("BACKUP").separator("_"));

    if std::path::Path::new("config/backup.yml").exists() {
        builder = builder.add_source(config::File::new(
            "config/backup.yml",
            config::FileFormat::Yaml,
        ));
    } else if std::path::Path::new("config/backup.yaml").exists() {
        builder = builder.add_source(config::File::new(
            "config/backup.yaml",
            config::FileFormat::Yaml,
        ));
    }

    let config: BackupConfig = builder
        .build()
        .map_err(|e| BackupError::Config(format!("Failed to build config: {e}")))?
        .try_deserialize()
        .map_err(|e| BackupError::Config(format!("Failed to deserialize config: {e}")))?;

    config.validate()?;
    Ok(config)
}

mod cron {
    use std::str::FromStr;

    pub struct Schedule;

    impl Schedule {
        pub fn from_str(_s: &str) -> Result<Self, String> {
            // Simplified - in real implementation use cron crate
            Ok(Self)
        }
    }
}
