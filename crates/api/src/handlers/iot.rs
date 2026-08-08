//! IoT Device Registry Handlers

use crate::AppState;
use crate::dto::{
    CommandStatus, CreateIoTDeviceDto, DeviceStatusDto, HaDiscoveryConfigResponse, IoTCapabilityDto,
    IoTCommandRequestDto, IoTCommandResponseDto, IoTDeviceListResponse, IoTDeviceResponse,
    IoTDeviceTelemetryResponse, IoTMeasurementDto, UpdateIoTDeviceDto,
};
use crate::error::ApiError;
use actix_web::{HttpResponse, web};
use agrocore_messaging::{
    create_ha_device_info, generate_ha_discovery_configs, DeviceStatus, IoTCapability,
    IoTDeviceConfig,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/iot")
            .service(
                web::resource("/devices")
                    .route(web::get().to(list_devices))
                    .route(web::post().to(create_device)),
            )
            .service(
                web::resource("/devices/{device_id}")
                    .route(web::get().to(get_device))
                    .route(web::put().to(update_device))
                    .route(web::delete().to(delete_device)),
            )
            .service(
                web::resource("/devices/{device_id}/telemetry")
                    .route(web::get().to(get_device_telemetry)),
            )
            .service(
                web::resource("/devices/{device_id}/command")
                    .route(web::post().to(send_command)),
            )
            .service(
                web::resource("/devices/{device_id}/ha-discovery")
                    .route(web::get().to(get_ha_discovery)),
            ),
    );
}

// In-memory storage for IoT devices (in production, use database)
type DeviceStore = Arc<tokio::sync::RwLock<HashMap<String, IoTDeviceResponse>>>;

fn get_device_store(state: &web::Data<AppState>) -> DeviceStore {
    // In a real implementation, this would be part of AppState
    // For now, we'll create a new one per request (not production-ready)
    // TODO: Add to AppState properly
    state.app_data::<DeviceStore>().cloned().unwrap_or_else(|| {
        Arc::new(tokio::sync::RwLock::new(HashMap::new()))
    })
}

/// List all IoT devices for a tenant
#[utoipa::path(
    get,
    path = "/api/v1/iot/devices",
    params(
        ("tenant_id" = Uuid, Query, description = "Tenant ID"),
        ("site_id" = Option<Uuid>, Query, description = "Optional site filter"),
        ("status" = Option<DeviceStatusDto>, Query, description = "Optional status filter"),
    ),
    responses(
        (status = 200, description = "List of IoT devices", body = IoTDeviceListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    tag = "iot",
    security(("bearer_auth" = []))
)]
async fn list_devices(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager", "viewer"])?;
    
    let tenant_id = query
        .get("tenant_id")
        .and_then(|s| s.parse::<Uuid>().ok())
        .unwrap_or(auth.0.tenant_id);
    
    let site_id = query
        .get("site_id")
        .and_then(|s| s.parse::<Uuid>().ok());
    
    let status_filter = query
        .get("status")
        .and_then(|s| s.parse::<DeviceStatusDto>().ok());
    
    let store = get_device_store(&state);
    let devices = store.read().await;
    
    let filtered: Vec<IoTDeviceResponse> = devices
        .values()
        .filter(|d| d.tenant_id == tenant_id)
        .filter(|d| site_id.map_or(true, |s| d.site_id == Some(s)))
        .filter(|d| status_filter.as_ref().map_or(true, |f| d.status == *f))
        .cloned()
        .collect();
    
    let total = filtered.len();
    
    Ok(HttpResponse::Ok().json(IoTDeviceListResponse {
        devices: filtered,
        total,
    }))
}

/// Get a single IoT device by ID
#[utoipa::path(
    get,
    path = "/api/v1/iot/devices/{device_id}",
    params(
        ("device_id" = String, Path, description = "Device ID"),
    ),
    responses(
        (status = 200, description = "IoT device details", body = IoTDeviceResponse),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    tag = "iot",
    security(("bearer_auth" = []))
)]
async fn get_device(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager", "viewer"])?;
    
    let device_id = path.into_inner();
    let store = get_device_store(&state);
    let devices = store.read().await;
    
    if let Some(device) = devices.get(&device_id) {
        if device.tenant_id != auth.0.tenant_id {
            return Err(ApiError::Forbidden("Device belongs to different tenant".to_string()));
        }
        Ok(HttpResponse::Ok().json(device.clone()))
    } else {
        Err(ApiError::NotFound("Device not found".to_string()))
    }
}

