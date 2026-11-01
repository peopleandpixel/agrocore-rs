use crate::postgres::tenant_pool::TenantPool;
use agrocore_domain::entities::iot::{
    CreateIoTDeviceDto, DeviceStatus, IoTCommand, IoTDevice, IoTDeviceData, IoTTelemetry,
    UpdateIoTDeviceDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{IotDeviceRepository, RepositoryFuture};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgIotDeviceRepo);

impl IotDeviceRepository for PgIotDeviceRepo {
    fn find_by_device_id(
        &self,
        tid: TenantId,
        device_id: &str,
    ) -> RepositoryFuture<Option<IoTDevice>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let device_id_owned = device_id.to_owned();
        Box::pin(async move {
            sqlx::query_as::<_, IoTDevice>(
                "SELECT device_id, tenant_id, site_id, device FROM iot_devices WHERE device_id = $1 AND tenant_id = $2",
            )
            .bind(&device_id_owned)
            .bind(tid.0)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(
        &self,
        tid: TenantId,
        site_id: Option<Uuid>,
        status: Option<DeviceStatus>,
    ) -> RepositoryFuture<Vec<IoTDevice>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            let base_query = "SELECT device_id, tenant_id, site_id, device FROM iot_devices WHERE tenant_id = $1";

            let (query, has_filters) = if site_id.is_some() || status.is_some() {
                let mut q = base_query.to_string();
                if site_id.is_some() {
                    q.push_str(" AND site_id = $2");
                }
                if status.is_some() {
                    if site_id.is_some() {
                        q.push_str(" AND (device->>'status')::text = $3");
                    } else {
                        q.push_str(" AND (device->>'status')::text = $2");
                    }
                }
                (q, true)
            } else {
                (base_query.to_string(), false)
            };

            let mut query_builder = sqlx::query_as::<_, IoTDevice>(&query).bind(tid.0);

            if has_filters {
                if let Some(site) = site_id {
                    query_builder = query_builder.bind(site);
                }
                if let Some(st) = status {
                    query_builder = query_builder.bind(format!("{st:?}").to_lowercase());
                }
            }

            query_builder
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn upsert(&self, tid: TenantId, device: &IoTDevice) -> RepositoryFuture<()> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let device_id = device.device_id.clone();
        let site_id = device.site_id;
        let payload = serde_json::to_value(device.device.clone())
            .map_err(|e| SharedError::Internal(e.to_string()))
            .unwrap_or(serde_json::Value::Null);
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO iot_devices (device_id, tenant_id, site_id, device) VALUES ($1, $2, $3, $4)
                 ON CONFLICT (device_id) DO UPDATE SET tenant_id = EXCLUDED.tenant_id,
                 site_id = EXCLUDED.site_id, device = EXCLUDED.device, updated_at = NOW()",
            )
            .bind(&device_id)
            .bind(tid.0)
            .bind(site_id)
            .bind(payload)
            .execute(&pool)
            .await
            .map(|_| ())
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, device_id: &str) -> RepositoryFuture<bool> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let device_id_owned = device_id.to_owned();
        Box::pin(async move {
            sqlx::query("DELETE FROM iot_devices WHERE device_id = $1 AND tenant_id = $2")
                .bind(&device_id_owned)
                .bind(tid.0)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn get_telemetry(
        &self,
        tid: TenantId,
        device_id: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
        limit: i64,
    ) -> RepositoryFuture<Vec<IoTTelemetry>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let device_id_owned = device_id.to_owned();
        let _ = tid; // tenant already scoped via TenantPool
        Box::pin(async move {
            if let Some(since) = since {
                sqlx::query_as::<_, IoTTelemetry>(
                    r#"SELECT device_id, capability, value, unit, quality, received_at
                       FROM iot_telemetry
                       WHERE device_id = $1 AND received_at >= $2
                       ORDER BY received_at DESC
                       LIMIT $3"#,
                )
                .bind(&device_id_owned)
                .bind(since)
                .bind(limit as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            } else {
                sqlx::query_as::<_, IoTTelemetry>(
                    r#"SELECT device_id, capability, value, unit, quality, received_at
                       FROM iot_telemetry
                       WHERE device_id = $1
                       ORDER BY received_at DESC
                       LIMIT $2"#,
                )
                .bind(&device_id_owned)
                .bind(limit as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            }
        })
    }

    fn create_command(&self, command: &IoTCommand) -> RepositoryFuture<()> {
        let pool = self.pool.clone();
        let id = command.id;
        let device_id = command.device_id.clone();
        let command_type = command.command_type.clone();
        let payload = command.payload.clone();
        let status = command.status.clone();
        let requested_at = command.requested_at;
        let timeout_seconds = command.timeout_seconds;
        Box::pin(async move {
            sqlx::query(
                r#"INSERT INTO iot_commands 
                   (id, device_id, command_type, payload, status, requested_at, timeout_seconds)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            )
            .bind(id)
            .bind(&device_id)
            .bind(&command_type)
            .bind(&payload)
            .bind(&status)
            .bind(requested_at)
            .bind(timeout_seconds)
            .execute(&pool)
            .await
            .map(|_| ())
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
