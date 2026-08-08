-- Livestock Tables


CREATE TABLE IF NOT EXISTS grazing_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_grazing_animal ON grazing_records(animal_id);
CREATE INDEX IF NOT EXISTS idx_grazing_site ON grazing_records(site_id);
CREATE INDEX IF NOT EXISTS idx_grazing_date ON grazing_records(start_date);

CREATE TABLE IF NOT EXISTS treatment_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
    date TIMESTAMPTZ NOT NULL,
    treatment_type TEXT NOT NULL,
    medication TEXT,
    dosage TEXT,
    veterinarian TEXT,
    withdrawal_days INTEGER,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_treatment_animal ON treatment_records(animal_id);
CREATE INDEX IF NOT EXISTS idx_treatment_date ON treatment_records(date);

-- Trigger für updated_at
CREATE TRIGGER set_updated_at_animals BEFORE UPDATE ON animals FOR EACH ROW EXECUTE FUNCTION set_updated_at();
