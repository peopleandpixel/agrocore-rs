use crate::config::RetentionConfig;
use crate::error::BackupResult;
use crate::storage::StorageBackendTrait;
use agrocore_logging::{info, warn};
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;

/// Object prefix scanned by retention.
const PREFIX: &str = "";

pub struct RetentionManager {
    config: RetentionConfig,
}

impl RetentionManager {
    pub fn new(config: RetentionConfig) -> Self {
        Self { config }
    }

    pub async fn cleanup(
        &self,
        storage: &Arc<dyn StorageBackendTrait>,
        targets: &[crate::config::BackupTarget],
    ) -> BackupResult<()> {
        info!("Starting retention cleanup");

        for target in targets {
            if let Err(e) = self.cleanup_target(storage, target).await {
                warn!(
                    "Retention cleanup failed for target {}: {}",
                    target.target_id(),
                    e
                );
            }
        }

        info!("Retention cleanup completed");
        Ok(())
    }

    async fn cleanup_target(
        &self,
        storage: &Arc<dyn StorageBackendTrait>,
        target: &crate::config::BackupTarget,
    ) -> BackupResult<()> {
        // List all backup objects for this target
        let backups = self.list_backups(storage, target, PREFIX).await?;

        // Group by date and classify
        let mut daily = Vec::new();
        let mut weekly = Vec::new();
        let mut monthly = Vec::new();
        let mut yearly = Vec::new();

        for backup in backups {
            let age = Utc::now().signed_duration_since(backup.timestamp);
            let days = age.num_days();

            if days <= self.config.daily as i64 {
                daily.push(backup);
            } else if days <= (self.config.daily + self.config.weekly * 7) as i64 {
                weekly.push(backup);
            } else if days
                <= (self.config.daily + self.config.weekly * 7 + self.config.monthly * 30) as i64
            {
                monthly.push(backup);
            } else {
                yearly.push(backup);
            }
        }

        // Keep only the most recent in each category
        let to_delete = self
            .select_for_deletion(&daily, self.config.daily as usize)
            .into_iter()
            .chain(self.select_for_deletion(&weekly, self.config.weekly as usize))
            .chain(self.select_for_deletion(&monthly, self.config.monthly as usize))
            .chain(self.select_for_deletion(&yearly, self.config.yearly as usize))
            .collect::<Vec<_>>();

        // Apply grace period
        let grace_cutoff = Utc::now() - Duration::days(self.config.grace_period_days as i64);
        let to_delete: Vec<_> = to_delete
            .into_iter()
            .filter(|b| b.timestamp < grace_cutoff)
            .collect();

        // Delete selected backups from storage.
        for backup in to_delete {
            match storage.delete_object(target, &backup.object_name).await {
                Ok(()) => info!(
                    "Deleted backup {} ({} days old, {} bytes)",
                    backup.object_name,
                    (Utc::now() - backup.timestamp).num_days(),
                    backup.size_bytes
                ),
                Err(e) => warn!("Failed to delete backup {}: {}", backup.object_name, e),
            }
        }

        Ok(())
    }

    fn select_for_deletion(&self, backups: &[BackupInfo], keep: usize) -> Vec<BackupInfo> {
        if backups.len() <= keep {
            return Vec::new();
        }
        let mut sorted = backups.to_vec();
        sorted.sort_by_key(|a| std::cmp::Reverse(a.timestamp));
        sorted.into_iter().skip(keep).collect()
    }

    /// List backup objects under `prefix`, newest first.
    ///
    /// Objects whose name does not embed a `dump_YYYYMMDD_HHMMSS.dump`
    /// timestamp are skipped: they are not managed by retention and must not
    /// be deleted by it.
    async fn list_backups(
        &self,
        storage: &Arc<dyn StorageBackendTrait>,
        target: &crate::config::BackupTarget,
        prefix: &str,
    ) -> BackupResult<Vec<BackupInfo>> {
        let objects = storage.list_objects(target, prefix).await?;
        let mut out = Vec::new();

        for (name, size) in objects {
            let Some(timestamp) = Self::parse_dump_timestamp(&name) else {
                continue;
            };
            out.push(BackupInfo {
                object_name: name,
                timestamp,
                size_bytes: size,
            });
        }

        out.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        Ok(out)
    }

    /// Test hook: expose the dump timestamp parser.
    pub fn debug_parse(name: &str) -> Option<DateTime<Utc>> {
        Self::parse_dump_timestamp(name)
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
}

#[derive(Debug, Clone)]
struct BackupInfo {
    /// Storage object key, used for deletion.
    object_name: String,
    timestamp: DateTime<Utc>,
    size_bytes: u64,
}
