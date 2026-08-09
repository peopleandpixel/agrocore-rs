CREATE TABLE IF NOT EXISTS iot_devices (
    device_id TEXT PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id) ON DELETE SET NULL,
    device JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_iot_devices_tenant ON iot_devices(tenant_id);
CREATE INDEX IF NOT EXISTS idx_iot_devices_site ON iot_devices(site_id);

CREATE TRIGGER set_updated_at_iot_devices
BEFORE UPDATE ON iot_devices
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
