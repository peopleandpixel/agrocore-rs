-- 2026081801_updated_at_triggers.sql
-- Ensures all tables with an `updated_at` column automatically update to NOW()
-- on row update. Uses a single reusable trigger function applied per-table.

-- Reusable trigger function: sets updated_at = NOW() on UPDATE
CREATE OR REPLACE FUNCTION trigger_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
-- Per-table triggers for automatic updated_at

CREATE TRIGGER trigger_updated_at_tenants
    BEFORE UPDATE ON tenants
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_users
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_sites
    BEFORE UPDATE ON sites
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_equipment
    BEFORE UPDATE ON equipment
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_orders
    BEFORE UPDATE ON orders
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_order_sites
    BEFORE UPDATE ON order_sites
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_weather_stations
    BEFORE UPDATE ON weather_stations
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_weather_data
    BEFORE UPDATE ON weather_data
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_animals
    BEFORE UPDATE ON animals
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_task_data
    BEFORE UPDATE ON task_data
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_tasks
    BEFORE UPDATE ON tasks
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_phenology_records
    BEFORE UPDATE ON phenology_records
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_olive_groves
    BEFORE UPDATE ON olive_groves
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_olive_oil_records
    BEFORE UPDATE ON olive_oil_records
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_vineyards
    BEFORE UPDATE ON vineyards
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_kelter_deliveries
    BEFORE UPDATE ON kelter_deliveries
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_water_sources
    BEFORE UPDATE ON water_sources
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_water_usage
    BEFORE UPDATE ON water_usage
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_water_quotas
    BEFORE UPDATE ON water_quotas
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_fertilizer_records
    BEFORE UPDATE ON fertilizer_records
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_compliance_checklists
    BEFORE UPDATE ON compliance_checklists
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_compliance_items
    BEFORE UPDATE ON compliance_items
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_audit_logs
    BEFORE UPDATE ON audit_logs
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_cost_centers
    BEFORE UPDATE ON cost_centers
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_financial_records
    BEFORE UPDATE ON financial_records
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_pac_applications
    BEFORE UPDATE ON pac_applications
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_plant_protection_records
    BEFORE UPDATE ON plant_protection_records
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_worker_task_statuses
    BEFORE UPDATE ON worker_task_statuses
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_lpis_reference_parcels
    BEFORE UPDATE ON lpis_reference_parcels
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_iot_devices
    BEFORE UPDATE ON iot_devices
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_inventory_locations
    BEFORE UPDATE ON inventory_locations
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_inventory_items
    BEFORE UPDATE ON inventory_items
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_clock_entries
    BEFORE UPDATE ON clock_entries
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_frost_warnings
    BEFORE UPDATE ON frost_warnings
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_growing_degree_days
    BEFORE UPDATE ON growing_degree_days
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_pest_risks
    BEFORE UPDATE ON pest_risks
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_soil_moisture_configs
    BEFORE UPDATE ON soil_moisture_configs
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_soil_moisture_readings
    BEFORE UPDATE ON soil_moisture_readings
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_soil_moisture_alerts
    BEFORE UPDATE ON soil_moisture_alerts
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trigger_updated_at_customers
    BEFORE UPDATE ON customers
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();
