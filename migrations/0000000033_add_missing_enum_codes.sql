-- Migration 33: Add missing enum codes to type_entries
--
-- Fills the gaps between Rust enum variants and the seeded type_entries.
-- Uses INSERT ... ON CONFLICT DO NOTHING so it is safe to re-run.

DO $$
DECLARE
    cat_crop_type UUID;
    cat_site_type UUID;
    cat_inventory_category UUID;
    cat_unit_of_measure UUID;
    cat_inventory_method UUID;
    cat_water_source_type UUID;
    cat_irrigation_method UUID;
    cat_variety_category UUID;
    cat_checklist_type UUID;
    cat_compliance_status UUID;
    cat_cost_center_type UUID;
    cat_lot_status UUID;
    cat_worker_task_status_type UUID;
    cat_animal_species UUID;
    cat_animal_status UUID;
    cat_species UUID;
    cat_task_execution_mode UUID;
    cat_license_type UUID;
    cat_bbch_stage UUID;

BEGIN

    -- Get all category IDs
    SELECT id INTO cat_crop_type FROM type_categories WHERE name = 'crop_type';
    SELECT id INTO cat_site_type FROM type_categories WHERE name = 'site_type';
    SELECT id INTO cat_inventory_category FROM type_categories WHERE name = 'inventory_category';
    SELECT id INTO cat_unit_of_measure FROM type_categories WHERE name = 'unit_of_measure';
    SELECT id INTO cat_inventory_method FROM type_categories WHERE name = 'inventory_method';
    SELECT id INTO cat_water_source_type FROM type_categories WHERE name = 'water_source_type';
    SELECT id INTO cat_irrigation_method FROM type_categories WHERE name = 'irrigation_method';
    SELECT id INTO cat_variety_category FROM type_categories WHERE name = 'variety_category';
    SELECT id INTO cat_checklist_type FROM type_categories WHERE name = 'checklist_type';
    SELECT id INTO cat_compliance_status FROM type_categories WHERE name = 'compliance_status';
    SELECT id INTO cat_cost_center_type FROM type_categories WHERE name = 'cost_center_type';
    SELECT id INTO cat_lot_status FROM type_categories WHERE name = 'lot_status';
    SELECT id INTO cat_worker_task_status_type FROM type_categories WHERE name = 'worker_task_status_type';
    SELECT id INTO cat_animal_species FROM type_categories WHERE name = 'animal_species';
    SELECT id INTO cat_animal_status FROM type_categories WHERE name = 'animal_status';
    SELECT id INTO cat_species FROM type_categories WHERE name = 'species';
    SELECT id INTO cat_task_execution_mode FROM type_categories WHERE name = 'task_execution_mode';
    SELECT id INTO cat_bbch_stage FROM type_categories WHERE name = 'bbch_stage';

    -- =====================================================================
    -- Create missing category: license_type
    -- =====================================================================

    INSERT INTO type_categories (name, label, description, is_hierarchical)
    VALUES ('license_type', 'License Type', 'Types of applicator licenses', false)
    ON CONFLICT (name) DO NOTHING;

    SELECT id INTO cat_license_type FROM type_categories WHERE name = 'license_type';

    -- =====================================================================
    -- CropType: add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_crop_type, NULL, NULL, 'Wheat', 'wheat', 19),
    (cat_crop_type, NULL, NULL, 'Corn', 'corn', 20),
    (cat_crop_type, NULL, NULL, 'Rice', 'rice', 21),
    (cat_crop_type, NULL, NULL, 'Vegetables', 'vegetables', 22),
    (cat_crop_type, NULL, NULL, 'Fruit', 'fruit', 23),
    (cat_crop_type, NULL, NULL, 'Other', 'other', 24)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- SiteType: add missing Rust variant
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_site_type, NULL, NULL, 'Other', 'other', 11)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- InventoryCategory: add missing Rust variants (snake_case aliases)
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_inventory_category, NULL, NULL, 'Seeds', 'seeds', 10),
    (cat_inventory_category, NULL, NULL, 'Fertilizers', 'fertilizers', 11),
    (cat_inventory_category, NULL, NULL, 'Plant Protection', 'plant_protection', 12),
    (cat_inventory_category, NULL, NULL, 'Other', 'other', 13)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- UnitOfMeasure: add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_unit_of_measure, NULL, NULL, 'Meters', 'meters', 7),
    (cat_unit_of_measure, NULL, NULL, 'Pieces', 'pieces', 8),
    (cat_unit_of_measure, NULL, NULL, 'Hectoliters', 'hectoliters', 9)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- InventoryMethod: add missing Rust variant
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_inventory_method, NULL, NULL, 'Average Cost', 'average_cost', 3)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- WaterSourceType: add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_water_source_type, NULL, NULL, 'Rainwater', 'rainwater', 8),
    (cat_water_source_type, NULL, NULL, 'Municipal', 'municipal', 9),
    (cat_water_source_type, NULL, NULL, 'Other', 'other', 10)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- IrrigationMethod: add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_irrigation_method, NULL, NULL, 'Center Pivot', 'center_pivot', 7),
    (cat_irrigation_method, NULL, NULL, 'Other', 'other', 8)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- VarietyCategory: add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_variety_category, NULL, NULL, 'Wheat', 'wheat', 9),
    (cat_variety_category, NULL, NULL, 'Corn', 'corn', 10),
    (cat_variety_category, NULL, NULL, 'Other', 'other', 11)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- ChecklistType: add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_checklist_type, NULL, NULL, 'Safety', 'safety', 5),
    (cat_checklist_type, NULL, NULL, 'Environmental', 'environmental', 6),
    (cat_checklist_type, NULL, NULL, 'Quality', 'quality', 7),
    (cat_checklist_type, NULL, NULL, 'Regulatory', 'regulatory', 8),
    (cat_checklist_type, NULL, NULL, 'Custom', 'custom', 9)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- ComplianceStatus: add missing Rust variant
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_compliance_status, NULL, NULL, 'Overdue', 'overdue', 6)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- CostCenterType: add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_cost_center_type, NULL, NULL, 'Production', 'production', 5),
    (cat_cost_center_type, NULL, NULL, 'Administration', 'administration', 6),
    (cat_cost_center_type, NULL, NULL, 'Sales', 'sales', 7),
    (cat_cost_center_type, NULL, NULL, 'Other', 'other', 8)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- LotStatus: add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_lot_status, NULL, NULL, 'Processing', 'processing', 5),
    (cat_lot_status, NULL, NULL, 'Cancelled', 'cancelled', 6)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- WorkerTaskStatusType: add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_worker_task_status_type, NULL, NULL, 'In Progress', 'in_progress', 6),
    (cat_worker_task_status_type, NULL, NULL, 'Resumed', 'resumed', 7)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- AnimalSpecies: add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_species, NULL, NULL, 'Poultry', 'poultry', 15),
    (cat_animal_species, NULL, NULL, 'Horse', 'horse', 16),
    (cat_animal_species, NULL, NULL, 'Other', 'other', 17)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- AnimalStatus: add missing Rust variant
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_status, NULL, NULL, 'Transferred', 'transferred', 5)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- Species (breed): add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_species, NULL, NULL, 'Poultry', 'poultry', 7),
    (cat_species, NULL, NULL, 'Other', 'other', 8)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- TaskExecutionMode: add missing Rust variants
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_task_execution_mode, NULL, NULL, 'Automatic', 'automatic', 4),
    (cat_task_execution_mode, NULL, NULL, 'Semi Automatic', 'semi_automatic', 5)
    ON CONFLICT DO NOTHING;

    -- =====================================================================
    -- LicenseType: seed entries (new category)
    -- =====================================================================

    IF cat_license_type IS NOT NULL THEN
        INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
        (cat_license_type, NULL, NULL, 'Basic', 'basic', 1),
        (cat_license_type, NULL, NULL, 'Advanced', 'advanced', 2),
        (cat_license_type, NULL, NULL, 'Drone', 'drone', 3);
    END IF;

    -- =====================================================================
    -- BBCH Stage: add Rust scale variants alongside the specific stages
    -- The Rust enum uses Scale0..Scale9 which map to BBCH decimal scales.
    -- These are added as additional entries so the Rust code can resolve them.
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_bbch_stage, NULL, NULL, 'Scale 0', 'scale0', 100),
    (cat_bbch_stage, NULL, NULL, 'Scale 1', 'scale1', 101),
    (cat_bbch_stage, NULL, NULL, 'Scale 2', 'scale2', 102),
    (cat_bbch_stage, NULL, NULL, 'Scale 3', 'scale3', 103),
    (cat_bbch_stage, NULL, NULL, 'Scale 4', 'scale4', 104),
    (cat_bbch_stage, NULL, NULL, 'Scale 5', 'scale5', 105),
    (cat_bbch_stage, NULL, NULL, 'Scale 6', 'scale6', 106),
    (cat_bbch_stage, NULL, NULL, 'Scale 7', 'scale7', 107),
    (cat_bbch_stage, NULL, NULL, 'Scale 8', 'scale8', 108),
    (cat_bbch_stage, NULL, NULL, 'Scale 9', 'scale9', 109)
    ON CONFLICT DO NOTHING;

END $$;

-- ---------------------------------------------------------------------------
-- Verification
-- ---------------------------------------------------------------------------

DO $$
DECLARE
    entry_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO entry_count FROM type_entries;
    RAISE NOTICE 'Total type entries after migration 33: %', entry_count;
END $$;
