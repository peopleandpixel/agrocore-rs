//! PostgreSQL implementation of [`RoleRepository`] and [`UserPermissionOverrideRepository`].

use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::{
    Module, PermissionAction, PermissionEntry, PermissionScope, Role, RolePermissionMatrix,
    UserPermissionOverrides,
};
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, RoleRepository, UserPermissionOverrideRepository,
};
use agrocore_shared::SharedError;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::postgres::error_mapper::map_db_error;
use crate::postgres::tenant_pool::TenantPool;

agrocore_shared::pg_repo!(PgRoleRepo);

impl RoleRepository for PgRoleRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Role>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            let row: Option<(Uuid, String, String, bool, chrono::DateTime<Utc>, chrono::DateTime<Utc>)> = sqlx::query_as(
                "SELECT id, name, COALESCE(description, ''), is_system, created_at, updated_at FROM roles WHERE id = $1 AND tenant_id = $2"
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(map_db_error)?;

            Ok(row.map(|(id, name, description, is_system, created_at, updated_at)| Role {
                id,
                name,
                description: if description.is_empty() { None } else { Some(description) },
                permissions: Vec::new(),
                is_system,
                created_at,
                updated_at,
            }))
        })
    }

    fn find_by_name(&self, tid: TenantId, name: &str) -> RepositoryFuture<Option<Role>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let name = name.to_string();
        Box::pin(async move {
            let row: Option<(Uuid, String, String, bool, chrono::DateTime<Utc>, chrono::DateTime<Utc>)> = sqlx::query_as(
                "SELECT id, name, COALESCE(description, ''), is_system, created_at, updated_at FROM roles WHERE name = $1 AND tenant_id = $2"
            )
            .bind(&name)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(map_db_error)?;

            Ok(row.map(|(id, name, description, is_system, created_at, updated_at)| Role {
                id,
                name,
                description: if description.is_empty() { None } else { Some(description) },
                permissions: Vec::new(),
                is_system,
                created_at,
                updated_at,
            }))
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Role>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let page = p.effective_page();
        let per_page = p.effective_per_page();
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM roles WHERE tenant_id = $1"
            )
            .bind(tid)
            .fetch_one(&pool)
            .await
            .map_err(map_db_error)?;

            let rows: Vec<(Uuid, String, String, bool, chrono::DateTime<Utc>, chrono::DateTime<Utc>)> = sqlx::query_as(
                "SELECT id, name, COALESCE(description, ''), is_system, created_at, updated_at FROM roles WHERE tenant_id = $1 ORDER BY name LIMIT $2 OFFSET $3"
            )
            .bind(tid)
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(map_db_error)?;

            let data = rows.into_iter().map(|(id, name, description, is_system, created_at, updated_at)| Role {
                id,
                name,
                description: if description.is_empty() { None } else { Some(description) },
                permissions: Vec::new(),
                is_system,
                created_at,
                updated_at,
            }).collect();

            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: ((total as f64 / per_page as f64).ceil() as u64),
            })
        })
    }

    fn create(&self, tid: TenantId, name: String, description: Option<String>, by: Uuid) -> RepositoryFuture<Role> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            let now = Utc::now();
            let role = sqlx::query_as::<_, (Uuid, String, String, bool, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
                "INSERT INTO roles (tenant_id, name, description, is_system, created_at, updated_at)
                 VALUES ($1, $2, $3, false, $4, $4)
                 RETURNING id, name, COALESCE(description, ''), is_system, created_at, updated_at"
            )
            .bind(tid)
            .bind(&name)
            .bind(description.as_deref())
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(map_db_error)?;

            // Set up default empty permissions for all modules
            let all_modules = Module::all();
            for module in &all_modules {
                let module_str = serde_json::to_string(module).map_err(|e| SharedError::Internal(format!("serialize module: {}", e)))?;
                sqlx::query(
                    "INSERT INTO role_permissions (role_id, tenant_id, module, actions, scope)
                     VALUES ($1, $2, $3, '[]'::jsonb, 'tenant')
                     ON CONFLICT (role_id, module) DO NOTHING"
                )
                .bind(role.0)
                .bind(tid)
                .bind(module_str)
                .execute(&pool)
                .await
                .map_err(map_db_error)?;
            }

            // Audit log
            sqlx::query(
                "INSERT INTO audit_logs (tenant_id, user_id, action, entity_type, entity_id, created_at)
                 VALUES ($1, $2, 'created', 'role', $3, NOW())"
            )
            .bind(tid)
            .bind(by)
            .bind(role.0)
            .execute(&pool)
            .await
            .ok();

            Ok(Role {
                id: role.0,
                name: role.1,
                description: if role.2.is_empty() { None } else { Some(role.2) },
                permissions: Vec::new(),
                is_system: role.3,
                created_at: role.4,
                updated_at: role.5,
            })
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        by: Uuid,
    ) -> RepositoryFuture<Option<Role>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            let mut sets = Vec::new();
            let mut params: Vec<Box<dyn sqlx::types::Type<sqlx::Postgres> + sqlx::Encode<'_, sqlx::Postgres>>> = Vec::new();
            let mut idx = 1usize;

            if let Some(n) = name {
                sets.push("name = $1".to_string());
                params.push(Box::new(n));
                idx += 1;
            }
            if let Some(d) = description {
                sets.push("description = $1".to_string());
                params.push(Box::new(d));
                idx += 1;
            }

            if sets.is_empty() {
                return self.find_by_id(tid, id);
            }

            sets.push("updated_at = NOW()".to_string());
            let query = format!(
                "UPDATE roles SET {} WHERE id = ${} AND tenant_id = ${} AND is_system = false
                 RETURNING id, name, COALESCE(description, ''), is_system, created_at, updated_at",
                sets.join(", "),
                idx,
                idx + 1
            );

            let mut q = sqlx::query_as::<_, (Uuid, String, String, bool, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(&query);
            for p in params {
                q = q.bind(*p);
            }
            q = q.bind(id).bind(tid);

            let row = q.fetch_optional(&pool).await.map_err(map_db_error)?;

            // Audit log
            if row.is_some() {
                sqlx::query(
                    "INSERT INTO audit_logs (tenant_id, user_id, action, entity_type, entity_id, created_at)
                     VALUES ($1, $2, 'updated', 'role', $3, NOW())"
                )
                .bind(tid)
                .bind(by)
                .bind(id)
                .execute(&pool)
                .await
                .ok();
            }

            Ok(row.map(|(id, name, description, is_system, created_at, updated_at)| Role {
                id,
                name,
                description: if description.is_empty() { None } else { Some(description) },
                permissions: Vec::new(),
                is_system,
                created_at,
                updated_at,
            }))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            let result = sqlx::query(
                "DELETE FROM roles WHERE id = $1 AND tenant_id = $2 AND is_system = false"
            )
            .bind(id)
            .bind(tid)
            .execute(&pool)
            .await
            .map_err(map_db_error)?;

            Ok(result.rows_affected() > 0)
        })
    }

    fn get_permissions(&self, tid: TenantId, role_id: Uuid) -> RepositoryFuture<Vec<PermissionEntry>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            let rows: Vec<(String, serde_json::Value, String)> = sqlx::query_as(
                "SELECT module, actions, scope FROM role_permissions WHERE role_id = $1 AND tenant_id = $2"
            )
            .bind(role_id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(map_db_error)?;

            let mut permissions = Vec::new();
            for (module_str, actions_val, scope_str) in rows {
                let module: Module = serde_json::from_str(&module_str)
                    .unwrap_or_else(|_| Module::Sites); // fallback
                let actions: Vec<PermissionAction> = serde_json::from_value(actions_val)
                    .unwrap_or_default();
                let scope: PermissionScope = match scope_str.as_str() {
                    "all" => PermissionScope::All,
                    "own" => PermissionScope::Own,
                    _ => PermissionScope::Tenant,
                };
                permissions.push(PermissionEntry { module, actions, scope });
            }

            Ok(permissions)
        })
    }

    fn set_permissions(
        &self,
        tid: TenantId,
        role_id: Uuid,
        permissions: Vec<PermissionEntry>,
        by: Uuid,
    ) -> RepositoryFuture<()> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            // Delete existing permissions
            sqlx::query("DELETE FROM role_permissions WHERE role_id = $1 AND tenant_id = $2")
                .bind(role_id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(map_db_error)?;

            // Insert new permissions
            for entry in permissions {
                let module_str = serde_json::to_string(&entry.module)
                    .map_err(|e| SharedError::Internal(format!("serialize module: {}", e)))?;
                let actions_val = serde_json::to_value(&entry.actions)
                    .map_err(|e| SharedError::Internal(format!("serialize actions: {}", e)))?;
                let scope_str = match entry.scope {
                    PermissionScope::All => "all",
                    PermissionScope::Own => "own",
                    PermissionScope::Tenant => "tenant",
                };

                sqlx::query(
                    "INSERT INTO role_permissions (role_id, tenant_id, module, actions, scope, updated_at)
                     VALUES ($1, $2, $3, $4, $5, NOW())
                     ON CONFLICT (role_id, module) DO UPDATE SET actions = $4, scope = $5, updated_at = NOW()"
                )
                .bind(role_id)
                .bind(tid)
                .bind(module_str)
                .bind(actions_val)
                .bind(scope_str)
                .execute(&pool)
                .await
                .map_err(map_db_error)?;
            }

            // Audit log
            sqlx::query(
                "INSERT INTO audit_logs (tenant_id, user_id, action, entity_type, entity_id, created_at)
                 VALUES ($1, $2, 'updated', 'role_permissions', $3, NOW())"
            )
            .bind(tid)
            .bind(by)
            .bind(role_id)
            .execute(&pool)
            .await
            .ok();

            Ok(())
        })
    }

    fn set_module_permissions(
        &self,
        tid: TenantId,
        role_id: Uuid,
        module: Module,
        actions: Vec<PermissionAction>,
        scope: PermissionScope,
        by: Uuid,
    ) -> RepositoryFuture<()> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let module_str = serde_json::to_string(&module)
            .map_err(|e| SharedError::Internal(format!("serialize module: {}", e)))?;
        let actions_val = serde_json::to_value(&actions)
            .map_err(|e| SharedError::Internal(format!("serialize actions: {}", e)))?;
        let scope_str = match scope {
            PermissionScope::All => "all",
            PermissionScope::Own => "own",
            PermissionScope::Tenant => "tenant",
        };

        Box::pin(async move {
            sqlx::query(
                "INSERT INTO role_permissions (role_id, tenant_id, module, actions, scope, updated_at)
                 VALUES ($1, $2, $3, $4, $5, NOW())
                 ON CONFLICT (role_id, module) DO UPDATE SET actions = $4, scope = $5, updated_at = NOW()"
            )
            .bind(role_id)
            .bind(tid)
            .bind(module_str)
            .bind(actions_val)
            .bind(scope_str)
            .execute(&pool)
            .await
            .map_err(map_db_error)?;

            // Audit log
            sqlx::query(
                "INSERT INTO audit_logs (tenant_id, user_id, action, entity_type, entity_id, created_at)
                 VALUES ($1, $2, 'updated', 'role_module_permissions', $3, NOW())"
            )
            .bind(tid)
            .bind(by)
            .bind(role_id)
            .execute(&pool)
            .await
            .ok();

            Ok(())
        })
    }

    fn get_all_permission_matrices(&self, tid: TenantId) -> RepositoryFuture<Vec<RolePermissionMatrix>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            // Get all roles
            let roles: Vec<(Uuid, String)> = sqlx::query_as(
                "SELECT id, name FROM roles WHERE tenant_id = $1 ORDER BY name"
            )
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(map_db_error)?;

            let mut matrices = Vec::new();
            for (role_id, role_name) in roles {
                let permissions = self.get_permissions(tid, role_id).await?;
                matrices.push(RolePermissionMatrix {
                    role_id,
                    role_name,
                    permissions,
                });
            }

            Ok(matrices)
        })
    }

    fn initialize_system_roles(&self, tid: TenantId, by: Uuid) -> RepositoryFuture<Vec<Role>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            let system_roles = vec![
                ("SuperAdmin", Some("Full system access across all tenants".to_string()), true),
                ("Admin", Some("Full access within tenant".to_string()), true),
                ("Manager", Some("Broad access with some restrictions".to_string()), true),
                ("Worker", Some("Operational access for field workers".to_string()), true),
                ("Viewer", Some("Read-only access".to_string()), true),
            ];

            let mut created = Vec::new();
            let now = Utc::now();

            for (name, description, is_system) in system_roles {
                // Check if role already exists
                let existing: Option<Uuid> = sqlx::query_scalar(
                    "SELECT id FROM roles WHERE name = $1 AND tenant_id = $2"
                )
                .bind(name)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(map_db_error)?;

                if existing.is_some() {
                    continue;
                }

                let row: (Uuid, String, String, bool, chrono::DateTime<Utc>, chrono::DateTime<Utc>) = sqlx::query_as(
                    "INSERT INTO roles (tenant_id, name, description, is_system, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $5)
                     RETURNING id, name, COALESCE(description, ''), is_system, created_at, updated_at"
                )
                .bind(tid)
                .bind(name)
                .bind(description.as_deref())
                .bind(is_system)
                .bind(now)
                .fetch_one(&pool)
                .await
                .map_err(map_db_error)?;

                // Set up default permissions using the database function
                sqlx::query(
                    "SELECT setup_default_role_permissions($1, $2, $3)"
                )
                .bind(row.0)
                .bind(tid)
                .bind(row.1.clone())
                .execute(&pool)
                .await
                .map_err(map_db_error)?;

                created.push(Role {
                    id: row.0,
                    name: row.1,
                    description: if row.2.is_empty() { None } else { Some(row.2) },
                    permissions: Vec::new(),
                    is_system: row.3,
                    created_at: row.4,
                    updated_at: row.5,
                });
            }

            // Audit log
            if !created.is_empty() {
                sqlx::query(
                    "INSERT INTO audit_logs (tenant_id, user_id, action, entity_type, entity_id, created_at)
                     VALUES ($1, $2, 'created', 'system_roles', $3, NOW())"
                )
                .bind(tid)
                .bind(by)
                .bind(Uuid::nil())
                .execute(&pool)
                .await
                .ok();
            }

            Ok(created)
        })
    }
}

