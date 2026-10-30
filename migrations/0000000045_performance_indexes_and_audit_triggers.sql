-- Migration 0045: Performance indexes and remove audit triggers from high-frequency tables
-- I6: Remove audit triggers from high-frequency tables
-- I10: Add 10 missing indexes

-- ============================================================
-- I6: Remove audit triggers from high-frequency tables
-- These tables have high write throughput; audit is covered by work_logs anyway
-- ============================================================

DROP TRIGGER IF EXISTS audit_trigger_worker_locations ON worker_locations;
DROP TRIGGER IF EXISTS audit_trigger_clock_entries ON clock_entries;
DROP TRIGGER IF EXISTS audit_trigger_worker_task_statuses ON worker_task_statuses;
DROP TRIGGER IF EXISTS audit_trigger_task_data ON task_data;
DROP TRIGGER IF EXISTS audit_trigger_weather_data ON weather_data;
DROP TRIGGER IF EXISTS audit_trigger_soil_moisture_readings ON soil_moisture_readings;

-- ============================================================
-- I10: Add 10 missing indexes
-- ============================================================

-- 1. inventory_transactions: item_id + created_at for balance aggregation (I8)
CREATE INDEX IF NOT EXISTS idx_inv_txns_item_created
    ON inventory_transactions(item_id, created_at DESC);

-- 2. orders: tenant_id + created_at for listing orders by tenant
CREATE INDEX IF NOT EXISTS idx_orders_tenant_created
    ON orders(tenant_id, created_at DESC);

-- 3. weather_data: tenant_id + timestamp for tenant-scoped weather queries
CREATE INDEX IF NOT EXISTS idx_weather_data_tenant_timestamp
    ON weather_data(tenant_id, timestamp DESC);

-- 4. tasks: tenant_id + status for filtering tasks by status within tenant
CREATE INDEX IF NOT EXISTS idx_tasks_tenant_status
    ON tasks(tenant_id, status);

-- 5. tasks: status + scheduled_start for upcoming tasks queries
CREATE INDEX IF NOT EXISTS idx_tasks_status_scheduled
    ON tasks(status, scheduled_start);

-- 6. animals: tenant_id + created_at for listing animals by tenant
CREATE INDEX IF NOT EXISTS idx_animals_tenant_created
    ON animals(tenant_id, created_at DESC);

-- 7. harvest_lots: tenant_id + created_at for listing harvest lots by tenant
CREATE INDEX IF NOT EXISTS idx_harvest_lots_tenant_created
    ON harvest_lots(tenant_id, created_at DESC);

-- 8. frost_warnings: partial index on active warnings (tenant_id + created_at)
CREATE INDEX IF NOT EXISTS idx_frost_warnings_active
    ON frost_warnings(tenant_id, created_at DESC)
    WHERE is_active;

-- 9. tenants: is_active + id for active tenant lookups
CREATE INDEX IF NOT EXISTS idx_tenants_active_id
    ON tenants(is_active, id);

-- 10. customers: trigram indexes on company and customer_number for search
-- (trgm extension should already be enabled)
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX IF NOT EXISTS idx_customers_company_trgm
    ON customers USING gin (company gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_customers_number_trgm
    ON customers USING gin (customer_number gin_trgm_ops);

-- 11. equipment: trigram index on label and code for search
CREATE INDEX IF NOT EXISTS idx_equipment_label_trgm
    ON equipment USING gin (lower(label) gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_equipment_code_trgm
    ON equipment USING gin (lower(code) gin_trgm_ops);
