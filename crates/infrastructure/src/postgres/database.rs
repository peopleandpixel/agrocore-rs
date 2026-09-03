use crate::postgres::{
    animal::PgAnimalRepo, audit_log::PgAuditLogRepo, breed::PgBreedRepo, building::PgBuildingRepo,
    clock_entry::PgClockEntryRepo, cold_chain_log::PgColdChainLogRepo,
    compliance::PgComplianceChecklistRepo, cost_center::PgCostCenterRepo, customer::PgCustomerRepo,
    equipment::PgEquipmentRepo, fertilizer_record::PgFertilizerRecordRepo,
    financial_record::PgFinancialRecordRepo, frost_warning::PgFrostWarningRepo, group::PgGroupRepo,
    growing_degree_day::PgGrowingDegreeDayRepo, harvest_delivery::PgHarvestDeliveryRepo,
    harvest_lot::PgHarvestLotRepo, harvest_season::PgHarvestSeasonRepo,
    inventory_item::PgInventoryItemRepo, inventory_location::PgInventoryLocationRepo,
    inventory_transaction::PgInventoryTransactionRepo, kelter_delivery::PgKelterDeliveryRepo,
    livestock::PgLivestockRepo, olive_grove::PgOliveGroveRepo,
    olive_oil_record::PgOliveOilRecordRepo, order::PgOrderRepo,
    pac_application::PgPACApplicationRepo, pest_risk::PgPestRiskRepo,
    phenology_record::PgPhenologyRecordRepo, plant_protection_record::PgPlantProtectionRecordRepo,
    site::PgSiteRepo, soil_moisture_config::PgSoilMoistureConfigRepo, task_data::PgTaskDataRepo,
    tenant::PgTenantRepo, tree::PgTreeRepo, user::PgUserRepo, variety::PgVarietyRepo,
    vineyard::PgVineyardRepo, weather_data::PgWeatherDataRepo,
    weather_station::PgWeatherStationRepo, work_log::PgWorkLogRepo, worker::PgWorkerRepo,
    worker_location::PgWorkerLocationRepo, worker_task_status::PgWorkerTaskStatusRepo,
};
use agrocore_domain::repositories::{
    AnimalRepository, AuditLogRepo, BreedRepository, BuildingRepository, ClockEntryRepo,
    ColdChainLogRepo, ComplianceChecklistRepo, CostCenterRepo, CustomerRepository,
    EquipmentRepository, FertilizerRecordRepo, FinancialRecordRepo, FrostWarningRepo,
    GroupRepository, GrowingDegreeDayRepo, HarvestDeliveryRepo, HarvestLotRepo, HarvestSeasonRepo,
    InventoryItemRepository, InventoryLocationRepo, InventoryTransactionRepo, KelterDeliveryRepo,
    LivestockRepository, OliveGroveRepo, OliveOilRecordRepo, OrderRepository, PACApplicationRepo,
    PestRiskRepo, PhenologyRecordRepo, PlantProtectionRecordRepo, SiteRepository,
    SoilMoistureConfigRepo, SpatialObjectRepository, TaskDataRepository, TenantRepository,
    TreeRepository, UserRepository, VarietyRepository, VineyardRepo, WaterQuotaRepo,
    WaterSourceRepo, WaterUsageRepo, WeatherDataRepo, WeatherStationRepo, WorkLogRepo,
    WorkerLocationRepo, WorkerRepo, WorkerTaskStatusRepository,
};
use agrocore_logging::{debug, info};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Database {
    Postgres(PostgresDb),
    #[cfg(feature = "mocks")]
    Mock(Box<MockDatabase>),
}

