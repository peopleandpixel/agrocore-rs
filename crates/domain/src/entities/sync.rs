//! Offline-First Sync Engine Domain Entities
//!
//! This module provides the core types for the sync engine including:
//! - Sync operations (create, update, delete)
//! - Conflict resolution strategies
//! - Sync state tracking
//! - Vector clocks for causal ordering

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::entities::tenant::TenantId;

/// Unique identifier for a sync client (device/app instance)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ClientId(pub Uuid);

impl ClientId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Entity type identifier for sync operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncEntityType {
    Site,
    Order,
    Task,
    TaskData,
    Equipment,
    Animal,
    WeatherStation,
    WeatherData,
    HarvestSeason,
    HarvestLot,
    HarvestDelivery,
    ColdChainLog,
    Vineyard,
    KelterDelivery,
    OliveGrove,
    OliveOilRecord,
    WaterSource,
    WaterUsage,
    WaterQuota,
    PlantProtectionRecord,
    ApplicatorLicense,
    FertilizerRecord,
    ComplianceChecklist,
    ComplianceItem,
    AuditLog,
    PacApplication,
    CostCenter,
    FinancialRecord,
    User,
    Worker,
    WorkerLocation,
    WorkLog,
    WorkerTaskStatus,
    PhenologyRecord,
}

impl std::fmt::Display for SyncEntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SyncEntityType::Site => "site",
            SyncEntityType::Order => "order",
            SyncEntityType::Task => "task",
            SyncEntityType::TaskData => "task_data",
            SyncEntityType::Equipment => "equipment",
            SyncEntityType::Animal => "animal",
            SyncEntityType::WeatherStation => "weather_station",
            SyncEntityType::WeatherData => "weather_data",
            SyncEntityType::HarvestSeason => "harvest_season",
            SyncEntityType::HarvestLot => "harvest_lot",
            SyncEntityType::HarvestDelivery => "harvest_delivery",
            SyncEntityType::ColdChainLog => "cold_chain_log",
            SyncEntityType::Vineyard => "vineyard",
            SyncEntityType::KelterDelivery => "kelter_delivery",
            SyncEntityType::OliveGrove => "olive_grove",
            SyncEntityType::OliveOilRecord => "olive_oil_record",
            SyncEntityType::WaterSource => "water_source",
            SyncEntityType::WaterUsage => "water_usage",
            SyncEntityType::WaterQuota => "water_quota",
            SyncEntityType::PlantProtectionRecord => "plant_protection_record",
            SyncEntityType::ApplicatorLicense => "applicator_license",
            SyncEntityType::FertilizerRecord => "fertilizer_record",
            SyncEntityType::ComplianceChecklist => "compliance_checklist",
            SyncEntityType::ComplianceItem => "compliance_item",
            SyncEntityType::AuditLog => "audit_log",
            SyncEntityType::PacApplication => "pac_application",
            SyncEntityType::CostCenter => "cost_center",
            SyncEntityType::FinancialRecord => "financial_record",
            SyncEntityType::User => "user",
            SyncEntityType::Worker => "worker",
            SyncEntityType::WorkerLocation => "worker_location",
            SyncEntityType::WorkLog => "work_log",
            SyncEntityType::WorkerTaskStatus => "worker_task_status",
            SyncEntityType::PhenologyRecord => "phenology_record",
        };
        write!(f, "{}", s)
    }
}

/// Sync operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncOperation {
    Create,
    Update,
    Delete,
}

/// Vector clock for causal ordering of sync operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct VectorClock {
    /// Map of client_id -> logical timestamp
    pub clocks: HashMap<ClientId, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    pub fn increment(&mut self, client_id: ClientId) {
        *self.clocks.entry(client_id).or_insert(0) += 1;
    }

    pub fn get(&self, client_id: &ClientId) -> u64 {
        self.clocks.get(client_id).copied().unwrap_or(0)
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (client_id, timestamp) in &other.clocks {
            let current = self.clocks.get(client_id).copied().unwrap_or(0);
            if *timestamp > current {
                self.clocks.insert(*client_id, *timestamp);
            }
        }
    }

    pub fn happens_before(&self, other: &VectorClock) -> bool {
        let mut at_least_one_less = false;
        for (client_id, timestamp) in &self.clocks {
            let other_ts = other.get(client_id);
            if *timestamp > other_ts {
                return false;
            }
            if *timestamp < other_ts {
                at_least_one_less = true;
            }
        }
        // Check for clients in other but not in self
        for (client_id, timestamp) in &other.clocks {
            if !self.clocks.contains_key(client_id) && *timestamp > 0 {
                at_least_one_less = true;
            }
        }
        at_least_one_less
    }

    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

/// A single sync operation (mutation) to be applied
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SyncMutation {
    /// Unique ID for this mutation
    pub id: Uuid,
    /// Entity type being modified
    pub entity_type: SyncEntityType,
    /// Entity ID
    pub entity_id: Uuid,
    /// Operation type
    pub operation: SyncOperation,
    /// Entity data (for create/update)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Vector clock for causal ordering
    pub vector_clock: VectorClock,
    /// Client that originated this mutation
    pub client_id: ClientId,
    /// Tenant ID for isolation
    pub tenant_id: TenantId,
    /// Timestamp when mutation was created
    pub created_at: DateTime<Utc>,
    /// Optional: base version for optimistic locking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_version: Option<u64>,
}

