-- Migration: Create trigger_updated_at() function for automatic updated_at updates.
-- This function is required by all triggers that use EXECUTE FUNCTION trigger_updated_at();
-- It updates the `updated_at` column to the current timestamp on UPDATE.

CREATE OR REPLACE FUNCTION trigger_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to all tables with updated_at column that reference it
-- Note: This is a baseline trigger; individual table triggers must be created separately
-- or rely on this function being available.
COMMENT ON FUNCTION trigger_updated_at() IS 'Auto-updates updated_at column on UPDATE triggers';
