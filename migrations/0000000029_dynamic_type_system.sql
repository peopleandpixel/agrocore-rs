-- Migration 29: Dynamic Type System
--
-- Replaces all ~55 hardcoded enum types with a database-driven dynamic type
-- classification system. Types are stored in two generic tables:
-- `type_categories` (metadata) and `type_entries` (all types).
--
-- Key design decisions:
-- 1. Two-table generic design: maximum flexibility, single source of truth.
-- 2. UUID references everywhere: entities reference types via UUID foreign keys.
-- 3. Global + tenant types: tenant_id NULL = global (system-provided), NOT NULL = tenant-specific.
-- 4. Hierarchy via parent_id: self-referencing FK for parent-child relationships.
-- 5. Soft delete: is_active flag instead of hard delete.
-- 6. Multi-level type references: entities can reference any node (root, intermediate, leaf).
--
-- Existing reference catalogues (varieties, breeds, crop_species) are preserved as-is.
-- Only the enum types used in their DTOs (VarietyCategory, Species) are migrated.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ---------------------------------------------------------------------------
-- 1. Type Categories Table
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS type_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    label VARCHAR(200) NOT NULL,
    description TEXT,
    is_hierarchical BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE type_categories IS
'Defines type classification categories (e.g., animal_species, site_type, order_type). Each category contains type entries that replace hardcoded enums.';

COMMENT ON COLUMN type_categories.name IS
'Slug identifier for the category, e.g., animal_species, site_type, order_type';

COMMENT ON COLUMN type_categories.label IS
'Human-readable display name, e.g., Animal Species, Site Type, Order Type';

COMMENT ON COLUMN type_categories.is_hierarchical IS
'Whether entries in this category support parent-child hierarchy relationships';

