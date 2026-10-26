-- Migration 0043: Migrate variety free-text fields to type_entries references
-- Affects: harvest_lots, olive_groves, vineyards
-- All three tables will use variety_id (FK to type_entries) instead of free-text variety/grape_variety

-- ============================================================================
-- 1. Harvest Lots: Add variety_id, keep variety as deprecated
-- ============================================================================

ALTER TABLE harvest_lots ADD COLUMN IF NOT EXISTS variety_id UUID REFERENCES type_entries(id);
ALTER TABLE harvest_lots ADD COLUMN IF NOT EXISTS site_ids UUID[];

-- Migrate existing variety text to type_entries if possible
-- First, ensure crop_varieties category exists
INSERT INTO type_categories (id, name, label, description, is_hierarchical, is_active)
VALUES (
    gen_random_uuid(),
    'crop_varieties',
    'Crop Varieties',
    'Plant varieties and cultivars (grape, olive, etc.)',
    true,
    true
) ON CONFLICT (name) DO NOTHING;

-- Migrate existing variety text values to type_entries
INSERT INTO type_entries (id, category_id, tenant_id, name, code, description, is_active)
SELECT
    gen_random_uuid(),
    (SELECT id FROM type_categories WHERE code = 'crop_varieties' LIMIT 1),
    NULL,  -- Global entries, visible to all tenants
    hl.variety,
    LOWER(REGEXP_REPLACE(hl.variety, '[^a-z0-9]+', '-', 'g')),
    'Migrated from harvest_lots variety text',
    true
FROM (
    SELECT DISTINCT variety
    FROM harvest_lots
    WHERE variety IS NOT NULL AND variety != ''
) hl
ON CONFLICT (category_id, code) DO NOTHING;

-- Update harvest_lots to reference the migrated type_entries
UPDATE harvest_lots hl
SET variety_id = te.id
FROM type_entries te
WHERE te.category_id = (SELECT id FROM type_categories WHERE code = 'crop_varieties' LIMIT 1)
  AND te.code = LOWER(REGEXP_REPLACE(hl.variety, '[^a-z0-9]+', '-', 'g'))
  AND hl.variety IS NOT NULL
  AND hl.variety_id IS NULL;

-- ============================================================================
-- 2. Olive Groves: Add variety_id, keep variety as deprecated
-- ============================================================================

ALTER TABLE olive_groves ADD COLUMN IF NOT EXISTS variety_id UUID REFERENCES type_entries(id);

-- Migrate existing variety text values to type_entries
INSERT INTO type_entries (id, category_id, tenant_id, name, code, description, is_active)
SELECT
    gen_random_uuid(),
    (SELECT id FROM type_categories WHERE code = 'crop_varieties' LIMIT 1),
    NULL,
    og.variety,
    LOWER(REGEXP_REPLACE(og.variety, '[^a-z0-9]+', '-', 'g')),
    'Migrated from olive_groves variety text',
    true
FROM (
    SELECT DISTINCT variety
    FROM olive_groves
    WHERE variety IS NOT NULL AND variety != ''
) og
ON CONFLICT (category_id, code) DO NOTHING;

-- Update olive_groves to reference the migrated type_entries
UPDATE olive_groves og
SET variety_id = te.id
FROM type_entries te
WHERE te.category_id = (SELECT id FROM type_categories WHERE code = 'crop_varieties' LIMIT 1)
  AND te.code = LOWER(REGEXP_REPLACE(og.variety, '[^a-z0-9]+', '-', 'g'))
  AND og.variety IS NOT NULL
  AND og.variety_id IS NULL;

-- Make variety nullable since variety_id will be the primary field
ALTER TABLE olive_groves ALTER COLUMN variety DROP NOT NULL;

-- ============================================================================
-- 3. Vineyards: Add variety_id, keep grape_variety as deprecated
-- ============================================================================

ALTER TABLE vineyards ADD COLUMN IF NOT EXISTS variety_id UUID REFERENCES type_entries(id);

-- Migrate existing grape_variety text values to type_entries
INSERT INTO type_entries (id, category_id, tenant_id, name, code, description, is_active)
SELECT
    gen_random_uuid(),
    (SELECT id FROM type_categories WHERE code = 'crop_varieties' LIMIT 1),
    NULL,
    v.grape_variety,
    LOWER(REGEXP_REPLACE(v.grape_variety, '[^a-z0-9]+', '-', 'g')),
    'Migrated from vineyards grape_variety text',
    true
FROM (
    SELECT DISTINCT grape_variety
    FROM vineyards
    WHERE grape_variety IS NOT NULL AND grape_variety != ''
) v
ON CONFLICT (category_id, code) DO NOTHING;

-- Update vineyards to reference the migrated type_entries
UPDATE vineyards v
SET variety_id = te.id
FROM type_entries te
WHERE te.category_id = (SELECT id FROM type_categories WHERE code = 'crop_varieties' LIMIT 1)
  AND te.code = LOWER(REGEXP_REPLACE(v.grape_variety, '[^a-z0-9]+', '-', 'g'))
  AND v.grape_variety IS NOT NULL
  AND v.variety_id IS NULL;

-- ============================================================================
-- 4. Indexes
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_harvest_lots_variety_id ON harvest_lots(variety_id);
CREATE INDEX IF NOT EXISTS idx_olive_groves_variety_id ON olive_groves(variety_id);
CREATE INDEX IF NOT EXISTS idx_vineyards_variety_id ON vineyards(variety_id);
