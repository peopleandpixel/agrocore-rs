use agrocore_domain::entities::weather::{CreateWeatherStationDto, WeatherStation};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{RepositoryFuture, WeatherStationRepository};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct PgWeatherStationRepo {
    pool: PgPool,
}

impl PgWeatherStationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WeatherStation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query_as::<_, (WeatherStation,)>("SELECT row_to_json(weather_stations) FROM weather_stations WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            row.map(|(w,)| w).ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    pub fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<Option<WeatherStation>> {
        self.find_by_id(tid, id)
    }

    pub fn find_all(
        &self,
        tid: TenantId,
        p: agrocore_shared::Pagination,
    ) -> RepositoryFuture<agrocore_shared::PaginatedResponse<WeatherStation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weather_stations WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<WeatherStation> = sqlx::query_as("SELECT * FROM weather_stations WHERE tenant_id = $1::uuid ORDER BY label LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(p.limit as i32)
                .bind(((p.page - 1) * p.limit) as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(agrocore_shared::PaginatedResponse { items, total, page: p.page, limit: p.limit })
        })
    }

    pub fn create(&self, tid: TenantId, dto: CreateWeatherStationDto) -> Fut<WeatherStation> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let ws = WeatherStation {
                id: Uuid::new_v4(),
                tenant_id: tid,
                label: dto.label,
                station_type: dto.station_type,
                location: dto.location,
                manufacturer: dto.manufacturer,
                model: dto.model,
                serial_number: dto.serial_number,
                api_key_config: dto.api_key_config,
                is_active: true,
                sensor_metadata: None,
                firmware_version: None,
                created_at: now,
                updated_at: now,
            };
            let station = sqlx::query_as::<_, WeatherStation>(
                "INSERT INTO weather_stations (id, tenant_id, label, station_type, location, manufacturer, model, serial_number, api_key_config, is_active, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING *"
            )
            .bind(ws.id)
            .bind(tid.to_string())
            .bind(&ws.label)
            .bind(serde_json::to_string(&ws.station_type).unwrap_or_else(|_| "null".to_string()))
            .bind(serde_json::to_string(&ws.location).ok())
            .bind(&ws.manufacturer)
            .bind(&ws.model)
            .bind(&ws.serial_number)
            .bind(&ws.api_key_config)
            .bind(ws.is_active)
            .bind(ws.created_at)
            .bind(ws.updated_at)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(station)
        })
    }
}