use agrocore_domain::entities::equipment::{
    CreateEquipmentDto, DepreciationMethod, DepreciationScheduleEntry, Equipment,
    EquipmentDepreciationDto, FuelConsumptionDto, MaintenanceCostSummaryDto, MaintenanceLogDto,
    UpdateEquipmentDto, UsageLogDto, UsageSummaryDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{
    EquipmentRepository, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use chrono::{DateTime, Datelike, Utc};
use sqlx::PgPool;
use sqlx::Row;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = agrocore_shared::Result<T>> + Send>>;

agrocore_shared::pg_repo!(PgEquipmentRepo);

impl EquipmentRepository for PgEquipmentRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Equipment>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Equipment>(
                "SELECT * FROM equipment WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<Equipment>> {
        let pool = self.pool.clone();
        let roles_vec = roles.to_vec();
        Box::pin(async move {
            let can_see_all =
                roles_vec.contains(&UserRole::Admin) || roles_vec.contains(&UserRole::Manager);

            if can_see_all {
                sqlx::query_as::<_, Equipment>(
                    "SELECT * FROM equipment WHERE id = $1 AND tenant_id = $2",
                )
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            } else {
                sqlx::query_as::<_, Equipment>(
                    "SELECT * FROM equipment WHERE id = $1 AND tenant_id = $2 AND assigned_to = $3",
                )
                .bind(id)
                .bind(tid)
                .bind(user_id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            }
        })
    }

    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Equipment>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)")
                .bind(tid)
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<Equipment> = sqlx::query_as("SELECT * FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) ORDER BY label LIMIT $2 OFFSET $3")
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

    fn find_all_filtered(
        &self,
        tid: TenantId,
        p: Pagination,
        search: Option<&str>,
        equipment_type: Option<&str>,
        in_usage: Option<bool>,
        needs_maintenance: Option<bool>,
    ) -> RepositoryFuture<PaginatedResponse<Equipment>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        let search_owned = search.map(|s| s.to_lowercase());
        let eq_type_owned = equipment_type.map(|s| s.to_lowercase());

        Box::pin(async move {
            // Build dynamic WHERE clauses
            // Base condition: tenant_id + is_active
            let mut where_clauses: Vec<String> = vec!["tenant_id = $1".to_string()];
            let mut count_clauses: Vec<String> = vec!["tenant_id = $1".to_string()];
            let mut bind_idx = 2i32;

            // Search filter: matches label or code (case-insensitive LIKE)
            if let Some(ref search_term) = search_owned {
                where_clauses.push(format!(
                    "(LOWER(label) LIKE ${} OR LOWER(code) LIKE ${})",
                    bind_idx,
                    bind_idx + 1
                ));
                count_clauses.push(format!(
                    "(LOWER(label) LIKE ${} OR LOWER(code) LIKE ${})",
                    bind_idx,
                    bind_idx + 1
                ));
                bind_idx += 2;
            }

            // equipment_type filter (case-insensitive match against JSON enum)
            if let Some(ref eq_type) = eq_type_owned {
                where_clauses.push(format!("LOWER(equipment_type::text) LIKE ${}", bind_idx));
                count_clauses.push(format!("LOWER(equipment_type::text) LIKE ${}", bind_idx));
                bind_idx += 1;
            }

            // in_usage filter
            if let Some(in_use_val) = in_usage {
                where_clauses.push(format!("in_usage = ${}", bind_idx));
                count_clauses.push(format!("in_usage = ${}", bind_idx));
                bind_idx += 1;
            }

            // needs_maintenance filter: next_maintenance_date <= now or is NULL
            if needs_maintenance.unwrap_or(false) {
                where_clauses.push(format!("(is_active IS NULL OR is_active = true) AND (next_maintenance_date IS NULL OR next_maintenance_date <= ${})", bind_idx));
                bind_idx += 1;
            } else {
                where_clauses.push("(is_active IS NULL OR is_active = true)".to_string());
            }

            let where_clause = format!("WHERE {}", where_clauses.join(" AND "));
            let count_where = format!("WHERE {}", count_clauses.join(" AND "));

            let count_sql = format!("SELECT COUNT(*) FROM equipment {}", count_where);
            let items_sql = format!(
                "SELECT * FROM equipment {} ORDER BY label LIMIT ${} OFFSET ${}",
                where_clause,
                bind_idx,
                bind_idx + 1
            );

            // Execute count
            let total: i64 = {
                let mut q = sqlx::query_scalar::<_, i64>(&count_sql);
                q = q.bind(tid);
                if let Some(ref search_term) = search_owned {
                    let pattern = format!("%{}%", search_term);
                    q = q.bind(pattern.clone());
                    q = q.bind(pattern);
                }
                if let Some(ref eq_type) = eq_type_owned {
                    let pattern = format!("%{}%", eq_type);
                    q = q.bind(pattern);
                }
                if let Some(in_use_val) = in_usage {
                    q = q.bind(in_use_val);
                }
                if needs_maintenance.unwrap_or(false) {
                    q = q.bind(chrono::Utc::now());
                }
                q.fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?
            };

            // Execute items query
            let items: Vec<Equipment> = {
                let mut q = sqlx::query_as::<_, Equipment>(&items_sql);
                q = q.bind(tid);
                if let Some(ref search_term) = search_owned {
                    let pattern = format!("%{}%", search_term);
                    q = q.bind(pattern.clone());
                    q = q.bind(pattern);
                }
                if let Some(ref eq_type) = eq_type_owned {
                    let pattern = format!("%{}%", eq_type);
                    q = q.bind(pattern);
                }
                if let Some(in_use_val) = in_usage {
                    q = q.bind(in_use_val);
                }
                if needs_maintenance.unwrap_or(false) {
                    q = q.bind(chrono::Utc::now());
                }
                q = q.bind(per_page as i32);
                q = q.bind(offset as i32);
                q.fetch_all(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?
            };

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

    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        _user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Equipment>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        let roles_vec = roles.to_vec();

        Box::pin(async move {
            let can_see_all =
                roles_vec.contains(&UserRole::Admin) || roles_vec.contains(&UserRole::Manager);

            if can_see_all {
                let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

                let items: Vec<Equipment> = sqlx::query_as("SELECT * FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) ORDER BY label LIMIT $2 OFFSET $3")
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
            } else {
                let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

                let items: Vec<Equipment> = sqlx::query_as("SELECT * FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) ORDER BY label LIMIT $2 OFFSET $3")
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
            }
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateEquipmentDto,
        _by: Uuid,
    ) -> RepositoryFuture<Equipment> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();
            let equipment_type =
                serde_json::to_string(&dto.equipment_type).unwrap_or_else(|_| "{}".to_string());
            let maintenance_intervals = dto
                .maintenance_intervals
                .map(|m| serde_json::to_string(&m).unwrap_or_else(|_| "[]".to_string()));

            let equipment = sqlx::query_as::<_, Equipment>(
                r#"INSERT INTO equipment (id, tenant_id, label, code, equipment_type, maintenance_intervals, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                   RETURNING *"#
            )
            .bind(id)
            .bind(tid)
            .bind(&dto.label)
            .bind(&dto.code)
            .bind(equipment_type)
            .bind(maintenance_intervals)
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(equipment)
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateEquipmentDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<Equipment>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let equipment_type = dto
                .equipment_type
                .as_ref()
                .map(|t| serde_json::to_string(t).unwrap_or_else(|_| "{}".to_string()));
            let maintenance_intervals = dto
                .maintenance_intervals
                .as_ref()
                .map(|m| serde_json::to_string(m).unwrap_or_else(|_| "{}".to_string()));
            let next_maintenance = dto.next_maintenance_date;

            let equipment = sqlx::query_as::<_, Equipment>(
                r#"UPDATE equipment SET 
                    label = COALESCE($1, label), 
                    code = COALESCE($2, code), 
                    equipment_type = COALESCE($3, equipment_type),
                    in_usage = COALESCE($4, in_usage),
                    maintenance_intervals = COALESCE($5, maintenance_intervals),
                    next_maintenance_date = COALESCE($6, next_maintenance_date),
                    last_maintenance_hours = COALESCE($7, last_maintenance_hours),
                    updated_at = $8
                   WHERE id = $9 AND tenant_id = $10
                   RETURNING *"#,
            )
            .bind(&dto.label)
            .bind(&dto.code)
            .bind(equipment_type)
            .bind(dto.in_usage)
            .bind(maintenance_intervals)
            .bind(next_maintenance)
            .bind(dto.last_maintenance_hours)
            .bind(now)
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(equipment)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("UPDATE equipment SET is_active = false, updated_at = $1 WHERE id = $2 AND tenant_id = $3")
                .bind(Utc::now())
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }

    fn find_maintenance_due(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Equipment>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM equipment \
                 WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) \
                 AND (next_maintenance_date IS NULL OR next_maintenance_date <= $2)",
            )
            .bind(tid)
            .bind(Utc::now())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<Equipment> = sqlx::query_as::<_, Equipment>(
                "SELECT * FROM equipment \
                 WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) \
                 AND (next_maintenance_date IS NULL OR next_maintenance_date <= $2) \
                 ORDER BY next_maintenance_date ASC NULLS LAST, label \
                 LIMIT $3 OFFSET $4",
            )
            .bind(tid)
            .bind(Utc::now())
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

    fn record_maintenance(
        &self,
        tid: TenantId,
        id: Uuid,
        hours: f64,
        note: Option<String>,
    ) -> RepositoryFuture<Option<Equipment>> {
        let pool = self.pool.clone();

        Box::pin(async move {
            let mut tx = pool
                .begin()
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            // Get current equipment to calculate next_maintenance_date
            let current: Option<Equipment> = sqlx::query_as::<_, Equipment>(
                "SELECT * FROM equipment WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let eq = current.ok_or_else(|| SharedError::NotFound("Equipment not found".into()))?;

            // Calculate next maintenance date based on intervals
            let now = Utc::now();
            let next_maintenance = if let Some(intervals) = &eq.maintenance_intervals {
                // Use the shortest interval for next_maintenance_date
                let min_interval_hours: Option<f64> = intervals
                    .iter()
                    .filter_map(|i| i.interval_hours)
                    .fold(None, |min: Option<f64>, h| {
                        Some(match min {
                            Some(m) if m <= h => m,
                            _ => h,
                        })
                    });

                if let Some(min_hours) = min_interval_hours {
                    let days_ahead = (min_hours / 24.0).ceil() as i64;
                    if days_ahead > 0 {
                        Some(now + chrono::Duration::days(days_ahead))
                    } else {
                        // Less than a day — set to 7 days
                        Some(now + chrono::Duration::days(7))
                    }
                } else {
                    // Check interval_days
                    let min_interval_days: Option<i64> = intervals
                        .iter()
                        .filter_map(|i| i.interval_days.map(|d| d as i64))
                        .min();
                    min_interval_days.map(|min_days| now + chrono::Duration::days(min_days))
                }
            } else {
                None
            };

            // Record the maintenance event
            let _ = sqlx::query(
                r#"INSERT INTO equipment_maintenance_log
                   (equipment_id, tenant_id, hours, note, performed_at)
                   VALUES ($1, $2, $3, $4, $5)"#,
            )
            .bind(id)
            .bind(tid)
            .bind(hours)
            .bind(&note)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            // Update equipment with new hours and next_maintenance_date
            let updated: Equipment = sqlx::query_as::<_, Equipment>(
                r#"UPDATE equipment SET
                   last_maintenance_hours = $1,
                   next_maintenance_date = $2,
                   updated_at = $3
                   WHERE id = $4 AND tenant_id = $5
                   RETURNING *"#,
            )
            .bind(hours)
            .bind(next_maintenance)
            .bind(now)
            .bind(id)
            .bind(tid)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            tx.commit()
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(Some(updated))
        })
    }

    fn get_maintenance_log(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Vec<MaintenanceLogDto>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, MaintenanceLogDto>(
                r#"
                SELECT id, equipment_id, tenant_id, hours, note, performed_at, created_at,
                       parts_cost, labor_hours, downtime_hours
                FROM equipment_maintenance_log
                WHERE equipment_id = $1 AND tenant_id = $2
                ORDER BY performed_at DESC
                "#,
            )
            .bind(id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn get_maintenance_cost_summary(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Option<MaintenanceCostSummaryDto>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, MaintenanceCostSummaryDto>(
                r#"
                SELECT
                    $1::uuid as equipment_id,
                    COALESCE(SUM(parts_cost), 0.0) as total_parts_cost,
                    COALESCE(SUM(labor_hours), 0.0) as total_labor_hours,
                    COALESCE(SUM(downtime_hours), 0.0) as total_downtime_hours,
                    COUNT(*) as total_maintenance_count,
                    COALESCE(SUM(parts_cost), 0.0) as total_cost
                FROM equipment_maintenance_log
                WHERE equipment_id = $1 AND tenant_id = $2
                "#,
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update_maintenance_costs(
        &self,
        tid: TenantId,
        log_id: Uuid,
        parts_cost: f64,
        labor_hours: f64,
        downtime_hours: f64,
    ) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let rows = sqlx::query(
                r#"
                UPDATE equipment_maintenance_log
                SET parts_cost = $3::numeric, labor_hours = $4::numeric, downtime_hours = $5::numeric
                WHERE id = $1 AND tenant_id = $2
                "#,
            )
            .bind(log_id)
            .bind(tid)
            .bind(parts_cost)
            .bind(labor_hours)
            .bind(downtime_hours)
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(rows.rows_affected() > 0)
        })
    }
    /// Get fuel consumption history for an equipment, newest first.
    fn get_fuel_consumption(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Vec<FuelConsumptionDto>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let records: Vec<FuelConsumptionDto> = sqlx::query_as::<_, FuelConsumptionDto>(
                r#"SELECT * FROM equipment_fuel_consumption
                   WHERE equipment_id = $1 AND tenant_id = $2
                   ORDER BY consumed_at DESC"#,
            )
            .bind(id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(records)
        })
    }
    /// Record a fuel consumption entry.
    fn record_fuel_consumption(
        &self,
        tid: TenantId,
        equipment_id: Uuid,
        liters: f64,
        cost_per_liter: Option<f64>,
        operation_type: Option<&str>,
        field_id: Option<Uuid>,
        hours_operated: Option<f64>,
        notes: Option<&str>,
    ) -> RepositoryFuture<Option<FuelConsumptionDto>> {
        let pool = self.pool.clone();
        let op_type_owned = operation_type.map(|s| s.to_string());
        let notes_owned = notes.map(|s| s.to_string());
        Box::pin(async move {
            let now = Utc::now();
            let new_id = Uuid::new_v4();
            let total_cost = cost_per_liter.map(|c| liters * c);

            let record = sqlx::query_as::<_, FuelConsumptionDto>(
                r#"INSERT INTO equipment_fuel_consumption
                   (id, equipment_id, tenant_id, liters, cost_per_liter, total_cost,
                    operation_type, field_id, hours_operated, consumed_at, notes, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $10)
                   RETURNING *"#,
            )
            .bind(new_id)
            .bind(equipment_id)
            .bind(tid)
            .bind(liters)
            .bind(cost_per_liter)
            .bind(total_cost)
            .bind(op_type_owned)
            .bind(field_id)
            .bind(hours_operated)
            .bind(now)
            .bind(notes_owned)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(Some(record))
        })
    }
    /// Get usage log history for an equipment, newest first.
    fn get_usage_log(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Vec<UsageLogDto>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let records: Vec<UsageLogDto> = sqlx::query_as::<_, UsageLogDto>(
                r#"SELECT * FROM equipment_usage_log
                   WHERE equipment_id = $1 AND tenant_id = $2
                   ORDER BY started_at DESC"#,
            )
            .bind(id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(records)
        })
    }
    /// Get aggregated usage summary for an equipment.
    fn get_usage_summary(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Option<UsageSummaryDto>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let summary: Option<UsageSummaryDto> = sqlx::query_as::<_, UsageSummaryDto>(
                r#"SELECT
                    $1 as equipment_id,
                    COALESCE(SUM(hours_operated), 0.0) as total_hours,
                    COUNT(*) as total_sessions,
                    CASE WHEN COUNT(*) > 0 THEN COALESCE(SUM(hours_operated), 0.0) / COUNT(*)::f64 ELSE 0.0 END as avg_hours_per_session,
                    MIN(started_at) as first_used,
                    MAX(started_at) as last_used
                   FROM equipment_usage_log
                   WHERE equipment_id = $2 AND tenant_id = $3"#,
            )
            .bind(id)
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(summary)
        })
    }

    fn record_usage(
        &self,
        tid: TenantId,
        equipment_id: Uuid,
        worker_id: Option<Uuid>,
        task_id: Option<Uuid>,
        operation_type: Option<&str>,
        started_at: chrono::DateTime<chrono::Utc>,
        ended_at: Option<chrono::DateTime<chrono::Utc>>,
        hours_operated: f64,
        note: Option<&str>,
    ) -> RepositoryFuture<Option<UsageLogDto>> {
        let pool = self.pool.clone();
        let op_type_owned = operation_type.map(|s| s.to_string());
        let note_owned = note.map(|s| s.to_string());
        Box::pin(async move {
            let now = Utc::now();
            let started = started_at;
            let ended = ended_at.unwrap_or(started);
            let hours_operated_val = if hours_operated > 0.0 {
                hours_operated
            } else if ended > started {
                (ended - started).num_seconds() as f64 / 3600.0
            } else {
                0.0
            };

            let record: UsageLogDto = sqlx::query_as::<_, UsageLogDto>(
                r#"INSERT INTO equipment_usage_log
                    (id, equipment_id, tenant_id, worker_id, task_id, operation_type,
                     started_at, ended_at, hours_operated, note, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                RETURNING id, equipment_id, tenant_id, worker_id, task_id, operation_type,
                          started_at, ended_at, hours_operated, note, created_at, updated_at"#,
            )
            .bind(Uuid::new_v4())
            .bind(equipment_id)
            .bind(tid)
            .bind(worker_id)
            .bind(task_id)
            .bind(op_type_owned)
            .bind(started)
            .bind(ended)
            .bind(hours_operated_val)
            .bind(note_owned)
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(Some(record))
        })
    }

    fn get_depreciation(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Option<EquipmentDepreciationDto>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query(
                r#"SELECT
                    id as equipment_id,
                    tenant_id,
                    original_cost,
                    salvage_value,
                    purchase_date,
                    depreciation_method,
                    useful_life_years,
                    COALESCE(
                        (SELECT COALESCE(SUM(depreciation_amount), 0.0)
                         FROM equipment_depreciation_schedule
                         WHERE equipment_id = $1 AND tenant_id = $2),
                        0.0
                    ) as accumulated_depreciation,
                    now() as created_at,
                    now() as updated_at
                FROM equipment
                WHERE id = $1 AND tenant_id = $2"#,
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            match row {
                Some(r) => {
                    let method_str: String = r.try_get("depreciation_method")?;
                    let method = match method_str.as_str() {
                        "double_declining" => DepreciationMethod::DoubleDeclining,
                        _ => DepreciationMethod::StraightLine,
                    };
                    Ok(Some(EquipmentDepreciationDto {
                        equipment_id: r.try_get("equipment_id")?,
                        tenant_id: r.try_get("tenant_id")?,
                        original_cost: r.try_get::<f64, _>("original_cost")?,
                        salvage_value: r.try_get::<f64, _>("salvage_value")?,
                        purchase_date: r.try_get::<Option<DateTime<Utc>>, _>("purchase_date")?,
                        depreciation_method: method,
                        useful_life_years: r.try_get::<i32, _>("useful_life_years")? as u32,
                        accumulated_depreciation: r
                            .try_get::<f64, _>("accumulated_depreciation")?,
                        net_book_value: r.try_get::<f64, _>("original_cost")?
                            - r.try_get::<f64, _>("accumulated_depreciation")?,
                        created_at: r.try_get("created_at")?,
                        updated_at: r.try_get("updated_at")?,
                    }))
                }
                None => Ok(None),
            }
        })
    }

    fn get_depreciation_schedule(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Vec<DepreciationScheduleEntry>> {
        let pool = self.pool.clone();
        let now = Utc::now();
        Box::pin(async move {
            // If schedule entries exist in DB, use them; otherwise compute on the fly
            let rows: Vec<DepreciationScheduleEntry> = sqlx::query_as::<_, DepreciationScheduleEntry>(
                "SELECT year as year, depreciation_amount as depreciation_amount, accumulated_depreciation as accumulated_depreciation, net_book_value as net_book_value FROM equipment_depreciation_schedule WHERE equipment_id = $1 AND tenant_id = $2 ORDER BY year",
            )
            .bind(id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            if rows.is_empty() {
                // Compute schedule dynamically from equipment financials
                let eq_row = sqlx::query(
                    r#"SELECT original_cost, salvage_value, purchase_date, depreciation_method, useful_life_years
                       FROM equipment WHERE id = $1 AND tenant_id = $2"#,
                )
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                match eq_row {
                    Some(r) => {
                        let cost: f64 = r.try_get("original_cost")?;
                        let salvage: f64 = r.try_get("salvage_value")?;
                        let method_str: String = r.try_get("depreciation_method")?;
                        let life: i32 = r.try_get("useful_life_years")?;
                        let purchase: Option<DateTime<Utc>> = r.try_get("purchase_date")?;

                        let depreciable = cost - salvage;
                        let life_u32 = life.max(1) as u32;
                        let start_year = purchase.map(|d| d.year()).unwrap_or_else(|| now.year());

                        let mut entries = Vec::new();
                        let mut accumulated = 0.0;

                        for year_offset in 0..life_u32 {
                            let year = start_year + year_offset as i32;
                            let amount = if method_str == "double_declining" {
                                let rate = 2.0 / life_u32 as f64;
                                let book_value = cost - accumulated;
                                (book_value * rate).min(depreciable - accumulated).max(0.0)
                            } else {
                                // Straight-line
                                depreciable / life_u32 as f64
                            };

                            accumulated += amount;
                            let net_book = (cost - accumulated).max(salvage);

                            entries.push(DepreciationScheduleEntry {
                                year,
                                depreciation_amount: amount,
                                accumulated_depreciation: accumulated,
                                net_book_value: net_book,
                            });
                        }

                        Ok(entries)
                    }
                    None => Ok(Vec::new()),
                }
            } else {
                Ok(rows)
            }
        })
    }
}
