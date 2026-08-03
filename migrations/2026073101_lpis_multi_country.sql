-- Migration: Add multi-country LPIS support to sites table
-- Date: 2026-07-31
-- Description: Add lpis_country and lpis_data columns to sites table for multi-country LPIS support

-- Add LPIS country column
ALTER TABLE sites 
ADD COLUMN IF NOT EXISTS lpis_country VARCHAR(2);

-- Add LPIS generic data column (JSONB)
ALTER TABLE sites 
ADD COLUMN IF NOT EXISTS lpis_data JSONB;

-- Create index on lpis_country for filtering
CREATE INDEX IF NOT EXISTS idx_sites_lpis_country ON sites(lpis_country);

-- Create GIN index on lpis_data for JSONB queries
CREATE INDEX IF NOT EXISTS idx_sites_lpis_data ON sites USING GIN(lpis_data);

-- Add comment for documentation
COMMENT ON COLUMN sites.lpis_country IS 'LPIS country code (ES=SIGPAC, PT=iLPIS, FR=RPG, IT=SIAN, NL=BRP, DE=LPIS, PL=LPIS, AT=INVEKOS)';
COMMENT ON COLUMN sites.lpis_data IS 'Generic LPIS data structure: {country, reference, province, municipality, aggregate, zone, polygon, parcel, enclosure, usage_code, usage_description, geometry, area_hectares, official_area_ha, source_dataset, source_year}';

-- Example of lpis_data structure for different countries:
-- SIGPAC (ES): {"country": "ES", "reference": "ES411234567890123", "province": "41", "municipality": "123", "aggregate": "456", "zone": "789", "parcel": "01234", "enclosure": "567", "usage_code": "AGR", "usage_description": "Agricultural land", "geometry": {...}, "area_hectares": 10.5, "official_area_ha": 10.4, "source_dataset": "SIGPAC", "source_year": 2024}
-- iLPIS (PT): {"country": "PT", "reference": "PT1112345678", "province": "11", "municipality": "123", "aggregate": "4567", "polygon": "0123", "usage_code": "OLV", "usage_description": "Olive grove", "geometry": {...}, "area_hectares": 5.2, "official_area_ha": 5.1, "source_dataset": "iLPIS", "source_year": 2024}
-- BRP (NL): {"country": "NL", "reference": "NL12345678901234", "municipality": "GM0123", "parcel": "12345678901234", "usage_code": "265", "usage_description": "Grassland", "geometry": {...}, "area_hectares": 2.3, "official_area_ha": 2.25, "source_dataset": "BRP", "source_year": 2024}
-- RPG (FR): {"country": "FR", "reference": "FR131234567", "province": "13", "municipality": "123", "aggregate": "4567", "usage_code": "HER", "usage_description": "Herbaceous", "geometry": {...}, "area_hectares": 8.7, "official_area_ha": 8.6, "source_dataset": "RPG", "source_year": 2024}
-- SIAN (IT): {"country": "IT", "reference": "IT123456789", "province": "12", "municipality": "345", "aggregate": "6789", "usage_code": "VIT", "usage_description": "Vineyard", "geometry": {...}, "area_hectares": 3.4, "official_area_ha": 3.35, "source_dataset": "SIAN", "source_year": 2024}