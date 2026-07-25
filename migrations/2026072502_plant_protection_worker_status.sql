-- Plant Protection Records
CREATE TABLE IF NOT EXISTS plant_protection_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    applicator_id UUID REFERENCES users(id) ON DELETE SET NULL,
    product_name TEXT NOT NULL,
    active_substance TEXT NOT NULL,
    concentration DOUBLE PRECISION,
    dosage_per_ha DOUBLE PRECISION NOT NULL,
    area_ha DOUBLE PRECISION NOT NULL,
    application_date TIMESTAMPTZ NOT NULL,
    pre_harvest_interval_days INTEGER,
    re_entry_interval_days INTEGER,
    weather_conditions JSONB,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_plant_protection_tenant ON plant_protection_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_plant_protection_site ON plant_protection_records(site_id);
CREATE INDEX IF NOT EXISTS idx_plant_protection_date ON plant_protection_records(application_date);

CREATE TRIGGER set_updated_at_plant_protection BEFORE UPDATE ON plant_protection_records FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Worker Task Statuses
CREATE TABLE IF NOT EXISTS worker_task_statuses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    worker_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(task_id, worker_id)
);

CREATE INDEX IF NOT EXISTS idx_worker_task_status_tenant ON worker_task_statuses(tenant_id);
CREATE INDEX IF NOT EXISTS idx_worker_task_status_task ON worker_task_statuses(task_id);
CREATE INDEX IF NOT EXISTS idx_worker_task_status_worker ON worker_task_statuses(worker_id);

CREATE TRIGGER set_updated_at_worker_task_status BEFORE UPDATE ON worker_task_statuses FOR EACH ROW EXECUTE FUNCTION set_updated_at();