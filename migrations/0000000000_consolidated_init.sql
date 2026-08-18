-- 0000000000_consolidated_init.sql
-- Single consolidated migration: ALL tables created directly with ALL columns.
-- No ALTER TABLE statements. Fresh schema each time (dev.sh destroys Docker).

-- Clean slate: drop all existing tables (for dev re-runs)
DROP TABLE IF EXISTS
    sites, user_sites, order_sites, users, tenants, equipment, orders, cost_centers,
    customers, weather_data, weather_stations, animals, grazing_records,
    treatment_records, harvest_seasons, harvest_lots, harvest_deliveries,
    cold_chain_logs, olive_groves, olive_oil_records, vineyards, kelter_deliveries,
    water_sources, water_usage, water_quotas, fertilizer_records, compliance_items,
    compliance_checklists, audit_logs, financial_records, pac_applications,
    plant_protection_records, applicant_licenses, worker_task_statuses,
    lpis_reference_parcels, sigpac_parcels, iot_devices, inventory_transactions,
    inventory_items, inventory_locations, clock_entries, workers,
    worker_locations, work_logs, frost_warnings, growing_degree_days, pest_risks,
    soil_moisture_configs, soil_moisture_readings, soil_moisture_alerts
CASCADE;

DROP TYPE IF EXISTS license_type CASCADE;

CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================
-- 1. Reusable Functions
-- ============================================================

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_current_tenant_id()
RETURNS UUID AS $$
BEGIN
    RETURN current_setting('app.current_tenant_id', true)::UUID;
EXCEPTION WHEN OTHERS THEN
    RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION is_superadmin()
RETURNS BOOLEAN AS $$
BEGIN
    RETURN current_setting('app.is_superadmin', true)::BOOLEAN;
EXCEPTION WHEN OTHERS THEN
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

GRANT EXECUTE ON FUNCTION get_current_tenant_id() TO PUBLIC;
GRANT EXECUTE ON FUNCTION is_superadmin() TO PUBLIC;

CREATE OR REPLACE FUNCTION has_column(table_name TEXT, column_name TEXT)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
        AND table_name = $1
        AND column_name = $2
    );
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION calculate_area_hectares(p_geometry GEOMETRY)
RETURNS NUMERIC(10,4) AS $$
BEGIN
    IF p_geometry IS NULL THEN
        RETURN NULL;
    END IF;
    RETURN ROUND(ST_Area(p_geometry::GEOGRAPHY) / 10000.0, 4);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_site_area_trigger()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.boundary IS NOT NULL AND
       (OLD.boundary IS NULL OR NOT ST_Equals(NEW.boundary, OLD.boundary)) THEN
        NEW.area := calculate_area_hectares(NEW.boundary);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION validate_parcel_against_lpis(
    p_province SMALLINT,
    p_municipality SMALLINT,
    p_aggregate SMALLINT,
    p_zone SMALLINT,
    p_polygon SMALLINT,
    p_parcel SMALLINT,
    p_enclosure SMALLINT,
    p_boundary GEOMETRY(POLYGON, 4326),
    p_area_ha NUMERIC(10,4)
) RETURNS JSONB AS $$
DECLARE
    v_ref sigpac_parcels%ROWTYPE;
    v_area_diff NUMERIC(10,4);
    v_boundary_diff NUMERIC(10,4);
    v_warnings TEXT[];
    v_is_valid BOOLEAN := FALSE;
BEGIN
    SELECT * INTO v_ref
    FROM sigpac_parcels
    WHERE province = p_province
      AND municipality = p_municipality
      AND aggregate = p_aggregate
      AND zone = p_zone
      AND polygon = p_polygon
      AND parcel = p_parcel
      AND enclosure = p_enclosure
    LIMIT 1;

    IF NOT FOUND THEN
        v_warnings := array_append(v_warnings, 'SIGPAC reference not found in database');
        RETURN jsonb_build_object(
            'is_valid', false,
            'sigpac_reference', LPAD(p_province::TEXT, 2, '0') || LPAD(p_municipality::TEXT, 3, '0') ||
                                LPAD(p_aggregate::TEXT, 3, '0') || LPAD(p_zone::TEXT, 3, '0') ||
                                LPAD(p_polygon::TEXT, 3, '0') || LPAD(p_parcel::TEXT, 3, '0') ||
                                LPAD(p_enclosure::TEXT, 3, '0'),
            'area_difference', NULL,
            'boundary_difference', NULL,
            'warnings', v_warnings
        );
    END IF;

    IF p_area_ha IS NOT NULL AND v_ref.official_area_ha IS NOT NULL THEN
        v_area_diff := ABS(p_area_ha - v_ref.official_area_ha);
        IF v_area_diff > 0.5 THEN
            v_warnings := array_append(v_warnings,
                format('Area difference: %.2f ha (declared: %.2f, official: %.2f)',
                       v_area_diff, p_area_ha, v_ref.official_area_ha));
        END IF;
    END IF;

    IF p_boundary IS NOT NULL THEN
        v_boundary_diff := ST_HausdorffDistance(p_boundary, v_ref.geometry);
        IF v_boundary_diff > 10 THEN
            v_warnings := array_append(v_warnings,
                format('Boundary difference: %.2f meters', v_boundary_diff));
        END IF;
        IF ST_Area(ST_Intersection(p_boundary, v_ref.geometry)) /
           ST_Area(ST_Union(p_boundary, v_ref.geometry)) < 0.9 THEN
            v_warnings := array_append(v_warnings, 'Low geometry overlap with reference parcel');
        END IF;
    END IF;

    v_is_valid := array_length(v_warnings, 1) IS NULL OR
                  (SELECT COUNT(*) FROM unnest(v_warnings) w WHERE w LIKE '%difference: %') = 0;

    RETURN jsonb_build_object(
        'is_valid', v_is_valid,
        'sigpac_reference', v_ref.sigpac_reference,
        'area_difference', v_area_diff,
        'boundary_difference', v_boundary_diff,
        'warnings', COALESCE(v_warnings, '{}'::TEXT[])
    );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION find_duplicate_parcel(
    p_tenant_id UUID,
    p_sigpac_province SMALLINT DEFAULT NULL,
    p_sigpac_municipality SMALLINT DEFAULT NULL,
    p_sigpac_aggregate SMALLINT DEFAULT NULL,
    p_sigpac_zone SMALLINT DEFAULT NULL,
    p_sigpac_polygon SMALLINT DEFAULT NULL,
    p_sigpac_parcel SMALLINT DEFAULT NULL,
    p_sigpac_enclosure SMALLINT DEFAULT NULL,
    p_regepac_id TEXT DEFAULT NULL,
    p_boundary GEOMETRY(POLYGON, 4326) DEFAULT NULL
) RETURNS TABLE (
    existing_id UUID,
    match_type TEXT,
    similarity_score NUMERIC(5,4)
) AS $$
BEGIN
    IF p_sigpac_province IS NOT NULL THEN
        RETURN QUERY
        SELECT s.id, 'SigpacMatch'::TEXT, 1.0::NUMERIC(5,4)
        FROM sites s
        WHERE s.tenant_id = p_tenant_id
          AND s.sigpac_data->>'province' = p_sigpac_province::TEXT
          AND s.sigpac_data->>'municipality' = p_sigpac_municipality::TEXT
          AND s.sigpac_data->>'aggregate' = p_sigpac_aggregate::TEXT
          AND s.sigpac_data->>'zone' = p_sigpac_zone::TEXT
          AND s.sigpac_data->>'polygon' = p_sigpac_polygon::TEXT
          AND s.sigpac_data->>'parcel' = p_sigpac_parcel::TEXT
          AND s.sigpac_data->>'enclosure' = p_sigpac_enclosure::TEXT
          AND s.is_active = TRUE
        LIMIT 1;
        IF FOUND THEN RETURN; END IF;
    END IF;

    IF p_regepac_id IS NOT NULL THEN
        RETURN QUERY
        SELECT s.id, 'RegepacMatch'::TEXT, 0.95::NUMERIC(5,4)
        FROM sites s
        WHERE s.tenant_id = p_tenant_id
          AND s.regepac_id = p_regepac_id
          AND s.is_active = TRUE
        LIMIT 1;
        IF FOUND THEN RETURN; END IF;
    END IF;

    IF p_boundary IS NOT NULL THEN
        RETURN QUERY
        SELECT s.id, 'ExactBoundary'::TEXT, 1.0::NUMERIC(5,4)
        FROM sites s
        WHERE s.tenant_id = p_tenant_id
          AND ST_Equals(s.boundary, p_boundary)
          AND s.is_active = TRUE
        LIMIT 1;
        IF FOUND THEN RETURN; END IF;

        RETURN QUERY
        SELECT s.id, 'HighSimilarity'::TEXT,
               (ST_Area(ST_Intersection(s.boundary, p_boundary)) /
                ST_Area(ST_Union(s.boundary, p_boundary)))::NUMERIC(5,4)
        FROM sites s
        WHERE s.tenant_id = p_tenant_id
          AND ST_Intersects(s.boundary, p_boundary)
          AND s.is_active = TRUE
          AND (ST_Area(ST_Intersection(s.boundary, p_boundary)) /
               ST_Area(ST_Union(s.boundary, p_boundary))) > 0.95
        ORDER BY 3 DESC
        LIMIT 1;
    END IF;

    RETURN;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION audit_trigger_function()
