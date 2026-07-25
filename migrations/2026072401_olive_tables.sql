-- Olive & Olive Oil Tables
CREATE TABLE IF NOT EXISTS olive_groves (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    variety TEXT NOT NULL,
    tree_count INTEGER,
    planting_year INTEGER,
    area_ha NUMERIC(10,2),
    spacing_m NUMERIC(10,2),
    irrigation_type TEXT,
    is_organic BOOLEAN DEFAULT false,
    certification_body TEXT,
    certification_number TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_olive_groves_tenant ON olive_groves(tenant_id);
CREATE INDEX IF NOT EXISTS idx_olive_groves_site ON olive_groves(site_id);

CREATE TABLE IF NOT EXISTS olive_oil_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    olive_grove_id UUID NOT NULL REFERENCES olive_groves(id) ON DELETE CASCADE,
    harvest_year INTEGER NOT NULL,
    harvest_date TIMESTAMPTZ NOT NULL,
    quantity_kg DOUBLE PRECISION,
    oil_yield_kg DOUBLE PRECISION,
    oil_yield_pct DOUBLE PRECISION,
    oil_grade TEXT,
    acidity_pct DOUBLE PRECISION,
    peroxide_value DOUBLE PRECISION,
    k232 DOUBLE PRECISION,
    k270 DOUBLE PRECISION,
    sensory_score DOUBLE PRECISION,
    liters_produced DOUBLE PRECISION,
    mill_name TEXT,
    lot_number TEXT,
    quality_grade TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_olive_oil_records_tenant ON olive_oil_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_olive_oil_records_grove ON olive_oil_records(olive_grove_id);
CREATE INDEX IF NOT EXISTS idx_olive_oil_records_harvest_year ON olive_oil_records(harvest_year);

-- Trigger für updated_at
CREATE TRIGGER set_updated_at_olive_groves BEFORE UPDATE ON olive_groves FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_olive_oil_records BEFORE UPDATE ON olive_oil_records FOR EACH ROW EXECUTE FUNCTION set_updated_at();