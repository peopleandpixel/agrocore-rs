use agrocore_domain::entities::workforce::{WorkerLocation, CreateWorkerLocationDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{WorkerLocationRepo, RepositoryFuture};
use agrocore_shared::{SharedError, Pagination, PaginatedResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWorkerLocationRepo { pool: PgPool }
impl PgWorkerLocationRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl WorkerLocationRepo for PgWorkerLocationRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WorkerLocation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkerLocation>(
                "SELECT id, tenant_id, worker_id, ST_X(location) as lng, ST_Y(location) as lat, timestamp FROM worker_locations WHERE tenant_id = $1::uuid AND id = $2")
            .bind(tid.to_string()).bind(id)
            .fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn create(&self, tid: TenantId, dto: CreateWorkerLocationDto) -> RepositoryFuture<WorkerLocation> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkerLocation>(
                "INSERT INTO worker_locations (id, tenant_id, worker_id, location, timestamp) VALUES ($1, $2, $3, ST_SetSRID(ST_MakePoint($4, $5), 4326), $6) RETURNING id, tenant_id, worker_id, ST_X(location) as lng, ST_Y(location) as lat, timestamp")
            .bind(Uuid::new_v4()).bind(tid.to_string()).bind(dto.worker_id.to_string()).bind(dto.lng).bind(dto.lat).bind(dto.timestamp)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WorkerLocation>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM worker_locations WHERE tenant_id = $1::uuid")
                .bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<WorkerLocation> = sqlx::query_as(
                "SELECT id, tenant_id, worker_id, ST_X(location) as lng, ST_Y(location) as lat, timestamp FROM worker_locations WHERE tenant_id = $1::uuid ORDER BY timestamp DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
    fn find_latest_by_worker(&self, tid: Uuid, worker_id: Uuid) -> RepositoryFuture<Option<WorkerLocation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkerLocation>(
                "SELECT id, tenant_id, worker_id, ST_X(location) as lng, ST_Y(location) as lat, timestamp FROM worker_locations WHERE tenant_id = $1::uuid AND worker_id = $2 ORDER BY timestamp DESC LIMIT 1")
            .bind(tid.to_string()).bind(worker_id.to_string())
            .fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn get_latest_locations(&self, tid: Uuid) -> RepositoryFuture<Vec<WorkerLocation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkerLocation>(
                "SELECT DISTINCT ON (worker_id) id, tenant_id, worker_id, ST_X(location) as lng, ST_Y(location) as lat, timestamp FROM worker_locations WHERE tenant_id = $1::uuid ORDER BY worker_id, timestamp DESC")
            .bind(tid.to_string())
            .fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}