//! IoT Device Registry Handlers

use crate::AppState;
use crate::dto::{
    CommandStatus, CreateIoTDeviceDto, DeviceStatusDto, HaDiscoveryConfigResponse,
    IoTCapabilityDto, IoTCapabilityType, IoTCommandRequestDto, IoTCommandResponseDto,
    IoTDeviceListResponse, IoTDeviceResponse, IoTDeviceTelemetryResponse, IoTMeasurementDto,
    UpdateIoTDeviceDto,
};
use crate::error::ApiError;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::iot::{
    DeviceStatus, IoTCapability, IoTCapabilityType as DomainCapabilityType, IoTCommand, IoTDevice,
    IoTDeviceData, IoTTelemetry,
};
use agrocore_logging::info;
use agrocore_messaging::{
    IoTCommandEvent, IoTDeviceConfig, IoTMeasurement, IoTTelemetryEvent,
    NATS_SUBJECT_COMMANDS_BRIDGE, NATS_SUBJECT_TELEMETRY, generate_ha_discovery_configs,
};
use agrocore_shared::SharedError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

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
                web::resource("/devices/{device_id}/command").route(web::post().to(send_command)),
            )
            .service(
                web::resource("/devices/{device_id}/ha-discovery")
                    .route(web::get().to(get_ha_discovery)),
            )
            .service(
                // J8: Telemetry ingestion endpoint — devices report measurements here
                web::resource("/telemetry").route(web::post().to(ingest_telemetry)),
            ),
    );
}

fn to_domain_device(dto: &IoTDeviceResponse) -> IoTDevice {
    IoTDevice {
        device_id: dto.device_id.clone(),
        tenant_id: agrocore_domain::entities::tenant::TenantId(dto.tenant_id),
        site_id: dto.site_id,
        device: IoTDeviceData {
            device_type: dto.device_type.clone(),
            capabilities: dto
                .capabilities
                .iter()
                .map(|c| IoTCapability {
                    capability_type: match &c.capability_type {
                        IoTCapabilityType::Temperature => DomainCapabilityType::Temperature,
                        IoTCapabilityType::Humidity => DomainCapabilityType::Humidity,
                        IoTCapabilityType::SoilMoisture => DomainCapabilityType::SoilMoisture,
                        IoTCapabilityType::Light => DomainCapabilityType::Light,
                        IoTCapabilityType::GPS => DomainCapabilityType::GPS,
                        IoTCapabilityType::BatteryLevel => DomainCapabilityType::BatteryLevel,
                        IoTCapabilityType::SignalStrength => DomainCapabilityType::SignalStrength,
                        IoTCapabilityType::ActuatorControl => DomainCapabilityType::ActuatorControl,
                        IoTCapabilityType::FirmwareUpdate => DomainCapabilityType::FirmwareUpdate,
                        IoTCapabilityType::Power => DomainCapabilityType::Power,
                        IoTCapabilityType::Energy => DomainCapabilityType::Energy,
                        IoTCapabilityType::Pressure => DomainCapabilityType::Pressure,
                        IoTCapabilityType::Voltage => DomainCapabilityType::Voltage,
                        IoTCapabilityType::Current => DomainCapabilityType::Current,
                        IoTCapabilityType::Custom(s) => DomainCapabilityType::Custom(s.clone()),
                    },
                    unit: c.unit.clone(),
                    min_value: c.min_value,
                    max_value: c.max_value,
                    precision: c.precision,
                })
                .collect(),
            metadata: dto.metadata.clone(),
            status: match dto.status {
                DeviceStatusDto::Online => DeviceStatus::Online,
                DeviceStatusDto::Offline => DeviceStatus::Offline,
                DeviceStatusDto::Error => DeviceStatus::Error,
                DeviceStatusDto::Maintenance => DeviceStatus::Maintenance,
                DeviceStatusDto::Updating => DeviceStatus::Updating,
            },
            firmware_version: dto.firmware_version.clone(),
            battery_level: dto.battery_level,
            signal_strength: dto.signal_strength,
            error_message: dto.error_message.clone(),
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            last_seen: dto.last_seen,
            topic_prefix: dto.topic_prefix.clone(),
        },
    }
}

