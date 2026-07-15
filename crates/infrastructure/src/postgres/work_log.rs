use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::workforce::{CreateWorkLogDto, UpdateWorkLogDto, WorkLog};
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, WorkLogRepo};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWorkLogRepo {
    pool: PgPool,
}
impl PgWorkLogRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl WorkLogRepo for PgWorkLogRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WorkLog>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkLog>(
                "SELECT * FROM work_logs WHERE tenant_id = $1::uuid AND id = $2",
            )
            .bind(tid.to_string())
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WorkLog>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM work_logs WHERE tenant_id = $1::uuid")
                    .bind(tid.to_string())
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<WorkLog> = sqlx::query_as("SELECT * FROM work_logs WHERE tenant_id = $1::uuid ORDER BY date DESC, created_at DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
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
    fn create(&self, tid: TenantId, dto: CreateWorkLogDto, _by: Uuid) -> RepositoryFuture<WorkLog> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WorkLog>(
                "INSERT INTO work_logs (id, tenant_id, worker_id, date, hours_worked, overtime_hours, rest_period_hours, task_description, site_id, is_night_shift, breaks_taken) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING *"
            )
            .bind(Uuid::new_v4())
            .bind(tid.to_string())
            .bind(dto.worker_id)
            .bind(dto.date)
            .bind(dto.hours_worked)
            .bind(dto.overtime_hours)
            .bind(dto.rest_period_hours)
            .bind(dto.task_description)
            .bind(dto.site_id)
            .bind(dto.is_night_shift)
            .bind(dto.breaks_taken)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateWorkLogDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<WorkLog>> {
        let pool = self.pool.clone();
        let self_clone = self.clone();
        Box::pin(async move {
            let mut query = String::from("UPDATE work_logs SET ");
            let mut idx = 1;
            let mut parts = Vec::new();

            if dto.date.is_some() {
                parts.push(format!("date = ${}", idx + 2));
                idx += 1;
            }
            if dto.hours_worked.is_some() {
                parts.push(format!("hours_worked = ${}", idx + 2));
                idx += 1;
            }
            if dto.overtime_hours.is_some() {
                parts.push(format!("overtime_hours = ${}", idx + 2));
                idx += 1;
            }
            if dto.rest_period_hours.is_some() {
                parts.push(format!("rest_period_hours = ${}", idx + 2));
                idx += 1;
            }
            if dto.task_description.is_some() {
                parts.push(format!("task_description = ${}", idx + 2));
                idx += 1;
            }
            if dto.site_id.is_some() {
                parts.push(format!("site_id = ${}", idx + 2));
                idx += 1;
            }
            if dto.is_night_shift.is_some() {
                parts.push(format!("is_night_shift = ${}", idx + 2));
                idx += 1;
            }
            if dto.breaks_taken.is_some() {
                parts.push(format!("breaks_taken = ${}", idx + 2));
            }

            if parts.is_empty() {
                return self_clone.find_by_id(tid, id).await;
            }

            query.push_str(&parts.join(", "));
            query.push_str(" WHERE tenant_id = $1::uuid AND id = $2 RETURNING *");

            let mut q = sqlx::query_as::<_, WorkLog>(&query)
                .bind(tid.to_string())
                .bind(id);

            if let Some(v) = dto.date {
                q = q.bind(v);
            }
            if let Some(v) = dto.hours_worked {
                q = q.bind(v);
            }
            if let Some(v) = dto.overtime_hours {
                q = q.bind(v);
            }
            if let Some(v) = dto.rest_period_hours {
                q = q.bind(v);
            }
            if let Some(v) = dto.task_description {
                q = q.bind(v);
            }
            if let Some(v) = dto.site_id {
                q = q.bind(v);
            }
            if let Some(v) = dto.is_night_shift {
                q = q.bind(v);
            }
            if let Some(v) = dto.breaks_taken {
                q = q.bind(v as i32);
            }

            q.fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let res = sqlx::query("DELETE FROM work_logs WHERE tenant_id = $1::uuid AND id = $2")
                .bind(tid.to_string())
                .bind(id)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(res.rows_affected() > 0)
        })
    }
    fn find_by_worker(
        &self,
        tid: TenantId,
        worker_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WorkLog>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM work_logs WHERE tenant_id = $1::uuid AND worker_id = $2",
            )
            .bind(tid.to_string())
            .bind(worker_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<WorkLog> = sqlx::query_as("SELECT * FROM work_logs WHERE tenant_id = $1::uuid AND worker_id = $2 ORDER BY date DESC, created_at DESC LIMIT $3 OFFSET $4")
                .bind(tid.to_string())
                .bind(worker_id)
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
}
