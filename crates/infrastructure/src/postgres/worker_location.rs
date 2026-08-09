use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::workforce::{CreateWorkerLocationDto, WorkerLocation};
use agrocore_domain::repositories::{RepositoryFuture, WorkerLocationRepo};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWorkerLocationRepo {
    pool: PgPool,
}
impl PgWorkerLocationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl WorkerLocationRepo for PgWorkerLocationRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WorkerLocation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkerLocation>(
                "SELECT id, tenant_id, worker_id, ST_X(location) as lng, ST_Y(location) as lat, timestamp FROM worker_locations WHERE tenant_id = $1 AND id = $2"
            )
            .bind(tid).bind(id)
            .fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        _roles: &[agrocore_domain::entities::user::UserRole],
    ) -> RepositoryFuture<Option<WorkerLocation>> {
        self.find_by_id(tid, id)
    }
    fn create(
        &self,
        tid: TenantId,
        dto: CreateWorkerLocationDto,
    ) -> RepositoryFuture<WorkerLocation> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkerLocation>(
                "INSERT INTO worker_locations (id, tenant_id, worker_id, location, timestamp) VALUES ($1, $2, $3, ST_SetSRID(ST_MakePoint($4, $5), 4326), $6) RETURNING id, tenant_id, worker_id, ST_X(location) as lng, ST_Y(location) as lat, timestamp"
            )
            .bind(Uuid::new_v4()).bind(tid).bind(dto.worker_id).bind(dto.lng).bind(dto.lat).bind(dto.timestamp)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WorkerLocation>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM worker_locations WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<WorkerLocation> = sqlx::query_as(
                "SELECT id, tenant_id, worker_id, ST_X(location) as lng, ST_Y(location) as lat, timestamp FROM worker_locations WHERE tenant_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3"
            )
                .bind(tid).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 {
                0
            } else {
                (total as f64 / per_page as f64).ceil() as u64
            };
            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }
    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        _user_id: Uuid,
        _roles: &[agrocore_domain::entities::user::UserRole],
    ) -> RepositoryFuture<PaginatedResponse<WorkerLocation>> {
        self.find_all(tid, p)
    }
    fn find_latest_by_worker(
        &self,
        tid: TenantId,
        worker_id: Uuid,
    ) -> RepositoryFuture<Option<WorkerLocation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkerLocation>(
                "SELECT id, tenant_id, worker_id, ST_X(location) as lng, ST_Y(location) as lat, timestamp FROM worker_locations WHERE tenant_id = $1 AND worker_id = $2 ORDER BY timestamp DESC LIMIT 1"
            )
            .bind(tid).bind(worker_id)
            .fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn get_latest_locations(&self, tid: TenantId) -> RepositoryFuture<Vec<WorkerLocation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkerLocation>(
                "SELECT DISTINCT ON (worker_id) id, tenant_id, worker_id, ST_X(location) as lng, ST_Y(location) as lat, timestamp FROM worker_locations WHERE tenant_id = $1 ORDER BY worker_id, timestamp DESC"
            )
            .bind(tid)
            .fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
