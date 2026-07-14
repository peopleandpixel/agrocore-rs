use crate::entities::tenant::TenantId;
use agrocore_shared::{PaginatedResponse, Pagination, Result};
use serde::Serialize;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

#[cfg(test)]
use mockall::automock;

pub type RepositoryFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

use crate::entities::user::{CreateUserDto, LoginDto, UpdateUserDto, User};
use crate::entities::order::{CreateOrderDto, MyTask, Order, UpdateOrderDto};
use crate::entities::tenant::{CreateTenantDto, Tenant, UpdateTenantDto};
use crate::entities::equipment::{CreateEquipmentDto, Equipment, UpdateEquipmentDto};
use crate::entities::site::{CreateSiteDto, Site, UpdateSiteDto};
use crate::entities::spatial::SpatialObject;
use crate::entities::livestock::{Animal, CreateAnimalDto, UpdateAnimalDto, TreatmentRecord, GrazingRecord};
use crate::entities::weather::{WeatherStation, CreateWeatherStationDto, WeatherData, CreateWeatherDataDto, PhenologyRecord, CreatePhenologyRecordDto};
use crate::entities::plant_protection::{PlantProtectionRecord, CreatePlantProtectionDto};
use crate::entities::harvest::{HarvestSeason, CreateHarvestSeasonDto, HarvestLot, CreateHarvestLotDto, HarvestDelivery, CreateHarvestDeliveryDto, UpdateHarvestDeliveryDto, UpdateHarvestLotDto};
use crate::entities::olive::{OliveGrove, CreateOliveGroveDto, UpdateOliveGroveDto, OliveOilRecord, CreateOliveOilRecordDto, UpdateOliveOilRecordDto};
use crate::entities::vineyard::{Vineyard, CreateVineyardDto, UpdateVineyardDto};
use crate::entities::water::{WaterSource, CreateWaterSourceDto, WaterUsage, CreateWaterUsageDto, UpdateWaterUsageDto, WaterQuota};
use crate::entities::workforce::{Worker, CreateWorkerDto, UpdateWorkerDto, WorkLog, CreateWorkLogDto, UpdateWorkLogDto, WorkerLocation, CreateWorkerLocationDto};
use crate::entities::finance::{PACApplication, CreatePACApplicationDto, CostCenter, CreateCostCenterDto, FinancialRecord, CreateFinancialRecordDto};
use crate::entities::coldchain::ColdChainLog;
use crate::entities::compliance::FertilizerRecord;

#[cfg_attr(test, automock)]
pub trait UserRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<User>>;
    fn find_by_id_visible(&self, tid: TenantId, id: Uuid, user_id: Uuid, roles: &[crate::entities::user::UserRole]) -> RepositoryFuture<Option<User>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<User>>;
    fn find_all_visible(&self, tid: TenantId, p: Pagination, user_id: Uuid, roles: &[crate::entities::user::UserRole]) -> RepositoryFuture<PaginatedResponse<User>>;
    fn find_by_email(&self, email: &str) -> RepositoryFuture<Option<User>>;
    fn create(&self, tid: TenantId, dto: CreateUserDto, by: Uuid) -> RepositoryFuture<User>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateUserDto, by: Uuid) -> RepositoryFuture<Option<User>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    fn authenticate(&self, dto: LoginDto) -> RepositoryFuture<crate::entities::user::AuthResponse>;
    fn find_by_refresh_token(&self, refresh_token: &str) -> RepositoryFuture<Option<User>>;
    fn invalidate_refresh_token(&self, user_id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait OrderRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Order>>;
    fn find_by_id_visible(&self, tid: TenantId, id: Uuid, user_id: Uuid, roles: &[crate::entities::user::UserRole]) -> RepositoryFuture<Option<Order>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Order>>;
    fn find_all_visible(&self, tid: TenantId, p: Pagination, user_id: Uuid, roles: &[crate::entities::user::UserRole]) -> RepositoryFuture<PaginatedResponse<Order>>;
    fn create(&self, tid: TenantId, dto: CreateOrderDto, by: Uuid) -> RepositoryFuture<Order>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateOrderDto, by: Uuid) -> RepositoryFuture<Option<Order>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    fn find_my_tasks(&self, tid: TenantId, worker_id: Uuid) -> RepositoryFuture<Vec<MyTask>>;
    fn find_assigned_to_worker(&self, tid: TenantId, worker_id: Uuid) -> RepositoryFuture<Vec<Order>>;
}