RETURNS TRIGGER AS $$
DECLARE
    v_old_value JSONB;
    v_new_value JSONB;
    v_changed_fields TEXT[];
    v_user_id UUID;
    v_tenant_id UUID;
    v_action TEXT;
    v_ip INET;
    v_user_agent TEXT;
    v_request_id UUID;
    v_session_id UUID;
BEGIN
    IF TG_TABLE_NAME = 'audit_logs' THEN
        RETURN NEW;
    END IF;

    v_user_id := COALESCE(current_setting('app.current_user_id', true)::UUID, NULL);
    v_tenant_id := COALESCE(current_setting('app.current_tenant_id', true)::UUID, NULL);
    v_ip := COALESCE(current_setting('app.current_ip', true)::INET, NULL);
    v_user_agent := current_setting('app.current_user_agent', true);
    v_request_id := COALESCE(current_setting('app.current_request_id', true)::UUID, gen_random_uuid());
    v_session_id := COALESCE(current_setting('app.current_session_id', true)::UUID, NULL);

    v_action := CASE TG_OP
        WHEN 'INSERT' THEN 'Created'
        WHEN 'UPDATE' THEN 'Updated'
        WHEN 'DELETE' THEN 'Deleted'
        ELSE TG_OP
    END;

    IF TG_OP = 'UPDATE' THEN
        v_old_value := to_jsonb(OLD) - 'created_at' - 'updated_at';
        v_new_value := to_jsonb(NEW) - 'created_at' - 'updated_at';
        SELECT ARRAY(
            SELECT key FROM jsonb_each(v_new_value)
            WHERE value IS DISTINCT FROM (v_old_value -> key)
        ) INTO v_changed_fields;
    ELSIF TG_OP = 'INSERT' THEN
        v_new_value := to_jsonb(NEW) - 'created_at' - 'updated_at';
        v_old_value := NULL;
        v_changed_fields := ARRAY[]::TEXT[];
    ELSIF TG_OP = 'DELETE' THEN
        v_old_value := to_jsonb(OLD) - 'created_at' - 'updated_at';
        v_new_value := NULL;
        v_changed_fields := ARRAY[]::TEXT[];
    END IF;

    INSERT INTO audit_logs (
        tenant_id, user_id, action, entity_type, entity_id,
        old_value, new_value, changed_fields,
        ip_address, user_agent, request_id, session_id
    ) VALUES (
        COALESCE(v_tenant_id,
            CASE
                WHEN TG_TABLE_NAME = 'tenants' THEN NEW.id
                WHEN has_column(TG_TABLE_NAME, 'tenant_id') THEN
                    COALESCE(NEW.tenant_id, OLD.tenant_id)
                ELSE NULL
            END
        ),
        v_user_id,
        v_action,
        TG_TABLE_NAME,
        COALESCE(NEW.id, OLD.id),
        v_old_value,
        v_new_value,
        v_changed_fields,
        v_ip,
        v_user_agent,
        v_request_id,
        v_session_id
    );

    RETURN CASE TG_OP
        WHEN 'DELETE' THEN OLD
        ELSE NEW
    END;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- ============================================================
-- 2. Core Tables
-- ============================================================

CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    slug VARCHAR(50) UNIQUE,
    config JSONB,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    address TEXT,
    company VARCHAR(200),
    customer_number VARCHAR(50) NOT NULL,
    vat_rate NUMERIC(5,2) NOT NULL DEFAULT 19.0,
    payment_terms VARCHAR(100),
    preferred_delivery_location TEXT,
    preferences JSONB DEFAULT '{}'::JSONB,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    firstname VARCHAR(100) NOT NULL,
    lastname VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    roles JSONB NOT NULL DEFAULT '[]'::JSONB,
    is_active BOOLEAN NOT NULL DEFAULT true,
    internal_cost_per_hour NUMERIC(10,2),
    external_cost_per_hour NUMERIC(10,2),
    color VARCHAR(7),
    language VARCHAR(10),
    last_login TIMESTAMPTZ,
    refresh_token TEXT,
    refresh_token_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS equipment (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    code TEXT,
    equipment_type JSONB,
    in_usage BOOLEAN NOT NULL DEFAULT false,
    maintenance_intervals JSONB,
    next_maintenance_date DATE,
    last_maintenance_hours INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    business_id UUID,
    label TEXT NOT NULL,
    code TEXT,
    description TEXT,
    site_type JSONB,
    crop_type JSONB,
    variety VARCHAR(100),
    area NUMERIC(15,2),
    gross_area NUMERIC(15,2),
    center_lng NUMERIC(10,8),
    center_lat NUMERIC(10,8),
    boundary GEOMETRY(POLYGON, 4326),
    center GEOMETRY(POINT, 4326),
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_temporary BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_by UUID,
    plots JSONB NOT NULL DEFAULT '[]'::JSONB,
    row_config JSONB,
    bbch_stage JSONB,
    planted_date TIMESTAMPTZ,
    cleared_date TIMESTAMPTZ,
    soil_type TEXT,
    slope DOUBLE PRECISION,
    slope_facing TEXT,
    altitude DOUBLE PRECISION,
    organic BOOLEAN,
    organic_eligible BOOLEAN,
    sigpac_data JSONB,
    regepac_id TEXT,
    properties JSONB DEFAULT '[]'::JSONB,
    custom_fields JSONB DEFAULT '{}'::JSONB,
    note1 TEXT,
    note2 TEXT,
    lpis_country VARCHAR(2),
    lpis_data JSONB
);

CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    title TEXT,
    description TEXT,
    priority INTEGER DEFAULT 0,
    status JSONB,
    order_type VARCHAR(50),
    assigned_to UUID REFERENCES users(id),
    assigned_worker_ids JSONB NOT NULL DEFAULT '[]'::JSONB,
    site_ids JSONB NOT NULL DEFAULT '[]'::JSONB,
    scheduled_start TIMESTAMPTZ,
    scheduled_end TIMESTAMPTZ,
    actual_start TIMESTAMPTZ,
    actual_end TIMESTAMPTZ,
    deadline_date DATE,
    planned_date DATE,
    recurrence JSONB,
    last_completed_at TIMESTAMPTZ,
    execution_policy JSONB,
    automation_state JSONB,
    articles JSONB,
    quantities JSONB,
    results TEXT,
    weather JSONB,
    custom_fields JSONB DEFAULT '{}'::JSONB,
    parent_order_id UUID,
    workflow_config JSONB,
    cost_center_id UUID,
    customer_id UUID REFERENCES customers(id) ON DELETE SET NULL,
    created_by UUID,
    updated_by UUID,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS order_sites (
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    PRIMARY KEY (order_id, site_id)
);

CREATE TABLE IF NOT EXISTS user_sites (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, site_id)
);

CREATE TABLE IF NOT EXISTS cost_centers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES cost_centers(id) ON DELETE SET NULL,
    code TEXT NOT NULL,
    label TEXT NOT NULL,
    description TEXT,
    budget_eur DOUBLE PRECISION,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, code)
);

