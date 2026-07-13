use agrocore_domain::entities::tenant::{CreateTenantDto, Tenant, UpdateTenantDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, TenantRepository};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use serde_json;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgTenantRepo {
    pool: PgPool,
}

impl PgTenantRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl TenantRepository for PgTenantRepo {
    fn find_by_id(&self, id: TenantId) -> RepositoryFuture<Option<Tenant>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, (Tenant,)>("SELECT row_to_json(tenants) FROM tenants WHERE id = $1")
                .bind(id.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?
                .map(|(t,)| t)
                .ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    fn find_all(&self, p: Pagination) -> RepositoryFuture<PaginatedResponse<Tenant>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tenants")
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Tenant> = sqlx::query_as("SELECT * FROM tenants ORDER BY name LIMIT $1 OFFSET $2")
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

    fn create(&self, dto: CreateTenantDto, _by: Uuid) -> RepositoryFuture<Tenant> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();

            sqlx::query_as::<_, Tenant>(
                r#"INSERT INTO tenants (id, name, slug, config, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   RETURNING *"#
            )
            .bind(id)
            .bind(&dto.name)
            .bind(&dto.slug)
            .bind(serde_json::to_string(&dto.config).unwrap_or_else(|_| "{}".to_string()))
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(&self, id: TenantId, dto: UpdateTenantDto, _by: Uuid) -> RepositoryFuture<Option<Tenant>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();

            sqlx::query_as::<_, Tenant>(
                r#"UPDATE tenants SET
                    name = COALESCE($1, name),
                    config = COALESCE($2, config),
                    updated_at = $3
                   WHERE id = $4
                   RETURNING *"#
            )
            .bind(&dto.name)
            .bind(dto.config.as_ref().map(|c| serde_json::to_string(c).unwrap_or_else(|_| "{}".to_string())))
            .bind(now)
            .bind(id.to_string())
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, id: TenantId) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("DELETE FROM tenants WHERE id = $1")
                .bind(id.to_string())
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(result.rows_affected() > 0)
        })
    }
}