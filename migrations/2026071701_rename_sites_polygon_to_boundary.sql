-- Rename polygon column to boundary in sites table to match domain model and repository
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'sites' AND column_name = 'polygon') THEN
        ALTER TABLE sites RENAME COLUMN polygon TO boundary;
    END IF;
END $$;

-- Drop redundant index if it exists
DROP INDEX IF EXISTS idx_sites_polygon;

-- Ensure idx_sites_boundary exists on the new boundary column
CREATE INDEX IF NOT EXISTS idx_sites_boundary ON sites USING GIST(boundary);