// --- User Permission Override Repository ---

agrocore_shared::pg_repo!(PgUserPermissionOverrideRepo);

impl UserPermissionOverrideRepository for PgUserPermissionOverrideRepo {
    fn get_overrides(&self, tid: TenantId, user_id: Uuid) -> RepositoryFuture<UserPermissionOverrides> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            let grants: Vec<(String, serde_json::Value, String, chrono::DateTime<Utc>, Uuid)> = sqlx::query_as(
                "SELECT module, actions, scope, updated_at, updated_by FROM user_permission_overrides
                 WHERE user_id = $1 AND tenant_id = $2 AND is_grant = true"
            )
            .bind(user_id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(map_db_error)?;

            let revokes: Vec<(String, serde_json::Value, String)> = sqlx::query_as(
                "SELECT module, actions, scope FROM user_permission_overrides
                 WHERE user_id = $1 AND tenant_id = $2 AND is_grant = false"
            )
            .bind(user_id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(map_db_error)?;

            let mut additional_permissions = Vec::new();
            let max_updated = grants.iter().map(|(_, _, _, updated_at, _)| *updated_at).max().unwrap_or(Utc::now());
            let updated_by = grants.last().map(|(_, _, _, _, uid)| *uid).unwrap_or(Uuid::nil());

            for (module_str, actions_val, scope_str) in grants {
                let module: Module = serde_json::from_str(&module_str).unwrap_or(Module::Sites);
                let actions: Vec<PermissionAction> = serde_json::from_value(actions_val).unwrap_or_default();
                let scope: PermissionScope = match scope_str.as_str() {
                    "all" => PermissionScope::All,
                    "own" => PermissionScope::Own,
                    _ => PermissionScope::Tenant,
                };
                additional_permissions.push(PermissionEntry { module, actions, scope });
            }

            let mut restricted_permissions = Vec::new();
            for (module_str, actions_val, scope_str) in revokes {
                let module: Module = serde_json::from_str(&module_str).unwrap_or(Module::Sites);
                let actions: Vec<PermissionAction> = serde_json::from_value(actions_val).unwrap_or_default();
                let scope: PermissionScope = match scope_str.as_str() {
                    "all" => PermissionScope::All,
                    "own" => PermissionScope::Own,
                    _ => PermissionScope::Tenant,
                };
                restricted_permissions.push(PermissionEntry { module, actions, scope });
            }

            Ok(UserPermissionOverrides {
                user_id,
                additional_permissions,
                restricted_permissions,
                updated_at: max_updated,
                updated_by,
            })
        })
    }

    fn set_grant(
        &self,
        tid: TenantId,
        user_id: Uuid,
        module: Module,
        actions: Vec<PermissionAction>,
        scope: PermissionScope,
        by: Uuid,
    ) -> RepositoryFuture<()> {
        set_override(self.pool.clone(), tid, user_id, module, actions, scope, true, by)
    }

    fn set_revoke(
        &self,
        tid: TenantId,
        user_id: Uuid,
        module: Module,
        actions: Vec<PermissionAction>,
        scope: PermissionScope,
        by: Uuid,
    ) -> RepositoryFuture<()> {
        set_override(self.pool.clone(), tid, user_id, module, actions, scope, false, by)
    }

    fn remove_override(
        &self,
        tid: TenantId,
        user_id: Uuid,
        module: Module,
        is_grant: bool,
        by: Uuid,
    ) -> RepositoryFuture<bool> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let module_str = serde_json::to_string(&module)
            .map_err(|e| SharedError::Internal(format!("serialize module: {}", e)))?;

        Box::pin(async move {
            let result = sqlx::query(
                "DELETE FROM user_permission_overrides WHERE user_id = $1 AND tenant_id = $2 AND module = $3 AND is_grant = $4"
            )
            .bind(user_id)
            .bind(tid)
            .bind(&module_str)
            .bind(is_grant)
            .execute(&pool)
            .await
            .map_err(map_db_error)?;

            // Audit log
            if result.rows_affected() > 0 {
                sqlx::query(
                    "INSERT INTO audit_logs (tenant_id, user_id, action, entity_type, entity_id, created_at)
                     VALUES ($1, $2, 'updated', 'user_permission_override', $3, NOW())"
                )
                .bind(tid)
                .bind(by)
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            }

            Ok(result.rows_affected() > 0)
        })
    }

    fn clear_all_overrides(&self, tid: TenantId, user_id: Uuid, by: Uuid) -> RepositoryFuture<()> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            sqlx::query(
                "DELETE FROM user_permission_overrides WHERE user_id = $1 AND tenant_id = $2"
            )
            .bind(user_id)
            .bind(tid)
            .execute(&pool)
            .await
            .map_err(map_db_error)?;

            // Audit log
            sqlx::query(
                "INSERT INTO audit_logs (tenant_id, user_id, action, entity_type, entity_id, created_at)
                 VALUES ($1, $2, 'updated', 'user_permission_overrides', $3, NOW())"
            )
            .bind(tid)
            .bind(by)
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();

            Ok(())
        })
    }
}