/// Create a new IoT device
#[utoipa::path(
    post,
    path = "/api/v1/iot/devices",
    request_body = CreateIoTDeviceDto,
    responses(
        (status = 201, description = "IoT device created", body = IoTDeviceResponse),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Device ID already exists")
    ),
    tag = "iot",
    security(("bearer_auth" = []))
)]
async fn create_device(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    dto: web::Json<CreateIoTDeviceDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager"])?;
    
    let dto = dto.0;
    dto.validate().map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    
    // Check tenant access
    if dto.tenant_id != auth.0.tenant_id && !auth.0.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Cannot create device for different tenant".to_string()));
    }
    
    let store = get_device_store(&state);
    let mut devices = store.write().await;
    
    if devices.contains_key(&dto.device_id) {
        return Err(ApiError::Conflict("Device ID already exists".to_string()));
    }
    
    let now = chrono::Utc::now();
    let device = IoTDeviceResponse {
        device_id: dto.device_id.clone(),
        device_type: dto.device_type,
        tenant_id: dto.tenant_id,
        site_id: dto.site_id,
        capabilities: dto.capabilities,
        metadata: dto.metadata.unwrap_or_default(),
        status: DeviceStatusDto::Offline,
        firmware_version: None,
        battery_level: None,
        signal_strength: None,
        error_message: None,
        created_at: now,
        updated_at: now,
        last_seen: None,
    };
    
    devices.insert(dto.device_id.clone(), device.clone());
    
    info!("IoT device created: {} for tenant {}", dto.device_id, dto.tenant_id);
    
    Ok(HttpResponse::Created().json(device))
}

/// Update an IoT device
#[utoipa::path(
    put,
    path = "/api/v1/iot/devices/{device_id}",
    params(
        ("device_id" = String, Path, description = "Device ID"),
    ),
    request_body = UpdateIoTDeviceDto,
    responses(
        (status = 200, description = "IoT device updated", body = IoTDeviceResponse),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Device not found")
    ),
    tag = "iot",
    security(("bearer_auth" = []))
)]
async fn update_device(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
    dto: web::Json<UpdateIoTDeviceDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager"])?;
    
    let device_id = path.into_inner();
    let dto = dto.0;
    dto.validate().map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    
    let store = get_device_store(&state);
    let mut devices = store.write().await;
    
    if let Some(device) = devices.get_mut(&device_id) {
        if device.tenant_id != auth.0.tenant_id && !auth.0.roles.contains(&"admin".to_string()) {
            return Err(ApiError::Forbidden("Cannot update device from different tenant".to_string()));
        }
        
        if let Some(device_type) = dto.device_type {
            if !device_type.is_empty() {
                device.device_type = device_type;
            }
        }
        if let Some(site_id) = dto.site_id {
            device.site_id = Some(site_id);
        }
        if let Some(capabilities) = dto.capabilities {
            device.capabilities = capabilities;
        }
        if let Some(metadata) = dto.metadata {
            device.metadata = metadata;
        }
        if let Some(status) = dto.status {
            device.status = status;
        }
        
        device.updated_at = chrono::Utc::now();
        
        info!("IoT device updated: {}", device_id);
        
        Ok(HttpResponse::Ok().json(device.clone()))
    } else {
        Err(ApiError::NotFound("Device not found".to_string()))
    }
}

/// Delete an IoT device
#[utoipa::path(
    delete,
    path = "/api/v1/iot/devices/{device_id}",
    params(
        ("device_id" = String, Path, description = "Device ID"),
    ),
    responses(
        (status = 204, description = "IoT device deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Device not found")
    ),
    tag = "iot",
    security(("bearer_auth" = []))
)]
async fn delete_device(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;
    
    let device_id = path.into_inner();
    let store = get_device_store(&state);
    let mut devices = store.write().await;
    
    if let Some(device) = devices.get(&device_id) {
        if device.tenant_id != auth.0.tenant_id && !auth.0.roles.contains(&"admin".to_string()) {
            return Err(ApiError::Forbidden("Cannot delete device from different tenant".to_string()));
        }
    }
    
    if devices.remove(&device_id).is_some() {
        info!("IoT device deleted: {}", device_id);
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::NotFound("Device not found".to_string()))
    }
}

/// Get device telemetry
#[utoipa::path(
    get,
    path = "/api/v1/iot/devices/{device_id}/telemetry",
    params(
        ("device_id" = String, Path, description = "Device ID"),
        ("limit" = Option<i32>, Query, description = "Limit results"),
        ("since" = Option<chrono::DateTime<chrono::Utc>>, Query, description = "Filter by timestamp"),
    ),
    responses(
        (status = 200, description = "Device telemetry", body = IoTDeviceTelemetryResponse),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    tag = "iot",
    security(("bearer_auth" = []))
)]
async fn get_device_telemetry(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager", "viewer"])?;
    
    let device_id = path.into_inner();
    let store = get_device_store(&state);
    let devices = store.read().await;
    
    if let Some(device) = devices.get(&device_id) {
        if device.tenant_id != auth.0.tenant_id && !auth.0.roles.contains(&"admin".to_string()) {
            return Err(ApiError::Forbidden("Device belongs to different tenant".to_string()));
        }
        
        // In a real implementation, this would query a time-series database
        // For now, return mock data
        let response = IoTDeviceTelemetryResponse {
            device_id: device_id.clone(),
            measurements: vec![],
            timestamp: chrono::Utc::now(),
        };
        
        Ok(HttpResponse::Ok().json(response))
    } else {
        Err(ApiError::NotFound("Device not found".to_string()))
    }
}

