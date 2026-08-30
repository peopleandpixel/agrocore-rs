use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::{CreateTreeDto, Tree, TreeType, UpdateTreeDto};
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, TreeRepository,
};
use agrocore_shared::SharedError;
use agrocore_shared::pg_repo;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = agrocore_shared::Result<T>> + Send>>;

pg_repo!(PgTreeRepo);

impl TreeRepository for PgTreeRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Tree>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Tree>("SELECT * FROM trees WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Tree>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM trees WHERE tenant_id = $1")
                .bind(tid)
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Tree> = sqlx::query_as(
                "SELECT * FROM trees WHERE tenant_id = $1 ORDER BY label LIMIT $2 OFFSET $3",
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

    fn find_by_plot(
        &self,
        tid: TenantId,
        plot_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Tree>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM trees WHERE tenant_id = $1 AND plot_id = $2",
            )
            .bind(tid)
            .bind(plot_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Tree> = sqlx::query_as(
                "SELECT * FROM trees WHERE tenant_id = $1 AND plot_id = $2 ORDER BY label LIMIT $3 OFFSET $4",
            )
            .bind(tid)
            .bind(plot_id)
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

    fn find_by_group(&self, tid: TenantId, group_id: Uuid) -> RepositoryFuture<Vec<Tree>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Tree>(
                "SELECT * FROM trees WHERE tenant_id = $1 AND group_id = $2 ORDER BY label",
            )
            .bind(tid)
            .bind(group_id.to_string())
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(&self, tid: TenantId, dto: CreateTreeDto, _by: Uuid) -> RepositoryFuture<Tree> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            let now = chrono::Utc::now();

            sqlx::query(
                r#"
                INSERT INTO trees (id, plot_id, group_id, tree_type, count, label, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(id)
            .bind(dto.plot_id)
            .bind(&dto.group_id)
            .bind(dto.tree_type.as_str())
            .bind(dto.count as i32)
            .bind(&dto.label)
            .bind(now)
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(Tree {
                id,
                plot_id: dto.plot_id,
                group_id: dto.group_id,
                tree_type: dto.tree_type.as_str().to_string(),
                count: dto.count as i32,
                label: dto.label,
            })
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateTreeDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<Tree>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let existing =
                sqlx::query_as::<_, Tree>("SELECT * FROM trees WHERE id = $1 AND tenant_id = $2")
                    .bind(id)
                    .bind(tid)
                    .fetch_optional(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            if let Some(mut tree) = existing {
                if let Some(group_id) = dto.group_id {
                    tree.group_id = Some(group_id);
                }
                if let Some(tree_type) = dto.tree_type {
                    tree.tree_type = tree_type.as_str().to_string();
                }
                if let Some(count) = dto.count {
                    tree.count = count as i32;
                }
                if let Some(label) = dto.label {
                    tree.label = Some(label);
                }

                sqlx::query(
                    r#"
                    UPDATE trees SET group_id = $1, tree_type = $2, count = $3, label = $4 WHERE id = $5 AND tenant_id = $6
                    "#,
                )
                .bind(&tree.group_id)
                .bind(&tree.tree_type)
                .bind(tree.count as i32)
                .bind(&tree.label)
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                Ok(Some(tree))
            } else {
                Ok(None)
            }
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("DELETE FROM trees WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }
}
