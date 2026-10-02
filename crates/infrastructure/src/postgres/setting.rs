//! PostgreSQL implementation of [`SettingsRepository`] (tasks.md F1).

use std::str::FromStr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use agrocore_domain::entities::setting::{
    SettingEntry, SettingValueType, SettingWithDefault, UpdateSetting,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{RepositoryFuture, SettingsRepository};
use agrocore_shared::{SharedError, pg_repo};
use serde_json::Value as JsonValue;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

use crate::postgres::tenant_pool::TenantPool;

pg_repo!(PgSettingsRepo);

#[derive(FromRow, Clone)]
struct SettingRow {
    key: String,
    value: JsonValue,
    value_type: String,
    description: Option<String>,
    is_sensitive: bool,
    updated_at: DateTime<Utc>,
    updated_by: Option<Uuid>,
    tenant_id: Option<Uuid>,
}

impl SettingRow {
    fn into_entry(self) -> SettingEntry {
        SettingEntry {
            key: self.key,
            is_default: self.tenant_id.is_none(),
            value_type: SettingValueType::from_str(&self.value_type)
                // A row with an unknown type predates this code or was written
                // by hand. Treating it as a string still renders it; failing
                // here would make one bad row hide an entire settings page.
                .unwrap_or(SettingValueType::String),
            value: self.value,
            description: self.description,
            is_sensitive: self.is_sensitive,
            updated_at: self.updated_at,
            updated_by: self.updated_by,
        }
    }
}

/// Validate a value against its declared type.
///
/// Without this, `value_type = 'boolean'` with the string `"yes"` would be
/// stored and only fail later wherever the setting is read as a bool.
fn validate(value: &JsonValue, value_type: SettingValueType) -> Result<(), SharedError> {
    let ok = match value_type {
        SettingValueType::String | SettingValueType::Json => value.is_string() || value.is_object(),
        SettingValueType::Number => value.is_number(),
        SettingValueType::Boolean => value.is_boolean(),
        SettingValueType::Array => value.is_array(),
    };
    if ok {
        Ok(())
    } else {
        Err(SharedError::Validation(format!(
            "value does not match declared type {value_type}"
        )))
    }
}

impl SettingsRepository for PgSettingsRepo {
    fn list_effective(&self, tid: TenantId) -> RepositoryFuture<Vec<SettingWithDefault>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            // Tenant overrides and system defaults in one pass. The defaults
            // are not readable through the tenant policy on their own rows, so
            // `is_default` distinguishes them here rather than relying on the
            // caller to join.
            let rows: Vec<SettingRow> = sqlx::query_as(
                "SELECT key, value, value_type, description, is_sensitive, updated_at, updated_by,
                        tenant_id
                 FROM system_settings
                 WHERE tenant_id IS NULL OR tenant_id = $1
                 ORDER BY key",
            )
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(merge_defaults(rows))
        })
    }

    fn get(&self, tid: TenantId, key: &str) -> RepositoryFuture<Option<SettingEntry>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let key = key.to_string();
        Box::pin(async move {
            let row: Option<SettingRow> = sqlx::query_as(
                "SELECT key, value, value_type, description, is_sensitive, updated_at, updated_by,
                        tenant_id
                 FROM system_settings
                 WHERE key = $1 AND (tenant_id IS NULL OR tenant_id = $2)
                 ORDER BY tenant_id NULLS LAST
                 LIMIT 1",
            )
            .bind(&key)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(row.map(SettingRow::into_entry))
        })
    }

    fn set(&self, tid: TenantId, user_id: Uuid, update: UpdateSetting) -> RepositoryFuture<()> {
        self.set_many(tid, user_id, vec![update])
    }

    fn set_many(
        &self,
        tid: TenantId,
        user_id: Uuid,
        updates: Vec<UpdateSetting>,
    ) -> RepositoryFuture<()> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

            for update in updates {
                // The declared type comes from the default row when one
                // exists, otherwise from this tenant's existing override.
                //
                // Reading only the override was wrong: on a fresh install every
                // key has a default and no override, so a string would have been
                // written against `backup.enabled` and only failed later at the
                // point of use.
                let existing: Option<(String,)> = sqlx::query_as(
                    "SELECT value_type FROM system_settings
                     WHERE key = $1 AND (tenant_id IS NULL OR tenant_id = $2)
                     ORDER BY tenant_id NULLS LAST
                     LIMIT 1",
                )
                .bind(&update.key)
                .bind(tid)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                let value_type = match existing {
                    Some((t,)) => SettingValueType::from_str(&t).unwrap_or_default(),
                    None => infer_type(&update.value),
                };
                validate(&update.value, value_type)?;

                sqlx::query(
                    "INSERT INTO system_settings
                         (tenant_id, key, value, value_type, updated_by, updated_at)
                     VALUES ($1, $2, $3, $4, $5, NOW())
                     ON CONFLICT (COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid), key)
                     DO UPDATE SET value = EXCLUDED.value,
                                   value_type = EXCLUDED.value_type,
                                   updated_by = EXCLUDED.updated_by,
                                   updated_at = NOW()",
                )
                .bind(tid)
                .bind(&update.key)
                .bind(&update.value)
                .bind(value_type.as_str())
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            }

            tx.commit()
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(())
        })
    }

    fn reset(&self, tid: TenantId, key: &str) -> RepositoryFuture<bool> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let key = key.to_string();
        Box::pin(async move {
            // Reports whether an override actually existed, so the UI can tell
            // "reset" from "there was nothing to reset".
            let result =
                sqlx::query("DELETE FROM system_settings WHERE tenant_id = $1 AND key = $2")
                    .bind(tid)
                    .bind(&key)
                    .execute(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(result.rows_affected() > 0)
        })
    }

    fn set_default(
        &self,
        key: &str,
        value: JsonValue,
        value_type: SettingValueType,
        description: Option<String>,
        is_sensitive: bool,
    ) -> RepositoryFuture<()> {
        // Not tenant-scoped, so this deliberately uses the unscoped handle.
        // System defaults are written by migrations and by an explicit admin
        // action, never from a tenant request.
        let pool = TenantPool::unscoped(&self.pool);
        let key = key.to_string();
        Box::pin(async move {
            validate(&value, value_type)?;
            sqlx::query(
                "INSERT INTO system_settings
                     (tenant_id, key, value, value_type, description, is_sensitive, updated_at)
                 VALUES (NULL, $1, $2, $3, $4, $5, NOW())
                 ON CONFLICT (COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid), key)
                 DO UPDATE SET value = EXCLUDED.value,
                               value_type = EXCLUDED.value_type,
                               description = EXCLUDED.description,
                               is_sensitive = EXCLUDED.is_sensitive,
                               updated_at = NOW()",
            )
            .bind(&key)
            .bind(&value)
            .bind(value_type.as_str())
            .bind(&description)
            .bind(is_sensitive)
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(())
        })
    }

    fn restore_defaults(&self) -> RepositoryFuture<Vec<String>> {
        let pool = TenantPool::unscoped(&self.pool);
        Box::pin(async move {
            // Deletes tenant overrides rather than rewriting them: the point of
            // "restore defaults" is that no tenant row remains.
            let result = sqlx::query("DELETE FROM system_settings WHERE tenant_id IS NOT NULL")
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let keys: Vec<String> = sqlx::query_scalar(
                "SELECT key FROM system_settings WHERE tenant_id IS NULL ORDER BY key",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(keys)
        })
    }

    fn list_keys(&self) -> RepositoryFuture<Vec<(String, SettingValueType, Option<String>, bool)>> {
        let pool = TenantPool::unscoped(&self.pool);
        Box::pin(async move {
            let rows: Vec<(String, String, Option<String>, bool)> = sqlx::query_as(
                "SELECT key, value_type, description, is_sensitive
                 FROM system_settings WHERE tenant_id IS NULL ORDER BY key",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(rows
                .into_iter()
                .map(|(k, t, d, s)| (k, SettingValueType::from_str(&t).unwrap_or_default(), d, s))
                .collect())
        })
    }
}

