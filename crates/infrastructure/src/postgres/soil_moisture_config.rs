use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::weather::{CreateSoilMoistureConfigDto, SoilMoistureConfig};
use agrocore_domain::repositories::{RepositoryFuture, SoilMoistureConfigRepo};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgSoilMoistureConfigRepo);

impl SoilMoistureConfigRepo for PgSoilMoistureConfigRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<SoilMoistureConfig>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, SoilMoistureConfig>(
                "SELECT * FROM soil_moisture_configs WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_by_station(
        &self,
        tid: TenantId,
        station_id: Uuid,
    ) -> RepositoryFuture<Vec<SoilMoistureConfig>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, SoilMoistureConfig>(
                "SELECT * FROM soil_moisture_configs WHERE tenant_id = $1 AND station_id = $2 AND is_active = true",
            )
            .bind(tid)
            .bind(station_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<SoilMoistureConfig>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM soil_moisture_configs WHERE tenant_id = $1 AND site_id = $2",
            )
            .bind(tid)
            .bind(site_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<SoilMoistureConfig> = sqlx::query_as(
                "SELECT * FROM soil_moisture_configs WHERE tenant_id = $1 AND site_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(tid)
            .bind(site_id)
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

    fn create(
        &self,
        tid: TenantId,
        dto: CreateSoilMoistureConfigDto,
    ) -> RepositoryFuture<SoilMoistureConfig> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, SoilMoistureConfig>(
                r#"INSERT INTO soil_moisture_configs (
                    id, tenant_id, station_id, site_id,
                    moisture_threshold_percent, min_interval_minutes,
                    irrigation_duration_minutes, notify_email, notify_sms,
                    is_active, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
                RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(dto.station_id)
            .bind(dto.site_id)
            .bind(dto.moisture_threshold_percent)
            .bind(dto.min_interval_minutes)
            .bind(dto.irrigation_duration_minutes)
            .bind(dto.notify_email)
            .bind(dto.notify_sms)
            .bind(dto.is_active)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: CreateSoilMoistureConfigDto,
    ) -> RepositoryFuture<Option<SoilMoistureConfig>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, SoilMoistureConfig>(
                r#"UPDATE soil_moisture_configs SET
                    station_id = $3, site_id = $4,
                    moisture_threshold_percent = $5, min_interval_minutes = $6,
                    irrigation_duration_minutes = $7, notify_email = $8, notify_sms = $9,
                    is_active = $10, updated_at = NOW()
                WHERE id = $1 AND tenant_id = $2
                RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(dto.station_id)
            .bind(dto.site_id)
            .bind(dto.moisture_threshold_percent)
            .bind(dto.min_interval_minutes)
            .bind(dto.irrigation_duration_minutes)
            .bind(dto.notify_email)
            .bind(dto.notify_sms)
            .bind(dto.is_active)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM soil_moisture_configs WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
