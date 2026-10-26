use crate::postgres::tenant_pool::TenantPool;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::weather::SoilMoistureReading;
use agrocore_domain::repositories::{RepositoryFuture, SoilMoistureReadingRepo};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgSoilMoistureReadingRepo);

impl SoilMoistureReadingRepo for PgSoilMoistureReadingRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<SoilMoistureReading>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            sqlx::query_as::<_, SoilMoistureReading>(
                "SELECT * FROM soil_moisture_readings WHERE id = $1 AND tenant_id = $2",
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
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<SoilMoistureReading>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM soil_moisture_readings WHERE tenant_id = $1 AND station_id = $2",
            )
            .bind(tid)
            .bind(station_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<SoilMoistureReading> = sqlx::query_as(
                "SELECT * FROM soil_moisture_readings WHERE tenant_id = $1 AND station_id = $2 ORDER BY timestamp DESC LIMIT $3 OFFSET $4",
            )
            .bind(tid)
            .bind(station_id)
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

    fn find_recent(
        &self,
        tid: TenantId,
        station_id: Uuid,
        limit: u32,
    ) -> RepositoryFuture<Vec<SoilMoistureReading>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            sqlx::query_as::<_, SoilMoistureReading>(
                "SELECT * FROM soil_moisture_readings WHERE tenant_id = $1 AND station_id = $2 ORDER BY timestamp DESC LIMIT $3",
            )
            .bind(tid)
            .bind(station_id)
            .bind(limit as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(
        &self,
        tid: TenantId,
        reading: SoilMoistureReading,
    ) -> RepositoryFuture<SoilMoistureReading> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            sqlx::query_as::<_, SoilMoistureReading>(
                r#"INSERT INTO soil_moisture_readings (
                    id, tenant_id, station_id, site_id, device_id,
                    timestamp, moisture_percent, soil_temperature_c,
                    soil_depth_cm, battery_level, signal_strength, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
                RETURNING *"#,
            )
            .bind(reading.id)
            .bind(tid)
            .bind(reading.station_id)
            .bind(reading.site_id)
            .bind(reading.device_id)
            .bind(reading.timestamp)
            .bind(reading.moisture_percent)
            .bind(reading.soil_temperature_c)
            .bind(reading.soil_depth_cm)
            .bind(reading.battery_level)
            .bind(reading.signal_strength)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
