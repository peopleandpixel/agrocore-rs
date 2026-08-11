use crate::postgres::{
    animal::PgAnimalRepo, audit_log::PgAuditLogRepo, cold_chain_log::PgColdChainLogRepo,
    compliance::PgComplianceChecklistRepo, cost_center::PgCostCenterRepo,
    equipment::PgEquipmentRepo, fertilizer_record::PgFertilizerRecordRepo,
    financial_record::PgFinancialRecordRepo, harvest_delivery::PgHarvestDeliveryRepo,
    harvest_lot::PgHarvestLotRepo, harvest_season::PgHarvestSeasonRepo,
    kelter_delivery::PgKelterDeliveryRepo, olive_grove::PgOliveGroveRepo,
    olive_oil_record::PgOliveOilRecordRepo, order::PgOrderRepo,
    phenology_record::PgPhenologyRecordRepo, plant_protection_record::PgPlantProtectionRecordRepo,
    site::PgSiteRepo, task_data::PgTaskDataRepo, tenant::PgTenantRepo, user::PgUserRepo,
    vineyard::PgVineyardRepo, weather_data::PgWeatherDataRepo,
    weather_station::PgWeatherStationRepo, work_log::PgWorkLogRepo, worker::PgWorkerRepo,
    worker_location::PgWorkerLocationRepo, worker_task_status::PgWorkerTaskStatusRepo,
};
use agrocore_domain::repositories::{
    AnimalRepository, AuditLogRepo, ColdChainLogRepo, ComplianceChecklistRepo, CostCenterRepo,
    EquipmentRepository, FertilizerRecordRepo, FinancialRecordRepo, HarvestDeliveryRepo,
    HarvestLotRepo, HarvestSeasonRepo, KelterDeliveryRepo, OliveGroveRepo, OliveOilRecordRepo,
    OrderRepository, PACApplicationRepo, PhenologyRecordRepo, PlantProtectionRecordRepo,
    SiteRepository, SpatialObjectRepository, TaskDataRepository, TenantRepository, UserRepository,
    VineyardRepo, WaterQuotaRepo, WaterSourceRepo, WaterUsageRepo, WeatherDataRepo,
    WeatherStationRepo, WorkLogRepo, WorkerLocationRepo, WorkerRepo, WorkerTaskStatusRepository,
};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub enum Database {
    Postgres(PostgresDb),
    #[cfg(any(test, feature = "mocks"))]
    Mock(Box<MockDatabase>),
}

#[cfg(any(test, feature = "mocks"))]
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
    pub financial_record_repo: Option<Arc<agrocore_domain::repositories::MockFinancialRecordRepo>>,
    pub kelter_delivery_repo: Option<Arc<agrocore_domain::repositories::MockKelterDeliveryRepo>>,
}

impl Database {
    pub fn pool(&self) -> &PgPool {
        match self {
            Self::Postgres(db) => db.pool(),
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(m) => {
                m.site_repo.clone().expect("site_repo mock not set") as Arc<dyn SiteRepository>
            }
        }
    }

    pub fn user_repo(&self) -> Arc<dyn UserRepository> {
        match self {
            Self::Postgres(db) => db.user_repo(),
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(m) => {
                m.user_repo.clone().expect("user_repo mock not set") as Arc<dyn UserRepository>
            }
        }
    }

    pub fn order_repo(&self) -> Arc<dyn OrderRepository> {
        match self {
            Self::Postgres(db) => db.order_repo(),
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(m) => {
                m.order_repo.clone().expect("order_repo mock not set") as Arc<dyn OrderRepository>
            }
        }
    }

    pub fn tenant_repo(&self) -> Arc<dyn TenantRepository> {
        match self {
            Self::Postgres(db) => Arc::new(PgTenantRepo::new(db.pool.clone())),
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(m) => m.tenant_repo.clone().expect("tenant_repo mock not set")
                as Arc<dyn TenantRepository>,
        }
    }

    pub fn equipment_repo(&self) -> Arc<dyn EquipmentRepository> {
        match self {
            Self::Postgres(db) => db.equipment_repo(),
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(m) => m.animal_repo.clone().expect("animal_repo mock not set")
                as Arc<dyn AnimalRepository>,
        }
    }

    pub fn weather_station_repo(&self) -> Arc<dyn WeatherStationRepo> {
        match self {
            Self::Postgres(db) => db.weather_station_repo(),
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(m) => m.vineyard_repo.clone().expect("vineyard_repo mock not set")
                as Arc<dyn VineyardRepo>,
        }
    }

