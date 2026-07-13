-- PostgreSQL Schema für agrocore-rs mit PostGIS Unterstützung
-- Führe ich aus: psql -U agrocore -d agrocore -f migrations/001_initial_schema.sql

-- Aktiviere PostGIS Erweiterung
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Gemeinsame Funktionen
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Tabelle: tenants
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tabelle: users
CREATE TABLE users (
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
    assigned_site_ids JSONB, -- Array von UUIDs
    last_login TIMESTAMPTZ,
    refresh_token TEXT,
    refresh_token_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_tenant_id ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_refresh_token ON users(refresh_token) WHERE refresh_token IS NOT NULL;

-- Tabelle: sites (Flächen)
CREATE TABLE sites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    code TEXT,
    description TEXT,
    area NUMERIC(15,2), -- Hektar
    center_lng NUMERIC(10,8),
    center_lat NUMERIC(10,8),
    polygon GEOMETRY(POLYGON, 4326), -- PostGIS Polygon
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sites_tenant_id ON sites(tenant_id);
CREATE INDEX idx_sites_polygon ON sites USING GIST(polygon); -- Spatial Index

-- Tabelle: equipment
CREATE TABLE equipment (
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

CREATE INDEX idx_equipment_tenant_id ON equipment(tenant_id);

-- Tabelle: orders (Aufträge)
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    priority INTEGER,
    status TEXT,
    assigned_to UUID REFERENCES users(id),
    scheduled_start TIMESTAMPTZ,
    scheduled_end TIMESTAMPTZ,
    actual_start TIMESTAMPTZ,
    actual_end TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orders_tenant_id ON orders(tenant_id);
CREATE INDEX idx_orders_assigned_to ON orders(assigned_to);

-- Tabelle: animals (Viehhaltung)
CREATE TABLE animals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    tag_number TEXT,
    species TEXT,
    breed TEXT,
    birth_date DATE,
    gender TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_animals_tenant_id ON animals(tenant_id);

-- Tabelle: task_data (Worker-Task-Daten)
CREATE TABLE task_data (
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

CREATE INDEX idx_task_data_tenant ON task_data(tenant_id);
CREATE INDEX idx_task_data_task_worker ON task_data(task_id, worker_id);

-- Weitere Tabellen folgen in separaten Migrations-Dateien...

-- Trigger für updated_at
CREATE TRIGGER set_updated_at_users BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_sites BEFORE UPDATE ON sites
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_equipment BEFORE UPDATE ON equipment
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_updated_at_orders BEFORE UPDATE ON orders
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();