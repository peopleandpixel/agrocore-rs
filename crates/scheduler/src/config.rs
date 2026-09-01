use crate::error::{SchedulerError, SchedulerResult};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub timezone: String,
    pub default_job_timeout_seconds: u64,
    pub max_concurrent_jobs: usize,
    pub retry_failed_jobs: bool,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timezone: "UTC".to_string(),
            default_job_timeout_seconds: 3600,
            max_concurrent_jobs: 10,
            retry_failed_jobs: true,
            max_retries: 3,
            retry_delay_seconds: 60,
        }
    }
}

impl SchedulerConfig {
    pub fn validate(&self) -> SchedulerResult<()> {
        if self.enabled {
            cron::Schedule::from_str(&self.timezone).map_err(|e| {
                SchedulerError::Config(format!("Invalid timezone cron expression: {e}"))
            })?;
        }

        if self.default_job_timeout_seconds == 0 {
            return Err(SchedulerError::Config(
                "default_job_timeout_seconds must be > 0".to_string(),
            ));
        }

        if self.max_concurrent_jobs == 0 {
            return Err(SchedulerError::Config(
                "max_concurrent_jobs must be > 0".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub schedule: String,
    pub timezone: Option<String>,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub timeout_seconds: Option<u64>,
    pub max_retries: Option<u32>,
    pub retry_delay_seconds: Option<u64>,
    pub enabled: bool,
    pub tags: Vec<String>,
}

impl JobDefinition {
    pub fn validate(&self) -> SchedulerResult<()> {
        if self.id.is_empty() {
            return Err(SchedulerError::Config("Job ID cannot be empty".to_string()));
        }

        if self.name.is_empty() {
            return Err(SchedulerError::Config(
                "Job name cannot be empty".to_string(),
            ));
        }

        // OneTime jobs don't need a cron schedule
        if !matches!(self.job_type, JobType::OneTime { .. }) {
            if self.schedule.is_empty() {
                return Err(SchedulerError::Config(
                    "Job schedule cannot be empty".to_string(),
                ));
            }

            let tz = self.timezone.as_deref().unwrap_or("UTC");
            cron::Schedule::from_str(&self.schedule).map_err(|e| {
                SchedulerError::Config(format!("Invalid schedule cron expression: {e}"))
            })?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobType {
    Builtin {
        handler: String,
    },
    Command {
        command: String,
        args: Vec<String>,
    },
    Http {
        url: String,
        method: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    },
    Nats {
        subject: String,
        payload: serde_json::Value,
    },
    OneTime {
        execute_at: chrono::DateTime<chrono::Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    pub id: String,
    pub name: String,
    pub status: JobRunStatus,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run: Option<chrono::DateTime<chrono::Utc>>,
    pub run_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_error: Option<String>,
    pub last_duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Disabled,
}