    pub fn water_source_repo(&self) -> Arc<dyn WaterSourceRepo> {
        match self {
            Self::Postgres(db) => db.water_source_repo(),
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(m) => {
                m.worker_repo.clone().expect("worker_repo mock not set") as Arc<dyn WorkerRepo>
            }
        }
    }

    pub fn worker_location_repo(&self) -> Arc<dyn WorkerLocationRepo> {
        match self {
            Self::Postgres(db) => db.worker_location_repo(),
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(m) => {
                m.work_log_repo.clone().expect("work_log_repo mock not set") as Arc<dyn WorkLogRepo>
            }
        }
    }

    pub fn worker_task_status_repo(&self) -> Arc<dyn WorkerTaskStatusRepository> {
        match self {
            Self::Postgres(db) => db.worker_task_status_repo(),
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(m) => m
                .cost_center_repo
                .clone()
                .expect("cost_center_repo mock not set")
                as Arc<dyn CostCenterRepo>,
        }
    }

    pub fn financial_record_repo(&self) -> Arc<dyn FinancialRecordRepo> {
        match self {
            Self::Postgres(db) => db.financial_record_repo(),
            #[cfg(any(test, feature = "mocks"))]
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
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(m) => m
                .kelter_delivery_repo
                .clone()
                .expect("kelter_delivery_repo mock not set")
                as Arc<dyn KelterDeliveryRepo>,
        }
    }
}

/// PostgreSQL Database Wrapper
#[derive(Clone)]
pub struct PostgresDb {
    pub pool: PgPool,
}

