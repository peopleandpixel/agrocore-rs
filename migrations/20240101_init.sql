-- PostGIS Extension aktivieren
CREATE EXTENSION IF NOT EXISTS postgis;

-- ========================================
-- Tenant (Tenant Management)
-- ========================================
CREATE TABLE tenants (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    config JSONB,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- ========================================
-- Sites (Flächen)
-- ========================================
CREATE TABLE sites (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    business_id UUID,
    label VARCHAR(200) NOT NULL,
    site_type VARCHAR(30) NOT NULL,
    crop_type VARCHAR(30) NOT NULL,
    variety VARCHAR(100),
    area DOUBLE PRECISION NOT NULL CHECK (area >= 0),
    gross_area DOUBLE PRECISION CHECK (gross_area >= 0),
    boundary GEOMETRY(POLYGON, 4326), -- WGS84
    center GEOMETRY(POINT, 4326),
    is_active BOOLEAN DEFAULT true,
    is_temporary BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    created_by UUID,
    updated_by UUID
);

CREATE INDEX idx_sites_boundary ON sites USING GIST(boundary);
CREATE INDEX idx_sites_tenant ON sites(tenant_id);

-- ========================================
-- Users
-- ========================================
CREATE TABLE users (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    firstname VARCHAR(100) NOT NULL,
    lastname VARCHAR(100) NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    roles JSONB NOT NULL, -- Vec<UserRole> als JSONB Array
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);

-- ========================================
-- Orders
-- ========================================
CREATE TABLE orders (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    label VARCHAR(200) NOT NULL,
    order_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    site_ids UUID[], -- Many-to-Many mit sites via order_sites
    deadline_date DATE,
    planned_date DATE,
    recurrence JSONB,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Junction table für sites
CREATE TABLE order_sites (
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    PRIMARY KEY (order_id, site_id)
);

-- ========================================
-- Weather Data
-- ========================================
CREATE TABLE weather_stations (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    label VARCHAR(100) NOT NULL,
    station_type VARCHAR(20) NOT NULL,
    location GEOMETRY(POINT, 4326),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_weather_stations_location ON weather_stations USING GIST(location);

CREATE TABLE weather_data (
    id UUID PRIMARY KEY,
    station_id UUID REFERENCES weather_stations(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL,
    temperature_c DOUBLE PRECISION,
    humidity_percent DOUBLE PRECISION,
    precipitation_mm DOUBLE PRECISION,
    wind_speed_kmh DOUBLE PRECISION,
    -- ... weitere Spalten
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_weather_data_station_time ON weather_data(station_id, timestamp DESC);

-- ========================================
-- Tasks (Arbeiten)
-- ========================================
CREATE TABLE tasks (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    worker_id UUID REFERENCES users(id) ON DELETE SET NULL,
    site_id UUID REFERENCES sites(id) ON DELETE SET NULL,
    description TEXT NOT NULL,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Weitere Tabellen: equipment, livestock, plant_protection, etc.