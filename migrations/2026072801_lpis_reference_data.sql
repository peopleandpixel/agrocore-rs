-- Migration: LPIS/SIGPAC Reference Data for Parcel Validation
-- Creates table for official SIGPAC parcel boundaries from Spanish cadastral data

-- LPIS/SIGPAC Reference Parcels Table
CREATE TABLE IF NOT EXISTS lpis_reference_parcels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- SIGPAC identification (Spain)
    province SMALLINT NOT NULL,           -- 2 digits: 01-52
    municipality SMALLINT NOT NULL,       -- 3 digits
    aggregate SMALLINT NOT NULL,          -- 3 digits
    zone SMALLINT NOT NULL,               -- 3 digits
    polygon SMALLINT NOT NULL,            -- 3 digits
    parcel SMALLINT NOT NULL,             -- 3 digits
    enclosure SMALLINT NOT NULL,          -- 3 digits
    
    -- Computed reference key
    sigpac_reference VARCHAR(20) GENERATED ALWAYS AS (
        LPAD(province::text, 2, '0') ||
        LPAD(municipality::text, 3, '0') ||
        LPAD(aggregate::text, 3, '0') ||
        LPAD(zone::text, 3, '0') ||
        LPAD(polygon::text, 3, '0') ||
        LPAD(parcel::text, 3, '0') ||
        LPAD(enclosure::text, 3, '0')
    ) STORED,
    
    usage_code VARCHAR(10),               -- SIGPAC usage code (e.g., 'OL' for olives, 'VI' for vineyard)
    crop_group VARCHAR(50),               -- Crop group classification
    
    -- Geometry
    geometry GEOMETRY(POLYGON, 4326) NOT NULL,
    area_hectares DOUBLE PRECISION GENERATED ALWAYS AS (
        ST_Area(geometry::geography) / 10000.0
    ) STORED,
    
    -- Metadata
    source_year SMALLINT,                 -- Year of SIGPAC data (e.g., 2024)
    source_dataset VARCHAR(100),          -- e.g., 'SIGPAC_2024', 'LPIS_ES'
    last_verified TIMESTAMPTZ,
    is_current BOOLEAN NOT NULL DEFAULT true,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_lpis_tenant_id ON lpis_reference_parcels(tenant_id);
CREATE INDEX IF NOT EXISTS idx_lpis_sigpac_ref ON lpis_reference_parcels(sigpac_reference);
CREATE INDEX IF NOT EXISTS idx_lpis_geometry ON lpis_reference_parcels USING GIST(geometry);
CREATE INDEX IF NOT EXISTS idx_lpis_province_municipality ON lpis_reference_parcels(province, municipality);
CREATE INDEX IF NOT EXISTS idx_lpis_usage_code ON lpis_reference_parcels(usage_code);
CREATE INDEX IF NOT EXISTS idx_lpis_is_current ON lpis_reference_parcels(is_current);

-- Composite unique constraint for parcel identification
CREATE UNIQUE INDEX IF NOT EXISTS uq_lpis_parcel 
ON lpis_reference_parcels(tenant_id, province, municipality, aggregate, zone, polygon, parcel, enclosure);

-- Trigger for updated_at
CREATE TRIGGER set_updated_at_lpis 
BEFORE UPDATE ON lpis_reference_parcels 
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- RLS Policies for Multi-Tenant Isolation
ALTER TABLE lpis_reference_parcels ENABLE ROW LEVEL SECURITY;

CREATE POLICY lpis_tenant_isolation ON lpis_reference_parcels
    FOR ALL TO application_role
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- Function to validate a parcel against LPIS reference data
CREATE OR REPLACE FUNCTION validate_parcel_against_lpis(
    p_tenant_id UUID,
    p_sigpac_data JSONB,
    p_boundary GEOMETRY(POLYGON, 4326)
) RETURNS JSONB AS $$
DECLARE
    v_province SMALLINT := (p_sigpac_data->>'province')::SMALLINT;
    v_municipality SMALLINT := (p_sigpac_data->>'municipality')::SMALLINT;
    v_aggregate SMALLINT := (p_sigpac_data->>'aggregate')::SMALLINT;
    v_zone SMALLINT := (p_sigpac_data->>'zone')::SMALLINT;
    v_polygon SMALLINT := (p_sigpac_data->>'polygon')::SMALLINT;
    v_parcel SMALLINT := (p_sigpac_data->>'parcel')::SMALLINT;
    v_enclosure SMALLINT := (p_sigpac_data->>'enclosure')::SMALLINT;
    v_ref_geom GEOMETRY;
    v_area_diff DOUBLE PRECISION;
    v_boundary_diff DOUBLE PRECISION;
    v_is_valid BOOLEAN := true;
    v_warnings TEXT[] := ARRAY[]::TEXT[];
