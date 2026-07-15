use crate::postgres::{
    animal::PgAnimalRepo, cold_chain_log::PgColdChainLogRepo, equipment::PgEquipmentRepo,
    fertilizer_record::PgFertilizerRecordRepo, harvest_delivery::PgHarvestDeliveryRepo,
    harvest_lot::PgHarvestLotRepo, harvest_season::PgHarvestSeasonRepo,
    olive_grove::PgOliveGroveRepo, olive_oil_record::PgOliveOilRecordRepo, order::PgOrderRepo,
    phenology_record::PgPhenologyRecordRepo, plant_protection_record::PgPlantProtectionRecordRepo,
    site::PgSiteRepo, tenant::PgTenantRepo, user::PgUserRepo, audit_log::PgAuditLogRepo,
    task_data::PgTaskDataRepo,
    compliance::PgComplianceChecklistRepo, cost_center::PgCostCenterRepo, financial_record::PgFinancialRecordRepo,
    vineyard::PgVineyardRepo, weather_data::PgWeatherDataRepo, weather_station::PgWeatherStationRepo,
    worker::PgWorkerRepo, worker_location::PgWorkerLocationRepo, work_log::PgWorkLogRepo,
    worker_task_status::PgWorkerTaskStatusRepo, kelter_delivery::PgKelterDeliveryRepo,
};
use agrocore_domain::repositories::{
    AnimalRepository, EquipmentRepository, FertilizerRecordRepo, HarvestDeliveryRepo, HarvestLotRepo, HarvestSeasonRepo,
    OliveGroveRepo, OliveOilRecordRepo, OrderRepository, PhenologyRecordRepo, PlantProtectionRecordRepo,
    SiteRepository, TenantRepository, UserRepository, VineyardRepo,
    WaterQuotaRepo, WaterSourceRepo, WaterUsageRepo,
    WeatherDataRepo, WeatherStationRepo, WorkerLocationRepo, WorkerRepo, WorkLogRepo,
    WorkerTaskStatusRepository, TaskDataRepository, SpatialObjectRepository,
    PACApplicationRepo, ColdChainLogRepo, AuditLogRepo, ComplianceChecklistRepo,
    CostCenterRepo, FinancialRecordRepo, KelterDeliveryRepo,
};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub enum Database {
    Postgres(PostgresDb),
}

