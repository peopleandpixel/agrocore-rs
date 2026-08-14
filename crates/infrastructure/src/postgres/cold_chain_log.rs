use agrocore_domain::entities::harvest::{
    ColdChainLog, CreateColdChainLogDto, UpdateColdChainLogDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    ColdChainLogRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgColdChainLogRepo);

impl ColdChainLogRepo for PgColdChainLogRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<ColdChainLog>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, ColdChainLog>(
                "SELECT * FROM cold_chain_logs WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<ColdChainLog>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM cold_chain_logs WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<ColdChainLog> = sqlx::query_as("SELECT * FROM cold_chain_logs WHERE tenant_id = $1 ORDER BY recorded_at DESC LIMIT $2 OFFSET $3")
                .bind(tid)
                .bind(per_page as i32)
                .bind(offset as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

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

    fn find_by_lot(
        &self,
        tid: TenantId,
        lot_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<ColdChainLog>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM cold_chain_logs WHERE tenant_id = $1 AND lot_id = $2",
            )
            .bind(tid)
            .bind(lot_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<ColdChainLog> = sqlx::query_as("SELECT * FROM cold_chain_logs WHERE tenant_id = $1 AND lot_id = $2 ORDER BY recorded_at DESC LIMIT $3 OFFSET $4")
                .bind(tid)
                .bind(lot_id)
                .bind(per_page as i32)
                .bind(offset as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

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

    fn create(&self, tid: TenantId, dto: CreateColdChainLogDto) -> RepositoryFuture<ColdChainLog> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, ColdChainLog>(
                r#"INSERT INTO cold_chain_logs (id, tenant_id, lot_id, sensor_id, recorded_at, temperature_c, humidity_pct, location, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(dto.lot_id)
            .bind(&dto.sensor_id)
            .bind(dto.recorded_at)
            .bind(dto.temperature_c)
            .bind(dto.humidity_pct)
            .bind(&dto.location)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateColdChainLogDto,
    ) -> RepositoryFuture<Option<ColdChainLog>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, ColdChainLog>(
                r#"UPDATE cold_chain_logs SET 
                    lot_id = COALESCE($1, lot_id),
                    sensor_id = COALESCE($2, sensor_id),
                    recorded_at = COALESCE($3, recorded_at),
                    temperature_c = COALESCE($4, temperature_c),
                    humidity_pct = COALESCE($5, humidity_pct),
                    location = COALESCE($6, location)
                   WHERE id = $7 AND tenant_id = $8 RETURNING *"#,
            )
            .bind(dto.lot_id)
            .bind(&dto.sensor_id)
            .bind(dto.recorded_at)
            .bind(dto.temperature_c)
            .bind(dto.humidity_pct)
            .bind(&dto.location)
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM cold_chain_logs WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
