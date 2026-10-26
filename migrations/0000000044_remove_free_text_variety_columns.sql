-- Migration 0044: Remove all free-text variety columns and add missing variety_id to sites
-- Affects: sites, harvest_lots, olive_groves, vineyards
-- After this migration, all variety references use variety_id (FK to type_entries, crop_varieties category)

-- ============================================================================
-- 1. Sites: Add variety_id (was missing!) and drop free-text variety
-- ============================================================================

-- The variety_id column was referenced in code but never created in the DB
ALTER TABLE sites ADD COLUMN IF NOT EXISTS variety_id UUID REFERENCES type_entries(id);

-- Migrate existing variety text values to type_entries
INSERT INTO type_entries (id, category_id, tenant_id, name, code, description, is_active)
SELECT
    gen_random_uuid(),
    (SELECT id FROM type_categories WHERE code = 'crop_varieties' LIMIT 1),
    NULL,  -- Global entries, visible to all tenants
    s.variety,
    LOWER(REGEXP_REPLACE(s.variety, '[^a-z0-9]+', '-', 'g')),
    'Migrated from sites variety text',
    true
FROM (
    SELECT DISTINCT variety
    FROM sites
    WHERE variety IS NOT NULL AND variety != ''
) s
ON CONFLICT (category_id, code) DO NOTHING;

-- Update sites to reference the migrated type_entries
UPDATE sites s
SET variety_id = te.id
FROM type_entries te
WHERE te.category_id = (SELECT id FROM type_categories WHERE code = 'crop_varieties' LIMIT 1)
  AND te.code = LOWER(REGEXP_REPLACE(s.variety, '[^a-z0-9]+', '-', 'g'))
  AND s.variety IS NOT NULL
  AND s.variety_id IS NULL;

-- Drop the free-text variety column
ALTER TABLE sites DROP COLUMN IF EXISTS variety;

CREATE INDEX IF NOT EXISTS idx_sites_variety_id ON sites(variety_id);

-- ============================================================================
-- 2. Harvest Lots: Drop free-text variety column
-- ============================================================================

ALTER TABLE harvest_lots DROP COLUMN IF EXISTS variety;

-- ============================================================================
-- 3. Olive Groves: Drop free-text variety column
-- ============================================================================

ALTER TABLE olive_groves DROP COLUMN IF EXISTS variety;

-- ============================================================================
-- 4. Vineyards: Drop free-text grape_variety column
-- ============================================================================

ALTER TABLE vineyards DROP COLUMN IF EXISTS grape_variety;

-- ============================================================================
-- 5. Update materialized views
-- ============================================================================

-- Refresh materialized views to reflect the schema changes
DO $$
BEGIN
    -- Refresh site-related materialized views if they exist
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'mv_site_summary') THEN
        REFRESH MATERIALIZED VIEW mv_site_summary;
    END IF;
END $$;
