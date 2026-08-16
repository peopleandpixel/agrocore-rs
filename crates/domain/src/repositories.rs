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

use crate::entities::equipment::{CreateEquipmentDto, Equipment, UpdateEquipmentDto};
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
        point: crate::entities::site::GeoPoint,
        site_id: Option<Uuid>,
    ) -> RepositoryFuture<Vec<SpatialObject>>;
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
    fn find_by_refresh_token(&self, refresh_token: &str) -> RepositoryFuture<Option<User>>;
    fn invalidate_refresh_token(&self, user_id: Uuid) -> RepositoryFuture<bool>;
    fn update_refresh_token(
        &self,
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
    CreatePhenologyRecordDto, CreateWeatherDataDto, CreateWeatherStationDto, PhenologyRecord,
    UpdatePhenologyRecordDto, UpdateWeatherDataDto, UpdateWeatherStationDto, WeatherData,
    WeatherStation,
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

// --- Harvest Repository ---
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
