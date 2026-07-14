use agrocore_domain::entities::workforce::WorkerLocation;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{WorkerLocationRepo, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWorkerLocationRepo { pool: PgPool, }
impl PgWorkerLocationRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl WorkerLocationRepo for PgWorkerLocationRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<WorkerLocation>> {
        Box::pin(async move { Ok(None) })
    }
    fn create(&self, tid: TenantId, _dto: agrocore_domain::entities::workforce::CreateWorkerLocationDto) -> RepositoryFuture<WorkerLocation> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".to_string()).to_error()) })
    }
}