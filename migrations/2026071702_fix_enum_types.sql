-- Behebung von Typen-Mismatches bei Enum-Spalten
-- Umstellung von VARCHAR/TEXT auf JSONB für Spalten, die in Rust als #[sqlx(json)] markiert sind

-- Tabelle: sites
ALTER TABLE sites 
ALTER COLUMN site_type TYPE JSONB USING to_jsonb(site_type),
ALTER COLUMN crop_type TYPE JSONB USING to_jsonb(crop_type);

-- Tabelle: orders
ALTER TABLE orders
ALTER COLUMN status TYPE JSONB USING to_jsonb(status),
ALTER COLUMN order_type TYPE JSONB USING to_jsonb(order_type);

-- Tabelle: equipment
ALTER TABLE equipment
ALTER COLUMN equipment_type TYPE JSONB USING to_jsonb(equipment_type);

-- Tabelle: weather_stations
ALTER TABLE weather_stations
ALTER COLUMN station_type TYPE JSONB USING to_jsonb(station_type);

-- Tabelle: phenology_records
ALTER TABLE phenology_records
ALTER COLUMN stage TYPE JSONB USING to_jsonb(stage);

-- Tabelle: harvest_lots
ALTER TABLE harvest_lots
ALTER COLUMN status TYPE JSONB USING to_jsonb(status);
