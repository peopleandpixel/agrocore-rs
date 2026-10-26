-- Migration 0000000030: Migrate entity enum columns to UUID type references
-- This migration adds type_id UUID columns to all entity tables that currently use JSONB enum values,
-- migrates existing data by matching codes to type_entries, and adds foreign key constraints.

-- ============================================================================
-- SITES: site_type and crop_type
-- ============================================================================

ALTER TABLE sites ADD COLUMN IF NOT EXISTS site_type_id UUID;
ALTER TABLE sites ADD COLUMN IF NOT EXISTS crop_type_id UUID;

-- Migrate site_type from JSONB to UUID reference (only if column exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'sites' AND column_name = 'site_type') THEN
        UPDATE sites s
        SET site_type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'site_type'
          AND te.code = (s.site_type->>'value')::text
          AND s.site_type_id IS NULL;

        UPDATE sites s
        SET site_type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'site_type'
          AND te.code = s.site_type::text
          AND s.site_type_id IS NULL;
    END IF;
END $$;

-- Migrate crop_type from JSONB to UUID reference (only if column exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'sites' AND column_name = 'crop_type') THEN
        UPDATE sites s
        SET crop_type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'crop_type'
          AND te.code = (s.crop_type->>'value')::text
          AND s.crop_type_id IS NULL;

        UPDATE sites s
        SET crop_type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'crop_type'
          AND te.code = s.crop_type::text
          AND s.crop_type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- SPATIAL_OBJECTS: object_type
-- ============================================================================

ALTER TABLE spatial_objects ADD COLUMN IF NOT EXISTS type_id UUID;

UPDATE spatial_objects so
SET type_id = te.id
FROM type_categories tc
JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
WHERE tc.name = 'spatial_object_type'
  AND te.code = so.object_type::text
  AND so.type_id IS NULL;

-- ============================================================================
-- EQUIPMENT: equipment_type
-- ============================================================================

ALTER TABLE equipment ADD COLUMN IF NOT EXISTS type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'equipment' AND column_name = 'equipment_type') THEN
        UPDATE equipment e
        SET type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'equipment_type'
          AND te.code = e.equipment_type::text
          AND e.type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- BUILDINGS: building_type
-- ============================================================================

ALTER TABLE buildings ADD COLUMN IF NOT EXISTS type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'buildings' AND column_name = 'building_type') THEN
        UPDATE buildings b
        SET type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'building_type'
          AND te.code = b.building_type::text
          AND b.type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- GROUPS: group_type
-- ============================================================================

ALTER TABLE groups ADD COLUMN IF NOT EXISTS type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'groups' AND column_name = 'group_type') THEN
        UPDATE groups g
        SET type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'group_type'
          AND te.code = g.group_type::text
          AND g.type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- TREES: tree_type
-- ============================================================================

ALTER TABLE trees ADD COLUMN IF NOT EXISTS type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'trees' AND column_name = 'tree_type') THEN
        UPDATE trees t
        SET type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'tree_type'
          AND te.code = t.tree_type::text
          AND t.type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- WATER_SOURCES: source_type
-- ============================================================================

ALTER TABLE water_sources ADD COLUMN IF NOT EXISTS source_type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'water_sources' AND column_name = 'source_type') THEN
        UPDATE water_sources ws
        SET source_type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'water_source_type'
          AND te.code = ws.source_type::text
          AND ws.source_type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- WATER_USAGE: irrigation_method
-- ============================================================================

ALTER TABLE water_usages ADD COLUMN IF NOT EXISTS irrigation_method_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'water_usages' AND column_name = 'irrigation_method') THEN
        UPDATE water_usages wu
        SET irrigation_method_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'irrigation_method'
          AND te.code = wu.irrigation_method::text
          AND wu.irrigation_method_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- WEATHER_STATIONS: station_type
-- ============================================================================

ALTER TABLE weather_stations ADD COLUMN IF NOT EXISTS station_type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'weather_stations' AND column_name = 'station_type') THEN
        UPDATE weather_stations ws
        SET station_type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'weather_station_type'
          AND te.code = ws.station_type::text
          AND ws.station_type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- INVENTORY_ITEMS: category and unit_of_measure
-- ============================================================================

ALTER TABLE inventory_items ADD COLUMN IF NOT EXISTS category_id UUID;
ALTER TABLE inventory_items ADD COLUMN IF NOT EXISTS unit_of_measure_id UUID;
ALTER TABLE inventory_items ADD COLUMN IF NOT EXISTS inventory_method_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'inventory_items' AND column_name = 'category') THEN
        UPDATE inventory_items ii
        SET category_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'inventory_category'
          AND te.code = ii.category::text
          AND ii.category_id IS NULL;
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'inventory_items' AND column_name = 'unit_of_measure') THEN
        UPDATE inventory_items ii
        SET unit_of_measure_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'unit_of_measure'
          AND te.code = ii.unit_of_measure::text
          AND ii.unit_of_measure_id IS NULL;
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'inventory_items' AND column_name = 'inventory_method') THEN
        UPDATE inventory_items ii
        SET inventory_method_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'inventory_method'
          AND te.code = ii.inventory_method::text
          AND ii.inventory_method_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- INVENTORY_TRANSACTIONS: transaction_type