#[cfg_attr(test, automock)]
pub trait TenantRepository: Send + Sync {
    fn find_by_id(&self, id: Uuid) -> RepositoryFuture<Option<Tenant>>;
    fn create(&self, dto: CreateTenantDto) -> RepositoryFuture<Tenant>;
    fn update(&self, id: Uuid, dto: UpdateTenantDto) -> RepositoryFuture<Option<Tenant>>;
    fn delete(&self, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait EquipmentRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Equipment>>;
    fn find_by_id_visible(&self, tid: TenantId, id: Uuid, user_id: Uuid, roles: &[crate::entities::user::UserRole]) -> RepositoryFuture<Option<Equipment>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Equipment>>;
    fn find_all_visible(&self, tid: TenantId, p: Pagination, user_id: Uuid, roles: &[crate::entities::user::UserRole]) -> RepositoryFuture<PaginatedResponse<Equipment>>;
    fn create(&self, tid: TenantId, dto: CreateEquipmentDto, by: Uuid) -> RepositoryFuture<Equipment>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateEquipmentDto, by: Uuid) -> RepositoryFuture<Option<Equipment>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait SiteRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Site>>;
    fn find_by_id_visible(&self, tid: TenantId, id: Uuid, user_id: Uuid, roles: &[crate::entities::user::UserRole]) -> RepositoryFuture<Option<Site>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Site>>;
    fn find_all_visible(&self, tid: TenantId, p: Pagination, user_id: Uuid, roles: &[crate::entities::user::UserRole]) -> RepositoryFuture<PaginatedResponse<Site>>;
    fn create(&self, tid: TenantId, dto: CreateSiteDto, by: Uuid) -> RepositoryFuture<Site>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateSiteDto, by: Uuid) -> RepositoryFuture<Option<Site>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait AnimalRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Animal>>;
    fn find_by_id_visible(&self, tid: TenantId, id: Uuid, user_id: Uuid, roles: &[crate::entities::user::UserRole]) -> RepositoryFuture<Option<Animal>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Animal>>;
    fn create(&self, tid: TenantId, dto: CreateAnimalDto, by: Uuid) -> RepositoryFuture<Animal>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateAnimalDto, by: Uuid) -> RepositoryFuture<Option<Animal>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    fn add_treatment(&self, tid: TenantId, id: Uuid, treatment: TreatmentRecord) -> RepositoryFuture<bool>;
    fn add_grazing_record(&self, tid: TenantId, id: Uuid, record: GrazingRecord) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait WeatherStationRepo: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WeatherStation>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WeatherStation>>;
    fn create(&self, tid: TenantId, dto: CreateWeatherStationDto) -> RepositoryFuture<WeatherStation>;
    fn update(&self, tid: TenantId, id: Uuid, _dto: serde_json::Value) -> RepositoryFuture<Option<WeatherStation>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait WeatherDataRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<WeatherData>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WeatherData>>;
    fn create(&self, tid: TenantId, dto: CreateWeatherDataDto) -> RepositoryFuture<WeatherData>;
}

#[cfg_attr(test, automock)]
pub trait PhenologyRecordRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<PhenologyRecord>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<PhenologyRecord>>;
    fn create(&self, tid: TenantId, dto: CreatePhenologyRecordDto) -> RepositoryFuture<PhenologyRecord>;
}

#[cfg_attr(test, automock)]
pub trait PlantProtectionRecordRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<PlantProtectionRecord>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<PlantProtectionRecord>>;
    fn create(&self, tid: TenantId, dto: CreatePlantProtectionDto) -> RepositoryFuture<PlantProtectionRecord>;
    fn update(&self, tid: TenantId, id: Uuid, _dto: serde_json::Value) -> RepositoryFuture<Option<PlantProtectionRecord>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait FertilizerRecordRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<FertilizerRecord>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<FertilizerRecord>>;
    fn create(&self, tid: TenantId, dto: crate::entities::compliance::CreateFertilizerRecordDto) -> RepositoryFuture<FertilizerRecord>;
}

