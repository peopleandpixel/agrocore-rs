use agrocore_domain::entities::harvest::{
    CreateHarvestSeasonDto, HarvestSeason, UpdateHarvestSeasonDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    HarvestSeasonRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgHarvestSeasonRepo {
    pool: PgPool,
}
impl PgHarvestSeasonRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl HarvestSeasonRepo for PgHarvestSeasonRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<HarvestSeason>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, HarvestSeason>(
                "SELECT * FROM harvest_seasons WHERE id = $1 AND tenant_id = $2",
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
    ) -> RepositoryFuture<PaginatedResponse<HarvestSeason>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM harvest_seasons WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<HarvestSeason> = sqlx::query_as("SELECT * FROM harvest_seasons WHERE tenant_id = $1 ORDER BY year DESC, start_date DESC LIMIT $2 OFFSET $3")
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

    fn create(
        &self,
        tid: TenantId,
        dto: CreateHarvestSeasonDto,
        _by: Uuid,
    ) -> RepositoryFuture<HarvestSeason> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, HarvestSeason>(
                r#"INSERT INTO harvest_seasons (id, tenant_id, year, label, start_date, end_date, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(dto.year)
            .bind(&dto.label)
            .bind(dto.start_date)
            .bind(dto.end_date)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateHarvestSeasonDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<HarvestSeason>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, HarvestSeason>(
                r#"UPDATE harvest_seasons SET 
                    label = COALESCE($1, label),
                    start_date = COALESCE($2, start_date),
                    end_date = COALESCE($3, end_date),
                    is_active = COALESCE($4, is_active),
                    updated_at = NOW()
                   WHERE id = $5 AND tenant_id = $6 RETURNING *"#,
            )
            .bind(&dto.label)
            .bind(dto.start_date)
            .bind(dto.end_date)
            .bind(dto.is_active)
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
            sqlx::query("DELETE FROM harvest_seasons WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
