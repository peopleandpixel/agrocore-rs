-- Migration 41: Migrate varieties and breeds to type_entries system
--
-- Replaces the standalone `varieties` and `breeds` tables with the dynamic
-- type_entries system. This enables:
-- 1. UI management via searchable_type_dropdown with "Add new type" modal
-- 2. Consistent hierarchy: crop_varieties → Grape → Merlot, Cabernet, ...
-- 3. Consistent hierarchy: animal_breeds → Sheep → Merino, Romney, ...
-- 4. Global entries (tenant_id = NULL) visible to all tenants
-- 5. Tenant-specific entries for custom varieties/breeds
--
-- The old tables are preserved but deprecated for backward compatibility.
-- New code uses type_entries exclusively.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- 1. Create new type categories
-- ============================================================================

INSERT INTO type_categories (name, label, description, is_hierarchical) VALUES
('crop_varieties', 'Crop Varieties', 'Plant varieties and cultivars (grape, olive, etc.)', true),
('animal_breeds', 'Animal Breeds', 'Animal breeds (sheep, cattle, poultry, etc.)', true)
ON CONFLICT (name) DO NOTHING;

-- ============================================================================
-- 2. Migrate varieties to type_entries (crop_varieties category)
-- ============================================================================

-- First, create parent nodes for variety categories (Grape, Olive, etc.)
-- These serve as hierarchy roots for filtering in the UI.
DO $$
DECLARE
    cat_crop_varieties UUID;
    var_row RECORD;
    parent_id UUID;
    code_val TEXT;
BEGIN
    -- Get the crop_varieties category ID
    SELECT id INTO cat_crop_varieties FROM type_categories WHERE name = 'crop_varieties';
    
    IF cat_crop_varieties IS NULL THEN
        RAISE NOTICE 'crop_varieties category not found, skipping migration';
        RETURN;
    END IF;
    
    -- Migrate each variety row to type_entries
    FOR var_row IN 
        SELECT id, tenant_id, category_id, name, origin, species_key, active, created_at
        FROM varieties
        WHERE active IS NOT FALSE  -- Only migrate active varieties
    LOOP
        -- Generate a code from the variety name (slugified)
        code_val := lower(regexp_replace(var_row.name, '[^a-z0-9]+', '-', 'g'));
        code_val := trim(both '-' from code_val);
        IF code_val = '' THEN
            code_val := 'variety_' || var_row.id::text;
        END IF;
        
        -- Check if there's a category parent (species_key or category_id)
        parent_id := NULL;
        
        -- Try to find a parent entry based on species_key or category_id
        IF var_row.species_key IS NOT NULL AND var_row.species_key != '' THEN
            -- Look for an existing parent entry with this species_key as code
            SELECT id INTO parent_id FROM type_entries
            WHERE category_id = cat_crop_varieties
              AND code = var_row.species_key
              AND parent_id IS NULL
              AND tenant_id IS NULL
            LIMIT 1;
            
            -- If no parent exists, create one from the category_id or species_key
            IF parent_id IS NULL AND var_row.category_id IS NOT NULL THEN
                -- Create the parent node (e.g., "Grape", "Olive")
                INSERT INTO type_entries (category_id, tenant_id, name, code, description, is_active, sort_order)
                VALUES (
                    cat_crop_varieties,
                    NULL,  -- Global parent
                    var_row.category_id::text,
                    lower(regexp_replace(var_row.category_id::text, '[^a-z0-9]+', '-', 'g')),
                    'Variety category: ' || var_row.category_id,
                    true,
                    0
                )
                RETURNING id INTO parent_id;
            END IF;
        END IF;
        
        -- Insert the variety as a type_entry
        -- tenant_id: sentinel UUID → NULL (global), real UUID → tenant-specific
        INSERT INTO type_entries (
            category_id,
            parent_id,
            tenant_id,
            name,
            code,
            description,
            is_active,
            sort_order,
            created_at,
            updated_at
        ) VALUES (
            cat_crop_varieties,
            parent_id,
            CASE 
                WHEN var_row.tenant_id = '00000000-0000-0000-0000-000000000000'::uuid THEN NULL
                ELSE var_row.tenant_id
            END,
            var_row.name,
            code_val,
            var_row.origin,
            COALESCE(var_row.active, true),
            0,
            var_row.created_at,
            NOW()
        )
        ON CONFLICT (category_id, code, tenant_id) DO NOTHING;
    END LOOP;
