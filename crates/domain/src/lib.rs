pub mod entities;
pub mod repositories;
pub mod services;

// Re-export weather service provider trait and types
pub use services::weather::{WeatherDataProvider, WeatherFetchResult, WeatherServiceType};

#[cfg(feature = "mocks")]
pub mod mocks {
    pub use super::repositories::*;
    use std::sync::OnceLock;
    pub static MOCK_INIT: OnceLock<()> = OnceLock::new();
    pub fn init_mocks() {
        MOCK_INIT.get_or_init(|| {});
    }
    // Mock repositories initialized lazily — only when first accessed.
    // This avoids allocating all mock repositories at startup when mocks feature is enabled.
}

pub use agrocore_shared::{PaginatedResponse, Pagination, Result};
pub use entities::tenant::TenantId;
pub use entities::user::UserRole;
pub use repositories::{
    AnimalRepository, AuditLogRepo, ColdChainLogRepo, ComplianceChecklistRepo, CostCenterRepo,
    EquipmentRepository, FertilizerRecordRepo, FinancialRecordRepo, HarvestDeliveryRepo,
    HarvestLotRepo, HarvestSeasonRepo, InventoryItemRepository, InventoryLocationRepo,
    InventoryTransactionRepo, KelterDeliveryRepo, OliveGroveRepo, OliveOilRecordRepo,
    OrderRepository, PACApplicationRepo, PhenologyRecordRepo, PlantProtectionRecordRepo,
    RepositoryFuture, SiteRepository, SpatialObjectRepository, TaskDataRepository,
    TenantRepository, UserRepository, VineyardRepo, WaterQuotaRepo, WaterSourceRepo,
    WaterUsageRepo, WeatherDataRepo, WeatherStationRepo, WorkLogRepo, WorkerLocationRepo,
    WorkerRepo, WorkerTaskStatusRepository,
};

// Re-export mock types when mocks feature is enabled
#[cfg(feature = "mocks")]
pub use mocks::*;