#[cfg(feature = "mocks")]
#[derive(Clone, Default)]
pub struct MockDatabase {
    pub site_repo: Option<Arc<agrocore_domain::repositories::MockSiteRepository>>,
    pub user_repo: Option<Arc<agrocore_domain::repositories::MockUserRepository>>,
    pub order_repo: Option<Arc<agrocore_domain::repositories::MockOrderRepository>>,
    pub tenant_repo: Option<Arc<agrocore_domain::repositories::MockTenantRepository>>,
    pub equipment_repo: Option<Arc<agrocore_domain::repositories::MockEquipmentRepository>>,
    pub animal_repo: Option<Arc<agrocore_domain::repositories::MockAnimalRepository>>,
    pub worker_repo: Option<Arc<agrocore_domain::repositories::MockWorkerRepo>>,
    pub worker_location_repo: Option<Arc<agrocore_domain::repositories::MockWorkerLocationRepo>>,
    pub work_log_repo: Option<Arc<agrocore_domain::repositories::MockWorkLogRepo>>,
    pub worker_task_status_repo:
        Option<Arc<agrocore_domain::repositories::MockWorkerTaskStatusRepository>>,
    pub task_data_repo: Option<Arc<agrocore_domain::repositories::MockTaskDataRepository>>,
    pub weather_station_repo: Option<Arc<agrocore_domain::repositories::MockWeatherStationRepo>>,
    pub weather_data_repo: Option<Arc<agrocore_domain::repositories::MockWeatherDataRepo>>,
    pub fertilizer_record_repo:
        Option<Arc<agrocore_domain::repositories::MockFertilizerRecordRepo>>,
    pub plant_protection_record_repo:
        Option<Arc<agrocore_domain::repositories::MockPlantProtectionRecordRepo>>,
    pub harvest_season_repo: Option<Arc<agrocore_domain::repositories::MockHarvestSeasonRepo>>,
    pub harvest_lot_repo: Option<Arc<agrocore_domain::repositories::MockHarvestLotRepo>>,
    pub harvest_delivery_repo: Option<Arc<agrocore_domain::repositories::MockHarvestDeliveryRepo>>,
    pub olive_grove_repo: Option<Arc<agrocore_domain::repositories::MockOliveGroveRepo>>,
    pub olive_oil_record_repo: Option<Arc<agrocore_domain::repositories::MockOliveOilRecordRepo>>,
    pub vineyard_repo: Option<Arc<agrocore_domain::repositories::MockVineyardRepo>>,
    pub water_source_repo: Option<Arc<agrocore_domain::repositories::MockWaterSourceRepo>>,
    pub water_usage_repo: Option<Arc<agrocore_domain::repositories::MockWaterUsageRepo>>,
    pub water_quota_repo: Option<Arc<agrocore_domain::repositories::MockWaterQuotaRepo>>,
    pub spatial_object_repo:
        Option<Arc<agrocore_domain::repositories::MockSpatialObjectRepository>>,
    pub phenology_record_repo: Option<Arc<agrocore_domain::repositories::MockPhenologyRecordRepo>>,
    pub pac_application_repo: Option<Arc<agrocore_domain::repositories::MockPACApplicationRepo>>,
    pub cold_chain_log_repo: Option<Arc<agrocore_domain::repositories::MockColdChainLogRepo>>,
    pub audit_log_repo: Option<Arc<agrocore_domain::repositories::MockAuditLogRepo>>,
    pub compliance_checklist_repo:
        Option<Arc<agrocore_domain::repositories::MockComplianceChecklistRepo>>,
    pub cost_center_repo: Option<Arc<agrocore_domain::repositories::MockCostCenterRepo>>,
    pub customer_repo: Option<Arc<agrocore_domain::repositories::MockCustomerRepository>>,
    pub financial_record_repo: Option<Arc<agrocore_domain::repositories::MockFinancialRecordRepo>>,
    pub kelter_delivery_repo: Option<Arc<agrocore_domain::repositories::MockKelterDeliveryRepo>>,
    pub inventory_item_repo:
        Option<Arc<agrocore_domain::repositories::MockInventoryItemRepository>>,
    pub inventory_transaction_repo:
        Option<Arc<agrocore_domain::repositories::MockInventoryTransactionRepo>>,
    pub inventory_location_repo:
        Option<Arc<agrocore_domain::repositories::MockInventoryLocationRepo>>,
    pub clock_entry_repo: Option<Arc<agrocore_domain::repositories::MockClockEntryRepo>>,
    pub frost_warning_repo: Option<Arc<agrocore_domain::repositories::MockFrostWarningRepo>>,
    pub growing_degree_day_repo:
        Option<Arc<agrocore_domain::repositories::MockGrowingDegreeDayRepo>>,
    pub pest_risk_repo: Option<Arc<agrocore_domain::repositories::MockPestRiskRepo>>,
    pub soil_moisture_config_repo:
        Option<Arc<agrocore_domain::repositories::MockSoilMoistureConfigRepo>>,
    pub building_repo: Option<Arc<agrocore_domain::repositories::MockBuildingRepository>>,
    pub group_repo: Option<Arc<agrocore_domain::repositories::MockGroupRepository>>,
    pub tree_repo: Option<Arc<agrocore_domain::repositories::MockTreeRepository>>,
    pub livestock_repo: Option<Arc<agrocore_domain::repositories::MockLivestockRepository>>,
    pub variety_repo: Option<Arc<agrocore_domain::repositories::MockVarietyRepository>>,
    pub breed_repo: Option<Arc<agrocore_domain::repositories::MockBreedRepository>>,
}

