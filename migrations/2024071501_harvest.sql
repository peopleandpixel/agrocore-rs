-- Ernte-Logistik Tabellen
CREATE TABLE harvest_seasons (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    label TEXT NOT NULL,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE harvest_lots (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    season_id UUID NOT NULL REFERENCES harvest_seasons(id) ON DELETE CASCADE,
    lot_number TEXT NOT NULL,
    site_ids UUID[] NOT NULL DEFAULT '{}',
    crop_type TEXT NOT NULL,
    variety TEXT,
    quality_target TEXT,
    total_weight_kg DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE harvest_deliveries (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    lot_id UUID NOT NULL REFERENCES harvest_lots(id) ON DELETE CASCADE,
    delivery_date TIMESTAMPTZ NOT NULL,
    gross_weight_kg DOUBLE PRECISION NOT NULL,
    tare_weight_kg DOUBLE PRECISION NOT NULL,
    net_weight_kg DOUBLE PRECISION NOT NULL,
    carrier_name TEXT,
    vehicle_id TEXT,
    quality_notes TEXT,
    temperature_at_delivery DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE cold_chain_logs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    lot_id UUID NOT NULL REFERENCES harvest_lots(id) ON DELETE CASCADE,
    sensor_id TEXT NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL,
    temperature_c DOUBLE PRECISION NOT NULL,
    humidity_pct DOUBLE PRECISION,
    location TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indizes
CREATE INDEX idx_harvest_seasons_tenant ON harvest_seasons(tenant_id);
CREATE INDEX idx_harvest_lots_season ON harvest_lots(season_id);
CREATE INDEX idx_harvest_lots_tenant ON harvest_lots(tenant_id);
CREATE INDEX idx_harvest_deliveries_lot ON harvest_deliveries(lot_id);
CREATE INDEX idx_cold_chain_logs_lot ON cold_chain_logs(lot_id);

-- Trigger für updated_at
CREATE TRIGGER set_updated_at_harvest_seasons BEFORE UPDATE ON harvest_seasons FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_harvest_lots BEFORE UPDATE ON harvest_lots FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_harvest_deliveries BEFORE UPDATE ON harvest_deliveries FOR EACH ROW EXECUTE FUNCTION set_updated_at();