CREATE TABLE IF NOT EXISTS weather_stations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    label VARCHAR(100) NOT NULL,
    station_type JSONB NOT NULL,
    location GEOMETRY(POINT, 4326),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS weather_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    station_id UUID REFERENCES weather_stations(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL,
    temperature_c DOUBLE PRECISION,
    humidity_percent DOUBLE PRECISION,
    precipitation_mm DOUBLE PRECISION,
    wind_speed_kmh DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS animals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    tag_number TEXT,
    species TEXT,
    breed TEXT,
    birth_date DATE,
    gender TEXT,
    current_site_id UUID REFERENCES sites(id) ON DELETE SET NULL,
    mother_id UUID REFERENCES animals(id) ON DELETE SET NULL,
    father_id UUID REFERENCES animals(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_animal_tag_per_tenant UNIQUE (tenant_id, tag_number)
);

CREATE TABLE IF NOT EXISTS grazing_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS treatment_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
    date TIMESTAMPTZ NOT NULL,
    treatment_type TEXT NOT NULL,
    medication TEXT,
    dosage TEXT,
    veterinarian TEXT,
    withdrawal_days INTEGER,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    employee_id TEXT NOT NULL,
    firstname TEXT NOT NULL,
    lastname TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    role_in_company TEXT,
    hourly_rate NUMERIC(10,2),
    social_security_number TEXT,
    bank_account TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    worker_id UUID REFERENCES users(id) ON DELETE SET NULL,
    site_id UUID REFERENCES sites(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    description TEXT,
    priority INTEGER DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    scheduled_start TIMESTAMPTZ,
    scheduled_end TIMESTAMPTZ,
    actual_start TIMESTAMPTZ,
    actual_end TIMESTAMPTZ,
    estimated_duration_hours DOUBLE PRECISION,
    actual_duration_hours DOUBLE PRECISION,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS task_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    task_id UUID NOT NULL,
    worker_id UUID NOT NULL,
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    site_id UUID REFERENCES sites(id) ON DELETE SET NULL,
    description TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    status TEXT,
    notes TEXT,
    paused_at TIMESTAMPTZ,
    resume_at TIMESTAMPTZ,
    duration_minutes INTEGER,
    machine_id UUID,
    machine_hours DOUBLE PRECISION,
    cost_center_id UUID,
    area_covered DOUBLE PRECISION,
    materials_used JSONB,
    observations TEXT,
    gps_track JSONB,
    photo_urls JSONB,
    pause_resume_cycles JSONB NOT NULL DEFAULT '[]'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS harvest_seasons (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    year INTEGER NOT NULL,
    label TEXT NOT NULL,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS harvest_lots (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    season_id UUID NOT NULL REFERENCES harvest_seasons(id) ON DELETE CASCADE,
    lot_number TEXT NOT NULL,
    crop_type TEXT NOT NULL,
    variety TEXT,
    quality_target TEXT,
    total_weight_kg DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS harvest_deliveries (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    lot_id UUID NOT NULL REFERENCES harvest_lots(id) ON DELETE CASCADE,
    delivery_date TIMESTAMPTZ NOT NULL,
    gross_weight_kg DOUBLE PRECISION NOT NULL,
    tare_weight_kg DOUBLE PRECISION NOT NULL,
    net_weight_kg DOUBLE PRECISION NOT NULL,
    carrier_name TEXT,
    vehicle_id TEXT,
    quality_notes TEXT,
    temperature_at_delivery DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cold_chain_logs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    lot_id UUID NOT NULL REFERENCES harvest_lots(id) ON DELETE CASCADE,
    sensor_id TEXT NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL,
    temperature_c DOUBLE PRECISION NOT NULL,
    humidity_pct DOUBLE PRECISION,
    location TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS olive_groves (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id),
    label TEXT NOT NULL,
    variety TEXT NOT NULL,
    tree_count INTEGER,
    planting_year INTEGER,
    area_ha NUMERIC(10,2),
    spacing_m NUMERIC(10,2),
    irrigation_type TEXT,
    is_organic BOOLEAN DEFAULT false,
    certification_body TEXT,
    certification_number TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS olive_oil_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    olive_grove_id UUID NOT NULL REFERENCES olive_groves(id) ON DELETE CASCADE,
    harvest_year INTEGER NOT NULL,
    harvest_date TIMESTAMPTZ NOT NULL,
    quantity_kg DOUBLE PRECISION,
    oil_yield_kg DOUBLE PRECISION,
    oil_yield_pct DOUBLE PRECISION,
    oil_grade TEXT,
    acidity_pct DOUBLE PRECISION,
    peroxide_value DOUBLE PRECISION,
    k232 DOUBLE PRECISION,
    k270 DOUBLE PRECISION,
    sensory_score DOUBLE PRECISION,
    liters_produced DOUBLE PRECISION,
    mill_name TEXT,
    lot_number TEXT,
    quality_grade TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS vineyards (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id),
    doc_area TEXT,
    vintage INTEGER,
    grape_variety TEXT,
    brix_at_harvest DOUBLE PRECISION,
    ph_at_harvest DOUBLE PRECISION,
    acidity DOUBLE PRECISION,
    yield_tons DOUBLE PRECISION,
    quality_grade TEXT,
    slope_percent DOUBLE PRECISION,
    altitude_m DOUBLE PRECISION,
    is_organic BOOLEAN DEFAULT false,
    certification_body TEXT,
    certification_number TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS kelter_deliveries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    vineyard_id UUID NOT NULL REFERENCES vineyards(id) ON DELETE CASCADE,
    delivery_date TIMESTAMPTZ NOT NULL,
    gross_weight_kg DOUBLE PRECISION NOT NULL,
    net_weight_kg DOUBLE PRECISION NOT NULL,
    lot_number TEXT NOT NULL,
    kelter_name TEXT NOT NULL,
    transport_company TEXT,
    temperature_c DOUBLE PRECISION,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS water_sources (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id),
    source_type TEXT NOT NULL,
    name TEXT NOT NULL,
    capacity_m3 DOUBLE PRECISION,
    current_level_m3 DOUBLE PRECISION,
    license_number TEXT,
    license_expiry TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS water_usage (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id),
    source_id UUID NOT NULL REFERENCES water_sources(id) ON DELETE CASCADE,
    usage_date TIMESTAMPTZ NOT NULL,
    volume_m3 DOUBLE PRECISION NOT NULL,
    irrigation_method TEXT NOT NULL,
    efficiency_pct DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS water_quotas (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    source_id UUID NOT NULL REFERENCES water_sources(id),
    site_id UUID NOT NULL REFERENCES sites(id),
    year INTEGER NOT NULL,
    allocated_m3 DOUBLE PRECISION NOT NULL,
    used_m3 DOUBLE PRECISION NOT NULL DEFAULT 0,
    remaining_m3 DOUBLE PRECISION NOT NULL DEFAULT 0,
    comunidad_id UUID REFERENCES water_sources(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, source_id, site_id, year)
);

CREATE TABLE IF NOT EXISTS fertilizer_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id),
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    product_name TEXT NOT NULL,
    nutrient_n DOUBLE PRECISION NOT NULL,
    nutrient_p DOUBLE PRECISION NOT NULL,
    nutrient_k DOUBLE PRECISION NOT NULL,
    quantity_kg DOUBLE PRECISION NOT NULL,
    area_ha DOUBLE PRECISION NOT NULL,
    application_date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_checklists (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id),
    checklist_type TEXT NOT NULL,
    status TEXT NOT NULL,
    label TEXT,
    description TEXT,
    due_date TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    checklist_id UUID NOT NULL REFERENCES compliance_checklists(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id),
    label TEXT NOT NULL,
    status TEXT NOT NULL,
    evidence_url TEXT,
    completed_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID REFERENCES users(id),
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    old_value JSONB,
    new_value JSONB,
    changed_fields TEXT[],
    ip_address INET,
    user_agent TEXT,
    request_id UUID,
    session_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS financial_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    cost_center_id UUID REFERENCES cost_centers(id),
    site_id UUID REFERENCES sites(id),
    record_type TEXT NOT NULL,
    amount_eur DOUBLE PRECISION NOT NULL,
    currency TEXT DEFAULT 'EUR',
    date TIMESTAMPTZ NOT NULL,
    description TEXT,
    invoice_ref TEXT,
    external_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS pac_applications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    application_number TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    total_area_ha DOUBLE PRECISION,
    declared_area_ha DOUBLE PRECISION,
    subsidy_amount_eur DOUBLE PRECISION,
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, year)
);

CREATE TABLE IF NOT EXISTS plant_protection_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id),
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    applicator_id UUID REFERENCES users(id),
    product_name TEXT NOT NULL,
    active_substance TEXT NOT NULL,
    concentration DOUBLE PRECISION,
    dosage_per_ha DOUBLE PRECISION NOT NULL,
    area_ha DOUBLE PRECISION NOT NULL,
    application_date TIMESTAMPTZ NOT NULL,
    pre_harvest_interval_days INTEGER,
    re_entry_interval_days INTEGER,
    weather_conditions JSONB,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TYPE license_type AS ENUM ('basic', 'advanced', 'professional', 'custom');

CREATE TABLE IF NOT EXISTS applicator_licenses (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    license_type license_type NOT NULL,
    license_number VARCHAR(255) NOT NULL,
    issued_by VARCHAR(255) NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL,
    valid_until TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS worker_task_statuses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    worker_id UUID NOT NULL REFERENCES users(id),
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS lpis_reference_parcels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    province SMALLINT NOT NULL,
    municipality SMALLINT NOT NULL,
    aggregate SMALLINT NOT NULL,
    zone SMALLINT NOT NULL,
    polygon SMALLINT NOT NULL,
    parcel SMALLINT NOT NULL,
    enclosure SMALLINT NOT NULL,
    sigpac_reference VARCHAR(20) GENERATED ALWAYS AS (
        LPAD(province::text, 2, '0') ||
        LPAD(municipality::text, 3, '0') ||
        LPAD(aggregate::text, 3, '0') ||
        LPAD(zone::text, 3, '0') ||
        LPAD(polygon::text, 3, '0') ||
        LPAD(parcel::text, 3, '0') ||
        LPAD(enclosure::text, 3, '0')
    ) STORED,
    usage_code VARCHAR(10),
    crop_group VARCHAR(50),
    geometry GEOMETRY(POLYGON, 4326) NOT NULL,
    area_hectares DOUBLE PRECISION GENERATED ALWAYS AS (
        ST_Area(geometry::geography) / 10000.0
    ) STORED,
    source_year SMALLINT,
    source_dataset VARCHAR(100),
    last_verified TIMESTAMPTZ,
    is_current BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sigpac_parcels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL,
    province SMALLINT NOT NULL,
    municipality SMALLINT NOT NULL,
    aggregate SMALLINT NOT NULL,
    zone SMALLINT NOT NULL,
    polygon SMALLINT NOT NULL,
    parcel SMALLINT NOT NULL,
    enclosure SMALLINT NOT NULL,
    sigpac_reference VARCHAR(20) GENERATED ALWAYS AS (
        LPAD(province::TEXT, 2, '0') ||
        LPAD(municipality::TEXT, 3, '0') ||
        LPAD(aggregate::TEXT, 3, '0') ||
        LPAD(zone::TEXT, 3, '0') ||
        LPAD(polygon::TEXT, 3, '0') ||
        LPAD(parcel::TEXT, 3, '0') ||
        LPAD(enclosure::TEXT, 3, '0')
    ) STORED,
    usage_code VARCHAR(10),
    usage_description TEXT,
    official_area_ha NUMERIC(10,4),
    geometry GEOMETRY(POLYGON, 4326) NOT NULL,
    source_year SMALLINT,
    last_updated TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS iot_devices (
    device_id TEXT PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id),
    device JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

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

CREATE TABLE IF NOT EXISTS inventory_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    category JSONB NOT NULL,
    name TEXT NOT NULL,
    sku TEXT,
    description TEXT,
    unit JSONB NOT NULL,
    minimum_stock DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    inventory_method JSONB NOT NULL DEFAULT '"FIFO"'::JSONB,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

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
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS clock_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    worker_id UUID NOT NULL REFERENCES workers(id) ON DELETE CASCADE,
    entry_type TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    lat DOUBLE PRECISION,
    lng DOUBLE PRECISION,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS worker_locations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    worker_id UUID NOT NULL REFERENCES workers(id) ON DELETE CASCADE,
    location GEOMETRY(POINT, 4326),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    accuracy_meters DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS work_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    task_id UUID REFERENCES tasks(id),
    worker_id UUID NOT NULL REFERENCES workers(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_minutes INTEGER,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS frost_warnings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    station_id UUID NOT NULL,
    threshold_temp_c NUMERIC(5,2) NOT NULL DEFAULT -2.0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    notify_email BOOLEAN NOT NULL DEFAULT true,
    notify_sms BOOLEAN NOT NULL DEFAULT false,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS growing_degree_days (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id),
    station_id UUID,
    date DATE NOT NULL,
    min_temp_c NUMERIC(5,2),
    max_temp_c NUMERIC(5,2),
    avg_temp_c NUMERIC(5,2),
    gdd_base_10 DOUBLE PRECISION,
    gdd_base_5 DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS pest_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id),
    assessment_date TIMESTAMPTZ NOT NULL,
    risk_level TEXT NOT NULL,
    affected_area_ha NUMERIC(10,2),
    pest_type TEXT NOT NULL,
    recommended_action TEXT NOT NULL,
    model_version VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS soil_moisture_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    station_id UUID NOT NULL,
    site_id UUID,
    moisture_threshold_percent NUMERIC(5,2) NOT NULL DEFAULT 30.0,
    min_interval_minutes INTEGER NOT NULL DEFAULT 120,
    irrigation_duration_minutes INTEGER NOT NULL DEFAULT 15,
    notify_email BOOLEAN NOT NULL DEFAULT true,
    notify_sms BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS soil_moisture_readings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    station_id UUID NOT NULL,
    site_id UUID,
    device_id VARCHAR(100),
    ts TIMESTAMP NOT NULL DEFAULT NOW(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    moisture_percent NUMERIC(5,2) NOT NULL,
    soil_temperature_c NUMERIC(5,2),
    soil_depth_cm NUMERIC(4,1),
    battery_level NUMERIC(5,2),
    signal_strength INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS soil_moisture_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    config_id UUID NOT NULL REFERENCES soil_moisture_configs(id),
    alert_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    moisture_percent NUMERIC(5,2) NOT NULL,
    threshold_percent NUMERIC(5,2) NOT NULL,
    action_taken TEXT,
    is_resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- 3. Indexes
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_refresh_token ON users(refresh_token) WHERE refresh_token IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_users_is_active_tenant ON users(is_active, tenant_id);

CREATE INDEX IF NOT EXISTS idx_sites_tenant_id ON sites(tenant_id);
CREATE INDEX IF NOT EXISTS idx_sites_boundary ON sites USING GIST(boundary);
CREATE INDEX IF NOT EXISTS idx_sites_is_active_tenant ON sites(is_active, tenant_id);
CREATE INDEX IF NOT EXISTS idx_sites_regepac_id ON sites(regepac_id);
CREATE INDEX IF NOT EXISTS idx_sites_lpis_country ON sites(lpis_country);
CREATE INDEX IF NOT EXISTS idx_sites_lpis_data ON sites USING GIN(lpis_data);

CREATE INDEX IF NOT EXISTS idx_equipment_tenant_id ON equipment(tenant_id);
CREATE INDEX IF NOT EXISTS idx_orders_tenant_id ON orders(tenant_id);
CREATE INDEX IF NOT EXISTS idx_orders_assigned_to ON orders(assigned_to);
CREATE INDEX IF NOT EXISTS idx_orders_customer ON orders(customer_id);
CREATE INDEX IF NOT EXISTS idx_orders_type_customer ON orders(order_type, customer_id) WHERE order_type = 'SalesOrder';
CREATE INDEX IF NOT EXISTS idx_orders_parent ON orders(parent_order_id);
CREATE INDEX IF NOT EXISTS idx_task_data_tenant ON task_data(tenant_id);
CREATE INDEX IF NOT EXISTS idx_task_data_task_worker ON task_data(task_id, worker_id);
CREATE INDEX IF NOT EXISTS idx_task_data_order ON task_data(order_id);
CREATE INDEX IF NOT EXISTS idx_task_data_site ON task_data(site_id);
CREATE INDEX IF NOT EXISTS idx_tasks_tenant ON tasks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tasks_order ON tasks(order_id);
CREATE INDEX IF NOT EXISTS idx_tasks_worker ON tasks(worker_id);
CREATE INDEX IF NOT EXISTS idx_tasks_site ON tasks(site_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

CREATE INDEX IF NOT EXISTS idx_weather_data_station_time ON weather_data(station_id, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_harvest_seasons_tenant ON harvest_seasons(tenant_id);
CREATE INDEX IF NOT EXISTS idx_harvest_lots_season ON harvest_lots(season_id);
CREATE INDEX IF NOT EXISTS idx_harvest_lots_tenant ON harvest_lots(tenant_id);
CREATE INDEX IF NOT EXISTS idx_harvest_deliveries_lot ON harvest_deliveries(lot_id);

CREATE INDEX IF NOT EXISTS idx_olive_groves_tenant ON olive_groves(tenant_id);
CREATE INDEX IF NOT EXISTS idx_olive_groves_site ON olive_groves(site_id);
CREATE INDEX IF NOT EXISTS idx_olive_oil_records_tenant ON olive_oil_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_olive_oil_records_grove ON olive_oil_records(olive_grove_id);
CREATE INDEX IF NOT EXISTS idx_olive_oil_records_year ON olive_oil_records(harvest_year);
CREATE INDEX IF NOT EXISTS idx_vineyards_tenant ON vineyards(tenant_id);
CREATE INDEX IF NOT EXISTS idx_vineyards_site ON vineyards(site_id);
CREATE INDEX IF NOT EXISTS idx_vineyards_active ON vineyards(is_active);
CREATE INDEX IF NOT EXISTS idx_kelter_deliveries_vineyard ON kelter_deliveries(vineyard_id);
CREATE INDEX IF NOT EXISTS idx_kelter_deliveries_date ON kelter_deliveries(delivery_date);

CREATE INDEX IF NOT EXISTS idx_water_sources_tenant ON water_sources(tenant_id);
CREATE INDEX IF NOT EXISTS idx_water_sources_site ON water_sources(site_id);
CREATE INDEX IF NOT EXISTS idx_water_usage_tenant ON water_usage(tenant_id);
CREATE INDEX IF NOT EXISTS idx_water_usage_source ON water_usage(source_id);
CREATE INDEX IF NOT EXISTS idx_water_usage_site ON water_usage(site_id);
CREATE INDEX IF NOT EXISTS idx_water_usage_date ON water_usage(usage_date);
CREATE INDEX IF NOT EXISTS idx_water_quotas_tenant ON water_quotas(tenant_id);
CREATE INDEX IF NOT EXISTS idx_water_quotas_year ON water_quotas(year);

CREATE INDEX IF NOT EXISTS idx_fertilizer_tenant ON fertilizer_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_fertilizer_site ON fertilizer_records(site_id);
CREATE INDEX IF NOT EXISTS idx_fertilizer_date ON fertilizer_records(application_date);
CREATE INDEX IF NOT EXISTS idx_fertilizer_order ON fertilizer_records(order_id);
CREATE INDEX IF NOT EXISTS idx_compliance_checklists_tenant ON compliance_checklists(tenant_id);
CREATE INDEX IF NOT EXISTS idx_compliance_checklists_type ON compliance_checklists(checklist_type);
CREATE INDEX IF NOT EXISTS idx_compliance_checklists_status ON compliance_checklists(status);
CREATE INDEX IF NOT EXISTS idx_compliance_items_checklist ON compliance_items(checklist_id);
CREATE INDEX IF NOT EXISTS idx_compliance_items_tenant ON compliance_items(tenant_id);

CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant_id ON audit_logs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_request ON audit_logs(request_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_session ON audit_logs(session_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_financial_tenant ON financial_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_financial_cost_center ON financial_records(cost_center_id);
CREATE INDEX IF NOT EXISTS idx_financial_site ON financial_records(site_id);
CREATE INDEX IF NOT EXISTS idx_financial_date ON financial_records(date);
CREATE INDEX IF NOT EXISTS idx_pac_applications_tenant ON pac_applications(tenant_id);
CREATE INDEX IF NOT EXISTS idx_pac_applications_year ON pac_applications(year);

CREATE INDEX IF NOT EXISTS idx_plant_protection_tenant ON plant_protection_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_plant_protection_site ON plant_protection_records(site_id);
CREATE INDEX IF NOT EXISTS idx_plant_protection_order ON plant_protection_records(order_id);
CREATE INDEX IF NOT EXISTS idx_applicator_licenses_tenant ON applicator_licenses(tenant_id);
CREATE INDEX IF NOT EXISTS idx_applicator_licenses_user ON applicator_licenses(user_id);
CREATE INDEX IF NOT EXISTS idx_applicator_licenses_active ON applicator_licenses(is_active);
CREATE INDEX IF NOT EXISTS idx_applicator_licenses_valid_until ON applicator_licenses(valid_until);
CREATE INDEX IF NOT EXISTS idx_worker_task_statuses_tenant ON worker_task_statuses(tenant_id);
CREATE INDEX IF NOT EXISTS idx_worker_task_statuses_task ON worker_task_statuses(task_id);
CREATE INDEX IF NOT EXISTS idx_worker_task_statuses_worker ON worker_task_statuses(worker_id);

CREATE INDEX IF NOT EXISTS idx_lpis_tenant_id ON lpis_reference_parcels(tenant_id);
CREATE INDEX IF NOT EXISTS idx_lpis_sigpac_ref ON lpis_reference_parcels(sigpac_reference);
CREATE INDEX IF NOT EXISTS idx_lpis_geometry ON lpis_reference_parcels USING GIST(geometry);
CREATE INDEX IF NOT EXISTS idx_lpis_usage_code ON lpis_reference_parcels(usage_code);
CREATE INDEX IF NOT EXISTS idx_lpis_is_current ON lpis_reference_parcels(is_current);
CREATE UNIQUE INDEX IF NOT EXISTS uq_lpis_parcel ON lpis_reference_parcels(tenant_id, province, municipality, aggregate, zone, polygon, parcel, enclosure);

CREATE INDEX IF NOT EXISTS idx_sigpac_parcels_reference ON sigpac_parcels(sigpac_reference);
CREATE INDEX IF NOT EXISTS idx_sigpac_parcels_geometry ON sigpac_parcels USING GIST(geometry);
CREATE INDEX IF NOT EXISTS idx_sigpac_parcels_tenant ON sigpac_parcels(tenant_id);
CREATE INDEX IF NOT EXISTS idx_sigpac_parcels_components ON sigpac_parcels(province, municipality, aggregate, zone, polygon, parcel, enclosure);
CREATE UNIQUE INDEX IF NOT EXISTS uq_sigpac_parcels_reference ON sigpac_parcels(sigpac_reference);

CREATE INDEX IF NOT EXISTS idx_iot_devices_tenant ON iot_devices(tenant_id);
CREATE INDEX IF NOT EXISTS idx_iot_devices_site ON iot_devices(site_id);
CREATE INDEX IF NOT EXISTS idx_inventory_locations_tenant ON inventory_locations(tenant_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_inventory_locations_code ON inventory_locations(tenant_id, code) WHERE code IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_inventory_items_tenant ON inventory_items(tenant_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_inventory_items_sku ON inventory_items(tenant_id, sku) WHERE sku IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_inventory_txns_tenant ON inventory_transactions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_inventory_txns_type ON inventory_transactions(transaction_type);
CREATE INDEX IF NOT EXISTS idx_inventory_txns_created_at ON inventory_transactions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_inventory_txns_batch ON inventory_transactions(batch_number) WHERE batch_number IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_inventory_txns_expiration ON inventory_transactions(expiration_date) WHERE expiration_date IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_clock_entries_tenant ON clock_entries(tenant_id);
CREATE INDEX IF NOT EXISTS idx_clock_entries_worker ON clock_entries(worker_id);
CREATE INDEX IF NOT EXISTS idx_clock_entries_type ON clock_entries(entry_type);
CREATE INDEX IF NOT EXISTS idx_clock_entries_timestamp ON clock_entries(timestamp);
CREATE INDEX IF NOT EXISTS idx_clock_entries_worker_time ON clock_entries(worker_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_clock_entries_task ON clock_entries(task_id);
CREATE INDEX IF NOT EXISTS idx_worker_locations_tenant ON worker_locations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_worker_locations_worker ON worker_locations(worker_id);
CREATE INDEX IF NOT EXISTS idx_worker_locations_time ON worker_locations(worker_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_workers_tenant ON workers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_work_logs_tenant ON work_logs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_work_logs_task ON work_logs(task_id);
CREATE INDEX IF NOT EXISTS idx_work_logs_worker ON work_logs(worker_id);

CREATE INDEX IF NOT EXISTS idx_frost_warnings_tenant ON frost_warnings(tenant_id);
CREATE INDEX IF NOT EXISTS idx_frost_warnings_station ON frost_warnings(station_id);
CREATE INDEX IF NOT EXISTS idx_frost_warnings_active ON frost_warnings(is_active);
CREATE INDEX IF NOT EXISTS idx_gdd_tenant ON growing_degree_days(tenant_id);
CREATE INDEX IF NOT EXISTS idx_gdd_site ON growing_degree_days(site_id);
CREATE INDEX IF NOT EXISTS idx_gdd_date ON growing_degree_days(date);
CREATE INDEX IF NOT EXISTS idx_gdd_station ON growing_degree_days(station_id);
CREATE INDEX IF NOT EXISTS idx_pest_risks_tenant ON pest_risks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_pest_risks_site ON pest_risks(site_id);
CREATE INDEX IF NOT EXISTS idx_pest_risks_date ON pest_risks(assessment_date);
CREATE INDEX IF NOT EXISTS idx_soil_configs_tenant ON soil_moisture_configs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_soil_configs_station ON soil_moisture_configs(station_id);
CREATE INDEX IF NOT EXISTS idx_soil_configs_active ON soil_moisture_configs(is_active);
CREATE INDEX IF NOT EXISTS idx_soil_readings_tenant ON soil_moisture_readings(tenant_id);
CREATE INDEX IF NOT EXISTS idx_soil_readings_station ON soil_moisture_readings(station_id);
CREATE INDEX IF NOT EXISTS idx_soil_readings_ts ON soil_moisture_readings(ts);
CREATE INDEX IF NOT EXISTS idx_soil_readings_timestamp ON soil_moisture_readings(timestamp);
CREATE INDEX IF NOT EXISTS idx_soil_alerts_tenant ON soil_moisture_alerts(tenant_id);

CREATE INDEX IF NOT EXISTS idx_customers_tenant ON customers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_customers_active ON customers(is_active);
CREATE UNIQUE INDEX IF NOT EXISTS idx_customers_number_tenant ON customers(customer_number, tenant_id);
CREATE INDEX IF NOT EXISTS idx_customers_name ON customers USING gin(name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_customers_email ON customers USING gin(email gin_trgm_ops);

-- ============================================================
-- 4. Triggers (updated_at)
-- ============================================================

DO $$
DECLARE
    t TEXT;
BEGIN
    FOR t IN
        SELECT tablename::TEXT AS name
        FROM pg_tables
        WHERE schemaname = 'public'
          AND tablename IN (
            'tenants', 'users', 'sites', 'equipment', 'orders',
            'weather_stations', 'weather_data', 'animals', 'tasks', 'task_data',
            'audit_logs', 'cost_centers', 'financial_records', 'pac_applications',
            'compliance_checklists', 'compliance_items',
            'fertilizer_records', 'harvest_seasons', 'harvest_lots', 'harvest_deliveries',
            'cold_chain_logs', 'olive_groves', 'olive_oil_records', 'vineyards',
            'kelter_deliveries', 'water_sources', 'water_usage', 'water_quotas',
            'plant_protection_records', 'worker_task_statuses', 'lpis_reference_parcels',
            'iot_devices', 'inventory_locations', 'inventory_items',
            'customers', 'clock_entries', 'frost_warnings', 'growing_degree_days',
            'pest_risks', 'soil_moisture_configs', 'soil_moisture_readings',
            'soil_moisture_alerts', 'workers', 'worker_locations', 'work_logs'
          )
          AND EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
            AND table_name = t
            AND column_name = 'updated_at'
          )
    LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS set_updated_at_%I ON %I', t, t);
        EXECUTE format('CREATE TRIGGER set_updated_at_%I BEFORE UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION set_updated_at()', t, t);
    END LOOP;
END $$;

-- Site area trigger
DROP TRIGGER IF EXISTS trigger_update_site_area ON sites;
CREATE TRIGGER trigger_update_site_area
    BEFORE INSERT OR UPDATE ON sites
    FOR EACH ROW EXECUTE FUNCTION update_site_area_trigger();

-- ============================================================
-- 5. Enable RLS
-- ============================================================

ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE sites ENABLE ROW LEVEL SECURITY;
ALTER TABLE equipment ENABLE ROW LEVEL SECURITY;
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE order_sites ENABLE ROW LEVEL SECURITY;
ALTER TABLE weather_stations ENABLE ROW LEVEL SECURITY;
ALTER TABLE weather_data ENABLE ROW LEVEL SECURITY;
ALTER TABLE animals ENABLE ROW LEVEL SECURITY;
ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;
ALTER TABLE task_data ENABLE ROW LEVEL SECURITY;
ALTER TABLE harvest_seasons ENABLE ROW LEVEL SECURITY;
ALTER TABLE harvest_lots ENABLE ROW LEVEL SECURITY;
ALTER TABLE harvest_deliveries ENABLE ROW LEVEL SECURITY;
ALTER TABLE cold_chain_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE olive_groves ENABLE ROW LEVEL SECURITY;
ALTER TABLE olive_oil_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE vineyards ENABLE ROW LEVEL SECURITY;
ALTER TABLE kelter_deliveries ENABLE ROW LEVEL SECURITY;
ALTER TABLE water_sources ENABLE ROW LEVEL SECURITY;
ALTER TABLE water_usage ENABLE ROW LEVEL SECURITY;
ALTER TABLE water_quotas ENABLE ROW LEVEL SECURITY;
ALTER TABLE fertilizer_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE compliance_checklists ENABLE ROW LEVEL SECURITY;
ALTER TABLE compliance_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE cost_centers ENABLE ROW LEVEL SECURITY;
ALTER TABLE financial_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE pac_applications ENABLE ROW LEVEL SECURITY;
ALTER TABLE plant_protection_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE applicator_licenses ENABLE ROW LEVEL SECURITY;
ALTER TABLE worker_task_statuses ENABLE ROW LEVEL SECURITY;
ALTER TABLE lpis_reference_parcels ENABLE ROW LEVEL SECURITY;
ALTER TABLE sigpac_parcels ENABLE ROW LEVEL SECURITY;
ALTER TABLE iot_devices ENABLE ROW LEVEL SECURITY;
ALTER TABLE inventory_locations ENABLE ROW LEVEL SECURITY;
ALTER TABLE inventory_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE inventory_transactions ENABLE ROW LEVEL SECURITY;
ALTER TABLE customers ENABLE ROW LEVEL SECURITY;
ALTER TABLE clock_entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE frost_warnings ENABLE ROW LEVEL SECURITY;
ALTER TABLE growing_degree_days ENABLE ROW LEVEL SECURITY;
ALTER TABLE pest_risks ENABLE ROW LEVEL SECURITY;
ALTER TABLE soil_moisture_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE soil_moisture_readings ENABLE ROW LEVEL SECURITY;
ALTER TABLE soil_moisture_alerts ENABLE ROW LEVEL SECURITY;
ALTER TABLE workers ENABLE ROW LEVEL SECURITY;
ALTER TABLE worker_locations ENABLE ROW LEVEL SECURITY;
ALTER TABLE work_logs ENABLE ROW LEVEL SECURITY;

-- ============================================================
-- 6. Audit Triggers
-- ============================================================

DO $$
DECLARE
    t TEXT;
BEGIN
    FOR t IN
        SELECT table_name FROM information_schema.tables
        WHERE table_schema = 'public'
          AND table_type = 'BASE TABLE'
          AND table_name NOT IN (
            'audit_logs', 'spatial_ref_sys', 'schema_migrations',
            'user_sites', 'order_sites', 'sigpac_parcels',
            'harvest_lots', 'cold_chain_logs', 'lpis_reference_parcels'
          )
    LOOP
        IF EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
            AND table_name = t
            AND column_name = 'tenant_id'
        ) THEN
            EXECUTE format('DROP TRIGGER IF EXISTS audit_trigger_%I ON %I', t, t);
            EXECUTE format('CREATE TRIGGER audit_trigger_%I AFTER INSERT OR UPDATE OR DELETE ON %I FOR EACH ROW EXECUTE FUNCTION audit_trigger_function()', t, t);
        END IF;
    END LOOP;
END $$;

DROP TRIGGER IF EXISTS audit_trigger_user_sites ON user_sites;
CREATE TRIGGER audit_trigger_user_sites
    AFTER INSERT OR DELETE ON user_sites
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

DROP TRIGGER IF EXISTS audit_trigger_order_sites ON order_sites;
CREATE TRIGGER audit_trigger_order_sites
    AFTER INSERT OR DELETE ON order_sites
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- ============================================================
-- 7. RLS Policies
-- ============================================================

CREATE POLICY tenants_select ON tenants FOR SELECT USING (is_superadmin() OR id = get_current_tenant_id());
CREATE POLICY tenants_insert ON tenants FOR INSERT WITH CHECK (is_superadmin());
CREATE POLICY tenants_update ON tenants FOR UPDATE USING (is_superadmin() OR id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR id = get_current_tenant_id());
CREATE POLICY tenants_delete ON tenants FOR DELETE USING (is_superadmin());

CREATE POLICY users_select ON users FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY users_insert ON users FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY users_update ON users FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY users_delete ON users FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY sites_select ON sites FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY sites_insert ON sites FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY sites_update ON sites FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY sites_delete ON sites FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY equipment_select ON equipment FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY equipment_insert ON equipment FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY equipment_update ON equipment FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY equipment_delete ON equipment FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY orders_select ON orders FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY orders_insert ON orders FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY orders_update ON orders FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY orders_delete ON orders FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY order_sites_select ON order_sites FOR SELECT USING (is_superadmin() OR order_id IN (SELECT id FROM orders WHERE tenant_id = get_current_tenant_id()) OR site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id()));
CREATE POLICY order_sites_insert ON order_sites FOR INSERT WITH CHECK (is_superadmin() OR order_id IN (SELECT id FROM orders WHERE tenant_id = get_current_tenant_id()) AND site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id()));
CREATE POLICY order_sites_delete ON order_sites FOR DELETE USING (is_superadmin() OR order_id IN (SELECT id FROM orders WHERE tenant_id = get_current_tenant_id()) OR site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id()));

CREATE POLICY weather_stations_select ON weather_stations FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY weather_stations_insert ON weather_stations FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY weather_stations_update ON weather_stations FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY weather_stations_delete ON weather_stations FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY weather_data_select ON weather_data FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id() OR station_id IN (SELECT id FROM weather_stations WHERE tenant_id = get_current_tenant_id()));
CREATE POLICY weather_data_insert ON weather_data FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id() OR station_id IN (SELECT id FROM weather_stations WHERE tenant_id = get_current_tenant_id()));
CREATE POLICY weather_data_update ON weather_data FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id() OR station_id IN (SELECT id FROM weather_stations WHERE tenant_id = get_current_tenant_id())) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id() OR station_id IN (SELECT id FROM weather_stations WHERE tenant_id = get_current_tenant_id()));
CREATE POLICY weather_data_delete ON weather_data FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id() OR station_id IN (SELECT id FROM weather_stations WHERE tenant_id = get_current_tenant_id()));

CREATE POLICY animals_select ON animals FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY animals_insert ON animals FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY animals_update ON animals FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY animals_delete ON animals FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY tasks_select ON tasks FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY tasks_insert ON tasks FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY tasks_update ON tasks FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY tasks_delete ON tasks FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY task_data_select ON task_data FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY task_data_insert ON task_data FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY task_data_update ON task_data FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY task_data_delete ON task_data FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY harvest_seasons_select ON harvest_seasons FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY harvest_seasons_insert ON harvest_seasons FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY harvest_seasons_update ON harvest_seasons FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY harvest_seasons_delete ON harvest_seasons FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY harvest_lots_select ON harvest_lots FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY harvest_lots_insert ON harvest_lots FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY harvest_lots_update ON harvest_lots FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY harvest_lots_delete ON harvest_lots FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY harvest_deliveries_select ON harvest_deliveries FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY harvest_deliveries_insert ON harvest_deliveries FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY harvest_deliveries_update ON harvest_deliveries FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY harvest_deliveries_delete ON harvest_deliveries FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY cold_chain_logs_select ON cold_chain_logs FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY cold_chain_logs_insert ON cold_chain_logs FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY cold_chain_logs_update ON cold_chain_logs FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY cold_chain_logs_delete ON cold_chain_logs FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY olive_groves_select ON olive_groves FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY olive_groves_insert ON olive_groves FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY olive_groves_update ON olive_groves FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY olive_groves_delete ON olive_groves FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY olive_oil_records_select ON olive_oil_records FOR SELECT USING (is_superadmin() OR olive_grove_id IN (SELECT id FROM olive_groves WHERE tenant_id = get_current_tenant_id()));
CREATE POLICY olive_oil_records_insert ON olive_oil_records FOR INSERT WITH CHECK (is_superadmin() OR olive_grove_id IN (SELECT id FROM olive_groves WHERE tenant_id = get_current_tenant_id()));
CREATE POLICY olive_oil_records_update ON olive_oil_records FOR UPDATE USING (is_superadmin() OR olive_grove_id IN (SELECT id FROM olive_groves WHERE tenant_id = get_current_tenant_id())) WITH CHECK (is_superadmin() OR olive_grove_id IN (SELECT id FROM olive_groves WHERE tenant_id = get_current_tenant_id()));
CREATE POLICY olive_oil_records_delete ON olive_oil_records FOR DELETE USING (is_superadmin() OR olive_grove_id IN (SELECT id FROM olive_groves WHERE tenant_id = get_current_tenant_id()));

CREATE POLICY vineyards_select ON vineyards FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY vineyards_insert ON vineyards FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY vineyards_update ON vineyards FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY vineyards_delete ON vineyards FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY kelter_deliveries_select ON kelter_deliveries FOR SELECT USING (is_superadmin() OR vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = get_current_tenant_id()));
CREATE POLICY kelter_deliveries_insert ON kelter_deliveries FOR INSERT WITH CHECK (is_superadmin() OR vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = get_current_tenant_id()));
CREATE POLICY kelter_deliveries_update ON kelter_deliveries FOR UPDATE USING (is_superadmin() OR vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = get_current_tenant_id())) WITH CHECK (is_superadmin() OR vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = get_current_tenant_id()));
CREATE POLICY kelter_deliveries_delete ON kelter_deliveries FOR DELETE USING (is_superadmin() OR vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = get_current_tenant_id()));

