use agrocore_domain::entities::tenant::{CreateTenantDto, Tenant, UpdateTenantDto};
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, TenantRepository,
};
use agrocore_shared::SharedError;
use agrocore_shared::pg_repo;
use uuid::Uuid;

pg_repo!(PgTenantRepo);

impl TenantRepository for PgTenantRepo {
    fn find_by_id(&self, id: Uuid) -> RepositoryFuture<Option<Tenant>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Tenant>("SELECT * FROM tenants WHERE id = $1")
                .bind(id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(&self, p: Pagination) -> RepositoryFuture<PaginatedResponse<Tenant>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE is_active = true")
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Tenant> =
                sqlx::query_as("SELECT * FROM tenants WHERE is_active = true LIMIT $1 OFFSET $2")
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

    fn create(&self, dto: CreateTenantDto) -> RepositoryFuture<Tenant> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, Tenant>(
                r#"INSERT INTO tenants (id, name, slug, config, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, true, NOW(), NOW())
                   RETURNING *"#,
            )
            .bind(id)
            .bind(&dto.name)
            .bind(&dto.slug)
            .bind(serde_json::to_value(dto.config.unwrap_or_default()).unwrap())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(&self, id: Uuid, dto: UpdateTenantDto) -> RepositoryFuture<Option<Tenant>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Tenant>(
                r#"UPDATE tenants SET name = COALESCE($1, name), updated_at = NOW()
                   WHERE id = $2 RETURNING *"#,
            )
            .bind(&dto.name)
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("UPDATE tenants SET is_active = false WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
