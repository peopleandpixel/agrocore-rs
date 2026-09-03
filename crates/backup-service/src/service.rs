use crate::config::BackupConfig;
use crate::encryption::EncryptionManager;
use crate::error::{BackupError, BackupResult};
use crate::manifest::ManifestManager;
use crate::nats_client::NatsClient;
use crate::pg_dump::PgDump;
use crate::retention::RetentionManager;
use crate::storage::StorageBackendTrait;
use crate::verification::VerificationManager;
use agrocore_logging::{debug, info, warn};
use agrocore_scheduler::{JobDefinition, JobType, SchedulerConfig, SchedulerService};
use async_nats::Client as NatsClientInner;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;
use uuid::Uuid;

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupType {
    Database,
    Config,
    Full,
}

impl BackupType {
    fn prefix(&self) -> &'static str {
        match self {
            BackupType::Database => "db/",
            BackupType::Config => "config/",
            BackupType::Full => "full/",
        }
    }
}

impl std::fmt::Display for BackupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupType::Database => write!(f, "database"),
            BackupType::Config => write!(f, "config"),
            BackupType::Full => write!(f, "full"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupStatus {
    Pending,
    Running,
    Completed,
    Failed,
    VerificationFailed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJob {
    pub id: Uuid,
    pub backup_type: BackupType,
    pub status: BackupStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub target_ids: Vec<String>,
    pub total_size_bytes: u64,
    pub error: Option<String>,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSummary {
    pub id: Uuid,
    pub backup_type: BackupType,
    pub status: BackupStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub total_size_bytes: u64,
    pub target_count: usize,
}

pub struct BackupService {
    config: BackupConfig,
    db_pool: sqlx::PgPool,
    nats: crate::nats_client::NatsClient,
    pg_dump: crate::pg_dump::PgDump,
    storage: Arc<dyn crate::storage::StorageBackendTrait>,
    encryption: Arc<crate::encryption::EncryptionManager>,
    retention: Arc<crate::retention::RetentionManager>,
    verification: Arc<crate::verification::VerificationManager>,
    manifest: Arc<crate::manifest::ManifestManager>,
    job_state: Arc<RwLock<HashMap<Uuid, BackupJob>>>,
    pub scheduler: Option<Arc<SchedulerService>>,
}

impl BackupService {
    pub async fn new(
        config: crate::config::BackupConfig,
        database_url: String,
        nats: crate::nats_client::NatsClient,
    ) -> BackupResult<Self> {
        // Create database pool
        let db_pool = sqlx::PgPool::connect(&database_url)
            .await
            .map_err(|e| BackupError::Database(e))?;

        // Initialize components
        let pg_dump = crate::pg_dump::PgDump::new(db_pool.clone(), config.pg_dump.clone());
        let storage = Arc::new(crate::storage::StorageBackend::new(config.targets.clone()).await?);
        let encryption = Arc::new(crate::encryption::EncryptionManager::new(
            config.encryption.clone(),
        ));
        let retention = Arc::new(crate::retention::RetentionManager::new(
            config.retention.clone(),
        ));
        let verification = Arc::new(crate::verification::VerificationManager::new(
            config.verification.clone(),
        ));
        let manifest = Arc::new(crate::manifest::ManifestManager::new(
            config.metadata.clone(),
        ));

        Ok(Self {
            config,
            db_pool,
            nats,
            pg_dump,
            storage,
            encryption,
            retention,
            verification,
            manifest,
            job_state: Arc::new(RwLock::new(HashMap::new())),
            scheduler: None,
        })
    }

    pub fn config(&self) -> &crate::config::BackupConfig {
        &self.config
    }

    pub async fn start_scheduler(&mut self) -> BackupResult<()> {
        let scheduler_config = SchedulerConfig {
            enabled: self.config.enabled,
            timezone: self.config.timezone.clone(),
            default_job_timeout_seconds: 3600,
            max_concurrent_jobs: 10,
            retry_failed_jobs: true,
            max_retries: 3,
            retry_delay_seconds: 60,
        };

        let nats_client = self.nats.clone();
        let scheduler = Arc::new(
            SchedulerService::new(scheduler_config, Some(nats_client.inner().clone())).await?,
        );

        // Register builtin handlers for backup jobs
        let service = self.clone();
        scheduler
            .register_handler("backup_database", move |job_def| {
                let service = service.clone();
                let backup_type = job_def
                    .payload
                    .get("backup_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("database");
                let bt = match backup_type {
                    "config" => crate::service::BackupType::Config,
                    _ => crate::service::BackupType::Database,
                };
                Box::pin(async move {
                    if let Err(e) = service.run_backup(bt).await {
                        error!("Scheduled backup failed: {}", e);
                    }
                    Ok(())
                })
            })
            .await;

        // Start the scheduler
        scheduler.start().await?;

        // Add DB backup job
        let db_job = JobDefinition {
            id: "backup-database".to_string(),
            name: "Database Backup".to_string(),
            description: "Scheduled database backup".to_string(),
            schedule: self.config.schedule_db.clone(),
            timezone: Some(self.config.timezone.clone()),
            job_type: JobType::Builtin {
                handler: "backup_database".to_string(),
            },
            payload: serde_json::json!({ "backup_type": "database" }),
            timeout_seconds: Some(3600),
            max_retries: Some(3),
            retry_delay_seconds: Some(60),
            enabled: self.config.enabled,
            tags: vec!["backup".to_string(), "database".to_string()],
        };
        scheduler.add_job(db_job).await?;

        // Add Config backup job
        let config_job = JobDefinition {
            id: "backup-config".to_string(),
            name: "Config Backup".to_string(),
            description: "Scheduled config backup".to_string(),
            schedule: self.config.schedule_config.clone(),
            timezone: Some(self.config.timezone.clone()),
            job_type: JobType::Builtin {
                handler: "backup_database".to_string(),
            },
            payload: serde_json::json!({ "backup_type": "config" }),
            timeout_seconds: Some(3600),
            max_retries: Some(3),
            retry_delay_seconds: Some(60),
            enabled: self.config.enabled,
            tags: vec!["backup".to_string(), "config".to_string()],
        };
        scheduler.add_job(config_job).await?;

        self.scheduler = Some(scheduler);

        Ok(())
    }

    pub async fn run_backup(&self, backup_type: BackupType) -> BackupResult<BackupJob> {
        let job_id = Uuid::new_v4();
        let started_at = Utc::now();

        let mut job = BackupJob {
            id: job_id,
            backup_type: backup_type.clone(),
            status: BackupStatus::Running,
            started_at,
            completed_at: None,
            target_ids: vec![],
            total_size_bytes: 0,
            error: None,
            progress: 0.0,
        };

        // Register job
        {
            let mut state = self.job_state.write().await;
            state.insert(job_id, job.clone());
        }

        // Publish start event
        self.nats.publish_backup_started(&job).await;

        info!("Starting {} backup: {}", backup_type, job_id);

        // Run backup for each target
        let mut total_size = 0u64;
        let mut target_ids = Vec::new();

        for target_ref in &self.config.targets {
            let target_id = target_ref.target_id();
            target_ids.push(target_id.clone());

            // Update progress
            self.update_job_progress(job_id, 0.1).await;

            match self
                .backup_to_target(target_ref, &backup_type, job_id)
                .await
            {
                Ok(size) => {
                    total_size += size;
                    info!("Backup to {} completed: {} bytes", target_id, size);
                }
                Err(e) => {
                    error!("Backup to {} failed: {}", target_id, e);
                    job.status = BackupStatus::Failed;
                    job.error = Some(e.to_string());
                    job.completed_at = Some(Utc::now());
                    self.update_job(job_id, job.clone()).await;
                    self.nats
                        .publish_backup_failed(&backup_type.to_string(), &e.to_string())
                        .await;
                    return Err(e);
                }
            }

            self.update_job_progress(job_id, 0.9).await;
        }

        // Verify if enabled
        if self.config.verification.enabled {
            self.update_job_progress(job_id, 0.95).await;
            if let Err(e) = self
                .verification
                .verify_backup(&target_ids, &backup_type)
                .await
            {
                error!("Backup verification failed: {}", e);
                job.status = BackupStatus::VerificationFailed;
                job.error = Some(e.to_string());
                job.completed_at = Some(Utc::now());
                self.update_job(job_id, job.clone()).await;
                self.nats
                    .publish_backup_failed(&backup_type.to_string(), &e.to_string())
                    .await;
                return Err(e);
            }
        }

        // Create manifest
        let manifest = self
            .manifest
            .create_manifest(
                job_id,
                &backup_type,
                &target_ids,
                total_size,
                started_at,
                Utc::now(),
            )
            .await?;

        // Save manifest to all targets
        for target_ref in &self.config.targets {
            self.storage.save_manifest(target_ref, &manifest).await?;
        }

        // Run retention cleanup
        if let Err(e) = self
            .retention
            .cleanup(&self.storage, &self.config.targets)
            .await
        {
            warn!("Retention cleanup failed: {}", e);
        }

        // Complete job
        job.status = BackupStatus::Completed;
        job.completed_at = Some(Utc::now());
        job.target_ids = target_ids;
        job.total_size_bytes = total_size;
        job.progress = 1.0;

        self.update_job(job_id, job.clone()).await;
        self.nats.publish_backup_completed(&job).await;

        info!(
            "Backup {} completed successfully: {} bytes to {} targets",
            job_id,
            total_size,
            self.config.targets.len()
        );

        Ok(job)
    }

    async fn backup_to_target(
        &self,
        target: &crate::config::BackupTarget,
        backup_type: &BackupType,
        job_id: Uuid,
    ) -> BackupResult<u64> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let prefix = format!("{}{}/", backup_type.prefix(), timestamp);

        match backup_type {
            BackupType::Database => {
                self.pg_dump
                    .dump_to_storage(&self.storage, target, &prefix, job_id)
                    .await
            }
            BackupType::Config => self.backup_config_to_storage(target, &prefix, job_id).await,
            BackupType::Full => {
                let db_size = self
                    .pg_dump
                    .dump_to_storage(&self.storage, target, &prefix, job_id)
                    .await?;
                let config_size = self
                    .backup_config_to_storage(target, &prefix, job_id)
                    .await?;
                Ok(db_size + config_size)
            }
        }
    }

    async fn backup_config_to_storage(
        &self,
        target: &crate::config::BackupTarget,
        prefix: &str,
        job_id: Uuid,
    ) -> BackupResult<u64> {
        let temp_dir = tempfile::tempdir().map_err(|e| BackupError::Io(e))?;
        let config_path = temp_dir.path().join("config_backup.tar.gz");

        // Collect config files
        let mut total_size = 0u64;

        // Create tar.gz of config files
        let tar_gz = std::fs::File::create(&config_path).map_err(|e| BackupError::Io(e))?;
        let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
        let mut tar = tar::Builder::new(enc);

        for env_file in &self.config.config_paths.env_files {
            if env_file.exists() {
                tar.append_path_with_name(env_file, env_file.file_name().unwrap())
                    .map_err(|e| BackupError::Io(e))?;
            }
        }

        for config_dir in &self.config.config_paths.config_dirs {
            if config_dir.exists() {
                tar.append_dir_all("config", config_dir)
                    .map_err(|e| BackupError::Io(e))?;
            }
        }

        for compose_file in &self.config.config_paths.docker_compose_files {
            if compose_file.exists() {
                tar.append_path_with_name(compose_file, compose_file.file_name().unwrap())
                    .map_err(|e| BackupError::Io(e))?;
            }
        }

        for dockerfile in &self.config.config_paths.dockerfiles {
            if dockerfile.exists() {
                tar.append_path_with_name(dockerfile, dockerfile.file_name().unwrap())
                    .map_err(|e| BackupError::Io(e))?;
            }
        }

        for tls_dir in &self.config.config_paths.tls_certs {
            if tls_dir.exists() {
                tar.append_dir_all("tls", tls_dir)
                    .map_err(|e| BackupError::Io(e))?;
            }
        }

        for secrets_dir in &self.config.config_paths.secrets {
            if secrets_dir.exists() {
                tar.append_dir_all("secrets", secrets_dir)
                    .map_err(|e| BackupError::Io(e))?;
            }
        }

        tar.finish().map_err(|e| BackupError::Io(e))?;

        // Encrypt if needed
        let final_path = if self.config.encryption.default != crate::config::EncryptionMethod::None
        {
            let encrypted_path = temp_dir.path().join("config_backup.tar.gz.enc");
            self.encryption
                .encrypt_file(&config_path, &encrypted_path)
                .await?;
            encrypted_path
        } else {
            config_path
        };

        // Upload
        let object_name = format!(
            "{}config_backup.tar.gz{}",
            prefix,
            if self.config.encryption.default != crate::config::EncryptionMethod::None {
                ".enc"
            } else {
                ""
            }
        );
        let size = self
            .storage
            .upload_file(target, &object_name, &final_path)
            .await?;

        total_size += size;
        Ok(total_size)
    }

    async fn update_job_progress(&self, job_id: Uuid, progress: f32) {
        let mut state = self.job_state.write().await;
        if let Some(job) = state.get_mut(&job_id) {
            job.progress = progress;
        }
        // Publish progress event
        if let Some(job) = state.get(&job_id) {
            self.nats.publish_backup_progress(job).await;
        }
    }

    async fn update_job(&self, job_id: Uuid, job: BackupJob) {
        let mut state = self.job_state.write().await;
        state.insert(job_id, job);
    }

    pub async fn create_manual_backup(&self, backup_type: BackupType) -> BackupResult<BackupJob> {
        info!("Manual backup requested: {:?}", backup_type);
        self.run_backup(backup_type).await
    }

    pub async fn restore(&self, backup_id: Uuid, target_db: Option<String>) -> BackupResult<()> {
        info!("Starting restore for backup: {}", backup_id);
        // TODO: Implement restore logic
        Err(BackupError::InvalidState(
            "Restore not yet implemented".to_string(),
        ))
    }

    pub async fn list_backups(&self) -> BackupResult<Vec<BackupSummary>> {
        // TODO: List backups from all targets
        Ok(vec![])
    }

    pub async fn get_job_status(&self, job_id: Uuid) -> Option<BackupJob> {
        let state = self.job_state.read().await;
        state.get(&job_id).cloned()
    }
}

impl Clone for BackupService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            db_pool: self.db_pool.clone(),
            nats: self.nats.clone(),
            pg_dump: self.pg_dump.clone(),
            storage: self.storage.clone(),
            encryption: self.encryption.clone(),
            retention: self.retention.clone(),
            verification: self.verification.clone(),
            manifest: self.manifest.clone(),
            job_state: self.job_state.clone(),
            scheduler: self.scheduler.clone(),
        }
    }
}
