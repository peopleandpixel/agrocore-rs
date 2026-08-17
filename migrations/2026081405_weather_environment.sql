-- Wetter & Umwelt: Additional weather monitoring tables
-- Supports frost warnings, growing degree day tracking, and pest risk modeling

-- Frost warnings configuration table
CREATE TABLE IF NOT EXISTS frost_warnings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    station_id UUID NOT NULL,
    threshold_temp_c NUMERIC(5,2) NOT NULL DEFAULT -2.0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    notify_email BOOLEAN NOT NULL DEFAULT true,
    notify_sms BOOLEAN NOT NULL DEFAULT false,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_frost_warnings_tenant ON frost_warnings(tenant_id);
CREATE INDEX IF NOT EXISTS idx_frost_warnings_station ON frost_warnings(station_id);
CREATE INDEX IF NOT EXISTS idx_frost_warnings_active ON frost_warnings(is_active);

-- Growing degree days tracking for crop maturity prediction
CREATE TABLE IF NOT EXISTS growing_degree_days (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    site_id UUID NOT NULL,
    date TIMESTAMPTZ NOT NULL,
    base_temp_c NUMERIC(5,2) NOT NULL DEFAULT 10.0,
    actual_mean_temp_c NUMERIC(5,2) NOT NULL,
    gdd NUMERIC(6,2) NOT NULL,
    accumulated_gdd NUMERIC(8,2) NOT NULL,
    crop_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_gdd_tenant ON growing_degree_days(tenant_id);
CREATE INDEX IF NOT EXISTS idx_gdd_site ON growing_degree_days(site_id);
CREATE INDEX IF NOT EXISTS idx_gdd_crop ON growing_degree_days(crop_type);
CREATE INDEX IF NOT EXISTS idx_gdd_date ON growing_degree_days(date);

-- Pest and disease risk assessment results
CREATE TABLE IF NOT EXISTS pest_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    site_id UUID NOT NULL,
    assessment_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    risk_level TEXT NOT NULL,
    pest_type VARCHAR(100) NOT NULL,
    confidence NUMERIC(5,4) NOT NULL DEFAULT 0.0,
    recommended_action TEXT NOT NULL,
    model_version VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pest_risks_tenant ON pest_risks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_pest_risks_site ON pest_risks(site_id);
CREATE INDEX IF NOT EXISTS idx_pest_risks_date ON pest_risks(assessment_date);

-- Soil moisture monitoring configuration
CREATE TABLE IF NOT EXISTS soil_moisture_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
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

CREATE INDEX IF NOT EXISTS idx_soil_configs_tenant ON soil_moisture_configs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_soil_configs_station ON soil_moisture_configs(station_id);
CREATE INDEX IF NOT EXISTS idx_soil_configs_active ON soil_moisture_configs(is_active);

-- Soil moisture sensor readings
CREATE TABLE IF NOT EXISTS soil_moisture_readings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    station_id UUID NOT NULL,
    site_id UUID,
    device_id VARCHAR(100),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    moisture_percent NUMERIC(5,2) NOT NULL,
    soil_temperature_c NUMERIC(5,2),
    soil_depth_cm NUMERIC(4,1),
    battery_level NUMERIC(5,2),
    signal_strength INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_soil_readings_tenant ON soil_moisture_readings(tenant_id);
CREATE INDEX IF NOT EXISTS idx_soil_readings_station ON soil_moisture_readings(station_id);
CREATE INDEX IF NOT EXISTS idx_soil_readings_timestamp ON soil_moisture_readings(timestamp);

-- Soil moisture alerts (when moisture drops below threshold)
CREATE TABLE IF NOT EXISTS soil_moisture_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    station_id UUID NOT NULL,
    site_id UUID,
    moisture_percent NUMERIC(5,2) NOT NULL,
    threshold_percent NUMERIC(5,2) NOT NULL,
    is_resolved BOOLEAN NOT NULL DEFAULT false,
    irrigation_commanded BOOLEAN NOT NULL DEFAULT false,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_soil_alerts_tenant ON soil_moisture_alerts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_soil_alerts_station ON soil_moisture_alerts(station_id);
CREATE INDEX IF NOT EXISTS idx_soil_alerts_unresolved ON soil_moisture_alerts(is_resolved);
CREATE INDEX IF NOT EXISTS idx_soil_alerts_triggered_at ON soil_moisture_alerts(triggered_at);

-- Customers table (Kundenverwaltung für Kunden & Verkauf)
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
    preferences JSONB,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_customers_tenant ON customers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_customers_active ON customers(is_active);
CREATE UNIQUE INDEX IF NOT EXISTS idx_customers_number_tenant ON customers(customer_number, tenant_id);
CREATE INDEX IF NOT EXISTS idx_customers_name ON customers USING gin(name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_customers_email ON customers USING gin(email gin_trgm_ops);
