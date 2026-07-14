use agrocore_domain::entities::weather::{CreateWeatherStationDto, WeatherStation};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{WeatherStationRepo, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination};;
use agrocore_shared::{PaginatedResponse, Pagination, RepositoryFuture, Result, SharedError};
use chrono::Utc;
use serde_json;
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

    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WeatherStation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, (WeatherStation,)>("SELECT row_to_json(weather_stations) FROM weather_stations WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?
                .map(|(w,)| w)
                .ok_or_else(|| SharedError::NotFound.to_error())
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

    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WeatherStation>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weather_stations WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true)")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<WeatherStation> = sqlx::query_as("SELECT * FROM weather_stations WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true) ORDER BY label LIMIT $2 OFFSET $3")
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

    pub fn create(&self, tid: TenantId, dto: CreateWeatherStationDto) -> RepositoryFuture<WeatherStation> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();

            sqlx::query_as::<_, WeatherStation>(
                r#"INSERT INTO weather_stations (id, tenant_id, label, station_type, location, manufacturer, model, serial_number, api_key_config, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, $10, $11)
                   RETURNING *"#)
            .bind(id)
            .bind(tid.to_string())
            .bind(&dto.label)
            .bind(serde_json::to_string(&dto.station_type).unwrap_or_else(|_| "null".to_string()))
            .bind(serde_json::to_string(&dto.location).ok())
            .bind(&dto.manufacturer)
            .bind(&dto.model)
            .bind(&dto.serial_number)
            .bind(&dto.api_key_config)
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}