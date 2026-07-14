mod defaults;
mod postgres;

use std::sync::Arc;

pub use defaults::{default_bind_addr, default_nats_url};

// PostgreSQL Re-exports
pub use postgres::PostgresDb;
pub use postgres::PgSiteRepo;
pub use postgres::PgUserRepo;
pub use postgres::PgOrderRepo;
pub use postgres::PgTenantRepo;
pub use postgres::PgEquipmentRepo;
pub use postgres::PgAnimalRepo;
pub use postgres::PgTaskDataRepo;
pub use postgres::PgWeatherStationRepo;
pub use postgres::PgWeatherDataRepo;
pub use postgres::PgFertilizerRecordRepo;
pub use postgres::PgPlantProtectionRecordRepo;
pub use postgres::PgHarvestSeasonRepo;
pub use postgres::PgHarvestLotRepo;
pub use postgres::PgHarvestDeliveryRepo;
pub use postgres::PgVineyardRepo;
pub use postgres::PgPhenologyRecordRepo;

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