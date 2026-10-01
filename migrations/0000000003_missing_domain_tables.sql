-- Create the eight domain tables the repositories query but that no migration
-- ever created. Seven repos (spatial_object, tree, group, building, livestock,
-- water_usage, animal treatments/grazing) are dead at runtime until this runs:
-- every query against them fails with `relation "..." does not exist`.
--
-- Schema is taken verbatim from the repositories and domain entities:
--   trees/groups/buildings/livestock  <- tree.rs, group.rs, building.rs, livestock.rs
--   spatial_objects                   <- site.rs:463-540
--   water_usages                      <- water_usage.rs (renames water_usage)
--
-- Column types match the Rust structs in crates/domain/src/entities/.
--
-- Note on tenant_id: every repository filters with `WHERE tenant_id = $1`, but
-- three of the INSERT statements (trees, groups, livestock) do not bind it.
-- The column is therefore NOT NULL DEFAULT to the seed tenant is NOT an option;
-- instead the repositories are fixed in a follow-up change and the column is
-- created NOT NULL here so no row can land without a tenant.

-- ---------------------------------------------------------------------------
-- spatial_objects: generic geometric overlay on a plot (trees, groups,
-- buildings, livestock are stored separately but share the same shape).
-- Read by every worker GPS ping via site.rs:540.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS spatial_objects (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id         UUID REFERENCES sites(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES spatial_objects(id) ON DELETE CASCADE,
    -- NOT NULL: SpatialObject.label is String, not Option<String>.
    label           TEXT NOT NULL,
    object_type     TEXT NOT NULL,
    geometry        GEOMETRY(Geometry, 4326) NOT NULL,
    area            DOUBLE PRECISION,
    buffer_meters   DOUBLE PRECISION,
    properties      JSONB,
    custom_fields   JSONB,
    note            TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    is_temporary    BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      UUID,
    updated_by      UUID
);

CREATE INDEX IF NOT EXISTS idx_spatial_objects_tenant
    ON spatial_objects(tenant_id);
CREATE INDEX IF NOT EXISTS idx_spatial_objects_tenant_active
    ON spatial_objects(tenant_id) WHERE is_active;
CREATE INDEX IF NOT EXISTS idx_spatial_objects_site
    ON spatial_objects(site_id);
CREATE INDEX IF NOT EXISTS idx_spatial_objects_geometry
    ON spatial_objects USING GIST(geometry);

-- ---------------------------------------------------------------------------
-- groups: hierarchical plot grouping (herd, grove, coop).
-- parent_group_id references groups for nesting.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS groups (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    plot_id         UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    parent_group_id UUID REFERENCES groups(id) ON DELETE SET NULL,
    group_type      TEXT NOT NULL,
    label           TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_groups_tenant ON groups(tenant_id);
CREATE INDEX IF NOT EXISTS idx_groups_plot ON groups(plot_id);
CREATE INDEX IF NOT EXISTS idx_groups_parent ON groups(parent_group_id);
CREATE INDEX IF NOT EXISTS idx_groups_tenant_type
    ON groups(tenant_id, group_type);

-- ---------------------------------------------------------------------------
-- trees: counts per tree type inside a group, not one row per individual.
-- count is i32 in the Rust entity.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS trees (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    plot_id     UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    -- The repository binds group_id as Option<String>; the schema keeps TEXT so
    -- the binding matches instead of failing on a type mismatch.
    group_id    TEXT,
    tree_type   TEXT NOT NULL,
    count       INTEGER NOT NULL DEFAULT 0,
    label       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trees_tenant ON trees(tenant_id);
CREATE INDEX IF NOT EXISTS idx_trees_plot ON trees(plot_id);
CREATE INDEX IF NOT EXISTS idx_trees_group ON trees(group_id);
CREATE INDEX IF NOT EXISTS idx_trees_tenant_type ON trees(tenant_id, tree_type);

-- ---------------------------------------------------------------------------
-- buildings: farm structures on a plot.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS buildings (
    id             UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id      UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    plot_id        UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    building_type  TEXT NOT NULL,
    label          TEXT,
    geometry       GEOMETRY(Geometry, 4326),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_buildings_tenant ON buildings(tenant_id);
CREATE INDEX IF NOT EXISTS idx_buildings_plot ON buildings(plot_id);
CREATE INDEX IF NOT EXISTS idx_buildings_geometry
    ON buildings USING GIST(geometry);

-- ---------------------------------------------------------------------------
-- livestock: herd counts per plot, one row per group not per animal.
-- individual animals live in the existing `animals` table.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS livestock (
    id             UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id      UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    plot_id        UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    herd_id        TEXT,
    livestock_type TEXT NOT NULL,
    count          INTEGER NOT NULL DEFAULT 0,
    label          TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_livestock_tenant ON livestock(tenant_id);
CREATE INDEX IF NOT EXISTS idx_livestock_plot ON livestock(plot_id);
CREATE INDEX IF NOT EXISTS idx_livestock_herd ON livestock(herd_id);

-- ---------------------------------------------------------------------------
-- water_usage -> water_usages: the repository queries the plural form, the
-- migration created the singular one. A rename fixes every call site at once
-- instead of touching water_usage.rs.
-- ---------------------------------------------------------------------------
DO $$
BEGIN
    IF to_regclass('public.water_usage') IS NOT NULL
       AND to_regclass('public.water_usages') IS NULL
    THEN
        EXECUTE 'ALTER TABLE public.water_usage RENAME TO water_usages';
        RAISE NOTICE 'Renamed water_usage -> water_usages';
    END IF;
END
$$;

-- The repository additionally binds source_id, irrigation_method and
-- efficiency_pct, which the original table already carries. Only indexes the
-- queries rely on are added here.
CREATE INDEX IF NOT EXISTS idx_water_usages_tenant
    ON water_usages(tenant_id);
CREATE INDEX IF NOT EXISTS idx_water_usages_site
    ON water_usages(site_id);
CREATE INDEX IF NOT EXISTS idx_water_usages_source
    ON water_usages(source_id);
CREATE INDEX IF NOT EXISTS idx_water_usages_tenant_date
    ON water_usages(tenant_id, usage_date DESC);

-- ---------------------------------------------------------------------------
-- Align `animals` with crates/domain/src/entities/livestock.rs::Animal.
-- The repository reads identifier, livestock_type and status; the table has
-- tag_number and is_active instead, and has no livestock_type at all.
-- ---------------------------------------------------------------------------
ALTER TABLE public.animals
    ADD COLUMN IF NOT EXISTS identifier TEXT,
    ADD COLUMN IF NOT EXISTS livestock_type TEXT,
    ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'active';

-- identifier mirrors tag_number for the repository's read path.
UPDATE public.animals
SET identifier = tag_number
WHERE identifier IS NULL AND tag_number IS NOT NULL;

-- Keep livestock_type in sync with species: the repository reads livestock_type,
-- but every insert predating this column (including the demo seed) only supplies
-- species. The trigger also overwrites the 'unknown' sentinel whenever a real
-- species is present, so the DEFAULT below can never mask real data.
CREATE OR REPLACE FUNCTION public.sync_animal_livestock_type()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.livestock_type IS NULL OR NEW.livestock_type = ''
       OR (NEW.livestock_type = 'unknown' AND NULLIF(NEW.species, '') IS NOT NULL)
    THEN
        NEW.livestock_type := COALESCE(NULLIF(NEW.species, ''), 'unknown');
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_sync_animal_livestock_type ON public.animals;
CREATE TRIGGER trg_sync_animal_livestock_type
    BEFORE INSERT OR UPDATE ON public.animals
    FOR EACH ROW EXECUTE FUNCTION public.sync_animal_livestock_type();

ALTER TABLE public.animals
    ALTER COLUMN livestock_type SET DEFAULT 'unknown';

-- Backfill rows that already existed before the trigger was installed.
UPDATE public.animals
SET livestock_type = species
WHERE (livestock_type IS NULL OR livestock_type = '' OR livestock_type = 'unknown')
  AND species IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_animals_tenant_identifier
    ON public.animals(tenant_id, identifier);

-- ---------------------------------------------------------------------------
-- RLS on the new tables, mirroring the existing policy pattern.
-- Same caveat as everywhere else in this schema: without FORCE ROW LEVEL
-- SECURITY and without app.current_tenant_id being set, these policies never
-- evaluate (see tasks.md A3). They are added for consistency, not as a
-- guarantee.
-- ---------------------------------------------------------------------------
ALTER TABLE spatial_objects ENABLE ROW LEVEL SECURITY;
ALTER TABLE groups         ENABLE ROW LEVEL SECURITY;
ALTER TABLE trees          ENABLE ROW LEVEL SECURITY;
ALTER TABLE buildings      ENABLE ROW LEVEL SECURITY;
ALTER TABLE livestock      ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS spatial_objects_tenant_isolation ON spatial_objects;
CREATE POLICY spatial_objects_tenant_isolation ON spatial_objects
    USING (tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS groups_tenant_isolation ON groups;
CREATE POLICY groups_tenant_isolation ON groups
    USING (tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS trees_tenant_isolation ON trees;
CREATE POLICY trees_tenant_isolation ON trees
    USING (tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS buildings_tenant_isolation ON buildings;
CREATE POLICY buildings_tenant_isolation ON buildings
    USING (tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS livestock_tenant_isolation ON livestock;
CREATE POLICY livestock_tenant_isolation ON livestock
    USING (tenant_id = get_current_tenant_id());