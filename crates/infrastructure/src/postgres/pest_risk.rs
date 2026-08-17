use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::weather::{CreatePestRiskDto, PestRisk};
use agrocore_domain::repositories::{PestRiskRepo, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgPestRiskRepo);

impl PestRiskRepo for PgPestRiskRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<PestRisk>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, PestRisk>(
                "SELECT * FROM pest_risks WHERE id = $1 AND tenant_id = $2",
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
    ) -> RepositoryFuture<PaginatedResponse<PestRisk>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM pest_risks WHERE tenant_id = $1 AND site_id = $2",
            )
            .bind(tid)
            .bind(site_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<PestRisk> = sqlx::query_as(
                "SELECT * FROM pest_risks WHERE tenant_id = $1 AND site_id = $2 ORDER BY assessment_date DESC LIMIT $3 OFFSET $4",
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

    fn create(&self, tid: TenantId, dto: CreatePestRiskDto) -> RepositoryFuture<PestRisk> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, PestRisk>(
                r#"INSERT INTO pest_risks (id, tenant_id, site_id, assessment_date, risk_level, pest_type, confidence, recommended_action, model_version, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
                   RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(dto.site_id)
            .bind(dto.assessment_date)
            .bind(serde_json::to_value(&dto.risk_level).unwrap_or_default())
            .bind(&dto.pest_type)
            .bind(dto.confidence)
            .bind(&dto.recommended_action)
            .bind(&dto.model_version)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
