-- Fix: update_site_area_trigger was storing area in hectares instead of square meters.
-- The frontend sends area in m², and the display divides by 10000 to show ha.
-- The trigger must also store area in m² to be consistent.

CREATE OR REPLACE FUNCTION update_site_area_trigger()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.boundary IS NOT NULL AND
       (OLD.boundary IS NULL OR NOT ST_Equals(NEW.boundary, OLD.boundary)) THEN
        -- Store area in square meters (m²), not hectares.
        -- calculate_area_hectares returns ha, so multiply by 10000.
        NEW.area := calculate_area_hectares(NEW.boundary) * 10000;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
