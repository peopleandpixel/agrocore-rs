use agrocore_domain::entities::workforce::{WorkLog, CreateWorkLogDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{WorkLogRepo, RepositoryFuture, PaginatedResponse, Pagination};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWorkLogRepo { pool: PgPool }
impl PgWorkLogRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl WorkLogRepo for PgWorkLogRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<WorkLog>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WorkLog>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_logs WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true)").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<WorkLog> = sqlx::query_as("SELECT * FROM work_logs WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true) ORDER BY date DESC LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
    fn create(&self, tid: TenantId, dto: CreateWorkLogDto, _by: Uuid) -> RepositoryFuture<WorkLog> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkLog>(
                "INSERT INTO work_logs (tenant_id, worker_id, task_id, date, hours, description) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
            .bind(tid.to_string()).bind(dto.worker_id.to_string()).bind(None::<Uuid>).bind(dto.date).bind(dto.hours_worked).bind(&dto.task_description)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(&self, _tid: TenantId, _id: Uuid, _dto: agrocore_domain::entities::workforce::UpdateWorkLogDto, _by: Uuid) -> RepositoryFuture<Option<WorkLog>> {
        Box::pin(async move { Ok(None) })
    }
    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Ok(false) })
    }
    fn find_by_worker(&self, _tid: TenantId, _worker_id: Uuid, _p: Pagination) -> RepositoryFuture<PaginatedResponse<agrocore_domain::entities::workforce::WorkLog>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
}