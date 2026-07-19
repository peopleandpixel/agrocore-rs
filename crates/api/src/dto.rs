use agrocore_domain::entities::equipment::{
    CreateEquipmentDto as DomainCreateEquipmentDto, Equipment, EquipmentType,
    UpdateEquipmentDto as DomainUpdateEquipmentDto,
};
use agrocore_domain::entities::order::{
    CreateOrderDto as DomainCreateOrderDto, Order, RecurrenceRule, TaskExecutionPolicy,
    UpdateOrderDto as DomainUpdateOrderDto,
};
use agrocore_domain::entities::plant_protection::PlantProtectionAreaMethod;
use agrocore_domain::entities::site::{
    Boundary, CreateSiteDto as DomainCreateSiteDto, GeoPoint, Site, SiteProperty,
    UpdateSiteDto as DomainUpdateSiteDto,
};
use agrocore_domain::entities::task::{CreateTaskDataDto as DomainCreateTaskDataDto, TaskData};
use agrocore_domain::entities::user::User;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::entities::{BbchStage, CropType, OrderStatus, OrderType, SiteType};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedEquipmentResponse {
    pub data: Vec<EquipmentDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EquipmentDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub label: String,
    pub code: Option<String>,
    pub equipment_type: EquipmentType,
    pub in_usage: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Equipment> for EquipmentDto {
    fn from(e: Equipment) -> Self {
        Self {
            id: e.id,
            tenant_id: e.tenant_id,
            label: e.label,
            code: e.code,
            equipment_type: e.equipment_type,
            in_usage: e.in_usage,
            created_at: e.created_at.to_rfc3339(),
            updated_at: e.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateEquipmentDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub code: Option<String>,
    pub equipment_type: EquipmentType,
}

impl From<CreateEquipmentDto> for DomainCreateEquipmentDto {
    fn from(dto: CreateEquipmentDto) -> Self {
        Self {
            label: dto.label,
            code: dto.code,
            equipment_type: dto.equipment_type,
            maintenance_intervals: None,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateEquipmentDto {
    #[validate(length(min = 1, max = 200))]
    pub label: Option<String>,
    pub code: Option<String>,
    pub equipment_type: Option<EquipmentType>,
    pub in_usage: Option<bool>,
}

impl From<UpdateEquipmentDto> for DomainUpdateEquipmentDto {
    fn from(dto: UpdateEquipmentDto) -> Self {
        Self {
            label: dto.label,
            code: dto.code,
            equipment_type: dto.equipment_type,
            in_usage: dto.in_usage,
            maintenance_intervals: None,
            next_maintenance_date: None,
            last_maintenance_hours: None,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedSiteResponse {
    pub data: Vec<SiteDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedOrderResponse {
    pub data: Vec<OrderDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedUserResponse {
    pub data: Vec<UserDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedTaskResponse {
    pub data: Vec<TaskDataDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedWeatherStationResponse {
    pub data: Vec<agrocore_domain::entities::weather::WeatherStation>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedWeatherDataResponse {
    pub data: Vec<agrocore_domain::entities::weather::WeatherData>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedPhenologyResponse {
    pub data: Vec<agrocore_domain::entities::weather::PhenologyRecord>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponseDto<T: Serialize> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedPACApplicationResponse {
    pub data: Vec<agrocore_domain::entities::finance::PACApplication>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedCostCenterResponse {
    pub data: Vec<agrocore_domain::entities::finance::CostCenter>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedFinancialRecordResponse {
    pub data: Vec<agrocore_domain::entities::finance::FinancialRecord>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedAnimalResponse {
    pub data: Vec<agrocore_domain::entities::livestock::Animal>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedWorkerTaskStatusResponse {
    pub data: Vec<WorkerTaskStatusDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WorkerTaskStatusDto {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub tenant_id: Uuid,
    pub status: WorkerTaskStatusTypeDto,
    pub started_at: Option<String>,
    pub paused_at: Option<String>,
    pub resumed_at: Option<String>,
    pub stopped_at: Option<String>,
    pub done_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkerTaskStatusTypeDto {
    New,
    Started,
    Paused,
    Stopped,
    Done,
}

impl From<agrocore_domain::entities::worker_task_status::WorkerTaskStatus> for WorkerTaskStatusDto {
    fn from(w: agrocore_domain::entities::worker_task_status::WorkerTaskStatus) -> Self {
        Self {
            task_id: w.task_id,
            worker_id: w.worker_id,
            tenant_id: w.tenant_id,
            status: w.status.into(),
            started_at: w.started_at.map(|d| d.to_rfc3339()),
            paused_at: w.paused_at.map(|d| d.to_rfc3339()),
            resumed_at: w.resumed_at.map(|d| d.to_rfc3339()),
            stopped_at: w.stopped_at.map(|d| d.to_rfc3339()),
            done_at: w.done_at.map(|d| d.to_rfc3339()),
            created_at: w.created_at.to_rfc3339(),
            updated_at: w.updated_at.to_rfc3339(),
        }
    }
}

impl From<WorkerTaskStatusTypeDto>
    for agrocore_domain::entities::worker_task_status::WorkerTaskStatusType
{
    fn from(s: WorkerTaskStatusTypeDto) -> Self {
        match s {
            WorkerTaskStatusTypeDto::New => Self::New,
            WorkerTaskStatusTypeDto::Started => Self::Started,
            WorkerTaskStatusTypeDto::Paused => Self::Paused,
            WorkerTaskStatusTypeDto::Stopped => Self::Stopped,
            WorkerTaskStatusTypeDto::Done => Self::Done,
        }
    }
}

impl From<agrocore_domain::entities::worker_task_status::WorkerTaskStatusType>
    for WorkerTaskStatusTypeDto
{
    fn from(s: agrocore_domain::entities::worker_task_status::WorkerTaskStatusType) -> Self {
        match s {
            agrocore_domain::entities::worker_task_status::WorkerTaskStatusType::New => Self::New,
            agrocore_domain::entities::worker_task_status::WorkerTaskStatusType::Started => {
                Self::Started
            }
            agrocore_domain::entities::worker_task_status::WorkerTaskStatusType::Paused => {
                Self::Paused
            }
            agrocore_domain::entities::worker_task_status::WorkerTaskStatusType::Stopped => {
                Self::Stopped
            }
            agrocore_domain::entities::worker_task_status::WorkerTaskStatusType::Done => Self::Done,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateWorkerTaskStatusDto {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub tenant_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateWorkerTaskStatusDto {
    pub status: WorkerTaskStatusTypeDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WorkerTaskStatusAggregateDto {
    pub task_id: Uuid,
    pub aggregated_status: WorkerTaskStatusTypeDto,
    pub worker_statuses: Vec<WorkerTaskStatusDto>,
}

// =============================================================================
// Applicator License DTOs
// =============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LicenseTypeDto {
    Basic,
    Advanced,
    Professional,
    Custom(String),
}

impl From<agrocore_domain::entities::plant_protection::LicenseType> for LicenseTypeDto {
    fn from(lt: agrocore_domain::entities::plant_protection::LicenseType) -> Self {
        match lt {
            agrocore_domain::entities::plant_protection::LicenseType::Basic => Self::Basic,
            agrocore_domain::entities::plant_protection::LicenseType::Advanced => Self::Advanced,
            agrocore_domain::entities::plant_protection::LicenseType::Professional => {
                Self::Professional
            }
            agrocore_domain::entities::plant_protection::LicenseType::Custom(s) => Self::Custom(s),
        }
    }
}

impl From<LicenseTypeDto> for agrocore_domain::entities::plant_protection::LicenseType {
    fn from(dto: LicenseTypeDto) -> Self {
        match dto {
            LicenseTypeDto::Basic => Self::Basic,
            LicenseTypeDto::Advanced => Self::Advanced,
            LicenseTypeDto::Professional => Self::Professional,
            LicenseTypeDto::Custom(s) => Self::Custom(s),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApplicatorLicenseDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub license_type: LicenseTypeDto,
    pub license_number: String,
    pub issued_by: String,
    pub valid_from: String,
    pub valid_until: String,
    pub is_active: bool,
    pub created_at: String,
}

impl From<agrocore_domain::entities::plant_protection::ApplicatorLicense> for ApplicatorLicenseDto {
    fn from(al: agrocore_domain::entities::plant_protection::ApplicatorLicense) -> Self {
        Self {
            id: al.id,
            user_id: al.user_id,
            license_type: al.license_type.into(),
            license_number: al.license_number,
            issued_by: al.issued_by,
            valid_from: al.valid_from.to_rfc3339(),
            valid_until: al.valid_until.to_rfc3339(),
            is_active: al.is_active,
            created_at: al.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateApplicatorLicenseDto {
    pub user_id: Uuid,
    pub license_type: LicenseTypeDto,
    #[validate(length(min = 1))]
    pub license_number: String,
    #[validate(length(min = 1))]
    pub issued_by: String,
    pub valid_from: String,
    pub valid_until: String,
}

impl From<CreateApplicatorLicenseDto>
    for agrocore_domain::entities::plant_protection::CreateApplicatorLicenseDto
{
    fn from(dto: CreateApplicatorLicenseDto) -> Self {
        Self {
            user_id: dto.user_id,
            license_type: dto.license_type.into(),
            license_number: dto.license_number,
            issued_by: dto.issued_by,
            valid_from: chrono::DateTime::parse_from_rfc3339(&dto.valid_from)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            valid_until: chrono::DateTime::parse_from_rfc3339(&dto.valid_until)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateApplicatorLicenseDto {
    pub license_type: Option<LicenseTypeDto>,
    #[validate(length(min = 1))]
    pub license_number: Option<String>,
    #[validate(length(min = 1))]
    pub issued_by: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub is_active: Option<bool>,
}

impl From<UpdateApplicatorLicenseDto>
    for agrocore_domain::entities::plant_protection::UpdateApplicatorLicenseDto
{
    fn from(dto: UpdateApplicatorLicenseDto) -> Self {
        Self {
            license_type: dto.license_type.map(|lt| lt.into()),
            license_number: dto.license_number,
            issued_by: dto.issued_by,
            valid_from: dto.valid_from.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            valid_until: dto.valid_until.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            is_active: dto.is_active,
        }
    }
}

// =============================================================================
// Compliance Checklist DTOs
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateComplianceChecklistDto {
    pub site_id: Uuid,
    pub checklist_type: agrocore_domain::entities::compliance::ChecklistType,
    pub items: Vec<agrocore_domain::entities::compliance::ChecklistItem>,
    pub due_date: Option<String>,
}

impl From<CreateComplianceChecklistDto>
    for agrocore_domain::entities::compliance::CreateComplianceChecklistDto
{
    fn from(dto: CreateComplianceChecklistDto) -> Self {
        Self {
            site_id: dto.site_id,
            checklist_type: dto.checklist_type,
            items: dto.items,
            due_date: dto.due_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateComplianceChecklistDto {
    pub status: Option<agrocore_domain::entities::compliance::ComplianceStatus>,
    pub items: Option<Vec<agrocore_domain::entities::compliance::ChecklistItem>>,
    pub due_date: Option<String>,
    pub completed_at: Option<String>,
}

impl From<UpdateComplianceChecklistDto>
    for agrocore_domain::entities::compliance::UpdateComplianceChecklistDto
{
    fn from(dto: UpdateComplianceChecklistDto) -> Self {
        Self {
            status: dto.status,
            items: dto.items,
            due_date: dto.due_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            completed_at: dto.completed_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
        }
    }
}

// =============================================================================
// Fertilizer Record DTOs
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateFertilizerRecordDto {
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    #[validate(length(min = 1, max = 200))]
    pub product_name: String,
    #[validate(range(min = 0.0))]
    pub nutrient_n: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_p: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_k: f64,
    #[validate(range(min = 0.0))]
    pub quantity_kg: f64,
    #[validate(range(min = 0.0))]
    pub area_ha: f64,
    pub application_date: String,
}

impl From<CreateFertilizerRecordDto>
    for agrocore_domain::entities::compliance::CreateFertilizerRecordDto
{
    fn from(dto: CreateFertilizerRecordDto) -> Self {
        Self {
            site_id: dto.site_id,
            order_id: dto.order_id,
            product_name: dto.product_name,
            nutrient_n: dto.nutrient_n,
            nutrient_p: dto.nutrient_p,
            nutrient_k: dto.nutrient_k,
            quantity_kg: dto.quantity_kg,
            area_ha: dto.area_ha,
            application_date: chrono::DateTime::parse_from_rfc3339(&dto.application_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateFertilizerRecordDto {
    pub site_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub nutrient_n: Option<f64>,
    pub nutrient_p: Option<f64>,
    pub nutrient_k: Option<f64>,
    pub quantity_kg: Option<f64>,
    pub area_ha: Option<f64>,
    pub application_date: Option<String>,
}

impl From<UpdateFertilizerRecordDto>
    for agrocore_domain::entities::fertilizer::UpdateFertilizerRecordDto
{
    fn from(dto: UpdateFertilizerRecordDto) -> Self {
        Self {
            site_id: dto.site_id,
            order_id: dto.order_id,
            product_name: dto.product_name,
            nutrient_n: dto.nutrient_n,
            nutrient_p: dto.nutrient_p,
            nutrient_k: dto.nutrient_k,
            quantity_kg: dto.quantity_kg,
            area_ha: dto.area_ha,
            application_date: dto.application_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponseDto {
    pub token: String,
    pub refresh_token: Option<String>,
    pub token_expires_in: i64, // Sekunden bis expiry
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub roles: Vec<UserRole>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SiteDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub label: String,
    pub site_type: SiteType,
    pub crop_type: CropType,
    pub variety: Option<String>,
    pub area: f64,
    pub gross_area: Option<f64>,
    pub bbch_stage: Option<BbchStage>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub center: Option<GeoPoint>,
    pub boundary: Option<Vec<GeoPoint>>,
    pub properties: Option<Vec<SiteProperty>>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Site> for SiteDto {
    fn from(s: Site) -> Self {
        Self {
            id: s.id,
            tenant_id: s.tenant_id,
            label: s.label,
            site_type: s.site_type,
            crop_type: s.crop_type,
            variety: s.variety,
            area: s.area,
            gross_area: s.gross_area,
            bbch_stage: s.bbch_stage,
            soil_type: s.soil_type,
            slope: s.slope,
            altitude: s.altitude,
            organic: s.organic,
            center: s.center,
            boundary: s.boundary.map(|b| b.0),
            properties: s.properties,
            is_active: s.is_active,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateSiteDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub site_type: SiteType,
    pub crop_type: CropType,
    pub variety: Option<String>,
    #[validate(range(min = 0.0))]
    pub area: f64,
    pub gross_area: Option<f64>,
    pub center: Option<GeoPoint>,
    pub boundary: Option<Vec<GeoPoint>>,
    pub plots: Option<Vec<agrocore_domain::entities::site::Plot>>,
    pub properties: Option<Vec<SiteProperty>>,
}

impl From<CreateSiteDto> for DomainCreateSiteDto {
    fn from(dto: CreateSiteDto) -> Self {
        Self {
            label: dto.label,
            site_type: dto.site_type,
            crop_type: dto.crop_type,
            variety: dto.variety,
            area: dto.area,
            gross_area: dto.gross_area,
            center: dto.center,
            boundary: dto.boundary.map(Boundary),
            plots: dto.plots,
            row_config: None,
            bbch_stage: None,
            planted_date: None,
            soil_type: None,
            slope: None,
            slope_facing: None,
            altitude: None,
            organic: None,
            sigpac_data: None,
            regepac_id: None,
            properties: dto.properties,
            custom_fields: None,
            note1: None,
            note2: None,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateSiteDto {
    #[validate(length(min = 1, max = 200))]
    pub label: Option<String>,
    pub variety: Option<String>,
    #[validate(range(min = 0.0))]
    pub area: Option<f64>,
    pub gross_area: Option<f64>,
    pub center: Option<GeoPoint>,
    pub boundary: Option<Vec<GeoPoint>>,
    pub properties: Option<Vec<SiteProperty>>,
    pub is_active: Option<bool>,
}

impl From<UpdateSiteDto> for DomainUpdateSiteDto {
    fn from(dto: UpdateSiteDto) -> Self {
        Self {
            label: dto.label,
            variety: dto.variety,
            area: dto.area,
            gross_area: dto.gross_area,
            center: dto.center,
            boundary: dto.boundary.map(Boundary),
            properties: dto.properties,
            is_active: dto.is_active,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub label: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub site_ids: Vec<Uuid>,
    pub assigned_worker_ids: Vec<Uuid>,
    pub planned_date: Option<String>,
    pub deadline_date: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub last_completed_at: Option<String>,
    pub recurrence: Option<RecurrenceRule>,
    pub execution_policy: Option<TaskExecutionPolicy>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Order> for OrderDto {
    fn from(o: Order) -> Self {
        Self {
            id: o.id,
            tenant_id: o.tenant_id,
            label: o.label,
            order_type: o.order_type,
            status: o.status,
            site_ids: o.site_ids,
            assigned_worker_ids: o.assigned_worker_ids,
            planned_date: o.planned_date.map(|d| d.to_rfc3339()),
            deadline_date: o.deadline_date.map(|d| d.to_rfc3339()),
            started_at: o.started_at.map(|d| d.to_rfc3339()),
            completed_at: o.completed_at.map(|d| d.to_rfc3339()),
            last_completed_at: o.last_completed_at.map(|d| d.to_rfc3339()),
            recurrence: o.recurrence,
            execution_policy: o.execution_policy,
            is_active: o.is_active,
            created_at: o.created_at.to_rfc3339(),
            updated_at: o.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateOrderDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub order_type: OrderType,
    #[validate(length(min = 1))]
    pub site_ids: Vec<Uuid>,
    pub assigned_worker_ids: Option<Vec<Uuid>>,
    pub planned_date: Option<chrono::DateTime<chrono::Utc>>,
    pub deadline_date: Option<chrono::DateTime<chrono::Utc>>,
    pub recurrence: Option<RecurrenceRule>,
    pub execution_policy: Option<TaskExecutionPolicy>,
}

impl From<CreateOrderDto> for DomainCreateOrderDto {
    fn from(dto: CreateOrderDto) -> Self {
        Self {
            label: dto.label,
            order_type: dto.order_type,
            site_ids: dto.site_ids,
            assigned_worker_ids: dto.assigned_worker_ids,
            planned_date: dto.planned_date,
            deadline_date: dto.deadline_date,
            articles: None,
            quantities: None,
            custom_fields: None,
            parent_order_id: None,
            workflow_config: None,
            recurrence: dto.recurrence,
            execution_policy: dto.execution_policy,
            cost_center_id: None,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateOrderDto {
    #[validate(length(min = 1, max = 200))]
    pub label: Option<String>,
    pub status: Option<OrderStatus>,
    #[validate(length(min = 1))]
    pub site_ids: Option<Vec<Uuid>>,
    pub assigned_worker_ids: Option<Vec<Uuid>>,
    pub planned_date: Option<chrono::DateTime<chrono::Utc>>,
    pub deadline_date: Option<chrono::DateTime<chrono::Utc>>,
    pub recurrence: Option<RecurrenceRule>,
    pub execution_policy: Option<TaskExecutionPolicy>,
    pub is_active: Option<bool>,
}

impl From<UpdateOrderDto> for DomainUpdateOrderDto {
    fn from(dto: UpdateOrderDto) -> Self {
        Self {
            label: dto.label,
            status: dto.status,
            site_ids: dto.site_ids,
            assigned_worker_ids: dto.assigned_worker_ids,
            planned_date: dto.planned_date,
            deadline_date: dto.deadline_date,
            recurrence: dto.recurrence,
            execution_policy: dto.execution_policy,
            is_active: dto.is_active,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct UserDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub roles: Vec<UserRole>,
    pub is_active: bool,
    pub language: Option<String>,
    pub color: Option<String>,
    pub last_login: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            tenant_id: u.tenant_id,
            firstname: u.firstname,
            lastname: u.lastname,
            email: u.email,
            roles: u.roles,
            is_active: u.is_active,
            language: u.language,
            color: u.color,
            last_login: u.last_login.map(|d| d.to_rfc3339()),
            created_at: u.created_at.to_rfc3339(),
            updated_at: u.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate, Clone)]
pub struct CreateUserDto {
    #[validate(length(min = 1, max = 100))]
    pub firstname: String,
    #[validate(length(min = 1, max = 100))]
    pub lastname: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub roles: Option<Vec<UserRole>>,
}

impl From<CreateUserDto> for agrocore_domain::entities::user::CreateUserDto {
    fn from(dto: CreateUserDto) -> Self {
        Self {
            firstname: dto.firstname,
            lastname: dto.lastname,
            email: dto.email,
            password: dto.password,
            roles: dto.roles,
            internal_cost_per_hour: None,
            external_cost_per_hour: None,
            language: None,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateUserDto {
    #[validate(length(min = 1, max = 100))]
    pub firstname: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub lastname: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub roles: Option<Vec<UserRole>>,
    pub is_active: Option<bool>,
}

impl From<UpdateUserDto> for agrocore_domain::entities::user::UpdateUserDto {
    fn from(dto: UpdateUserDto) -> Self {
        Self {
            firstname: dto.firstname,
            lastname: dto.lastname,
            email: dto.email,
            roles: dto.roles,
            is_active: dto.is_active,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskDataDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub order_id: Uuid,
    pub worker_id: Uuid,
    pub site_id: Uuid,
    pub description: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_minutes: Option<u32>,
    pub area_covered: Option<f64>,
    pub observations: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<TaskData> for TaskDataDto {
    fn from(t: TaskData) -> Self {
        Self {
            id: t.id,
            tenant_id: t.tenant_id,
            order_id: t.order_id,
            worker_id: t.worker_id,
            site_id: t.site_id,
            description: t.description,
            started_at: t.started_at.to_rfc3339(),
            ended_at: t.ended_at.map(|d| d.to_rfc3339()),
            duration_minutes: t.duration_minutes.map(|d| d as u32),
            area_covered: t.area_covered,
            observations: t.observations,
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateTaskDataDto {
    pub order_id: Uuid,
    pub site_id: Uuid,
    #[validate(length(min = 1, max = 1000))]
    pub description: String,
}

impl From<CreateTaskDataDto> for DomainCreateTaskDataDto {
    fn from(dto: CreateTaskDataDto) -> Self {
        Self {
            order_id: dto.order_id,
            site_id: dto.site_id,
            description: dto.description,
            started_at: None,
            ended_at: None,
            paused_at: None,
            resume_at: None,
            duration_minutes: None,
            area_covered: None,
            materials_used: None,
            observations: None,
            gps_track: None,
            photo_urls: None,
            machine_id: None,
            machine_hours: None,
            cost_center_id: None,
        }
    }
}

impl From<CreateTaskDataDto> for agrocore_domain::entities::task::UpdateTaskDataDto {
    fn from(dto: CreateTaskDataDto) -> Self {
        Self {
            order_id: Some(dto.order_id),
            site_id: Some(dto.site_id),
            description: Some(dto.description),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MaterialCalculationRequestDto {
    pub method: PlantProtectionAreaMethod,
    pub site_id: Uuid,
    pub dosage_per_ha: f64,
    pub application_date: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MaterialCalculationResponseDto {
    pub treated_area_ha: f64,
    pub total_material_amount: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WaterRateCalculationRequestDto {
    pub speed_kmh: f64,
    pub nozzle_flow_lmin: f64,
    pub lane_width: f64,
    pub number_of_nozzles: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WaterRateCalculationResponseDto {
    pub water_rate_lha: f64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct NutritionDemandRequestDto {
    pub site_id: uuid::Uuid,
    pub target_yield_t_ha: f64,
    pub demand_per_t: agrocore_domain::services::nutrition::NutrientValues,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct NutritionDemandResponseDto {
    pub total_demand: agrocore_domain::services::nutrition::NutrientValues,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct FertilizerCalculationRequestDto {
    pub demand: agrocore_domain::services::nutrition::NutrientValues,
    pub fertilizer: agrocore_domain::services::nutrition::Fertilizer,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct FertilizerCalculationResponseDto {
    pub fertilizer_amount_kg: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    #[test]
    fn create_site_dto_into_domain_preserves_geodata_and_optional_fields() {
        let center = GeoPoint {
            lng: 14.0,
            lat: 47.0,
        };
        let boundary = vec![
            GeoPoint {
                lng: 14.0,
                lat: 47.0,
            },
            GeoPoint {
                lng: 14.1,
                lat: 47.0,
            },
            GeoPoint {
                lng: 14.1,
                lat: 47.1,
            },
        ];
        let dto = CreateSiteDto {
            label: String::from("Plot A"),
            site_type: SiteType::Field,
            crop_type: CropType::Grape,
            variety: Some(String::from("Syrah")),
            area: 12.5,
            gross_area: Some(13.0),
            center: Some(center.clone()),
            boundary: Some(boundary.clone()),
            plots: None,
            properties: Some(vec![SiteProperty {
                key: String::from("soil_ph"),
                value: json!(6.4),
                group: Some(String::from("soil")),
            }]),
        };

        let domain: agrocore_domain::entities::site::CreateSiteDto = dto.into();
        assert_eq!(domain.label, "Plot A");
        assert_eq!(domain.site_type, SiteType::Field);
        assert_eq!(domain.crop_type, CropType::Grape);
        assert_eq!(domain.variety.as_deref(), Some("Syrah"));
        assert_eq!(domain.area, 12.5);
        assert_eq!(domain.gross_area, Some(13.0));
        assert!(domain.plots.is_none());
        assert!(domain.row_config.is_none());
        assert!(domain.bbch_stage.is_none());
        assert_eq!(domain.center.as_ref().map(|p| p.lng), Some(center.lng));
        assert_eq!(domain.center.as_ref().map(|p| p.lat), Some(center.lat));
        assert_eq!(
            domain.boundary.as_ref().map(|b| b.len()),
            Some(boundary.len())
        );
        assert!(domain.custom_fields.is_none());
        assert_eq!(domain.properties.as_ref().map(Vec::len), Some(1));
    }

    #[test]
    fn create_order_dto_into_domain_keeps_assignment_information() {
        let site_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let worker_ids = vec![Uuid::new_v4()];
        let recurrence = RecurrenceRule {
            cadence: agrocore_domain::entities::order::RecurrenceCadence::Daily,
            every: 2,
            day_of_month: None,
            month: None,
        };
        let dto = CreateOrderDto {
            label: String::from("Harvest 2026"),
            order_type: OrderType::Harvest,
            site_ids: site_ids.clone(),
            assigned_worker_ids: Some(worker_ids.clone()),
            planned_date: None,
            deadline_date: None,
            recurrence: Some(recurrence.clone()),
            execution_policy: None,
        };

        let domain: agrocore_domain::entities::order::CreateOrderDto = dto.into();
        assert_eq!(domain.label, "Harvest 2026");
        assert_eq!(domain.order_type, OrderType::Harvest);
        assert_eq!(domain.site_ids, site_ids);
        assert_eq!(domain.assigned_worker_ids, Some(worker_ids));
        assert!(domain.planned_date.is_none());
        assert!(domain.deadline_date.is_none());
        assert_eq!(domain.recurrence, Some(recurrence));
        assert!(domain.execution_policy.is_none());
        assert!(domain.workflow_config.is_none());
    }

    #[test]
    fn create_user_dto_into_domain_sets_backend_defaults() {
        let dto = CreateUserDto {
            firstname: String::from("Anna"),
            lastname: String::from("Meyer"),
            email: String::from("anna@example.com"),
            password: String::from("secure-pass-123"),
            roles: Some(vec![UserRole::Manager]),
        };

        let domain: agrocore_domain::entities::user::CreateUserDto = dto.into();
        assert_eq!(domain.firstname, "Anna");
        assert_eq!(domain.lastname, "Meyer");
        assert_eq!(domain.email, "anna@example.com");
        assert_eq!(domain.password, "secure-pass-123");
        assert_eq!(domain.roles, Some(vec![UserRole::Manager]));
        assert!(domain.internal_cost_per_hour.is_none());
        assert!(domain.external_cost_per_hour.is_none());
        assert!(domain.language.is_none());
    }

    #[test]
    fn dto_response_types_format_timestamps_as_rfc3339() {
        let user = User {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            firstname: String::from("Anna"),
            lastname: String::from("Meyer"),
            email: String::from("anna@example.com"),
            password_hash: String::from("hash"),
            roles: vec![UserRole::Viewer],
            is_active: true,
            internal_cost_per_hour: None,
            external_cost_per_hour: None,
            color: None,
            language: Some(String::from("de")),
            assigned_site_ids: None,
            last_login: Some(Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap()),
            refresh_token: None,
            refresh_token_expires_at: None,
            created_at: Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 3, 4, 5, 6).unwrap(),
        };

        let dto = UserDto::from(user);
        assert_eq!(dto.last_login.as_deref(), Some("2026-01-02T03:04:05+00:00"));
        assert_eq!(dto.created_at, "2026-01-02T03:04:05+00:00");
        assert_eq!(dto.updated_at, "2026-01-03T04:05:06+00:00");
    }
}
