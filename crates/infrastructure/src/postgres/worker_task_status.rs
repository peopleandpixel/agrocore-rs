use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::worker_task_status::{
    CreateWorkerTaskStatusDto, WorkerTaskStatus, WorkerTaskStatusType,
};
use agrocore_domain::repositories::{RepositoryFuture, WorkerTaskStatusRepository};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgWorkerTaskStatusRepo);

impl WorkerTaskStatusRepository for PgWorkerTaskStatusRepo {
    fn find_by_task_and_worker(
        &self,
        tid: TenantId,
        task_id: Uuid,
        worker_id: Uuid,
    ) -> RepositoryFuture<Option<WorkerTaskStatus>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkerTaskStatus>("SELECT * FROM worker_task_statuses WHERE task_id = $1 AND worker_id = $2 AND tenant_id = $3")
                .bind(task_id)
                .bind(worker_id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all_for_task(
        &self,
        tid: TenantId,
        task_id: Uuid,
    ) -> RepositoryFuture<Vec<WorkerTaskStatus>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkerTaskStatus>(
                "SELECT * FROM worker_task_statuses WHERE task_id = $1 AND tenant_id = $2",
            )
            .bind(task_id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateWorkerTaskStatusDto,
    ) -> RepositoryFuture<WorkerTaskStatus> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, WorkerTaskStatus>(
                r#"INSERT INTO worker_task_statuses (id, tenant_id, task_id, worker_id, status, updated_at)
                   VALUES ($1, $2, $3, $4, $5, NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(dto.task_id)
            .bind(dto.worker_id)
            .bind(serde_json::to_value(&WorkerTaskStatusType::New).unwrap())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update_status(
        &self,
        tid: TenantId,
        task_id: Uuid,
        worker_id: Uuid,
        status: WorkerTaskStatusType,
    ) -> RepositoryFuture<Option<WorkerTaskStatus>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkerTaskStatus>(
                r#"UPDATE worker_task_statuses SET status = $1, updated_at = NOW()
                   WHERE task_id = $2 AND worker_id = $3 AND tenant_id = $4
                   RETURNING *"#,
            )
            .bind(serde_json::to_value(&status).unwrap())
            .bind(task_id)
            .bind(worker_id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