fn set_override(
    pool: PgPool,
    tid: TenantId,
    user_id: Uuid,
    module: Module,
    actions: Vec<PermissionAction>,
    scope: PermissionScope,
    is_grant: bool,
    by: Uuid,
) -> RepositoryFuture<()> {
    let pool = TenantPool::new(&pool, tid.0);
    let module_str = serde_json::to_string(&module)
        .map_err(|e| SharedError::Internal(format!("serialize module: {}", e)))?;
    let actions_val = serde_json::to_value(&actions)
        .map_err(|e| SharedError::Internal(format!("serialize actions: {}", e)))?;
    let scope_str = match scope {
        PermissionScope::All => "all",
        PermissionScope::Own => "own",
        PermissionScope::Tenant => "tenant",
    };

    Box::pin(async move {
        sqlx::query(
            "INSERT INTO user_permission_overrides (user_id, tenant_id, module, actions, scope, is_grant, updated_by, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
             ON CONFLICT (user_id, module, is_grant) DO UPDATE SET actions = $4, scope = $5, updated_by = $7, updated_at = NOW()"
        )
        .bind(user_id)
        .bind(tid)
        .bind(module_str)
        .bind(actions_val)
        .bind(scope_str)
        .bind(is_grant)
        .bind(by)
        .execute(&pool)
        .await
        .map_err(map_db_error)?;

        // Audit log
        sqlx::query(
            "INSERT INTO audit_logs (tenant_id, user_id, action, entity_type, entity_id, created_at)
             VALUES ($1, $2, 'updated', 'user_permission_override', $3, NOW())"
        )
        .bind(tid)
        .bind(by)
        .bind(user_id)
        .execute(&pool)
        .await
        .ok();

        Ok(())
    })
}
