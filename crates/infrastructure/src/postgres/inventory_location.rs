use agrocore_domain::entities::inventory::{
    CreateInventoryLocationDto, InventoryLocation, UpdateInventoryLocationDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    InventoryLocationRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use agrocore_shared::pg_repo;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pg_repo!(PgInventoryLocationRepo);

impl InventoryLocationRepo for PgInventoryLocationRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<InventoryLocation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, InventoryLocation>(
                "SELECT * FROM inventory_locations WHERE id = $1 AND tenant_id = $2 AND (is_active IS NULL OR is_active = true)",
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
    ) -> RepositoryFuture<PaginatedResponse<InventoryLocation>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM inventory_locations WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)",
            )
            .bind(tid)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<InventoryLocation> = sqlx::query_as(
                "SELECT * FROM inventory_locations WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) ORDER BY name LIMIT $2 OFFSET $3",
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
                data: items,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }

    fn find_by_code(
        &self,
        tid: TenantId,
        code: &str,
    ) -> RepositoryFuture<Option<InventoryLocation>> {
        let pool = self.pool.clone();
        let code = code.to_string();
        Box::pin(async move {
            sqlx::query_as::<_, InventoryLocation>(
                "SELECT * FROM inventory_locations WHERE code = $1 AND tenant_id = $2 AND (is_active IS NULL OR is_active = true)",
            )
            .bind(&code)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateInventoryLocationDto,
        _by: Uuid,
    ) -> RepositoryFuture<InventoryLocation> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();

            let location = sqlx::query_as::<_, InventoryLocation>(
                r#"INSERT INTO inventory_locations (id, tenant_id, name, code, description, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(&dto.name)
            .bind(&dto.code)
            .bind(&dto.description)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(location)
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateInventoryLocationDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<InventoryLocation>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();

            let result = sqlx::query_as::<_, InventoryLocation>(
                r#"UPDATE inventory_locations SET
                    name = $1,
                    code = COALESCE($2, code),
                    description = $3,
                    updated_at = $4
                   WHERE id = $5 AND tenant_id = $6 AND (is_active IS NULL OR is_active = true)
                   RETURNING *"#,
            )
            .bind(&dto.name)
            .bind(&dto.code)
            .bind(&dto.description)
            .bind(now)
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query(
                "UPDATE inventory_locations SET is_active = false, updated_at = $1 WHERE id = $2 AND tenant_id = $3 AND (is_active IS NULL OR is_active = true)",
            )
            .bind(Utc::now())
            .bind(id)
            .bind(tid)
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }
}
