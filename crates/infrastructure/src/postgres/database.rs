use crate::postgres::{
    animal::PgAnimalRepo, cold_chain_log::PgColdChainLogRepo, equipment::PgEquipmentRepo,
    fertilizer_record::PgFertilizerRecordRepo, harvest_delivery::PgHarvestDeliveryRepo,
    harvest_lot::PgHarvestLotRepo, harvest_season::PgHarvestSeasonRepo, kelter_delivery::PgKelterDeliveryRepo,
    olive_grove::PgOliveGroveRepo, olive_oil_record::PgOliveOilRecordRepo, order::PgOrderRepo,
    phenology_record::PgPhenologyRecordRepo, plant_protection_record::PgPlantProtectionRecordRepo,
    site::PgSiteRepo, task_data::PgTaskDataRepo, tenant::PgTenantRepo, user::PgUserRepo,
    vineyard::PgVineyardRepo, weather_data::PgWeatherDataRepo, weather_station::PgWeatherStationRepo,
    worker::PgWorkerRepo,
};
use agrocore_domain::repositories::{
    AnimalRepository, EquipmentRepository, FertilizerRecordRepo, HarvestDeliveryRepo, HarvestLotRepo, HarvestSeasonRepo,
    OliveGroveRepo, OliveOilRecordRepo, OrderRepository, PhenologyRecordRepo, PlantProtectionRecordRepo,
    SiteRepository, TaskDataRepository, TenantRepository, UserRepository, VineyardRepo,
    WeatherDataRepo, WeatherStationRepo, WorkerRepository,
};
use sqlx::PgPool;
use std::sync::Arc;

pub enum Database {
    Postgres(PostgresDb),
    #[cfg(feature = "mongodb")]
    Mongo(crate::Database),
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

    pub fn task_data_repo(&self) -> Arc<dyn TaskDataRepository> {
        Arc::new(PgTaskDataRepo::new(self.pool.clone()))
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

    // Kelter/repository
    pub fn kelter_delivery_repo(&self) -> Arc<dyn crate::postgres::kelter_delivery::PgKelterDeliveryRepo> {
        Arc::new(PgKelterDeliveryRepo::new(self.pool.clone()))
    }

    // Worker
    pub fn worker_repo(&self) -> Arc<dyn WorkerRepository> {
        Arc::new(PgWorkerRepo::new(self.pool.clone()))
    }

    // Phenology
    pub fn phenology_record_repo(&self) -> Arc<dyn PhenologyRecordRepo> {
        Arc::new(PgPhenologyRecordRepo::new(self.pool.clone()))
    }
}

// Re-exports
pub use site::PgSiteRepo;
pub use user::PgUserRepo;
pub use order::PgOrderRepo;
pub use tenant::PgTenantRepo;