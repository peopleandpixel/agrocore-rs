use agrocore_domain::entities::weather::WeatherStation;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{WeatherStationRepository, RepositoryFuture, SharedError};
use agrocore_shared::{Result, PaginatedResponse};
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = RepositoryFuture<T>;

#[derive(Clone)]
pub struct PgWeatherStationRepo {
    pool: PgPool,
}

impl PgWeatherStationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl WeatherStationRepository for PgWeatherStationRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> Fut<Option<WeatherStation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            Ok(None) // TODO
        })
    }
    fn find_all(&self, tid: TenantId, _p: agrocore_domain::repositories::Pagination) -> Fut<PaginatedResponse<WeatherStation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            Ok(PaginatedResponse { items: vec![], total: 0, page: 1, limit: 20 })
        })
    }
}

// WeatherData, PhenologyRecord, etc. follow same pattern