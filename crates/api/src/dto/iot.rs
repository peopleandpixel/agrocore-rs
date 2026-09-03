//! IoT Device Registry DTOs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateIoTDeviceDto {
    #[validate(length(min = 1, max = 100))]
    pub device_id: String,
    #[validate(length(min = 1, max = 100))]
    pub device_type: String,
    pub tenant_id: Uuid,
    pub site_id: Option<Uuid>,
    pub capabilities: Vec<IoTCapabilityDto>,
    pub metadata: Option<serde_json::Value>,
    pub topic_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateIoTDeviceDto {
    #[validate(length(min = 1, max = 100))]
    pub device_type: Option<String>,
    pub site_id: Option<Uuid>,
    pub capabilities: Option<Vec<IoTCapabilityDto>>,
    pub metadata: Option<serde_json::Value>,
    pub status: Option<DeviceStatusDto>,
    pub topic_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IoTDeviceResponse {
    pub device_id: String,
    pub device_type: String,
    pub tenant_id: Uuid,
    pub site_id: Option<Uuid>,
    pub capabilities: Vec<IoTCapabilityDto>,
    pub metadata: serde_json::Value,
    pub status: DeviceStatusDto,
    pub firmware_version: Option<String>,
    pub battery_level: Option<f64>,
    pub signal_strength: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub topic_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Validate)]
pub struct IoTCapabilityDto {
    #[serde(rename = "type")]
    pub capability_type: IoTCapabilityType,
    pub unit: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub precision: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
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

impl From<agrocore_messaging::IoTCapability> for IoTCapabilityDto {
    fn from(cap: agrocore_messaging::IoTCapability) -> Self {
        match cap {
            agrocore_messaging::IoTCapability::Temperature => Self {
                capability_type: IoTCapabilityType::Temperature,
                unit: Some("°C".to_string()),
                min_value: Some(-40.0),
                max_value: Some(85.0),
                precision: Some(1),
            },
            agrocore_messaging::IoTCapability::Humidity => Self {
                capability_type: IoTCapabilityType::Humidity,
                unit: Some("%".to_string()),
                min_value: Some(0.0),
                max_value: Some(100.0),
                precision: Some(1),
            },
            agrocore_messaging::IoTCapability::SoilMoisture => Self {
                capability_type: IoTCapabilityType::SoilMoisture,
                unit: Some("%".to_string()),
                min_value: Some(0.0),
                max_value: Some(100.0),
                precision: Some(1),
            },
            agrocore_messaging::IoTCapability::Light => Self {
                capability_type: IoTCapabilityType::Light,
                unit: Some("lux".to_string()),
                min_value: Some(0.0),
                max_value: Some(200000.0),
                precision: Some(0),
            },
            agrocore_messaging::IoTCapability::GPS => Self {
                capability_type: IoTCapabilityType::GPS,
                unit: Some("degrees".to_string()),
                min_value: Some(-180.0),
                max_value: Some(180.0),
                precision: Some(6),
            },
            agrocore_messaging::IoTCapability::BatteryLevel => Self {
                capability_type: IoTCapabilityType::BatteryLevel,
                unit: Some("%".to_string()),
                min_value: Some(0.0),
                max_value: Some(100.0),
                precision: Some(1),
            },
            agrocore_messaging::IoTCapability::SignalStrength => Self {
                capability_type: IoTCapabilityType::SignalStrength,
                unit: Some("dBm".to_string()),
                min_value: Some(-120.0),
                max_value: Some(0.0),
                precision: Some(0),
            },
            agrocore_messaging::IoTCapability::ActuatorControl => Self {
                capability_type: IoTCapabilityType::ActuatorControl,
                unit: None,
                min_value: None,
                max_value: None,
                precision: None,
            },
            agrocore_messaging::IoTCapability::FirmwareUpdate => Self {
                capability_type: IoTCapabilityType::FirmwareUpdate,
                unit: None,
                min_value: None,
                max_value: None,
                precision: None,
            },
            agrocore_messaging::IoTCapability::Power => Self {
                capability_type: IoTCapabilityType::Power,
                unit: Some("W".to_string()),
                min_value: Some(0.0),
                max_value: Some(10000.0),
                precision: Some(1),
            },
            agrocore_messaging::IoTCapability::Energy => Self {
                capability_type: IoTCapabilityType::Energy,
                unit: Some("kWh".to_string()),
                min_value: Some(0.0),
                max_value: Some(1000000.0),
                precision: Some(2),
            },
            agrocore_messaging::IoTCapability::Pressure => Self {
                capability_type: IoTCapabilityType::Pressure,
                unit: Some("hPa".to_string()),
                min_value: Some(300.0),
                max_value: Some(1200.0),
                precision: Some(1),
            },
            agrocore_messaging::IoTCapability::Voltage => Self {
                capability_type: IoTCapabilityType::Voltage,
                unit: Some("V".to_string()),
                min_value: Some(0.0),
                max_value: Some(500.0),
                precision: Some(2),
            },
            agrocore_messaging::IoTCapability::Current => Self {
                capability_type: IoTCapabilityType::Current,
                unit: Some("A".to_string()),
                min_value: Some(0.0),
                max_value: Some(100.0),
                precision: Some(2),
            },
            agrocore_messaging::IoTCapability::Custom(s) => Self {
                capability_type: IoTCapabilityType::Custom(s),
                unit: None,
                min_value: None,
                max_value: None,
                precision: None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum DeviceStatusDto {
    Online,
    Offline,
    Error,
    Maintenance,
    Updating,
}

impl From<agrocore_messaging::DeviceStatus> for DeviceStatusDto {
    fn from(status: agrocore_messaging::DeviceStatus) -> Self {
        match status {
            agrocore_messaging::DeviceStatus::Online => Self::Online,
            agrocore_messaging::DeviceStatus::Offline => Self::Offline,
            agrocore_messaging::DeviceStatus::Error => Self::Error,
            agrocore_messaging::DeviceStatus::Maintenance => Self::Maintenance,
            agrocore_messaging::DeviceStatus::Updating => Self::Updating,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IoTDeviceListResponse {
    pub devices: Vec<IoTDeviceResponse>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IoTDeviceTelemetryResponse {
    pub device_id: String,
    pub measurements: Vec<IoTMeasurementDto>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IoTMeasurementDto {
    pub capability: IoTCapabilityDto,
    pub value: f64,
    pub unit: String,
    pub quality: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct IoTCommandRequestDto {
    #[validate(length(min = 1, max = 100))]
    pub command_type: String,
    pub payload: serde_json::Value,
    #[validate(range(min = 1, max = 300))]
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IoTCommandResponseDto {
    pub command_id: Uuid,
    pub device_id: String,
    pub command_type: String,
    pub status: CommandStatus,
    pub response_payload: Option<serde_json::Value>,
    pub requested_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum CommandStatus {
    Pending,
    Sent,
    Delivered,
    Completed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HaDiscoveryConfigResponse {
    pub configs: Vec<(String, serde_json::Value)>,
}
