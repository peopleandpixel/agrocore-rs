use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::weather::{
    CreatePhenologyRecordDto, PhenologyRecord, UpdatePhenologyRecordDto,
};
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, PhenologyRecordRepo, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgPhenologyRecordRepo);

impl PhenologyRecordRepo for PgPhenologyRecordRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<PhenologyRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, PhenologyRecord>(
                "SELECT * FROM phenology_records WHERE id = $1 AND tenant_id = $2",
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
    ) -> RepositoryFuture<PaginatedResponse<PhenologyRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM phenology_records WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<PhenologyRecord> = sqlx::query_as("SELECT * FROM phenology_records WHERE tenant_id = $1 ORDER BY observation_date DESC LIMIT $2 OFFSET $3")
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

    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<PhenologyRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM phenology_records WHERE tenant_id = $1 AND site_id = $2",
            )
            .bind(tid)
            .bind(site_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<PhenologyRecord> = sqlx::query_as("SELECT * FROM phenology_records WHERE tenant_id = $1 AND site_id = $2 ORDER BY observation_date DESC LIMIT $3 OFFSET $4")
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
        dto: CreatePhenologyRecordDto,
        _by: Uuid,
    ) -> RepositoryFuture<PhenologyRecord> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, PhenologyRecord>(
                r#"INSERT INTO phenology_records (id, tenant_id, site_id, observation_date, stage, notes, photo_url, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(dto.site_id)
            .bind(dto.observation_date)
            .bind(serde_json::to_value(&dto.stage).unwrap())
            .bind(&dto.notes)
            .bind(&dto.photo_url)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdatePhenologyRecordDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<PhenologyRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as("UPDATE phenology_records SET site_id = COALESCE($1, site_id), observation_date = COALESCE($2, observation_date), stage = COALESCE($3, stage), forecast_next_stage_date = COALESCE($4, forecast_next_stage_date), notes = COALESCE($5, notes), photo_url = COALESCE($6, photo_url), observer_id = COALESCE($7, observer_id) WHERE id = $8 AND tenant_id = $9 RETURNING *")
                .bind(dto.site_id).bind(dto.observation_date).bind(dto.stage.map(|v| serde_json::to_value(v).unwrap())).bind(dto.forecast_next_stage_date).bind(dto.notes).bind(dto.photo_url).bind(dto.observer_id).bind(id).bind(tid)
                .fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM phenology_records WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
