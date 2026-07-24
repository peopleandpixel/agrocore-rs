//! API Data Transfer Objects

pub mod calculations;
pub mod common;
pub mod compliance;
pub mod equipment;
pub mod finance;
pub mod harvest;
pub mod livestock;
pub mod order;
pub mod plant_protection;
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
pub use livestock::*;
pub use order::*;
pub use plant_protection::*;
pub use site::*;
pub use specialized::*;
pub use water::*;
pub use weather::*;
pub use workforce::*;

// Re-export user module items with explicit names to avoid conflicts
pub use user::{
    AuthResponseDto, CreateUserDto, LoginDto, PaginatedUserResponse, RefreshRequest, UpdateUserDto,
    UserDto,
};