impl SyncMutation {
    pub fn new(
        entity_type: SyncEntityType,
        entity_id: Uuid,
        operation: SyncOperation,
        data: Option<serde_json::Value>,
        client_id: ClientId,
        tenant_id: TenantId,
    ) -> Self {
        let mut vector_clock = VectorClock::new();
        vector_clock.increment(client_id);

        Self {
            id: Uuid::new_v4(),
            entity_type,
            entity_id,
            operation,
            data,
            vector_clock,
            client_id,
            tenant_id,
            created_at: Utc::now(),
            base_version: None,
        }
    }
}

/// Batch of sync mutations from a client
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SyncBatch {
    /// Unique batch ID
    pub id: Uuid,
    /// Client that sent this batch
    pub client_id: ClientId,
    /// Tenant ID
    pub tenant_id: TenantId,
    /// List of mutations in this batch
    pub mutations: Vec<SyncMutation>,
    /// Client's current vector clock
    pub client_vector_clock: VectorClock,
    /// Timestamp when batch was created
    pub created_at: DateTime<Utc>,
}

impl SyncBatch {
    pub fn new(
        client_id: ClientId,
        tenant_id: TenantId,
        mutations: Vec<SyncMutation>,
        client_vector_clock: VectorClock,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            client_id,
            tenant_id,
            mutations,
            client_vector_clock,
            created_at: Utc::now(),
        }
    }
}

/// Result of applying a sync batch
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SyncBatchResult {
    /// Batch ID that was processed
    pub batch_id: Uuid,
    /// Results for each mutation
    pub mutation_results: Vec<SyncMutationResult>,
    /// Updated server vector clock
    pub server_vector_clock: VectorClock,
    /// Any conflicts detected
    pub conflicts: Vec<SyncConflict>,
}

/// Result of applying a single mutation
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SyncMutationResult {
    /// Original mutation ID
    pub mutation_id: Uuid,
    /// Whether the mutation was applied successfully
    pub success: bool,
    /// Entity ID (for creates, this might be newly generated)
    pub entity_id: Uuid,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Conflict info if a conflict occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict: Option<SyncConflict>,
}

/// Conflict detected during sync
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SyncConflict {
    /// Entity type
    pub entity_type: SyncEntityType,
    /// Entity ID
    pub entity_id: Uuid,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Server version of the entity
    pub server_data: Option<serde_json::Value>,
    /// Client version of the entity
    pub client_data: Option<serde_json::Value>,
    /// Suggested resolution
    pub suggested_resolution: ConflictResolution,
}

/// Type of conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    /// Same entity modified on both sides
    ConcurrentModification,
    /// Client tries to update deleted entity
    UpdateAfterDelete,
    /// Client tries to create entity that already exists
    DuplicateCreate,
    /// Version mismatch (optimistic locking)
    VersionMismatch,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    /// Last writer wins (based on timestamp)
    LastWriterWins,
    /// Server wins (authoritative)
    ServerWins,
    /// Client wins (force push)
    ClientWins,
    /// Merge if possible (for compatible fields)
    Merge,
    /// Manual resolution required
    Manual,
}

/// Sync state for a client
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ClientSyncState {
    /// Client ID
    pub client_id: ClientId,
    /// Tenant ID
    pub tenant_id: TenantId,
    /// Last successful sync timestamp
    pub last_sync_at: Option<DateTime<Utc>>,
    /// Client's vector clock at last sync
    pub last_vector_clock: VectorClock,
    /// Pending mutations not yet acknowledged
    pub pending_mutations: Vec<SyncMutation>,
    /// Entity versions known to client
    pub known_versions: HashMap<Uuid, u64>,
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SyncConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Default conflict resolution strategy
    pub default_conflict_resolution: ConflictResolution,
    /// Whether to enable CRDT-based merging
    pub enable_crdt: bool,
    /// Sync interval in seconds (for background sync)
    pub sync_interval_seconds: u64,
    /// Maximum retries for failed mutations
    pub max_retries: u32,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            default_conflict_resolution: ConflictResolution::LastWriterWins,
            enable_crdt: false,
            sync_interval_seconds: 300, // 5 minutes
            max_retries: 3,
        }
    }
}

/// Request to pull changes from server
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SyncPullRequest {
    /// Client ID
    pub client_id: ClientId,
    /// Tenant ID
    pub tenant_id: TenantId,
    /// Client's vector clock
    pub client_vector_clock: VectorClock,
    /// Entity types to sync (empty = all)
    #[serde(default)]
    pub entity_types: Vec<SyncEntityType>,
    /// Maximum number of mutations to return
    pub max_mutations: Option<usize>,
}

/// Response to pull request
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SyncPullResponse {
    /// Mutations since client's vector clock
    pub mutations: Vec<SyncMutation>,
    /// Server's current vector clock
    pub server_vector_clock: VectorClock,
    /// Whether there are more mutations available
    pub has_more: bool,
}

/// Full sync request (push + pull)
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SyncRequest {
    /// Push mutations from client
    #[serde(default)]
    pub push: Option<SyncBatch>,
    /// Pull changes from server
    #[serde(default)]
    pub pull: Option<SyncPullRequest>,
}

/// Full sync response
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SyncResponse {
    /// Push results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_result: Option<SyncBatchResult>,
    /// Pull response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_response: Option<SyncPullResponse>,
}
