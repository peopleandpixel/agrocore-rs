use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::{CreateLivestockDto, Livestock, LivestockType, UpdateLivestockDto};
use agrocore_domain::repositories::{
    LivestockRepository, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use agrocore_shared::pg_repo;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = agrocore_shared::Result<T>> + Send>>;

pg_repo!(PgLivestockRepo);

impl LivestockRepository for PgLivestockRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Livestock>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Livestock>(
                "SELECT * FROM livestock WHERE id = $1 AND tenant_id = $2",
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
    ) -> RepositoryFuture<PaginatedResponse<Livestock>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM livestock WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Livestock> = sqlx::query_as(
                "SELECT * FROM livestock WHERE tenant_id = $1 ORDER BY label LIMIT $2 OFFSET $3",
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
    ) -> RepositoryFuture<PaginatedResponse<Livestock>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM livestock WHERE tenant_id = $1 AND plot_id = $2",
            )
            .bind(tid)
            .bind(plot_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Livestock> = sqlx::query_as(
                "SELECT * FROM livestock WHERE tenant_id = $1 AND plot_id = $2 ORDER BY label LIMIT $3 OFFSET $4",
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

    fn find_by_herd(&self, tid: TenantId, herd_id: String) -> RepositoryFuture<Vec<Livestock>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Livestock>(
                "SELECT * FROM livestock WHERE tenant_id = $1 AND herd_id = $2 ORDER BY label",
            )
            .bind(tid)
            .bind(herd_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateLivestockDto,
        _by: Uuid,
    ) -> RepositoryFuture<Livestock> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            let now = chrono::Utc::now();

            sqlx::query(
                r#"
                INSERT INTO livestock (id, plot_id, herd_id, livestock_type, count, label, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(id)
            .bind(dto.plot_id)
            .bind(&dto.herd_id)
            .bind(&dto.livestock_type)
            .bind(dto.count)
            .bind(&dto.label)
            .bind(now)
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(Livestock {
                id,
                plot_id: dto.plot_id,
                herd_id: dto.herd_id,
                livestock_type: dto.livestock_type,
                count: dto.count,
                label: dto.label,
                created_at: Some(now),
            })
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateLivestockDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<Livestock>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let existing = sqlx::query_as::<_, Livestock>(
                "SELECT * FROM livestock WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            if let Some(mut livestock) = existing {
                if let Some(herd_id) = dto.herd_id {
                    livestock.herd_id = Some(herd_id);
                }
                if let Some(livestock_type) = dto.livestock_type {
                    livestock.livestock_type = livestock_type;
                }
                if let Some(count) = dto.count {
                    livestock.count = count;
                }
                if let Some(label) = dto.label {
                    livestock.label = Some(label);
                }

                sqlx::query(
                    r#"
                    UPDATE livestock SET herd_id = $1, livestock_type = $2, count = $3, label = $4 WHERE id = $5 AND tenant_id = $6
                    "#,
                )
                .bind(&livestock.herd_id)
                .bind(&livestock.livestock_type)
                .bind(livestock.count)
                .bind(&livestock.label)
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                Ok(Some(livestock))
            } else {
                Ok(None)
            }
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("DELETE FROM livestock WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }
}