-- ---------------------------------------------------------------------------
-- 2. Type Entries Table
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS type_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id UUID NOT NULL REFERENCES type_categories(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES type_entries(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    code VARCHAR(100) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(category_id, code, tenant_id)
);

COMMENT ON TABLE type_entries IS
'All type entries across all categories. tenant_id NULL means global (system-provided), visible to all tenants. Non-NULL means tenant-specific.';

COMMENT ON COLUMN type_entries.category_id IS
'Reference to the type category this entry belongs to';

COMMENT ON COLUMN type_entries.parent_id IS
'Self-referencing FK for hierarchy. NULL = root entry. Enables parent-child relationships (e.g., Geflügel → Huhn, Ente, Gans)';

COMMENT ON COLUMN type_entries.tenant_id IS
'NULL = global type visible to all tenants. Non-NULL = tenant-specific type';

COMMENT ON COLUMN type_entries.name IS
'Human-readable display name, e.g., Chicken, Vineyard, Tractor';

COMMENT ON COLUMN type_entries.code IS
'Slug identifier, e.g., chicken, vineyard, tractor. Used for backward compatibility with enum serialization';

COMMENT ON COLUMN type_entries.is_active IS
'Soft delete flag. FALSE means logically deleted but preserved for referential integrity';

-- Indexes for common query patterns

CREATE INDEX IF NOT EXISTS idx_type_entries_category ON type_entries(category_id);
CREATE INDEX IF NOT EXISTS idx_type_entries_parent ON type_entries(parent_id);
CREATE INDEX IF NOT EXISTS idx_type_entries_tenant ON type_entries(tenant_id);
CREATE INDEX IF NOT EXISTS idx_type_entries_code ON type_entries(code);
CREATE INDEX IF NOT EXISTS idx_type_entries_active ON type_entries(category_id, is_active) WHERE is_active;
CREATE INDEX IF NOT EXISTS idx_type_entries_tenant_category ON type_entries(tenant_id, category_id) WHERE is_active;

-- ---------------------------------------------------------------------------
-- 3. Seed Type Categories
-- ---------------------------------------------------------------------------

-- Helper function to get category ID by name
CREATE OR REPLACE FUNCTION get_type_category_id(category_name VARCHAR)
RETURNS UUID AS $$
DECLARE
    cat_id UUID;
BEGIN
    SELECT id INTO cat_id FROM type_categories WHERE name = category_name;
    RETURN cat_id;
END;
$$ LANGUAGE plpgsql;

-- Helper function to get type entry ID by category, code, and tenant
CREATE OR REPLACE FUNCTION get_type_entry_id(category_name VARCHAR, code VARCHAR, tenant_id UUID DEFAULT NULL)
RETURNS UUID AS $$
DECLARE
    cat_id UUID;
    entry_id UUID;
BEGIN
    cat_id := get_type_category_id(category_name);
    SELECT id INTO entry_id FROM type_entries WHERE category_id = cat_id AND code = code AND tenant_id IS NULL;
    RETURN entry_id;
END;
$$ LANGUAGE plpgsql;

-- Insert all categories
INSERT INTO type_categories (name, label, description, is_hierarchical) VALUES
('animal_species', 'Animal Species', 'Species of livestock animals', true),
('site_type', 'Site Type', 'Types of agricultural sites and areas', true),
('crop_type', 'Crop Type', 'Types of crops grown on sites', false),
('order_type', 'Order Type', 'Types of farm orders and tasks', true),
('order_status', 'Order Status', 'Status of orders', false),
('equipment_type', 'Equipment Type', 'Types of farm equipment', true),
('building_type', 'Building Type', 'Types of farm buildings', true),
('group_type', 'Group Type', 'Types of animal or plant groups', false),
('tree_type', 'Tree Type', 'Types of trees', true),
('water_source_type', 'Water Source Type', 'Types of water sources', false),
('irrigation_method', 'Irrigation Method', 'Methods of irrigation', false),
('animal_status', 'Animal Status', 'Status of individual animals', false),
('weather_station_type', 'Weather Station Type', 'Types of weather stations', false),
('inventory_category', 'Inventory Category', 'Categories of inventory items', false),
('unit_of_measure', 'Unit of Measure', 'Units for measuring quantities', false),
('oil_grade', 'Oil Grade', 'Grades of olive oil', false),
('quality_grade', 'Quality Grade', 'Quality grades for wine', false),
('doc_area', 'DOC Area', 'Portuguese wine designation of origin areas', false),
('bbch_stage', 'BBCH Stage', 'BBCH growth stages for grapes', false),
('spatial_object_type', 'Spatial Object Type', 'Types of spatial objects on the farm', true),
('variety_category', 'Variety Category', 'Categories of plant varieties', false),
('species', 'Species', 'Animal species for breed catalogue', false),
('checklist_type', 'Checklist Type', 'Types of compliance checklists', false),
('compliance_status', 'Compliance Status', 'Status of compliance items', false),
('audit_action', 'Audit Action', 'Types of audit log actions', false),
('pac_status', 'PAC Status', 'Status of PAC (EU subsidy) applications', false),
('cost_center_type', 'Cost Center Type', 'Types of cost centers', false),
('financial_record_type', 'Financial Record Type', 'Types of financial records', false),
('contract_type', 'Contract Type', 'Types of worker contracts', false),
('clock_entry_type', 'Clock Entry Type', 'Types of clock entries', false),
('lot_status', 'Lot Status', 'Status of harvest lots', false),
('plant_protection_area_method', 'Plant Protection Area Method', 'Methods for calculating treatment area', false),
('task_execution_mode', 'Task Execution Mode', 'Modes for task execution', false),
('recurrence_cadence', 'Recurrence Cadence', 'Cadence for recurring tasks', false),
('sub_task_status', 'Sub-Task Status', 'Status of sub-tasks', false),
('overall_task_status', 'Overall Task Status', 'Overall status of tasks', false),
('depreciation_method', 'Depreciation Method', 'Methods for equipment depreciation', false),
('inventory_method', 'Inventory Method', 'Methods for inventory valuation', false),
('transaction_type', 'Transaction Type', 'Types of inventory transactions', false),
('risk_level', 'Risk Level', 'Levels of risk', false),
('iot_device_type', 'IoT Device Type', 'Types of IoT devices', false),
('setting_value_type', 'Setting Value Type', 'Data types for system settings', false),
('sync_entity_type', 'Sync Entity Type', 'Entity types for sync system', false),
('sync_operation', 'Sync Operation', 'Types of sync operations', false),
('conflict_type', 'Conflict Type', 'Types of sync conflicts', false),
('conflict_resolution', 'Conflict Resolution', 'Methods for resolving sync conflicts', false),
('worker_task_status_type', 'Worker Task Status Type', 'Status types for worker tasks', false),
('livestock_type', 'Livestock Type', 'Types of livestock (alias for animal_species)', false),
('transaction_type_inventory', 'Inventory Transaction Type', 'Types of inventory stock transactions', false)
ON CONFLICT (name) DO NOTHING;

-- ---------------------------------------------------------------------------
-- 4. Seed Type Entries
-- ---------------------------------------------------------------------------

-- Helper to insert type entries and return their IDs for hierarchy building
-- We use a DO block to seed all entries with proper parent-child relationships

DO $$
DECLARE
    cat_animal_species UUID;
    cat_site_type UUID;
    cat_crop_type UUID;
    cat_order_type UUID;
    cat_order_status UUID;
    cat_equipment_type UUID;
    cat_building_type UUID;
    cat_group_type UUID;
    cat_tree_type UUID;
    cat_water_source_type UUID;
    cat_irrigation_method UUID;
    cat_animal_status UUID;
    cat_weather_station_type UUID;
    cat_inventory_category UUID;
    cat_unit_of_measure UUID;
    cat_oil_grade UUID;
    cat_quality_grade UUID;
    cat_doc_area UUID;
    cat_bbch_stage UUID;
    cat_spatial_object_type UUID;
    cat_variety_category UUID;
    cat_species UUID;
    cat_checklist_type UUID;
    cat_compliance_status UUID;
    cat_audit_action UUID;
    cat_pac_status UUID;
    cat_cost_center_type UUID;
    cat_financial_record_type UUID;
    cat_contract_type UUID;
    cat_clock_entry_type UUID;
    cat_lot_status UUID;
    cat_plant_protection_area_method UUID;
    cat_task_execution_mode UUID;
    cat_recurrence_cadence UUID;
    cat_sub_task_status UUID;
    cat_overall_task_status UUID;
    cat_depreciation_method UUID;
    cat_inventory_method UUID;
    cat_transaction_type UUID;
    cat_risk_level UUID;
    cat_iot_device_type UUID;
    cat_setting_value_type UUID;
    cat_sync_entity_type UUID;
    cat_sync_operation UUID;
    cat_conflict_type UUID;
    cat_conflict_resolution UUID;
    cat_worker_task_status_type UUID;
    cat_livestock_type UUID;

    -- Hierarchy parent IDs for animal_species
    parent_gefluegel UUID;

    -- Hierarchy parent IDs for spatial_object_type
    parent_farm UUID;
    parent_site UUID;
    parent_building UUID;
    parent_tree UUID;

    -- Hierarchy parent IDs for equipment_type
    parent_tractor UUID;
    parent_sprayer UUID;
    parent_harvester UUID;

    -- Hierarchy parent IDs for building_type
    parent_chicken_coop UUID;
    parent_goat_stable UUID;
    parent_barn UUID;

    -- Hierarchy parent IDs for tree_type
    parent_cork_oak UUID;
    parent_olive UUID;
    parent_almond UUID;

BEGIN

    -- Get all category IDs
    SELECT id INTO cat_animal_species FROM type_categories WHERE name = 'animal_species';
    SELECT id INTO cat_site_type FROM type_categories WHERE name = 'site_type';
    SELECT id INTO cat_crop_type FROM type_categories WHERE name = 'crop_type';
    SELECT id INTO cat_order_type FROM type_categories WHERE name = 'order_type';
    SELECT id INTO cat_order_status FROM type_categories WHERE name = 'order_status';
    SELECT id INTO cat_equipment_type FROM type_categories WHERE name = 'equipment_type';
    SELECT id INTO cat_building_type FROM type_categories WHERE name = 'building_type';
    SELECT id INTO cat_group_type FROM type_categories WHERE name = 'group_type';
    SELECT id INTO cat_tree_type FROM type_categories WHERE name = 'tree_type';
    SELECT id INTO cat_water_source_type FROM type_categories WHERE name = 'water_source_type';
    SELECT id INTO cat_irrigation_method FROM type_categories WHERE name = 'irrigation_method';
    SELECT id INTO cat_animal_status FROM type_categories WHERE name = 'animal_status';
    SELECT id INTO cat_weather_station_type FROM type_categories WHERE name = 'weather_station_type';
    SELECT id INTO cat_inventory_category FROM type_categories WHERE name = 'inventory_category';
    SELECT id INTO cat_unit_of_measure FROM type_categories WHERE name = 'unit_of_measure';
    SELECT id INTO cat_oil_grade FROM type_categories WHERE name = 'oil_grade';
    SELECT id INTO cat_quality_grade FROM type_categories WHERE name = 'quality_grade';
    SELECT id INTO cat_doc_area FROM type_categories WHERE name = 'doc_area';
    SELECT id INTO cat_bbch_stage FROM type_categories WHERE name = 'bbch_stage';
    SELECT id INTO cat_spatial_object_type FROM type_categories WHERE name = 'spatial_object_type';
    SELECT id INTO cat_variety_category FROM type_categories WHERE name = 'variety_category';
    SELECT id INTO cat_species FROM type_categories WHERE name = 'species';
    SELECT id INTO cat_checklist_type FROM type_categories WHERE name = 'checklist_type';
    SELECT id INTO cat_compliance_status FROM type_categories WHERE name = 'compliance_status';
    SELECT id INTO cat_audit_action FROM type_categories WHERE name = 'audit_action';
    SELECT id INTO cat_pac_status FROM type_categories WHERE name = 'pac_status';
    SELECT id INTO cat_cost_center_type FROM type_categories WHERE name = 'cost_center_type';
    SELECT id INTO cat_financial_record_type FROM type_categories WHERE name = 'financial_record_type';
    SELECT id INTO cat_contract_type FROM type_categories WHERE name = 'contract_type';
    SELECT id INTO cat_clock_entry_type FROM type_categories WHERE name = 'clock_entry_type';
    SELECT id INTO cat_lot_status FROM type_categories WHERE name = 'lot_status';
    SELECT id INTO cat_plant_protection_area_method FROM type_categories WHERE name = 'plant_protection_area_method';
    SELECT id INTO cat_task_execution_mode FROM type_categories WHERE name = 'task_execution_mode';
    SELECT id INTO cat_recurrence_cadence FROM type_categories WHERE name = 'recurrence_cadence';
    SELECT id INTO cat_sub_task_status FROM type_categories WHERE name = 'sub_task_status';
    SELECT id INTO cat_overall_task_status FROM type_categories WHERE name = 'overall_task_status';
    SELECT id INTO cat_depreciation_method FROM type_categories WHERE name = 'depreciation_method';
    SELECT id INTO cat_inventory_method FROM type_categories WHERE name = 'inventory_method';
    SELECT id INTO cat_transaction_type FROM type_categories WHERE name = 'transaction_type';
    SELECT id INTO cat_risk_level FROM type_categories WHERE name = 'risk_level';
    SELECT id INTO cat_iot_device_type FROM type_categories WHERE name = 'iot_device_type';
    SELECT id INTO cat_setting_value_type FROM type_categories WHERE name = 'setting_value_type';
    SELECT id INTO cat_sync_entity_type FROM type_categories WHERE name = 'sync_entity_type';
    SELECT id INTO cat_sync_operation FROM type_categories WHERE name = 'sync_operation';
    SELECT id INTO cat_conflict_type FROM type_categories WHERE name = 'conflict_type';
    SELECT id INTO cat_conflict_resolution FROM type_categories WHERE name = 'conflict_resolution';
    SELECT id INTO cat_worker_task_status_type FROM type_categories WHERE name = 'worker_task_status_type';
    SELECT id INTO cat_livestock_type FROM type_categories WHERE name = 'livestock_type';

    -- =====================================================================
    -- Animal Species (with Geflügel hierarchy)
    -- =====================================================================

    -- Insert Geflügel as parent first
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_animal_species, NULL, NULL, 'Geflügel', 'gefluegel', 10)
    RETURNING id INTO parent_gefluegel;

    -- Insert individual species
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_species, NULL, NULL, 'Goat', 'goat', 1),
    (cat_animal_species, NULL, NULL, 'Sheep', 'sheep', 2),
    (cat_animal_species, NULL, NULL, 'Cattle', 'cattle', 3),
    (cat_animal_species, NULL, NULL, 'Pig', 'pig', 4),
    (cat_animal_species, NULL, NULL, 'Horse', 'horse', 5),
    -- Geflügel children
    (cat_animal_species, parent_gefluegel, NULL, 'Chicken', 'chicken', 11),
    (cat_animal_species, parent_gefluegel, NULL, 'Duck', 'duck', 12),
    (cat_animal_species, parent_gefluegel, NULL, 'Turkey', 'turkey', 13),
    (cat_animal_species, parent_gefluegel, NULL, 'Goose', 'goose', 14);

    -- =====================================================================
    -- Site Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_site_type, NULL, NULL, 'Vineyard', 'vineyard', 1),
    (cat_site_type, NULL, NULL, 'Field', 'field', 2),
    (cat_site_type, NULL, NULL, 'Cork Oak Montado', 'corkOakMontado', 3),
    (cat_site_type, NULL, NULL, 'Holm Oak Montado', 'holmOakMontado', 4),
    (cat_site_type, NULL, NULL, 'Olive Grove', 'oliveGrove', 5),
    (cat_site_type, NULL, NULL, 'Orchard', 'orchard', 6),
    (cat_site_type, NULL, NULL, 'Almond Orchard', 'almondOrchard', 7),
    (cat_site_type, NULL, NULL, 'Citrus Grove', 'citrusGrove', 8),
    (cat_site_type, NULL, NULL, 'Pasture', 'pasture', 9),
    (cat_site_type, NULL, NULL, 'Greenhouse', 'greenhouse', 10);

    -- =====================================================================
    -- Crop Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_crop_type, NULL, NULL, 'Grape', 'grape', 1),
    (cat_crop_type, NULL, NULL, 'Olive', 'olive', 2),
    (cat_crop_type, NULL, NULL, 'Apple', 'apple', 3),
    (cat_crop_type, NULL, NULL, 'Citrus', 'citrus', 4),
    (cat_crop_type, NULL, NULL, 'Nut', 'nut', 5),
    (cat_crop_type, NULL, NULL, 'Cork Oak', 'corkOak', 6),
    (cat_crop_type, NULL, NULL, 'Almond', 'almond', 7),
    (cat_crop_type, NULL, NULL, 'Hazelnut', 'hazelnut', 8),
    (cat_crop_type, NULL, NULL, 'Chestnut', 'chestnut', 9),
    (cat_crop_type, NULL, NULL, 'Berry', 'berry', 10),
    (cat_crop_type, NULL, NULL, 'Tropical', 'tropical', 11),
    (cat_crop_type, NULL, NULL, 'Poultry', 'poultry', 12),
    (cat_crop_type, NULL, NULL, 'Livestock', 'livestock', 13),
    (cat_crop_type, NULL, NULL, 'Fallow', 'fallow', 14),
    (cat_crop_type, NULL, NULL, 'Forest', 'forest', 15),
    (cat_crop_type, NULL, NULL, 'Pasture', 'pasture', 16),
    (cat_crop_type, NULL, NULL, 'Grain', 'grain', 17),
    (cat_crop_type, NULL, NULL, 'Unknown', 'unknown', 18);

    -- =====================================================================
    -- Order Types (with hierarchy for livestock-related orders)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_order_type, NULL, NULL, 'Plant Protection', 'plantProtection', 1),
    (cat_order_type, NULL, NULL, 'Fertilization', 'fertilization', 2),
    (cat_order_type, NULL, NULL, 'Pruning', 'pruning', 3),
    (cat_order_type, NULL, NULL, 'Harvest', 'harvest', 4),
    (cat_order_type, NULL, NULL, 'Soil Work', 'soilWork', 5),
    (cat_order_type, NULL, NULL, 'Irrigation', 'irrigation', 6),
    (cat_order_type, NULL, NULL, 'Monitoring', 'monitoring', 7),
    (cat_order_type, NULL, NULL, 'Livestock Feeding', 'livestockFeeding', 8),
    (cat_order_type, NULL, NULL, 'Livestock Watering', 'livestockWatering', 9),
    (cat_order_type, NULL, NULL, 'Livestock Relocation', 'livestockRelocation', 10),
    (cat_order_type, NULL, NULL, 'Livestock Health Check', 'livestockHealthCheck', 11),
    (cat_order_type, NULL, NULL, 'Barn Cleaning', 'barnCleaning', 12),
    (cat_order_type, NULL, NULL, 'Egg Collection', 'eggCollection', 13),
    (cat_order_type, NULL, NULL, 'Shearing', 'shearing', 14),
    (cat_order_type, NULL, NULL, 'Milking', 'milking', 15),
    (cat_order_type, NULL, NULL, 'Sales Order', 'salesOrder', 16);

    -- =====================================================================
    -- Order Status
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_order_status, NULL, NULL, 'Draft', 'draft', 1),
    (cat_order_status, NULL, NULL, 'Planned', 'planned', 2),
    (cat_order_status, NULL, NULL, 'In Progress', 'inProgress', 3),
    (cat_order_status, NULL, NULL, 'Completed', 'completed', 4),
    (cat_order_status, NULL, NULL, 'Cancelled', 'cancelled', 5);

    -- =====================================================================
    -- Equipment Types (with hierarchy)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_equipment_type, NULL, NULL, 'Tractor', 'tractor', 1),
    (cat_equipment_type, NULL, NULL, 'Sprayer', 'sprayer', 2),
    (cat_equipment_type, NULL, NULL, 'Harvester', 'harvester', 3),
    (cat_equipment_type, NULL, NULL, 'Mulcher', 'mulcher', 4),
    (cat_equipment_type, NULL, NULL, 'Plow', 'plow', 5),
    (cat_equipment_type, NULL, NULL, 'Trailer', 'trailer', 6),
    (cat_equipment_type, NULL, NULL, 'Tool', 'tool', 7);

    -- =====================================================================
    -- Building Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_building_type, NULL, NULL, 'Chicken Coop', 'chickenCoop', 1),
    (cat_building_type, NULL, NULL, 'Goat Stable', 'goatStable', 2),
    (cat_building_type, NULL, NULL, 'Barn', 'barn', 3);

    -- =====================================================================
    -- Group Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_group_type, NULL, NULL, 'Herd', 'herd', 1),
    (cat_group_type, NULL, NULL, 'Flock', 'flock', 2),
    (cat_group_type, NULL, NULL, 'Grove', 'grove', 3),
    (cat_group_type, NULL, NULL, 'Coop', 'coop', 4),
    (cat_group_type, NULL, NULL, 'Barn', 'barn', 5);

    -- =====================================================================
    -- Tree Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_tree_type, NULL, NULL, 'Cork Oak', 'corkOak', 1),
    (cat_tree_type, NULL, NULL, 'Olive', 'olive', 2),
    (cat_tree_type, NULL, NULL, 'Almond', 'almond', 3);

    -- =====================================================================
    -- Water Source Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_water_source_type, NULL, NULL, 'Well', 'well', 1),
    (cat_water_source_type, NULL, NULL, 'Reservoir', 'reservoir', 2),
    (cat_water_source_type, NULL, NULL, 'River', 'river', 3),
    (cat_water_source_type, NULL, NULL, 'Canal', 'canal', 4),
    (cat_water_source_type, NULL, NULL, 'Comunidad De Regantes', 'comunidadDeRegantes', 5),
    (cat_water_source_type, NULL, NULL, 'Rainwater Harvesting', 'rainwaterHarvesting', 6),
    (cat_water_source_type, NULL, NULL, 'Desalination', 'desalination', 7);

    -- =====================================================================
    -- Irrigation Methods
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_irrigation_method, NULL, NULL, 'Drip', 'drip', 1),
    (cat_irrigation_method, NULL, NULL, 'Sprinkler', 'sprinkler', 2),
    (cat_irrigation_method, NULL, NULL, 'Flood', 'flood', 3),
    (cat_irrigation_method, NULL, NULL, 'Pivot', 'pivot', 4),
    (cat_irrigation_method, NULL, NULL, 'Micro Sprinkler', 'microSprinkler', 5),
    (cat_irrigation_method, NULL, NULL, 'Subsurface', 'subsurface', 6);

    -- =====================================================================
    -- Animal Status
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_status, NULL, NULL, 'Active', 'active', 1),
    (cat_animal_status, NULL, NULL, 'Inactive', 'inactive', 2),
    (cat_animal_status, NULL, NULL, 'Sold', 'sold', 3),
    (cat_animal_status, NULL, NULL, 'Deceased', 'deceased', 4);

    -- =====================================================================
    -- Weather Station Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_weather_station_type, NULL, NULL, 'IoT', 'iot', 1),
    (cat_weather_station_type, NULL, NULL, 'Manual', 'manual', 2),
    (cat_weather_station_type, NULL, NULL, 'Virtual', 'virtual', 3),
    (cat_weather_station_type, NULL, NULL, 'External API', 'externalApi', 4);

    -- =====================================================================
    -- Inventory Categories
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_inventory_category, NULL, NULL, 'Seed', 'seed', 1),
    (cat_inventory_category, NULL, NULL, 'Fertilizer', 'fertilizer', 2),
    (cat_inventory_category, NULL, NULL, 'Pesticide', 'pesticide', 3),
    (cat_inventory_category, NULL, NULL, 'Herbicide', 'herbicide', 4),
    (cat_inventory_category, NULL, NULL, 'Fuel', 'fuel', 5),
    (cat_inventory_category, NULL, NULL, 'Feed', 'feed', 6),
    (cat_inventory_category, NULL, NULL, 'Medicine', 'medicine', 7),
    (cat_inventory_category, NULL, NULL, 'Spare Part', 'sparePart', 8),
    (cat_inventory_category, NULL, NULL, 'Equipment', 'equipment', 9);

    -- =====================================================================
    -- Units of Measure
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_unit_of_measure, NULL, NULL, 'Kilogram', 'kg', 1),
    (cat_unit_of_measure, NULL, NULL, 'Liter', 'liter', 2),
    (cat_unit_of_measure, NULL, NULL, 'Bag', 'bag', 3),
    (cat_unit_of_measure, NULL, NULL, 'Unit', 'unit', 4),
    (cat_unit_of_measure, NULL, NULL, 'Box', 'box', 5),
    (cat_unit_of_measure, NULL, NULL, 'Bale', 'bale', 6);

    -- =====================================================================
    -- Oil Grades
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_oil_grade, NULL, NULL, 'Extra Virgin', 'extraVirgin', 1),
    (cat_oil_grade, NULL, NULL, 'Virgin', 'virgin', 2),
    (cat_oil_grade, NULL, NULL, 'Lampante', 'lampante', 3),
    (cat_oil_grade, NULL, NULL, 'Refined', 'refined', 4),
    (cat_oil_grade, NULL, NULL, 'Pomace', 'pomace', 5);

    -- =====================================================================
    -- Quality Grades (Wine)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_quality_grade, NULL, NULL, 'Reserva', 'reserva', 1),
    (cat_quality_grade, NULL, NULL, 'Grande Reserva', 'grandeReserva', 2),
    (cat_quality_grade, NULL, NULL, 'Garrafeira', 'garrafeira', 3),
    (cat_quality_grade, NULL, NULL, 'Superior', 'superior', 4),
    (cat_quality_grade, NULL, NULL, 'Classic', 'classic', 5),
    (cat_quality_grade, NULL, NULL, 'Late Harvest', 'lateHarvest', 6);

    -- =====================================================================
    -- DOC Areas (Portuguese Wine Regions)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_doc_area, NULL, NULL, 'Douro', 'douro', 1),
    (cat_doc_area, NULL, NULL, 'Alentejo', 'alentejo', 2),
    (cat_doc_area, NULL, NULL, 'Vinho Verde', 'vinhoVerde', 3),
    (cat_doc_area, NULL, NULL, 'Dão', 'dao', 4),
    (cat_doc_area, NULL, NULL, 'Bairrada', 'bairrada', 5),
    (cat_doc_area, NULL, NULL, 'Lisboa', 'lisboa', 6),
    (cat_doc_area, NULL, NULL, 'Península de Setúbal', 'peninsulaDeSetubal', 7),
    (cat_doc_area, NULL, NULL, 'Algarve', 'algarve', 8),
    (cat_doc_area, NULL, NULL, 'Tejo', 'tejo', 9),
    (cat_doc_area, NULL, NULL, 'Minho', 'minho', 10),
    (cat_doc_area, NULL, NULL, 'Trasmontes', 'trasmontes', 11);

    -- =====================================================================
    -- BBCH Stages (Grape Growth Stages)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_bbch_stage, NULL, NULL, 'Dormancy', 'dormancy', 0),
    (cat_bbch_stage, NULL, NULL, 'Bud Swelling', 'budSwelling', 1),
    (cat_bbch_stage, NULL, NULL, 'Wool Stage', 'woolStage', 5),
    (cat_bbch_stage, NULL, NULL, 'Bud Break', 'budBreak', 9),
    (cat_bbch_stage, NULL, NULL, 'First Leaf', 'firstLeaf', 11),
    (cat_bbch_stage, NULL, NULL, 'Leaf Development', 'leafDevelopment', 15),
    (cat_bbch_stage, NULL, NULL, 'Inflorescence Visible', 'inflorescenceVisible', 53),
    (cat_bbch_stage, NULL, NULL, 'Inflorescence Fully Developed', 'inflorescenceFullyDeveloped', 57),
    (cat_bbch_stage, NULL, NULL, 'Flowering Begins', 'floweringBegins', 61),
    (cat_bbch_stage, NULL, NULL, 'Flowering Ends', 'floweringEnds', 69),
    (cat_bbch_stage, NULL, NULL, 'Fruit Set', 'fruitSet', 71),
    (cat_bbch_stage, NULL, NULL, 'Pea Size Berries', 'peaSizeBerries', 75),
    (cat_bbch_stage, NULL, NULL, 'Berries Touching', 'berriesTouching', 77),
    (cat_bbch_stage, NULL, NULL, 'Veraison Begins', 'veraisonBegins', 81),
    (cat_bbch_stage, NULL, NULL, 'Berries Ripe', 'berriesRipe', 89),
    (cat_bbch_stage, NULL, NULL, 'After Harvest', 'afterHarvest', 91),
    (cat_bbch_stage, NULL, NULL, 'Winter Dormancy', 'winterDormancy', 97);

    -- =====================================================================
    -- Spatial Object Types (with hierarchy)
    -- =====================================================================

    -- Farm (root)
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_spatial_object_type, NULL, NULL, 'Farm', 'farm', 1)
    RETURNING id INTO parent_farm;

    -- Site (child of Farm)
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_spatial_object_type, parent_farm, NULL, 'Site', 'site', 2)
    RETURNING id INTO parent_site;

    -- Field & Pasture (children of Site)
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_spatial_object_type, parent_site, NULL, 'Field', 'field', 3),
    (cat_spatial_object_type, parent_site, NULL, 'Pasture', 'pasture', 4);

    -- Building (root)
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_spatial_object_type, NULL, NULL, 'Building', 'building', 10)
    RETURNING id INTO parent_building;

    -- Barn, Stable, Shed, Pen (children of Building)
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_spatial_object_type, parent_building, NULL, 'Barn', 'barn', 11),
    (cat_spatial_object_type, parent_building, NULL, 'Stable', 'stable', 12),
    (cat_spatial_object_type, parent_building, NULL, 'Shed', 'shed', 13),
    (cat_spatial_object_type, parent_building, NULL, 'Pen', 'pen', 14);

    -- Tree (root)
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_spatial_object_type, NULL, NULL, 'Tree', 'tree', 20)
    RETURNING id INTO parent_tree;

    -- CorkOak & OliveTree (children of Tree)
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_spatial_object_type, parent_tree, NULL, 'Cork Oak', 'corkOak', 21),
    (cat_spatial_object_type, parent_tree, NULL, 'Olive Tree', 'oliveTree', 22);

    -- Standalone roots
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_spatial_object_type, NULL, NULL, 'Water Point', 'waterPoint', 30),
    (cat_spatial_object_type, NULL, NULL, 'Fence', 'fence', 40);

    -- =====================================================================
    -- Variety Categories
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_variety_category, NULL, NULL, 'Grape', 'grape', 1),
    (cat_variety_category, NULL, NULL, 'Olive', 'olive', 2),
    (cat_variety_category, NULL, NULL, 'Apple', 'apple', 3),
    (cat_variety_category, NULL, NULL, 'Citrus', 'citrus', 4),
    (cat_variety_category, NULL, NULL, 'Nut', 'nut', 5),
    (cat_variety_category, NULL, NULL, 'Berry', 'berry', 6),
    (cat_variety_category, NULL, NULL, 'Vegetable', 'vegetable', 7),
    (cat_variety_category, NULL, NULL, 'Grain', 'grain', 8);

    -- =====================================================================
    -- Species (for breed catalogue)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_species, NULL, NULL, 'Goat', 'goat', 1),
    (cat_species, NULL, NULL, 'Sheep', 'sheep', 2),
    (cat_species, NULL, NULL, 'Cattle', 'cattle', 3),
    (cat_species, NULL, NULL, 'Chicken', 'chicken', 4),
    (cat_species, NULL, NULL, 'Pig', 'pig', 5),
    (cat_species, NULL, NULL, 'Horse', 'horse', 6);

    -- =====================================================================
    -- Checklist Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_checklist_type, NULL, NULL, 'GAP', 'gap', 1),
    (cat_checklist_type, NULL, NULL, 'Organic', 'organic', 2),
    (cat_checklist_type, NULL, NULL, 'GlobalGAP', 'globalGAP', 3),
    (cat_checklist_type, NULL, NULL, 'HACCP', 'haccp', 4);

    -- =====================================================================
    -- Compliance Status
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_compliance_status, NULL, NULL, 'Pending', 'pending', 1),
    (cat_compliance_status, NULL, NULL, 'In Progress', 'inProgress', 2),
    (cat_compliance_status, NULL, NULL, 'Completed', 'completed', 3),
    (cat_compliance_status, NULL, NULL, 'Failed', 'failed', 4),
    (cat_compliance_status, NULL, NULL, 'Expired', 'expired', 5);

    -- =====================================================================
    -- Audit Actions
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_audit_action, NULL, NULL, 'Created', 'created', 1),
    (cat_audit_action, NULL, NULL, 'Updated', 'updated', 2),
    (cat_audit_action, NULL, NULL, 'Deleted', 'deleted', 3),
    (cat_audit_action, NULL, NULL, 'Viewed', 'viewed', 4),
    (cat_audit_action, NULL, NULL, 'Exported', 'exported', 5),
    (cat_audit_action, NULL, NULL, 'Approved', 'approved', 6),
    (cat_audit_action, NULL, NULL, 'Rejected', 'rejected', 7);

    -- =====================================================================
    -- PAC Status (EU Subsidy Applications)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_pac_status, NULL, NULL, 'Draft', 'draft', 1),
    (cat_pac_status, NULL, NULL, 'Submitted', 'submitted', 2),
    (cat_pac_status, NULL, NULL, 'In Review', 'inReview', 3),
    (cat_pac_status, NULL, NULL, 'Approved', 'approved', 4),
    (cat_pac_status, NULL, NULL, 'Rejected', 'rejected', 5),
    (cat_pac_status, NULL, NULL, 'Paid', 'paid', 6);

    -- =====================================================================
    -- Cost Center Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_cost_center_type, NULL, NULL, 'Site', 'site', 1),
    (cat_cost_center_type, NULL, NULL, 'Crop', 'crop', 2),
    (cat_cost_center_type, NULL, NULL, 'Activity', 'activity', 3),
    (cat_cost_center_type, NULL, NULL, 'General', 'general', 4);

    -- =====================================================================
    -- Financial Record Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_financial_record_type, NULL, NULL, 'Expense', 'expense', 1),
    (cat_financial_record_type, NULL, NULL, 'Income', 'income', 2),
    (cat_financial_record_type, NULL, NULL, 'Subsidy', 'subsidy', 3);

    -- =====================================================================
    -- Contract Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_contract_type, NULL, NULL, 'Permanent', 'permanent', 1),
    (cat_contract_type, NULL, NULL, 'Temporary', 'temporary', 2),
    (cat_contract_type, NULL, NULL, 'Seasonal', 'seasonal', 3),
    (cat_contract_type, NULL, NULL, 'Freelance', 'freelance', 4),
    (cat_contract_type, NULL, NULL, 'Apprenticeship', 'apprenticeship', 5);

    -- =====================================================================
    -- Clock Entry Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_clock_entry_type, NULL, NULL, 'Clock In', 'clockIn', 1),
    (cat_clock_entry_type, NULL, NULL, 'Clock Out', 'clockOut', 2);

    -- =====================================================================
    -- Lot Status (Harvest)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_lot_status, NULL, NULL, 'Collecting', 'collecting', 1),
    (cat_lot_status, NULL, NULL, 'Processed', 'processed', 2),
    (cat_lot_status, NULL, NULL, 'Shipped', 'shipped', 3),
    (cat_lot_status, NULL, NULL, 'Stored', 'stored', 4);

    -- =====================================================================
    -- Plant Protection Area Methods
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_plant_protection_area_method, NULL, NULL, 'Net Area', 'netArea', 1),
    (cat_plant_protection_area_method, NULL, NULL, 'Gross Area', 'grossArea', 2),
    (cat_plant_protection_area_method, NULL, NULL, 'Leaf Wall Measured', 'leafWallMeasured', 3),
    (cat_plant_protection_area_method, NULL, NULL, 'Leaf Wall Gross', 'leafWallGross', 4),
    (cat_plant_protection_area_method, NULL, NULL, 'Ground Protection', 'groundProtection', 5),
    (cat_plant_protection_area_method, NULL, NULL, 'Herbicide Protection', 'herbicideProtection', 6),
    (cat_plant_protection_area_method, NULL, NULL, 'Tree Crown Volume', 'treeCrownVolume', 7);

    -- =====================================================================
    -- Task Execution Modes
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_task_execution_mode, NULL, NULL, 'Manual', 'manual', 1),
    (cat_task_execution_mode, NULL, NULL, 'Auto Presence', 'autoPresence', 2),
    (cat_task_execution_mode, NULL, NULL, 'Manual with Default Duration', 'manualWithDefaultDuration', 3);

    -- =====================================================================
    -- Recurrence Cadence
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_recurrence_cadence, NULL, NULL, 'Hourly', 'hourly', 1),
    (cat_recurrence_cadence, NULL, NULL, 'Daily', 'daily', 2),
    (cat_recurrence_cadence, NULL, NULL, 'Weekly', 'weekly', 3),
    (cat_recurrence_cadence, NULL, NULL, 'Monthly', 'monthly', 4),
    (cat_recurrence_cadence, NULL, NULL, 'Yearly', 'yearly', 5);

    -- =====================================================================
    -- Sub-Task Status
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_sub_task_status, NULL, NULL, 'New', 'new', 1),
    (cat_sub_task_status, NULL, NULL, 'Started', 'started', 2),
    (cat_sub_task_status, NULL, NULL, 'Paused', 'paused', 3),
    (cat_sub_task_status, NULL, NULL, 'Stopped', 'stopped', 4),
    (cat_sub_task_status, NULL, NULL, 'Done', 'done', 5);

    -- =====================================================================
    -- Overall Task Status
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_overall_task_status, NULL, NULL, 'New', 'new', 1),
    (cat_overall_task_status, NULL, NULL, 'In Progress', 'inProgress', 2),
    (cat_overall_task_status, NULL, NULL, 'Stopped', 'stopped', 3),
    (cat_overall_task_status, NULL, NULL, 'Completed', 'completed', 4);

    -- =====================================================================
    -- Depreciation Methods
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_depreciation_method, NULL, NULL, 'Straight Line', 'straightLine', 1),
    (cat_depreciation_method, NULL, NULL, 'Double Declining', 'doubleDeclining', 2);

    -- =====================================================================
    -- Inventory Methods
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_inventory_method, NULL, NULL, 'FIFO', 'fifo', 1),
    (cat_inventory_method, NULL, NULL, 'FIFO (Expired First)', 'fefo', 2);

    -- =====================================================================
    -- Transaction Types (Inventory)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_transaction_type, NULL, NULL, 'Stock In', 'stockIn', 1),
    (cat_transaction_type, NULL, NULL, 'Stock Out', 'stockOut', 2),
    (cat_transaction_type, NULL, NULL, 'Transfer', 'transfer', 3),
    (cat_transaction_type, NULL, NULL, 'Adjustment', 'adjustment', 4);

    -- =====================================================================
    -- Risk Levels
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_risk_level, NULL, NULL, 'Low', 'low', 1),
    (cat_risk_level, NULL, NULL, 'Medium', 'medium', 2),
    (cat_risk_level, NULL, NULL, 'High', 'high', 3),
    (cat_risk_level, NULL, NULL, 'Critical', 'critical', 4);

    -- =====================================================================
    -- IoT Device Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_iot_device_type, NULL, NULL, 'Weather Station', 'weatherStation', 1),
    (cat_iot_device_type, NULL, NULL, 'Soil Sensor', 'soilSensor', 2),
    (cat_iot_device_type, NULL, NULL, 'Soil Moisture Probe', 'soilMoistureProbe', 3),
    (cat_iot_device_type, NULL, NULL, 'Irrigation Valve', 'irrigationValve', 4),
    (cat_iot_device_type, NULL, NULL, 'Generic', 'generic', 5);

    -- =====================================================================
    -- Setting Value Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_setting_value_type, NULL, NULL, 'String', 'string', 1),
    (cat_setting_value_type, NULL, NULL, 'Number', 'number', 2),
    (cat_setting_value_type, NULL, NULL, 'Boolean', 'boolean', 3),
    (cat_setting_value_type, NULL, NULL, 'JSON', 'json', 4),
    (cat_setting_value_type, NULL, NULL, 'Array', 'array', 5);

    -- =====================================================================
    -- Sync Entity Types (35 entity types)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_sync_entity_type, NULL, NULL, 'Site', 'site', 1),
    (cat_sync_entity_type, NULL, NULL, 'Order', 'order', 2),
    (cat_sync_entity_type, NULL, NULL, 'Task', 'task', 3),
    (cat_sync_entity_type, NULL, NULL, 'Task Data', 'taskData', 4),
    (cat_sync_entity_type, NULL, NULL, 'Equipment', 'equipment', 5),
    (cat_sync_entity_type, NULL, NULL, 'Animal', 'animal', 6),
    (cat_sync_entity_type, NULL, NULL, 'Weather Station', 'weatherStation', 7),
    (cat_sync_entity_type, NULL, NULL, 'Weather Data', 'weatherData', 8),
    (cat_sync_entity_type, NULL, NULL, 'Harvest Season', 'harvestSeason', 9),
    (cat_sync_entity_type, NULL, NULL, 'Harvest Lot', 'harvestLot', 10),
    (cat_sync_entity_type, NULL, NULL, 'Harvest Delivery', 'harvestDelivery', 11),
    (cat_sync_entity_type, NULL, NULL, 'Cold Chain Log', 'coldChainLog', 12),
    (cat_sync_entity_type, NULL, NULL, 'Vineyard', 'vineyard', 13),
    (cat_sync_entity_type, NULL, NULL, 'Kelter Delivery', 'kelterDelivery', 14),
    (cat_sync_entity_type, NULL, NULL, 'Olive Grove', 'oliveGrove', 15),
    (cat_sync_entity_type, NULL, NULL, 'Olive Oil Record', 'oliveOilRecord', 16),
    (cat_sync_entity_type, NULL, NULL, 'Water Source', 'waterSource', 17),
    (cat_sync_entity_type, NULL, NULL, 'Water Usage', 'waterUsage', 18),
    (cat_sync_entity_type, NULL, NULL, 'Water Quota', 'waterQuota', 19),
    (cat_sync_entity_type, NULL, NULL, 'Plant Protection Record', 'plantProtectionRecord', 20),
    (cat_sync_entity_type, NULL, NULL, 'Applicator License', 'applicatorLicense', 21),
    (cat_sync_entity_type, NULL, NULL, 'Fertilizer Record', 'fertilizerRecord', 22),
    (cat_sync_entity_type, NULL, NULL, 'Compliance Checklist', 'complianceChecklist', 23),
    (cat_sync_entity_type, NULL, NULL, 'Compliance Item', 'complianceItem', 24),
    (cat_sync_entity_type, NULL, NULL, 'Audit Log', 'auditLog', 25),
    (cat_sync_entity_type, NULL, NULL, 'PAC Application', 'pacApplication', 26),
    (cat_sync_entity_type, NULL, NULL, 'Cost Center', 'costCenter', 27),
    (cat_sync_entity_type, NULL, NULL, 'Financial Record', 'financialRecord', 28),
    (cat_sync_entity_type, NULL, NULL, 'User', 'user', 29),
    (cat_sync_entity_type, NULL, NULL, 'Worker', 'worker', 30),
    (cat_sync_entity_type, NULL, NULL, 'Worker Location', 'workerLocation', 31),
    (cat_sync_entity_type, NULL, NULL, 'Work Log', 'workLog', 32),
    (cat_sync_entity_type, NULL, NULL, 'Worker Task Status', 'workerTaskStatus', 33),
    (cat_sync_entity_type, NULL, NULL, 'Phenology Record', 'phenologyRecord', 34),
    (cat_sync_entity_type, NULL, NULL, 'Type Category', 'typeCategory', 35),
    (cat_sync_entity_type, NULL, NULL, 'Type Entry', 'typeEntry', 36);

    -- =====================================================================
    -- Sync Operations
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_sync_operation, NULL, NULL, 'Create', 'create', 1),
    (cat_sync_operation, NULL, NULL, 'Update', 'update', 2),
    (cat_sync_operation, NULL, NULL, 'Delete', 'delete', 3);

    -- =====================================================================
    -- Conflict Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_conflict_type, NULL, NULL, 'Concurrent Modification', 'concurrentModification', 1),
    (cat_conflict_type, NULL, NULL, 'Update After Delete', 'updateAfterDelete', 2),
    (cat_conflict_type, NULL, NULL, 'Duplicate Create', 'duplicateCreate', 3),
    (cat_conflict_type, NULL, NULL, 'Version Mismatch', 'versionMismatch', 4);

    -- =====================================================================
    -- Conflict Resolution
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_conflict_resolution, NULL, NULL, 'Last Writer Wins', 'lastWriterWins', 1),
    (cat_conflict_resolution, NULL, NULL, 'Server Wins', 'serverWins', 2),
    (cat_conflict_resolution, NULL, NULL, 'Client Wins', 'clientWins', 3),
    (cat_conflict_resolution, NULL, NULL, 'Merge', 'merge', 4),
    (cat_conflict_resolution, NULL, NULL, 'Manual', 'manual', 5);

    -- =====================================================================
    -- Worker Task Status Types
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_worker_task_status_type, NULL, NULL, 'New', 'new', 1),
    (cat_worker_task_status_type, NULL, NULL, 'Started', 'started', 2),
    (cat_worker_task_status_type, NULL, NULL, 'Paused', 'paused', 3),
    (cat_worker_task_status_type, NULL, NULL, 'Stopped', 'stopped', 4),
    (cat_worker_task_status_type, NULL, NULL, 'Done', 'done', 5);

    -- =====================================================================
    -- Livestock Types (alias for animal_species, same codes)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_livestock_type, NULL, NULL, 'Goat', 'goat', 1),
    (cat_livestock_type, NULL, NULL, 'Chicken', 'chicken', 2),
    (cat_livestock_type, NULL, NULL, 'Sheep', 'sheep', 3),
    (cat_livestock_type, NULL, NULL, 'Cattle', 'cattle', 4),
    (cat_livestock_type, NULL, NULL, 'Pig', 'pig', 5),
    (cat_livestock_type, NULL, NULL, 'Horse', 'horse', 6),
    (cat_livestock_type, NULL, NULL, 'Duck', 'duck', 7),
    (cat_livestock_type, NULL, NULL, 'Turkey', 'turkey', 8),
    (cat_livestock_type, NULL, NULL, 'Goose', 'goose', 9);

END $$;

-- ---------------------------------------------------------------------------
-- 5. Clean up helper functions
-- ---------------------------------------------------------------------------

DROP FUNCTION IF EXISTS get_type_category_id(VARCHAR);
DROP FUNCTION IF EXISTS get_type_entry_id(VARCHAR, VARCHAR, UUID);

-- ---------------------------------------------------------------------------
-- 6. Verification
-- ---------------------------------------------------------------------------

-- Verify category count
DO $$
DECLARE
    cat_count INTEGER;
    entry_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO cat_count FROM type_categories;
    SELECT COUNT(*) INTO entry_count FROM type_entries;
    RAISE NOTICE 'Seeded % type categories and % type entries', cat_count, entry_count;
    IF cat_count < 50 THEN
        RAISE WARNING 'Expected at least 50 categories, got %', cat_count;
    END IF;
    IF entry_count < 200 THEN
        RAISE WARNING 'Expected at least 200 entries, got %', entry_count;
    END IF;
END $$;
