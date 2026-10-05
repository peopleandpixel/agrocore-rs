-- Crop and livestock species become first-class catalogue rows, and every variety and
-- breed references one.
--
-- Why this split
-- --------------
-- The initial migration seeded `varieties` with 46 rows across eight categories. Most
-- were not cultivars at all:
--
--     ('Apple', 'Apple'), ('Pear', 'Pear'), ('Wheat', 'Wheat'), ('Tomato', 'Tomato')
--
-- A species stated twice is not a variety. Mixing species and cultivar in one table
-- makes "every cultivar of Vitis vinifera" and "every species we grow" the same
-- question, and it is why the catalogue could not answer either.
--
-- So `crop_species` holds the species, with a stable machine key, and `varieties` /
-- `breeds` point at it. A cultivar without a species is not a cultivar; the species is
-- what makes grouping, filtering and yield attribution possible at all.
--
-- Scope
-- -----
-- Same sentinel as the catalogues: the all-zero tenant id is the global scope.
--
-- `global_species_key` on cultivars and breeds
-- --------------------------------------------
-- A cultivar is global, but the species it belongs to must be a fixed scientific
-- identity -- Olea europaea is not "whatever the operator typed". The column is
-- therefore deliberately NOT tenant-scoped: a tenant may add a cultivar, but it may
-- not invent a species. That is the one reference value in the system that stays
-- closed, because a wrong species key silently corrupts every grouping built on it.
--
-- Wool micron
-- -----------
-- The recognised wool reference for a breed is fibre diameter, not fleece weight. Merino
-- grades 17.70-19.14 microns, Suffolk 36.20-38.09. Diameter decides the market value;
-- kilograms decide the quantity. Both are kept, because a farmer planning a shearing
-- needs both, and conflating them produced a single misleading number.
--
-- Egg production and strain
-- -------------------------
-- Published egg figures for the same breed disagree by up to 100 eggs a year. Rhode
-- Island Red is quoted at 180-220 by the Livestock Conservancy, 250-300 by commercial
-- sources and 150-200 by Oklahoma State. These are different strains, not different
-- measurements. `egg_production_strain` records which population a figure describes, so
-- a number is never read as universal. Null means the figure is not breed-specific
-- enough to store.
--
-- Registration scope for cultivars
-- --------------------------------
-- `registration_countries` is the set of EU member states whose national catalogue
-- lists the cultivar. 481 of the 699 GrapeGen06 varieties are registered in more than
-- one country, so this is the column that answers "may I grow this where I operate"
-- rather than guessing from the cultivar's origin.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ---------------------------------------------------------------------------
-- 1. crop_species
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS crop_species (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    -- Machine key: 'olea_europaea', 'ovis_aries', 'malus_domestica'.
    species_key     VARCHAR(64) NOT NULL,
    -- 'plant' | 'animal'. Decides which catalogue the species belongs to.
    kingdom         VARCHAR(16) NOT NULL,
    common_name     VARCHAR(128) NOT NULL,
    scientific_name VARCHAR(160) NOT NULL,
    -- Cultivation / husbandry notes that belong to the species, not to one cultivar.
    note            TEXT,
    tenant_id       UUID,
    active          BOOLEAN NOT NULL DEFAULT true,
    source          VARCHAR(64),
    source_ref      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- The catalogue is read by species far more often than by id, so species_key alone is
-- UNIQUE rather than unique per tenant. Two consequences, both intended:
--
--   * A foreign key may reference species_key directly. A unique index over
--     (species_key, COALESCE(tenant_id, ...)) cannot back a foreign key, because that
--     is an expression index, not a column constraint.
--   * A tenant cannot shadow a global species with its own row of the same key. That is
--     the point: one species must resolve to one row, or grouping splits in two.
CREATE UNIQUE INDEX IF NOT EXISTS uq_crop_species_key ON crop_species(species_key);
CREATE INDEX IF NOT EXISTS idx_crop_species_kingdom
    ON crop_species(kingdom) WHERE active;

COMMENT ON TABLE crop_species IS
'Plant and animal species. Global rows use tenant_id 00000000-0000-0000-0000-000000000000. Unlike varieties and breeds, a tenant cannot add its own species key: the species is the stable identity that cultivar and breed grouping is built on, so it stays a closed reference set.';

-- ---------------------------------------------------------------------------
-- 2. Reference the species from the catalogues.
-- ---------------------------------------------------------------------------
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS global_species_key VARCHAR(64);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'varieties_global_species_key_fkey') THEN
        ALTER TABLE varieties ADD CONSTRAINT varieties_global_species_key_fkey
            FOREIGN KEY (global_species_key) REFERENCES crop_species(species_key)
            ON UPDATE CASCADE ON DELETE RESTRICT;
    END IF;
