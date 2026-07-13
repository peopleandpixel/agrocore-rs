mod animal;
mod database;
mod equipment;
mod fertilizer_record;
mod harvest_delivery;
mod harvest_lot;
mod harvest_season;
mod cold_chain_log;
mod order;
mod olive_grove;
mod olive_oil_record;
mod phenology_record;
mod plant_protection_record;
mod site;
mod task_data;
mod tenant;
mod user;
mod vineyard;
mod kelter_delivery;
mod weather_data;
mod weather_station;
mod water_source;
mod water_usage;
mod water_quota;
mod worker;
mod work_log;
mod worker_location;
mod cost_center;
mod financial_record;
mod pac_application;

pub use animal::PgAnimalRepo;
pub use database::PostgresDb;
pub use equipment::PgEquipmentRepo;
pub use fertilizer_record::PgFertilizerRecordRepo;
pub use harvest_delivery::PgHarvestDeliveryRepo;
pub use harvest_lot::PgHarvestLotRepo;
pub use harvest_season::PgHarvestSeasonRepo;
pub use cold_chain_log::PgColdChainLogRepo;
pub use order::PgOrderRepo;
pub use olive_grove::PgOliveGroveRepo;
pub use olive_oil_record::PgOliveOilRecordRepo;
pub use phenology_record::PgPhenologyRecordRepo;
pub use plant_protection_record::PgPlantProtectionRecordRepo;
pub use site::PgSiteRepo;
pub use task_data::PgTaskDataRepo;
pub use tenant::PgTenantRepo;
pub use user::PgUserRepo;
pub use vineyard::PgVineyardRepo;
pub use kelter_delivery::PgKelterDeliveryRepo;
pub use weather_data::PgWeatherDataRepo;
pub use weather_station::PgWeatherStationRepo;
pub use water_source::PgWaterSourceRepo;
pub use water_usage::PgWaterUsageRepo;
pub use water_quota::PgWaterQuotaRepo;
pub use worker::PgWorkerRepo;
pub use work_log::PgWorkLogRepo;
pub use worker_location::PgWorkerLocationRepo;
pub use cost_center::PgCostCenterRepo;
pub use financial_record::PgFinancialRecordRepo;
pub use pac_application::PgPACApplicationRepo;

// Stub implementations für fehlende Module
pub mod harvest_delivery {
    pub struct PgHarvestDeliveryRepo;
}
pub mod harvest_lot {
    pub struct PgHarvestLotRepo;
}
pub mod cold_chain_log {
    pub struct PgColdChainLogRepo;
}
pub mod olive_grove {
    pub struct PgOliveGroveRepo;
}
pub mod olive_oil_record {
    pub struct PgOliveOilRecordRepo;
}