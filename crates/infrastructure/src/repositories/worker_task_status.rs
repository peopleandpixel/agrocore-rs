use crate::repositories::base::MongoRepository;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::worker_task_status::{
    CreateWorkerTaskStatusDto, WorkerTaskStatus, WorkerTaskStatusType,
};
use agrocore_domain::repositories::{RepositoryFuture, WorkerTaskStatusRepository};
use agrocore_shared::SharedError;
use chrono::Utc;
use mongodb::Collection;
use mongodb::bson::{Document, doc};
use uuid::Uuid;

#[derive(Clone)]
pub struct WorkerTaskStatusRepo {
    base: MongoRepository<WorkerTaskStatus>,
}

impl WorkerTaskStatusRepo {
    pub fn new(c: Collection<WorkerTaskStatus>) -> Self {
        Self {
            base: MongoRepository::new(c),
        }
    }
}

impl WorkerTaskStatusRepository for WorkerTaskStatusRepo {
    fn find_by_task_and_worker(
        &self,
        tid: TenantId,
        task_id: Uuid,
        worker_id: Uuid,
    ) -> RepositoryFuture<Option<WorkerTaskStatus>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            c.find_one(doc! { "tenant_id": tid.to_string(), "task_id": task_id.to_string(), "worker_id": worker_id.to_string() })
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all_for_task(
        &self,
        tid: TenantId,
        task_id: Uuid,
    ) -> RepositoryFuture<Vec<WorkerTaskStatus>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            use futures::stream::StreamExt;
            let mut cursor = c
                .find(doc! { "tenant_id": tid.to_string(), "task_id": task_id.to_string() })
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let mut results = Vec::new();
            while let Some(result) = cursor.next().await {
                results.push(result.map_err(|e| SharedError::Database(e.to_string()))?);
            }
            Ok(results)
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateWorkerTaskStatusDto,
    ) -> RepositoryFuture<WorkerTaskStatus> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let status = WorkerTaskStatus {
                task_id: dto.task_id,
                worker_id: dto.worker_id,
                tenant_id: tid,
                status: WorkerTaskStatusType::New,
                started_at: None,
                paused_at: None,
                resumed_at: None,
                stopped_at: None,
                done_at: None,
                created_at: now,
                updated_at: now,
            };
            c.insert_one(&status)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(status)
        })
    }

    fn update_status(
        &self,
        tid: TenantId,
        task_id: Uuid,
        worker_id: Uuid,
        status: WorkerTaskStatusType,
    ) -> RepositoryFuture<Option<WorkerTaskStatus>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let mut update = Document::new();
            update.insert("status", status.to_string());
            update.insert("updated_at", Utc::now());

            match status {
                WorkerTaskStatusType::Started => {
                    update.insert("started_at", Utc::now());
                }
                WorkerTaskStatusType::Paused => {
                    update.insert("paused_at", Utc::now());
                }
                WorkerTaskStatusType::Stopped => {
                    update.insert("stopped_at", Utc::now());
                }
                WorkerTaskStatusType::Done => {
                    update.insert("done_at", Utc::now());
                }
                _ => {}
            }

            c.update_one(
                doc! { "tenant_id": tid.to_string(), "task_id": task_id.to_string(), "worker_id": worker_id.to_string() },
                doc! { "$set": update },
            )
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            c.find_one(doc! { "tenant_id": tid.to_string(), "task_id": task_id.to_string(), "worker_id": worker_id.to_string() })
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
