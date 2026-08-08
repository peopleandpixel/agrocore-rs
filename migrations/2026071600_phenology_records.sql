-- Create phenology_records table
-- This table is required by the domain model for PhenologyRecord entity

CREATE TABLE IF NOT EXISTS phenology_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    observation_date TIMESTAMPTZ NOT NULL,
    stage JSONB NOT NULL,  -- BbchStage as JSONB
    forecast_next_stage_date TIMESTAMPTZ,
    notes TEXT,
    photo_url TEXT,
    observer_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_phenology_records_tenant ON phenology_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_phenology_records_site ON phenology_records(site_id);
CREATE INDEX IF NOT EXISTS idx_phenology_records_observation_date ON phenology_records(observation_date);

-- Trigger for updated_at (if needed, though this table doesn't have updated_at column)
-- DO $$
-- BEGIN
--     IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'set_updated_at_phenology_records') THEN
--         CREATE TRIGGER set_updated_at_phenology_records BEFORE UPDATE ON phenology_records FOR EACH ROW EXECUTE FUNCTION set_updated_at();
--     END IF;
-- END $$;