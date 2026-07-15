use agrocore_domain::entities::weather::{CreateWeatherStationDto, WeatherStation, UpdateWeatherStationDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{WeatherStationRepo, RepositoryFuture, PaginatedResponse, Pagination};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWeatherStationRepo {
    pool: PgPool,
}

impl PgWeatherStationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl WeatherStationRepo for PgWeatherStationRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WeatherStation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WeatherStation>("SELECT * FROM weather_stations WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WeatherStation>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weather_stations WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<WeatherStation> = sqlx::query_as("SELECT * FROM weather_stations WHERE tenant_id = $1::uuid LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(per_page as i32)
                .bind(offset as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            
            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }

    fn create(&self, tid: TenantId, dto: CreateWeatherStationDto, _by: Uuid) -> RepositoryFuture<WeatherStation> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, WeatherStation>(
                r#"INSERT INTO weather_stations (id, tenant_id, label, station_type, manufacturer, model, serial_number, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid.to_string())
            .bind(&dto.label)
            .bind(serde_json::to_value(&dto.station_type).unwrap())
            .bind(&dto.manufacturer)
            .bind(&dto.model)
            .bind(&dto.serial_number)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateWeatherStationDto, _by: Uuid) -> RepositoryFuture<Option<WeatherStation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WeatherStation>(
                r#"UPDATE weather_stations SET label = COALESCE($1, label), is_active = COALESCE($2, is_active), updated_at = NOW()
                   WHERE id = $3 AND tenant_id = $4 RETURNING *"#)
            .bind(&dto.label)
            .bind(dto.is_active)
            .bind(id)
            .bind(tid.to_string())
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM weather_stations WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
