-- Migration: LPIS/SIGPAC Reference Data for Parcel Validation
-- Provides reference tables for Spanish SIGPAC parcel validation

-- Enable PostGIS if not already enabled
CREATE EXTENSION IF NOT EXISTS postgis;

-- Table: sigpac_parcels
-- Stores official SIGPAC parcel geometries for validation
CREATE TABLE IF NOT EXISTS sigpac_parcels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL,  -- Optional: can be global or per-tenant
    
    -- SIGPAC identification (official codes)
    province SMALLINT NOT NULL,           -- 2 digits: 01-52
    municipality SMALLINT NOT NULL,       -- 3 digits: 001-999
    aggregate SMALLINT NOT NULL,          -- 3 digits
    zone SMALLINT NOT NULL,               -- 3 digits
    polygon SMALLINT NOT NULL,            -- 3 digits
    parcel SMALLINT NOT NULL,             -- 3 digits
    enclosure SMALLINT NOT NULL,          -- 3 digits
    
    -- Composite SIGPAC reference (e.g., "02013001001001001")
    sigpac_reference VARCHAR(20) GENERATED ALWAYS AS (
        LPAD(province::TEXT, 2, '0') ||
        LPAD(municipality::TEXT, 3, '0') ||
        LPAD(aggregate::TEXT, 3, '0') ||
        LPAD(zone::TEXT, 3, '0') ||
        LPAD(polygon::TEXT, 3, '0') ||
        LPAD(parcel::TEXT, 3, '0') ||
        LPAD(enclosure::TEXT, 3, '0')
    ) STORED,
    
    -- Official land use code (CULTIVO, PASTO, MONTE, etc.)
    usage_code VARCHAR(10),
    usage_description TEXT,
    
    -- Official area in hectares (from SIGPAC)
    official_area_ha NUMERIC(10,4),
    
    -- Geometry in WGS84 (EPSG:4326)
    geometry GEOMETRY(POLYGON, 4326) NOT NULL,
    
    -- Metadata
    source_year SMALLINT,                 -- Year of SIGPAC data
    last_updated TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Unique index on SIGPAC reference
CREATE UNIQUE INDEX IF NOT EXISTS idx_sigpac_parcels_reference 
    ON sigpac_parcels(sigpac_reference);

-- Spatial index for geometry queries
CREATE INDEX IF NOT EXISTS idx_sigpac_parcels_geometry 
    ON sigpac_parcels USING GIST(geometry);

-- Index for tenant filtering
CREATE INDEX IF NOT EXISTS idx_sigpac_parcels_tenant 
    ON sigpac_parcels(tenant_id);

-- Index for component lookups
CREATE INDEX IF NOT EXISTS idx_sigpac_parcels_components 
    ON sigpac_parcels(province, municipality, aggregate, zone, polygon, parcel, enclosure);

-- Function: Validate a parcel against SIGPAC reference
-- Returns validation result with area/boundary differences
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
    -- Find reference parcel
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
    
    -- Calculate area difference
    IF p_area_ha IS NOT NULL AND v_ref.official_area_ha IS NOT NULL THEN
        v_area_diff := ABS(p_area_ha - v_ref.official_area_ha);
        IF v_area_diff > 0.5 THEN  -- More than 0.5 ha difference
            v_warnings := array_append(v_warnings, 
                format('Area difference: %.2f ha (declared: %.2f, official: %.2f)', 
                       v_area_diff, p_area_ha, v_ref.official_area_ha));
        END IF;
    END IF;
    
    -- Calculate boundary difference (Hausdorff distance as proxy)
    IF p_boundary IS NOT NULL THEN
        v_boundary_diff := ST_HausdorffDistance(p_boundary, v_ref.geometry);
        IF v_boundary_diff > 10 THEN  -- More than 10 meters
            v_warnings := array_append(v_warnings,
                format('Boundary difference: %.2f meters', v_boundary_diff));
        END IF;
        
        -- Check if geometries are roughly equivalent (90% overlap)
        IF ST_Area(ST_Intersection(p_boundary, v_ref.geometry)) / 
           ST_Area(ST_Union(p_boundary, v_ref.geometry)) < 0.9 THEN
            v_warnings := array_append(v_warnings, 'Low geometry overlap with reference parcel');
        END IF;
    END IF;
    
    -- Determine validity (valid if no critical warnings)
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

-- Function: Find duplicate parcels by boundary or SIGPAC
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
    -- 1. Check by SIGPAC reference
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
    
    -- 2. Check by REGEPAC ID
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
    
    -- 3. Check by exact boundary match (PostGIS ST_Equals)
    IF p_boundary IS NOT NULL THEN
        RETURN QUERY
        SELECT s.id, 'ExactBoundary'::TEXT, 1.0::NUMERIC(5,4)
        FROM sites s
        WHERE s.tenant_id = p_tenant_id
          AND ST_Equals(s.boundary, p_boundary)
          AND s.is_active = TRUE
        LIMIT 1;
        
        IF FOUND THEN RETURN; END IF;
        
        -- 4. Check by high similarity (ST_Area intersection > 95%)
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
    
    -- No duplicate found
    RETURN;
END;
$$ LANGUAGE plpgsql;

-- Function: Auto-calculate area from boundary geometry (hectares)
CREATE OR REPLACE FUNCTION calculate_area_hectares(p_geometry GEOMETRY) 
RETURNS NUMERIC(10,4) AS $$
BEGIN
    IF p_geometry IS NULL THEN
        RETURN NULL;
    END IF;
    -- Use geography for accurate area calculation on WGS84
    RETURN ROUND(ST_Area(p_geometry::GEOGRAPHY) / 10000.0, 4);
END;
$$ LANGUAGE plpgsql;

-- Trigger: Auto-update area when boundary changes
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

DROP TRIGGER IF EXISTS trigger_update_site_area ON sites;
CREATE TRIGGER trigger_update_site_area
    BEFORE INSERT OR UPDATE ON sites
    FOR EACH ROW EXECUTE FUNCTION update_site_area_trigger();

-- Grant permissions
GRANT SELECT ON sigpac_parcels TO agrocore_app;
GRANT EXECUTE ON FUNCTION validate_parcel_against_lpis TO agrocore_app;
GRANT EXECUTE ON FUNCTION find_duplicate_parcel TO agrocore_app;
GRANT EXECUTE ON FUNCTION calculate_area_hectares TO agrocore_app;

-- Comments
COMMENT ON TABLE sigpac_parcels IS 'Official SIGPAC parcel reference data for LPIS validation';
COMMENT ON FUNCTION validate_parcel_against_lpis IS 'Validates a declared parcel against official SIGPAC reference';
COMMENT ON FUNCTION find_duplicate_parcel IS 'Finds existing parcels by SIGPAC, REGEPAC, or boundary similarity';
COMMENT ON FUNCTION calculate_area_hectares IS 'Calculates area in hectares from geometry using geography';