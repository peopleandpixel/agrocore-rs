use agrocore_domain::entities::harvest::{CreateHarvestSeasonDto, HarvestSeason};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct PgHarvestSeasonRepo {
    pool: PgPool,
}

impl PgHarvestSeasonRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<HarvestSeason>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query_as::<_, (HarvestSeason,)>("SELECT row_to_json(harvest_seasons) FROM harvest_seasons WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            row.map(|(s,)| s).ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    pub fn find_all(
        &self,
        tid: TenantId,
        p: agrocore_shared::Pagination,
    ) -> RepositoryFuture<agrocore_shared::PaginatedResponse<HarvestSeason>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM harvest_seasons WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<HarvestSeason> = sqlx::query_as("SELECT * FROM harvest_seasons WHERE tenant_id = $1::uuid ORDER BY year DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(p.limit as i32)
                .bind(((p.page - 1) * p.limit) as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(agrocore_shared::PaginatedResponse { items, total, page: p.page, limit: p.limit })
        })
    }

    pub fn create(&self, tid: TenantId, dto: CreateHarvestSeasonDto) -> Fut<HarvestSeason> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let season = HarvestSeason {
                id: Uuid::new_v4(),
                tenant_id: tid,
                year: dto.year,
                label: dto.label,
                start_date: dto.start_date,
                end_date: dto.end_date,
                is_active: true,
                created_at: now,
            };
            
            let s = sqlx::query_as::<_, HarvestSeason>(
                "INSERT INTO harvest_seasons (id, tenant_id, year, label, start_date, end_date, is_active, created_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"
            )
            .bind(season.id)
            .bind(tid.to_string())
            .bind(season.year)
            .bind(&season.label)
            .bind(season.start_date)
            .bind(season.end_date)
            .bind(season.is_active)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(s)
        })
    }
}