//! Clock entry DTOs

use agrocore_domain::entities::workforce::{
    ClockEntry, CreateClockEntryDto as DomainCreateClockEntryDto,
    UpdateClockEntryDto as DomainUpdateClockEntryDto,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct ClockEntryDto {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub entry_type_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub task_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<ClockEntry> for ClockEntryDto {
    fn from(e: ClockEntry) -> Self {
        Self {
            id: e.id,
            worker_id: e.worker_id,
            entry_type_id: e.entry_type_id,
            timestamp: e.timestamp,
            lat: e.lat,
            lng: e.lng,
            task_id: e.task_id,
            notes: e.notes,
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedClockEntryResponse {
    pub data: Vec<ClockEntryDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateClockEntryRequest {
    pub worker_id: Uuid,
    pub entry_type_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub task_id: Option<Uuid>,
    pub notes: Option<String>,
}

impl From<CreateClockEntryRequest> for DomainCreateClockEntryDto {
    fn from(dto: CreateClockEntryRequest) -> Self {
        Self {
            worker_id: dto.worker_id,
            entry_type_id: dto.entry_type_id,
            timestamp: dto.timestamp,
            lat: dto.lat,
            lng: dto.lng,
            task_id: dto.task_id,
            notes: dto.notes,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateClockEntryRequest {
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub task_id: Option<Uuid>,
    pub notes: Option<String>,
}

impl From<UpdateClockEntryRequest> for DomainUpdateClockEntryDto {
    fn from(dto: UpdateClockEntryRequest) -> Self {
        Self {
            lat: dto.lat,
            lng: dto.lng,
            task_id: dto.task_id,
            notes: dto.notes,
        }
    }
}
