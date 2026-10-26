use crate::postgres::tenant_pool::TenantPool;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::weather::SoilMoistureAlert;
use agrocore_domain::repositories::{RepositoryFuture, SoilMoistureAlertRepo};
use agrocore_shared::{SharedError};
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgSoilMoistureAlertRepo);

impl SoilMoistureAlertRepo for PgSoilMoistureAlertRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<SoilMoistureAlert>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            sqlx::query_as::<_, SoilMoistureAlert>(
                "SELECT * FROM soil_moisture_alerts WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_unresolved(
        &self,
        tid: TenantId,
        station_id: Uuid,
    ) -> RepositoryFuture<Vec<SoilMoistureAlert>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            // Alerts are linked to configs via config_id, not directly to stations.
            // We join soil_moisture_configs to filter by station_id.
            sqlx::query_as::<_, SoilMoistureAlert>(
                r#"SELECT sa.* FROM soil_moisture_alerts sa
                   JOIN soil_moisture_configs smc ON sa.config_id = smc.id
                   WHERE sa.tenant_id = $1 AND smc.station_id = $2 AND sa.is_resolved = false
                   ORDER BY sa.alert_time DESC"#,
            )
            .bind(tid)
            .bind(station_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(
        &self,
        tid: TenantId,
        alert: SoilMoistureAlert,
    ) -> RepositoryFuture<SoilMoistureAlert> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            sqlx::query_as::<_, SoilMoistureAlert>(
                r#"INSERT INTO soil_moisture_alerts (
                    id, tenant_id, config_id, alert_time,
                    moisture_percent, threshold_percent, action_taken,
                    is_resolved, resolved_at, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
                RETURNING *"#,
            )
            .bind(alert.id)
            .bind(tid)
            .bind(alert.config_id)
            .bind(alert.alert_time)
            .bind(alert.moisture_percent)
            .bind(alert.threshold_percent)
            .bind(alert.action_taken)
            .bind(alert.is_resolved)
            .bind(alert.resolved_at)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn mark_resolved(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            sqlx::query(
                "UPDATE soil_moisture_alerts SET is_resolved = true, resolved_at = NOW(), updated_at = NOW() WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .execute(&pool)
            .await
            .map(|r| r.rows_affected() > 0)
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
