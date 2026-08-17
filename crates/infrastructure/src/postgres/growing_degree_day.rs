use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::weather::{CreateGrowingDegreeDayDto, GrowingDegreeDay};
use agrocore_domain::repositories::{GrowingDegreeDayRepo, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgGrowingDegreeDayRepo);

impl GrowingDegreeDayRepo for PgGrowingDegreeDayRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<GrowingDegreeDay>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, GrowingDegreeDay>(
                "SELECT * FROM growing_degree_days WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<GrowingDegreeDay>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM growing_degree_days WHERE tenant_id = $1 AND site_id = $2",
            )
            .bind(tid)
            .bind(site_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<GrowingDegreeDay> = sqlx::query_as(
                "SELECT * FROM growing_degree_days WHERE tenant_id = $1 AND site_id = $2 ORDER BY date DESC LIMIT $3 OFFSET $4",
            )
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
        dto: CreateGrowingDegreeDayDto,
    ) -> RepositoryFuture<GrowingDegreeDay> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            let gdd = (dto.actual_mean_temp_c - dto.base_temp_c).max(0.0);
            sqlx::query_as::<_, GrowingDegreeDay>(
                r#"INSERT INTO growing_degree_days (id, tenant_id, site_id, date, base_temp_c, actual_mean_temp_c, gdd, accumulated_gdd, crop_type, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
                   RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(dto.site_id)
            .bind(dto.date)
            .bind(dto.base_temp_c)
            .bind(dto.actual_mean_temp_c)
            .bind(gdd)
            .bind(gdd)
            .bind(&dto.crop_type)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn accumulated_gdd(
        &self,
        tid: TenantId,
        site_id: Uuid,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
        crop_type: String,
    ) -> RepositoryFuture<f64> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result: (Option<f64>,) = sqlx::query_as(
                "SELECT COALESCE(SUM(gdd), 0.0) FROM growing_degree_days WHERE tenant_id = $1 AND site_id = $2 AND date BETWEEN $3 AND $4 AND crop_type = $5",
            )
            .bind(tid)
            .bind(site_id)
            .bind(from)
            .bind(to)
            .bind(crop_type)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(result.0.unwrap_or(0.0))
        })
    }
}
