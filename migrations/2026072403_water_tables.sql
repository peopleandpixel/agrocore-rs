-- Water Management Tables
CREATE TABLE IF NOT EXISTS water_sources (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    source_type TEXT NOT NULL,
    name TEXT NOT NULL,
    capacity_m3 DOUBLE PRECISION,
    current_level_m3 DOUBLE PRECISION,
    license_number TEXT,
    license_expiry TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_water_sources_tenant ON water_sources(tenant_id);
CREATE INDEX IF NOT EXISTS idx_water_sources_site ON water_sources(site_id);

CREATE TABLE IF NOT EXISTS water_usage (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    source_id UUID NOT NULL REFERENCES water_sources(id) ON DELETE CASCADE,
    usage_date TIMESTAMPTZ NOT NULL,
    volume_m3 DOUBLE PRECISION NOT NULL,
    irrigation_method TEXT NOT NULL,
    efficiency_pct DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_water_usage_tenant ON water_usage(tenant_id);
CREATE INDEX IF NOT EXISTS idx_water_usage_source ON water_usage(source_id);
CREATE INDEX IF NOT EXISTS idx_water_usage_site ON water_usage(site_id);
CREATE INDEX IF NOT EXISTS idx_water_usage_date ON water_usage(usage_date);

CREATE TABLE IF NOT EXISTS water_quotas (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    source_id UUID NOT NULL REFERENCES water_sources(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    allocated_m3 DOUBLE PRECISION NOT NULL,
    used_m3 DOUBLE PRECISION NOT NULL DEFAULT 0,
    remaining_m3 DOUBLE PRECISION NOT NULL DEFAULT 0,
    comunidad_id UUID REFERENCES water_sources(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, source_id, site_id, year)
);

CREATE INDEX IF NOT EXISTS idx_water_quotas_tenant ON water_quotas(tenant_id);
CREATE INDEX IF NOT EXISTS idx_water_quotas_year ON water_quotas(year);

-- Trigger für updated_at
CREATE TRIGGER set_updated_at_water_sources BEFORE UPDATE ON water_sources FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_water_quotas BEFORE UPDATE ON water_quotas FOR EACH ROW EXECUTE FUNCTION set_updated_at();