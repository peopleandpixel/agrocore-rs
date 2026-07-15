// Worker-Task-Status für Multi-Worker-Aufgaben
// Jeder Worker hat einen separaten Status inne

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerTaskStatusType {
    New,
    Started,
    Paused,
    Stopped,
    Done,
}

impl std::fmt::Display for WorkerTaskStatusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerTaskStatusType::New => write!(f, "new"),
            WorkerTaskStatusType::Started => write!(f, "started"),
            WorkerTaskStatusType::Paused => write!(f, "paused"),
            WorkerTaskStatusType::Stopped => write!(f, "stopped"),
            WorkerTaskStatusType::Done => write!(f, "done"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkerTaskStatus {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub tenant_id: TenantId,
    #[sqlx(json)]
    pub status: WorkerTaskStatusType,
    pub started_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub resumed_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub done_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateWorkerTaskStatusDto {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub tenant_id: TenantId,
}

impl WorkerTaskStatus {
    pub fn aggregate_status(statuses: &[WorkerTaskStatus]) -> WorkerTaskStatusType {
        // Mindestens ein Worker arbeitet = started
        // Alle gestoppt/paused = deren Status
        // Mixed = started (weil jemand aktiv)
        for s in statuses {
            if s.status == WorkerTaskStatusType::Started || s.status == WorkerTaskStatusType::Paused
            {
                return WorkerTaskStatusType::Started; // Jemand arbeitet bereits
            }
        }
        // Wenn keiner läuft, höchstwertigen Status zurückgeben
        statuses
            .iter()
            .map(|s| s.status.clone())
            .max_by(|a, b| {
                let order = [
                    WorkerTaskStatusType::Done,
                    WorkerTaskStatusType::Stopped,
                    WorkerTaskStatusType::Paused,
                    WorkerTaskStatusType::Started,
                    WorkerTaskStatusType::New,
                ];
                let ia = order.iter().position(|x| x == a).unwrap_or(0);
                let ib = order.iter().position(|x| x == b).unwrap_or(0);
                ia.cmp(&ib)
            })
            .unwrap_or(WorkerTaskStatusType::New)
    }
}
