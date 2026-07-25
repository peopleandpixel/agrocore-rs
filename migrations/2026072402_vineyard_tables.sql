-- Vineyard & Kelter Delivery Tables
CREATE TABLE IF NOT EXISTS vineyards (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    doc_area TEXT,
    vintage INTEGER,
    grape_variety TEXT,
    brix_at_harvest DOUBLE PRECISION,
    ph_at_harvest DOUBLE PRECISION,
    acidity DOUBLE PRECISION,
    yield_tons DOUBLE PRECISION,
    quality_grade TEXT,
    slope_percent DOUBLE PRECISION,
    altitude_m DOUBLE PRECISION,
    is_organic BOOLEAN DEFAULT false,
    certification_body TEXT,
    certification_number TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vineyards_tenant ON vineyards(tenant_id);
CREATE INDEX IF NOT EXISTS idx_vineyards_site ON vineyards(site_id);

CREATE TABLE IF NOT EXISTS kelter_deliveries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    vineyard_id UUID NOT NULL REFERENCES vineyards(id) ON DELETE CASCADE,
    delivery_date TIMESTAMPTZ NOT NULL,
    gross_weight_kg DOUBLE PRECISION NOT NULL,
    net_weight_kg DOUBLE PRECISION NOT NULL,
    lot_number TEXT NOT NULL,
    kelter_name TEXT NOT NULL,
    transport_company TEXT,
    temperature_c DOUBLE PRECISION,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_kelter_deliveries_vineyard ON kelter_deliveries(vineyard_id);
CREATE INDEX IF NOT EXISTS idx_kelter_deliveries_delivery_date ON kelter_deliveries(delivery_date);

-- Trigger für updated_at
CREATE TRIGGER set_updated_at_vineyards BEFORE UPDATE ON vineyards FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_kelter_deliveries BEFORE UPDATE ON kelter_deliveries FOR EACH ROW EXECUTE FUNCTION set_updated_at();