impl Database {
    pub fn pool(&self) -> &PgPool {
        match self {
            Self::Postgres(db) => db.pool(),
            #[cfg(feature = "mocks")]
            Self::Mock(_) => panic!("MockDatabase has no pool"),
        }
    }

    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let db = PostgresDb::connect(database_url).await?;
        Ok(Self::Postgres(db))
    }

    pub fn site_repo(&self) -> Arc<dyn SiteRepository> {
        match self {
            Self::Postgres(db) => db.site_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => {
                m.site_repo.clone().expect("site_repo mock not set") as Arc<dyn SiteRepository>
            }
        }
    }

    pub fn user_repo(&self) -> Arc<dyn UserRepository> {
        match self {
            Self::Postgres(db) => db.user_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => {
                m.user_repo.clone().expect("user_repo mock not set") as Arc<dyn UserRepository>
            }
        }
    }

    pub fn order_repo(&self) -> Arc<dyn OrderRepository> {
        match self {
            Self::Postgres(db) => db.order_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => {
                m.order_repo.clone().expect("order_repo mock not set") as Arc<dyn OrderRepository>
            }
        }
    }

    pub fn tenant_repo(&self) -> Arc<dyn TenantRepository> {
        match self {
            Self::Postgres(db) => db.tenant_repo.clone(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m.tenant_repo.clone().expect("tenant_repo mock not set")
                as Arc<dyn TenantRepository>,
        }
    }

    pub fn equipment_repo(&self) -> Arc<dyn EquipmentRepository> {
        match self {
            Self::Postgres(db) => db.equipment_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .equipment_repo
                .clone()
                .expect("equipment_repo mock not set")
                as Arc<dyn EquipmentRepository>,
        }
    }

    pub fn animal_repo(&self) -> Arc<dyn AnimalRepository> {
        match self {
            Self::Postgres(db) => db.animal_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m.animal_repo.clone().expect("animal_repo mock not set")
                as Arc<dyn AnimalRepository>,
        }
    }

    pub fn weather_station_repo(&self) -> Arc<dyn WeatherStationRepo> {
        match self {
            Self::Postgres(db) => db.weather_station_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .weather_station_repo
                .clone()
                .expect("weather_station_repo mock not set")
                as Arc<dyn WeatherStationRepo>,
        }
    }

    pub fn weather_data_repo(&self) -> Arc<dyn WeatherDataRepo> {
        match self {
            Self::Postgres(db) => db.weather_data_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .weather_data_repo
                .clone()
                .expect("weather_data_repo mock not set")
                as Arc<dyn WeatherDataRepo>,
        }
    }

    pub fn fertilizer_record_repo(&self) -> Arc<dyn FertilizerRecordRepo> {
        match self {
            Self::Postgres(db) => db.fertilizer_record_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .fertilizer_record_repo
                .clone()
                .expect("fertilizer_record_repo mock not set")
                as Arc<dyn FertilizerRecordRepo>,
        }
    }

    pub fn plant_protection_record_repo(&self) -> Arc<dyn PlantProtectionRecordRepo> {
        match self {
            Self::Postgres(db) => db.plant_protection_record_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .plant_protection_record_repo
                .clone()
                .expect("plant_protection_record_repo mock not set")
                as Arc<dyn PlantProtectionRecordRepo>,
        }
    }

    pub fn harvest_season_repo(&self) -> Arc<dyn HarvestSeasonRepo> {
        match self {
            Self::Postgres(db) => db.harvest_season_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .harvest_season_repo
                .clone()
                .expect("harvest_season_repo mock not set")
                as Arc<dyn HarvestSeasonRepo>,
        }
    }

    pub fn harvest_lot_repo(&self) -> Arc<dyn HarvestLotRepo> {
        match self {
            Self::Postgres(db) => db.harvest_lot_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .harvest_lot_repo
                .clone()
                .expect("harvest_lot_repo mock not set")
                as Arc<dyn HarvestLotRepo>,
        }
    }

    pub fn harvest_delivery_repo(&self) -> Arc<dyn HarvestDeliveryRepo> {
        match self {
            Self::Postgres(db) => db.harvest_delivery_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .harvest_delivery_repo
                .clone()
                .expect("harvest_delivery_repo mock not set")
                as Arc<dyn HarvestDeliveryRepo>,
        }
    }

    pub fn olive_grove_repo(&self) -> Arc<dyn OliveGroveRepo> {
        match self {
            Self::Postgres(db) => db.olive_grove_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .olive_grove_repo
                .clone()
                .expect("olive_grove_repo mock not set")
                as Arc<dyn OliveGroveRepo>,
        }
    }

    pub fn olive_oil_record_repo(&self) -> Arc<dyn OliveOilRecordRepo> {
        match self {
            Self::Postgres(db) => db.olive_oil_record_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .olive_oil_record_repo
                .clone()
                .expect("olive_oil_record_repo mock not set")
                as Arc<dyn OliveOilRecordRepo>,
        }
    }

    pub fn vineyard_repo(&self) -> Arc<dyn VineyardRepo> {
        match self {
            Self::Postgres(db) => db.vineyard_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m.vineyard_repo.clone().expect("vineyard_repo mock not set")
                as Arc<dyn VineyardRepo>,
        }
    }

    pub fn water_source_repo(&self) -> Arc<dyn WaterSourceRepo> {
        match self {
            Self::Postgres(db) => db.water_source_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .water_source_repo
                .clone()
                .expect("water_source_repo mock not set")
                as Arc<dyn WaterSourceRepo>,
        }
    }

    pub fn water_usage_repo(&self) -> Arc<dyn WaterUsageRepo> {
        match self {
            Self::Postgres(db) => db.water_usage_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .water_usage_repo
                .clone()
                .expect("water_usage_repo mock not set")
                as Arc<dyn WaterUsageRepo>,
        }
    }

    pub fn water_quota_repo(&self) -> Arc<dyn WaterQuotaRepo> {
        match self {
            Self::Postgres(db) => db.water_quota_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .water_quota_repo
                .clone()
                .expect("water_quota_repo mock not set")
                as Arc<dyn WaterQuotaRepo>,
        }
    }

    pub fn worker_repo(&self) -> Arc<dyn WorkerRepo> {
        match self {
            Self::Postgres(db) => db.worker_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => {
                m.worker_repo.clone().expect("worker_repo mock not set") as Arc<dyn WorkerRepo>
            }
        }
    }

    pub fn worker_location_repo(&self) -> Arc<dyn WorkerLocationRepo> {
        match self {
            Self::Postgres(db) => db.worker_location_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .worker_location_repo
                .clone()
                .expect("worker_location_repo mock not set")
                as Arc<dyn WorkerLocationRepo>,
        }
    }

    pub fn work_log_repo(&self) -> Arc<dyn WorkLogRepo> {
        match self {
            Self::Postgres(db) => db.work_log_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => {
                m.work_log_repo.clone().expect("work_log_repo mock not set") as Arc<dyn WorkLogRepo>
            }
        }
    }

    pub fn worker_task_status_repo(&self) -> Arc<dyn WorkerTaskStatusRepository> {
        match self {
            Self::Postgres(db) => db.worker_task_status_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .worker_task_status_repo
                .clone()
                .expect("worker_task_status_repo mock not set")
                as Arc<dyn WorkerTaskStatusRepository>,
        }
    }

    pub fn task_data_repo(&self) -> Arc<dyn TaskDataRepository> {
        match self {
            Self::Postgres(db) => db.task_data_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .task_data_repo
                .clone()
                .expect("task_data_repo mock not set")
                as Arc<dyn TaskDataRepository>,
        }
    }

    pub fn spatial_object_repo(&self) -> Arc<dyn SpatialObjectRepository> {
        match self {
            Self::Postgres(db) => db.spatial_object_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .spatial_object_repo
                .clone()
                .expect("spatial_object_repo mock not set")
                as Arc<dyn SpatialObjectRepository>,
        }
    }

    pub fn phenology_record_repo(&self) -> Arc<dyn PhenologyRecordRepo> {
        match self {
            Self::Postgres(db) => db.phenology_record_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .phenology_record_repo
                .clone()
                .expect("phenology_record_repo mock not set")
                as Arc<dyn PhenologyRecordRepo>,
        }
    }

    pub fn pac_application_repo(&self) -> Arc<dyn PACApplicationRepo> {
        match self {
            Self::Postgres(db) => db.pac_application_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .pac_application_repo
                .clone()
                .expect("pac_application_repo mock not set")
                as Arc<dyn PACApplicationRepo>,
        }
    }

    pub fn cold_chain_log_repo(&self) -> Arc<dyn ColdChainLogRepo> {
        match self {
            Self::Postgres(db) => db.cold_chain_log_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .cold_chain_log_repo
                .clone()
                .expect("cold_chain_log_repo mock not set")
                as Arc<dyn ColdChainLogRepo>,
        }
    }

    pub fn audit_log_repo(&self) -> Arc<dyn AuditLogRepo> {
        match self {
            Self::Postgres(db) => db.audit_log_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => {
                m.audit_log_repo
                    .clone()
                    .expect("audit_log_repo mock not set") as Arc<dyn AuditLogRepo>
            }
        }
    }

    pub fn compliance_checklist_repo(&self) -> Arc<dyn ComplianceChecklistRepo> {
        match self {
            Self::Postgres(db) => db.compliance_checklist_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .compliance_checklist_repo
                .clone()
                .expect("compliance_checklist_repo mock not set")
                as Arc<dyn ComplianceChecklistRepo>,
        }
    }

    pub fn cost_center_repo(&self) -> Arc<dyn CostCenterRepo> {
        match self {
            Self::Postgres(db) => db.cost_center_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .cost_center_repo
                .clone()
                .expect("cost_center_repo mock not set")
                as Arc<dyn CostCenterRepo>,
        }
    }

    pub fn customer_repo(&self) -> Arc<dyn CustomerRepository> {
        match self {
            Self::Postgres(db) => db.customer_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m.customer_repo.clone().expect("customer_repo mock not set")
                as Arc<dyn CustomerRepository>,
        }
    }

    pub fn financial_record_repo(&self) -> Arc<dyn FinancialRecordRepo> {
        match self {
            Self::Postgres(db) => db.financial_record_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .financial_record_repo
                .clone()
                .expect("financial_record_repo mock not set")
                as Arc<dyn FinancialRecordRepo>,
        }
    }

    pub fn kelter_delivery_repo(&self) -> Arc<dyn KelterDeliveryRepo> {
        match self {
            Self::Postgres(db) => db.kelter_delivery_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .kelter_delivery_repo
                .clone()
                .expect("kelter_delivery_repo mock not set")
                as Arc<dyn KelterDeliveryRepo>,
        }
    }

    pub fn inventory_item_repo(&self) -> Arc<dyn InventoryItemRepository> {
        match self {
            Self::Postgres(db) => db.inventory_item_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .inventory_item_repo
                .clone()
                .expect("inventory_item_repo mock not set")
                as Arc<dyn InventoryItemRepository>,
        }
    }

    pub fn inventory_transaction_repo(&self) -> Arc<dyn InventoryTransactionRepo> {
        match self {
            Self::Postgres(db) => db.inventory_transaction_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .inventory_transaction_repo
                .clone()
                .expect("inventory_transaction_repo mock not set")
                as Arc<dyn InventoryTransactionRepo>,
        }
    }

    pub fn inventory_location_repo(&self) -> Arc<dyn InventoryLocationRepo> {
        match self {
            Self::Postgres(db) => db.inventory_location_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .inventory_location_repo
                .clone()
                .expect("inventory_location_repo mock not set")
                as Arc<dyn InventoryLocationRepo>,
        }
    }

    pub fn clock_entry_repo(&self) -> Arc<dyn ClockEntryRepo> {
        match self {
            Self::Postgres(db) => db.clock_entry_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .clock_entry_repo
                .clone()
                .expect("clock_entry_repo mock not set")
                as Arc<dyn ClockEntryRepo>,
        }
    }

    pub fn frost_warning_repo(&self) -> Arc<dyn FrostWarningRepo> {
        match self {
            Self::Postgres(db) => db.frost_warning_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .frost_warning_repo
                .clone()
                .expect("frost_warning_repo mock not set")
                as Arc<dyn FrostWarningRepo>,
        }
    }

    pub fn growing_degree_day_repo(&self) -> Arc<dyn GrowingDegreeDayRepo> {
        match self {
            Self::Postgres(db) => db.growing_degree_day_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .growing_degree_day_repo
                .clone()
                .expect("growing_degree_day_repo mock not set")
                as Arc<dyn GrowingDegreeDayRepo>,
        }
    }

    pub fn pest_risk_repo(&self) -> Arc<dyn PestRiskRepo> {
        match self {
            Self::Postgres(db) => db.pest_risk_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => {
                m.pest_risk_repo
                    .clone()
                    .expect("pest_risk_repo mock not set") as Arc<dyn PestRiskRepo>
            }
        }
    }

    pub fn soil_moisture_config_repo(&self) -> Arc<dyn SoilMoistureConfigRepo> {
        match self {
            Self::Postgres(db) => db.soil_moisture_config_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .soil_moisture_config_repo
                .clone()
                .expect("soil_moisture_config_repo mock not set")
                as Arc<dyn SoilMoistureConfigRepo>,
        }
    }

    pub fn building_repo(&self) -> Arc<dyn BuildingRepository> {
        match self {
            Self::Postgres(db) => db.building_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m.building_repo.clone().expect("building_repo mock not set")
                as Arc<dyn BuildingRepository>,
        }
    }

    pub fn group_repo(&self) -> Arc<dyn GroupRepository> {
        match self {
            Self::Postgres(db) => db.group_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => {
                m.group_repo.clone().expect("group_repo mock not set") as Arc<dyn GroupRepository>
            }
        }
    }

    pub fn tree_repo(&self) -> Arc<dyn TreeRepository> {
        match self {
            Self::Postgres(db) => db.tree_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => {
                m.tree_repo.clone().expect("tree_repo mock not set") as Arc<dyn TreeRepository>
            }
        }
    }

    pub fn livestock_repo(&self) -> Arc<dyn LivestockRepository> {
        match self {
            Self::Postgres(db) => db.livestock_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m
                .livestock_repo
                .clone()
                .expect("livestock_repo mock not set")
                as Arc<dyn LivestockRepository>,
        }
    }

    pub fn variety_repo(&self) -> Arc<dyn VarietyRepository> {
        match self {
            Self::Postgres(db) => db.variety_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => m.variety_repo.clone().expect("variety_repo mock not set")
                as Arc<dyn VarietyRepository>,
        }
    }

    pub fn breed_repo(&self) -> Arc<dyn BreedRepository> {
        match self {
            Self::Postgres(db) => db.breed_repo(),
            #[cfg(feature = "mocks")]
            Self::Mock(m) => {
                m.breed_repo.clone().expect("breed_repo mock not set") as Arc<dyn BreedRepository>
            }
        }
    }
}

