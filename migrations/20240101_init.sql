-- PostgreSQL Schema für agrocore-rs mit PostGIS Unterstützung
-- Ersetzt 001_initial_schema.sql und 20240101_init.sql

-- Aktiviere PostGIS und pgtrgm Erweiterungen
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Gemeinsame Funktionen
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Tabelle: tenants
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    slug VARCHAR(50) UNIQUE,
    config JSONB,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tabelle: users
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    firstname VARCHAR(100) NOT NULL,
    lastname VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    roles JSONB NOT NULL DEFAULT '[]'::JSONB, -- Array von UserRole Enums
    is_active BOOLEAN NOT NULL DEFAULT true,
    internal_cost_per_hour NUMERIC(10,2),
    external_cost_per_hour NUMERIC(10,2),
    color VARCHAR(7), -- Hex Color
    language VARCHAR(10),
    assigned_site_ids JSONB, -- Array von UUIDs (wird in optimization.sql migriert)
    last_login TIMESTAMPTZ,
    refresh_token TEXT,
    refresh_token_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_refresh_token ON users(refresh_token) WHERE refresh_token IS NOT NULL;

-- Tabelle: sites (Flächen)
CREATE TABLE IF NOT EXISTS sites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    business_id UUID,
    label TEXT NOT NULL,
    code TEXT,
    description TEXT,
    site_type VARCHAR(30),
    crop_type VARCHAR(30),
    variety VARCHAR(100),
    area NUMERIC(15,2), -- Hektar
    gross_area NUMERIC(15,2),
    center_lng NUMERIC(10,8),
    center_lat NUMERIC(10,8),
    polygon GEOMETRY(POLYGON, 4326), -- PostGIS Polygon
    center GEOMETRY(POINT, 4326),
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_temporary BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_by UUID
);

CREATE INDEX IF NOT EXISTS idx_sites_tenant_id ON sites(tenant_id);
CREATE INDEX IF NOT EXISTS idx_sites_polygon ON sites USING GIST(polygon);
CREATE INDEX IF NOT EXISTS idx_sites_boundary ON sites USING GIST(polygon); -- Kompatibilität mit altem Index-Namen

-- Tabelle: equipment
CREATE TABLE IF NOT EXISTS equipment (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    code TEXT,
    equipment_type TEXT,
    in_usage BOOLEAN NOT NULL DEFAULT false,
    maintenance_intervals JSONB,
    next_maintenance_date DATE,
    last_maintenance_hours INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_equipment_tenant_id ON equipment(tenant_id);

-- Tabelle: orders (Aufträge)
CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    title TEXT, -- Alternative Bezeichnung
    description TEXT,
    priority INTEGER,
    status TEXT NOT NULL,
    order_type VARCHAR(50),
    assigned_to UUID REFERENCES users(id),
    scheduled_start TIMESTAMPTZ,
    scheduled_end TIMESTAMPTZ,
    actual_start TIMESTAMPTZ,
    actual_end TIMESTAMPTZ,
    deadline_date DATE,
    planned_date DATE,
    recurrence JSONB,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orders_tenant_id ON orders(tenant_id);
CREATE INDEX IF NOT EXISTS idx_orders_assigned_to ON orders(assigned_to);

-- Junction table für sites
CREATE TABLE IF NOT EXISTS order_sites (
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    PRIMARY KEY (order_id, site_id)
);

-- Tabelle: weather_stations
CREATE TABLE IF NOT EXISTS weather_stations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    label VARCHAR(100) NOT NULL,
    station_type VARCHAR(20) NOT NULL,
    location GEOMETRY(POINT, 4326),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_weather_stations_location ON weather_stations USING GIST(location);

-- Tabelle: weather_data
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

CREATE INDEX IF NOT EXISTS idx_weather_data_station_time ON weather_data(station_id, timestamp DESC);

-- Tabelle: animals (Viehhaltung)
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
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_animals_tenant_id ON animals(tenant_id);
CREATE INDEX IF NOT EXISTS idx_animals_site ON animals(current_site_id);
CREATE INDEX IF NOT EXISTS idx_animals_species ON animals(species);

-- Tabelle: task_data (Worker-Task-Daten)
CREATE TABLE IF NOT EXISTS task_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    task_id UUID NOT NULL,
    worker_id UUID NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    status TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_task_data_tenant ON task_data(tenant_id);
CREATE INDEX IF NOT EXISTS idx_task_data_task_worker ON task_data(task_id, worker_id);

-- Tabelle: tasks (Arbeiten)
CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    worker_id UUID REFERENCES users(id) ON DELETE SET NULL,
    site_id UUID REFERENCES sites(id) ON DELETE SET NULL,
    description TEXT NOT NULL,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Trigger für updated_at
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'set_updated_at_users') THEN
        CREATE TRIGGER set_updated_at_users BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION set_updated_at();
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'set_updated_at_sites') THEN
        CREATE TRIGGER set_updated_at_sites BEFORE UPDATE ON sites FOR EACH ROW EXECUTE FUNCTION set_updated_at();
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'set_updated_at_equipment') THEN
        CREATE TRIGGER set_updated_at_equipment BEFORE UPDATE ON equipment FOR EACH ROW EXECUTE FUNCTION set_updated_at();
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'set_updated_at_orders') THEN
        CREATE TRIGGER set_updated_at_orders BEFORE UPDATE ON orders FOR EACH ROW EXECUTE FUNCTION set_updated_at();
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'set_updated_at_tenants') THEN
        CREATE TRIGGER set_updated_at_tenants BEFORE UPDATE ON tenants FOR EACH ROW EXECUTE FUNCTION set_updated_at();
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'set_updated_at_animals') THEN
        CREATE TRIGGER set_updated_at_animals BEFORE UPDATE ON animals FOR EACH ROW EXECUTE FUNCTION set_updated_at();
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'set_updated_at_task_data') THEN
        CREATE TRIGGER set_updated_at_task_data BEFORE UPDATE ON task_data FOR EACH ROW EXECUTE FUNCTION set_updated_at();
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'set_updated_at_tasks') THEN
        CREATE TRIGGER set_updated_at_tasks BEFORE UPDATE ON tasks FOR EACH ROW EXECUTE FUNCTION set_updated_at();
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'set_updated_at_weather_stations') THEN
        CREATE TRIGGER set_updated_at_weather_stations BEFORE UPDATE ON weather_stations FOR EACH ROW EXECUTE FUNCTION set_updated_at();
    END IF;
END $$;
