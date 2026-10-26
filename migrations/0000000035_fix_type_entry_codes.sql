-- Migration 0035: Fix code inconsistencies between Rust code and type_entries
--
-- The Rust code uses different category names and entry codes than the migrations.
-- This migration fixes all inconsistencies by:
-- 1. Adding snake_case aliases for entry codes where Rust uses snake_case
-- 2. Adding missing entry codes used by Rust repositories
-- 3. Renaming categories where Rust code uses different names

-- =====================================================================
-- A. Fix category name mismatches
-- =====================================================================

-- Rust code in harvest_lot.rs uses 'harvest_lot_status' but migration has 'lot_status'
-- Solution: Add alias category
INSERT INTO type_categories (name, label, description, is_hierarchical) VALUES
('harvest_lot_status', 'Harvest Lot Status', 'Alias for lot_status - used by Rust code', false)
ON CONFLICT (name) DO NOTHING;

-- Rust code in worker_task_status.rs uses 'worker_task_status' but migration has 'worker_task_status_type'
-- Solution: Add alias category
INSERT INTO type_categories (name, label, description, is_hierarchical) VALUES
('worker_task_status', 'Worker Task Status', 'Alias for worker_task_status_type - used by Rust code', false)
ON CONFLICT (name) DO NOTHING;

-- Rust code in inventory_transaction.rs uses 'transaction_type_inventory' but migration has 'transaction_type'
-- 'transaction_type_inventory' already exists in migration 0029, but is empty
-- Solution: Copy entries from transaction_type to transaction_type_inventory

DO $$
DECLARE
    cat_transaction_type UUID;
    cat_transaction_type_inventory UUID;
    cat_harvest_lot_status UUID;
    cat_harvest_lot_status_orig UUID;
    cat_worker_task_status UUID;
    cat_worker_task_status_orig UUID;
BEGIN
    SELECT id INTO cat_transaction_type FROM type_categories WHERE name = 'transaction_type';
    SELECT id INTO cat_transaction_type_inventory FROM type_categories WHERE name = 'transaction_type_inventory';
    SELECT id INTO cat_harvest_lot_status FROM type_categories WHERE name = 'harvest_lot_status';
    SELECT id INTO cat_harvest_lot_status_orig FROM type_categories WHERE name = 'lot_status';
    SELECT id INTO cat_worker_task_status FROM type_categories WHERE name = 'worker_task_status';
    SELECT id INTO cat_worker_task_status_orig FROM type_categories WHERE name = 'worker_task_status_type';

    -- Copy lot_status entries to harvest_lot_status alias
    IF cat_harvest_lot_status IS NOT NULL AND cat_harvest_lot_status_orig IS NOT NULL THEN
        INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
        SELECT cat_harvest_lot_status, parent_id, tenant_id, name, code, sort_order
        FROM type_entries WHERE category_id = cat_harvest_lot_status_orig
        ON CONFLICT (category_id, code) DO NOTHING;
    END IF;

    -- Copy worker_task_status_type entries to worker_task_status alias
    IF cat_worker_task_status IS NOT NULL AND cat_worker_task_status_orig IS NOT NULL THEN
        INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
        SELECT cat_worker_task_status, parent_id, tenant_id, name, code, sort_order
        FROM type_entries WHERE category_id = cat_worker_task_status_orig
        ON CONFLICT (category_id, code) DO NOTHING;
    END IF;

    -- Copy transaction_type entries to transaction_type_inventory alias
    IF cat_transaction_type_inventory IS NOT NULL AND cat_transaction_type IS NOT NULL THEN
        INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
        SELECT cat_transaction_type_inventory, parent_id, tenant_id, name, code, sort_order
        FROM type_entries WHERE category_id = cat_transaction_type
        ON CONFLICT (category_id, code) DO NOTHING;
    END IF;
END $$;

-- =====================================================================
-- B. Add snake_case aliases for entry codes
-- =====================================================================

-- Rust code uses snake_case codes: stock_in, stock_out, etc.
-- Migration uses camelCase: stockIn, stockOut, etc.
-- Add snake_case aliases to transaction_type category

DO $$
DECLARE
    cat_transaction_type UUID;
BEGIN
    SELECT id INTO cat_transaction_type FROM type_categories WHERE name = 'transaction_type';
    
    -- Add snake_case aliases for transaction types
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_transaction_type, NULL, NULL, 'Stock In (alias)', 'stock_in', 10),
    (cat_transaction_type, NULL, NULL, 'Stock Out (alias)', 'stock_out', 11),
    (cat_transaction_type, NULL, NULL, 'Adjustment (alias)', 'adjustment_snake', 13)
    ON CONFLICT (category_id, code) DO NOTHING;
END $$;

-- =====================================================================
-- C. Add missing entry codes used by Rust repositories
-- =====================================================================

