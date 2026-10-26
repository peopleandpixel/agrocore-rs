---
sessionId: session-261022-151712-1be1
---

# Requirements

### Overview & Goals
Replace all ~55 hardcoded enum types in the AgroCore domain with a database-driven dynamic type classification system. Types are stored in two generic tables (`type_categories` + `type_entries`), support hierarchical parent-child relationships, and can be extended by users with appropriate permissions (Admin/SuperAdmin).

### User Stories
- As an Admin, I want to create new types (e.g., a new animal species, equipment type, or order type) so that my farm can use categories not in the default list.
- As a Manager, I want to see all available types in dropdowns, including tenant-specific types created by my Admin.
- As a Worker, I want to select from a consistent list of types when creating orders, equipment, animals, etc.
- As a SuperAdmin, I want to manage global types that are available to all tenants.

### Scope

#### In Scope
- Two new database tables: `type_categories` and `type_entries`
- Migration of all ~55 hardcoded enums to database-driven types
- New domain entities, repository traits, and PostgreSQL implementations
- New API endpoints for type management (CRUD + hierarchy queries)
- New API endpoints for listing available types per category
- Updating all existing entities to reference `type_entries` via UUID instead of enum values
- Seeding all initial types from current enum variants
- Permission checks: only Admin/SuperAdmin can create/update/delete types
- Sync system support for type entries

#### Out of Scope
- Frontend implementation
- I18n/localization of type names (types store a single `name` + optional `translations` JSONB)
- Type validation rules per category
- Bulk import/export of types

### Functional Requirements
1. **Type Categories**: Each enum becomes a category (e.g., `animal_species`, `site_type`, `order_type`, `equipment_type`).
2. **Type Entries**: Each enum variant becomes a type entry with `category_id`, `parent_id` (for hierarchy), `name`, `code` (slug), and `tenant_id` (NULL = global).
3. **Hierarchy**: Entries can have parent entries (e.g., Geflügel → Huhn, Ente, Gans, Truthahn).
4. **Tenant Isolation**: Global types (`tenant_id=NULL`) are visible to all tenants; tenant-specific types only to their tenant.
5. **CRUD Operations**: Admins can create, update, delete tenant-specific types.
6. **Backward Compatibility**: Initial seed data maps 1:1 from existing enum variants.

### Non-Functional Requirements
- Type lookups must be cached to avoid N+1 queries
- Deleting a type with references must be prevented (foreign key constraints)
- API responses include type name and code alongside UUID for display purposes

# Technical Design

### Current Implementation
All types are hardcoded as Rust enums in `crates/domain/src/entities/`:
- `SiteType`, `CropType`, `OrderType`, `OrderStatus` in `mod.rs`
- `AnimalSpecies`, `LivestockType`, `AnimalStatus` in `livestock.rs`
- `EquipmentType` in `equipment.rs`
- `BuildingType` in `building.rs`
- `GroupType` in `group.rs`
- `TreeType` in `tree.rs`
- `WaterSourceType`, `IrrigationMethod` in `water.rs`
- `WeatherStationType` in `weather.rs`
- `DocArea`, `QualityGrade` in `vineyard.rs`
- `OilGrade` in `olive.rs`
- `InventoryCategory`, `UnitOfMeasure` in `inventory.rs`
- `BbchStage` in `mod.rs`
- And ~40 more enums across compliance, finance, workforce, sync, etc.

DB columns store these as `JSONB` (e.g., `site_type JSONB`) or `VARCHAR` (e.g., `order_type VARCHAR(50)`).

### Existing Reference Catalogues (Preserved)
The following tables already exist as database-driven reference data and will **not** be migrated into `type_entries`. They serve a different purpose — detailed catalogues rather than simple type classifications:

| Table | Purpose | Seed Data | Relationship to type_entries |
|-------|---------|-----------|------------------------------|
| `varieties` | Plant cultivar catalogue (Cabernet Sauvignon, Merlot, Arbequina, etc.) | 46 varieties across Grape, Olive, Nut, Berry, Fruit, Citrus, Vegetable, Grain | `VarietyCategory` enum (Grape, Olive, etc.) becomes a `type_entries` category. The varieties table itself stays intact. |
| `breeds` | Animal breed catalogue (Murciano-Granadina, Holstein, Merino, etc.) | ~35 breeds across Goat, Chicken, Sheep, Cattle, Horse, etc. | `Species` enum (Goat, Sheep, etc.) becomes a `type_entries` category. The breeds table itself stays intact. |
| `crop_species` | Scientific species reference (Vitis vinifera, Olea europaea, etc.) | Empty — populated from code | Stays as-is. Links varieties and breeds to their scientific identity. |