/// PostgreSQL Database Wrapper
///
/// Repositories are pre-instantiated during `connect()` and cached as `Arc`
/// clones. Since all PostgreSQL repositories are stateless (they only hold a
/// `PgPool` reference), we can clone the `Arc` instead of allocating a new
/// `Arc::new(Repo::new(pool))` on every method call. This eliminates redundant
/// heap allocations on every repository access.
#[derive(Clone)]
pub struct PostgresDb {
    pub pool: PgPool,
    /// Pre-instantiated repository Arcs — cloned on access instead of re-created.
    pub site_repo: Arc<dyn SiteRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub order_repo: Arc<dyn OrderRepository>,
    pub tenant_repo: Arc<dyn TenantRepository>,
    pub equipment_repo: Arc<dyn EquipmentRepository>,
    pub animal_repo: Arc<dyn AnimalRepository>,
    pub weather_station_repo: Arc<dyn WeatherStationRepo>,
    pub weather_data_repo: Arc<dyn WeatherDataRepo>,
    pub fertilizer_record_repo: Arc<dyn FertilizerRecordRepo>,
    pub plant_protection_record_repo: Arc<dyn PlantProtectionRecordRepo>,
    pub harvest_season_repo: Arc<dyn HarvestSeasonRepo>,
    pub harvest_lot_repo: Arc<dyn HarvestLotRepo>,
    pub harvest_delivery_repo: Arc<dyn HarvestDeliveryRepo>,
    pub olive_grove_repo: Arc<dyn OliveGroveRepo>,
    pub olive_oil_record_repo: Arc<dyn OliveOilRecordRepo>,
    pub vineyard_repo: Arc<dyn VineyardRepo>,
    pub water_source_repo: Arc<dyn WaterSourceRepo>,
    pub water_usage_repo: Arc<dyn WaterUsageRepo>,
    pub water_quota_repo: Arc<dyn WaterQuotaRepo>,
    pub worker_repo: Arc<dyn WorkerRepo>,
    pub worker_location_repo: Arc<dyn WorkerLocationRepo>,
    pub work_log_repo: Arc<dyn WorkLogRepo>,
    pub worker_task_status_repo: Arc<dyn WorkerTaskStatusRepository>,
    pub task_data_repo: Arc<dyn TaskDataRepository>,
    pub spatial_object_repo: Arc<dyn SpatialObjectRepository>,
    pub phenology_record_repo: Arc<dyn PhenologyRecordRepo>,
    pub pac_application_repo: Arc<dyn PACApplicationRepo>,
    pub cold_chain_log_repo: Arc<dyn ColdChainLogRepo>,
    pub audit_log_repo: Arc<dyn AuditLogRepo>,
    pub compliance_checklist_repo: Arc<dyn ComplianceChecklistRepo>,
    pub cost_center_repo: Arc<dyn CostCenterRepo>,
    pub customer_repo: Arc<dyn CustomerRepository>,
    pub financial_record_repo: Arc<dyn FinancialRecordRepo>,
    pub kelter_delivery_repo: Arc<dyn KelterDeliveryRepo>,
    pub inventory_item_repo: Arc<dyn InventoryItemRepository>,
    pub inventory_transaction_repo: Arc<dyn InventoryTransactionRepo>,
    pub inventory_location_repo: Arc<dyn InventoryLocationRepo>,
    pub clock_entry_repo: Arc<dyn ClockEntryRepo>,
    pub frost_warning_repo: Arc<dyn FrostWarningRepo>,
    pub growing_degree_day_repo: Arc<dyn GrowingDegreeDayRepo>,
    pub pest_risk_repo: Arc<dyn PestRiskRepo>,
    pub soil_moisture_config_repo: Arc<dyn SoilMoistureConfigRepo>,
    pub building_repo: Arc<dyn BuildingRepository>,
    pub group_repo: Arc<dyn GroupRepository>,
    pub tree_repo: Arc<dyn TreeRepository>,
    pub livestock_repo: Arc<dyn LivestockRepository>,
    pub variety_repo: Arc<dyn VarietyRepository>,
    pub breed_repo: Arc<dyn BreedRepository>,
}