-- ============================================================================

ALTER TABLE inventory_transactions ADD COLUMN IF NOT EXISTS transaction_type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'inventory_transactions' AND column_name = 'transaction_type') THEN
        UPDATE inventory_transactions it
        SET transaction_type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'transaction_type'
          AND te.code = it.transaction_type::text
          AND it.transaction_type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- VINEYARDS: quality_grade
-- ============================================================================

ALTER TABLE vineyards ADD COLUMN IF NOT EXISTS quality_grade_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'vineyards' AND column_name = 'quality_grade') THEN
        UPDATE vineyards v
        SET quality_grade_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'quality_grade'
          AND te.code = v.quality_grade::text
          AND v.quality_grade_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- OLIVE_OIL_RECORDS: oil_grade
-- ============================================================================

ALTER TABLE olive_oil_records ADD COLUMN IF NOT EXISTS oil_grade_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'olive_oil_records' AND column_name = 'oil_grade') THEN
        UPDATE olive_oil_records oor
        SET oil_grade_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'oil_grade'
          AND te.code = oor.oil_grade::text
          AND oor.oil_grade_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- OLIVE_GROVES: variety_category
-- ============================================================================

ALTER TABLE olive_groves ADD COLUMN IF NOT EXISTS variety_category_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'olive_groves' AND column_name = 'variety_category') THEN
        UPDATE olive_groves og
        SET variety_category_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'variety_category'
          AND te.code = og.variety_category::text
          AND og.variety_category_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- PLANT_PROTECTION_RECORDS: area_method
-- ============================================================================

ALTER TABLE plant_protection_records ADD COLUMN IF NOT EXISTS area_method_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'plant_protection_records' AND column_name = 'area_method') THEN
        UPDATE plant_protection_records ppr
        SET area_method_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'plant_protection_area_method'
          AND te.code = ppr.area_method::text
          AND ppr.area_method_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- COMPLIANCE_CHECKLISTS: checklist_type and status
-- ============================================================================

ALTER TABLE compliance_checklists ADD COLUMN IF NOT EXISTS type_id UUID;
ALTER TABLE compliance_checklists ADD COLUMN IF NOT EXISTS status_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'compliance_checklists' AND column_name = 'checklist_type') THEN
        UPDATE compliance_checklists cc
        SET type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'checklist_type'
          AND te.code = cc.checklist_type::text
          AND cc.type_id IS NULL;
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'compliance_checklists' AND column_name = 'status') THEN
        UPDATE compliance_checklists cc
        SET status_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'compliance_status'
          AND te.code = cc.status::text
          AND cc.status_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- PAC_APPLICATIONS: status
-- ============================================================================

ALTER TABLE pac_applications ADD COLUMN IF NOT EXISTS status_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'pac_applications' AND column_name = 'status') THEN
        UPDATE pac_applications pa
        SET status_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'pac_status'
          AND te.code = pa.status::text
          AND pa.status_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- COST_CENTERS: cost_center_type
-- ============================================================================

ALTER TABLE cost_centers ADD COLUMN IF NOT EXISTS type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'cost_centers' AND column_name = 'cost_center_type') THEN
        UPDATE cost_centers cc
        SET type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'cost_center_type'
          AND te.code = cc.cost_center_type::text
          AND cc.type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- FINANCIAL_RECORDS: record_type
-- ============================================================================

ALTER TABLE financial_records ADD COLUMN IF NOT EXISTS type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'financial_records' AND column_name = 'record_type') THEN
        UPDATE financial_records fr
        SET type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'financial_record_type'
          AND te.code = fr.record_type::text
          AND fr.type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- WORKERS: contract_type
-- ============================================================================

ALTER TABLE workers ADD COLUMN IF NOT EXISTS contract_type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'workers' AND column_name = 'contract_type') THEN
        UPDATE workers w
        SET contract_type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'contract_type'
          AND te.code = w.contract_type::text
          AND w.contract_type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- HARVEST_LOTS: status
-- ============================================================================

ALTER TABLE harvest_lots ADD COLUMN IF NOT EXISTS status_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'harvest_lots' AND column_name = 'status') THEN
        UPDATE harvest_lots hl
        SET status_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'lot_status'
          AND te.code = hl.status::text
          AND hl.status_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- TASKS: execution_mode
-- ============================================================================

ALTER TABLE tasks ADD COLUMN IF NOT EXISTS execution_mode_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tasks' AND column_name = 'execution_mode') THEN
        UPDATE tasks t
        SET execution_mode_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'task_execution_mode'
          AND te.code = t.execution_mode::text
          AND t.execution_mode_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- ORDERS: order_type
-- ============================================================================