/// Send command to device
#[utoipa::path(
    post,
    path = "/api/v1/iot/devices/{device_id}/command",
    params(
        ("device_id" = String, Path, description = "Device ID"),
    ),
    request_body = IoTCommandRequestDto,
    responses(
        (status = 200, description = "Command sent", body = IoTCommandResponseDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Device not found")
    ),
    tag = "iot",
    security(("bearer_auth" = []))
)]
async fn send_command(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
    dto: web::Json<IoTCommandRequestDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager"])?;
    
    let device_id = path.into_inner();
    let dto = dto.0;
    
    let store = get_device_store(&state);
    let devices = store.read().await;
    
    if let Some(device) = devices.get(&device_id) {
        if device.tenant_id != auth.0.tenant_id && !auth.0.roles.contains(&"admin".to_string()) {
            return Err(ApiError::Forbidden("Device belongs to different tenant".to_string()));
        }
        
        if device.status != DeviceStatusDto::Online {
            return Err(ApiError::BadRequest("Device is not online".to_string()));
        }
        
        let command_id = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        // In a real implementation, this would publish to MQTT
        // For now, return a mock response
        let response = IoTCommandResponseDto {
            command_id,
            device_id: device_id.clone(),
            command_type: dto.command_type,
            status: CommandStatus::Sent,
            response_payload: None,
            requested_at: now,
            completed_at: None,
        };
        
        info!("Command sent to device {}: {}", device_id, dto.command_type);
        
        Ok(HttpResponse::Ok().json(response))
    } else {
        Err(ApiError::NotFound("Device not found".to_string()))
    }
}

/// Get Home Assistant discovery configuration for a device
#[utoipa::path(
    get,
    path = "/api/v1/iot/devices/{device_id}/ha-discovery",
    params(
        ("device_id" = String, Path, description = "Device ID"),
    ),
    responses(
        (status = 200, description = "Home Assistant discovery configs", body = HaDiscoveryConfigResponse),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    tag = "iot",
    security(("bearer_auth" = []))
)]
async fn get_ha_discovery(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager"])?;
    
    let device_id = path.into_inner();
    let store = get_device_store(&state);
    let devices = store.read().await;
    
    if let Some(device) = devices.get(&device_id) {
        if device.tenant_id != auth.0.tenant_id && !auth.0.roles.contains(&"admin".to_string()) {
            return Err(ApiError::Forbidden("Device belongs to different tenant".to_string()));
        }
        
        // Convert to IoTDeviceConfig for HA discovery generation
        let iot_config = IoTDeviceConfig {
            device_id: device.device_id.clone(),
            device_type: device.device_type.clone(),
            tenant_id: device.tenant_id,
            site_id: device.site_id,
            capabilities: device.capabilities.iter().map(|c| {
                match c.capability_type {
                    IoTCapabilityType::Temperature => agrocore_messaging::IoTCapability::Temperature,
                    IoTCapabilityType::Humidity => agrocore_messaging::IoTCapability::Humidity,
                    IoTCapabilityType::SoilMoisture => agrocore_messaging::IoTCapability::SoilMoisture,
                    IoTCapabilityType::Light => agrocore_messaging::IoTCapability::Light,
                    IoTCapabilityType::Gps => agrocore_messaging::IoTCapability::GPS,
                    IoTCapabilityType::BatteryLevel => agrocore_messaging::IoTCapability::BatteryLevel,
                    IoTCapabilityType::SignalStrength => agrocore_messaging::IoTCapability::SignalStrength,
                    IoTCapabilityType::ActuatorControl => agrocore_messaging::IoTCapability::ActuatorControl,
                    IoTCapabilityType::FirmwareUpdate => agrocore_messaging::IoTCapability::FirmwareUpdate,
                    IoTCapabilityType::Custom(s) => agrocore_messaging::IoTCapability::Custom(s),
                }
            }).collect(),
            metadata: device.metadata.clone(),
        };
        
        let configs = generate_ha_discovery_configs(&iot_config, "agrocore");
        
        Ok(HttpResponse::Ok().json(HaDiscoveryConfigResponse { configs }))
    } else {
        Err(ApiError::NotFound("Device not found".to_string()))
    }
}