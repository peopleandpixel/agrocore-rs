use agrocore_domain::entities::workforce::{WorkerLocation, CreateWorkerLocationDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{WorkerLocationRepo, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
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
                "INSERT INTO worker_locations (tenant_id, worker_id, latitude, longitude, recorded_at) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(tid.to_string()).bind(dto.worker_id.to_string()).bind(dto.latitude).bind(dto.longitude).bind(dto.recorded_at)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}