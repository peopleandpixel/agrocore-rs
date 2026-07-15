use agrocore_domain::entities::fertilizer::{CreateFertilizerRecordDto, FertilizerRecord, UpdateFertilizerRecordDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{FertilizerRecordRepo, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgFertilizerRecordRepo {
    pool: PgPool,
}

impl PgFertilizerRecordRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl FertilizerRecordRepo for PgFertilizerRecordRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<FertilizerRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, FertilizerRecord>("SELECT * FROM fertilizer_records WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<FertilizerRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fertilizer_records WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<FertilizerRecord> = sqlx::query_as("SELECT * FROM fertilizer_records WHERE tenant_id = $1::uuid ORDER BY application_date DESC LIMIT $2 OFFSET $3")
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

    fn find_by_site(&self, tid: TenantId, site_id: Uuid, p: Pagination) -> RepositoryFuture<PaginatedResponse<FertilizerRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fertilizer_records WHERE tenant_id = $1::uuid AND site_id = $2::uuid")
                .bind(tid.to_string())
                .bind(site_id.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<FertilizerRecord> = sqlx::query_as("SELECT * FROM fertilizer_records WHERE tenant_id = $1::uuid AND site_id = $2::uuid ORDER BY application_date DESC LIMIT $3 OFFSET $4")
                .bind(tid.to_string())
                .bind(site_id.to_string())
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

    fn create(&self, tid: TenantId, dto: CreateFertilizerRecordDto, _by: Uuid) -> RepositoryFuture<FertilizerRecord> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, FertilizerRecord>(
                r#"INSERT INTO fertilizer_records (id, tenant_id, site_id, order_id, product_name, nutrient_n, nutrient_p, nutrient_k, quantity_kg, area_ha, application_date, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid.to_string())
            .bind(dto.site_id)
            .bind(dto.order_id)
            .bind(&dto.product_name)
            .bind(dto.nutrient_n)
            .bind(dto.nutrient_p)
            .bind(dto.nutrient_k)
            .bind(dto.quantity_kg)
            .bind(dto.area_ha)
            .bind(dto.application_date)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateFertilizerRecordDto, _by: Uuid) -> RepositoryFuture<Option<FertilizerRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, FertilizerRecord>(
                r#"UPDATE fertilizer_records SET product_name = COALESCE($1, product_name), updated_at = NOW()
                   WHERE id = $2 AND tenant_id = $3 RETURNING *"#)
            .bind(&dto.product_name)
            .bind(id)
            .bind(tid.to_string())
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM fertilizer_records WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