END $$;

-- ============================================================================
-- 3. Migrate breeds to type_entries (animal_breeds category)
-- ============================================================================

DO $$
DECLARE
    cat_animal_breeds UUID;
    breed_row RECORD;
    parent_id UUID;
    code_val TEXT;
BEGIN
    -- Get the animal_breeds category ID
    SELECT id INTO cat_animal_breeds FROM type_categories WHERE name = 'animal_breeds';
    
    IF cat_animal_breeds IS NULL THEN
        RAISE NOTICE 'animal_breeds category not found, skipping migration';
        RETURN;
    END IF;
    
    -- Migrate each breed row to type_entries
    FOR breed_row IN 
        SELECT id, tenant_id, species_id, name, origin, species_key, active, created_at
        FROM breeds
        WHERE active IS NOT FALSE  -- Only migrate active breeds
    LOOP
        -- Generate a code from the breed name (slugified)
        code_val := lower(regexp_replace(breed_row.name, '[^a-z0-9]+', '-', 'g'));
        code_val := trim(both '-' from code_val);
        IF code_val = '' THEN
            code_val := 'breed_' || breed_row.id::text;
        END IF;
        
        -- Check if there's a species parent
        parent_id := NULL;
        
        -- Try to find a parent entry based on species_id or species_key
        IF breed_row.species_key IS NOT NULL AND breed_row.species_key != '' THEN
            -- Look for an existing parent entry with this species_key as code
            SELECT id INTO parent_id FROM type_entries
            WHERE category_id = cat_animal_breeds
              AND code = breed_row.species_key
              AND parent_id IS NULL
              AND tenant_id IS NULL
            LIMIT 1;
            
            -- If no parent exists and we have a species_id, try to find the species name
            IF parent_id IS NULL AND breed_row.species_id IS NOT NULL THEN
                -- Try to get the species name from type_entries (species category)
                DECLARE
                    species_name TEXT;
                    species_code TEXT;
                BEGIN
                    SELECT name INTO species_name FROM type_entries
                    WHERE category_id = (SELECT id FROM type_categories WHERE name = 'species')
                      AND id = breed_row.species_id
                    LIMIT 1;
                    
                    IF species_name IS NOT NULL THEN
                        species_code := lower(regexp_replace(species_name, '[^a-z0-9]+', '-', 'g'));
                        
                        -- Create the parent node (e.g., "Sheep", "Cattle")
                        INSERT INTO type_entries (category_id, tenant_id, name, code, description, is_active, sort_order)
                        VALUES (
                            cat_animal_breeds,
                            NULL,  -- Global parent
                            species_name,
                            species_code,
                            'Breed species: ' || species_name,
                            true,
                            0
                        )
                        RETURNING id INTO parent_id;
                    END IF;
                END;
            END IF;
        END IF;
        
        -- Insert the breed as a type_entry
        INSERT INTO type_entries (
            category_id,
            parent_id,
            tenant_id,
            name,
            code,
            description,
            is_active,
            sort_order,
            created_at,
            updated_at
        ) VALUES (
            cat_animal_breeds,
            parent_id,
            CASE 
                WHEN breed_row.tenant_id = '00000000-0000-0000-0000-000000000000'::uuid THEN NULL
                ELSE breed_row.tenant_id
            END,
            breed_row.name,
            code_val,
            breed_row.origin,
            COALESCE(breed_row.active, true),
            0,
            breed_row.created_at,
            NOW()
        )
        ON CONFLICT (category_id, code, tenant_id) DO NOTHING;
    END LOOP;
END $$;

-- ============================================================================
-- 4. Add breed_id to animals table (references type_entries)
-- ============================================================================

