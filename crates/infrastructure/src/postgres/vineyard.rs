use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::vineyard::{CreateVineyardDto, UpdateVineyardDto, Vineyard};
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, VineyardRepo,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgVineyardRepo {
    pool: PgPool,
}

impl PgVineyardRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl VineyardRepo for PgVineyardRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Vineyard>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Vineyard>(
                "SELECT * FROM vineyards WHERE id = $1 AND tenant_id = $2",
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
    ) -> RepositoryFuture<PaginatedResponse<Vineyard>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM vineyards WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Vineyard> =
                sqlx::query_as("SELECT * FROM vineyards WHERE tenant_id = $1 LIMIT $2 OFFSET $3")
                    .bind(tid)
                    .bind(per_page as i32)
                    .bind(offset as i32)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: ((total as f64 / per_page as f64).ceil() as u64),
            })
        })
    }

    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Vineyard>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM vineyards WHERE tenant_id = $1 AND site_id = $2",
            )
            .bind(tid)
            .bind(site_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Vineyard> = sqlx::query_as(
                "SELECT * FROM vineyards WHERE tenant_id = $1 AND site_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(tid)
            .bind(site_id)
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: ((total as f64 / per_page as f64).ceil() as u64),
            })
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateVineyardDto,
        _by: Uuid,
    ) -> RepositoryFuture<Vineyard> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, Vineyard>(
                r#"INSERT INTO vineyards (id, tenant_id, site_id, doc_area, vintage, grape_variety, brix_at_harvest, ph_at_harvest, acidity, yield_tons, quality_grade, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(dto.site_id)
            .bind(&dto.doc_area)
            .bind(dto.vintage)
            .bind(&dto.grape_variety)
            .bind(dto.brix_at_harvest)
            .bind(dto.ph_at_harvest)
            .bind(dto.acidity)
            .bind(dto.yield_tons)
            .bind(serde_json::to_value(&dto.quality_grade).unwrap())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateVineyardDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<Vineyard>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Vineyard>(
                r#"UPDATE vineyards SET doc_area = COALESCE($1, doc_area), vintage = COALESCE($2, vintage), updated_at = NOW()
                   WHERE id = $3 AND tenant_id = $4 RETURNING *"#)
            .bind(&dto.doc_area)
            .bind(dto.vintage)
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
            sqlx::query("UPDATE vineyards SET is_active = false WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
