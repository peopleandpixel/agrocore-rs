use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::water::WaterUsage;
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, WaterUsageRepo,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWaterUsageRepo {
    pool: PgPool,
}
impl PgWaterUsageRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl WaterUsageRepo for PgWaterUsageRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WaterUsage>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WaterUsage>(
                "SELECT * FROM water_usages WHERE tenant_id = $1::uuid AND id = $2",
            )
            .bind(tid.to_string())
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WaterUsage>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM water_usages WHERE tenant_id = $1::uuid")
                    .bind(tid.to_string())
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<WaterUsage> = sqlx::query_as("SELECT * FROM water_usages WHERE tenant_id = $1::uuid ORDER BY usage_date DESC, created_at DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
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
        dto: agrocore_domain::entities::water::CreateWaterUsageDto,
        _by: Uuid,
    ) -> RepositoryFuture<WaterUsage> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WaterUsage>(
                "INSERT INTO water_usages (id, tenant_id, source_id, site_id, usage_date, volume_m3, irrigation_method, efficiency_pct) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"
            )
            .bind(Uuid::new_v4())
            .bind(tid.to_string())
            .bind(dto.source_id)
            .bind(dto.site_id)
            .bind(dto.usage_date)
            .bind(dto.volume_m3)
            .bind(serde_json::to_value(dto.irrigation_method).unwrap())
            .bind(dto.efficiency_pct)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: agrocore_domain::entities::water::UpdateWaterUsageDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<WaterUsage>> {
        let pool = self.pool.clone();
        let self_clone = self.clone();
        Box::pin(async move {
            let mut query = String::from("UPDATE water_usages SET ");
            let mut idx = 1;
            let mut parts = Vec::new();

            if dto.source_id.is_some() {
                parts.push(format!("source_id = ${}", idx + 2));
                idx += 1;
            }
            if dto.site_id.is_some() {
                parts.push(format!("site_id = ${}", idx + 2));
                idx += 1;
            }
            if dto.usage_date.is_some() {
                parts.push(format!("usage_date = ${}", idx + 2));
                idx += 1;
            }
            if dto.volume_m3.is_some() {
                parts.push(format!("volume_m3 = ${}", idx + 2));
                idx += 1;
            }
            if dto.irrigation_method.is_some() {
                parts.push(format!("irrigation_method = ${}", idx + 2));
                idx += 1;
            }
            if dto.efficiency_pct.is_some() {
                parts.push(format!("efficiency_pct = ${}", idx + 2));
            }

            if parts.is_empty() {
                return self_clone.find_by_id(tid, id).await;
            }

            query.push_str(&parts.join(", "));
            query.push_str(" WHERE tenant_id = $1::uuid AND id = $2 RETURNING *");

            let mut q = sqlx::query_as::<_, WaterUsage>(&query)
                .bind(tid.to_string())
                .bind(id);

            if let Some(v) = dto.source_id {
                q = q.bind(v);
            }
            if let Some(v) = dto.site_id {
                q = q.bind(v);
            }
            if let Some(v) = dto.usage_date {
                q = q.bind(v);
            }
            if let Some(v) = dto.volume_m3 {
                q = q.bind(v);
            }
            if let Some(v) = dto.irrigation_method {
                q = q.bind(serde_json::to_value(v).unwrap());
            }
            if let Some(v) = dto.efficiency_pct {
                q = q.bind(v);
            }

            q.fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let res =
                sqlx::query("DELETE FROM water_usages WHERE tenant_id = $1::uuid AND id = $2")
                    .bind(tid.to_string())
                    .bind(id)
                    .execute(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(res.rows_affected() > 0)
        })
    }
    fn find_by_source(
        &self,
        tid: TenantId,
        source_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WaterUsage>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM water_usages WHERE tenant_id = $1::uuid AND source_id = $2",
            )
            .bind(tid.to_string())
            .bind(source_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<WaterUsage> = sqlx::query_as("SELECT * FROM water_usages WHERE tenant_id = $1::uuid AND source_id = $2 ORDER BY usage_date DESC, created_at DESC LIMIT $3 OFFSET $4")
                .bind(tid.to_string())
                .bind(source_id)
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
    ) -> RepositoryFuture<PaginatedResponse<WaterUsage>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM water_usages WHERE tenant_id = $1::uuid AND site_id = $2",
            )
            .bind(tid.to_string())
            .bind(site_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<WaterUsage> = sqlx::query_as("SELECT * FROM water_usages WHERE tenant_id = $1::uuid AND site_id = $2 ORDER BY usage_date DESC, created_at DESC LIMIT $3 OFFSET $4")
                .bind(tid.to_string())
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
}