ALTER TABLE orders ADD COLUMN IF NOT EXISTS order_type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'orders' AND column_name = 'order_type') THEN
        UPDATE orders o
        SET order_type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'order_type'
          AND te.code = o.order_type::text
          AND o.order_type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- ANIMALS: species and status (if stored as JSONB)
-- ============================================================================

ALTER TABLE animals ADD COLUMN IF NOT EXISTS species_id UUID;
ALTER TABLE animals ADD COLUMN IF NOT EXISTS status_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'animals' AND column_name = 'species') THEN
        UPDATE animals a
        SET species_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'animal_species'
          AND LOWER(te.code) = LOWER(a.species)
          AND a.species_id IS NULL;
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'animals' AND column_name = 'status') THEN
        UPDATE animals a
        SET status_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'animal_status'
          AND te.code = a.status
          AND a.status_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- SETTINGS: value_type
-- ============================================================================

ALTER TABLE system_settings ADD COLUMN IF NOT EXISTS value_type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'system_settings' AND column_name = 'value_type') THEN
        UPDATE system_settings s
        SET value_type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'setting_value_type'
          AND te.code = s.value_type::text
          AND s.value_type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- IOT_DEVICES: device_type
-- ============================================================================

ALTER TABLE iot_devices ADD COLUMN IF NOT EXISTS device_type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'iot_devices' AND column_name = 'device_type') THEN
        UPDATE iot_devices id
        SET device_type_id = te.id
        FROM type_categories tc
        JOIN type_entries te ON te.category_id = tc.id AND te.tenant_id IS NULL
        WHERE tc.name = 'iot_device_type'
          AND te.code = id.device_type::text
          AND id.device_type_id IS NULL;
    END IF;
END $$;

-- ============================================================================
-- Add indexes for new UUID columns
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_sites_site_type_id ON sites(site_type_id);
CREATE INDEX IF NOT EXISTS idx_sites_crop_type_id ON sites(crop_type_id);
CREATE INDEX IF NOT EXISTS idx_spatial_objects_type_id ON spatial_objects(type_id);
CREATE INDEX IF NOT EXISTS idx_equipment_type_id ON equipment(type_id);
CREATE INDEX IF NOT EXISTS idx_buildings_type_id ON buildings(type_id);
CREATE INDEX IF NOT EXISTS idx_groups_type_id ON groups(type_id);
CREATE INDEX IF NOT EXISTS idx_trees_type_id ON trees(type_id);
CREATE INDEX IF NOT EXISTS idx_water_sources_source_type_id ON water_sources(source_type_id);
CREATE INDEX IF NOT EXISTS idx_water_usages_irrigation_method_id ON water_usages(irrigation_method_id);
CREATE INDEX IF NOT EXISTS idx_weather_stations_station_type_id ON weather_stations(station_type_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_category_id ON inventory_items(category_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_unit_of_measure_id ON inventory_items(unit_of_measure_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_inventory_method_id ON inventory_items(inventory_method_id);
CREATE INDEX IF NOT EXISTS idx_inventory_transactions_transaction_type_id ON inventory_transactions(transaction_type_id);
CREATE INDEX IF NOT EXISTS idx_vineyards_quality_grade_id ON vineyards(quality_grade_id);
CREATE INDEX IF NOT EXISTS idx_olive_oil_records_oil_grade_id ON olive_oil_records(oil_grade_id);
CREATE INDEX IF NOT EXISTS idx_olive_groves_variety_category_id ON olive_groves(variety_category_id);
CREATE INDEX IF NOT EXISTS idx_plant_protection_records_area_method_id ON plant_protection_records(area_method_id);
CREATE INDEX IF NOT EXISTS idx_compliance_checklists_type_id ON compliance_checklists(type_id);
CREATE INDEX IF NOT EXISTS idx_compliance_checklists_status_id ON compliance_checklists(status_id);
CREATE INDEX IF NOT EXISTS idx_pac_applications_status_id ON pac_applications(status_id);
CREATE INDEX IF NOT EXISTS idx_cost_centers_type_id ON cost_centers(type_id);
CREATE INDEX IF NOT EXISTS idx_financial_records_type_id ON financial_records(type_id);
CREATE INDEX IF NOT EXISTS idx_workers_contract_type_id ON workers(contract_type_id);
CREATE INDEX IF NOT EXISTS idx_harvest_lots_status_id ON harvest_lots(status_id);
CREATE INDEX IF NOT EXISTS idx_tasks_execution_mode_id ON tasks(execution_mode_id);
CREATE INDEX IF NOT EXISTS idx_orders_order_type_id ON orders(order_type_id);
CREATE INDEX IF NOT EXISTS idx_animals_species_id ON animals(species_id);
CREATE INDEX IF NOT EXISTS idx_animals_status_id ON animals(status_id);
CREATE INDEX IF NOT EXISTS idx_system_settings_value_type_id ON system_settings(value_type_id);
CREATE INDEX IF NOT EXISTS idx_iot_devices_device_type_id ON iot_devices(device_type_id);