#[cfg_attr(test, automock)]
pub trait HarvestSeasonRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<HarvestSeason>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<HarvestSeason>>;
    fn create(&self, tid: TenantId, dto: CreateHarvestSeasonDto) -> RepositoryFuture<HarvestSeason>;
}

#[cfg_attr(test, automock)]
pub trait HarvestLotRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<HarvestLot>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<HarvestLot>>;
    fn create(&self, tid: TenantId, dto: CreateHarvestLotDto) -> RepositoryFuture<HarvestLot>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateHarvestLotDto) -> RepositoryFuture<Option<HarvestLot>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait HarvestDeliveryRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<HarvestDelivery>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<HarvestDelivery>>;
    fn create(&self, tid: TenantId, dto: CreateHarvestDeliveryDto) -> RepositoryFuture<HarvestDelivery>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateHarvestDeliveryDto) -> RepositoryFuture<Option<HarvestDelivery>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait OliveGroveRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<OliveGrove>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<OliveGrove>>;
    fn create(&self, tid: TenantId, dto: CreateOliveGroveDto) -> RepositoryFuture<OliveGrove>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateOliveGroveDto) -> RepositoryFuture<Option<OliveGrove>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait OliveOilRecordRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<OliveOilRecord>>;
    fn create(&self, tid: TenantId, dto: CreateOliveOilRecordDto) -> RepositoryFuture<OliveOilRecord>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateOliveOilRecordDto) -> RepositoryFuture<Option<OliveOilRecord>>;
}

#[cfg_attr(test, automock)]
pub trait VineyardRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<Vineyard>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Vineyard>>;
    fn create(&self, tid: TenantId, dto: CreateVineyardDto) -> RepositoryFuture<Vineyard>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateVineyardDto) -> RepositoryFuture<Option<Vineyard>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait WaterSourceRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<WaterSource>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WaterSource>>;
    fn create(&self, tid: TenantId, dto: CreateWaterSourceDto) -> RepositoryFuture<WaterSource>;
}

#[cfg_attr(test, automock)]
pub trait WaterUsageRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<WaterUsage>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WaterUsage>>;
    fn create(&self, tid: TenantId, dto: CreateWaterUsageDto) -> RepositoryFuture<WaterUsage>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateWaterUsageDto) -> RepositoryFuture<Option<WaterUsage>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait WaterQuotaRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<WaterQuota>>;
    fn create(&self, tid: TenantId, _dto: serde_json::Value) -> RepositoryFuture<WaterQuota>;
}

#[cfg_attr(test, automock)]
pub trait WorkerRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<Worker>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Worker>>;
    fn create(&self, tid: TenantId, dto: CreateWorkerDto) -> RepositoryFuture<Worker>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateWorkerDto) -> RepositoryFuture<Option<Worker>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait WorkLogRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<WorkLog>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WorkLog>>;
    fn create(&self, tid: TenantId, dto: CreateWorkLogDto) -> RepositoryFuture<WorkLog>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateWorkLogDto) -> RepositoryFuture<Option<WorkLog>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait WorkerLocationRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<WorkerLocation>>;
    fn create(&self, tid: TenantId, dto: CreateWorkerLocationDto) -> RepositoryFuture<WorkerLocation>;
}

#[cfg_attr(test, automock)]
pub trait PACApplicationRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<PACApplication>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<PACApplication>>;
    fn create(&self, tid: TenantId, dto: CreatePACApplicationDto) -> RepositoryFuture<PACApplication>;
}

#[cfg_attr(test, automock)]
pub trait CostCenterRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<CostCenter>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<CostCenter>>;
    fn create(&self, tid: TenantId, dto: CreateCostCenterDto) -> RepositoryFuture<CostCenter>;
}

#[cfg_attr(test, automock)]
pub trait FinancialRecordRepo: Send + Sync {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<FinancialRecord>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<FinancialRecord>>;
    fn create(&self, tid: TenantId, dto: CreateFinancialRecordDto) -> RepositoryFuture<FinancialRecord>;
}

#[cfg_attr(test, automock)]
pub trait ColdChainLogRepo: Send + Sync {
    fn create(&self, tid: TenantId, _dto: serde_json::Value) -> RepositoryFuture<ColdChainLog>;
}