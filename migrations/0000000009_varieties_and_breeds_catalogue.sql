-- Varieties and breeds become a tenant-aware reference catalogue with a global
-- scope, and individual spatial objects gain the attributes needed to group and
-- to compute yield per plant.
--
-- Background
-- ----------
-- `varieties` and `breeds` existed but were unusable in practice:
--
--   * No `tenant_id`, so no tenant could ever add its own cultivar or breed, and
--     no row-level security could ever apply. A global reference table that
--     cannot be extended by a tenant is only correct as long as nobody needs
--     something that is missing.
--   * No `active` flag, so removing a catalogue entry was impossible. Deleting
--     it instead would orphan every object that referenced it.
--   * No species/category distinction worth filtering on: olives and grapes were
--     both just `category`, and `species` was a free-text string on breeds.
--   * No performance reference values. "Merino for wool" cannot be turned into a
--     number without knowing what a Merino is expected to yield.
--
-- The scope question
-- -----------------
-- Global rows must be readable by every tenant, but only a platform operator may
-- create them. That combination cannot be expressed in row-level security, because
-- every policy is evaluated against a tenant id.
--
-- The resolution is a sentinel tenant id: the all-zero UUID means "global". Then
--
--     WHERE tenant_id = $tenant OR tenant_id = $GLOBAL_TENANT
--
-- is an ordinary indexed predicate, RLS needs no special case, and a tenant-owned
-- row can never be confused with a global one. The alternative -- a nullable
-- `tenant_id` with an `OR tenant_id IS NULL` -- reads more cleanly but puts the
-- exception in every query and every policy, and NULL is not index-friendly for
-- the half of the rows that matter.
--
-- No override semantics: a tenant cannot shadow a global entry. If a farm calls a
-- cultivar something else, it adds its own entry. That keeps resolution
-- unambiguous and makes "where did this name come from" answerable.
--
-- planted_at and variety_id on spatial_objects
-- -------------------------------------------
-- `planted_at` is the planting date of an individual object, not an attribute of
-- the cultivar, so that a block planted in two different years stays distinguishable
-- and age can be derived from it rather than stored redundantly.
--
-- Both columns are nullable on purpose. Objects imported from SIGPAC or any other
-- external source carry no cultivar and no planting date, and forcing a value
-- would either reject the import or invent data.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ---------------------------------------------------------------------------
-- 1. Sentinel constant for global catalogue rows.
-- ---------------------------------------------------------------------------
COMMENT ON TABLE varieties IS
'Cultivar catalogue. tenant_id = 00000000-0000-0000-0000-000000000000 marks a global row readable by every tenant; any other value is that tenant''s own entry. Global rows are managed by a platform operator, tenant rows by the tenant. A tenant cannot shadow a global entry -- it adds its own row instead.';

-- ---------------------------------------------------------------------------
-- 2. Extend the catalogue tables.
-- ---------------------------------------------------------------------------
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS tenant_id      UUID;
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS active         BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS species_key    VARCHAR(64);
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS source         VARCHAR(64);
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS source_ref     TEXT;
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS parent_variety_id UUID;

-- Variety rows are now tenant-scoped, so the category index alone no longer
-- serves the only query shape that matters: "the varieties I may use".
ALTER TABLE varieties DROP CONSTRAINT IF EXISTS varieties_pkey;
ALTER TABLE varieties ADD PRIMARY KEY (id);
CREATE INDEX IF NOT EXISTS idx_varieties_tenant_category
    ON varieties(tenant_id, category) WHERE active;
CREATE INDEX IF NOT EXISTS idx_varieties_tenant_species
    ON varieties(tenant_id, species_key) WHERE active;

-- Self reference for synonym/variant relations ("Touriga Nacional" vs its clone).
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'varieties_parent_variety_id_fkey') THEN
        ALTER TABLE varieties ADD CONSTRAINT varieties_parent_variety_id_fkey
            FOREIGN KEY (parent_variety_id) REFERENCES varieties(id) ON DELETE SET NULL;
    END IF;
END
$$;

COMMENT ON TABLE breeds IS
'Breed catalogue. Same scope rules as varieties: the all-zero tenant_id is the global scope, any other value is one tenant''s own entry. Performance columns are breed reference values, not measurements of an individual animal -- actual output belongs in a measurement series per animal.';

ALTER TABLE breeds ADD COLUMN IF NOT EXISTS tenant_id        UUID;
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS active           BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS species_key      VARCHAR(64);
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS source           VARCHAR(64);
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS source_ref       TEXT;

-- Reference performance values. Nullable throughout: a breed catalogue entry is
-- more useful with a birth weight alone than not entered at all.
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS birth_weight_kg       DOUBLE PRECISION;
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS mature_weight_kg      DOUBLE PRECISION;
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS productive_lifespan_years DOUBLE PRECISION;
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS eggs_per_year         DOUBLE PRECISION;
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS wool_kg_per_year      DOUBLE PRECISION;
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS daily_gain_grams      DOUBLE PRECISION;
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS litter_size           DOUBLE PRECISION;
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS gestation_days        INTEGER;

