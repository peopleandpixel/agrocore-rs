-- Inventory Management Tables
-- Multi-location inventory with batch/lot tracking, FIFO/FEFO, FEFO support

CREATE TABLE IF NOT EXISTS inventory_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    code TEXT,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_inventory_locations_tenant ON inventory_locations(tenant_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_inventory_locations_tenant_code ON inventory_locations(tenant_id, code) WHERE code IS NOT NULL;

CREATE TABLE IF NOT EXISTS inventory_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    category JSONB NOT NULL,
    name TEXT NOT NULL,
    sku TEXT,
    description TEXT,
    unit JSONB NOT NULL,
    minimum_stock DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    inventory_method JSONB NOT NULL DEFAULT '"FIFO"',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_inventory_items_tenant ON inventory_items(tenant_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_inventory_items_tenant_sku ON inventory_items(tenant_id, sku) WHERE sku IS NOT NULL;

CREATE TABLE IF NOT EXISTS inventory_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES inventory_items(id) ON DELETE CASCADE,
    transaction_type JSONB NOT NULL,
    quantity DOUBLE PRECISION NOT NULL,
    unit_cost DOUBLE PRECISION,
    total_cost DOUBLE PRECISION,
    batch_number TEXT,
    expiration_date TEXT,
    location TEXT,
    notes TEXT,
    reference_id TEXT,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_inventory_transactions_tenant ON inventory_transactions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_inventory_transactions_item ON inventory_transactions(item_id);
CREATE INDEX IF NOT EXISTS idx_inventory_transactions_type ON inventory_transactions(transaction_type);
CREATE INDEX IF NOT EXISTS idx_inventory_transactions_created_at ON inventory_transactions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_inventory_transactions_batch ON inventory_transactions(batch_number) WHERE batch_number IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_inventory_transactions_expiration ON inventory_transactions(expiration_date) WHERE expiration_date IS NOT NULL;
