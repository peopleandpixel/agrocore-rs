-- Migration for Applicator Licenses
CREATE TYPE license_type AS ENUM ('basic', 'advanced', 'professional', 'custom');

CREATE TABLE applicator_licenses (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    license_type license_type NOT NULL,
    license_number VARCHAR(255) NOT NULL,
    issued_by VARCHAR(255) NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL,
    valid_until TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_applicator_licenses_tenant ON applicator_licenses(tenant_id);
CREATE INDEX idx_applicator_licenses_user ON applicator_licenses(user_id);
CREATE INDEX idx_applicator_licenses_active ON applicator_licenses(is_active);
CREATE INDEX idx_applicator_licenses_valid_until ON applicator_licenses(valid_until);

-- Enable RLS
ALTER TABLE applicator_licenses ENABLE ROW LEVEL SECURITY;

-- RLS policy for tenant isolation
CREATE POLICY applicator_licenses_tenant_isolation ON applicator_licenses
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

COMMENT ON TABLE applicator_licenses IS 'Applicator licenses for plant protection operations';
COMMENT ON COLUMN applicator_licenses.license_type IS 'License type: basic, advanced, professional, custom';