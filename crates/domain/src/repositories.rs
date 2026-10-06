use crate::entities::equipment::{MaintenanceCostSummaryDto, MaintenanceLogDto};
use crate::entities::tenant::TenantId;
pub use agrocore_shared::{PaginatedResponse, Pagination, Result};
use serde::Serialize;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use crate::entities::user::UserRole;
#[cfg(feature = "mocks")]
use mockall::automock;

pub type RepositoryFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

pub trait VisibilityAwareEntity {}

#[cfg_attr(feature = "mocks", automock)]
pub trait Repository<T>: Send + Sync
where
    T: Serialize + Send + Sync + 'static,
{
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<T>>;
    fn find_all(
        &self,
        tid: TenantId,
        pagination: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<T>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

use crate::entities::equipment::{
    CreateEquipmentDto, DepreciationScheduleEntry, Equipment, EquipmentDepreciationDto,
    FuelConsumptionDto, UpdateEquipmentDto, UsageLogDto, UsageSummaryDto,
};
use crate::entities::site::{CreateSiteDto, Site, UpdateSiteDto};
use crate::entities::spatial::SpatialObject;

#[cfg_attr(feature = "mocks", automock)]
pub trait EquipmentRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Equipment>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<Option<Equipment>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Equipment>>;
    #[allow(clippy::too_many_arguments)]
    fn find_all_filtered<'b>(
        &'b self,
        tid: TenantId,
        p: Pagination,
        search: Option<&'b str>,
        equipment_type: Option<&'b str>,
        in_usage: Option<bool>,
        needs_maintenance: Option<bool>,
        fuel_efficiency_range: Option<(f64, f64)>,
        location_filter: Option<&'b str>,
    ) -> RepositoryFuture<PaginatedResponse<Equipment>>;
    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Equipment>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateEquipmentDto,
        by: Uuid,
    ) -> RepositoryFuture<Equipment>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateEquipmentDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Equipment>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    /// Get maintenance log entries for an equipment, newest first.
    fn get_maintenance_log(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Vec<MaintenanceLogDto>>;
    /// Find all equipment that needs maintenance (next_maintenance_date <= now)
    fn find_maintenance_due(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Equipment>>;
    /// Record a maintenance event and recalculate next_maintenance_date
    fn record_maintenance(
        &self,
        tid: TenantId,
        id: Uuid,
        hours: f64,
        note: Option<String>,
    ) -> RepositoryFuture<Option<Equipment>>;
    /// Get aggregated maintenance cost summary for an equipment
    fn get_maintenance_cost_summary(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Option<MaintenanceCostSummaryDto>>;
    /// Update a maintenance log entry with cost details
    fn update_maintenance_costs(
        &self,
        tid: TenantId,
        log_id: Uuid,
        parts_cost: f64,
        labor_hours: f64,
        downtime_hours: f64,
    ) -> RepositoryFuture<bool>;
    /// Get fuel consumption history for an equipment, newest first.
    fn get_fuel_consumption(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Vec<FuelConsumptionDto>>;
    /// Record a fuel consumption entry.
    #[allow(clippy::too_many_arguments)]
    fn record_fuel_consumption<'b>(
        &'b self,
        tid: TenantId,
        equipment_id: Uuid,
        liters: f64,
        cost_per_liter: Option<f64>,
        operation_type: Option<&'b str>,
        field_id: Option<Uuid>,
        hours_operated: Option<f64>,
        notes: Option<&'b str>,
    ) -> RepositoryFuture<Option<FuelConsumptionDto>>;
    /// Get usage log history for an equipment, newest first.
    fn get_usage_log(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Vec<UsageLogDto>>;
    /// Get aggregated usage summary for an equipment.
    fn get_usage_summary(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Option<UsageSummaryDto>>;
    /// Record a usage log entry for an equipment.
    #[allow(clippy::too_many_arguments)]
    fn record_usage<'b>(
        &'b self,
        tid: TenantId,
        equipment_id: Uuid,
        worker_id: Option<Uuid>,
        task_id: Option<Uuid>,
        operation_type: Option<&'b str>,
        started_at: DateTime<Utc>,
        ended_at: Option<DateTime<Utc>>,
        hours_operated: f64,
        note: Option<&'b str>,
    ) -> RepositoryFuture<Option<UsageLogDto>>;
    /// Get aggregated depreciation info for an equipment.
    fn get_depreciation(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Option<EquipmentDepreciationDto>>;
    /// Get the yearly depreciation schedule for an equipment.
    fn get_depreciation_schedule(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Vec<DepreciationScheduleEntry>>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait SiteRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Site>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<Option<Site>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Site>>;
    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Site>>;
    fn create(&self, tid: TenantId, dto: CreateSiteDto, by: Uuid) -> RepositoryFuture<Site>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateSiteDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Site>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait SpatialObjectRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<SpatialObject>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<SpatialObject>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    fn find_containing_point(
        &self,
        tid: TenantId,
        point: crate::entities::spatial::types::GeoPoint,
        site_id: Option<Uuid>,
    ) -> RepositoryFuture<Vec<SpatialObject>>;

    /// Objects on a plot, optionally narrowed to one object type.
    ///
    /// This is the map's read path. `find_all` pages the whole tenant, which is the wrong
    /// query for a viewport: a farm with 40,000 trees needs "the olives on plot 3", not
    /// page 1 of everything ordered by insertion.
    ///
    /// `bbox` is PostGIS `ST_MakeEnvelope(min_lng, min_lat, max_lng, max_lat)` order, as
    /// WGS84. The map supplies it from the viewport; `None` means no spatial filter.
    ///
    /// `include_inactive` defaults to false, because an object retired last season should
    /// not be drawn on the map by default -- but it is offered, because "show me the old
    /// orchard as well" is a real question when reconciling a planting plan.
    fn find_by_filter(
        &self,
        tid: TenantId,
        filter: SpatialObjectFilter,
    ) -> RepositoryFuture<Vec<SpatialObject>>;
}

/// Query for [`SpatialObjectRepository::find_by_filter`].
///
/// Every field is optional and they combine with AND. An empty filter is valid and means
/// "everything in the tenant", which is deliberately still a tenant-scoped query rather
/// than an unscoped one.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SpatialObjectFilter {
    pub site_id: Option<Uuid>,
    /// The `SpatialObjectType` stored in the column, e.g. `olive_tree`. Matched exactly:
    /// the map asks for a type, not for a family of types.
    pub object_type: Option<String>,
    pub parent_id: Option<Uuid>,
    /// Viewport as (min_lng, min_lat, max_lng, max_lat) in WGS84.
    pub bbox: Option<(f64, f64, f64, f64)>,
    /// Only objects with a planting date, or only those without. The map cannot tell a
    /// mature tree from a newly planted one without this.
    pub planted_at: Option<PlantedAtFilter>,
    pub include_inactive: bool,
    /// Hard cap on returned rows. The map viewport query is the one place where a
    /// generous limit is right, so this is explicit rather than paginated.
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum PlantedAtFilter {
    Before(chrono::NaiveDate),
    After(chrono::NaiveDate),
    IsNull,
    IsSet,
}

impl SpatialObjectFilter {
    pub fn on_plot(site_id: Uuid) -> Self {
        Self {
            site_id: Some(site_id),
            ..Default::default()
        }
    }

    pub fn of_type(object_type: impl Into<String>) -> Self {
        Self {
            object_type: Some(object_type.into()),
            ..Default::default()
        }
    }

    pub fn with_limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }
}

use crate::entities::livestock::{Animal, CreateAnimalDto, UpdateAnimalDto};

#[cfg_attr(feature = "mocks", automock)]
pub trait AnimalRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Animal>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<Option<Animal>>;
    fn find_all(&self, tid: TenantId, p: Pagination)
    -> RepositoryFuture<PaginatedResponse<Animal>>;
    fn create(&self, tid: TenantId, dto: CreateAnimalDto, by: Uuid) -> RepositoryFuture<Animal>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateAnimalDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Animal>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    fn add_treatment(
        &self,
        tid: TenantId,
        id: Uuid,
        treatment: crate::entities::livestock::TreatmentRecord,
    ) -> RepositoryFuture<bool>;
    fn find_treatments_by_animal(
        &self,
        tid: TenantId,
        animal_id: Uuid,
    ) -> RepositoryFuture<Option<Vec<crate::entities::livestock::TreatmentRecord>>>;
    fn add_grazing_record(
        &self,
        tid: TenantId,
        id: Uuid,
        record: crate::entities::livestock::GrazingRecord,
    ) -> RepositoryFuture<bool>;
}

use crate::entities::worker_task_status::{
    CreateWorkerTaskStatusDto, WorkerTaskStatus, WorkerTaskStatusType,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait WorkerTaskStatusRepository: Send + Sync {
    fn find_by_task_and_worker(
        &self,
        tid: TenantId,
        task_id: Uuid,
        worker_id: Uuid,
    ) -> RepositoryFuture<Option<WorkerTaskStatus>>;
    fn find_all_for_task(
        &self,
        tid: TenantId,
        task_id: Uuid,
    ) -> RepositoryFuture<Vec<WorkerTaskStatus>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateWorkerTaskStatusDto,
    ) -> RepositoryFuture<WorkerTaskStatus>;
    fn update_status(
        &self,
        tid: TenantId,
        task_id: Uuid,
        worker_id: Uuid,
        status: WorkerTaskStatusType,
    ) -> RepositoryFuture<Option<WorkerTaskStatus>>;
}

// --- Order Repository ---
use crate::entities::order::{CreateOrderDto, Order, UpdateOrderDto};

#[cfg_attr(feature = "mocks", automock)]
pub trait OrderRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Order>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<Option<Order>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Order>>;
    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Order>>;
    fn find_my_tasks(&self, tid: TenantId, user_id: Uuid) -> RepositoryFuture<Vec<Order>>;
    fn create(&self, tid: TenantId, dto: CreateOrderDto, by: Uuid) -> RepositoryFuture<Order>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateOrderDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Order>>;
    fn find_assigned_to_worker(
        &self,
        tid: TenantId,
        worker_id: Uuid,
    ) -> RepositoryFuture<Vec<Order>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    /// Find all sales orders for a specific customer
    fn find_by_customer(
        &self,
        tid: TenantId,
        customer_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Order>>;
}

// --- Customer Repository ---
use crate::entities::customer::{CreateCustomerDto, Customer, UpdateCustomerDto};

#[cfg_attr(feature = "mocks", automock)]
pub trait CustomerRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Customer>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<Option<Customer>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Customer>>;
    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Customer>>;
    fn find_by_customer_number(
        &self,
        tid: TenantId,
        number: &str,
    ) -> RepositoryFuture<Option<Customer>>;
    fn create(&self, tid: TenantId, dto: CreateCustomerDto, by: Uuid)
    -> RepositoryFuture<Customer>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateCustomerDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Customer>>;
    fn search(
        &self,
        tid: TenantId,
        query: &str,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Customer>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

// --- Tenant Repository ---
use crate::entities::tenant::{CreateTenantDto, Tenant, UpdateTenantDto};

#[cfg_attr(feature = "mocks", automock)]
pub trait TenantRepository: Send + Sync {
    fn find_by_id(&self, id: Uuid) -> RepositoryFuture<Option<Tenant>>;
    fn find_all(&self, p: Pagination) -> RepositoryFuture<PaginatedResponse<Tenant>>;
    fn create(&self, dto: CreateTenantDto) -> RepositoryFuture<Tenant>;
    fn update(&self, id: Uuid, dto: UpdateTenantDto) -> RepositoryFuture<Option<Tenant>>;
    fn delete(&self, id: Uuid) -> RepositoryFuture<bool>;
}

// --- User Repository ---
use crate::entities::user::{AuthResponse, CreateUserDto, LoginDto, UpdateUserDto, User};

#[cfg_attr(feature = "mocks", automock)]
pub trait UserRepository: Send + Sync {
    fn count_all(&self) -> RepositoryFuture<i64>;
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<User>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<User>>;
    fn find_by_email(&self, email: &str) -> RepositoryFuture<Option<User>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<User>>;
    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<PaginatedResponse<User>>;
    fn create(&self, tid: TenantId, dto: CreateUserDto, by: Uuid) -> RepositoryFuture<User>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateUserDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<User>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    fn authenticate(&self, dto: LoginDto) -> RepositoryFuture<AuthResponse>;
    /// Resolve a refresh token to its user.
    ///
    /// Runs before the tenant is known (the token is the credential), so it is
    /// deliberately tenant-agnostic and reaches `users` through the restricted
    /// `agrocore_auth` role rather than through a pin.
    fn find_by_refresh_token(&self, refresh_token: &str) -> RepositoryFuture<Option<User>>;
    /// Clear a stored refresh token. Tenant-scoped, like
    /// [`Self::update_refresh_token`].
    fn invalidate_refresh_token(
        &self,
        tenant_id: TenantId,
        user_id: Uuid,
    ) -> RepositoryFuture<bool>;
    /// Persist a refresh token.
    ///
    /// Takes the tenant because the write is subject to row-level security:
    /// `users_update` requires `tenant_id = get_current_tenant_id()`. Without
    /// it the UPDATE matches no rows and reports `false`.
    fn update_refresh_token(
        &self,
        tenant_id: TenantId,
        user_id: Uuid,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> RepositoryFuture<bool>;
}

// --- Plant Protection Repository ---
use crate::entities::plant_protection::{
    ApplicatorLicense, CreateApplicatorLicenseDto, CreatePlantProtectionDto, PlantProtectionRecord,
    UpdateApplicatorLicenseDto, UpdatePlantProtectionDto,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait PlantProtectionRecordRepo: Send + Sync {
    fn find_by_id(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Option<PlantProtectionRecord>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<PlantProtectionRecord>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreatePlantProtectionDto,
        by: Uuid,
    ) -> RepositoryFuture<PlantProtectionRecord>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdatePlantProtectionDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<PlantProtectionRecord>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    fn find_applicator_license_by_user(
        &self,
        tid: TenantId,
        user_id: Uuid,
    ) -> RepositoryFuture<Option<ApplicatorLicense>>;
    fn find_all_applicator_licenses(
        &self,
        tid: TenantId,
    ) -> RepositoryFuture<Vec<ApplicatorLicense>>;
    fn create_applicator_license(
        &self,
        tid: TenantId,
        dto: CreateApplicatorLicenseDto,
    ) -> RepositoryFuture<ApplicatorLicense>;
    fn update_applicator_license(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateApplicatorLicenseDto,
    ) -> RepositoryFuture<Option<ApplicatorLicense>>;
    fn delete_applicator_license(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

// --- Compliance Repository ---
use crate::entities::compliance::{
    AuditLog, ComplianceChecklist, CreateAuditLogDto, CreateComplianceChecklistDto,
    UpdateComplianceChecklistDto,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait ComplianceChecklistRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<ComplianceChecklist>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<ComplianceChecklist>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateComplianceChecklistDto,
        by: Uuid,
    ) -> RepositoryFuture<ComplianceChecklist>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateComplianceChecklistDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<ComplianceChecklist>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait AuditLogRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<AuditLog>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<AuditLog>>;
    fn create(&self, tid: TenantId, dto: CreateAuditLogDto) -> RepositoryFuture<AuditLog>;
}

// --- Weather Repository ---
use crate::entities::weather::{
    CreateFrostWarningDto, CreateGrowingDegreeDayDto, CreatePestRiskDto, CreatePhenologyRecordDto,
    CreateSoilMoistureConfigDto, CreateWeatherDataDto, CreateWeatherStationDto, FrostWarning,
    GrowingDegreeDay, PestRisk, PhenologyRecord, SoilMoistureAlert, SoilMoistureConfig,
    SoilMoistureReading, UpdateFrostWarningDto, UpdatePhenologyRecordDto, UpdateWeatherDataDto,
    UpdateWeatherStationDto, WeatherData, WeatherStation,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait WeatherStationRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WeatherStation>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WeatherStation>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateWeatherStationDto,
        by: Uuid,
    ) -> RepositoryFuture<WeatherStation>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateWeatherStationDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<WeatherStation>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait WeatherDataRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WeatherData>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WeatherData>>;
    fn find_by_station(
        &self,
        tid: TenantId,
        station_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WeatherData>>;
    fn create(&self, tid: TenantId, dto: CreateWeatherDataDto) -> RepositoryFuture<WeatherData>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateWeatherDataDto,
    ) -> RepositoryFuture<Option<WeatherData>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait PhenologyRecordRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<PhenologyRecord>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<PhenologyRecord>>;
    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<PhenologyRecord>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreatePhenologyRecordDto,
        by: Uuid,
    ) -> RepositoryFuture<PhenologyRecord>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdatePhenologyRecordDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<PhenologyRecord>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait FrostWarningRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<FrostWarning>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<FrostWarning>>;
    fn find_by_station(
        &self,
        tid: TenantId,
        station_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<FrostWarning>>;
    fn find_active(&self, tid: TenantId) -> RepositoryFuture<Vec<FrostWarning>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateFrostWarningDto,
        by: Uuid,
    ) -> RepositoryFuture<FrostWarning>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateFrostWarningDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<FrostWarning>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait GrowingDegreeDayRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<GrowingDegreeDay>>;
    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<GrowingDegreeDay>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateGrowingDegreeDayDto,
    ) -> RepositoryFuture<GrowingDegreeDay>;
    fn accumulated_gdd(
        &self,
        tid: TenantId,
        site_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        crop_type: String,
    ) -> RepositoryFuture<f64>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait PestRiskRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<PestRisk>>;
    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<PestRisk>>;
    fn create(&self, tid: TenantId, dto: CreatePestRiskDto) -> RepositoryFuture<PestRisk>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait SoilMoistureReadingRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<SoilMoistureReading>>;
    fn find_by_station(
        &self,
        tid: TenantId,
        station_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<SoilMoistureReading>>;
    fn find_recent(
        &self,
        tid: TenantId,
        station_id: Uuid,
        limit: u32,
    ) -> RepositoryFuture<Vec<SoilMoistureReading>>;
    fn create(
        &self,
        tid: TenantId,
        reading: SoilMoistureReading,
    ) -> RepositoryFuture<SoilMoistureReading>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait SoilMoistureConfigRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<SoilMoistureConfig>>;
    fn find_by_station(
        &self,
        tid: TenantId,
        station_id: Uuid,
    ) -> RepositoryFuture<Vec<SoilMoistureConfig>>;
    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<SoilMoistureConfig>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateSoilMoistureConfigDto,
    ) -> RepositoryFuture<SoilMoistureConfig>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: CreateSoilMoistureConfigDto,
    ) -> RepositoryFuture<Option<SoilMoistureConfig>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait SoilMoistureAlertRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<SoilMoistureAlert>>;
    fn find_unresolved(
        &self,
        tid: TenantId,
        station_id: Uuid,
    ) -> RepositoryFuture<Vec<SoilMoistureAlert>>;
    fn create(
        &self,
        tid: TenantId,
        alert: SoilMoistureAlert,
    ) -> RepositoryFuture<SoilMoistureAlert>;
    fn mark_resolved(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

use chrono::{DateTime, Utc};

use crate::entities::harvest::{
    ColdChainLog, CreateColdChainLogDto, CreateHarvestDeliveryDto, CreateHarvestLotDto,
    CreateHarvestSeasonDto, HarvestDelivery, HarvestLot, HarvestSeason, UpdateColdChainLogDto,
    UpdateHarvestDeliveryDto, UpdateHarvestLotDto, UpdateHarvestSeasonDto,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait HarvestSeasonRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<HarvestSeason>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<HarvestSeason>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateHarvestSeasonDto,
        by: Uuid,
    ) -> RepositoryFuture<HarvestSeason>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateHarvestSeasonDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<HarvestSeason>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait HarvestLotRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<HarvestLot>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<HarvestLot>>;
    fn find_by_season(
        &self,
        tid: TenantId,
        season_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<HarvestLot>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateHarvestLotDto,
        by: Uuid,
    ) -> RepositoryFuture<HarvestLot>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateHarvestLotDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<HarvestLot>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait HarvestDeliveryRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<HarvestDelivery>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<HarvestDelivery>>;
    fn find_by_lot(
        &self,
        tid: TenantId,
        lot_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<HarvestDelivery>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateHarvestDeliveryDto,
        by: Uuid,
    ) -> RepositoryFuture<HarvestDelivery>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateHarvestDeliveryDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<HarvestDelivery>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait ColdChainLogRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<ColdChainLog>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<ColdChainLog>>;
    fn find_by_lot(
        &self,
        tid: TenantId,
        lot_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<ColdChainLog>>;
    fn create(&self, tid: TenantId, dto: CreateColdChainLogDto) -> RepositoryFuture<ColdChainLog>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateColdChainLogDto,
    ) -> RepositoryFuture<Option<ColdChainLog>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

// --- Olive Repository ---
use crate::entities::olive::{
    CreateOliveGroveDto, CreateOliveOilRecordDto, OliveGrove, OliveOilRecord, UpdateOliveGroveDto,
    UpdateOliveOilRecordDto,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait OliveGroveRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<OliveGrove>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<OliveGrove>>;
    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<OliveGrove>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateOliveGroveDto,
        by: Uuid,
    ) -> RepositoryFuture<OliveGrove>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateOliveGroveDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<OliveGrove>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait OliveOilRecordRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<OliveOilRecord>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<OliveOilRecord>>;
    fn find_by_grove(
        &self,
        tid: TenantId,
        grove_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<OliveOilRecord>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateOliveOilRecordDto,
        by: Uuid,
    ) -> RepositoryFuture<OliveOilRecord>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateOliveOilRecordDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<OliveOilRecord>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

// --- Water Repository ---
use crate::entities::water::{
    CreateWaterQuotaDto, CreateWaterSourceDto, CreateWaterUsageDto, UpdateWaterQuotaDto,
    UpdateWaterSourceDto, UpdateWaterUsageDto, WaterQuota, WaterSource, WaterUsage,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait WaterSourceRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WaterSource>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WaterSource>>;
    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WaterSource>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateWaterSourceDto,
        by: Uuid,
    ) -> RepositoryFuture<WaterSource>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateWaterSourceDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<WaterSource>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait WaterUsageRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WaterUsage>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WaterUsage>>;
    fn find_by_source(
        &self,
        tid: TenantId,
        source_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WaterUsage>>;
    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WaterUsage>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateWaterUsageDto,
        by: Uuid,
    ) -> RepositoryFuture<WaterUsage>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateWaterUsageDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<WaterUsage>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait WaterQuotaRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WaterQuota>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WaterQuota>>;
    fn find_by_source(
        &self,
        tid: TenantId,
        source_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WaterQuota>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateWaterQuotaDto,
        by: Uuid,
    ) -> RepositoryFuture<WaterQuota>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateWaterQuotaDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<WaterQuota>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

// --- Worker Repository ---
use crate::entities::workforce::{
    CreateWorkLogDto, CreateWorkerDto, CreateWorkerLocationDto, UpdateWorkLogDto, UpdateWorkerDto,
    WorkLog, Worker, WorkerLocation,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait WorkerRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Worker>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<Worker>>;
    fn find_by_user_id(&self, tid: TenantId, user_id: Uuid) -> RepositoryFuture<Option<Worker>>;
    fn find_all(&self, tid: TenantId, p: Pagination)
    -> RepositoryFuture<PaginatedResponse<Worker>>;
    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Worker>>;
    fn create(&self, tid: TenantId, dto: CreateWorkerDto, by: Uuid) -> RepositoryFuture<Worker>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateWorkerDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Worker>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait WorkerLocationRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WorkerLocation>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<WorkerLocation>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WorkerLocation>>;
    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<PaginatedResponse<WorkerLocation>>;
    fn find_latest_by_worker(
        &self,
        tid: TenantId,
        worker_id: Uuid,
    ) -> RepositoryFuture<Option<WorkerLocation>>;
    fn get_latest_locations(&self, tid: TenantId) -> RepositoryFuture<Vec<WorkerLocation>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateWorkerLocationDto,
    ) -> RepositoryFuture<WorkerLocation>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait WorkLogRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WorkLog>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<WorkLog>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WorkLog>>;
    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<PaginatedResponse<WorkLog>>;
    fn find_by_worker(
        &self,
        tid: TenantId,
        worker_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WorkLog>>;
    fn create(&self, tid: TenantId, dto: CreateWorkLogDto, by: Uuid) -> RepositoryFuture<WorkLog>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateWorkLogDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<WorkLog>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

// --- Clock Entry Repository (Arbeitszeiterfassung) ---
use crate::entities::workforce::{ClockEntry, CreateClockEntryDto, UpdateClockEntryDto};

#[cfg_attr(feature = "mocks", automock)]
pub trait ClockEntryRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<ClockEntry>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<ClockEntry>>;
    fn find_by_worker(
        &self,
        tid: TenantId,
        worker_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<ClockEntry>>;
    fn find_active_session(
        &self,
        tid: TenantId,
        worker_id: Uuid,
    ) -> RepositoryFuture<Option<ClockEntry>>;
    fn find_sessions(
        &self,
        tid: TenantId,
        worker_id: Uuid,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> RepositoryFuture<Vec<crate::entities::workforce::ClockSession>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateClockEntryDto,
        by: Uuid,
    ) -> RepositoryFuture<ClockEntry>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateClockEntryDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<ClockEntry>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    fn total_hours_worked(
        &self,
        tid: TenantId,
        worker_id: Uuid,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> RepositoryFuture<f64>;
}

// --- Fertilizer Record Repository ---
use crate::entities::fertilizer::{
    CreateFertilizerRecordDto, FertilizerRecord, UpdateFertilizerRecordDto,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait FertilizerRecordRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<FertilizerRecord>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<FertilizerRecord>>;
    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<FertilizerRecord>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateFertilizerRecordDto,
        by: Uuid,
    ) -> RepositoryFuture<FertilizerRecord>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateFertilizerRecordDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<FertilizerRecord>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

// --- Finance Repository ---
use crate::entities::finance::{
    CostCenter, CreateCostCenterDto, CreateFinancialRecordDto, CreatePACApplicationDto,
    FinancialRecord, PACApplication, UpdateCostCenterDto, UpdateFinancialRecordDto,
    UpdatePACApplicationDto,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait PACApplicationRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<PACApplication>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<PACApplication>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<PACApplication>>;
    fn find_by_year(
        &self,
        tid: TenantId,
        year: i32,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<PACApplication>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreatePACApplicationDto,
        by: Uuid,
    ) -> RepositoryFuture<PACApplication>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdatePACApplicationDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<PACApplication>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait CostCenterRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<CostCenter>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<CostCenter>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<CostCenter>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateCostCenterDto,
        by: Uuid,
    ) -> RepositoryFuture<CostCenter>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateCostCenterDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<CostCenter>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait FinancialRecordRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<FinancialRecord>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<FinancialRecord>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<FinancialRecord>>;
    fn find_by_cost_center(
        &self,
        tid: TenantId,
        cost_center_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<FinancialRecord>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateFinancialRecordDto,
        by: Uuid,
    ) -> RepositoryFuture<FinancialRecord>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateFinancialRecordDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<FinancialRecord>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

// --- Vineyard Repository ---
use crate::entities::vineyard::{
    CreateKelterDeliveryDto, CreateVineyardDto, KelterDelivery, UpdateKelterDeliveryDto,
    UpdateVineyardDto, Vineyard,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait VineyardRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Vineyard>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Vineyard>>;
    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Vineyard>>;
    fn create(&self, tid: TenantId, dto: CreateVineyardDto, by: Uuid)
    -> RepositoryFuture<Vineyard>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateVineyardDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Vineyard>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait KelterDeliveryRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<KelterDelivery>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<KelterDelivery>>;
    fn find_by_vineyard(
        &self,
        tid: TenantId,
        vineyard_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<KelterDelivery>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateKelterDeliveryDto,
        by: Uuid,
    ) -> RepositoryFuture<KelterDelivery>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateKelterDeliveryDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<KelterDelivery>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

// --- Task Data Repository ---
use crate::entities::task::{CreateTaskDataDto, TaskData, UpdateTaskDataDto};

#[cfg_attr(feature = "mocks", automock)]
pub trait TaskDataRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<TaskData>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<TaskData>>;
    fn find_by_task(
        &self,
        tid: TenantId,
        task_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<TaskData>>;
    fn find_by_worker(
        &self,
        tid: TenantId,
        worker_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<TaskData>>;
    fn create(&self, tid: TenantId, dto: CreateTaskDataDto, by: Uuid)
    -> RepositoryFuture<TaskData>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateTaskDataDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<TaskData>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

// --- Inventory Repository ---
use crate::entities::inventory::{
    CreateInventoryItemDto, CreateInventoryLocationDto, CreateInventoryTransactionDto,
    InventoryBalance, InventoryItem, InventoryLocation, InventoryTransaction,
    UpdateInventoryItemDto, UpdateInventoryLocationDto,
};

#[cfg_attr(feature = "mocks", automock)]
pub trait InventoryItemRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<InventoryItem>>;
    fn find_by_sku(&self, tid: TenantId, sku: &str) -> RepositoryFuture<Option<InventoryItem>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<InventoryItem>>;
    fn find_below_minimum(&self, tid: TenantId) -> RepositoryFuture<Vec<InventoryBalance>>;
    fn find_balances(&self, tid: TenantId) -> RepositoryFuture<Vec<InventoryBalance>>;
    fn find_balance_by_item(
        &self,
        tid: TenantId,
        item_id: Uuid,
    ) -> RepositoryFuture<Option<InventoryBalance>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateInventoryItemDto,
        by: Uuid,
    ) -> RepositoryFuture<InventoryItem>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateInventoryItemDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<InventoryItem>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait InventoryTransactionRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid)
    -> RepositoryFuture<Option<InventoryTransaction>>;
    fn find_by_item(
        &self,
        tid: TenantId,
        item_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<InventoryTransaction>>;
    fn find_recent_transactions(
        &self,
        tid: TenantId,
        limit: u32,
    ) -> RepositoryFuture<Vec<InventoryTransaction>>;
    fn create_transaction(
        &self,
        tid: TenantId,
        dto: CreateInventoryTransactionDto,
        by: Uuid,
    ) -> RepositoryFuture<InventoryTransaction>;
    #[allow(clippy::too_many_arguments)]
    fn stock_in(
        &self,
        tid: TenantId,
        item_id: Uuid,
        quantity: f64,
        unit_cost: Option<f64>,
        batch_number: Option<String>,
        expiration_date: Option<String>,
        location: Option<String>,
        notes: Option<String>,
        by: Uuid,
    ) -> RepositoryFuture<InventoryTransaction>;
    fn stock_out(
        &self,
        tid: TenantId,
        item_id: Uuid,
        quantity: f64,
        location: Option<String>,
        notes: Option<String>,
        by: Uuid,
    ) -> RepositoryFuture<Option<InventoryTransaction>>;
    fn transfer(
        &self,
        tid: TenantId,
        item_id: Uuid,
        quantity: f64,
        from_location: &str,
        to_location: &str,
        by: Uuid,
    ) -> RepositoryFuture<Option<InventoryTransaction>>;
    fn adjust(
        &self,
        tid: TenantId,
        item_id: Uuid,
        quantity: f64,
        notes: &str,
        by: Uuid,
    ) -> RepositoryFuture<Option<InventoryTransaction>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait InventoryLocationRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<InventoryLocation>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<InventoryLocation>>;
    fn find_by_code(
        &self,
        tid: TenantId,
        code: &str,
    ) -> RepositoryFuture<Option<InventoryLocation>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateInventoryLocationDto,
        by: Uuid,
    ) -> RepositoryFuture<InventoryLocation>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateInventoryLocationDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<InventoryLocation>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

use crate::entities::{Breed, CreateBreedDto, Species, UpdateBreedDto};
use crate::entities::{Building, CreateBuildingDto, UpdateBuildingDto};
use crate::entities::{CreateGroupDto, Group, UpdateGroupDto};
use crate::entities::{CreateLivestockDto, Livestock, UpdateLivestockDto};
use crate::entities::{CreateTreeDto, Tree, UpdateTreeDto};
use crate::entities::{CreateVarietyDto, UpdateVarietyDto, Variety, VarietyCategory};

#[cfg_attr(feature = "mocks", automock)]
pub trait BuildingRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Building>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Building>>;
    fn find_by_plot(
        &self,
        tid: TenantId,
        plot_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Building>>;
    fn create(&self, tid: TenantId, dto: CreateBuildingDto, by: Uuid)
    -> RepositoryFuture<Building>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateBuildingDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Building>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait GroupRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Group>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Group>>;
    fn find_by_plot(
        &self,
        tid: TenantId,
        plot_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Group>>;
    fn find_children(&self, tid: TenantId, parent_id: Uuid) -> RepositoryFuture<Vec<Group>>;
    fn create(&self, tid: TenantId, dto: CreateGroupDto, by: Uuid) -> RepositoryFuture<Group>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateGroupDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Group>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait TreeRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Tree>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Tree>>;
    fn find_by_plot(
        &self,
        tid: TenantId,
        plot_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Tree>>;
    fn find_by_group(&self, tid: TenantId, group_id: Uuid) -> RepositoryFuture<Vec<Tree>>;
    fn create(&self, tid: TenantId, dto: CreateTreeDto, by: Uuid) -> RepositoryFuture<Tree>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateTreeDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Tree>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait LivestockRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Livestock>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Livestock>>;
    fn find_by_plot(
        &self,
        tid: TenantId,
        plot_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Livestock>>;
    fn find_by_herd(&self, tid: TenantId, herd_id: String) -> RepositoryFuture<Vec<Livestock>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateLivestockDto,
        by: Uuid,
    ) -> RepositoryFuture<Livestock>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateLivestockDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Livestock>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait VarietyRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Variety>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Variety>>;
    fn find_by_category(
        &self,
        tid: TenantId,
        category: VarietyCategory,
    ) -> RepositoryFuture<Vec<Variety>>;
    fn create(&self, tid: TenantId, dto: CreateVarietyDto, by: Uuid) -> RepositoryFuture<Variety>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateVarietyDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Variety>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(feature = "mocks", automock)]
pub trait BreedRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Breed>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Breed>>;
    fn find_by_species(&self, tid: TenantId, species: Species) -> RepositoryFuture<Vec<Breed>>;
    fn create(&self, tid: TenantId, dto: CreateBreedDto, by: Uuid) -> RepositoryFuture<Breed>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateBreedDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Breed>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

// --- Settings Repository ---

use crate::entities::setting::{SettingEntry, SettingValueType, SettingWithDefault, UpdateSetting};

/// Metadata for one known setting key: the key itself, its declared type, the
/// human-readable description and whether its value is a secret.
pub type SettingKeyDescriptor = (String, SettingValueType, Option<String>, bool);

/// Typed key/value settings.
///
/// `tenant_id` is `None` for the system-wide defaults. Reads merge the two so
/// callers get the effective value; writes are always tenant-scoped, so a
/// tenant can override a default but never edit the default itself.
#[cfg_attr(feature = "mocks", automock)]
pub trait SettingsRepository: Send + Sync {
    /// Every effective setting for the tenant: its own overrides, plus the
    /// system defaults for keys it has not overridden.
    fn list_effective(&self, tid: TenantId) -> RepositoryFuture<Vec<SettingWithDefault>>;

    /// A single key, or `None` if neither an override nor a default exists.
    fn get(&self, tid: TenantId, key: &str) -> RepositoryFuture<Option<SettingEntry>>;

    /// Write a tenant override. The default row is left untouched.
    fn set(&self, tid: TenantId, user_id: Uuid, update: UpdateSetting) -> RepositoryFuture<()>;

    /// Write several overrides in one transaction.
    fn set_many(
        &self,
        tid: TenantId,
        user_id: Uuid,
        updates: Vec<UpdateSetting>,
    ) -> RepositoryFuture<()>;

    /// Drop the tenant override, so the key falls back to its default.
    fn reset(&self, tid: TenantId, key: &str) -> RepositoryFuture<bool>;

    /// Write a system-wide default. Separate from `set` because it is not
    /// tenant-scoped and therefore not reachable through the tenant policies.
    fn set_default(
        &self,
        key: &str,
        value: serde_json::Value,
        value_type: SettingValueType,
        description: Option<String>,
        is_sensitive: bool,
    ) -> RepositoryFuture<()>;

    /// Restore the shipped defaults for every key the application knows.
    fn restore_defaults(&self) -> RepositoryFuture<Vec<String>>;

    /// All keys, including those with no value for this tenant. Used by the UI
    /// to render a field for a setting that has never been set.
    fn list_keys(&self) -> RepositoryFuture<Vec<SettingKeyDescriptor>>;
}

pub mod task_sub_task;
pub use task_sub_task::{TaskSubTaskRepository, TaskSubTask, TaskProgressAggregate, NearbySubTask};
