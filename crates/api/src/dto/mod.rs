//! API Data Transfer Objects

pub mod calculations;
pub mod common;
pub mod compliance;
pub mod customer;
pub mod equipment;
pub use equipment::{
    CreateFuelConsumptionRequest, CreateUsageLogRequest, DepreciationMethod,
    DepreciationScheduleEntry, EquipmentDepreciationDto, FuelConsumptionDto,
};
pub use equipment::{
    MaintenanceCostSummaryDto, MaintenanceLogDto, MaintenanceRecordDto, UsageLogDto,
    UsageSummaryDto,
};
pub mod finance;
pub mod harvest;
pub mod import;
pub mod inventory;
pub mod iot;
pub mod livestock;
pub mod order;
pub mod plant_protection;
pub mod settings;
pub mod site;
pub mod specialized;
pub mod user;
pub mod water;
pub mod weather;
pub mod workforce;

// Re-exports for handlers
pub use calculations::*;
pub use common::*;
pub use compliance::*;
pub use equipment::*;
pub use finance::*;
pub use harvest::*;
pub use import::*;
pub use inventory::*;
pub use iot::*;
pub use livestock::*;
pub use order::*;
pub use plant_protection::*;
pub use settings::*;
pub use site::*;
pub use specialized::*;
pub use user::*;
pub use water::*;
pub use weather::*;
pub use workforce::*;