-- inventory_method: Rust code may use 'average_cost' but migration only has 'fifo', 'fefo'
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Average Cost', 'average_cost', 3
FROM type_categories WHERE name = 'inventory_method'
ON CONFLICT (category_id, code) DO NOTHING;

-- depreciation_method: Rust code may use 'straight_line' but migration has 'straightLine'
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Straight Line (alias)', 'straight_line', 10
FROM type_categories WHERE name = 'depreciation_method'
ON CONFLICT (category_id, code) DO NOTHING;

-- order_status: Rust code may use 'planned' (already exists), 'in_progress' (missing)
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'In Progress (alias)', 'in_progress', 11
FROM type_categories WHERE name = 'order_status'
ON CONFLICT (category_id, code) DO NOTHING;

-- compliance_status: Rust code may use 'in_progress' (missing)
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'In Progress (alias)', 'in_progress', 11
FROM type_categories WHERE name = 'compliance_status'
ON CONFLICT (category_id, code) DO NOTHING;

-- worker_task_status_type: Rust code may use 'in_progress', 'resumed' (missing)
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'In Progress (alias)', 'in_progress', 11
FROM type_categories WHERE name = 'worker_task_status_type'
ON CONFLICT (category_id, code) DO NOTHING;

-- animal_status: Rust code may use 'transferred' (missing)
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Transferred', 'transferred', 11
FROM type_categories WHERE name = 'animal_status'
ON CONFLICT (category_id, code) DO NOTHING;

-- lot_status: Rust code may use 'processing', 'cancelled' (missing)
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Processing', 'processing', 11
FROM type_categories WHERE name = 'lot_status'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Cancelled', 'cancelled', 12
FROM type_categories WHERE name = 'lot_status'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- D. Add missing site_type and crop_type codes for import service
-- =====================================================================

-- Import service needs to look up site_type and crop_type by code
-- These already exist in migrations 0029 and 0034, but add common fallback codes

-- site_type: 'other' fallback
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Other', 'other', 99
FROM type_categories WHERE name = 'site_type'
ON CONFLICT (category_id, code) DO NOTHING;

-- crop_type: 'other' fallback
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Other', 'other', 99
FROM type_categories WHERE name = 'crop_type'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- E. Add quality_grade codes for specialized records
-- =====================================================================

-- specialized.rs uses quality_grade_id but needs actual entries
-- quality_grade already has: reserva, grandeReserva, garrafeira, superior, classic, lateHarvest
-- Add snake_case aliases
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Extra Virgin (alias)', 'extra_virgin', 10
FROM type_categories WHERE name = 'quality_grade'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- F. Add oil_grade codes for olive oil records
-- =====================================================================

-- oil_grade already has: extraVirgin, virgin, lampante, refined, pomace
-- Add snake_case aliases
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Extra Virgin (alias)', 'extra_virgin', 10
FROM type_categories WHERE name = 'oil_grade'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Virgin (alias)', 'virgin', 11
FROM type_categories WHERE name = 'oil_grade'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- G. Add missing inventory category codes
-- =====================================================================

-- Rust code may use: seeds, fertilizers, plant_protection, other
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Seeds', 'seeds', 10
FROM type_categories WHERE name = 'inventory_category'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Fertilizers', 'fertilizers', 11
FROM type_categories WHERE name = 'inventory_category'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Plant Protection', 'plant_protection', 12
FROM type_categories WHERE name = 'inventory_category'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Other', 'other', 99
FROM type_categories WHERE name = 'inventory_category'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- H. Add missing unit_of_measure codes
-- =====================================================================

-- Rust code may use: meters, pieces, hectoliters
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Meters', 'meters', 10
FROM type_categories WHERE name = 'unit_of_measure'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Pieces', 'pieces', 11
FROM type_categories WHERE name = 'unit_of_measure'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Hectoliters', 'hectoliters', 12
FROM type_categories WHERE name = 'unit_of_measure'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- I. Add missing water_source_type codes
-- =====================================================================

-- Rust code may use: rainwater, municipal, groundwater, lake
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Rainwater', 'rainwater', 10
FROM type_categories WHERE name = 'water_source_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Municipal', 'municipal', 11
FROM type_categories WHERE name = 'water_source_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Groundwater', 'groundwater', 12
FROM type_categories WHERE name = 'water_source_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Lake', 'lake', 13
FROM type_categories WHERE name = 'water_source_type'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- J. Add missing irrigation_method codes
-- =====================================================================

-- Rust code may use: center_pivot, micro_sprinkler
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Center Pivot (alias)', 'center_pivot', 10
FROM type_categories WHERE name = 'irrigation_method'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Micro Sprinkler (alias)', 'micro_sprinkler', 11
FROM type_categories WHERE name = 'irrigation_method'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- K. Add missing checklist_type codes
-- =====================================================================

