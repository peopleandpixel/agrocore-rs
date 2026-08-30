use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::{CreateVarietyDto, UpdateVarietyDto, Variety, VarietyCategory};
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, VarietyRepository,
};
use agrocore_shared::SharedError;
use agrocore_shared::pg_repo;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = agrocore_shared::Result<T>> + Send>>;

pg_repo!(PgVarietyRepo);

impl VarietyRepository for PgVarietyRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Variety>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Variety>("SELECT * FROM varieties WHERE id = $1 AND tenant_id = $2")
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
    ) -> RepositoryFuture<PaginatedResponse<Variety>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM varieties WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Variety> = sqlx::query_as(
                "SELECT * FROM varieties WHERE tenant_id = $1 ORDER BY name LIMIT $2 OFFSET $3",
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
                ((total as f64) / (per_page as f64)).ceil() as u64
            };

            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page: page as u64,
                per_page: per_page as u64,
                total_pages,
            })
        })
    }

    fn find_by_category(
        &self,
        tid: TenantId,
        category: VarietyCategory,
    ) -> RepositoryFuture<Vec<Variety>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Variety>(
                "SELECT * FROM varieties WHERE tenant_id = $1 AND category = $2 ORDER BY name",
            )
            .bind(tid)
            .bind(category.as_str())
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(&self, tid: TenantId, dto: CreateVarietyDto, _by: Uuid) -> RepositoryFuture<Variety> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            let now = chrono::Utc::now();

            sqlx::query(
                r#"
                INSERT INTO varieties (id, category, name, origin, created_at)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(id)
            .bind(dto.category.as_str())
            .bind(&dto.name)
            .bind(&dto.origin)
            .bind(now)
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(Variety {
                id,
                category: dto.category.as_str().to_string(),
                name: dto.name,
                origin: dto.origin,
            })
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateVarietyDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<Variety>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let existing = sqlx::query_as::<_, Variety>(
                "SELECT * FROM varieties WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            if let Some(mut variety) = existing {
                if let Some(category) = dto.category {
                    variety.category = category.as_str().to_string();
                }
                if let Some(name) = dto.name {
                    variety.name = name;
                }
                if let Some(origin) = dto.origin {
                    variety.origin = Some(origin);
                }

                sqlx::query(
                    r#"
                    UPDATE varieties SET category = $1, name = $2, origin = $3 WHERE id = $4 AND tenant_id = $5
                    "#,
                )
                .bind(&variety.category)
                .bind(&variety.name)
                .bind(&variety.origin)
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                Ok(Some(variety))
            } else {
                Ok(None)
            }
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("DELETE FROM varieties WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }
}
