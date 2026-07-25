-- Fertilizer & Compliance Tables
CREATE TABLE IF NOT EXISTS fertilizer_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    product_name TEXT NOT NULL,
    nutrient_n DOUBLE PRECISION NOT NULL,
    nutrient_p DOUBLE PRECISION NOT NULL,
    nutrient_k DOUBLE PRECISION NOT NULL,
    quantity_kg DOUBLE PRECISION NOT NULL,
    area_ha DOUBLE PRECISION NOT NULL,
    application_date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fertilizer_records_tenant ON fertilizer_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_fertilizer_records_site ON fertilizer_records(site_id);
CREATE INDEX IF NOT EXISTS idx_fertilizer_records_date ON fertilizer_records(application_date);

CREATE TABLE IF NOT EXISTS compliance_checklists (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    checklist_type TEXT NOT NULL,
    status TEXT NOT NULL,
    label TEXT,
    description TEXT,
    due_date TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_compliance_checklists_tenant ON compliance_checklists(tenant_id);
CREATE INDEX IF NOT EXISTS idx_compliance_checklists_type ON compliance_checklists(checklist_type);
CREATE INDEX IF NOT EXISTS idx_compliance_checklists_status ON compliance_checklists(status);

CREATE TABLE IF NOT EXISTS compliance_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    checklist_id UUID NOT NULL REFERENCES compliance_checklists(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    status TEXT NOT NULL,
    evidence_url TEXT,
    completed_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_compliance_items_checklist ON compliance_items(checklist_id);
CREATE INDEX IF NOT EXISTS idx_compliance_items_tenant ON compliance_items(tenant_id);

-- Trigger für updated_at
CREATE TRIGGER set_updated_at_fertilizer_records BEFORE UPDATE ON fertilizer_records FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_compliance_checklists BEFORE UPDATE ON compliance_checklists FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_compliance_items BEFORE UPDATE ON compliance_items FOR EACH ROW EXECUTE FUNCTION set_updated_at();