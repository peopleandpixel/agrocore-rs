use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::{CreateGroupDto, Group, GroupType, UpdateGroupDto};
use agrocore_domain::repositories::{
    GroupRepository, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use agrocore_shared::pg_repo;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = agrocore_shared::Result<T>> + Send>>;

pg_repo!(PgGroupRepo);

impl GroupRepository for PgGroupRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Group>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Group>("SELECT * FROM groups WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Group>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM groups WHERE tenant_id = $1")
                .bind(tid)
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Group> = sqlx::query_as(
                "SELECT * FROM groups WHERE tenant_id = $1 ORDER BY label LIMIT $2 OFFSET $3",
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
    ) -> RepositoryFuture<PaginatedResponse<Group>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM groups WHERE tenant_id = $1 AND plot_id = $2",
            )
            .bind(tid)
            .bind(plot_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Group> = sqlx::query_as(
                "SELECT * FROM groups WHERE tenant_id = $1 AND plot_id = $2 ORDER BY label LIMIT $3 OFFSET $4",
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

    fn find_children(&self, tid: TenantId, parent_id: Uuid) -> RepositoryFuture<Vec<Group>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Group>(
                "SELECT * FROM groups WHERE tenant_id = $1 AND parent_group_id = $2 ORDER BY label",
            )
            .bind(tid)
            .bind(parent_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(&self, tid: TenantId, dto: CreateGroupDto, _by: Uuid) -> RepositoryFuture<Group> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            let now = chrono::Utc::now();

            sqlx::query(
                r#"
                INSERT INTO groups (id, plot_id, parent_group_id, group_type, label, created_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(id)
            .bind(dto.plot_id)
            .bind(dto.parent_group_id)
            .bind(dto.group_type.as_str())
            .bind(&dto.label)
            .bind(now)
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(Group {
                id,
                plot_id: dto.plot_id,
                parent_group_id: dto.parent_group_id,
                group_type: dto.group_type.as_str().to_string(),
                label: dto.label,
            })
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateGroupDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<Group>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let existing =
                sqlx::query_as::<_, Group>("SELECT * FROM groups WHERE id = $1 AND tenant_id = $2")
                    .bind(id)
                    .bind(tid)
                    .fetch_optional(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            if let Some(mut group) = existing {
                if let Some(parent_group_id) = dto.parent_group_id {
                    group.parent_group_id = Some(parent_group_id);
                }
                if let Some(group_type) = dto.group_type {
                    group.group_type = group_type.as_str().to_string();
                }
                if let Some(label) = dto.label {
                    group.label = label;
                }

                sqlx::query(
                    r#"
                    UPDATE groups SET parent_group_id = $1, group_type = $2, label = $3 WHERE id = $4 AND tenant_id = $5
                    "#,
                )
                .bind(group.parent_group_id)
                .bind(&group.group_type)
                .bind(&group.label)
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                Ok(Some(group))
            } else {
                Ok(None)
            }
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("DELETE FROM groups WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }
}
