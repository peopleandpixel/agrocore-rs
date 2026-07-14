use crate::postgres::{
    animal::PgAnimalRepo, cold_chain_log::PgColdChainLogRepo, equipment::PgEquipmentRepo,
    fertilizer_record::PgFertilizerRecordRepo, harvest_delivery::PgHarvestDeliveryRepo,
    harvest_lot::PgHarvestLotRepo, harvest_season::PgHarvestSeasonRepo,
    olive_grove::PgOliveGroveRepo, olive_oil_record::PgOliveOilRecordRepo, order::PgOrderRepo,
    phenology_record::PgPhenologyRecordRepo, plant_protection_record::PgPlantProtectionRecordRepo,
    site::PgSiteRepo, tenant::PgTenantRepo, user::PgUserRepo,
    vineyard::PgVineyardRepo, weather_data::PgWeatherDataRepo, weather_station::PgWeatherStationRepo,
    worker::PgWorkerRepo, worker_location::PgWorkerLocationRepo, work_log::PgWorkLogRepo,
};
use agrocore_domain::repositories::{
    AnimalRepository, EquipmentRepository, FertilizerRecordRepo, HarvestDeliveryRepo, HarvestLotRepo, HarvestSeasonRepo,
    OliveGroveRepo, OliveOilRecordRepo, OrderRepository, PhenologyRecordRepo, PlantProtectionRecordRepo,
    SiteRepository, TenantRepository, UserRepository, VineyardRepo,
    WaterQuotaRepo, WaterSourceRepo, WaterUsageRepo,
    WeatherDataRepo, WeatherStationRepo, WorkerLocationRepo, WorkerRepo, WorkLogRepo,
    PACApplicationRepo, CostCenterRepo, FinancialRecordRepo, ColdChainLogRepo,
};
use sqlx::PgPool;
use std::sync::Arc;

pub enum Database {
    Postgres(PostgresDb),
}

/// PostgreSQL Database Wrapper
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
}