CREATE POLICY water_sources_select ON water_sources FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY water_sources_insert ON water_sources FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY water_sources_update ON water_sources FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY water_sources_delete ON water_sources FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY water_usage_select ON water_usage FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY water_usage_insert ON water_usage FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY water_usage_update ON water_usage FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY water_usage_delete ON water_usage FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY water_quotas_select ON water_quotas FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY water_quotas_insert ON water_quotas FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY water_quotas_update ON water_quotas FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY water_quotas_delete ON water_quotas FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY fertilizer_records_select ON fertilizer_records FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY fertilizer_records_insert ON fertilizer_records FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY fertilizer_records_update ON fertilizer_records FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY fertilizer_records_delete ON fertilizer_records FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY compliance_checklists_select ON compliance_checklists FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY compliance_checklists_insert ON compliance_checklists FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY compliance_checklists_update ON compliance_checklists FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY compliance_checklists_delete ON compliance_checklists FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY compliance_items_select ON compliance_items FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY compliance_items_insert ON compliance_items FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY compliance_items_update ON compliance_items FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY compliance_items_delete ON compliance_items FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY audit_logs_select ON audit_logs FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY audit_logs_insert ON audit_logs FOR INSERT WITH CHECK (is_superadmin());