impl PostgresDb {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let config = agrocore_shared::config::AgroCoreConfig::global();
        let pool_options = config.pg_pool_options();
        let connect_timeout_secs = config.database_connect_timeout_secs;

        // Append connect_timeout to the database URL as a query parameter
        let database_url_with_timeout = if database_url.contains('?') {
            format!("{}&connect_timeout={}", database_url, connect_timeout_secs)
        } else {
            format!("{}?connect_timeout={}", database_url, connect_timeout_secs)
        };

        let pool = agrocore_shared::with_retry("connect to database", 10, 1, || {
            let opts = pool_options.clone();
            let url = database_url_with_timeout.clone();
            Box::pin(async move {
                opts.connect(&url)
                    .await
                    .map_err(|e| anyhow::anyhow!("database connection error: {}", e))
            })
        })
        .await?;

        // Initialize metrics timer (only if monitoring enabled via env)
        if std::env::var("AGROCORE_METRICS_ENABLED")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
        {
            let pool_clone = pool.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    // Pool metrics would be updated here via sqlx PoolStats
                    debug!("Pool metrics tick");
                }
            });
        }

        // Monthly depreciation automation timer (OPT-Abschreibung)
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(30 * 24 * 60 * 60)); // ~monthly
            loop {
                interval.tick().await;
                info!("Monthly depreciation automation tick");
                // Here: call depreciation calculation for all active equipment
            }
        });

        sqlx::migrate!("../../migrations").run(&pool).await?;

        // Pre-instantiate all repositories once — Arc::clone is cheap (refcount increment)
        // compared to Arc::new(Repo::new(pool.clone())) which allocates on every call.
        Ok(Self {
            site_repo: Arc::new(PgSiteRepo::new(pool.clone())),
            user_repo: Arc::new(PgUserRepo::new(pool.clone())),
            order_repo: Arc::new(PgOrderRepo::new(pool.clone())),
            tenant_repo: Arc::new(PgTenantRepo::new(pool.clone())),
            equipment_repo: Arc::new(PgEquipmentRepo::new(pool.clone())),
            animal_repo: Arc::new(PgAnimalRepo::new(pool.clone())),
            weather_station_repo: Arc::new(PgWeatherStationRepo::new(pool.clone())),
            weather_data_repo: Arc::new(PgWeatherDataRepo::new(pool.clone())),
            fertilizer_record_repo: Arc::new(PgFertilizerRecordRepo::new(pool.clone())),
            plant_protection_record_repo: Arc::new(PgPlantProtectionRecordRepo::new(pool.clone())),
            harvest_season_repo: Arc::new(PgHarvestSeasonRepo::new(pool.clone())),
            harvest_lot_repo: Arc::new(PgHarvestLotRepo::new(pool.clone())),
            harvest_delivery_repo: Arc::new(PgHarvestDeliveryRepo::new(pool.clone())),
            olive_grove_repo: Arc::new(PgOliveGroveRepo::new(pool.clone())),
            olive_oil_record_repo: Arc::new(PgOliveOilRecordRepo::new(pool.clone())),
            vineyard_repo: Arc::new(PgVineyardRepo::new(pool.clone())),
            water_source_repo: Arc::new(crate::postgres::water_source::PgWaterSourceRepo::new(
                pool.clone(),
            )),
            water_usage_repo: Arc::new(crate::postgres::water_usage::PgWaterUsageRepo::new(
                pool.clone(),
            )),
            water_quota_repo: Arc::new(crate::postgres::water_quota::PgWaterQuotaRepo::new(
                pool.clone(),
            )),
            worker_repo: Arc::new(PgWorkerRepo::new(pool.clone())),
            worker_location_repo: Arc::new(PgWorkerLocationRepo::new(pool.clone())),
            work_log_repo: Arc::new(PgWorkLogRepo::new(pool.clone())),
            worker_task_status_repo: Arc::new(PgWorkerTaskStatusRepo::new(pool.clone())),
            task_data_repo: Arc::new(PgTaskDataRepo::new(pool.clone())),
            spatial_object_repo: Arc::new(PgSiteRepo::new(pool.clone())),
            phenology_record_repo: Arc::new(PgPhenologyRecordRepo::new(pool.clone())),
            pac_application_repo: Arc::new(
                crate::postgres::pac_application::PgPACApplicationRepo::new(pool.clone()),
            ),
            cold_chain_log_repo: Arc::new(PgColdChainLogRepo::new(pool.clone())),
            audit_log_repo: Arc::new(PgAuditLogRepo::new(pool.clone())),
            cost_center_repo: Arc::new(PgCostCenterRepo::new(pool.clone())),
            customer_repo: Arc::new(PgCustomerRepo::new(pool.clone())),
            financial_record_repo: Arc::new(PgFinancialRecordRepo::new(pool.clone())),
            compliance_checklist_repo: Arc::new(PgComplianceChecklistRepo::new(pool.clone())),
            kelter_delivery_repo: Arc::new(PgKelterDeliveryRepo::new(pool.clone())),
            inventory_item_repo: Arc::new(PgInventoryItemRepo::new(pool.clone())),
            inventory_transaction_repo: Arc::new(PgInventoryTransactionRepo::new(pool.clone())),
            inventory_location_repo: Arc::new(PgInventoryLocationRepo::new(pool.clone())),
            clock_entry_repo: Arc::new(PgClockEntryRepo::new(pool.clone())),
            frost_warning_repo: Arc::new(crate::postgres::frost_warning::PgFrostWarningRepo::new(
                pool.clone(),
            )),
            growing_degree_day_repo: Arc::new(
                crate::postgres::growing_degree_day::PgGrowingDegreeDayRepo::new(pool.clone()),
            ),
            pest_risk_repo: Arc::new(crate::postgres::pest_risk::PgPestRiskRepo::new(
                pool.clone(),
            )),
            soil_moisture_config_repo: Arc::new(
                crate::postgres::soil_moisture_config::PgSoilMoistureConfigRepo::new(pool.clone()),
            ),
            building_repo: Arc::new(PgBuildingRepo::new(pool.clone())),
            group_repo: Arc::new(PgGroupRepo::new(pool.clone())),
            tree_repo: Arc::new(PgTreeRepo::new(pool.clone())),
            livestock_repo: Arc::new(PgLivestockRepo::new(pool.clone())),
            variety_repo: Arc::new(PgVarietyRepo::new(pool.clone())),
            breed_repo: Arc::new(PgBreedRepo::new(pool.clone())),
            pool,
        })
    }

    /// Creates a PostgresDb from an already-established pool (used by tests).
    /// Pre-instantiates all repositories just like `connect()`.
    pub fn from_pool(pool: PgPool) -> Self {
        Self {
            site_repo: Arc::new(PgSiteRepo::new(pool.clone())),
            user_repo: Arc::new(PgUserRepo::new(pool.clone())),
            order_repo: Arc::new(PgOrderRepo::new(pool.clone())),
            tenant_repo: Arc::new(PgTenantRepo::new(pool.clone())),
            equipment_repo: Arc::new(PgEquipmentRepo::new(pool.clone())),
            animal_repo: Arc::new(PgAnimalRepo::new(pool.clone())),
            weather_station_repo: Arc::new(PgWeatherStationRepo::new(pool.clone())),
            weather_data_repo: Arc::new(PgWeatherDataRepo::new(pool.clone())),
            fertilizer_record_repo: Arc::new(PgFertilizerRecordRepo::new(pool.clone())),
            plant_protection_record_repo: Arc::new(PgPlantProtectionRecordRepo::new(pool.clone())),
            harvest_season_repo: Arc::new(PgHarvestSeasonRepo::new(pool.clone())),
            harvest_lot_repo: Arc::new(PgHarvestLotRepo::new(pool.clone())),
            harvest_delivery_repo: Arc::new(PgHarvestDeliveryRepo::new(pool.clone())),
            olive_grove_repo: Arc::new(PgOliveGroveRepo::new(pool.clone())),
            olive_oil_record_repo: Arc::new(PgOliveOilRecordRepo::new(pool.clone())),
            vineyard_repo: Arc::new(PgVineyardRepo::new(pool.clone())),
            water_source_repo: Arc::new(crate::postgres::water_source::PgWaterSourceRepo::new(
                pool.clone(),
            )),
            water_usage_repo: Arc::new(crate::postgres::water_usage::PgWaterUsageRepo::new(
                pool.clone(),
            )),
            water_quota_repo: Arc::new(crate::postgres::water_quota::PgWaterQuotaRepo::new(
                pool.clone(),
            )),
            worker_repo: Arc::new(PgWorkerRepo::new(pool.clone())),
            worker_location_repo: Arc::new(PgWorkerLocationRepo::new(pool.clone())),
            work_log_repo: Arc::new(PgWorkLogRepo::new(pool.clone())),
            worker_task_status_repo: Arc::new(PgWorkerTaskStatusRepo::new(pool.clone())),
            task_data_repo: Arc::new(PgTaskDataRepo::new(pool.clone())),
            spatial_object_repo: Arc::new(PgSiteRepo::new(pool.clone())),
            phenology_record_repo: Arc::new(PgPhenologyRecordRepo::new(pool.clone())),
            pac_application_repo: Arc::new(
                crate::postgres::pac_application::PgPACApplicationRepo::new(pool.clone()),
            ),
            cold_chain_log_repo: Arc::new(PgColdChainLogRepo::new(pool.clone())),
            audit_log_repo: Arc::new(PgAuditLogRepo::new(pool.clone())),
            cost_center_repo: Arc::new(PgCostCenterRepo::new(pool.clone())),
            customer_repo: Arc::new(PgCustomerRepo::new(pool.clone())),
            financial_record_repo: Arc::new(PgFinancialRecordRepo::new(pool.clone())),
            compliance_checklist_repo: Arc::new(PgComplianceChecklistRepo::new(pool.clone())),
            kelter_delivery_repo: Arc::new(PgKelterDeliveryRepo::new(pool.clone())),
            inventory_item_repo: Arc::new(PgInventoryItemRepo::new(pool.clone())),
            inventory_transaction_repo: Arc::new(PgInventoryTransactionRepo::new(pool.clone())),
            inventory_location_repo: Arc::new(PgInventoryLocationRepo::new(pool.clone())),
            clock_entry_repo: Arc::new(PgClockEntryRepo::new(pool.clone())),
            frost_warning_repo: Arc::new(crate::postgres::frost_warning::PgFrostWarningRepo::new(
                pool.clone(),
            )),
            growing_degree_day_repo: Arc::new(
                crate::postgres::growing_degree_day::PgGrowingDegreeDayRepo::new(pool.clone()),
            ),
            pest_risk_repo: Arc::new(crate::postgres::pest_risk::PgPestRiskRepo::new(
                pool.clone(),
            )),
            soil_moisture_config_repo: Arc::new(
                crate::postgres::soil_moisture_config::PgSoilMoistureConfigRepo::new(pool.clone()),
            ),
            building_repo: Arc::new(PgBuildingRepo::new(pool.clone())),
            group_repo: Arc::new(PgGroupRepo::new(pool.clone())),
            tree_repo: Arc::new(PgTreeRepo::new(pool.clone())),
            livestock_repo: Arc::new(PgLivestockRepo::new(pool.clone())),
            variety_repo: Arc::new(PgVarietyRepo::new(pool.clone())),
            breed_repo: Arc::new(PgBreedRepo::new(pool.clone())),

            pool,
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn map_db_error(e: sqlx::Error) -> agrocore_shared::SharedError {
        crate::postgres::error_mapper::map_db_error(e)
    }

    // Core repositories — clone the pre-instantiated Arc instead of creating new instances
    pub fn site_repo(&self) -> Arc<dyn SiteRepository> {
        self.site_repo.clone()
    }

    pub fn user_repo(&self) -> Arc<dyn UserRepository> {
        self.user_repo.clone()
    }

    pub fn order_repo(&self) -> Arc<dyn OrderRepository> {
        self.order_repo.clone()
    }

    pub fn tenant_repo(&self) -> Arc<dyn TenantRepository> {
        self.tenant_repo.clone()
    }

    pub fn equipment_repo(&self) -> Arc<dyn EquipmentRepository> {
        self.equipment_repo.clone()
    }

    pub fn animal_repo(&self) -> Arc<dyn AnimalRepository> {
        self.animal_repo.clone()
    }

    pub fn weather_station_repo(&self) -> Arc<dyn WeatherStationRepo> {
        self.weather_station_repo.clone()
    }

    pub fn weather_data_repo(&self) -> Arc<dyn WeatherDataRepo> {
        self.weather_data_repo.clone()
    }

    pub fn fertilizer_record_repo(&self) -> Arc<dyn FertilizerRecordRepo> {
        self.fertilizer_record_repo.clone()
    }

    pub fn plant_protection_record_repo(&self) -> Arc<dyn PlantProtectionRecordRepo> {
        self.plant_protection_record_repo.clone()
    }

    pub fn harvest_season_repo(&self) -> Arc<dyn HarvestSeasonRepo> {
        self.harvest_season_repo.clone()
    }

    pub fn harvest_lot_repo(&self) -> Arc<dyn HarvestLotRepo> {
        self.harvest_lot_repo.clone()
    }

    pub fn harvest_delivery_repo(&self) -> Arc<dyn HarvestDeliveryRepo> {
        self.harvest_delivery_repo.clone()
    }

    pub fn olive_grove_repo(&self) -> Arc<dyn OliveGroveRepo> {
        self.olive_grove_repo.clone()
    }

    pub fn olive_oil_record_repo(&self) -> Arc<dyn OliveOilRecordRepo> {
        self.olive_oil_record_repo.clone()
    }

    pub fn vineyard_repo(&self) -> Arc<dyn VineyardRepo> {
        self.vineyard_repo.clone()
    }

    pub fn water_source_repo(&self) -> Arc<dyn WaterSourceRepo> {
        self.water_source_repo.clone()
    }

    pub fn water_usage_repo(&self) -> Arc<dyn WaterUsageRepo> {
        self.water_usage_repo.clone()
    }

    pub fn water_quota_repo(&self) -> Arc<dyn WaterQuotaRepo> {
        self.water_quota_repo.clone()
    }

    pub fn worker_repo(&self) -> Arc<dyn WorkerRepo> {
        self.worker_repo.clone()
    }

    pub fn worker_location_repo(&self) -> Arc<dyn WorkerLocationRepo> {
        self.worker_location_repo.clone()
    }

    pub fn work_log_repo(&self) -> Arc<dyn WorkLogRepo> {
        self.work_log_repo.clone()
    }

    pub fn worker_task_status_repo(&self) -> Arc<dyn WorkerTaskStatusRepository> {
        self.worker_task_status_repo.clone()
    }

    pub fn task_data_repo(&self) -> Arc<dyn TaskDataRepository> {
        self.task_data_repo.clone()
    }

    pub fn spatial_object_repo(&self) -> Arc<dyn SpatialObjectRepository> {
        self.spatial_object_repo.clone()
    }

    pub fn phenology_record_repo(&self) -> Arc<dyn PhenologyRecordRepo> {
        self.phenology_record_repo.clone()
    }

    pub fn pac_application_repo(&self) -> Arc<dyn PACApplicationRepo> {
        self.pac_application_repo.clone()
    }

    pub fn cold_chain_log_repo(&self) -> Arc<dyn ColdChainLogRepo> {
        self.cold_chain_log_repo.clone()
    }

    pub fn audit_log_repo(&self) -> Arc<dyn AuditLogRepo> {
        self.audit_log_repo.clone()
    }

    pub fn cost_center_repo(&self) -> Arc<dyn CostCenterRepo> {
        self.cost_center_repo.clone()
    }

    pub fn customer_repo(&self) -> Arc<dyn CustomerRepository> {
        self.customer_repo.clone()
    }

    pub fn financial_record_repo(&self) -> Arc<dyn FinancialRecordRepo> {
        self.financial_record_repo.clone()
    }

    pub fn compliance_checklist_repo(&self) -> Arc<dyn ComplianceChecklistRepo> {
        self.compliance_checklist_repo.clone()
    }

    pub fn kelter_delivery_repo(&self) -> Arc<dyn KelterDeliveryRepo> {
        self.kelter_delivery_repo.clone()
    }

    pub fn inventory_item_repo(&self) -> Arc<dyn InventoryItemRepository> {
        self.inventory_item_repo.clone()
    }

    pub fn inventory_transaction_repo(&self) -> Arc<dyn InventoryTransactionRepo> {
        self.inventory_transaction_repo.clone()
    }

    pub fn inventory_location_repo(&self) -> Arc<dyn InventoryLocationRepo> {
        self.inventory_location_repo.clone()
    }

    pub fn clock_entry_repo(&self) -> Arc<dyn ClockEntryRepo> {
        self.clock_entry_repo.clone()
    }

    pub fn frost_warning_repo(&self) -> Arc<dyn FrostWarningRepo> {
        self.frost_warning_repo.clone()
    }

    pub fn growing_degree_day_repo(&self) -> Arc<dyn GrowingDegreeDayRepo> {
        self.growing_degree_day_repo.clone()
    }

    pub fn pest_risk_repo(&self) -> Arc<dyn PestRiskRepo> {
        self.pest_risk_repo.clone()
    }

    pub fn soil_moisture_config_repo(&self) -> Arc<dyn SoilMoistureConfigRepo> {
        self.soil_moisture_config_repo.clone()
    }

    pub fn building_repo(&self) -> Arc<dyn BuildingRepository> {
        self.building_repo.clone()
    }

    pub fn group_repo(&self) -> Arc<dyn GroupRepository> {
        self.group_repo.clone()
    }

    pub fn tree_repo(&self) -> Arc<dyn TreeRepository> {
        self.tree_repo.clone()
    }

    pub fn livestock_repo(&self) -> Arc<dyn LivestockRepository> {
        self.livestock_repo.clone()
    }

    pub fn variety_repo(&self) -> Arc<dyn VarietyRepository> {
        self.variety_repo.clone()
    }

    pub fn breed_repo(&self) -> Arc<dyn BreedRepository> {
        self.breed_repo.clone()
    }
}
