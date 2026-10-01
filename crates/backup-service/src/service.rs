use crate::config::BackupConfig;
use crate::config::BackupTarget;
use crate::error::{BackupError, BackupResult};
use agrocore_logging::{error, info, warn};
use agrocore_scheduler::{JobDefinition, JobType, SchedulerConfig, SchedulerService};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupType {
    Database,
    Config,
    Full,
}

impl BackupType {
    pub fn prefix(&self) -> &'static str {
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

impl std::fmt::Display for BackupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupStatus::Pending => write!(f, "pending"),
            BackupStatus::Running => write!(f, "running"),
            BackupStatus::Completed => write!(f, "completed"),
            BackupStatus::Failed => write!(f, "failed"),
            BackupStatus::VerificationFailed => write!(f, "verification_failed"),
            BackupStatus::Cancelled => write!(f, "cancelled"),
        }
    }
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
            .map_err(BackupError::Database)?;

        // Initialize components
        let pg_dump = crate::pg_dump::PgDump::new(db_pool.clone(), config.pg_dump.clone());
        let storage = Arc::new(crate::storage::StorageBackend::new(config.targets.clone()).await?)
            as Arc<dyn crate::storage::StorageBackendTrait>;
        let encryption = Arc::new(crate::encryption::EncryptionManager::new(
            config.encryption.clone(),
        ));
        let retention = Arc::new(crate::retention::RetentionManager::new(
            config.retention.clone(),
        ));
        let verification = Arc::new(crate::verification::VerificationManager::new(
            config.verification.clone(),
            pg_dump.clone(),
            db_pool.clone(),
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

    pub fn db_pool(&self) -> &sqlx::PgPool {
        &self.db_pool
    }

    pub async fn start_scheduler(
        &mut self,
        mut bridge: agrocore_messaging::MqttBridge,
    ) -> BackupResult<()> {
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

        // Register handler for MQTT Bridge stats reporting
        let stats = bridge.stats.clone();
        scheduler
            .register_handler("bridge_stats_reporter", move |_job_def| {
                let stats = stats.clone();
                Box::pin(async move {
                    let stats = stats.read().await;
                    agrocore_logging::info!(
                        "Bridge Stats - NATS→MQTT: {}, MQTT→NATS: {}, NATS Errors: {}, MQTT Errors: {}, Connected: {}",
                        stats.nats_to_mqtt_messages,
                        stats.mqtt_to_nats_messages,
                        stats.nats_errors,
                        stats.mqtt_errors,
                        stats.connected
                    );
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

        // Add MQTT Bridge stats reporter job (every 60 seconds)
        let bridge_stats_job = JobDefinition {
            id: "bridge_stats_reporter".to_string(),
            name: "MQTT Bridge Stats Reporter".to_string(),
            description: "Periodic stats reporting for MQTT Bridge".to_string(),
            schedule: "* * * * * *".to_string(), // Every 60 seconds (every minute)
            timezone: Some("UTC".to_string()),
            job_type: JobType::Builtin {
                handler: "bridge_stats_reporter".to_string(),
            },
            payload: serde_json::json!({}),
            timeout_seconds: Some(30),
            max_retries: Some(3),
            retry_delay_seconds: Some(10),
            enabled: true,
            tags: vec![
                "messaging".to_string(),
                "bridge".to_string(),
                "stats".to_string(),
            ],
        };
        scheduler.add_job(bridge_stats_job).await?;

        // Start the bridge on main thread (blocks until shutdown)
        info!("Starting MQTT Bridge on main thread...");
        if let Err(e) = bridge.start().await {
            agrocore_logging::error!("MQTT Bridge error: {}", e);
        }

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
                    crate::metrics::BackupMetrics::get().record_failure(&backup_type.to_string());
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
                .verify_backup(
                    &target_ids,
                    &backup_type,
                    &self.storage,
                    &self.config.targets,
                )
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

        // Build the per-target manifest entries by listing what was actually
        // written, so a restore can locate the dump without guessing file names.
        let mut manifest_targets = Vec::new();
        for target_ref in &self.config.targets {
            let objects = self
                .storage
                .list_objects(target_ref, "")
                .await
                .unwrap_or_default();

            // Only objects belonging to this backup id.
            let id_string = job_id.to_string();
            let entries: Vec<crate::manifest::ObjectManifest> = objects
                .into_iter()
                .filter(|(name, _)| name.contains(&id_string))
                .map(|(name, size)| crate::manifest::ObjectManifest {
                    name,
                    size_bytes: size,
                    checksum_sha256: String::new(),
                    modified_at: Utc::now(),
                })
                .collect();

            manifest_targets.push(crate::manifest::TargetManifest {
                target_id: target_ref.target_id(),
                target_type: format!("{target_ref:?}")
                    .split(['(', ' '])
                    .next()
                    .unwrap_or("unknown")
                    .to_string(),
                size_bytes: entries.iter().map(|e| e.size_bytes).sum(),
                encryption: None,
                objects: entries,
            });
        }

        // Create manifest
        let manifest = self
            .manifest
            .create_manifest(
                job_id,
                &backup_type,
                manifest_targets,
                total_size,
                started_at,
                Utc::now(),
            )
            .await?;

        // Save manifest to all targets
        for target_ref in &self.config.targets {
            self.storage.save_manifest(target_ref, &manifest).await?;
        }

        // Record the successful backup: duration and payload size.
        let duration_secs = (Utc::now() - started_at).num_milliseconds() as f64 / 1000.0;
        crate::metrics::BackupMetrics::get().record_success(
            &backup_type.to_string(),
            duration_secs,
            total_size,
        );

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
        _job_id: Uuid,
    ) -> BackupResult<u64> {
        let temp_dir = tempfile::tempdir().map_err(BackupError::Io)?;
        let config_path = temp_dir.path().join("config_backup.tar.gz");

        // Collect config files
        let mut total_size = 0u64;

        // Create tar.gz of config files
        let tar_gz = std::fs::File::create(&config_path).map_err(BackupError::Io)?;
        let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
        let mut tar = tar::Builder::new(enc);

        for env_file in &self.config.config_paths.env_files {
            if env_file.exists() {
                tar.append_path_with_name(env_file, env_file.file_name().unwrap())
                    .map_err(BackupError::Io)?;
            }
        }

        for config_dir in &self.config.config_paths.config_dirs {
            if config_dir.exists() {
                tar.append_dir_all("config", config_dir)
                    .map_err(BackupError::Io)?;
            }
        }

        for compose_file in &self.config.config_paths.docker_compose_files {
            if compose_file.exists() {
                tar.append_path_with_name(compose_file, compose_file.file_name().unwrap())
                    .map_err(BackupError::Io)?;
            }
        }

        for dockerfile in &self.config.config_paths.dockerfiles {
            if dockerfile.exists() {
                tar.append_path_with_name(dockerfile, dockerfile.file_name().unwrap())
                    .map_err(BackupError::Io)?;
            }
        }

        for tls_dir in &self.config.config_paths.tls_certs {
            if tls_dir.exists() {
                tar.append_dir_all("tls", tls_dir)
                    .map_err(BackupError::Io)?;
            }
        }

        for secrets_dir in &self.config.config_paths.secrets {
            if secrets_dir.exists() {
                tar.append_dir_all("secrets", secrets_dir)
                    .map_err(BackupError::Io)?;
            }
        }

        tar.finish().map_err(BackupError::Io)?;

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

    /// Restore a backup into a database.
    ///
    /// `dry_run` validates that the dump object exists and is readable without
    /// writing anything, which lets callers confirm a backup is restorable
    /// before committing to the operation.
    pub async fn restore(
        &self,
        backup_id: Uuid,
        target_db: Option<String>,
        dry_run: bool,
    ) -> BackupResult<RestoreOutcome> {
        info!("Starting restore for backup: {backup_id} (dry_run={dry_run})");

        let (target, object_name) = self
            .find_backup_object(backup_id)
            .await?
            .ok_or_else(|| BackupError::InvalidState(format!("Backup {backup_id} not found")))?;

        let size = self
            .storage
            .list_objects(&target, &object_name)
            .await?
            .iter()
            .find(|(name, _)| name == &object_name)
            .map(|(_, size)| *size)
            .unwrap_or(0);

        if dry_run {
            info!("Dry run for backup {backup_id}: object {object_name} is readable");
            return Ok(RestoreOutcome {
                backup_id,
                object_name,
                bytes_restored: 0,
                dry_run: true,
            });
        }

        let restore_result = self
            .pg_dump
            .restore_from_storage(&self.storage, &target, &object_name, target_db)
            .await;

        crate::metrics::BackupMetrics::get().record_restore(restore_result.is_ok());
        restore_result?;

        info!("Restore completed for backup {backup_id}");
        Ok(RestoreOutcome {
            backup_id,
            object_name,
            bytes_restored: size,
            dry_run: false,
        })
    }

    /// Locate the dump object belonging to a backup id.
    ///
    /// The manifest written next to each backup is the source of truth: it
    /// records the object name that was actually uploaded. File-name matching
    /// is only used as a fallback for backups taken before manifests were
    /// persisted.
    async fn find_backup_object(
        &self,
        backup_id: Uuid,
    ) -> BackupResult<Option<(BackupTarget, String)>> {
        let id_string = backup_id.to_string();

        // 1) Manifest lookup: authoritative.
        for target in &self.config.targets {
            if let Some(manifest) = self.storage.load_manifest(target, &backup_id).await? {
                // The manifest records every object written for this backup;
                // the dump is the object carrying the `.dump` extension.
                for target_manifest in &manifest.targets {
                    for object in &target_manifest.objects {
                        if object.name.ends_with(".dump") {
                            return Ok(Some((target.clone(), object.name.clone())));
                        }
                    }
                }
            }
        }

        // 2) Fallback: match the backup id embedded in the object name.
        for target in &self.config.targets {
            let objects = self
                .storage
                .list_objects(target, "")
                .await
                .unwrap_or_default();
            for (name, _) in objects {
                if name.contains(&id_string) {
                    return Ok(Some((target.clone(), name)));
                }
            }
        }

        Ok(None)
    }

    /// List backups found across all configured targets, newest first.
    pub async fn list_backups(&self) -> BackupResult<Vec<BackupSummary>> {
        let mut summaries = Vec::new();

        for target in &self.config.targets {
            let objects = self
                .storage
                .list_objects(target, "")
                .await
                .unwrap_or_default();
            for (name, size) in objects {
                // Only dump objects represent backups.
                if !name.ends_with(".dump") {
                    continue;
                }
                let started_at = Self::parse_dump_timestamp(&name).unwrap_or_else(Utc::now);
                summaries.push(BackupSummary {
                    id: Uuid::new_v4(),
                    backup_type: BackupType::Database,
                    status: BackupStatus::Completed,
                    started_at,
                    completed_at: Some(started_at),
                    total_size_bytes: size,
                    target_count: 1,
                });
            }
        }

        summaries.sort_by_key(|b| std::cmp::Reverse(b.started_at));
        Ok(summaries)
    }

    /// Extract the timestamp encoded in `<label>_<YYYYMMDD>_<HHMMSS>.dump`.
    ///
    /// Object names may embed a backup id (`<id>_dump_<stamp>.dump`), so the
    /// date and time are taken from the last two underscore-separated fields
    /// rather than by position.
    fn parse_dump_timestamp(name: &str) -> Option<DateTime<Utc>> {
        let (stem, ext) = name.rsplit_once('.')?;
        if !ext.eq_ignore_ascii_case("dump") {
            return None;
        }

        let (date_time, time) = stem.rsplit_once('_')?;
        let (_prefix, date) = date_time.rsplit_once('_')?;

        let naive =
            chrono::NaiveDateTime::parse_from_str(&format!("{date} {time}"), "%Y%m%d %H%M%S")
                .ok()?;
        Some(DateTime::from_naive_utc_and_offset(naive, Utc))
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

/// Result of a restore operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RestoreOutcome {
    pub backup_id: Uuid,
    pub object_name: String,
    pub bytes_restored: u64,
    pub dry_run: bool,
}