-- Add breed_id column that references type_entries (animal_breeds category)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'animals' AND column_name = 'breed_id'
    ) THEN
        ALTER TABLE animals ADD COLUMN breed_id UUID REFERENCES type_entries(id) ON DELETE SET NULL;
    END IF;
END $$;

-- Migrate existing free-text breed values to type_entries references
-- This is a best-effort match: we try to find the breed by name
DO $$
DECLARE
    animal_row RECORD;
    cat_animal_breeds UUID;
    matched_breed_id UUID;
BEGIN
    SELECT id INTO cat_animal_breeds FROM type_categories WHERE name = 'animal_breeds';
    
    IF cat_animal_breeds IS NULL THEN
        RETURN;
    END IF;
    
    -- For each animal with a free-text breed, try to match it to a type_entry
    FOR animal_row IN 
        SELECT id, breed, species
        FROM animals
        WHERE breed IS NOT NULL AND breed != '' AND breed_id IS NULL
        LIMIT 10000  -- Safety limit
    LOOP
        -- Try to find a matching type_entry by name
        SELECT id INTO matched_breed_id FROM type_entries
        WHERE category_id = cat_animal_breeds
          AND lower(name) = lower(animal_row.breed)
          AND is_active
        LIMIT 1;
        
        -- If found, update the animal
        IF matched_breed_id IS NOT NULL THEN
            UPDATE animals SET breed_id = matched_breed_id WHERE id = animal_row.id;
        END IF;
    END LOOP;
END $$;

CREATE INDEX IF NOT EXISTS idx_animals_breed_id ON animals(breed_id) WHERE breed_id IS NOT NULL;

-- ============================================================================
-- 5. Update spatial_objects.variety_id FK to reference type_entries
-- ============================================================================

-- The variety_id column currently references varieties(id).
-- We need to remap the UUIDs to the new type_entries IDs.

-- Create a mapping table to track old variety_id → new type_entry_id
CREATE TEMPORARY TABLE variety_mapping (
    old_variety_id UUID PRIMARY KEY,
    new_type_entry_id UUID NOT NULL
);

-- Populate the mapping based on the code we generated during migration
DO $$
DECLARE
    cat_crop_varieties UUID;
    var_row RECORD;
    code_val TEXT;
    new_entry_id UUID;
BEGIN
    SELECT id INTO cat_crop_varieties FROM type_categories WHERE name = 'crop_varieties';
    
    IF cat_crop_varieties IS NULL THEN
        RETURN;
    END IF;
    
    FOR var_row IN 
        SELECT id, name, tenant_id, category_id, species_key, active
        FROM varieties
        WHERE active IS NOT FALSE
    LOOP
        -- Generate the same code as in the migration
        code_val := lower(regexp_replace(var_row.name, '[^a-z0-9]+', '-', 'g'));
        code_val := trim(both '-' from code_val);
        IF code_val = '' THEN
            code_val := 'variety_' || var_row.id::text;
        END IF;
        
        -- Find the corresponding type_entry
        SELECT id INTO new_entry_id FROM type_entries
        WHERE category_id = cat_crop_varieties
          AND code = code_val
          AND tenant_id = CASE 
              WHEN var_row.tenant_id = '00000000-0000-0000-0000-000000000000'::uuid THEN NULL
              ELSE var_row.tenant_id
          END
        LIMIT 1;
        
        -- Insert into mapping if found
        IF new_entry_id IS NOT NULL THEN
            INSERT INTO variety_mapping (old_variety_id, new_type_entry_id)
            VALUES (var_row.id, new_entry_id)
            ON CONFLICT (old_variety_id) DO NOTHING;
        END IF;
    END LOOP;
END $$;

-- Update spatial_objects to use the new type_entry IDs
-- variety_id column will be reused to store type_entry IDs
UPDATE spatial_objects so
SET variety_id = vm.new_type_entry_id
FROM variety_mapping vm
WHERE so.variety_id = vm.old_variety_id;