impl PostgresDb {
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let mut retry_count = 0;
        let max_retries = 10;
        let pool = loop {
            match PgPool::connect(database_url).await {
                Ok(pool) => break pool,
                Err(e) if retry_count < max_retries => {
                    retry_count += 1;
                    tracing::warn!(
                        "Failed to connect to database (attempt {}/{}): {}. Retrying in 1s...",
                        retry_count,
                        max_retries,
                        e
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to connect to database after {} attempts: {}",
                        max_retries,
                        e
                    ));
                }
            }
        };
        sqlx::migrate!("../../migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    // Core repositories
    pub fn site_repo(&self) -> Arc<dyn SiteRepository> {
        Arc::new(PgSiteRepo::new(self.pool.clone()))
    }

    pub fn map_db_error(e: sqlx::Error) -> agrocore_shared::SharedError {
        crate::postgres::error_mapper::map_db_error(e)
    }

    pub fn user_repo(&self) -> Arc<dyn UserRepository> {
        Arc::new(PgUserRepo::new(self.pool.clone()))
    }

    pub fn order_repo(&self) -> Arc<dyn OrderRepository> {
        Arc::new(PgOrderRepo::new(self.pool.clone()))
    }

    pub fn tenant_repo(&self) -> Arc<dyn TenantRepository> {
        Arc::new(PgTenantRepo::new(self.pool.clone()))
    }

    pub fn equipment_repo(&self) -> Arc<dyn EquipmentRepository> {
        Arc::new(PgEquipmentRepo::new(self.pool.clone()))
    }

    pub fn animal_repo(&self) -> Arc<dyn AnimalRepository> {
        Arc::new(PgAnimalRepo::new(self.pool.clone()))
    }

    // Weather repositories
    pub fn weather_station_repo(&self) -> Arc<dyn WeatherStationRepo> {
        Arc::new(PgWeatherStationRepo::new(self.pool.clone()))
    }

    pub fn weather_data_repo(&self) -> Arc<dyn WeatherDataRepo> {
        Arc::new(PgWeatherDataRepo::new(self.pool.clone()))
    }

    // Agricultural records
    pub fn fertilizer_record_repo(&self) -> Arc<dyn FertilizerRecordRepo> {
        Arc::new(PgFertilizerRecordRepo::new(self.pool.clone()))
    }

    pub fn plant_protection_record_repo(&self) -> Arc<dyn PlantProtectionRecordRepo> {
        Arc::new(PgPlantProtectionRecordRepo::new(self.pool.clone()))
    }

    pub fn harvest_season_repo(&self) -> Arc<dyn HarvestSeasonRepo> {
        Arc::new(PgHarvestSeasonRepo::new(self.pool.clone()))
    }

    pub fn harvest_lot_repo(&self) -> Arc<dyn HarvestLotRepo> {
        Arc::new(PgHarvestLotRepo::new(self.pool.clone()))
    }

    pub fn harvest_delivery_repo(&self) -> Arc<dyn HarvestDeliveryRepo> {
        Arc::new(PgHarvestDeliveryRepo::new(self.pool.clone()))
    }

    // Olive repositories
    pub fn olive_grove_repo(&self) -> Arc<dyn OliveGroveRepo> {
        Arc::new(PgOliveGroveRepo::new(self.pool.clone()))
    }

    pub fn olive_oil_record_repo(&self) -> Arc<dyn OliveOilRecordRepo> {
        Arc::new(PgOliveOilRecordRepo::new(self.pool.clone()))
    }

    // Vineyard repository
    pub fn vineyard_repo(&self) -> Arc<dyn VineyardRepo> {
        Arc::new(PgVineyardRepo::new(self.pool.clone()))
    }

    // Water repositories
    pub fn water_source_repo(&self) -> Arc<dyn WaterSourceRepo> {
        Arc::new(crate::postgres::water_source::PgWaterSourceRepo::new(
            self.pool.clone(),
        ))
    }

    pub fn water_usage_repo(&self) -> Arc<dyn WaterUsageRepo> {
        Arc::new(crate::postgres::water_usage::PgWaterUsageRepo::new(
            self.pool.clone(),
        ))
    }

    pub fn water_quota_repo(&self) -> Arc<dyn WaterQuotaRepo> {
        Arc::new(crate::postgres::water_quota::PgWaterQuotaRepo::new(
            self.pool.clone(),
        ))
    }

    // Worker repositories
    pub fn worker_repo(&self) -> Arc<dyn WorkerRepo> {
        Arc::new(PgWorkerRepo::new(self.pool.clone()))
    }

    pub fn worker_location_repo(&self) -> Arc<dyn WorkerLocationRepo> {
        Arc::new(PgWorkerLocationRepo::new(self.pool.clone()))
    }

    pub fn work_log_repo(&self) -> Arc<dyn WorkLogRepo> {
        Arc::new(PgWorkLogRepo::new(self.pool.clone()))
    }

    pub fn worker_task_status_repo(&self) -> Arc<dyn WorkerTaskStatusRepository> {
        Arc::new(PgWorkerTaskStatusRepo::new(self.pool.clone()))
    }

    pub fn task_data_repo(&self) -> Arc<dyn TaskDataRepository> {
        Arc::new(PgTaskDataRepo::new(self.pool.clone()))
    }

    pub fn spatial_object_repo(&self) -> Arc<dyn SpatialObjectRepository> {
        Arc::new(PgSiteRepo::new(self.pool.clone()))
    }

    // Phenology
    pub fn phenology_record_repo(&self) -> Arc<dyn PhenologyRecordRepo> {
        Arc::new(PgPhenologyRecordRepo::new(self.pool.clone()))
    }

    // PAC
    pub fn pac_application_repo(&self) -> Arc<dyn PACApplicationRepo> {
        Arc::new(crate::postgres::pac_application::PgPACApplicationRepo::new(
            self.pool.clone(),
        ))
    }

    // ColdChain
    pub fn cold_chain_log_repo(&self) -> Arc<dyn ColdChainLogRepo> {
        Arc::new(PgColdChainLogRepo::new(self.pool.clone()))
    }

    pub fn audit_log_repo(&self) -> Arc<dyn AuditLogRepo> {
        Arc::new(PgAuditLogRepo::new(self.pool.clone()))
    }

    pub fn cost_center_repo(&self) -> Arc<dyn CostCenterRepo> {
        Arc::new(PgCostCenterRepo::new(self.pool.clone()))
    }

    pub fn financial_record_repo(&self) -> Arc<dyn FinancialRecordRepo> {
        Arc::new(PgFinancialRecordRepo::new(self.pool.clone()))
    }

    pub fn compliance_checklist_repo(&self) -> Arc<dyn ComplianceChecklistRepo> {
        Arc::new(PgComplianceChecklistRepo::new(self.pool.clone()))
    }

    pub fn kelter_delivery_repo(&self) -> Arc<dyn KelterDeliveryRepo> {
        Arc::new(PgKelterDeliveryRepo::new(self.pool.clone()))
    }
}
