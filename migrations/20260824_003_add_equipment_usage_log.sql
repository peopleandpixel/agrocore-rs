-- Migration: Add Equipment Usage Logging (Nutzung-Logging)
-- Tracks who used which equipment, when, and for how long.

-- Table: equipment_usage_log — records individual equipment usage sessions
CREATE TABLE equipment_usage_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id    UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    tenant_id       UUID NOT NULL,
    worker_id       UUID,
    task_id         UUID,
    operation_type  VARCHAR(100),
    started_at      TIMESTAMPTZ NOT NULL,
    ended_at        TIMESTAMPTZ,
    hours_operated  NUMERIC(8,2) DEFAULT 0.00,
    note            TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_equipment_usage_log_equipment_id ON equipment_usage_log(equipment_id);
CREATE INDEX idx_equipment_usage_log_tenant_id ON equipment_usage_log(tenant_id);
CREATE INDEX idx_equipment_usage_log_worker_id ON equipment_usage_log(worker_id);
CREATE INDEX idx_equipment_usage_log_task_id ON equipment_usage_log(task_id);
CREATE INDEX idx_equipment_usage_log_started_at ON equipment_usage_log(started_at);
CREATE INDEX idx_equipment_usage_log_ended_at ON equipment_usage_log(ended_at);
CREATE INDEX idx_equipment_usage_log_operation_type ON equipment_usage_log(operation_type);
CREATE INDEX idx_equipment_usage_log_equipment_started ON equipment_usage_log(equipment_id, started_at DESC);

-- Trigger: auto-update updated_at on row update
CREATE TRIGGER update_equipment_usage_log_updated_at
    BEFORE UPDATE ON equipment_usage_log
    FOR EACH ROW
    EXECUTE FUNCTION trigger_updated_at();

-- Comment on table
COMMENT ON TABLE equipment_usage_log IS 'Tracks equipment usage: who, what operation, when started/finished, duration.';
