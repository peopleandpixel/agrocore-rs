//! Integration tests for backup service

use agrocore_backup::{
    config::{
        BackupConfig, BackupTarget, ConfigPathsConfig, DumpFormat, EncryptionConfig,
        EncryptionMethod, PgDumpConfig, RetentionConfig, VerificationConfig,
    },
    error::{BackupError, BackupResult},
    service::{BackupStatus, BackupType},
};
use uuid::Uuid;

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_backup_config_default() {
        let config = BackupConfig::default();
        assert!(config.enabled);
        assert_eq!(config.schedule_db, "0 2 * * *");
        assert_eq!(config.schedule_config, "0 3 * * 0");
        assert_eq!(config.timezone, "UTC");
        assert!(config.targets.is_empty());
    }

    #[test]
    fn test_backup_config_validation_empty_targets() {
        let config = BackupConfig {
            enabled: true,
            targets: vec![],
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::Config(_)));
    }

    #[test]
    fn test_backup_config_validation_valid() {
        // Default EncryptionConfig is `Age` with no recipients, which
        // validate() rejects; so a config carrying it must fail.
        let age_no_recipients = BackupConfig {
            enabled: true,
            targets: vec![BackupTarget::Local {
                path: std::path::PathBuf::from("/tmp/backup"),
                encryption: Default::default(),
                permissions: None,
            }],
            ..Default::default()
        };

        let err = age_no_recipients
            .validate()
            .expect_err("Age encryption without recipients must fail validation");
        assert!(
            err.to_string().contains("recipient"),
            "expected a missing-recipient error, got {err}"
        );

        // With a usable encryption method the whole config validates.
        let valid = BackupConfig {
            enabled: true,
            targets: vec![BackupTarget::Local {
                path: std::path::PathBuf::from("/tmp/backup"),
                encryption: Default::default(),
                permissions: None,
            }],
            encryption: EncryptionConfig {
                default: EncryptionMethod::None,
                ..Default::default()
            },
            ..Default::default()
        };
        valid
            .validate()
            .expect("config with None encryption and one local target must validate");
    }

    #[test]
    fn test_s3_target_validation() {
        let target = BackupTarget::S3 {
            bucket: "test-bucket".to_string(),
            region: "us-east-1".to_string(),
            prefix: "backups/".to_string(),
            endpoint: None,
            credentials: None,
            encryption: Default::default(),
            kms_key_id: None,
            storage_class: None,
            max_concurrent_uploads: None,
            part_size: None,
        };

        let result = target.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_s3_target_validation_invalid() {
        let target = BackupTarget::S3 {
            bucket: "".to_string(),
            region: "us-east-1".to_string(),
            prefix: "backups/".to_string(),
            endpoint: None,
            credentials: None,
            encryption: Default::default(),
            kms_key_id: None,
            storage_class: None,
            max_concurrent_uploads: None,
            part_size: None,
        };

        let result = target.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_local_target_validation() {
        let target = BackupTarget::Local {
            path: std::path::PathBuf::from("/tmp/backup"),
            encryption: Default::default(),
            permissions: None,
        };

        let result = target.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_target_id_formats() {
        let s3 = BackupTarget::S3 {
            bucket: "my-bucket".to_string(),
            region: "us-east-1".to_string(),
            prefix: "backups/".to_string(),
            endpoint: None,
            credentials: None,
            encryption: Default::default(),
            kms_key_id: None,
            storage_class: None,
            max_concurrent_uploads: None,
            part_size: None,
        };
        assert_eq!(s3.target_id(), "s3://my-bucketbackups/");

        let local = BackupTarget::Local {
            path: std::path::PathBuf::from("/data/backup"),
            encryption: Default::default(),
            permissions: None,
        };
        assert_eq!(local.target_id(), "local:///data/backup");
    }
}

#[cfg(test)]
mod retention_tests {
    use super::*;

    #[test]
    fn test_retention_config_default() {
        let config = RetentionConfig::default();
        assert_eq!(config.daily, 7);
        assert_eq!(config.weekly, 4);
        assert_eq!(config.monthly, 12);
        assert_eq!(config.yearly, 7);
        assert_eq!(config.grace_period_days, 7);
    }

    #[test]
    fn test_retention_config_validation() {
        let config = RetentionConfig {
            daily: 0,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod verification_tests {
    use super::*;

    #[test]
    fn test_verification_config_default() {
        let config = VerificationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.test_db_name, "agrocore_backup_test");
        assert!(config.verify_checksums);
        assert!(config.verify_row_counts);
        assert!(config.verify_schema);
        assert_eq!(config.timeout_seconds, 300);
    }

    #[test]
    fn test_verification_config_validation() {
        let config = VerificationConfig {
            test_db_name: String::new(),
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod pg_dump_tests {
    use super::*;

    #[test]
    fn test_pg_dump_config_default() {
        let config = PgDumpConfig::default();
        assert_eq!(config.compression_level, 6);
        assert!(config.no_owner);
        assert!(config.no_privileges);
        assert_eq!(config.format, DumpFormat::Custom);
    }
}

#[cfg(test)]
mod backup_type_tests {
    use super::*;

    #[test]
    fn test_backup_type_prefix() {
        assert_eq!(BackupType::Database.prefix(), "db/");
        assert_eq!(BackupType::Config.prefix(), "config/");
        assert_eq!(BackupType::Full.prefix(), "full/");
    }

    #[test]
    fn test_backup_type_display() {
        assert_eq!(BackupType::Database.to_string(), "database");
        assert_eq!(BackupType::Config.to_string(), "config");
        assert_eq!(BackupType::Full.to_string(), "full");
    }
}

#[cfg(test)]
mod backup_status_tests {
    use super::*;

    #[test]
    fn test_backup_status_display() {
        assert_eq!(BackupStatus::Pending.to_string(), "pending");
        assert_eq!(BackupStatus::Running.to_string(), "running");
        assert_eq!(BackupStatus::Completed.to_string(), "completed");
        assert_eq!(BackupStatus::Failed.to_string(), "failed");
        assert_eq!(
            BackupStatus::VerificationFailed.to_string(),
            "verification_failed"
        );
        assert_eq!(BackupStatus::Cancelled.to_string(), "cancelled");
    }
}

#[cfg(test)]
mod backup_job_tests {
    use super::*;

    #[test]
    fn test_backup_job_creation() {
        let job_id = Uuid::new_v4();
        let job = agrocore_backup::service::BackupJob {
            id: job_id,
            backup_type: BackupType::Database,
            status: BackupStatus::Pending,
            started_at: chrono::Utc::now(),
            completed_at: None,
            target_ids: vec![],
            total_size_bytes: 0,
            error: None,
            progress: 0.0,
        };

        assert_eq!(job.id, job_id);
        assert_eq!(job.backup_type, BackupType::Database);
        assert_eq!(job.status, BackupStatus::Pending);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_load_config_from_env() {
        // Test loading config from environment variables
        unsafe {
            std::env::set_var("BACKUP_ENABLED", "true");
            std::env::set_var("BACKUP_SCHEDULE_DB", "0 1 * * *");
            std::env::set_var("BACKUP_TIMEZONE", "Europe/Berlin");
        }

        // Only env vars are set here, so required fields that are not part of
        // the BACKUP_* env surface (e.g. schedule_config) are absent and
        // deserialization must fail loudly rather than silently defaulting.
        let err = match agrocore_backup::config::load_config() {
            Ok(config) => {
                // If a config file is present and complete, the overrides must
                // have been honoured.
                assert!(config.enabled);
                assert_eq!(config.schedule_db, "0 1 * * *");
                assert_eq!(config.timezone, "Europe/Berlin");
                return;
            }
            Err(err) => err,
        };
        assert!(
            err.to_string().contains("deserialize") || err.to_string().contains("build"),
            "unexpected config error: {err}"
        );

        unsafe {
            std::env::remove_var("BACKUP_ENABLED");
            std::env::remove_var("BACKUP_SCHEDULE_DB");
            std::env::remove_var("BACKUP_TIMEZONE");
        }
    }

    #[test]
    fn test_backup_config_paths_default() {
        let config = ConfigPathsConfig::default();
        // Default config has some predefined paths
        assert!(!config.env_files.is_empty());
        assert!(!config.config_dirs.is_empty());
        assert!(!config.docker_compose_files.is_empty());
        assert!(!config.dockerfiles.is_empty());
        assert!(!config.tls_certs.is_empty());
        assert!(!config.secrets.is_empty());
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_backup_error_display() {
        let err = BackupError::Config("test config error".to_string());
        let display = format!("{}", err);
        assert!(display.contains("test config error"));
    }

    #[test]
    fn test_backup_result_type() {
        let ok: BackupResult<()> = Ok(());
        assert!(ok.is_ok());

        let err: BackupResult<()> = Err(BackupError::Config("error".to_string()));
        assert!(err.is_err());

        // Consume the error the way callers do, and check the rendered message.
        let Err(err) = err else {
            panic!("expected an Err result");
        };
        assert!(err.to_string().contains("error"));
    }
}
