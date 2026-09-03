use crate::config::{JobDefinition, JobRunStatus, JobStatus, JobType, SchedulerConfig};
use crate::error::{SchedulerError, SchedulerResult};
use agrocore_logging::{debug, error, info, warn};
use async_nats::Client as NatsClient;
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use cron::Schedule;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

pub type JobHandler = Arc<
    dyn Fn(
            JobDefinition,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = SchedulerResult<()>> + Send>>
        + Send
        + Sync,
>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub definition: JobDefinition,
    pub uuid: Uuid,
    pub status: JobRunStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct SchedulerService {
    config: SchedulerConfig,
    scheduler: Arc<JobScheduler>,
    jobs: Arc<RwLock<HashMap<String, ScheduledJob>>>,
    handlers: Arc<RwLock<HashMap<String, JobHandler>>>,
    nats: Option<NatsClient>,
    job_state: Arc<RwLock<HashMap<String, JobStatus>>>,
}

impl SchedulerService {
    pub async fn new(config: SchedulerConfig, nats: Option<NatsClient>) -> SchedulerResult<Self> {
        config.validate()?;

        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| SchedulerError::JobScheduler(e))?;

        let scheduler = Arc::new(scheduler);

        Ok(Self {
            config,
            scheduler,
            jobs: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            nats,
            job_state: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start(&self) -> SchedulerResult<()> {
        if !self.config.enabled {
            info!("Scheduler is disabled, not starting");
            return Ok(());
        }

        self.scheduler
            .start()
            .await
            .map_err(|e| SchedulerError::JobScheduler(e))?;

        info!("Scheduler started");
        Ok(())
    }

    pub async fn stop(&self) -> SchedulerResult<()> {
        // JobScheduler shutdown takes &mut self, so we need to use Arc::get_mut
        // Since we can't mutate through Arc, we'll just stop accepting new jobs
        // and let the scheduler finish current jobs
        // For now, we'll just return Ok as the scheduler doesn't have a proper shutdown
        info!("Scheduler stop requested (graceful shutdown not fully implemented)");
        Ok(())
    }

    pub async fn register_handler<F, Fut>(&self, name: &str, handler: F)
    where
        F: Fn(JobDefinition) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = SchedulerResult<()>> + Send + 'static,
    {
        let handler: JobHandler = Arc::new(move |def| Box::pin(handler(def)));
        let mut handlers = self.handlers.write().await;
        handlers.insert(name.to_string(), handler);
        info!("Registered handler: {}", name);
    }

    pub async fn add_job(&self, job_def: JobDefinition) -> SchedulerResult<()> {
        job_def.validate()?;

        // Check if job already exists
        {
            let jobs = self.jobs.read().await;
            if jobs.contains_key(&job_def.id) {
                return Err(SchedulerError::InvalidState(format!(
                    "Job with id '{}' already exists",
                    job_def.id
                )));
            }
        }

        // Check if handler exists
        let handler_name = match &job_def.job_type {
            JobType::Builtin { handler } => handler.clone(),
            JobType::OneTime { .. } => "onetime".to_string(),
            _ => "default".to_string(),
        };

        let handler_exists = {
            let handlers = self.handlers.read().await;
            handlers.contains_key(&handler_name)
        };

        if !handler_exists
            && !matches!(job_def.job_type, JobType::Builtin { .. })
            && !matches!(job_def.job_type, JobType::OneTime { .. })
        {
            // Allow non-builtin jobs without handlers for now
        }

        let uuid = Uuid::new_v4();
        let now = Utc::now();

        // Handle OneTime jobs differently
        if let JobType::OneTime { execute_at } = &job_def.job_type {
            let execute_at = *execute_at;
            return self.add_onetime_job(job_def, uuid, now, execute_at).await;
        }

        // Calculate next run for recurring jobs
        let schedule = <cron::Schedule as std::str::FromStr>::from_str(&job_def.schedule)
            .map_err(|e| SchedulerError::Cron(e))?;
        let next_run = schedule.upcoming(Utc).next();

        let scheduled_job = ScheduledJob {
            definition: job_def.clone(),
            uuid,
            status: if job_def.enabled {
                JobRunStatus::Pending
            } else {
                JobRunStatus::Disabled
            },
            created_at: now,
            updated_at: now,
        };

        // Store job
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_def.id.clone(), scheduled_job);
        }

        // Initialize job status
        {
            let mut state = self.job_state.write().await;
            state.insert(
                job_def.id.clone(),
                JobStatus {
                    id: job_def.id.clone(),
                    name: job_def.name.clone(),
                    status: if job_def.enabled {
                        JobRunStatus::Pending
                    } else {
                        JobRunStatus::Disabled
                    },
                    last_run: None,
                    next_run,
                    run_count: 0,
                    success_count: 0,
                    failure_count: 0,
                    last_error: None,
                    last_duration_ms: None,
                },
            );
        }

        // Create and add the job to scheduler
        if job_def.enabled {
            let service = self.clone_for_job();
            let job_id = job_def.id.clone();
            let schedule_str = job_def.schedule.clone();
            let timeout = job_def
                .timeout_seconds
                .unwrap_or(self.config.default_job_timeout_seconds);
            let max_retries = job_def.max_retries.unwrap_or(self.config.max_retries);
            let retry_delay = job_def
                .retry_delay_seconds
                .unwrap_or(self.config.retry_delay_seconds);

            let job = Job::new_async(
                <cron::Schedule as std::str::FromStr>::from_str(&schedule_str)
                    .map_err(|e| SchedulerError::Cron(e))?,
                move |_uuid, _lock| {
                    let service = service.clone();
                    let job_id = job_id.clone();
                    Box::pin(async move {
                        service
                            .execute_job(&job_id, timeout, max_retries, retry_delay)
                            .await;
                    })
                },
            )
            .map_err(|e| SchedulerError::JobScheduler(e))?;

            self.scheduler
                .add(job)
                .await
                .map_err(|e| SchedulerError::JobScheduler(e))?;
        }

        info!("Added job: {} ({})", job_def.name, job_def.id);
        Ok(())
    }

    fn clone_for_job(&self) -> Self {
        Self {
            config: self.config.clone(),
            scheduler: self.scheduler.clone(),
            jobs: self.jobs.clone(),
            handlers: self.handlers.clone(),
            nats: self.nats.clone(),
            job_state: self.job_state.clone(),
        }
    }

    async fn execute_job(&self, job_id: &str, timeout: u64, max_retries: u32, retry_delay: u64) {
        let job_def = {
            let jobs = self.jobs.read().await;
            jobs.get(job_id).map(|j| j.definition.clone())
        };

        let Some(job_def) = job_def else {
            error!("Job not found: {}", job_id);
            return;
        };

        if !job_def.enabled {
            debug!("Job {} is disabled, skipping", job_id);
            return;
        }

        let start_time = std::time::Instant::now();

        // Update status to running
        self.update_job_status(job_id, JobRunStatus::Running, None)
            .await;

        // Publish started event
        if let Some(nats) = &self.nats {
            self.publish_event(nats, "job.started", job_id).await;
        }

        let mut retries = 0;
        let mut last_error = None;

        while retries <= max_retries {
            if retries > 0 {
                info!(
                    "Retrying job {} (attempt {}/{})",
                    job_id, retries, max_retries
                );
                tokio::time::sleep(Duration::from_secs(retry_delay)).await;
            }

            let result =
                tokio::time::timeout(Duration::from_secs(timeout), self.run_job_handler(&job_def))
                    .await;

            match result {
                Ok(Ok(_)) => {
                    // Success
                    let duration = start_time.elapsed().as_millis() as u64;
                    self.update_job_status(job_id, JobRunStatus::Completed, Some(duration))
                        .await;

                    if let Some(nats) = &self.nats {
                        self.publish_event(nats, "job.completed", job_id).await;
                    }

                    info!("Job {} completed successfully in {}ms", job_id, duration);
                    return;
                }
                Ok(Err(e)) => {
                    last_error = Some(e.to_string());
                    error!("Job {} failed: {}", job_id, last_error.as_ref().unwrap());
                }
                Err(_) => {
                    last_error = Some(format!("Job timed out after {} seconds", timeout));
                    error!("Job {} timed out", job_id);
                }
            }

            retries += 1;
        }

        // All retries exhausted
        let duration = start_time.elapsed().as_millis() as u64;
        self.update_job_status(job_id, JobRunStatus::Failed, Some(duration))
            .await;

        if let Some(nats) = &self.nats {
            self.publish_event(nats, "job.failed", job_id).await;
        }
    }

    async fn run_job_handler(&self, job_def: &JobDefinition) -> SchedulerResult<()> {
        let handler_name = match &job_def.job_type {
            JobType::Builtin { handler } => handler.clone(),
            JobType::OneTime { .. } => "onetime".to_string(),
            _ => "default".to_string(),
        };

        let handler = {
            let handlers = self.handlers.read().await;
            handlers.get(&handler_name).cloned()
        };

        if let Some(handler) = handler {
            handler(job_def.clone()).await
        } else {
            // Default handler for non-builtin jobs
            self.default_job_handler(job_def).await
        }
    }

    async fn default_job_handler(&self, job_def: &JobDefinition) -> SchedulerResult<()> {
        match &job_def.job_type {
            JobType::Command { command, args } => {
                let output = Command::new(command)
                    .args(args)
                    .output()
                    .await
                    .map_err(|e| SchedulerError::Io(e))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(SchedulerError::InvalidState(format!(
                        "Command failed: {}",
                        stderr
                    )));
                }
                Ok(())
            }
            JobType::Http {
                url,
                method,
                headers,
                body,
            } => {
                let client = reqwest::Client::new();
                let mut req = client.request(method.parse().unwrap_or(reqwest::Method::POST), url);

                for (k, v) in headers {
                    req = req.header(k, v);
                }

                if let Some(body) = body {
                    req = req.body(body.to_string());
                }

                let response = req
                    .send()
                    .await
                    .map_err(|e| SchedulerError::Anyhow(e.into()))?;

                if !response.status().is_success() {
                    return Err(SchedulerError::InvalidState(format!(
                        "HTTP request failed: {}",
                        response.status()
                    )));
                }
                Ok(())
            }
            JobType::Nats { subject, payload } => {
                if let Some(nats) = &self.nats {
                    let data = serde_json::to_vec(payload)
                        .map_err(|e| SchedulerError::Serialization(e))?;
                    nats.publish(subject.to_string(), Bytes::from(data))
                        .await
                        .map_err(|e| SchedulerError::Nats(e.to_string()))?;
                }
                Ok(())
            }
            JobType::OneTime { .. } => {
                // OneTime jobs are handled by the scheduler directly, not by handlers
                Ok(())
            }
            JobType::Builtin { handler } => Err(SchedulerError::InvalidState(format!(
                "Builtin handler '{}' not registered",
                handler
            ))),
        }
    }

    async fn update_job_status(
        &self,
        job_id: &str,
        status: JobRunStatus,
        duration_ms: Option<u64>,
    ) {
        let mut state = self.job_state.write().await;
        if let Some(job_status) = state.get_mut(job_id) {
            job_status.status = status.clone();
            job_status.last_run = Some(Utc::now());
            job_status.run_count += 1;
            job_status.last_duration_ms = duration_ms;

            match status {
                JobRunStatus::Completed => {
                    job_status.success_count += 1;
                    job_status.last_error = None;
                }
                JobRunStatus::Failed => {
                    job_status.failure_count += 1;
                }
                _ => {}
            }

            // Calculate next run
            if let Some(job) = self.jobs.read().await.get(job_id) {
                let schedule =
                    <cron::Schedule as std::str::FromStr>::from_str(&job.definition.schedule).ok();
                if let Some(schedule) = schedule {
                    job_status.next_run = schedule.upcoming(Utc).next();
                }
            }
        }
    }

    async fn publish_event(&self, nats: &NatsClient, event_type: &str, job_id: &str) {
        let event = serde_json::json!({
            "event": event_type,
            "job_id": job_id,
            "timestamp": Utc::now(),
        });

        let subject = format!("scheduler.{}", event_type);
        let payload = Bytes::from(serde_json::to_vec(&event).unwrap_or_default());

        if let Err(e) = nats.publish(subject.to_string(), payload).await {
            warn!("Failed to publish NATS event: {}", e);
        }
    }

    pub async fn remove_job(&self, job_id: &str) -> SchedulerResult<()> {
        // Remove from scheduler
        let uuid = {
            let jobs = self.jobs.read().await;
            jobs.get(job_id).map(|j| j.uuid)
        };

        if let Some(uuid) = uuid {
            self.scheduler
                .remove(&uuid)
                .await
                .map_err(|e| SchedulerError::JobScheduler(e))?;
        }

        // Remove from local state
        {
            let mut jobs = self.jobs.write().await;
            jobs.remove(job_id);
        }

        {
            let mut state = self.job_state.write().await;
            state.remove(job_id);
        }

        info!("Removed job: {}", job_id);
        Ok(())
    }

    pub async fn enable_job(&self, job_id: &str) -> SchedulerResult<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(job_id) {
            if !job.definition.enabled {
                job.definition.enabled = true;
                job.status = JobRunStatus::Pending;
                job.updated_at = Utc::now();

                // Re-add to scheduler
                let job_def = job.definition.clone();
                let service = self.clone_for_job();
                let timeout = job_def
                    .timeout_seconds
                    .unwrap_or(self.config.default_job_timeout_seconds);
                let max_retries = job_def.max_retries.unwrap_or(self.config.max_retries);
                let retry_delay = job_def
                    .retry_delay_seconds
                    .unwrap_or(self.config.retry_delay_seconds);

                let job = Job::new_async(
                    <cron::Schedule as std::str::FromStr>::from_str(&job_def.schedule)
                        .map_err(|e| SchedulerError::Cron(e))?,
                    move |_uuid, _lock| {
                        let service = service.clone();
                        let job_id = job_def.id.clone();
                        Box::pin(async move {
                            service
                                .execute_job(&job_id, timeout, max_retries, retry_delay)
                                .await;
                        })
                    },
                )
                .map_err(|e| SchedulerError::JobScheduler(e))?;

                self.scheduler
                    .add(job)
                    .await
                    .map_err(|e| SchedulerError::JobScheduler(e))?;

                // Update status
                let mut state = self.job_state.write().await;
                if let Some(s) = state.get_mut(job_id) {
                    s.status = JobRunStatus::Pending;
                }

                info!("Enabled job: {}", job_id);
            }
        }
        Ok(())
    }

    pub async fn disable_job(&self, job_id: &str) -> SchedulerResult<()> {
        let uuid = {
            let jobs = self.jobs.read().await;
            jobs.get(job_id).map(|j| j.uuid)
        };

        if let Some(uuid) = uuid {
            self.scheduler
                .remove(&uuid)
                .await
                .map_err(|e| SchedulerError::JobScheduler(e))?;
        }

        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.definition.enabled = false;
                job.status = JobRunStatus::Disabled;
                job.updated_at = Utc::now();
            }
        }

        {
            let mut state = self.job_state.write().await;
            if let Some(s) = state.get_mut(job_id) {
                s.status = JobRunStatus::Disabled;
            }
        }

        info!("Disabled job: {}", job_id);
        Ok(())
    }

    pub async fn trigger_job(&self, job_id: &str) -> SchedulerResult<()> {
        let job_def = {
            let jobs = self.jobs.read().await;
            jobs.get(job_id).map(|j| j.definition.clone())
        };

        let Some(job_def) = job_def else {
            return Err(SchedulerError::JobNotFound(job_id.to_string()));
        };

        if !job_def.enabled {
            return Err(SchedulerError::InvalidState(format!(
                "Job {} is disabled",
                job_id
            )));
        }

        let timeout = job_def
            .timeout_seconds
            .unwrap_or(self.config.default_job_timeout_seconds);
        let max_retries = job_def.max_retries.unwrap_or(self.config.max_retries);
        let retry_delay = job_def
            .retry_delay_seconds
            .unwrap_or(self.config.retry_delay_seconds);

        self.execute_job(job_id, timeout, max_retries, retry_delay)
            .await;
        Ok(())
    }

    pub async fn get_job_status(&self, job_id: &str) -> Option<JobStatus> {
        let state = self.job_state.read().await;
        state.get(job_id).cloned()
    }

    pub async fn list_jobs(&self) -> Vec<JobStatus> {
        let state = self.job_state.read().await;
        state.values().cloned().collect()
    }

    pub async fn get_job_definition(&self, job_id: &str) -> Option<JobDefinition> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).map(|j| j.definition.clone())
    }

    async fn add_onetime_job(
        &self,
        job_def: JobDefinition,
        uuid: Uuid,
        now: DateTime<Utc>,
        execute_at: DateTime<Utc>,
    ) -> SchedulerResult<()> {
        // Store job
        let scheduled_job = ScheduledJob {
            definition: job_def.clone(),
            uuid,
            status: if job_def.enabled {
                JobRunStatus::Pending
            } else {
                JobRunStatus::Disabled
            },
            created_at: now,
            updated_at: now,
        };

        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_def.id.clone(), scheduled_job);
        }

        // Initialize job status
        {
            let mut state = self.job_state.write().await;
            state.insert(
                job_def.id.clone(),
                JobStatus {
                    id: job_def.id.clone(),
                    name: job_def.name.clone(),
                    status: if job_def.enabled {
                        JobRunStatus::Pending
                    } else {
                        JobRunStatus::Disabled
                    },
                    last_run: None,
                    next_run: Some(execute_at),
                    run_count: 0,
                    success_count: 0,
                    failure_count: 0,
                    last_error: None,
                    last_duration_ms: None,
                },
            );
        }

        // Schedule one-time execution using tokio::time::sleep_until
        if job_def.enabled {
            let service = self.clone_for_job();
            let job_id = job_def.id.clone();
            let timeout = job_def
                .timeout_seconds
                .unwrap_or(self.config.default_job_timeout_seconds);
            let max_retries = job_def.max_retries.unwrap_or(self.config.max_retries);
            let retry_delay = job_def
                .retry_delay_seconds
                .unwrap_or(self.config.retry_delay_seconds);

            // Calculate delay until execution
            let now_utc = Utc::now();
            let delay = if execute_at > now_utc {
                (execute_at - now_utc)
                    .to_std()
                    .unwrap_or(std::time::Duration::from_secs(0))
            } else {
                std::time::Duration::from_secs(0)
            };

            tokio::spawn(async move {
                tokio::time::sleep(delay).await;
                service
                    .execute_job(&job_id, timeout, max_retries, retry_delay)
                    .await;
            });
        }

        info!(
            "Added one-time job: {} ({}) at {}",
            job_def.name, job_def.id, execute_at
        );
        Ok(())
    }
}

impl Clone for SchedulerService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            scheduler: self.scheduler.clone(),
            jobs: self.jobs.clone(),
            handlers: self.handlers.clone(),
            nats: self.nats.clone(),
            job_state: self.job_state.clone(),
        }
    }
}
