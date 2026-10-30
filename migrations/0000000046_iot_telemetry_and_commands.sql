-- Migration 0046: IoT Telemetry and Commands tables
-- J8: Replace stub implementations with real data storage

-- ============================================================================
-- IoT Telemetry: stores measurements reported by devices
-- ============================================================================

CREATE TABLE IF NOT EXISTS iot_telemetry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    capability TEXT NOT NULL,        -- e.g. "Temperature", "Humidity", "SoilMoisture"
    value DOUBLE PRECISION NOT NULL,
    unit TEXT NOT NULL,              -- e.g. "°C", "%", "lux"
    quality TEXT,                    -- optional quality indicator
    metadata JSONB,                  -- optional extra data
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_iot_telemetry_device ON iot_telemetry(device_id, received_at DESC);
CREATE INDEX IF NOT EXISTS idx_iot_telemetry_tenant ON iot_telemetry(tenant_id);
CREATE INDEX IF NOT EXISTS idx_iot_telemetry_capability ON iot_telemetry(capability);
CREATE INDEX IF NOT EXISTS idx_iot_telemetry_received ON iot_telemetry(received_at DESC);

-- Retention: auto-delete telemetry older than 90 days (adjustable via cron)
-- CREATE POLICY iot_telemetry_select ON iot_telemetry FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
-- CREATE POLICY iot_telemetry_insert ON iot_telemetry FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

-- ============================================================================
-- IoT Commands: stores commands sent to devices
-- ============================================================================

CREATE TABLE IF NOT EXISTS iot_commands (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    command_type TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'Pending',  -- Pending, Sent, Delivered, Completed, Failed, Timeout
    response_payload JSONB,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    timeout_seconds INTEGER,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_iot_commands_device ON iot_commands(device_id, requested_at DESC);
CREATE INDEX IF NOT EXISTS idx_iot_commands_tenant ON iot_commands(tenant_id);
CREATE INDEX IF NOT EXISTS idx_iot_commands_status ON iot_commands(status);
CREATE INDEX IF NOT EXISTS idx_iot_commands_requested ON iot_commands(requested_at DESC);

-- ============================================================================
-- RLS Policies
-- ============================================================================

ALTER TABLE iot_telemetry ENABLE ROW LEVEL SECURITY;
ALTER TABLE iot_commands ENABLE ROW LEVEL SECURITY;

CREATE POLICY iot_telemetry_select ON iot_telemetry FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY iot_telemetry_insert ON iot_telemetry FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY iot_telemetry_delete ON iot_telemetry FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY iot_commands_select ON iot_commands FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY iot_commands_insert ON iot_commands FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY iot_commands_update ON iot_commands FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
