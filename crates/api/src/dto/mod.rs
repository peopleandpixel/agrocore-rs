//! API Data Transfer Objects

pub mod backup;
pub mod breed;
pub mod building;
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
pub mod group;
pub mod harvest;
pub mod import;
pub mod inventory;
pub mod iot;
pub mod livestock;
pub mod livestock_new;
pub mod order;
pub mod plant_protection;
pub mod settings;
pub mod site;
pub mod specialized;
pub mod tree;
pub mod user;
pub mod variety;
pub mod water;
pub mod weather;
pub mod workforce;

pub mod demo;

// Re-exports for handlers
pub use backup::*;
pub use breed::*;
pub use building::*;
pub use calculations::*;
pub use common::*;
pub use compliance::*;
pub use demo::*;
pub use equipment::*;
pub use finance::*;
pub use group::*;
pub use harvest::*;
pub use import::*;
pub use inventory::*;
pub use iot::*;
pub use livestock::*;
pub use livestock_new::*;
pub use order::*;
pub use plant_protection::*;
pub use settings::*;
pub use site::*;
pub use specialized::*;
pub use tree::*;
pub use user::*;
pub use variety::*;
pub use water::*;
pub use weather::*;
pub use workforce::*;
