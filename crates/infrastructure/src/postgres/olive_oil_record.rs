use agrocore_domain::entities::olive::OliveOilRecord;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{OliveOilRecordRepository, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgOliveOilRecordRepo {
    pool: PgPool,
}

impl PgOliveOilRecordRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl OliveOilRecordRepository for PgOliveOilRecordRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<OliveOilRecord>> {
        Box::pin(async move { Ok(None) })
    }
    fn create(&self, _tid: TenantId, _dto: agrocore_domain::entities::olive::CreateOliveOilRecordDto) -> RepositoryFuture<OliveOilRecord> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".to_string()).to_error()) })
    }
    fn update(&self, _tid: TenantId, _id: Uuid, _dto: agrocore_domain::entities::olive::UpdateOliveOilRecordDto) -> RepositoryFuture<Option<OliveOilRecord>> {
        Box::pin(async move { Ok(None) })
    }
}