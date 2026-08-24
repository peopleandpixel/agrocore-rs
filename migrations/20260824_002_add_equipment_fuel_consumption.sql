-- Add fuel consumption tracking for equipment
-- Tracks liters, cost, operation context, and field association

ALTER TABLE equipment
    ADD COLUMN IF NOT EXISTS fuel_capacity_liters NUMERIC(10,2) DEFAULT 0,
    ADD COLUMN IF NOT EXISTS fuel_type TEXT;

CREATE TABLE IF NOT EXISTS equipment_fuel_consumption (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    liters NUMERIC(10,3) NOT NULL, -- fuel volume in liters (3 decimal precision for accuracy)
    cost_per_liter NUMERIC(10,2) DEFAULT 0, -- cost in tenant currency per liter
    total_cost NUMERIC(10,2) GENERATED ALWAYS AS (liters * cost_per_liter) STORED,
    operation_type TEXT, -- "field_work", "transport", "standby", etc.
    field_id UUID REFERENCES sites(id) ON DELETE SET NULL,
    hours_operated NUMERIC(10,2) DEFAULT 0, -- hours equipment ran during this consumption
    consumed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    recorded_by UUID, -- user who recorded it
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fuel_consumption_equipment ON equipment_fuel_consumption(equipment_id);
CREATE INDEX IF NOT EXISTS idx_fuel_consumption_tenant ON equipment_fuel_consumption(tenant_id);
CREATE INDEX IF NOT EXISTS idx_fuel_consumption_date ON equipment_fuel_consumption(consumed_at);
CREATE INDEX IF NOT EXISTS idx_equipment_fuel_capacity ON equipment(fuel_capacity_liters);

COMMENT ON TABLE equipment_fuel_consumption IS 'Tracks fuel usage per equipment with operation context and optional field association';
COMMENT ON COLUMN equipment.fuel_capacity_liters IS 'Maximum fuel tank capacity in liters';