fn to_response(device: &IoTDevice) -> IoTDeviceResponse {
    IoTDeviceResponse {
        device_id: device.device_id.clone(),
        device_type: device.device.device_type.clone(),
        tenant_id: device.tenant_id.0,
        site_id: device.site_id,
        capabilities: device
            .device
            .capabilities
            .iter()
            .map(|c| IoTCapabilityDto {
                capability_type: match &c.capability_type {
                    DomainCapabilityType::Temperature => IoTCapabilityType::Temperature,
                    DomainCapabilityType::Humidity => IoTCapabilityType::Humidity,
                    DomainCapabilityType::SoilMoisture => IoTCapabilityType::SoilMoisture,
                    DomainCapabilityType::Light => IoTCapabilityType::Light,
                    DomainCapabilityType::GPS => IoTCapabilityType::GPS,
                    DomainCapabilityType::BatteryLevel => IoTCapabilityType::BatteryLevel,
                    DomainCapabilityType::SignalStrength => IoTCapabilityType::SignalStrength,
                    DomainCapabilityType::ActuatorControl => IoTCapabilityType::ActuatorControl,
                    DomainCapabilityType::FirmwareUpdate => IoTCapabilityType::FirmwareUpdate,
                    DomainCapabilityType::Power => IoTCapabilityType::Power,
                    DomainCapabilityType::Energy => IoTCapabilityType::Energy,
                    DomainCapabilityType::Pressure => IoTCapabilityType::Pressure,
                    DomainCapabilityType::Voltage => IoTCapabilityType::Voltage,
                    DomainCapabilityType::Current => IoTCapabilityType::Current,
                    DomainCapabilityType::Custom(s) => IoTCapabilityType::Custom(s.clone()),
                },
                unit: c.unit.clone(),
                min_value: c.min_value,
                max_value: c.max_value,
                precision: c.precision,
            })
            .collect(),
        metadata: device.device.metadata.clone(),
        status: match device.device.status {
            DeviceStatus::Online => DeviceStatusDto::Online,
            DeviceStatus::Offline => DeviceStatusDto::Offline,
            DeviceStatus::Error => DeviceStatusDto::Error,
            DeviceStatus::Maintenance => DeviceStatusDto::Maintenance,
            DeviceStatus::Updating => DeviceStatusDto::Updating,
        },
        firmware_version: device.device.firmware_version.clone(),
        battery_level: device.device.battery_level,
        signal_strength: device.device.signal_strength,
        error_message: device.device.error_message.clone(),
        created_at: device.device.created_at,
        updated_at: device.device.updated_at,
        last_seen: device.device.last_seen,
        topic_prefix: device.device.topic_prefix.clone(),
    }
}

async fn find_device(
    state: &web::Data<AppState>,
    tenant_id: uuid::Uuid,
    device_id: &str,
) -> Result<Option<IoTDeviceResponse>, ApiError> {
    let repo = state.db.iot_device_repo();
    let tid = agrocore_domain::entities::tenant::TenantId(tenant_id);
    let device = repo.find_by_device_id(tid, device_id).await?;
    Ok(device.map(|d| to_response(&d)))
}