CREATE POLICY cost_centers_select ON cost_centers FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY cost_centers_insert ON cost_centers FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY cost_centers_update ON cost_centers FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY cost_centers_delete ON cost_centers FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY financial_records_select ON financial_records FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY financial_records_insert ON financial_records FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY financial_records_update ON financial_records FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY financial_records_delete ON financial_records FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY pac_applications_select ON pac_applications FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY pac_applications_insert ON pac_applications FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY pac_applications_update ON pac_applications FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY pac_applications_delete ON pac_applications FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY plant_protection_records_select ON plant_protection_records FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY plant_protection_records_insert ON plant_protection_records FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY plant_protection_records_update ON plant_protection_records FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY plant_protection_records_delete ON plant_protection_records FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY applicator_licenses_select ON applicator_licenses FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY applicator_licenses_insert ON applicator_licenses FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY applicator_licenses_update ON applicator_licenses FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY applicator_licenses_delete ON applicator_licenses FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY worker_task_statuses_select ON worker_task_statuses FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY worker_task_statuses_insert ON worker_task_statuses FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY worker_task_statuses_update ON worker_task_statuses FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY worker_task_statuses_delete ON worker_task_statuses FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY lpis_tenant_isolation ON lpis_reference_parcels
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id')::UUID);

