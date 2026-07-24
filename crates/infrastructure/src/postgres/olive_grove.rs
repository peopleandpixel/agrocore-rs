use agrocore_domain::entities::olive::{CreateOliveGroveDto, OliveGrove, UpdateOliveGroveDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    OliveGroveRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgOliveGroveRepo {
    pool: PgPool,
}

impl PgOliveGroveRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl OliveGroveRepo for PgOliveGroveRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<OliveGrove>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, OliveGrove>(
                "SELECT * FROM olive_groves WHERE id = $1 AND tenant_id = $2",
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
    ) -> RepositoryFuture<PaginatedResponse<OliveGrove>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM olive_groves WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<OliveGrove> = sqlx::query_as(
                "SELECT * FROM olive_groves WHERE tenant_id = $1 LIMIT $2 OFFSET $3",
            )
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
    ) -> RepositoryFuture<PaginatedResponse<OliveGrove>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM olive_groves WHERE tenant_id = $1 AND site_id = $2",
            )
            .bind(tid)
            .bind(site_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<OliveGrove> = sqlx::query_as("SELECT * FROM olive_groves WHERE tenant_id = $1 AND site_id = $2 LIMIT $3 OFFSET $4")
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
        dto: CreateOliveGroveDto,
        _by: Uuid,
    ) -> RepositoryFuture<OliveGrove> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, OliveGrove>(
                r#"INSERT INTO olive_groves (id, tenant_id, site_id, label, variety, tree_count, planting_year, area_ha, spacing_m, irrigation_type, is_organic, certification_body, certification_number, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NOW())
                   RETURNING * "#)
            .bind(id)
            .bind(tid)
            .bind(dto.site_id)
            .bind(&dto.label)
            .bind(&dto.variety)
            .bind(dto.tree_count)
            .bind(dto.planting_year)
            .bind(dto.area_ha)
            .bind(dto.spacing_m)
            .bind(&dto.irrigation_type)
            .bind(dto.is_organic)
            .bind(&dto.certification_body)
            .bind(&dto.certification_number)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateOliveGroveDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<OliveGrove>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, OliveGrove>(
                r#"UPDATE olive_groves SET label = COALESCE($1, label), variety = COALESCE($2, variety), tree_count = COALESCE($3, tree_count), planting_year = COALESCE($4, planting_year), area_ha = COALESCE($5, area_ha), spacing_m = COALESCE($6, spacing_m), irrigation_type = COALESCE($7, irrigation_type), is_organic = COALESCE($8, is_organic), certification_body = COALESCE($9, certification_body), certification_number = COALESCE($10, certification_number), updated_at = NOW()
                   WHERE id = $11 AND tenant_id = $12 RETURNING * "#)
            .bind(&dto.label)
            .bind(&dto.variety)
            .bind(dto.tree_count)
            .bind(dto.planting_year)
            .bind(dto.area_ha)
            .bind(dto.spacing_m)
            .bind(&dto.irrigation_type)
            .bind(dto.is_organic)
            .bind(&dto.certification_body)
            .bind(&dto.certification_number)
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
            sqlx::query("DELETE FROM olive_groves WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