BEGIN
    -- Find reference parcel
    SELECT geometry INTO v_ref_geom
    FROM lpis_reference_parcels
    WHERE tenant_id = p_tenant_id
      AND province = v_province
      AND municipality = v_municipality
      AND aggregate = v_aggregate
      AND zone = v_zone
      AND polygon = v_polygon
      AND parcel = v_parcel
      AND enclosure = v_enclosure
      AND is_current = true
    LIMIT 1;
    
    IF v_ref_geom IS NULL THEN
        v_warnings := array_append(v_warnings, 'No LPIS reference parcel found for given SIGPAC data');
        v_is_valid := false;
    ELSE
        -- Calculate area difference (in hectares)
        v_area_diff := abs(
            ST_Area(p_boundary::geography) / 10000.0 - 
            ST_Area(v_ref_geom::geography) / 10000.0
        );
        
        -- Calculate boundary difference (Hausdorff distance in meters)
        v_boundary_diff := ST_HausdorffDistance(p_boundary, v_ref_geom);
        
        -- Check if area difference is within tolerance (5%)
        IF v_area_diff > (ST_Area(v_ref_geom::geography) / 10000.0 * 0.05) THEN
            v_warnings := array_append(v_warnings, 
                format('Area difference %.2f ha exceeds 5%% tolerance', v_area_diff)
            );
            v_is_valid := false;
        END IF;
        
        -- Check if boundary difference is within tolerance (10 meters)
        IF v_boundary_diff > 10.0 THEN
            v_warnings := array_append(v_warnings, 
                format('Boundary difference %.2f m exceeds 10 m tolerance', v_boundary_diff)
            );
            v_is_valid := false;
        END IF;
        
        -- Check if geometries are spatially equal (allowing small tolerance)
        IF NOT ST_Equals(p_boundary, v_ref_geom) THEN
            v_warnings := array_append(v_warnings, 'Boundary does not exactly match LPIS reference');
        END IF;
    END IF;
    
    RETURN jsonb_build_object(
        'is_valid', v_is_valid,
        'sigpac_reference', LPAD(v_province::text, 2, '0') || LPAD(v_municipality::text, 3, '0') || 
                            LPAD(v_aggregate::text, 3, '0') || LPAD(v_zone::text, 3, '0') ||
                            LPAD(v_polygon::text, 3, '0') || LPAD(v_parcel::text, 3, '0') ||
                            LPAD(v_enclosure::text, 3, '0'),
        'area_difference_ha', v_area_diff,
        'boundary_difference_m', v_boundary_diff,
        'warnings', v_warnings
    );
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to find duplicate sites by boundary
CREATE OR REPLACE FUNCTION find_duplicate_site_by_boundary(
    p_tenant_id UUID,
    p_boundary GEOMETRY(POLYGON, 4326),
    p_exclude_id UUID DEFAULT NULL
) RETURNS SETOF sites AS $$
BEGIN
    RETURN QUERY
    SELECT s.*
    FROM sites s
    WHERE s.tenant_id = p_tenant_id
      AND s.is_active = true
      AND s.polygon IS NOT NULL
      AND ST_Equals(s.polygon, p_boundary)
      AND (p_exclude_id IS NULL OR s.id != p_exclude_id)
    LIMIT 10;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to find duplicate sites by SIGPAC data
CREATE OR REPLACE FUNCTION find_duplicate_site_by_sigpac(
    p_tenant_id UUID,
    p_sigpac_data JSONB,
    p_exclude_id UUID DEFAULT NULL
) RETURNS SETOF sites AS $$
BEGIN
    RETURN QUERY
    SELECT s.*
    FROM sites s
    WHERE s.tenant_id = p_tenant_id
      AND s.is_active = true
      AND s.sigpac_data IS NOT NULL
      AND s.sigpac_data @> p_sigpac_data
      AND (p_exclude_id IS NULL OR s.id != p_exclude_id)
    LIMIT 10;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to find duplicate sites by REGEPAC ID
CREATE OR REPLACE FUNCTION find_duplicate_site_by_regepac(
    p_tenant_id UUID,
    p_regepac_id TEXT,
    p_exclude_id UUID DEFAULT NULL
) RETURNS SETOF sites AS $$
BEGIN
    RETURN QUERY
    SELECT s.*
    FROM sites s
    WHERE s.tenant_id = p_tenant_id
      AND s.is_active = true
      AND s.regepac_id = p_regepac_id
      AND (p_exclude_id IS NULL OR s.id != p_exclude_id)
    LIMIT 10;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to automatically calculate area from boundary
CREATE OR REPLACE FUNCTION calculate_area_hectares(p_geometry GEOMETRY)
RETURNS DOUBLE PRECISION AS $$
BEGIN
    RETURN ST_Area(p_geometry::geography) / 10000.0;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

COMMENT ON TABLE lpis_reference_parcels IS 'Official SIGPAC/LPIS reference parcels for validation. Imported from official Spanish cadastral data.';
COMMENT ON FUNCTION validate_parcel_against_lpis IS 'Validates an imported parcel against LPIS reference data, checking area and boundary differences.';
COMMENT ON FUNCTION find_duplicate_site_by_boundary IS 'Finds existing sites with identical boundaries using PostGIS ST_Equals.';
COMMENT ON FUNCTION find_duplicate_site_by_sigpac IS 'Finds existing sites with matching SIGPAC data.';
COMMENT ON FUNCTION find_duplicate_site_by_regepac IS 'Finds existing sites with matching REGEPAC ID.';