use agrocore_domain::entities::harvest::{ColdChainLog, CreateColdChainLogDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{ColdChainLogRepo, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgColdChainLogRepo { pool: PgPool }
impl PgColdChainLogRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl ColdChainLogRepo for PgColdChainLogRepo {
    fn create(&self, tid: TenantId, dto: CreateColdChainLogDto) -> RepositoryFuture<ColdChainLog> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, ColdChainLog>(
                "INSERT INTO cold_chain_logs (tenant_id, lot_id, sensor_id, recorded_at, temperature_c, humidity_pct, location) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *")
            .bind(tid.to_string()).bind(dto.lot_id.to_string()).bind(&dto.sensor_id).bind(dto.recorded_at).bind(dto.temperature_c).bind(dto.humidity_pct).bind(&dto.location)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}