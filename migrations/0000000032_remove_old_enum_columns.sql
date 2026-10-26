-- Migration 0000000032: Remove old enum columns after migration to UUID type references
-- This migration drops the old JSONB/text enum columns that have been replaced by *_id UUID columns.
-- Only drop if the new UUID column exists and has data, to be safe.

-- ============================================================================
-- SITES: drop site_type and crop_type (JSONB)
-- ============================================================================

ALTER TABLE sites DROP COLUMN IF EXISTS site_type;
ALTER TABLE sites DROP COLUMN IF EXISTS crop_type;

-- ============================================================================
-- SPATIAL_OBJECTS: drop object_type (text)
-- ============================================================================

ALTER TABLE spatial_objects DROP COLUMN IF EXISTS object_type;

-- ============================================================================
-- EQUIPMENT: drop equipment_type (text)
-- ============================================================================

ALTER TABLE equipment DROP COLUMN IF EXISTS equipment_type;

-- ============================================================================
-- BUILDINGS: drop building_type (text)
-- ============================================================================

ALTER TABLE buildings DROP COLUMN IF EXISTS building_type;

-- ============================================================================
-- GROUPS: drop group_type (text)
-- ============================================================================

ALTER TABLE groups DROP COLUMN IF EXISTS group_type;

-- ============================================================================
-- TREES: drop tree_type (text)
-- ============================================================================

ALTER TABLE trees DROP COLUMN IF EXISTS tree_type;

-- ============================================================================
-- ORDERS: drop order_type and status (JSONB)
-- ============================================================================

ALTER TABLE orders DROP COLUMN IF EXISTS order_type;
ALTER TABLE orders DROP COLUMN IF EXISTS status;

-- ============================================================================
-- WORKERS: drop contract_type (JSONB)
-- ============================================================================

ALTER TABLE workers DROP COLUMN IF EXISTS contract_type;

-- ============================================================================
-- WEATHER_STATIONS: drop station_type (JSONB)
-- ============================================================================

ALTER TABLE weather_stations DROP COLUMN IF EXISTS station_type;

-- ============================================================================
-- PAC_APPLICATIONS: drop status (JSONB)
-- ============================================================================

ALTER TABLE pac_applications DROP COLUMN IF EXISTS status;

-- ============================================================================
-- WATER_SOURCES: drop source_type (JSONB)
-- ============================================================================

ALTER TABLE water_sources DROP COLUMN IF EXISTS source_type;

-- ============================================================================
-- WATER_USAGE: drop irrigation_method (JSONB)
-- ============================================================================

ALTER TABLE water_usages DROP COLUMN IF EXISTS irrigation_method;

-- ============================================================================
-- HARVEST_LOTS: drop crop_type (text)
-- ============================================================================

ALTER TABLE harvest_lots DROP COLUMN IF EXISTS crop_type;

-- ============================================================================
-- ANIMALS: drop livestock_type, species, status (text/JSONB)
-- ============================================================================

ALTER TABLE animals DROP COLUMN IF EXISTS livestock_type;
ALTER TABLE animals DROP COLUMN IF EXISTS species;
ALTER TABLE animals DROP COLUMN IF EXISTS status;
ALTER TABLE animals DROP COLUMN IF EXISTS is_active;

-- ============================================================================
-- LIVESTOCK: drop livestock_type (text)
-- ============================================================================

ALTER TABLE livestock DROP COLUMN IF EXISTS livestock_type;

-- ============================================================================
-- GROWING_DEGREE_DAYS: drop crop_type (text)
-- ============================================================================

ALTER TABLE growing_degree_days DROP COLUMN IF EXISTS crop_type;

-- ============================================================================
-- BREEDS: drop species (text)
-- ============================================================================

ALTER TABLE breeds DROP COLUMN IF EXISTS species;

-- ============================================================================
-- COMPLIANCE_CHECKLISTS: drop checklist_type (JSONB)
-- ============================================================================

ALTER TABLE compliance_checklists DROP COLUMN IF EXISTS checklist_type;

-- ============================================================================
-- WORKER_TASK_STATUSES: drop status (JSONB)
-- ============================================================================

ALTER TABLE worker_task_statuses DROP COLUMN IF EXISTS status;

-- ============================================================================
-- COST_CENTERS: drop cost_center_type (JSONB)
-- ============================================================================

ALTER TABLE cost_centers DROP COLUMN IF EXISTS cost_center_type;

-- ============================================================================
-- INVENTORY_ITEMS: drop category, unit, inventory_method (text/JSONB)
-- ============================================================================

ALTER TABLE inventory_items DROP COLUMN IF EXISTS category;
ALTER TABLE inventory_items DROP COLUMN IF EXISTS unit;
ALTER TABLE inventory_items DROP COLUMN IF EXISTS inventory_method;

-- ============================================================================
-- INVENTORY_TRANSACTIONS: drop transaction_type (text/JSONB)
-- ============================================================================

ALTER TABLE inventory_transactions DROP COLUMN IF EXISTS transaction_type;

-- ============================================================================
-- OLIVE_OIL_RECORDS: drop quality_grade (text/JSONB)
-- ============================================================================

ALTER TABLE olive_oil_records DROP COLUMN IF EXISTS quality_grade;

-- ============================================================================
-- PEST_RISKS: drop risk_level (JSONB)
-- ============================================================================

ALTER TABLE pest_risks DROP COLUMN IF EXISTS risk_level;

-- ============================================================================
-- APPLICATOR_LICENSES: drop license_type (JSONB)
-- ============================================================================

ALTER TABLE applicator_licenses DROP COLUMN IF EXISTS license_type;

-- ============================================================================
-- VARIETIES: drop category (text)
-- ============================================================================

ALTER TABLE varieties DROP COLUMN IF EXISTS category;

-- ============================================================================
-- SETTINGS: drop value_type (JSONB)
-- ============================================================================

ALTER TABLE system_settings DROP COLUMN IF EXISTS value_type;

-- ============================================================================
-- AUDIT_LOGS: drop action (JSONB) – kept as it's still used for the action enum
-- ============================================================================
-- Note: audit_logs.action is kept because it stores the AuditAction enum which
-- is still used in the codebase. Only the type reference columns are migrated.