-- Drop the old FK constraint and recreate it pointing to type_entries
DO $$
BEGIN
    -- Drop old FK
    ALTER TABLE spatial_objects DROP CONSTRAINT IF EXISTS spatial_objects_variety_id_fkey;
    
    -- Add new FK to type_entries
    ALTER TABLE spatial_objects ADD CONSTRAINT spatial_objects_variety_id_fkey
        FOREIGN KEY (variety_id) REFERENCES type_entries(id) ON DELETE SET NULL;
END $$;

-- ============================================================================
-- 6. Mark old tables as deprecated
-- ============================================================================

COMMENT ON TABLE varieties IS 
'DEPRECATED (v0.60.0): Use type_entries with category "crop_varieties" instead. This table is preserved for backward compatibility but will be removed in a future version.';

COMMENT ON TABLE breeds IS 
'DEPRECATED (v0.60.0): Use type_entries with category "animal_breeds" instead. This table is preserved for backward compatibility but will be removed in a future version.';

-- ============================================================================
-- 7. Update materialized views to use type_entries
-- ============================================================================

-- Drop and recreate the olive trees materialized view to use type_entries
DROP MATERIALIZED VIEW IF EXISTS all_olive_trees CASCADE;
DROP MATERIALIZED VIEW IF EXISTS olive_trees_by_variety CASCADE;

CREATE MATERIALIZED VIEW IF NOT EXISTS all_olive_trees AS
SELECT
    so.id,
    so.tenant_id,
    so.site_id,
    so.label,
    so.geometry,
    so.area,
    so.buffer_meters,
    so.planted_at,
    so.variety_id,
    te.name AS variety_label,
    te.code AS variety_code,
    s.name AS site_label,
    s.area AS site_area_ha,
    so.created_at,
    so.updated_at
FROM spatial_objects so
LEFT JOIN sites s ON s.id = so.site_id AND s.tenant_id = so.tenant_id
LEFT JOIN type_entries te ON te.id = so.variety_id
WHERE so.object_type = 'olive_tree'
  AND so.is_active;

CREATE INDEX IF NOT EXISTS idx_all_olive_trees_tenant ON all_olive_trees (tenant_id);
CREATE INDEX IF NOT EXISTS idx_all_olive_trees_site ON all_olive_trees (site_id);
CREATE INDEX IF NOT EXISTS idx_all_olive_trees_variety ON all_olive_trees (variety_id);
CREATE INDEX IF NOT EXISTS idx_all_olive_trees_geometry ON all_olive_trees USING GIST (geometry);

CREATE MATERIALIZED VIEW IF NOT EXISTS olive_trees_by_variety AS
SELECT
    te.id AS variety_id,
    te.tenant_id,
    te.name AS variety_label,
    te.code AS variety_code,
    COUNT(so.id) AS tree_count,
    COALESCE(SUM(so.area), 0) AS total_area,
    COUNT(DISTINCT so.site_id) AS site_count
FROM spatial_objects so
JOIN type_entries te ON te.id = so.variety_id
WHERE so.object_type = 'olive_tree'
  AND so.is_active
GROUP BY te.id, te.tenant_id, te.name, te.code;

CREATE UNIQUE INDEX IF NOT EXISTS idx_olive_trees_by_variety_id ON olive_trees_by_variety (variety_id);
CREATE INDEX IF NOT EXISTS idx_olive_trees_by_variety_tenant ON olive_trees_by_variety (tenant_id);

-- ============================================================================
-- 8. Clean up temporary mapping table
-- ============================================================================

DROP TABLE IF EXISTS variety_mapping;

-- ============================================================================
-- Summary
-- ============================================================================
-- - crop_varieties category created with parent nodes for variety categories
-- - animal_breeds category created with parent nodes for species
-- - All active varieties migrated to type_entries (crop_varieties)
-- - All active breeds migrated to type_entries (animal_breeds)
-- - animals.breed_id column added (UUID → type_entries)
-- - spatial_objects.variety_id FK changed from varieties(id) to type_entries(id)
-- - Materialized views updated to use type_entries
-- - Old tables marked as deprecated
