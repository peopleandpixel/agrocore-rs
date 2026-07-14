use agrocore_domain::entities::olive::{OliveOilRecord, CreateOliveOilRecordDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{OliveOilRecordRepo, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgOliveOilRecordRepo { pool: PgPool }
impl PgOliveOilRecordRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl OliveOilRecordRepo for PgOliveOilRecordRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<OliveOilRecord>> {
        Box::pin(async move { Ok(None) })
    }
    fn create(&self, tid: TenantId, _dto: CreateOliveOilRecordDto) -> RepositoryFuture<OliveOilRecord> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, OliveOilRecord>("INSERT INTO olive_oil_records (tenant_id, lot_id, extraction_date, quantity_liters, quality_grade) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(tid.to_string()).bind(None::<Uuid>).bind(chrono::Utc::now()).bind(0.0).bind(None::<String>)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(&self, _tid: TenantId, _id: Uuid, _dto: agrocore_domain::entities::olive::UpdateOliveOilRecordDto) -> RepositoryFuture<Option<OliveOilRecord>> {
        Box::pin(async move { Ok(None) })
    }
}