CREATE POLICY iot_devices_select ON iot_devices FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY iot_devices_insert ON iot_devices FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY iot_devices_update ON iot_devices FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY iot_devices_delete ON iot_devices FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY inventory_locations_select ON inventory_locations FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY inventory_locations_insert ON inventory_locations FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY inventory_locations_update ON inventory_locations FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY inventory_locations_delete ON inventory_locations FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY inventory_items_select ON inventory_items FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY inventory_items_insert ON inventory_items FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY inventory_items_update ON inventory_items FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY inventory_items_delete ON inventory_items FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY inventory_transactions_select ON inventory_transactions FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY inventory_transactions_insert ON inventory_transactions FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY inventory_transactions_update ON inventory_transactions FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY inventory_transactions_delete ON inventory_transactions FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY customers_select ON customers FOR SELECT USING (is_superadmin() OR tenant_id::UUID = get_current_tenant_id());
CREATE POLICY customers_insert ON customers FOR INSERT WITH CHECK (is_superadmin() OR tenant_id::UUID = get_current_tenant_id());
CREATE POLICY customers_update ON customers FOR UPDATE USING (is_superadmin() OR tenant_id::UUID = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id::UUID = get_current_tenant_id());
CREATE POLICY customers_delete ON customers FOR DELETE USING (is_superadmin() OR tenant_id::UUID = get_current_tenant_id());