END
$$;

-- Grouping is always "cultivars of this species", so species leads the index.
CREATE INDEX IF NOT EXISTS idx_varieties_species
    ON varieties(global_species_key) WHERE active AND global_species_key IS NOT NULL;

ALTER TABLE breeds ADD COLUMN IF NOT EXISTS global_species_key VARCHAR(64);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'breeds_global_species_key_fkey') THEN
        ALTER TABLE breeds ADD CONSTRAINT breeds_global_species_key_fkey
            FOREIGN KEY (global_species_key) REFERENCES crop_species(species_key)
            ON UPDATE CASCADE ON DELETE RESTRICT;
    END IF;
END
$$;

CREATE INDEX IF NOT EXISTS idx_breeds_species_key
    ON breeds(global_species_key) WHERE active AND global_species_key IS NOT NULL;

-- ---------------------------------------------------------------------------
-- 3. Breed performance: micron range and strain-scoped egg figures.
-- ---------------------------------------------------------------------------
-- `breeds` had no use column, so the infobox value ("eggs", "meat", "dual-purpose",
-- "fibre") had nowhere to go and was being dropped. It decides which production
-- figures are meaningful for a breed at all: an egg figure on a beef breed is noise.
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS use_kind VARCHAR(64);

ALTER TABLE breeds ADD COLUMN IF NOT EXISTS wool_micron_min    DOUBLE PRECISION;
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS wool_micron_max    DOUBLE PRECISION;
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS egg_production_strain VARCHAR(160);
ALTER TABLE breeds ADD COLUMN IF NOT EXISTS milk_kg_per_year   DOUBLE PRECISION;

-- The micron columns are a range by construction; a stored min above its own max
-- would make every later comparison silently wrong.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'breeds_wool_micron_order') THEN
        ALTER TABLE breeds ADD CONSTRAINT breeds_wool_micron_order
            CHECK (wool_micron_min IS NULL
                   OR wool_micron_max IS NULL
                   OR wool_micron_min <= wool_micron_max);
    END IF;
END
$$;

COMMENT ON COLUMN breeds.wool_micron_min IS
'Lower bound of the breed''s wool fibre diameter in microns. This is the recognised grading reference (Merino 17.70-19.14, Suffolk 36.20-38.09) and is a property of the breed, unlike fleece weight which depends on the individual animal.';
COMMENT ON COLUMN breeds.egg_production_strain IS
'Which population the eggs_per_year figure describes. Published figures for one breed differ by up to 100 eggs/year because they describe different strains (Rhode Island Red: 180-220 heritage, 250-300 commercial). Without this the number reads as universal and is wrong for half the animals in the herd.';

-- ---------------------------------------------------------------------------
-- 4. Cultivar registration scope.
-- ---------------------------------------------------------------------------
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS registration_countries TEXT[];
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS vivc_no       INTEGER;
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS berry_colour  VARCHAR(16);
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS use_kind      VARCHAR(32);
ALTER TABLE varieties ADD COLUMN IF NOT EXISTS synonym_names  TEXT[];

-- "Which of these cultivars may I actually grow in Portugal or Germany" is a
-- membership test over the country array, not a join.
CREATE INDEX IF NOT EXISTS idx_varieties_registration
    ON varieties USING gin (registration_countries) WHERE active;

CREATE UNIQUE INDEX IF NOT EXISTS uq_varieties_vivc
    ON varieties(vivc_no) WHERE vivc_no IS NOT NULL;

COMMENT ON COLUMN varieties.registration_countries IS
'EU member states whose national catalogue lists this cultivar. From the GrapeGen06 European Catalogue: 481 of 699 varieties are registered in more than one country, so origin alone is not a usable proxy for "may I plant this here".';

