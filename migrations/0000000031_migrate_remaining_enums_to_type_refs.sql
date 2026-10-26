-- Migration 0031: Migrate remaining enum fields to type_entries UUID references
-- Adds UUID columns for: harvest_lots, animals, livestock, growing_degree_days, breeds

-- ============================================================
-- harvest_lots: crop_type -> crop_type_id
-- ============================================================
ALTER TABLE harvest_lots ADD COLUMN IF NOT EXISTS crop_type_id UUID;
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'harvest_lots' AND column_name = 'crop_type') THEN
        UPDATE harvest_lots SET crop_type_id = (
            SELECT id FROM type_entries
            WHERE code = LOWER(crop_type)
              AND category_id = (SELECT id FROM type_categories WHERE code = 'crop_types')
            LIMIT 1
        ) WHERE crop_type IS NOT NULL AND crop_type != '';
    END IF;
END $$;
CREATE INDEX IF NOT EXISTS idx_harvest_lots_crop_type_id ON harvest_lots(crop_type_id);

-- ============================================================
-- animals: livestock_type, species, status -> UUID refs
-- ============================================================
ALTER TABLE animals ADD COLUMN IF NOT EXISTS livestock_type_id UUID;
ALTER TABLE animals ADD COLUMN IF NOT EXISTS species_id UUID;
ALTER TABLE animals ADD COLUMN IF NOT EXISTS status_id UUID;

-- Migrate livestock_type
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'animals' AND column_name = 'livestock_type') THEN
        UPDATE animals SET livestock_type_id = (
            SELECT id FROM type_entries
            WHERE code = LOWER(livestock_type)
              AND category_id = (SELECT id FROM type_categories WHERE code = 'livestock_types')
            LIMIT 1
        ) WHERE livestock_type IS NOT NULL AND livestock_type != '';
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'animals' AND column_name = 'species') THEN
        UPDATE animals SET species_id = (
            SELECT id FROM type_entries
            WHERE code = LOWER(species)
              AND category_id = (SELECT id FROM type_categories WHERE code = 'species')
            LIMIT 1
        ) WHERE species IS NOT NULL AND species != '';
    END IF;
END $$;

-- Migrate status (default to 'active')
UPDATE animals SET status_id = (
    SELECT id FROM type_entries
    WHERE code = 'active'
      AND category_id = (SELECT id FROM type_categories WHERE code = 'animal_status')
    LIMIT 1
) WHERE status_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_animals_livestock_type_id ON animals(livestock_type_id);
CREATE INDEX IF NOT EXISTS idx_animals_species_id ON animals(species_id);
CREATE INDEX IF NOT EXISTS idx_animals_status_id ON animals(status_id);

-- ============================================================
-- livestock: livestock_type -> livestock_type_id
-- ============================================================
ALTER TABLE livestock ADD COLUMN IF NOT EXISTS livestock_type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'livestock' AND column_name = 'livestock_type') THEN
        UPDATE livestock SET livestock_type_id = (
            SELECT id FROM type_entries
            WHERE code = LOWER(livestock_type)
              AND category_id = (SELECT id FROM type_categories WHERE code = 'livestock_types')
            LIMIT 1
        ) WHERE livestock_type IS NOT NULL AND livestock_type != '';
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_livestock_livestock_type_id ON livestock(livestock_type_id);

-- ============================================================
-- growing_degree_days: crop_type -> crop_type_id
-- ============================================================
ALTER TABLE growing_degree_days ADD COLUMN IF NOT EXISTS crop_type_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'growing_degree_days' AND column_name = 'crop_type') THEN
        UPDATE growing_degree_days SET crop_type_id = (
            SELECT id FROM type_entries
            WHERE code = LOWER(crop_type)
              AND category_id = (SELECT id FROM type_categories WHERE code = 'crop_types')
            LIMIT 1
        ) WHERE crop_type IS NOT NULL AND crop_type != '';
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_growing_degree_days_crop_type_id ON growing_degree_days(crop_type_id);

-- ============================================================
-- breeds: species -> species_id
-- ============================================================
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS species_id UUID;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'breeds' AND column_name = 'species_key') THEN
        UPDATE breeds SET species_id = (
            SELECT id FROM type_entries
            WHERE code = LOWER(species_key)
              AND category_id = (SELECT id FROM type_categories WHERE code = 'species')
            LIMIT 1
        ) WHERE species_key IS NOT NULL AND species_key != '';
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_breeds_species_id ON breeds(species_id);

-- ============================================================
-- olive_oil_records: remove redundant quality_grade (has oil_grade_id)
-- ============================================================
ALTER TABLE olive_oil_records DROP COLUMN IF EXISTS quality_grade;