/// Derive the declared type from the value on first write.
fn infer_type(value: &JsonValue) -> SettingValueType {
    match value {
        JsonValue::Bool(_) => SettingValueType::Boolean,
        JsonValue::Number(_) => SettingValueType::Number,
        JsonValue::Array(_) => SettingValueType::Array,
        JsonValue::Object(_) => SettingValueType::Json,
        _ => SettingValueType::String,
    }
}

/// Collapse defaults and overrides into one effective entry per key.
fn merge_defaults(rows: Vec<SettingRow>) -> Vec<SettingWithDefault> {
    let mut defaults: std::collections::HashMap<String, JsonValue> =
        std::collections::HashMap::new();
    let mut effective: std::collections::HashMap<String, SettingWithDefault> =
        std::collections::HashMap::new();

    // Defaults first, so an override always overwrites them.
    for row in rows.iter().filter(|r| r.tenant_id.is_none()) {
        defaults.insert(row.key.clone(), row.value.clone());
        effective.insert(
            row.key.clone(),
            SettingWithDefault {
                entry: row.clone().into_entry(),
                default_value: None,
            },
        );
    }

    for row in rows.into_iter().filter(|r| r.tenant_id.is_some()) {
        let default_value = defaults.get(&row.key).cloned();
        effective.insert(
            row.key.clone(),
            SettingWithDefault {
                entry: row.into_entry(),
                default_value,
            },
        );
    }

    let mut out: Vec<SettingWithDefault> = effective.into_values().collect();
    out.sort_by(|a, b| a.entry.key.cmp(&b.entry.key));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn infer_type_matches_the_json_shape() {
        assert_eq!(infer_type(&json!(true)), SettingValueType::Boolean);
        assert_eq!(infer_type(&json!(42)), SettingValueType::Number);
        assert_eq!(infer_type(&json!([1, 2])), SettingValueType::Array);
        assert_eq!(infer_type(&json!({"a": 1})), SettingValueType::Json);
        assert_eq!(infer_type(&json!("x")), SettingValueType::String);
    }

    #[test]
    fn validate_rejects_a_mismatched_type() {
        assert!(validate(&json!(true), SettingValueType::Boolean).is_ok());
        assert!(validate(&json!("yes"), SettingValueType::Boolean).is_err());
        assert!(validate(&json!("x"), SettingValueType::Number).is_err());
        assert!(validate(&json!(7), SettingValueType::String).is_err());
    }

    fn row(key: &str, tenant: Option<Uuid>, value: JsonValue) -> SettingRow {
        SettingRow {
            key: key.to_string(),
            value,
            value_type: "string".into(),
            description: None,
            is_sensitive: false,
            updated_at: chrono::Utc::now(),
            updated_by: None,
            tenant_id: tenant,
        }
    }

    #[test]
    fn override_wins_over_default_and_records_it() {
        let tenant = Uuid::new_v4();
        let rows = vec![
            row("a.b", None, json!("default")),
            row("a.b", Some(tenant), json!("mine")),
            row("a.c", None, json!("only-default")),
        ];
        let merged = merge_defaults(rows);
        assert_eq!(merged.len(), 2);

        let b = merged.iter().find(|m| m.entry.key == "a.b").unwrap();
        assert_eq!(b.entry.value, json!("mine"));
        assert!(!b.entry.is_default);
        assert_eq!(b.default_value, Some(json!("default")));

        let c = merged.iter().find(|m| m.entry.key == "a.c").unwrap();
        assert!(c.entry.is_default);
        assert_eq!(c.default_value, None);
    }

    #[test]
    fn unknown_value_type_does_not_hide_the_row() {
        let mut r = row("x.y", Some(Uuid::new_v4()), json!("v"));
        r.value_type = "something_new".into();
        assert_eq!(r.into_entry().value_type, SettingValueType::String);
    }
}
