use crate::error::{BackupError, BackupResult};
use crate::service::{BackupJob, BackupStatus, BackupType};
use async_nats::{Client, ConnectOptions};
use futures::sink::SinkExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupEvent {
    Started(BackupJob),
    Progress(BackupJob),
    Completed(BackupJob),
    Failed {
        backup_type: String,
        error: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

pub struct NatsClient {
    client: Client,
}

impl Clone for NatsClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}

impl NatsClient {
    pub async fn connect(url: &str) -> BackupResult<Self> {
        let options = ConnectOptions::new()
            .max_reconnects(10)
            .reconnect_delay_callback(|_| std::time::Duration::from_secs(1));

        let client = async_nats::connect_with_options(url, options)
            .await
            .map_err(|e| BackupError::Nats(Box::new(e)))?;

        info!("Connected to NATS at {}", url);
        Ok(Self { client })
    }

    pub async fn publish_backup_started(&self, job: &crate::service::BackupJob) {
        let event = BackupEvent::Started(job.clone());
        self.publish_event("backup.started", &event).await;
    }

    pub async fn publish_backup_progress(&self, job: &crate::service::BackupJob) {
        let event = BackupEvent::Progress(job.clone());
        self.publish_event("backup.progress", &event).await;
    }

    pub async fn publish_backup_completed(&self, job: &crate::service::BackupJob) {
        let event = BackupEvent::Completed(job.clone());
        self.publish_event("backup.completed", &event).await;
    }

    pub async fn publish_backup_failed(&self, backup_type: &str, error: &str) {
        let event = BackupEvent::Failed {
            backup_type: backup_type.to_string(),
            error: error.to_string(),
            timestamp: chrono::Utc::now(),
        };
        self.publish_event("backup.failed", &event).await;
    }

    async fn publish_event(&self, subject: &str, event: &BackupEvent) {
        let payload = serde_json::to_vec(event).unwrap_or_default();
        if let Err(e) = self.client.publish(subject, payload.into()).await {
            warn!("Failed to publish NATS event: {}", e);
        }
    }

    pub async fn close(&mut self) {
        if let Err(e) = self.client.close().await {
            warn!("Failed to close NATS client: {}", e);
        }
    }
}
