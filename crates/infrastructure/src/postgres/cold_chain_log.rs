use agrocore_domain::entities::coldchain::{ColdChainLog, CreateColdChainLogDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{ColdChainLogRepo, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgColdChainLogRepo { pool: PgPool }
impl PgColdChainLogRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl ColdChainLogRepo for PgColdChainLogRepo {
    fn create(&self, tid: TenantId, dto: CreateColdChainLogDto) -> RepositoryFuture<ColdChainLog> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, ColdChainLog>(
                "INSERT INTO cold_chain_logs (tenant_id, delivery_id, temperature_c, timestamp, notes) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(tid.to_string()).bind(dto.delivery_id.to_string()).bind(dto.temperature_c).bind(dto.timestamp).bind(&dto.notes)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}