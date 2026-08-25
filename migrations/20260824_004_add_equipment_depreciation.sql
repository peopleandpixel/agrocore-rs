-- 20260824_004_add_equipment_depreciation.sql
-- Adds financial fields to equipment for depreciation tracking.
-- Supports straight-line amortization for financial reporting (Abschreibung).

-- Add financial columns to existing equipment table
ALTER TABLE equipment
ADD COLUMN IF NOT EXISTS original_cost NUMERIC(12,2) DEFAULT 0.0 NOT NULL,
ADD COLUMN IF NOT EXISTS salvage_value NUMERIC(12,2) DEFAULT 0.0 NOT NULL,
ADD COLUMN IF NOT EXISTS purchase_date DATE,
ADD COLUMN IF NOT EXISTS depreciation_method VARCHAR(20) DEFAULT 'straight_line' NOT NULL,
ADD COLUMN IF NOT EXISTS useful_life_years INTEGER DEFAULT 5 NOT NULL;

-- Depreciation schedule table — pre-calculated yearly depreciation values
-- This allows O(1) lookup of book value at any year without recomputation
CREATE TABLE IF NOT EXISTS equipment_depreciation_schedule (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    year INTEGER NOT NULL,             -- The fiscal year this entry applies to
    depreciation_amount NUMERIC(12,2) NOT NULL DEFAULT 0.0,
    accumulated_depreciation NUMERIC(12,2) NOT NULL DEFAULT 0.0,
    net_book_value NUMERIC(12,2) NOT NULL DEFAULT 0.0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(equipment_id, tenant_id, year)
);

-- Indexes for efficient lookup
CREATE INDEX IF NOT EXISTS idx_equipment_depreciation_equipment ON equipment_depreciation_schedule(equipment_id);
CREATE INDEX IF NOT EXISTS idx_equipment_depreciation_tenant ON equipment_depreciation_schedule(tenant_id);
CREATE INDEX IF NOT EXISTS idx_equipment_depreciation_year ON equipment_depreciation_schedule(year);
