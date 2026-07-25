-- Core additional tables
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    old_data JSONB,
    new_data JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant ON audit_logs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created ON audit_logs(created_at DESC);

CREATE TABLE IF NOT EXISTS cost_centers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES cost_centers(id) ON DELETE SET NULL,
    code TEXT NOT NULL,
    label TEXT NOT NULL,
    description TEXT,
    budget_eur DOUBLE PRECISION,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, code)
);

CREATE INDEX IF NOT EXISTS idx_cost_centers_tenant ON cost_centers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_cost_centers_parent ON cost_centers(parent_id);

CREATE TABLE IF NOT EXISTS financial_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    cost_center_id UUID REFERENCES cost_centers(id) ON DELETE SET NULL,
    site_id UUID REFERENCES sites(id) ON DELETE SET NULL,
    record_type TEXT NOT NULL,
    amount_eur DOUBLE PRECISION NOT NULL,
    currency TEXT DEFAULT 'EUR',
    date TIMESTAMPTZ NOT NULL,
    description TEXT,
    invoice_ref TEXT,
    external_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_financial_records_tenant ON financial_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_financial_records_cost_center ON financial_records(cost_center_id);
CREATE INDEX IF NOT EXISTS idx_financial_records_site ON financial_records(site_id);
CREATE INDEX IF NOT EXISTS idx_financial_records_date ON financial_records(date);

CREATE TABLE IF NOT EXISTS pac_applications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    application_number TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    total_area_ha DOUBLE PRECISION,
    declared_area_ha DOUBLE PRECISION,
    subsidy_amount_eur DOUBLE PRECISION,
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, year)
);

CREATE INDEX IF NOT EXISTS idx_pac_applications_tenant ON pac_applications(tenant_id);
CREATE INDEX IF NOT EXISTS idx_pac_applications_year ON pac_applications(year);

-- Triggers for updated_at
CREATE TRIGGER set_updated_at_audit_logs BEFORE UPDATE ON audit_logs FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_cost_centers BEFORE UPDATE ON cost_centers FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_financial_records BEFORE UPDATE ON financial_records FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_pac_applications BEFORE UPDATE ON pac_applications FOR EACH ROW EXECUTE FUNCTION set_updated_at();