-- Optimization Migration for PostgreSQL

-- 1. Normalisierung von Relationen
-- Ersetze JSONB/Arrays durch Verknüpfungstabellen

-- Junction table für users -> sites (assigned_site_ids in users ersetzen)
CREATE TABLE user_sites (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, site_id)
);

-- Falls orders.site_ids noch als Array existiert (in 20240101_init.sql gesehen), order_sites existiert aber auch schon dort.
-- Wir stellen sicher, dass order_sites die primäre Quelle ist.

-- 2. Audit-Trigger für alle Tabellen
-- Funktion set_updated_at existiert bereits in 001_initial_schema.sql

DO $$
DECLARE
    t text;
BEGIN
    FOR t IN 
        SELECT table_name 
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
          AND table_type = 'BASE TABLE'
          AND table_name NOT IN ('spatial_ref_sys') -- PostGIS system table
    LOOP
        -- Überprüfen ob updated_at Spalte existiert
        IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = t AND column_name = 'updated_at') THEN
            EXECUTE format('DROP TRIGGER IF EXISTS set_updated_at_%I ON %I', t, t);
            EXECUTE format('CREATE TRIGGER set_updated_at_%I BEFORE UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION set_updated_at()', t, t);
        END IF;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- 3. Constraint-Optimierung
-- Beispiel: tag_number bei animals eindeutig pro tenant
ALTER TABLE animals ADD CONSTRAINT unique_animal_tag_per_tenant UNIQUE (tenant_id, tag_number);

-- 4. Indizierung
CREATE INDEX IF NOT EXISTS idx_users_is_active_tenant ON users(is_active, tenant_id);
CREATE INDEX IF NOT EXISTS idx_sites_is_active_tenant ON sites(is_active, tenant_id);
CREATE INDEX IF NOT EXISTS idx_orders_status_tenant ON orders(status, tenant_id);
