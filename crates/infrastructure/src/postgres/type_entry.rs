use crate::postgres::error_mapper::map_db_error;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::type_system::{
    CreateTypeEntryDto, PendingTypeEntryDto, TypeEntry, TypeEntryUsage, UpdateTypeEntryDto,
};
use agrocore_domain::repositories::{RepositoryFuture, TypeEntryRepository};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgTypeEntryRepo);

impl TypeEntryRepository for PgTypeEntryRepo {
    fn find_by_id(&self, id: Uuid) -> RepositoryFuture<Option<TypeEntry>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, TypeEntry>("SELECT id, category_id, parent_id, tenant_id, name, code, description, is_active, sort_order, created_at, updated_at FROM type_entries WHERE id = $1")
                .bind(id)
                .fetch_optional(&pool)
                .await
                .map_err(map_db_error)
        })
    }

    fn find_by_ids(&self, ids: &[Uuid]) -> RepositoryFuture<Vec<TypeEntry>> {
        let pool = self.pool.clone();
        let ids_vec = ids.to_vec();
        Box::pin(async move {
            if ids_vec.is_empty() {
                return Ok(Vec::new());
            }
            sqlx::query_as::<_, TypeEntry>(
                "SELECT id, category_id, parent_id, tenant_id, name, code, description, is_active, sort_order, created_at, updated_at FROM type_entries WHERE id = ANY($1) AND is_active = true",
            )
            .bind(&ids_vec)
            .fetch_all(&pool)
            .await
            .map_err(map_db_error)
        })
    }

    fn find_by_category(
        &self,
        tid: TenantId,
        category_id: Uuid,
    ) -> RepositoryFuture<Vec<TypeEntry>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            // Return global entries (tenant_id IS NULL) + tenant-specific entries
            sqlx::query_as::<_, TypeEntry>(
                "SELECT id, category_id, parent_id, tenant_id, name, code, description, is_active, sort_order, created_at, updated_at FROM type_entries
                 WHERE category_id = $1 AND is_active = true
                   AND (tenant_id IS NULL OR tenant_id = $2)
                 ORDER BY sort_order, name",
            )
            .bind(category_id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(map_db_error)
        })
    }

    fn find_by_category_and_code(
        &self,
        tid: TenantId,
        category_id: Uuid,
        code: &str,
    ) -> RepositoryFuture<Option<TypeEntry>> {
        let pool = self.pool.clone();
        let code = code.to_string();
        Box::pin(async move {
            // Prefer tenant-specific over global
            sqlx::query_as::<_, TypeEntry>(
                "SELECT id, category_id, parent_id, tenant_id, name, code, description, is_active, sort_order, created_at, updated_at FROM type_entries
                 WHERE category_id = $1 AND code = $2 AND is_active = true
                   AND (tenant_id IS NULL OR tenant_id = $3)
                 ORDER BY tenant_id IS NULL
                 LIMIT 1",
            )
            .bind(category_id)
            .bind(&code)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(map_db_error)
        })
    }

    fn find_children(&self, tid: TenantId, parent_id: Uuid) -> RepositoryFuture<Vec<TypeEntry>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, TypeEntry>(
                "SELECT id, category_id, parent_id, tenant_id, name, code, description, is_active, sort_order, created_at, updated_at FROM type_entries
                 WHERE parent_id = $1 AND is_active = true
                   AND (tenant_id IS NULL OR tenant_id = $2)
                 ORDER BY sort_order, name",
            )
            .bind(parent_id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(map_db_error)
        })
    }

    fn find_root_entries(
        &self,
        tid: TenantId,
        category_id: Uuid,
    ) -> RepositoryFuture<Vec<TypeEntry>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, TypeEntry>(
                "SELECT id, category_id, parent_id, tenant_id, name, code, description, is_active, sort_order, created_at, updated_at FROM type_entries
                 WHERE category_id = $1 AND parent_id IS NULL AND is_active = true
                   AND (tenant_id IS NULL OR tenant_id = $2)
                 ORDER BY sort_order, name",
            )
            .bind(category_id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(map_db_error)
        })
    }

    fn create(&self, tid: TenantId, dto: CreateTypeEntryDto) -> RepositoryFuture<TypeEntry> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let pending = dto.pending_review.unwrap_or(false);
            sqlx::query_as::<_, TypeEntry>(
                "INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, description, sort_order, pending_review)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                 RETURNING *",
            )
            .bind(dto.category_id)
            .bind(dto.parent_id)
            .bind(Some(tid))
            .bind(&dto.name)
            .bind(&dto.code)
            .bind(&dto.description)
            .bind(dto.sort_order.unwrap_or(0))
            .bind(pending)
            .fetch_one(&pool)
            .await
            .map_err(map_db_error)
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateTypeEntryDto,
    ) -> RepositoryFuture<Option<TypeEntry>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let mut parts = Vec::new();
            let mut idx = 1u32;

            if let Some(parent_id) = &dto.parent_id {
                parts.push(format!("parent_id = ${}", idx));
                idx += 1;
            }
            if let Some(ref name) = dto.name {
                parts.push(format!("name = ${}", idx));
                idx += 1;
            }
            if let Some(ref code) = dto.code {
                parts.push(format!("code = ${}", idx));
                idx += 1;
            }
            if let Some(ref description) = dto.description {
                parts.push(format!("description = ${}", idx));
                idx += 1;
            }
            if let Some(is_active) = dto.is_active {
                parts.push(format!("is_active = ${}", idx));
                idx += 1;
            }
            if let Some(sort_order) = dto.sort_order {
                parts.push(format!("sort_order = ${}", idx));
                idx += 1;
            }

            if parts.is_empty() {
                return sqlx::query_as::<_, TypeEntry>(
                    "SELECT id, category_id, parent_id, tenant_id, name, code, description, is_active, sort_order, created_at, updated_at FROM type_entries WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(&pool)
                .await
                .map_err(map_db_error);
            }

            let id_param = idx;
            idx += 1;
            let tid_param = idx;

            let query = format!(
                "UPDATE type_entries SET {}, updated_at = NOW() WHERE id = ${} AND (tenant_id IS NULL OR tenant_id = ${}) RETURNING *",
                parts.join(", "),
                id_param,
                tid_param
            );

            let mut builder = sqlx::query_as::<_, TypeEntry>(&query);

            if let Some(parent_id) = &dto.parent_id {
                builder = builder.bind(*parent_id);
            }
            if let Some(ref name) = dto.name {
                builder = builder.bind(name.as_str());
            }
            if let Some(ref code) = dto.code {
                builder = builder.bind(code.as_str());
            }
            if let Some(ref description) = dto.description {
                builder = builder.bind(description.as_ref().map(|s| s.as_str()));
            }
            if let Some(is_active) = dto.is_active {
                builder = builder.bind(is_active);
            }
            if let Some(sort_order) = dto.sort_order {
                builder = builder.bind(sort_order);
            }
            builder = builder.bind(id);
            builder = builder.bind(tid);

            builder.fetch_optional(&pool).await.map_err(map_db_error)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query(
                "DELETE FROM type_entries WHERE id = $1 AND (tenant_id IS NULL OR tenant_id = $2)",
            )
            .bind(id)
            .bind(tid)
            .execute(&pool)
            .await
            .map_err(map_db_error)?;

            Ok(result.rows_affected() > 0)
        })
    }

    fn soft_delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query(
                "UPDATE type_entries SET is_active = false, updated_at = NOW()
                 WHERE id = $1 AND (tenant_id IS NULL OR tenant_id = $2)",
            )
            .bind(id)
            .bind(tid)
            .execute(&pool)
            .await
            .map_err(map_db_error)?;

            Ok(result.rows_affected() > 0)
        })
    }

    fn create_with_user_id(
        &self,
        tid: TenantId,
        dto: CreateTypeEntryDto,
        user_id: Uuid,
    ) -> RepositoryFuture<TypeEntry> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let pending = dto.pending_review.unwrap_or(true);
            sqlx::query_as::<_, TypeEntry>(
                "INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, description, sort_order, created_by_user_id, pending_review)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 RETURNING *",
            )
            .bind(dto.category_id)
            .bind(dto.parent_id)
            .bind(Some(tid))
            .bind(&dto.name)
            .bind(&dto.code)
            .bind(&dto.description)
            .bind(dto.sort_order.unwrap_or(0))
            .bind(user_id)
            .bind(pending)
            .fetch_one(&pool)
            .await
            .map_err(map_db_error)
        })
    }

    fn find_pending_review(&self) -> RepositoryFuture<Vec<PendingTypeEntryDto>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, PendingTypeEntryDto>("SELECT * FROM v_pending_type_entries")
                .fetch_all(&pool)
                .await
                .map_err(map_db_error)
        })
    }

    fn approve_entry(&self, entry_id: Uuid) -> RepositoryFuture<Option<TypeEntry>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, TypeEntry>(
                "UPDATE type_entries SET pending_review = false, updated_at = NOW()
                 WHERE id = $1 AND pending_review = true
                 RETURNING *",
            )
            .bind(entry_id)
            .fetch_optional(&pool)
            .await
            .map_err(map_db_error)
        })
    }

    fn merge_entry(
        &self,
        _tid: TenantId,
        old_entry_id: Uuid,
        new_entry_id: Uuid,
    ) -> RepositoryFuture<Vec<TypeEntryUsage>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            // Collect all entity tables and their type reference columns
            let entity_columns = vec![
                ("sites", "site_type_id"),
                ("sites", "crop_type_id"),
                ("orders", "order_type_id"),
                ("orders", "status_id"),
                ("equipment", "type_id"),
                ("inventory_items", "category_id"),
                ("inventory_items", "unit_of_measure_id"),
                ("inventory_items", "inventory_method_id"),
                ("water_sources", "source_type_id"),
                ("water_usage", "irrigation_method_id"),
                ("animals", "species_id"),
                ("animals", "livestock_type_id"),
                ("animals", "status_id"),
                ("livestock", "livestock_type_id"),
                ("compliance_checklists", "type_id"),
                ("compliance_checklists", "status_id"),
                ("buildings", "type_id"),
                ("pac_applications", "status_id"),
                ("harvest_lots", "crop_type_id"),
                ("harvest_lots", "status_id"),
                ("workers", "contract_type_id"),
                ("settings", "value_type_id"),
                ("cost_centers", "type_id"),
                ("financial_records", "type_id"),
                ("iot_devices", "device_type_id"),
                ("breeds", "species_id"),
                ("growing_degree_days", "crop_type_id"),
                ("tasks", "execution_mode_id"),
                ("weather_stations", "station_type_id"),
                ("pest_risks", "risk_level_id"),
                ("plant_protection_records", "license_type_id"),
                ("worker_task_statuses", "status_id"),
                ("inventory_transactions", "transaction_type_id"),
            ];

            let mut usages = Vec::new();

            for (table, column) in &entity_columns {
                // Check if the table exists and has the column
                let table_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1)",
                )
                .bind(table)
                .fetch_optional(&pool)
                .await
                .map_err(map_db_error)?
                .unwrap_or(false);

                if !table_exists {
                    continue;
                }

                // Find the name column for this table
                let name_col = if *table == "sites" {
                    "label"
                } else if *table == "equipment" {
                    "label"
                } else if *table == "inventory_items" {
                    "name"
                } else if *table == "water_sources" {
                    "name"
                } else if *table == "animals" {
                    "identifier"
                } else if *table == "buildings" {
                    "label"
                } else {
                    "name"
                };

                // Check if the column exists
                let col_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = $1 AND column_name = $2)",
                )
                .bind(table)
                .bind(column)
                .fetch_optional(&pool)
                .await
                .map_err(map_db_error)?
                .unwrap_or(false);

                if !col_exists {
                    continue;
                }

                // Find entities using the old entry
                let query = format!(
                    "SELECT id, {} as entity_name FROM {} WHERE {} = $1",
                    name_col, table, column
                );
                let rows = sqlx::query_as::<_, (Uuid, String)>(&query)
                    .bind(old_entry_id)
                    .fetch_all(&pool)
                    .await
                    .map_err(map_db_error)?;

                for (entity_id, entity_name) in rows {
                    usages.push(TypeEntryUsage {
                        entity_type: table.to_string(),
                        entity_id,
                        entity_name,
                    });

                    // Update the reference
                    let update_query = format!(
                        "UPDATE {} SET {} = $1, updated_at = NOW() WHERE id = $2",
                        table, column
                    );
                    sqlx::query(&update_query)
                        .bind(new_entry_id)
                        .bind(entity_id)
                        .execute(&pool)
                        .await
                        .map_err(map_db_error)?;
                }
            }

            // Mark the old entry as inactive and approved
            sqlx::query(
                "UPDATE type_entries SET is_active = false, pending_review = false, updated_at = NOW() WHERE id = $1",
            )
            .bind(old_entry_id)
            .execute(&pool)
            .await
            .map_err(map_db_error)?;

            Ok(usages)
        })
    }

    fn find_usages(&self, entry_id: Uuid) -> RepositoryFuture<Vec<TypeEntryUsage>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let entity_columns = vec![
                ("sites", "site_type_id", "label"),
                ("sites", "crop_type_id", "label"),
                ("orders", "order_type_id", "label"),
                ("orders", "status_id", "label"),
                ("equipment", "type_id", "label"),
                ("inventory_items", "category_id", "name"),
                ("inventory_items", "unit_of_measure_id", "name"),
                ("inventory_items", "inventory_method_id", "name"),
                ("water_sources", "source_type_id", "name"),
                ("water_usage", "irrigation_method_id", "id"),
                ("animals", "species_id", "identifier"),
                ("animals", "livestock_type_id", "identifier"),
                ("animals", "status_id", "identifier"),
                ("livestock", "livestock_type_id", "id"),
                ("compliance_checklists", "type_id", "id"),
                ("compliance_checklists", "status_id", "id"),
                ("buildings", "type_id", "label"),
                ("pac_applications", "status_id", "id"),
                ("harvest_lots", "crop_type_id", "label"),
                ("harvest_lots", "status_id", "label"),
                ("workers", "contract_type_id", "full_name"),
                ("settings", "value_type_id", "key"),
                ("cost_centers", "type_id", "name"),
                ("financial_records", "type_id", "description"),
                ("iot_devices", "device_type_id", "name"),
                ("breeds", "species_id", "name"),
                ("growing_degree_days", "crop_type_id", "id"),
                ("tasks", "execution_mode_id", "label"),
                ("weather_stations", "station_type_id", "name"),
                ("pest_risks", "risk_level_id", "id"),
                ("plant_protection_records", "license_type_id", "id"),
                ("worker_task_statuses", "status_id", "id"),
                ("inventory_transactions", "transaction_type_id", "id"),
            ];

            let mut usages = Vec::new();

            for (table, column, name_col) in &entity_columns {
                let table_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1)",
                )
                .bind(table)
                .fetch_optional(&pool)
                .await
                .map_err(map_db_error)?
                .unwrap_or(false);

                if !table_exists {
                    continue;
                }

                let col_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = $1 AND column_name = $2)",
                )
                .bind(table)
                .bind(column)
                .fetch_optional(&pool)
                .await
                .map_err(map_db_error)?
                .unwrap_or(false);

                if !col_exists {
                    continue;
                }

                let query = format!(
                    "SELECT id, {} as entity_name FROM {} WHERE {} = $1",
                    name_col, table, column
                );
                let rows = sqlx::query_as::<_, (Uuid, String)>(&query)
                    .bind(entry_id)
                    .fetch_all(&pool)
                    .await
                    .map_err(map_db_error)?;

                for (entity_id, entity_name) in rows {
                    usages.push(TypeEntryUsage {
                        entity_type: table.to_string(),
                        entity_id,
                        entity_name,
                    });
                }
            }

            Ok(usages)
        })
    }
}
