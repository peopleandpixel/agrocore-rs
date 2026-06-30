use agrocore_domain::entities::order::{Order, CreateOrderDto as DomainCreateOrderDto, UpdateOrderDto as DomainUpdateOrderDto};
use agrocore_domain::entities::site::{Site, CreateSiteDto as DomainCreateSiteDto, UpdateSiteDto as DomainUpdateSiteDto, SiteProperty};
use agrocore_domain::entities::equipment::{Equipment, CreateEquipmentDto as DomainCreateEquipmentDto, UpdateEquipmentDto as DomainUpdateEquipmentDto, EquipmentType};
use agrocore_domain::entities::task::{TaskData, CreateTaskDataDto as DomainCreateTaskDataDto};
use agrocore_domain::entities::plant_protection::PlantProtectionAreaMethod;
use agrocore_domain::entities::user::User;
use agrocore_domain::entities::{BbchStage, CropType, OrderStatus, OrderType, SiteType};
use agrocore_domain::entities::user::UserRole;
use serde::{Serialize, Deserialize};
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
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponseDto {
    pub token: String,
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
            plots: dto.plots,
            row_config: None,
            bbch_stage: None,
            planted_date: None,
            soil_type: None,
            slope: None,
            slope_facing: None,
            altitude: None,
            organic: None,
            center: None,
            sigpac_data: None,
            regepac_id: None,
            boundary: None,
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
}

impl From<CreateOrderDto> for DomainCreateOrderDto {
    fn from(dto: CreateOrderDto) -> Self {
        Self {
            label: dto.label,
            order_type: dto.order_type,
            site_ids: dto.site_ids,
            assigned_worker_ids: dto.assigned_worker_ids,
            planned_date: None,
            deadline_date: None,
            articles: None,
            quantities: None,
            custom_fields: None,
            parent_order_id: None,
            workflow_config: None,
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
    pub is_active: Option<bool>,
}

impl From<UpdateOrderDto> for DomainUpdateOrderDto {
    fn from(dto: UpdateOrderDto) -> Self {
        Self {
            label: dto.label,
            status: dto.status,
            site_ids: dto.site_ids,
            assigned_worker_ids: dto.assigned_worker_ids,
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
            duration_minutes: t.duration_minutes,
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