**Key distinction**: `type_entries` replaces simple classification enums (SiteType, OrderType, EquipmentType). The varieties/breeds/crop_species tables are rich reference catalogues with attributes like origin, registration countries, performance data, etc. They remain as separate tables and are not flattened into type_entries.

### Key Decisions
1. **Two-table generic design**: `type_categories` (metadata) + `type_entries` (all types). Single source of truth, maximum flexibility.
2. **UUID references everywhere**: Entities reference types via `UUID` foreign keys instead of enum strings.
3. **Global + tenant types**: `tenant_id NULL` = global (system-provided), `tenant_id NOT NULL` = tenant-specific.
4. **Immediate API switch**: DTOs use `Uuid` for type references; responses include resolved type name/code.
5. **Hierarchy via `parent_id`**: Self-referencing FK in `type_entries` for parent-child relationships.
6. **Soft delete**: `is_active` flag instead of hard delete to preserve referential integrity.
7. **Multi-level type references**: Entities can reference any node in the type hierarchy (root, intermediate, or leaf). For example, a spatial object can be typed as "Building" (intermediate) or "Barn" (leaf). The hierarchy provides context and grouping, not validation constraints.
8. **Spatial types hierarchy**: `SpatialObjectType` will be seeded with proper parent-child relationships: Farm → Site → Field/Pasture; Building → Barn/Stable/Shed/Pen; Tree → CorkOak/OliveTree; plus standalone entries like WaterPoint, Fence.

### Database Schema