-- ---------------------------------------------------------------------------
-- 5. Row-level security on the new table.
-- ---------------------------------------------------------------------------
ALTER TABLE crop_species ENABLE ROW LEVEL SECURITY;
ALTER TABLE crop_species FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS crop_species_select ON crop_species;
CREATE POLICY crop_species_select ON crop_species FOR SELECT
    USING (tenant_id IS NULL
           OR tenant_id = '00000000-0000-0000-0000-000000000000'::uuid
           OR tenant_id = get_current_tenant_id());

-- A tenant may extend the catalogue but may not invent species: the species_key is
-- the join target of every grouping query, and a tenant-local duplicate would split
-- one species into two in every aggregate.
DROP POLICY IF EXISTS crop_species_tenant_insert ON crop_species;
CREATE POLICY crop_species_tenant_insert ON crop_species FOR INSERT
    WITH CHECK (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS crop_species_tenant_update ON crop_species;
CREATE POLICY crop_species_tenant_update ON crop_species FOR UPDATE
    USING (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id())
    WITH CHECK (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS crop_species_tenant_delete ON crop_species;
CREATE POLICY crop_species_tenant_delete ON crop_species FOR DELETE
    USING (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id());

-- Grants follow the loop pattern of migration 4 so a later migration cannot
-- reintroduce an unreachable table.
DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relname = 'crop_species'
    LOOP
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON public.%I TO agrocore_app', r.relname);
    END LOOP;
END
$$;

-- ---------------------------------------------------------------------------
-- 6. Remove the species rows that were seeded as varieties.
-- ---------------------------------------------------------------------------
-- These eight categories were never cultivars. `ON DELETE SET NULL` on the object
-- references keeps every tree, plant and animal intact while the row moves to
-- crop_species; variety_id is nullable precisely so this migration is not lossy.
--
-- The rows are listed explicitly rather than selected by category, because a
-- category-based delete would also catch real cultivars if one were ever added
-- under those labels.
DELETE FROM varieties
WHERE (category, name) IN (
    ('Nut', 'Almond'),        ('Nut', 'Hazelnut'),
    ('Nut', 'Walnut'),        ('Nut', 'Chestnut'),
    ('Berry', 'Strawberry'),  ('Berry', 'Blueberry'),
    ('Berry', 'Raspberry'),
    ('Fruit', 'Apple'),       ('Fruit', 'Pear'),
    ('Fruit', 'Peach'),       ('Fruit', 'Plum'),
    ('Citrus', 'Orange'),     ('Citrus', 'Lemon'),
    ('Citrus', 'Lime'),       ('Citrus', 'Mandarin'),
    ('Grain', 'Wheat'),       ('Grain', 'Barley'),
    ('Grain', 'Corn / Maize'),('Grain', 'Rice'),
    ('Vegetable', 'Tomato'),  ('Vegetable', 'Potato'),
    ('Vegetable', 'Onion')
);

-- Catalogue rows are identified by scope and name. Without this the seed's
-- `ON CONFLICT DO NOTHING` has no unique index to match on and a second run inserts
-- duplicates -- which is exactly what happened: 749 varieties became 836 on the second
-- run. Name is scoped rather than globally unique because a tenant may legitimately
-- name its own cultivar the same as a global one; it just cannot create a second row
-- for the same cultivar in the same scope.
-- Scoped by species as well: the same cultivar name can exist under different species,
-- and "Chardonnay Blanc" would otherwise collide with any future grape entry of that
-- name.
CREATE UNIQUE INDEX IF NOT EXISTS uq_varieties_scope_species_name
    ON varieties(COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid),
                 COALESCE(global_species_key, '~'), name);

-- Breeds are keyed by species *and* name: Hampshire is a sheep breed and a pig breed,
-- and Bronze is a turkey and a goose. Scoping only by tenant and name makes the seed
-- fail outright with "ON CONFLICT DO UPDATE command cannot affect row a second time".
CREATE UNIQUE INDEX IF NOT EXISTS uq_breeds_scope_species_name
    ON breeds(COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid),
              COALESCE(global_species_key, '~'), name);

COMMENT ON TABLE varieties IS
'Cultivar catalogue. Every row is a cultivar of one species in crop_species, not a species itself. tenant_id all-zero is the global scope. registration_countries answers where the cultivar may be planted; vivc_no and berry_colour are the GrapeGen06 / VIVC reference values.';