async fn persist_device(
    state: &web::Data<AppState>,
    device: &IoTDeviceResponse,
) -> Result<(), ApiError> {
    let repo = state.db.iot_device_repo();
    let tid = agrocore_domain::entities::tenant::TenantId(device.tenant_id);
    let domain_device = to_domain_device(device);
    repo.upsert(tid, &domain_device).await?;
    Ok(())
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
pub async fn list_devices(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager", "viewer"])?;

    // The `tenant_id` query parameter is ignored. A tenant admin is not a
    // superadmin: `is_admin()` is a role within a tenant and must not widen the
    // tenant boundary, so honouring the parameter here let any tenant admin read
    // another tenant's device registry. Cross-tenant visibility needs a
    // separate, globally granted flag.
    if query.contains_key("tenant_id") {
        return Err(ApiError::forbidden(
            "Cross-tenant device listing is not permitted".to_string(),
        ));
    }
    let tenant_id = auth.0.tenant_id;

    let site_id = query.get("site_id").and_then(|s| s.parse::<Uuid>().ok());

    let status_filter = query
        .get("status")
        .and_then(|s| match s.to_ascii_lowercase().as_str() {
            "online" => Some(DeviceStatusDto::Online),
            "offline" => Some(DeviceStatusDto::Offline),
            "error" => Some(DeviceStatusDto::Error),
            "maintenance" => Some(DeviceStatusDto::Maintenance),
            "updating" => Some(DeviceStatusDto::Updating),
            _ => None,
        });

    let repo = state.db.iot_device_repo();
    let tid = agrocore_domain::entities::tenant::TenantId(tenant_id);

    // Map status_filter to domain DeviceStatus
    let domain_status = status_filter.as_ref().map(|s| match s {
        DeviceStatusDto::Online => DeviceStatus::Online,
        DeviceStatusDto::Offline => DeviceStatus::Offline,
        DeviceStatusDto::Error => DeviceStatus::Error,
        DeviceStatusDto::Maintenance => DeviceStatus::Maintenance,
        DeviceStatusDto::Updating => DeviceStatus::Updating,
    });

    let devices: Vec<IoTDeviceResponse> = repo
        .find_all(tid, site_id, domain_status)
        .await?
        .iter()
        .map(to_response)
        .collect();

    let filtered: Vec<IoTDeviceResponse> = devices
        .iter()
        .filter(|d| d.tenant_id == tenant_id)
        .filter(|d| site_id.is_none_or(|s| d.site_id == Some(s)))
        .filter(|d| status_filter.as_ref().is_none_or(|f| d.status == *f))
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
pub async fn get_device(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager", "viewer"])?;

    let device_id = path.into_inner();
    if let Some(device) = find_device(&state, auth.0.tenant_id, &device_id).await? {
        if device.tenant_id != auth.0.tenant_id {
            return Err(ApiError::forbidden(
                "Device belongs to different tenant".to_string(),
            ));
        }
        Ok(HttpResponse::Ok().json(device.clone()))
    } else {
        Err(ApiError::not_found("Device not found"))
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
pub async fn create_device(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    dto: web::Json<CreateIoTDeviceDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager"])?;

    let dto = dto.0;
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    // A device must be created in the caller's own tenant. `!auth.is_admin()` was
    // checked right after `require_any_role(vec!["admin", "manager"])`, so the
    // condition was constant-false and a tenant admin could register a device
    // against another tenant's id. The row would then be invisible to that
    // tenant and unreachable through the tenant-scoped queries.
    if dto.tenant_id != auth.0.tenant_id {
        return Err(ApiError::forbidden(
            "Cannot create device for different tenant".to_string(),
        ));
    }

    if find_device(&state, auth.0.tenant_id, &dto.device_id)
        .await?
        .is_some()
    {
        return Err(ApiError::conflict("Device ID already exists".to_string()));
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
        topic_prefix: dto.topic_prefix,
    };

    persist_device(&state, &device).await?;

    info!(
        "IoT device created: {} for tenant {}",
        dto.device_id, dto.tenant_id
    );

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
pub async fn update_device(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
    dto: web::Json<UpdateIoTDeviceDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager"])?;

    let device_id = path.into_inner();
    let dto = dto.0;
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    // `find_device` is already tenant-scoped, so a device belonging to another
    // tenant is simply not found. The explicit comparison that used to sit here
    // was unreachable for the same reason as in `delete_device`: `is_admin()` was
    // checked after `require_any_role`, making the condition constant-false.
    if let Some(mut device) = find_device(&state, auth.0.tenant_id, &device_id).await? {
        if let Some(device_type) = dto.device_type
            && !device_type.is_empty()
        {
            device.device_type = device_type;
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
        persist_device(&state, &device).await?;

        info!("IoT device updated: {}", device_id);

        Ok(HttpResponse::Ok().json(device.clone()))
    } else {
        Err(ApiError::not_found("Device not found"))
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
pub async fn delete_device(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;

    let device_id = path.into_inner();

    // The tenant filter belongs in the DELETE, not in a check before it. The
    // previous version read `if device.tenant_id != auth.0.tenant_id &&
    // !auth.is_admin()` after `require_admin()`, so the second clause was always
    // false and the whole condition could never be true — the DELETE that
    // followed had no tenant condition of its own and removed any device on the
    // server, including other tenants'.
    //
    // A tenant admin is not a superadmin. Cross-tenant access needs a separate,
    // globally granted flag; `is_admin()` is a role *within* a tenant and must
    // never widen the tenant boundary.
    let repo = state.db.iot_device_repo();
    let tid = agrocore_domain::entities::tenant::TenantId(auth.0.tenant_id);
    let deleted = repo.delete(tid, &device_id).await?;
    if deleted {
        info!("IoT device deleted: {}", device_id);
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::not_found("Device not found"))
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
pub async fn get_device_telemetry(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager", "viewer"])?;

    let device_id = path.into_inner();

    // Existence check
    find_device(&state, auth.0.tenant_id, &device_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Device not found"))?;

    // Parse query parameters
    let limit: i64 = query
        .get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50)
        .min(500);

    let since = query.get("since").and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    });

    // J8: Query real telemetry from iot_telemetry table via repository
    let repo = state.db.iot_device_repo();
    let tid = agrocore_domain::entities::tenant::TenantId(auth.0.tenant_id);
    let rows: Vec<IoTTelemetry> = repo
        .get_telemetry(tid, &device_id, since, limit)
        .await
        .map_err(|e| ApiError(SharedError::Database(e.to_string())))?;

    let measurements: Vec<IoTMeasurementDto> = rows
        .into_iter()
        .map(|t| IoTMeasurementDto {
            capability: IoTCapabilityDto {
                capability_type: IoTCapabilityType::Custom(t.capability),
                unit: None,
                min_value: None,
                max_value: None,
                precision: None,
            },
            value: t.value,
            unit: t.unit,
            quality: t.quality,
            timestamp: t.received_at,
        })
        .collect();

    let response = IoTDeviceTelemetryResponse {
        device_id,
        measurements,
        timestamp: chrono::Utc::now(),
    };

    Ok(HttpResponse::Ok().json(response))
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
pub async fn send_command(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
    dto: web::Json<IoTCommandRequestDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager"])?;

    let device_id = path.into_inner();
    let dto = dto.0;
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let device = find_device(&state, auth.0.tenant_id, &device_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Device not found"))?;

    if device.status != DeviceStatusDto::Online {
        return Err(ApiError::bad_request("Device is not online"));
    }

    let command_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // J8: Persist the command in iot_commands table via repository
    let command = IoTCommand {
        id: command_id,
        device_id: device_id.clone(),
        command_type: dto.command_type.clone(),
        payload: dto.payload.clone(),
        status: "Sent".to_string(),
        requested_at: now,
        timeout_seconds: dto.timeout_seconds.map(|s| s as i32),
    };
    let repo = state.db.iot_device_repo();
    repo.create_command(&command)
        .await
        .map_err(|e| ApiError(SharedError::Database(e.to_string())))?;

    // J8: Publish command to NATS so the MQTT bridge can forward it
    let command_event = IoTCommandEvent {
        device_id: device_id.clone(),
        command_id,
        command_type: dto.command_type.clone(),
        payload: dto.payload,
        requested_at: now,
        timeout_seconds: dto.timeout_seconds,
    };

    if let Some(ref messaging) = state.messaging {
        let _ = messaging
            .publish(NATS_SUBJECT_COMMANDS_BRIDGE.to_string(), &command_event)
            .await;
    }

    info!(
        "Command {} sent to device {}: {}",
        command_id, device_id, command_event.command_type
    );

    let response = IoTCommandResponseDto {
        command_id,
        device_id,
        command_type: command_event.command_type,
        status: CommandStatus::Sent,
        response_payload: None,
        requested_at: now,
        completed_at: None,
    };

    Ok(HttpResponse::Ok().json(response))
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
pub async fn get_ha_discovery(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager"])?;

    let device_id = path.into_inner();
    if let Some(device) = find_device(&state, auth.0.tenant_id, &device_id).await? {
        // Convert to IoTDeviceConfig for HA discovery generation
        let iot_config = IoTDeviceConfig {
            device_id: device.device_id.clone(),
            device_type: device.device_type.clone(),
            tenant_id: device.tenant_id,
            site_id: device.site_id,
            capabilities: device
                .capabilities
                .iter()
                .map(|c| match c.capability_type.clone() {
                    IoTCapabilityType::Temperature => {
                        agrocore_messaging::IoTCapability::Temperature
                    }
                    IoTCapabilityType::Humidity => agrocore_messaging::IoTCapability::Humidity,
                    IoTCapabilityType::SoilMoisture => {
                        agrocore_messaging::IoTCapability::SoilMoisture
                    }
                    IoTCapabilityType::Light => agrocore_messaging::IoTCapability::Light,
                    IoTCapabilityType::GPS => agrocore_messaging::IoTCapability::GPS,
                    IoTCapabilityType::BatteryLevel => {
                        agrocore_messaging::IoTCapability::BatteryLevel
                    }
                    IoTCapabilityType::SignalStrength => {
                        agrocore_messaging::IoTCapability::SignalStrength
                    }
                    IoTCapabilityType::ActuatorControl => {
                        agrocore_messaging::IoTCapability::ActuatorControl
                    }
                    IoTCapabilityType::FirmwareUpdate => {
                        agrocore_messaging::IoTCapability::FirmwareUpdate
                    }
                    IoTCapabilityType::Power => agrocore_messaging::IoTCapability::Power,
                    IoTCapabilityType::Energy => agrocore_messaging::IoTCapability::Energy,
                    IoTCapabilityType::Pressure => agrocore_messaging::IoTCapability::Pressure,
                    IoTCapabilityType::Voltage => agrocore_messaging::IoTCapability::Voltage,
                    IoTCapabilityType::Current => agrocore_messaging::IoTCapability::Current,
                    IoTCapabilityType::Custom(s) => agrocore_messaging::IoTCapability::Custom(s),
                })
                .collect(),
            metadata: device.metadata.clone(),
            topic_prefix: device
                .topic_prefix
                .clone()
                .unwrap_or_else(|| "agrocore".to_string()),
        };

        // Generate measurements from capabilities for HA discovery
        let measurements: Vec<IoTMeasurement> = iot_config
            .capabilities
            .iter()
            .map(|cap| {
                let (unit, capability) = match cap {
                    agrocore_messaging::IoTCapability::Temperature => {
                        ("°C", agrocore_messaging::IoTCapability::Temperature)
                    }
                    agrocore_messaging::IoTCapability::Humidity => {
                        ("%", agrocore_messaging::IoTCapability::Humidity)
                    }
                    agrocore_messaging::IoTCapability::SoilMoisture => {
                        ("%", agrocore_messaging::IoTCapability::SoilMoisture)
                    }
                    agrocore_messaging::IoTCapability::Light => {
                        ("lux", agrocore_messaging::IoTCapability::Light)
                    }
                    agrocore_messaging::IoTCapability::GPS => {
                        ("°", agrocore_messaging::IoTCapability::GPS)
                    }
                    agrocore_messaging::IoTCapability::BatteryLevel => {
                        ("%", agrocore_messaging::IoTCapability::BatteryLevel)
                    }
                    agrocore_messaging::IoTCapability::SignalStrength => {
                        ("dBm", agrocore_messaging::IoTCapability::SignalStrength)
                    }
                    agrocore_messaging::IoTCapability::ActuatorControl => {
                        ("%", agrocore_messaging::IoTCapability::ActuatorControl)
                    }
                    agrocore_messaging::IoTCapability::FirmwareUpdate => {
                        ("%", agrocore_messaging::IoTCapability::FirmwareUpdate)
                    }
                    agrocore_messaging::IoTCapability::Power => {
                        ("W", agrocore_messaging::IoTCapability::Power)
                    }
                    agrocore_messaging::IoTCapability::Energy => {
                        ("kWh", agrocore_messaging::IoTCapability::Energy)
                    }
                    agrocore_messaging::IoTCapability::Pressure => {
                        ("hPa", agrocore_messaging::IoTCapability::Pressure)
                    }
                    agrocore_messaging::IoTCapability::Voltage => {
                        ("V", agrocore_messaging::IoTCapability::Voltage)
                    }
                    agrocore_messaging::IoTCapability::Current => {
                        ("A", agrocore_messaging::IoTCapability::Current)
                    }
                    agrocore_messaging::IoTCapability::Custom(s) => (
                        s.as_str(),
                        agrocore_messaging::IoTCapability::Custom(s.clone()),
                    ),
                };
                IoTMeasurement {
                    capability,
                    value: 0.0,
                    unit: unit.to_string(),
                    quality: None,
                }
            })
            .collect();

        let configs = generate_ha_discovery_configs(&iot_config, &measurements);

        Ok(HttpResponse::Ok().json(HaDiscoveryConfigResponse { configs }))
    } else {
        Err(ApiError::not_found("Device not found"))
    }
}

/// J8: Ingest telemetry measurements from IoT devices
///
/// This endpoint accepts telemetry data from devices and stores it in the
/// iot_telemetry table. It also publishes to NATS so the MQTT bridge can
/// forward it to subscribed clients.
#[utoipa::path(
    post,
    path = "/api/v1/iot/telemetry",
    request_body = IoTTelemetryIngestDto,
    responses(
        (status = 201, description = "Telemetry ingested"),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    tag = "iot",
    security(("bearer_auth" = []))
)]
pub async fn ingest_telemetry(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    dto: web::Json<IoTTelemetryIngestDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager"])?;

    let dto = dto.0;
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let tenant_id = auth.0.tenant_id;

    // Verify device exists
    find_device(&state, tenant_id, &dto.device_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Device not found"))?;

    // J8: Store each measurement in iot_telemetry table
    // Telemetry is stored directly via sqlx since the repository layer
    // focuses on device CRUD; telemetry ingestion is a high-throughput
    // write path that benefits from batch inserts.
    let now = chrono::Utc::now();
    let metadata_ref = dto.metadata.as_ref();
    for m in &dto.measurements {
        let pool = state.db.tenant_pool(tenant_id);
        sqlx::query(
            r#"INSERT INTO iot_telemetry 
               (tenant_id, device_id, capability, value, unit, quality, metadata, received_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(tenant_id)
        .bind(&dto.device_id)
        .bind(&m.capability)
        .bind(m.value)
        .bind(&m.unit)
        .bind(m.quality.as_deref())
        .bind(metadata_ref.map(|v| v as &serde_json::Value))
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| ApiError(SharedError::Database(e.to_string())))?;
    }

    // J8: Publish to NATS so MQTT bridge can forward to Home Assistant, etc.
    let event = IoTTelemetryEvent {
        device_id: dto.device_id.clone(),
        tenant_id,
        site_id: None,
        timestamp: now,
        measurements: dto
            .measurements
            .iter()
            .map(|m| IoTMeasurement {
                capability: agrocore_messaging::IoTCapability::Custom(m.capability.clone()),
                value: m.value,
                unit: m.unit.clone(),
                quality: m.quality.clone(),
            })
            .collect(),
        metadata: dto.metadata.clone(),
    };

    if let Some(ref messaging) = state.messaging {
        let _ = messaging
            .publish(NATS_SUBJECT_TELEMETRY.to_string(), &event)
            .await;
    }

    info!(
        "Telemetry ingested for device {}: {} measurements",
        event.device_id,
        event.measurements.len()
    );

    Ok(HttpResponse::Created().finish())
}

/// Request body for telemetry ingestion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
struct IoTTelemetryIngestDto {
    #[validate(length(min = 1, max = 100))]
    device_id: String,
    #[validate(length(min = 1))]
    measurements: Vec<IoTTelemetryMeasurementDto>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
struct IoTTelemetryMeasurementDto {
    #[validate(length(min = 1, max = 50))]
    capability: String,
    value: f64,
    #[validate(length(min = 1, max = 20))]
    unit: String,
    quality: Option<String>,
}
