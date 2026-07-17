use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::weather::{CreateWeatherDataDto, WeatherData};
use agrocore_domain::repositories::{RepositoryFuture, WeatherDataRepo};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWeatherDataRepo {
    pool: PgPool,
}

impl PgWeatherDataRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl WeatherDataRepo for PgWeatherDataRepo {
    fn create(&self, tid: TenantId, dto: CreateWeatherDataDto) -> RepositoryFuture<WeatherData> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, WeatherData>(
                r#"INSERT INTO weather_data (id, station_id, tenant_id, timestamp, temperature_c, humidity_percent, precipitation_mm, wind_speed_kmh, wind_direction_deg, solar_radiation_wm2, pressure_hpa)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                   RETURNING *"#)
            .bind(id)
            .bind(dto.station_id)
            .bind(tid)
            .bind(dto.timestamp)
            .bind(dto.temperature_c)
            .bind(dto.humidity_percent)
            .bind(dto.precipitation_mm)
            .bind(dto.wind_speed_kmh)
            .bind(dto.wind_direction_deg)
            .bind(dto.solar_radiation_wm2)
            .bind(dto.pressure_hpa)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WeatherData>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WeatherData>(
                "SELECT * FROM weather_data WHERE id = $1 AND tenant_id = $2",
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
    ) -> RepositoryFuture<PaginatedResponse<WeatherData>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM weather_data WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<WeatherData> = sqlx::query_as("SELECT * FROM weather_data WHERE tenant_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3")
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

    fn find_by_station(
        &self,
        tid: TenantId,
        station_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WeatherData>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weather_data WHERE tenant_id = $1 AND station_id = $2")
                .bind(tid)
                .bind(station_id)
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<WeatherData> = sqlx::query_as("SELECT * FROM weather_data WHERE tenant_id = $1 AND station_id = $2 ORDER BY timestamp DESC LIMIT $3 OFFSET $4")
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

    fn update(
        &self,
        _tid: TenantId,
        _id: Uuid,
        _dto: agrocore_domain::entities::weather::UpdateWeatherDataDto,
    ) -> RepositoryFuture<Option<WeatherData>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }

    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
}