#### `type_categories` table
```sql
CREATE TABLE type_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,        -- e.g. 'animal_species', 'site_type'
    label VARCHAR(200) NOT NULL,              -- e.g. 'Animal Species', 'Site Type'
    description TEXT,
    is_hierarchical BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

#### `type_entries` table
```sql
CREATE TABLE type_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id UUID NOT NULL REFERENCES type_categories(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES type_entries(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,  -- NULL = global
    name VARCHAR(255) NOT NULL,              -- display name
    code VARCHAR(100) NOT NULL,              -- slug/identifier (e.g. 'chicken', 'vineyard')
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(category_id, code, tenant_id)
);

CREATE INDEX idx_type_entries_category ON type_entries(category_id);
CREATE INDEX idx_type_entries_parent ON type_entries(parent_id);
CREATE INDEX idx_type_entries_tenant ON type_entries(tenant_id);
CREATE INDEX idx_type_entries_code ON type_entries(code);
```

### Domain Entities

New module: `crates/domain/src/entities/type_system.rs`

```rust
pub struct TypeCategory {
    pub id: Uuid,
    pub name: String,          // e.g. 'animal_species'
    pub label: String,         // e.g. 'Animal Species'
    pub description: Option<String>,
    pub is_hierarchical: bool,
    pub created_at: DateTime<Utc>,
}

pub struct TypeEntry {
    pub id: Uuid,
    pub category_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub tenant_id: Option<TenantId>,  // None = global
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CreateTypeEntryDto {
    pub category_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
}

pub struct UpdateTypeEntryDto {
    pub parent_id: Option<Uuid>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}
```

### Repository Trait

In `crates/domain/src/repositories.rs`:

```rust
pub trait TypeCategoryRepository: Send + Sync {
    fn find_by_id(&self, id: Uuid) -> RepositoryFuture<Option<TypeCategory>>;
    fn find_all(&self) -> RepositoryFuture<Vec<TypeCategory>>;
    fn find_by_name(&self, name: &str) -> RepositoryFuture<Option<TypeCategory>>;
}

pub trait TypeEntryRepository: Send + Sync {
    fn find_by_id(&self, id: Uuid) -> RepositoryFuture<Option<TypeEntry>>;
    fn find_by_category(&self, tid: TenantId, category_id: Uuid) -> RepositoryFuture<Vec<TypeEntry>>;
    fn find_by_category_and_code(&self, tid: TenantId, category_id: Uuid, code: &str) -> RepositoryFuture<Option<TypeEntry>>;
    fn find_children(&self, tid: TenantId, parent_id: Uuid) -> RepositoryFuture<Vec<TypeEntry>>;
    fn find_root_entries(&self, tid: TenantId, category_id: Uuid) -> RepositoryFuture<Vec<TypeEntry>>;
    fn create(&self, tid: TenantId, dto: CreateTypeEntryDto) -> RepositoryFuture<TypeEntry>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateTypeEntryDto) -> RepositoryFuture<Option<TypeEntry>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    fn soft_delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}
```

### Entity Changes

Each entity that currently uses an enum will be changed to reference `type_entries` by UUID:

**Before** (e.g., `Equipment`):
```rust
pub struct Equipment {
    pub equipment_type: EquipmentType,  // enum
}
```

**After**:
```rust
pub struct Equipment {
    pub type_id: Uuid,  // FK to type_entries
}
```

The same pattern applies to: `Site` (site_type, crop_type), `Order` (order_type), `Animal` (species), `Building` (building_type), `Group` (group_type), `Tree` (tree_type), `WaterSource` (source_type), `WaterUsage` (irrigation_method), `WeatherStation` (station_type), `InventoryItem` (category, unit_of_measure), `Vineyard` (quality_grade), `OliveOilRecord` (oil_grade), and all other enum-using entities.

### API Changes

New handler: `crates/api/src/handlers/type_system.rs`

Endpoints:
- `GET /type-categories` — list all categories
- `GET /type-categories/{id}` — category details
- `GET /types?category_id=...&tenant_id=...` — list types with optional filters
- `GET /types/{id}` — type details
- `GET /types/{id}/children` — child types (hierarchy)
- `POST /types` — create type (Admin only)
- `PUT /types/{id}` — update type (Admin only)
- `DELETE /types/{id}` — soft delete (Admin only)

DTO changes: All DTOs that currently accept enum values will accept `Uuid` (type_id) instead. Responses include a `TypeRef` struct:
```rust
pub struct TypeRef {
    pub id: Uuid,
    pub name: String,
    pub code: String,
}
```

### Migration Strategy

Migration `0000000029_dynamic_type_system.sql`:
1. Create `type_categories` and `type_entries` tables
2. Seed all ~55 categories from existing enums
3. Seed all type entries from existing enum variants (with `tenant_id=NULL` for global)
4. For each entity table, add a `type_id UUID` column
5. Migrate existing data: join enum values to new type_entries by code
6. Drop old enum columns (in a follow-up migration for safety)

### Architecture Diagram

``` mermaid
graph LR
    A[API Handler type_system.rs] --> B[TypeEntryRepository]
    A --> C[TypeCategoryRepository]
    B --> D[Postgres type_entries]
    C --> E[Postgres type_categories]
    F[Site] --> D
    G[Order] --> D
    H[Equipment] --> D
    I[Animal] --> D
    J[Building] --> D
    K[Tree] --> D
    L[WaterSource] --> D
    M[InventoryItem] --> D
    D -->|parent_id| D
    D -->|category_id| E
    D -->|tenant_id| N[tenants]
```

### Risks
- **Data migration complexity**: Migrating JSONB/VARCHAR enum values to UUID FKs requires careful mapping. Mitigation: seed data uses identical codes to enum serialization.
- **Performance**: Every type reference requires a JOIN. Mitigation: include `TypeRef` (name + code) in entity responses via JOIN at query time; cache frequently accessed types.
- **Breaking API change**: Clients need to update from enum strings to UUIDs. Mitigation: response includes `TypeRef` with name/code for display.
- **Circular parent references**: Validation needed to prevent circular hierarchies. Mitigation: DB constraint + application-level validation.
- **Multi-level type references**: Entities can reference intermediate nodes (e.g., "Building" instead of "Barn"), which may cause ambiguity in queries. Mitigation: queries can use recursive CTEs to find all descendants when filtering by a parent type. For example, filtering by "Building" returns all objects typed as Building, Barn, Stable, Shed, or Pen.

# Testing

### Validation Approach
The agent will validate the implementation by:
1. Verifying the migration creates tables and seeds data correctly
2. Testing repository CRUD operations
3. Testing API endpoints with proper auth
4. Testing permission checks (non-Admin cannot create types)
5. Testing hierarchy queries (parent-child relationships)
6. Testing tenant isolation (tenant-specific types only visible to their tenant)

### Key Scenarios
- Create a new type entry (e.g., new animal species "Emu")
- List all types for a category (should include global + tenant-specific)
- Create a child type under a parent (e.g., "Emu" under "Geflügel")
- Create an entity (e.g., Animal) with a type reference
- Query entities filtered by type
- Soft delete a type (should fail if referenced by entities)
- Non-Admin user cannot create types

### Edge Cases
- Circular parent references (A → B → A)
- Deleting a global type referenced by tenant data
- Duplicate code within same category and tenant
- Type entry with parent in different category (should be rejected)
- Empty category (no entries)
- Tenant-specific type with same code as global type
- Entity references an intermediate type node (e.g., "Building" instead of "Barn") — queries should include descendants
- Moving a type node with children — all child references should remain valid
- Filtering entities by a parent type should return entities typed with any descendant

# Delivery Steps

### ✓ Step 1: Database schema and migration
Create the two core tables and seed all initial type data from existing enums.

**Table creation:**
- Create `type_categories` table with name, label, description, is_hierarchical columns
- Create `type_entries` table with category_id, parent_id, tenant_id, name, code, is_active, sort_order columns
- Add all indexes (category_id, parent_id, tenant_id, code)
- Add foreign key constraints with CASCADE delete

**Seed data (all ~55 categories and their entries):**
- `animal_species`: Goat, Chicken, Sheep, Cattle, Pig, Horse, Duck, Turkey, Goose (+ Geflügel parent for Duck, Turkey, Goose, Chicken, Goose)
- `site_type`: Vineyard, Field, CorkOakMontado, HolmOakMontado, OliveGrove, Orchard, AlmondOrchard, CitrusGrove, Pasture, Greenhouse
- `crop_type`: Grape, Olive, Apple, Citrus, Nut, CorkOak, Almond, Hazelnut, Chestnut, Berry, Tropical, Poultry, Livestock, Fallow, Forest, Pasture, Grain, Unknown
- `order_type`: PlantProtection, Fertilization, Pruning, Harvest, SoilWork, Irrigation, Monitoring, LivestockFeeding, LivestockWatering, LivestockRelocation, LivestockHealthCheck, BarnCleaning, EggCollection, Shearing, Milking, SalesOrder
- `order_status`: Draft, Planned, InProgress, Completed, Cancelled
- `equipment_type`: Tractor, Sprayer, Harvester, Mulcher, Plow, Trailer, Tool
- `building_type`: ChickenCoop, GoatStable, Barn
- `group_type`: Herd, Flock, Grove, Coop, Barn
- `tree_type`: CorkOak, Olive, Almond
- `water_source_type`: Well, Reservoir, River, Canal, ComunidadDeRegantes, RainwaterHarvesting, Desalination
- `irrigation_method`: Drip, Sprinkler, Flood, Pivot, MicroSprinkler, Subsurface
- `animal_status`: Active, Inactive, Sold, Deceased
- `weather_station_type`: Iot, Manual, Virtual, ExternalApi
- `inventory_category`: Seed, Fertilizer, Pesticide, Herbicide, Fuel, Feed, Medicine, SparePart, Equipment
- `unit_of_measure`: Kg, Liter, Bag, Unit, Box, Bale
- `oil_grade`: ExtraVirgin, Virgin, Lampante, Refined, Pomace
- `quality_grade`: Reserva, GrandeReserva, Garrafeira, Superior, Classic, LateHarvest
- `doc_area`: Douro, Alentejo, VinhoVerde, Dao, Bairrada, Lisboa, PeninsulaDeSetubal, Algarve, Tejo, Minho, Trasmontes
- `bbch_stage`: All 17 BBCH stages
- `spatial_object_type`: Hierarchical structure:
  - Farm (root)
    - Site (child of Farm)
      - Field (child of Site)
      - Pasture (child of Site)
  - Building (root)
    - Barn (child of Building)
    - Stable (child of Building)
    - Shed (child of Building)
    - Pen (child of Building)
  - Tree (root)
    - CorkOak (child of Tree)
    - OliveTree (child of Tree)
  - WaterPoint (standalone root)
  - Fence (standalone root)
  
  **Note**: Entities can reference any level (e.g., a spatial object can be typed as "Building" or "Barn"). The hierarchy provides grouping and context, not strict leaf-only validation.
- `variety_category`: Grape, Olive, Apple, Citrus, Nut, Berry, Vegetable, Grain
- `species`: Goat, Sheep, Cattle, Chicken, Pig, Horse
- `checklist_type`: GAP, Organic, GlobalGAP, HACCP
- `compliance_status`: Pending, InProgress, Completed, Failed, Expired
- `audit_action`: Created, Updated, Deleted, Viewed, Exported, Approved, Rejected
- `pac_status`: Draft, Submitted, InReview, Approved, Rejected, Paid
- `cost_center_type`: Site, Crop, Activity, General
- `financial_record_type`: Expense, Income, Subsidy
- `contract_type`: Permanent, Temporary, Seasonal, Freelance, Apprenticeship
- `clock_entry_type`: ClockIn, ClockOut
- `lot_status`: Collecting, Processed, Shipped, Stored
- `plant_protection_area_method`: NetArea, GrossArea, LeafWallMeasured, LeafWallGross, GroundProtection, HerbicideProtection, TreeCrownVolume
- `task_execution_mode`: Manual, AutoPresence, ManualWithDefaultDuration
- `recurrence_cadence`: Hourly, Daily, Weekly, Monthly, Yearly
- `sub_task_status`: New, Started, Paused, Stopped, Done
- `overall_task_status`: New, InProgress, Stopped, Completed
- `depreciation_method`: StraightLine, DoubleDeclining
- `inventory_method`: Fifo, Fefo
- `transaction_type`: StockIn, StockOut, Transfer, Adjustment
- `risk_level`: Low, Medium, High, Critical
- `iot_device_type`: WeatherStation, SoilSensor, SoilMoistureProbe, IrrigationValve, Generic
- `setting_value_type`: String, Number, Boolean, Json, Array
- `sync_entity_type`: All 35 entity types
- `sync_operation`: Create, Update, Delete
- `conflict_type`: ConcurrentModification, UpdateAfterDelete, DuplicateCreate, VersionMismatch
- `conflict_resolution`: LastWriterWins, ServerWins, ClientWins, Merge, Manual

**Existing seed data preservation:**
- **varieties table**: Keep as-is. All 46 seeded varieties (Cabernet Sauvignon, Merlot, Arbequina, etc.) remain in their table.
- **breeds table**: Keep as-is. All ~35 seeded breeds (Murciano-Granadina, Holstein, Merino, etc.) remain in their table.
- **crop_species table**: Keep as-is. No data loss.
- These tables are **not** migrated into type_entries. They are rich reference catalogues with attributes (origin, registration_countries, performance data) that don't fit the simple type_entries model.

**Entity column additions:**
- Add `type_id UUID` columns to: sites (site_type_id, crop_type_id), orders (order_type_id), equipment (type_id), animals (species_id), buildings (type_id), groups (type_id), trees (type_id), water_sources (source_type_id), water_usage (irrigation_method_id), weather_stations (station_type_id), inventory_items (category_id, unit_of_measure_id), vineyards (quality_grade_id), olive_oil_records (oil_grade_id), plant_protection_records (area_method_id), tasks (execution_mode_id), compliance_checklists (checklist_type_id, status_id), pac_applications (status_id), cost_centers (type_id), financial_records (type_id), workers (contract_type_id), harvest_lots (status_id), olive_groves (variety_category_id), livestock (species_id, status_id)
- Migrate existing enum data to UUID references by matching codes
- Add foreign key constraints to type_entries
- **No changes** to varieties, breeds, or crop_species tables or their references from other tables

### ✓ Step 2: Domain entities and repository traits
Create the type system domain module and add repository traits.

**New file `crates/domain/src/entities/type_system.rs`:**
- `TypeCategory` struct with id, name, label, description, is_hierarchical, created_at
- `TypeEntry` struct with id, category_id, parent_id, tenant_id (Option), name, code, description, is_active, sort_order, timestamps
- `CreateTypeEntryDto` and `UpdateTypeEntryDto`
- `CreateTypeCategoryDto` and `UpdateTypeCategoryDto`
- Helper methods: `is_global()`, `has_children()`, `path_to_root()`

**Update `crates/domain/src/entities/mod.rs`:**
- Add `pub mod type_system;`
- Re-export types from type_system module

**Update all entity structs to use UUID type references:**
- `Site`: `site_type: JSONB` → `site_type_id: Uuid`, `crop_type: JSONB` → `crop_type_id: Uuid`
- `Order`: `order_type: OrderType` → `order_type_id: Uuid`
- `Equipment`: `equipment_type: EquipmentType` → `type_id: Uuid`
- `Animal`/`Livestock`: `species: AnimalSpecies` → `species_id: Uuid`, `status: AnimalStatus` → `status_id: Uuid`
- `Building`: `building_type: String` → `type_id: Uuid`
- `Group`: `group_type: String` → `type_id: Uuid`
- `Tree`: `tree_type: String` → `type_id: Uuid`
- `WaterSource`: `source_type: WaterSourceType` → `source_type_id: Uuid`
- `WaterUsage`: `irrigation_method: IrrigationMethod` → `irrigation_method_id: Uuid`
- `WeatherStation`: `station_type: WeatherStationType` → `station_type_id: Uuid`
- `InventoryItem`: `category: InventoryCategory` → `category_id: Uuid`, `unit: UnitOfMeasure` → `unit_id: Uuid`
- `Vineyard`: `quality_grade: QualityGrade` → `quality_grade_id: Option<Uuid>`
- `OliveOilRecord`: `oil_grade: OilGrade` → `oil_grade_id: Uuid`
- `PlantProtectionRecord`: `area_method: PlantProtectionAreaMethod` → `area_method_id: Uuid`
- `Task`: `execution_mode: TaskExecutionMode` → `execution_mode_id: Uuid`
- `ComplianceChecklist`: `checklist_type: ChecklistType` → `type_id: Uuid`, `status: ComplianceStatus` → `status_id: Uuid`
- `PacApplication`: `status: PACStatus` → `status_id: Uuid`
- `CostCenter`: `cost_center_type: CostCenterType` → `type_id: Uuid`
- `FinancialRecord`: `record_type: FinancialRecordType` → `type_id: Uuid`
- `Worker`: `contract_type: ContractType` → `contract_type_id: Uuid`
- `HarvestLot`: `status: LotStatus` → `status_id: Uuid`

**Repository traits in `crates/domain/src/repositories.rs`:**
- `TypeCategoryRepository`: find_by_id, find_all, find_by_name
- `TypeEntryRepository`: find_by_id, find_by_category, find_by_category_and_code, find_children, find_root_entries, create, update, delete, soft_delete

### ✓ Step 3: PostgreSQL repository implementations
Implement the PostgreSQL repository for type categories and type entries.

**New file `crates/infrastructure/src/postgres/type_category.rs`:**
- `PgTypeCategoryRepo` struct implementing `TypeCategoryRepository`
- `find_by_id`: SELECT by UUID
- `find_all`: SELECT all categories ordered by name
- `find_by_name`: SELECT by unique name

**New file `crates/infrastructure/src/postgres/type_entry.rs`:**
- `PgTypeEntryRepo` struct implementing `TypeEntryRepository`
- `find_by_id`: SELECT by UUID with JOIN to category for name
- `find_by_category`: SELECT global (tenant_id IS NULL) + tenant-specific entries, ordered by sort_order then name
- `find_by_category_and_code`: SELECT by category + code, preferring tenant-specific over global
- `find_children`: SELECT WHERE parent_id = ? with tenant filtering
- `find_root_entries`: SELECT WHERE parent_id IS NULL for category
- `create`: INSERT with tenant_id, validate uniqueness of (category_id, code, tenant_id)
- `update`: UPDATE with validation (no circular parent, same category)
- `soft_delete`: SET is_active = FALSE
- `delete`: Hard delete (only if no references)

**Update `crates/infrastructure/src/postgres/database.rs`:**
- Add `type_category_repo` and `type_entry_repo` fields
- Initialize repos in `new()` constructor

**Update existing repos to handle UUID type references:**
- `site.rs`: Query includes JOIN to type_entries for site_type and crop_type names
- `order.rs`: Query includes JOIN for order_type name
- `equipment.rs`: Query includes JOIN for type name
- `animal.rs`/`livestock.rs`: Query includes JOIN for species and status names
- All other affected repos get similar JOIN queries

### ✓ Step 4: API handlers and DTOs
Create API handlers for type management and update all existing DTOs.

**New file `crates/api/src/handlers/type_system.rs`:**
- `list_type_categories` — GET /type-categories
- `get_type_category` — GET /type-categories/{id}
- `list_types` — GET /types?category_id=&include_children=&tenant_id=
- `get_type` — GET /types/{id}
- `get_type_children` — GET /types/{id}/children
- `create_type` — POST /types (Admin/SuperAdmin only)
- `update_type` — PUT /types/{id} (Admin/SuperAdmin only)
- `delete_type` — DELETE /types/{id} (Admin/SuperAdmin only, soft delete)
- `configure()` function to register routes under `/types` scope

**New file `crates/api/src/dto/type_system.rs`:**
- `TypeCategoryDto` — API response for categories
- `TypeEntryDto` — API response for type entries (includes category name, parent name)
- `TypeRef` — Lightweight reference: {id, name, code} for embedding in entity responses
- `CreateTypeEntryDto` — Request body for creating types
- `UpdateTypeEntryDto` — Request body for updating types
- `TypeHierarchyDto` — Nested structure for tree-like display

**Update `crates/api/src/handlers/mod.rs`:**
- Register the type_system handler module

**Update all existing DTOs to use UUID type references:**
- `CreateSiteDto`/`UpdateSiteDto`: `site_type: ApiSiteType` → `site_type_id: Uuid`, `crop_type: ApiCropType` → `crop_type_id: Uuid`
- `SiteDto`: Add `site_type: TypeRef`, `crop_type: TypeRef`
- `CreateOrderDto`/`UpdateOrderDto`: `order_type: OrderType` → `order_type_id: Uuid`
- `OrderDto`: Add `order_type: TypeRef`
- `CreateEquipmentDto`/`UpdateEquipmentDto`: `equipment_type: EquipmentType` → `type_id: Uuid`
- `EquipmentDto`: Add `equipment_type: TypeRef`
- `CreateAnimalDto`/`UpdateAnimalDto`: `species: AnimalSpecies` → `species_id: Uuid`
- `AnimalDto`: Add `species: TypeRef`, `status: TypeRef`
- `CreateBuildingDto`/`UpdateBuildingDto`: `building_type: BuildingType` → `type_id: Uuid`
- `BuildingDto`: Add `building_type: TypeRef`
- `CreateGroupDto`/`UpdateGroupDto`: `group_type: GroupType` → `type_id: Uuid`
- `GroupDto`: Add `group_type: TypeRef`
- `CreateTreeDto`/`UpdateTreeDto`: `tree_type: TreeType` → `type_id: Uuid`
- `TreeDto`: Add `tree_type: TypeRef`
- `CreateWaterSourceDto`/`UpdateWaterSourceDto`: `source_type: WaterSourceType` → `source_type_id: Uuid`
- `WaterSourceDto`: Add `source_type: TypeRef`
- `CreateWaterUsageDto`: `irrigation_method: IrrigationMethod` → `irrigation_method_id: Uuid`
- `WaterUsageDto`: Add `irrigation_method: TypeRef`
- `WeatherStationDto`: Add `station_type: TypeRef`
- `InventoryItemDto`: Add `category: TypeRef`, `unit_of_measure: TypeRef`
- `VineyardDto`: Add `quality_grade: Option<TypeRef>`
- `OliveOilRecordDto`: Add `oil_grade: TypeRef`
- All remaining DTOs follow the same pattern

**Remove `crates/api/src/dto/enums.rs`:**
- Delete ApiSiteType, ApiCropType and all conversion impls (no longer needed)

**Update OpenAPI schema in `crates/api/src/openapi.rs`:**
- Register new types in schema components
- Remove old enum types from schema

###  Step 5: Remove hardcoded enums and wire up
Remove all hardcoded enum definitions and wire everything together.

**STATUS: NOT STARTED** — The infrastructure (Steps 1-4) is complete and compiles successfully. However, Step 5 has NOT been executed yet:
- All ~55 enum definitions still exist in domain entity files
- All entity structs (Site, Order, Equipment, Animal, etc.) still use enum fields with `#[sqlx(json)]` serialization
- All DTOs (CreateSiteDto, UpdateSiteDto, etc.) still use enum types
- All PostgreSQL repositories still serialize/deserialize enums as JSONB
- API DTO enums (ApiSiteType, ApiCropType, IoTCapabilityType) still exist
- No entity uses UUID references to type_entries yet
- No repository queries use JOINs to type_entries for type name resolution

**What remains to be done:**
1. Update all ~25 entity structs to replace enum fields with UUID type references
2. Update all ~50 DTOs (Create/Update/Response) to use UUID type references
3. Update all PostgreSQL repository queries to use UUID joins instead of JSONB enum parsing
4. Remove all ~55 enum definitions from domain entities
5. Remove API DTO enum duplicates (ApiSiteType, ApiCropType, IoTCapabilityType)
6. Update OpenAPI schema registration
7. Add database migration for entity column changes (add type_id UUID columns, migrate data, drop old columns)
8. Final compilation verification with `cargo check`

**Remove enum definitions from domain entities:**
- `mod.rs`: Remove `SiteType`, `CropType`, `OrderType`, `OrderStatus`, `BbchStage` enum definitions and their FromStr impls
- `livestock.rs`: Remove `AnimalSpecies`, `AnimalStatus`, `LivestockType`
- `equipment.rs`: Remove `EquipmentType`
- `building.rs`: Remove `BuildingType`
- `group.rs`: Remove `GroupType`
- `tree.rs`: Remove `TreeType`
- `water.rs`: Remove `WaterSourceType`, `IrrigationMethod`
- `weather.rs`: Remove `WeatherStationType`
- `vineyard.rs`: Remove `DocArea`, `QualityGrade`
- `olive.rs`: Remove `OilGrade`
- `inventory.rs`: Remove `InventoryCategory`, `UnitOfMeasure`, `InventoryMethod`, `TransactionType`
- `compliance.rs`: Remove `ChecklistType`, `ComplianceStatus`, `AuditAction`
- `finance.rs`: Remove `PACStatus`, `CostCenterType`, `FinancialRecordType`
- `workforce.rs`: Remove `ContractType`, `ClockEntryType`
- `harvest.rs`: Remove `LotStatus`
- `plant_protection.rs`: Remove `PlantProtectionAreaMethod`
- `order.rs`: Remove `TaskExecutionMode`, `RecurrenceCadence`
- `task_state.rs`: Remove `SubTaskStatus`, `OverallTaskStatus`
- `worker_task_status.rs`: Remove `WorkerTaskStatusType`
- `variety.rs`: Remove `VarietyCategory` enum from DTOs. **Variety struct and varieties table stay intact** — only the category enum in CreateVarietyDto/UpdateVarietyDto is replaced with a `category_id: Uuid` reference to type_entries.
- `spatial/mod.rs`: Remove `SpatialObjectType`
- `breed.rs`: Remove `Species` enum from DTOs. **Breed struct and breeds table stay intact** — only the species enum in CreateBreedDto/UpdateBreedDto is replaced with a `species_id: Uuid` reference to type_entries.
- `setting.rs`: Remove `SettingValueType`
- `sync.rs`: Remove `SyncEntityType`, `SyncOperation`, `ConflictType`, `ConflictResolution`
- `tenant.rs`: Remove `Module` enum (if it's a type enum, keep if it's module feature flags)
- `user.rs`: Keep `UserRole`, `Resource`, `Action`, `PermissionScope` as these are auth-specific, not domain types
- `iot.rs` (API): Remove `IoTCapabilityType`
- `equipment.rs`: Remove `DepreciationMethod`

**Update sync system:**
- Add `TypeCategory` and `TypeEntry` to `SyncEntityType` (kept as a small internal enum since it's sync-specific)
- Or migrate SyncEntityType to use type_entries lookups

**Update `crates/api/src/openapi.rs`:**
- Remove all old enum types from schema registration
- Add new TypeCategoryDto, TypeEntryDto, TypeRef to schema

**Final integration:**
- Ensure all handlers compile with new DTO signatures
- Ensure all repository queries use UUID joins instead of enum parsing
- Update the `database.rs` to expose new repos
- Run `cargo check` to verify compilation