CREATE POLICY clock_entries_select ON clock_entries FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY clock_entries_insert ON clock_entries FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY clock_entries_update ON clock_entries FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY clock_entries_delete ON clock_entries FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY frost_warnings_select ON frost_warnings FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY frost_warnings_insert ON frost_warnings FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY frost_warnings_update ON frost_warnings FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY frost_warnings_delete ON frost_warnings FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY growing_degree_days_select ON growing_degree_days FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY growing_degree_days_insert ON growing_degree_days FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY growing_degree_days_update ON growing_degree_days FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY growing_degree_days_delete ON growing_degree_days FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY pest_risks_select ON pest_risks FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY pest_risks_insert ON pest_risks FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY pest_risks_update ON pest_risks FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY pest_risks_delete ON pest_risks FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY soil_moisture_configs_select ON soil_moisture_configs FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY soil_moisture_configs_insert ON soil_moisture_configs FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY soil_moisture_configs_update ON soil_moisture_configs FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY soil_moisture_configs_delete ON soil_moisture_configs FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY soil_moisture_readings_select ON soil_moisture_readings FOR SELECT USING (is_superadmin() OR tenant_id::UUID = get_current_tenant_id());
CREATE POLICY soil_moisture_readings_insert ON soil_moisture_readings FOR INSERT WITH CHECK (is_superadmin() OR tenant_id::UUID = get_current_tenant_id());
CREATE POLICY soil_moisture_readings_update ON soil_moisture_readings FOR UPDATE USING (is_superadmin() OR tenant_id::UUID = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id::UUID = get_current_tenant_id());
CREATE POLICY soil_moisture_readings_delete ON soil_moisture_readings FOR DELETE USING (is_superadmin() OR tenant_id::UUID = get_current_tenant_id());

