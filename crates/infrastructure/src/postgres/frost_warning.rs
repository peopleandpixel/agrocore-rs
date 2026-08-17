use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::weather::{
    CreateFrostWarningDto, FrostWarning, UpdateFrostWarningDto,
};
use agrocore_domain::repositories::{FrostWarningRepo, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgFrostWarningRepo);

impl FrostWarningRepo for PgFrostWarningRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<FrostWarning>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, FrostWarning>(
                "SELECT * FROM frost_warnings WHERE id = $1 AND tenant_id = $2",
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
    ) -> RepositoryFuture<PaginatedResponse<FrostWarning>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM frost_warnings WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<FrostWarning> = sqlx::query_as(
                "SELECT * FROM frost_warnings WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
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
                data,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }

    fn find_by_station(
        &self,
        tid: TenantId,
        station_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<FrostWarning>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM frost_warnings WHERE tenant_id = $1 AND station_id = $2",
            )
            .bind(tid)
            .bind(station_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<FrostWarning> = sqlx::query_as(
                "SELECT * FROM frost_warnings WHERE tenant_id = $1 AND station_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(tid)
            .bind(station_id)
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

    fn find_active(&self, tid: TenantId) -> RepositoryFuture<Vec<FrostWarning>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, FrostWarning>(
                "SELECT * FROM frost_warnings WHERE tenant_id = $1 AND is_active = true ORDER BY created_at DESC",
            )
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateFrostWarningDto,
        by: Uuid,
    ) -> RepositoryFuture<FrostWarning> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, FrostWarning>(
                r#"INSERT INTO frost_warnings (id, tenant_id, station_id, threshold_temp_c, is_active, notify_email, notify_sms, last_triggered_at, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
                   RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(dto.station_id)
            .bind(dto.threshold_temp_c)
            .bind(dto.is_active)
            .bind(dto.notify_email)
            .bind(dto.notify_sms)
            .bind(None::<chrono::DateTime<chrono::Utc>>)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateFrostWarningDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<FrostWarning>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, FrostWarning>(
                r#"UPDATE frost_warnings
                   SET threshold_temp_c = COALESCE($1, threshold_temp_c),
                       is_active = COALESCE($2, is_active),
                       notify_email = COALESCE($3, notify_email),
                       notify_sms = COALESCE($4, notify_sms),
                       last_triggered_at = COALESCE($5, last_triggered_at),
                       updated_at = NOW()
                   WHERE id = $6 AND tenant_id = $7
                   RETURNING *"#,
            )
            .bind(dto.threshold_temp_c)
            .bind(dto.is_active)
            .bind(dto.notify_email)
            .bind(dto.notify_sms)
            .bind(dto.last_triggered_at)
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM frost_warnings WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
