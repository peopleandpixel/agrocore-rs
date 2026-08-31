use crate::config::{BackupConfig, BackupTarget, RetentionConfig};
use crate::error::{BackupError, BackupResult};
use crate::storage::StorageBackendTrait;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

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
            if let Err(e) = self.cleanup_target(target).await {
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

    async fn cleanup_target(&self, target: &crate::config::BackupTarget) -> BackupResult<()> {
        // List all backup objects for this target
        let backups = self.list_backups(target).await?;

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

        // Delete selected backups
        for backup in to_delete {
            // TODO: Implement actual deletion from storage
            info!(
                "Would delete backup: {} (age: {} days)",
                backup.id,
                (Utc::now() - backup.timestamp).num_days()
            );
        }

        Ok(())
    }

    fn select_for_deletion(&self, backups: &[BackupInfo], keep: usize) -> Vec<BackupInfo> {
        if backups.len() <= keep {
            return Vec::new();
        }
        let mut sorted = backups.to_vec();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted.into_iter().skip(keep).collect()
    }

    async fn list_backups(
        &self,
        target: &crate::config::BackupTarget,
    ) -> BackupResult<Vec<BackupInfo>> {
        // TODO: Implement actual listing from storage
        Ok(vec![])
    }
}

#[derive(Debug, Clone)]
struct BackupInfo {
    id: Uuid,
    timestamp: DateTime<Utc>,
    size_bytes: u64,
    backup_type: crate::service::BackupType,
}
