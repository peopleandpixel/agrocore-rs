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
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<WorkerLocation>> {
        Box::pin(async move { Ok(None) })
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
    fn find_all(&self, _tid: TenantId, _p: Pagination) -> RepositoryFuture<PaginatedResponse<agrocore_domain::entities::workforce::WorkerLocation>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
    fn find_latest_by_worker(&self, _tid: Uuid, _worker_id: Uuid) -> RepositoryFuture<Option<agrocore_domain::entities::workforce::WorkerLocation>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
    fn get_latest_locations(&self, _tid: Uuid) -> RepositoryFuture<Vec<agrocore_domain::entities::workforce::WorkerLocation>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
}