use agrocore_domain::entities::weather::{CreateWeatherDataDto, WeatherData};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct PgWeatherDataRepo {
    pool: PgPool,
}

impl PgWeatherDataRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WeatherData>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query_as::<_, (WeatherData,)>("SELECT row_to_json(weather_data) FROM weather_data WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            row.map(|(w,)| w).ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    pub fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WeatherData>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weather_data WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<WeatherData> = sqlx::query_as("SELECT * FROM weather_data WHERE tenant_id = $1::uuid ORDER BY timestamp DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(p.limit as i32)
                .bind(((p.page - 1) * p.limit) as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(PaginatedResponse { items, total, page: p.page, limit: p.limit })
        })
    }

    pub fn find_by_station(
        &self,
        tid: TenantId,
        station_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WeatherData>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weather_data WHERE tenant_id = $1::uuid AND station_id = $2::uuid")
                .bind(tid.to_string())
                .bind(station_id.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<WeatherData> = sqlx::query_as("SELECT * FROM weather_data WHERE tenant_id = $1::uuid AND station_id = $2::uuid ORDER BY timestamp DESC LIMIT $3 OFFSET $4")
                .bind(tid.to_string())
                .bind(station_id.to_string())
                .bind(p.limit as i32)
                .bind(((p.page - 1) * p.limit) as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(PaginatedResponse { items, total, page: p.page, limit: p.limit })
        })
    }

    pub fn create(&self, tid: TenantId, dto: CreateWeatherDataDto) -> Fut<WeatherData> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            let wd = WeatherData {
                id,
                station_id: dto.station_id,
                tenant_id: tid,
                timestamp: dto.timestamp,
                temperature_c: dto.temperature_c,
                humidity_percent: dto.humidity_percent,
                precipitation_mm: dto.precipitation_mm,
                wind_speed_kmh: dto.wind_speed_kmh,
                wind_direction_deg: dto.wind_direction_deg,
                solar_radiation_wm2: dto.solar_radiation_wm2,
                pressure_hpa: dto.pressure_hpa,
                soil_temperature_c: None,
                soil_moisture_percent: None,
                leaf_wetness: None,
            };
            
            sqlx::query_as::<_, WeatherData>(
                "INSERT INTO weather_data (id, station_id, tenant_id, timestamp, temperature_c, humidity_percent, precipitation_mm, wind_speed_kmh, wind_direction_deg, solar_radiation_wm2, pressure_hpa) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING *"
            )
            .bind(wd.id)
            .bind(wd.station_id)
            .bind(tid.to_string())
            .bind(wd.timestamp)
            .bind(wd.temperature_c)
            .bind(wd.humidity_percent)
            .bind(wd.precipitation_mm)
            .bind(wd.wind_speed_kmh)
            .bind(wd.wind_direction_deg)
            .bind(wd.solar_radiation_wm2)
            .bind(wd.pressure_hpa)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(wd)
        })
    }
}