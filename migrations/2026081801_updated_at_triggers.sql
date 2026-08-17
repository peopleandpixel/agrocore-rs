-- 2026081801_updated_at_triggers.sql
-- Ensures all tables with an `updated_at` column automatically update to NOW()
-- on row update. Uses a single reusable trigger function applied per-table.
--
-- Note: The set_updated_at() function and some triggers already exist from
-- earlier migrations (20240101_init.sql etc.). This migration adds missing
-- triggers for tables that don't have them yet, using IF NOT EXISTS guards.

-- Reusable trigger function (CREATE OR REPLACE so it's idempotent)
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Per-table triggers for automatic updated_at
-- Using DO blocks with pg_trigger check to avoid duplicate trigger errors
DO $$
DECLARE
    tbl RECORD;
    trigger_name TEXT;
BEGIN
    FOR tbl IN
        SELECT tablename::TEXT AS name
        FROM pg_tables
        WHERE schemaname = 'public'
          AND tablename IN (
            'tenants', 'users', 'sites', 'equipment', 'orders', 'order_sites',
            'weather_stations', 'weather_data', 'animals', 'task_data', 'tasks',
            'phenology_records', 'olive_groves', 'olive_oil_records', 'vineyards',
            'kelter_deliveries', 'water_sources', 'water_usage', 'water_quotas',
            'fertilizer_records', 'compliance_checklists', 'compliance_items',
            'audit_logs', 'cost_centers', 'financial_records', 'pac_applications',
            'plant_protection_records', 'worker_task_statuses',
            'lpis_reference_parcels', 'iot_devices', 'inventory_locations',
            'inventory_items', 'clock_entries', 'frost_warnings',
            'growing_degree_days', 'pest_risks', 'soil_moisture_configs',
            'soil_moisture_readings', 'soil_moisture_alerts', 'customers'
          )
          AND EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_name = tbl.tablename
              AND column_name = 'updated_at'
          )
    LOOP
        trigger_name := 'set_updated_at_' || tbl.name;
        IF NOT EXISTS (
            SELECT 1 FROM pg_trigger WHERE tgname = trigger_name
        ) THEN
            EXECUTE format(
                'CREATE TRIGGER %I BEFORE UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION set_updated_at()',
                trigger_name, tbl.name
            );
        END IF;
    END LOOP;
END $$;
