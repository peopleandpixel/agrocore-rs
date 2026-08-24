-- Add maintenance cost tracking fields to equipment_maintenance_log
-- Enables tracking of parts costs, labor hours, downtime for cost analysis

ALTER TABLE equipment_maintenance_log
    ADD COLUMN IF NOT EXISTS parts_cost NUMERIC(10,2) DEFAULT 0.00 NOT NULL,
    ADD COLUMN IF NOT EXISTS labor_hours NUMERIC(10,2) DEFAULT 0.00 NOT NULL,
    ADD COLUMN IF NOT EXISTS downtime_hours NUMERIC(10,2) DEFAULT 0.00 NOT NULL;

-- Index for cost analysis queries
CREATE INDEX IF NOT EXISTS idx_equipment_maintenance_cost_equipment
    ON equipment_maintenance_log(equipment_id, tenant_id);

CREATE INDEX IF NOT EXISTS idx_equipment_maintenance_performed_at
    ON equipment_maintenance_log(performed_at DESC);