-- Rust code may use: safety, environmental, quality, regulatory, custom
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Safety', 'safety', 10
FROM type_categories WHERE name = 'checklist_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Environmental', 'environmental', 11
FROM type_categories WHERE name = 'checklist_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Quality', 'quality', 12
FROM type_categories WHERE name = 'checklist_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Regulatory', 'regulatory', 13
FROM type_categories WHERE name = 'checklist_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Custom', 'custom', 14
FROM type_categories WHERE name = 'checklist_type'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- L. Add missing compliance_status codes
-- =====================================================================

-- Rust code may use: overdue
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Overdue', 'overdue', 11
FROM type_categories WHERE name = 'compliance_status'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- M. Add missing cost_center_type codes
-- =====================================================================

-- Rust code may use: production, administration, sales, other
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Production', 'production', 10
FROM type_categories WHERE name = 'cost_center_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Administration', 'administration', 11
FROM type_categories WHERE name = 'cost_center_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Sales', 'sales', 12
FROM type_categories WHERE name = 'cost_center_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Other', 'other', 99
FROM type_categories WHERE name = 'cost_center_type'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- N. Add missing contract_type codes
-- =====================================================================

-- Rust code may use: mini_job, casual
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Mini Job', 'mini_job', 10
FROM type_categories WHERE name = 'contract_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Casual Worker', 'casual', 11
FROM type_categories WHERE name = 'contract_type'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- O. Add missing task_execution_mode codes
-- =====================================================================

-- Rust code may use: automatic, semi_automatic
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Automatic', 'automatic', 10
FROM type_categories WHERE name = 'task_execution_mode'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Semi Automatic', 'semi_automatic', 11
FROM type_categories WHERE name = 'task_execution_mode'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- P. Add missing variety_category codes
-- =====================================================================

-- Rust code may use: rye, potato, legumes, oil_seeds, fiber, herbal
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Rye', 'rye', 10
FROM type_categories WHERE name = 'variety_category'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Potato', 'potato', 11
FROM type_categories WHERE name = 'variety_category'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Legumes', 'legumes', 12
FROM type_categories WHERE name = 'variety_category'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Oil Seeds', 'oil_seeds', 13
FROM type_categories WHERE name = 'variety_category'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Fiber Crops', 'fiber', 14
FROM type_categories WHERE name = 'variety_category'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Herbal Plants', 'herbal', 15
FROM type_categories WHERE name = 'variety_category'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- Q. Add missing species codes
-- =====================================================================

-- Rust code may use: rabbit, bee, fish
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Rabbit', 'rabbit', 10
FROM type_categories WHERE name = 'species'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Bee', 'bee', 11
FROM type_categories WHERE name = 'species'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Fish', 'fish', 12
FROM type_categories WHERE name = 'species'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- R. Add missing animal_species codes
-- =====================================================================

-- Rust code may use: rabbit, bee, fish
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Rabbit', 'rabbit', 10
FROM type_categories WHERE name = 'animal_species'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Bee', 'bee', 11
FROM type_categories WHERE name = 'animal_species'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Fish', 'fish', 12
FROM type_categories WHERE name = 'animal_species'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- S. Add missing order_type codes (snake_case aliases)
-- =====================================================================

-- Rust code may use snake_case: plant_protection, soil_work, etc.
INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Plant Protection (alias)', 'plant_protection', 10
FROM type_categories WHERE name = 'order_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Soil Work (alias)', 'soil_work', 11
FROM type_categories WHERE name = 'order_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Livestock Feeding (alias)', 'livestock_feeding', 12
FROM type_categories WHERE name = 'order_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Livestock Watering (alias)', 'livestock_watering', 13
FROM type_categories WHERE name = 'order_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Barn Cleaning (alias)', 'barn_cleaning', 14
FROM type_categories WHERE name = 'order_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Egg Collection (alias)', 'egg_collection', 15
FROM type_categories WHERE name = 'order_type'
ON CONFLICT (category_id, code) DO NOTHING;

INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
SELECT id, NULL, NULL, 'Sales Order (alias)', 'sales_order', 16
FROM type_categories WHERE name = 'order_type'
ON CONFLICT (category_id, code) DO NOTHING;

-- =====================================================================
-- T. Add missing recurrence_cadence codes (snake_case aliases)
-- =====================================================================

-- Already has: hourly, daily, weekly, monthly, yearly (all snake_case compatible)

-- =====================================================================
-- U. Add missing setting_value_type codes
-- =====================================================================

-- Already has: string, number, boolean, json, array (all snake_case compatible)

-- =====================================================================
-- V. Verification
-- =====================================================================

DO $$
DECLARE
    cat_count INTEGER;
    entry_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO cat_count FROM type_categories;
    SELECT COUNT(*) INTO entry_count FROM type_entries;
    RAISE NOTICE 'Migration 0035: Total % type categories and % type entries', cat_count, entry_count;
END $$;