CREATE POLICY soil_moisture_alerts_select ON soil_moisture_alerts FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY soil_moisture_alerts_insert ON soil_moisture_alerts FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY soil_moisture_alerts_update ON soil_moisture_alerts FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY soil_moisture_alerts_delete ON soil_moisture_alerts FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY workers_select ON workers FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY workers_insert ON workers FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY workers_update ON workers FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY workers_delete ON workers FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY worker_locations_select ON worker_locations FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY worker_locations_insert ON worker_locations FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY worker_locations_update ON worker_locations FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY worker_locations_delete ON worker_locations FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY work_logs_select ON work_logs FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY work_logs_insert ON work_logs FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY work_logs_update ON work_logs FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id()) WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());
CREATE POLICY work_logs_delete ON work_logs FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

-- ============================================================
-- 8. Views
-- ============================================================

CREATE OR REPLACE VIEW audit_log_export AS
SELECT
    al.id, al.created_at, al.tenant_id, t.name AS tenant_name,
    al.user_id, u.email AS user_email,
    u.firstname || ' ' || u.lastname AS user_name,
    al.action, al.entity_type, al.entity_id, al.changed_fields,
    al.old_value, al.new_value, al.ip_address, al.user_agent,
    al.request_id, al.session_id
FROM audit_logs al
LEFT JOIN tenants t ON al.tenant_id = t.id
LEFT JOIN users u ON al.user_id = u.id
ORDER BY al.created_at DESC;

CREATE OR REPLACE VIEW audit_summary AS
SELECT
    tenant_id, entity_type, action, COUNT(*) as count,
    MIN(created_at) as first_occurrence, MAX(created_at) as last_occurrence
FROM audit_logs
GROUP BY tenant_id, entity_type, action
ORDER BY tenant_id, entity_type, action;

-- ============================================================
-- 9. Permissions
-- ============================================================

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'agrocore_app') THEN
        CREATE ROLE agrocore_app;
    END IF;
END $$;

GRANT SELECT ON audit_log_export TO agrocore_app;
GRANT SELECT ON audit_summary TO agrocore_app;
GRANT SELECT ON sigpac_parcels TO agrocore_app;
GRANT EXECUTE ON FUNCTION validate_parcel_against_lpis(SMALLINT, SMALLINT, SMALLINT, SMALLINT, SMALLINT, SMALLINT, SMALLINT, GEOMETRY, NUMERIC) TO agrocore_app;
GRANT EXECUTE ON FUNCTION find_duplicate_parcel TO agrocore_app;
GRANT EXECUTE ON FUNCTION calculate_area_hectares TO agrocore_app;
GRANT EXECUTE ON FUNCTION export_audit_logs(UUID, TIMESTAMPTZ, TIMESTAMPTZ, TEXT, TEXT, TEXT) TO agrocore_app;

COMMENT ON FUNCTION validate_parcel_against_lpis(SMALLINT, SMALLINT, SMALLINT, SMALLINT, SMALLINT, SMALLINT, SMALLINT, GEOMETRY, NUMERIC) IS 'Validates a declared parcel against official SIGPAC reference';
COMMENT ON FUNCTION find_duplicate_parcel IS 'Finds existing parcels by SIGPAC, REGEPAC, or boundary similarity';
COMMENT ON FUNCTION calculate_area_hectares IS 'Calculates area in hectares from geometry using geography';
COMMENT ON TABLE sigpac_parcels IS 'Official SIGPAC parcel reference data for LPIS validation';

-- ============================================================
-- 10. Final Triggers
-- ============================================================

DROP TRIGGER IF EXISTS trigger_update_site_area ON sites;
CREATE TRIGGER trigger_update_site_area
    BEFORE INSERT OR UPDATE ON sites
    FOR EACH ROW EXECUTE FUNCTION update_site_area_trigger();

-- ============================================================
-- 11. Phenology Records
-- ============================================================

CREATE TABLE IF NOT EXISTS phenology_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id),
    observation_date TIMESTAMPTZ NOT NULL,
    observer_id UUID REFERENCES users(id),
    bbch_stage INTEGER,
    description TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_phenology_tenant ON phenology_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_phenology_site ON phenology_records(site_id);
CREATE INDEX IF NOT EXISTS idx_phenology_date ON phenology_records(observation_date);

-- ============================================================
-- 12. Sites History (for audit trail)
-- ============================================================

CREATE TABLE IF NOT EXISTS sites_history (
    id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    label TEXT,
    code TEXT,
    area NUMERIC(15,2),
    crop_type JSONB,
    updated_at TIMESTAMPTZ NOT NULL,
    updated_by UUID,
    PRIMARY KEY (id, updated_at)
);

CREATE INDEX IF NOT EXISTS idx_sites_history_tenant ON sites_history(tenant_id);
CREATE INDEX IF NOT EXISTS idx_sites_history_site ON sites_history(id);

-- ============================================================
-- 13. Export function for audit logs
-- ============================================================

CREATE OR REPLACE FUNCTION export_audit_logs(
    p_tenant_id UUID DEFAULT NULL,
    p_start_date TIMESTAMPTZ DEFAULT NULL,
    p_end_date TIMESTAMPTZ DEFAULT NULL,
    p_entity_type TEXT DEFAULT NULL,
    p_action TEXT DEFAULT NULL,
    p_format TEXT DEFAULT 'json'
)
RETURNS TABLE (export_data TEXT) AS $$
DECLARE
    v_query TEXT;
    v_where_conditions TEXT[] := ARRAY[];
BEGIN
    IF p_tenant_id IS NOT NULL THEN
        v_where_conditions := v_where_conditions || format('al.tenant_id = %L', p_tenant_id);
    END IF;
    IF p_start_date IS NOT NULL THEN
        v_where_conditions := v_where_conditions || format('al.created_at >= %L', p_start_date);
    END IF;
    IF p_end_date IS NOT NULL THEN
        v_where_conditions := v_where_conditions || format('al.created_at <= %L', p_end_date);
    END IF;
    IF p_entity_type IS NOT NULL THEN
        v_where_conditions := v_where_conditions || format('al.entity_type = %L', p_entity_type);
    END IF;
    IF p_action IS NOT NULL THEN
        v_where_conditions := v_where_conditions || format('al.action = %L', p_action);
    END IF;
    v_query := 'SELECT ' ||
        'json_agg(json_build_object(''id'', al.id, ''created_at'', al.created_at, ''tenant'', t.name, ''user'', u.email, ''action'', al.action, ''entity_type'', al.entity_type, ''entity_id'', al.entity_id, ''changed_fields'', al.changed_fields, ''old_value'', al.old_value, ''new_value'', al.new_value))' ||
    ' FROM audit_logs al LEFT JOIN tenants t ON al.tenant_id = t.id LEFT JOIN users u ON al.user_id = u.id';
    IF array_length(v_where_conditions, 1) > 0 THEN
        v_query := v_query || ' WHERE ' || array_to_string(v_where_conditions, ' AND ');
    END IF;
    v_query := v_query || ' ORDER BY al.created_at DESC';
    RETURN QUERY EXECUTE v_query;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
