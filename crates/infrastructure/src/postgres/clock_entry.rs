use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::workforce::{
    ClockEntry, ClockSession, CreateClockEntryDto, UpdateClockEntryDto,
};
use agrocore_domain::repositories::{
    ClockEntryRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgClockEntryRepo);

impl ClockEntryRepo for PgClockEntryRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<ClockEntry>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, ClockEntry>(
                "SELECT * FROM clock_entries WHERE tenant_id = $1 AND id = $2",
            )
            .bind(tid)
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
    ) -> RepositoryFuture<PaginatedResponse<ClockEntry>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM clock_entries WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<ClockEntry> =
                sqlx::query_as("SELECT * FROM clock_entries WHERE tenant_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3")
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

    fn find_by_worker(
        &self,
        tid: TenantId,
        worker_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<ClockEntry>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM clock_entries WHERE tenant_id = $1 AND worker_id = $2",
            )
            .bind(tid)
            .bind(worker_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<ClockEntry> = sqlx::query_as(
                "SELECT * FROM clock_entries WHERE tenant_id = $1 AND worker_id = $2 ORDER BY timestamp DESC LIMIT $3 OFFSET $4",
            )
            .bind(tid)
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

    fn find_active_session(
        &self,
        tid: TenantId,
        worker_id: Uuid,
    ) -> RepositoryFuture<Option<ClockEntry>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            // An active session is a clock-in without a matching clock-out
            sqlx::query_as::<_, ClockEntry>(
                r#"SELECT ce.* FROM clock_entries ce
                   WHERE ce.tenant_id = $1 AND ce.worker_id = $2 AND ce.entry_type = 'ClockIn'
                   AND NOT EXISTS (
                     SELECT 1 FROM clock_entries co
                     WHERE co.tenant_id = ce.tenant_id AND co.worker_id = ce.worker_id
                     AND co.entry_type = 'ClockOut' AND co.timestamp > ce.timestamp
                   )
                   ORDER BY ce.timestamp DESC LIMIT 1"#,
            )
            .bind(tid)
            .bind(worker_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_sessions(
        &self,
        tid: TenantId,
        worker_id: Uuid,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> RepositoryFuture<Vec<ClockSession>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            // Fetch all clock-in entries within the date range
            let clock_ins: Vec<ClockEntry> = sqlx::query_as::<_, ClockEntry>(
                "SELECT * FROM clock_entries WHERE tenant_id = $1 AND worker_id = $2 AND entry_type = 'ClockIn' AND timestamp BETWEEN $3 AND $4 ORDER BY timestamp ASC",
            )
            .bind(tid)
            .bind(worker_id)
            .bind(from)
            .bind(to)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let mut sessions = Vec::new();
            for clock_in in clock_ins {
                // Find the next clock-out after this clock-in
                let clock_out: Option<ClockEntry> = sqlx::query_as::<_, ClockEntry>(
                    "SELECT * FROM clock_entries WHERE tenant_id = $1 AND worker_id = $2 AND entry_type = 'ClockOut' AND timestamp > $3 ORDER BY timestamp ASC LIMIT 1",
                )
                .bind(tid)
                .bind(worker_id)
                .bind(clock_in.timestamp)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                let duration_hours = match &clock_out {
                    Some(co) => {
                        let secs = (co.timestamp - clock_in.timestamp).num_seconds() as f64;
                        Some(secs / 3600.0)
                    }
                    None => None,
                };

                sessions.push(ClockSession {
                    worker_id,
                    clock_in,
                    clock_out,
                    duration_hours,
                });
            }
            Ok(sessions)
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateClockEntryDto,
        _by: Uuid,
    ) -> RepositoryFuture<ClockEntry> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, ClockEntry>(
                r#"INSERT INTO clock_entries (id, tenant_id, worker_id, entry_type, timestamp, lat, lng, task_id, notes)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"#,
            )
            .bind(Uuid::new_v4())
            .bind(tid)
            .bind(dto.worker_id)
            .bind(serde_json::to_value(&dto.entry_type).unwrap())
            .bind(dto.timestamp)
            .bind(dto.lat)
            .bind(dto.lng)
            .bind(dto.task_id)
            .bind(dto.notes)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateClockEntryDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<ClockEntry>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, ClockEntry>(
                "UPDATE clock_entries SET lat = $3, lng = $4, task_id = $5, notes = $6 WHERE tenant_id = $1 AND id = $2 RETURNING *",
            )
            .bind(tid)
            .bind(id)
            .bind(dto.lat)
            .bind(dto.lng)
            .bind(dto.task_id)
            .bind(dto.notes)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM clock_entries WHERE tenant_id = $1 AND id = $2")
                .bind(tid)
                .bind(id)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn total_hours_worked(
        &self,
        tid: TenantId,
        worker_id: Uuid,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> RepositoryFuture<f64> {
        let pool = self.pool.clone();
        Box::pin(async move {
            // Inline the session query to avoid borrowing self in async block
            let clock_ins: Vec<ClockEntry> = sqlx::query_as::<_, ClockEntry>(
                "SELECT * FROM clock_entries WHERE tenant_id = $1 AND worker_id = $2 AND entry_type = 'ClockIn' AND timestamp BETWEEN $3 AND $4 ORDER BY timestamp ASC",
            )
            .bind(tid)
            .bind(worker_id)
            .bind(from)
            .bind(to)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let mut total = 0.0;
            for clock_in in clock_ins {
                let clock_out: Option<ClockEntry> = sqlx::query_as::<_, ClockEntry>(
                    "SELECT * FROM clock_entries WHERE tenant_id = $1 AND worker_id = $2 AND entry_type = 'ClockOut' AND timestamp > $3 ORDER BY timestamp ASC LIMIT 1",
                )
                .bind(tid)
                .bind(worker_id)
                .bind(clock_in.timestamp)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                if let Some(co) = clock_out
                    && co.timestamp <= to
                {
                    let secs = (co.timestamp - clock_in.timestamp).num_seconds() as f64;
                    total += secs / 3600.0;
                }
            }
            Ok(total)
        })
    }
}