CREATE INDEX IF NOT EXISTS idx_breeds_tenant_species
    ON breeds(tenant_id, species_key) WHERE active;
CREATE INDEX IF NOT EXISTS idx_breeds_tenant_name
    ON breeds(tenant_id, name) WHERE active;

-- ---------------------------------------------------------------------------
-- 3. Attributes on the individual plant.
-- ---------------------------------------------------------------------------
-- `species`/`breed` on animals and `variety` on sites stayed free text. They are
-- left alone here: converting them is a separate change with its own data
-- migration, and mixing it into this one would make a failing backfill harder to
-- read.
ALTER TABLE spatial_objects ADD COLUMN IF NOT EXISTS planted_at DATE;
ALTER TABLE spatial_objects ADD COLUMN IF NOT EXISTS variety_id UUID;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'spatial_objects_variety_id_fkey') THEN
        ALTER TABLE spatial_objects ADD CONSTRAINT spatial_objects_variety_id_fkey
            FOREIGN KEY (variety_id) REFERENCES varieties(id) ON DELETE SET NULL;
    END IF;
END
$$;

-- The grouping queries this enables are "objects of this cultivar on this plot"
-- and "every plot that carries this cultivar". Both are tenant-scoped, so the
-- tenant id leads the index.
CREATE INDEX IF NOT EXISTS idx_spatial_objects_tenant_variety
    ON spatial_objects(tenant_id, variety_id) WHERE is_active AND variety_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_spatial_objects_tenant_planted_at
    ON spatial_objects(tenant_id, planted_at) WHERE is_active AND planted_at IS NOT NULL;

-- ---------------------------------------------------------------------------
-- 4. Row-level security.
-- ---------------------------------------------------------------------------
-- Migration 4 enables RLS by table list and FORCEs it, but only for tables whose
-- `relrowsecurity` was already set. Both catalogue tables were missing that, which
-- is why they held no policies at all. A tenant that could write them would have
-- been able to change a global cultivar for everyone.
--
-- The policies below are the global-scope rule: the global row is visible to every
-- tenant, and a tenant may only write its own rows. Creation of global rows stays
-- with a platform operator and is not granted to `agrocore_app` here.

ALTER TABLE varieties ENABLE ROW LEVEL SECURITY;
ALTER TABLE breeds    ENABLE ROW LEVEL SECURITY;

DO $$
BEGIN
    EXECUTE 'ALTER TABLE varieties FORCE ROW LEVEL SECURITY';
    EXECUTE 'ALTER TABLE breeds    FORCE ROW LEVEL SECURITY';
END
$$;

DROP POLICY IF EXISTS varieties_select ON varieties;
CREATE POLICY varieties_select ON varieties FOR SELECT
    USING (tenant_id IS NULL
           OR tenant_id = '00000000-0000-0000-0000-000000000000'::uuid
           OR tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS varieties_tenant_insert ON varieties;
CREATE POLICY varieties_tenant_insert ON varieties FOR INSERT
    WITH CHECK (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS varieties_tenant_update ON varieties;
CREATE POLICY varieties_tenant_update ON varieties FOR UPDATE
    USING (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id())
    WITH CHECK (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS varieties_tenant_delete ON varieties;
CREATE POLICY varieties_tenant_delete ON varieties FOR DELETE
    USING (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS breeds_select ON breeds;
CREATE POLICY breeds_select ON breeds FOR SELECT
    USING (tenant_id IS NULL
           OR tenant_id = '00000000-0000-0000-0000-000000000000'::uuid
           OR tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS breeds_tenant_insert ON breeds;
CREATE POLICY breeds_tenant_insert ON breeds FOR INSERT
    WITH CHECK (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS breeds_tenant_update ON breeds;
CREATE POLICY breeds_tenant_update ON breeds FOR UPDATE
    USING (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id())
    WITH CHECK (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS breeds_tenant_delete ON breeds;
CREATE POLICY breeds_tenant_delete ON breeds FOR DELETE
    USING (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id());

-- ---------------------------------------------------------------------------
-- 5. Backfill the scope of the existing rows.
-- ---------------------------------------------------------------------------
-- The existing catalogue rows were global from the start -- they are reference
-- data with no tenant context -- so they receive the sentinel rather than NULL.
-- NULL remains legal in the policies above so a half-finished backfill cannot make
-- a table unreadable, but the intended value is the sentinel.
UPDATE varieties SET tenant_id = '00000000-0000-0000-0000-000000000000'::uuid
    WHERE tenant_id IS NULL;
UPDATE breeds SET tenant_id = '00000000-0000-0000-0000-000000000000'::uuid
    WHERE tenant_id IS NULL;

-- `category` for grapes and olives is a display label; `species_key` is what code
-- filters on. Deriving it from the existing rows keeps the two consistent instead
-- of leaving a second, hand-maintained spelling of the same fact.
UPDATE varieties SET species_key = 'vitis_vinifera'
    WHERE category IN ('Grape', 'Grapes', 'grape');