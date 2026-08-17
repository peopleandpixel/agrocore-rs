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
