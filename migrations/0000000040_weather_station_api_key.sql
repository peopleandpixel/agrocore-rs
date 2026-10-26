-- ============================================================================
-- Add api_key_config to weather_stations for storing API credentials
-- ============================================================================

ALTER TABLE weather_stations ADD COLUMN IF NOT EXISTS api_key_config TEXT;