impl Database {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let db = PostgresDb::connect(database_url).await?;
        Ok(Self::Postgres(db))
    }

    pub fn site_repo(&self) -> Arc<dyn SiteRepository> {
        match self {
            Self::Postgres(db) => db.site_repo(),
        }
    }

    pub fn user_repo(&self) -> Arc<dyn UserRepository> {
        match self {
            Self::Postgres(db) => db.user_repo(),
        }
    }

    pub fn order_repo(&self) -> Arc<dyn OrderRepository> {
        match self {
            Self::Postgres(db) => db.order_repo(),
        }
    }

    pub fn tenant_repo(&self) -> Arc<dyn TenantRepository> {
        match self {
            Self::Postgres(db) => db.tenant_repo(),
        }
    }

    pub fn equipment_repo(&self) -> Arc<dyn EquipmentRepository> {
        match self {
            Self::Postgres(db) => db.equipment_repo(),
        }
    }

    pub fn animal_repo(&self) -> Arc<dyn AnimalRepository> {
        match self {
            Self::Postgres(db) => db.animal_repo(),
        }
    }

    pub fn weather_station_repo(&self) -> Arc<dyn WeatherStationRepo> {
        match self {
            Self::Postgres(db) => db.weather_station_repo(),
        }
    }

    pub fn weather_data_repo(&self) -> Arc<dyn WeatherDataRepo> {
        match self {
            Self::Postgres(db) => db.weather_data_repo(),
        }
    }

    pub fn fertilizer_record_repo(&self) -> Arc<dyn FertilizerRecordRepo> {
        match self {
            Self::Postgres(db) => db.fertilizer_record_repo(),
        }
    }

    pub fn plant_protection_record_repo(&self) -> Arc<dyn PlantProtectionRecordRepo> {
        match self {
            Self::Postgres(db) => db.plant_protection_record_repo(),
        }
    }

    pub fn harvest_season_repo(&self) -> Arc<dyn HarvestSeasonRepo> {
        match self {
            Self::Postgres(db) => db.harvest_season_repo(),
        }
    }

    pub fn harvest_lot_repo(&self) -> Arc<dyn HarvestLotRepo> {
        match self {
            Self::Postgres(db) => db.harvest_lot_repo(),
        }
    }

    pub fn harvest_delivery_repo(&self) -> Arc<dyn HarvestDeliveryRepo> {
        match self {
            Self::Postgres(db) => db.harvest_delivery_repo(),
        }
    }

    pub fn olive_grove_repo(&self) -> Arc<dyn OliveGroveRepo> {
        match self {
            Self::Postgres(db) => db.olive_grove_repo(),
        }
    }

    pub fn olive_oil_record_repo(&self) -> Arc<dyn OliveOilRecordRepo> {
        match self {
            Self::Postgres(db) => db.olive_oil_record_repo(),
        }
    }

    pub fn vineyard_repo(&self) -> Arc<dyn VineyardRepo> {
        match self {
            Self::Postgres(db) => db.vineyard_repo(),
        }
    }

    pub fn water_source_repo(&self) -> Arc<dyn WaterSourceRepo> {
        match self {
            Self::Postgres(db) => db.water_source_repo(),
        }
    }

    pub fn water_usage_repo(&self) -> Arc<dyn WaterUsageRepo> {
        match self {
            Self::Postgres(db) => db.water_usage_repo(),
        }
    }

    pub fn water_quota_repo(&self) -> Arc<dyn WaterQuotaRepo> {
        match self {
            Self::Postgres(db) => db.water_quota_repo(),
        }
    }

    pub fn worker_repo(&self) -> Arc<dyn WorkerRepo> {
        match self {
            Self::Postgres(db) => db.worker_repo(),
        }
    }

    pub fn worker_location_repo(&self) -> Arc<dyn WorkerLocationRepo> {
        match self {
            Self::Postgres(db) => db.worker_location_repo(),
        }
    }

    pub fn work_log_repo(&self) -> Arc<dyn WorkLogRepo> {
        match self {
            Self::Postgres(db) => db.work_log_repo(),
        }
    }

    pub fn worker_task_status_repo(&self) -> Arc<dyn WorkerTaskStatusRepository> {
        match self {
            Self::Postgres(db) => db.worker_task_status_repo(),
        }
    }

    pub fn task_data_repo(&self) -> Arc<dyn TaskDataRepository> {
        match self {
            Self::Postgres(db) => db.task_data_repo(),
        }
    }

    pub fn spatial_object_repo(&self) -> Arc<dyn SpatialObjectRepository> {
        match self {
            Self::Postgres(db) => db.spatial_object_repo(),
        }
    }

    pub fn phenology_record_repo(&self) -> Arc<dyn PhenologyRecordRepo> {
        match self {
            Self::Postgres(db) => db.phenology_record_repo(),
        }
    }

    pub fn pac_application_repo(&self) -> Arc<dyn PACApplicationRepo> {
        match self {
            Self::Postgres(db) => db.pac_application_repo(),
        }
    }

    pub fn cold_chain_log_repo(&self) -> Arc<dyn ColdChainLogRepo> {
        match self {
            Self::Postgres(db) => db.cold_chain_log_repo(),
        }
    }

    pub fn audit_log_repo(&self) -> Arc<dyn AuditLogRepo> {
        match self {
            Self::Postgres(db) => db.audit_log_repo(),
        }
    }

    pub fn compliance_checklist_repo(&self) -> Arc<dyn ComplianceChecklistRepo> {
        match self {
            Self::Postgres(db) => db.compliance_checklist_repo(),
        }
    }

    pub fn cost_center_repo(&self) -> Arc<dyn CostCenterRepo> {
        match self {
            Self::Postgres(db) => db.cost_center_repo(),
        }
    }

    pub fn financial_record_repo(&self) -> Arc<dyn FinancialRecordRepo> {
        match self {
            Self::Postgres(db) => db.financial_record_repo(),
        }
    }

    pub fn kelter_delivery_repo(&self) -> Arc<dyn KelterDeliveryRepo> {
        match self {
            Self::Postgres(db) => db.kelter_delivery_repo(),
        }
    }
}

/// PostgreSQL Database Wrapper
#[derive(Clone)]
pub struct PostgresDb {
    pub pool: PgPool,
}

impl PostgresDb {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    // Core repositories
    pub fn site_repo(&self) -> Arc<dyn SiteRepository> {
        Arc::new(PgSiteRepo::new(self.pool.clone()))
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
        Arc::new(crate::postgres::water_source::PgWaterSourceRepo::new(self.pool.clone()))
    }

    pub fn water_usage_repo(&self) -> Arc<dyn WaterUsageRepo> {
        Arc::new(crate::postgres::water_usage::PgWaterUsageRepo::new(self.pool.clone()))
    }

    pub fn water_quota_repo(&self) -> Arc<dyn WaterQuotaRepo> {
        Arc::new(crate::postgres::water_quota::PgWaterQuotaRepo::new(self.pool.clone()))
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
        Arc::new(crate::postgres::pac_application::PgPACApplicationRepo::new(self.pool.clone()))
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