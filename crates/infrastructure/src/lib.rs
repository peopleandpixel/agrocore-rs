mod defaults;
mod postgres;

use std::sync::Arc;

pub use defaults::{default_bind_addr, default_nats_url};
pub use postgres::PostgresDb;

pub use postgres::PgSiteRepo as SiteRepo;
pub use postgres::PgUserRepo as UserRepo;
pub use postgres::PgOrderRepo as OrderRepo;
pub use postgres::PgTenantRepo as TenantRepo;
pub use postgres::PgEquipmentRepo as EquipmentRepo;
pub use postgres::PgAnimalRepo as AnimalRepo;
pub use postgres::PgTaskDataRepo as TaskDataRepo;
pub use postgres::PgWeatherStationRepo as WeatherStationRepo;
pub use postgres::PgWeatherDataRepo as WeatherDataRepo;
pub use postgres::PgFertilizerRecordRepo as FertilizerRecordRepo;
pub use postgres::PgPlantProtectionRecordRepo as PlantProtectionRecordRepo;
pub use postgres::PgHarvestSeasonRepo as HarvestSeasonRepo;
pub use postgres::PgHarvestLotRepo as HarvestLotRepo;
pub use postgres::PgHarvestDeliveryRepo as HarvestDeliveryRepo;

// MongoDB removed - PostgreSQL only
use postgres::PostgresDb;

#[derive(Clone)]
pub enum Database {
    Postgres(PostgresDb),
}

impl Database {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let db = PostgresDb::connect(database_url).await?;
        Ok(Self::Postgres(db))
    }
}