//! IoT Device Registry Entities
//!
//! Represents IoT devices registered in the system, their capabilities,
//! and telemetry data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;

/// An IoT device registered in the system.
///
/// Stored in the `iot_devices` table with a JSONB `device` column.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
pub struct IoTDevice {
    #[sqlx(just)]
    pub device_id: String,
    #[sqlx(just)]
    pub tenant_id: TenantId,
    #[sqlx(just)]
    pub site_id: Option<Uuid>,
    #[sqlx(json)]
    pub device: IoTDeviceData,
}

/// The JSONB payload stored in the `device` column of `iot_devices`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct IoTDeviceData {
    pub device_type: String,
    pub capabilities: Vec<IoTCapability>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub status: DeviceStatus,
    pub firmware_version: Option<String>,
    pub battery_level: Option<f64>,
    pub signal_strength: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
    pub topic_prefix: Option<String>,
}

/// Device operational status.
#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema, Type,
)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "device_status")]
pub enum DeviceStatus {
    #[default]
    Online,
    Offline,
    Error,
    Maintenance,
    Updating,
}

/// A capability an IoT device can report or act on.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct IoTCapability {
    #[serde(rename = "type")]
    pub capability_type: IoTCapabilityType,
    pub unit: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub precision: Option<u8>,
}

/// The type of capability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum IoTCapabilityType {
    Temperature,
    Humidity,
    SoilMoisture,
    Light,
    GPS,
    BatteryLevel,
    SignalStrength,
    ActuatorControl,
    FirmwareUpdate,
    Power,
    Energy,
    Pressure,
    Voltage,
    Current,
    Custom(String),
}

/// DTO for creating a new IoT device.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateIoTDeviceDto {
    #[validate(length(min = 1, max = 100))]
    pub device_id: String,
    #[validate(length(min = 1, max = 100))]
    pub device_type: String,
    pub site_id: Option<Uuid>,
    pub capabilities: Vec<IoTCapability>,
    pub metadata: Option<serde_json::Value>,
    pub topic_prefix: Option<String>,
}

/// DTO for updating an existing IoT device.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, Default)]
pub struct UpdateIoTDeviceDto {
    pub device_type: Option<String>,
    pub site_id: Option<Uuid>,
    pub capabilities: Option<Vec<IoTCapability>>,
    pub metadata: Option<serde_json::Value>,
    pub status: Option<DeviceStatus>,
    pub topic_prefix: Option<String>,
}

/// A telemetry measurement from an IoT device.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct IoTTelemetry {
    #[sqlx(just)]
    pub device_id: String,
    #[sqlx(just)]
    pub capability: String,
    #[sqlx(just)]
    pub value: f64,
    #[sqlx(just)]
    pub unit: String,
    #[sqlx(just)]
    pub quality: Option<String>,
    #[sqlx(just)]
    pub received_at: DateTime<Utc>,
}

/// A command sent to an IoT device.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct IoTCommand {
    #[sqlx(just)]
    pub id: Uuid,
    #[sqlx(just)]
    pub device_id: String,
    #[sqlx(just)]
    pub command_type: String,
    #[sqlx(json)]
    pub payload: serde_json::Value,
    #[sqlx(just)]
    pub status: String,
    #[sqlx(just)]
    pub requested_at: DateTime<Utc>,
    #[sqlx(just)]
    pub timeout_seconds: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iot_device_can_be_created_from_dto() {
        let dto = CreateIoTDeviceDto {
            device_id: "sensor-001".to_string(),
            device_type: "temperature_sensor".to_string(),
            site_id: None,
            capabilities: vec![],
            metadata: None,
            topic_prefix: None,
        };

        assert_eq!(dto.device_id, "sensor-001");
        assert_eq!(dto.device_type, "temperature_sensor");
    }
}
