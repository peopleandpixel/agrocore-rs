-- General, typed key/value settings (tasks.md F1).
--
-- The audit found no settings table anywhere in the schema: the only match was
-- `soil_moisture_configs`, which is domain data. The AdminUI stored the company
-- profile in the browser's localStorage, so every setting was per-browser and
-- invisible to the backend.
--
-- Design notes:
--
-- * `key` + `tenant_id` is the identity. A NULL `tenant_id` row is a system-wide
--   default that every tenant inherits; a tenant row overrides it. That is why
--   the unique index uses COALESCE rather than a plain UNIQUE, which in
--   PostgreSQL would treat NULLs as distinct and allow duplicate defaults.
-- * `value` is JSONB so numbers, booleans and nested objects survive a
--   round-trip. `value_type` exists for validation and for the UI to render the
--   right input, not because JSONB loses it.
-- * `updated_by` is nullable because a migration or a startup default can write
--   a row without an authenticated user behind it.

CREATE TABLE IF NOT EXISTS system_settings (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID REFERENCES tenants(id) ON DELETE CASCADE,
    key         TEXT NOT NULL,
    value       JSONB NOT NULL DEFAULT 'null'::jsonb,
    value_type  TEXT NOT NULL DEFAULT 'string'
                CHECK (value_type IN ('string', 'number', 'boolean', 'json', 'array')),
    description TEXT,
    is_sensitive BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by  UUID REFERENCES users(id) ON DELETE SET NULL
);

-- One row per (tenant, key). COALESCE makes the NULL-tenant default unique
-- alongside the tenant-scoped overrides.
CREATE UNIQUE INDEX IF NOT EXISTS system_settings_tenant_key_idx
    ON system_settings (COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid), key);

-- The lookup path is "all effective settings for a tenant", which reads the
-- defaults and the overrides together.
CREATE INDEX IF NOT EXISTS system_settings_key_idx ON system_settings (key);
CREATE INDEX IF NOT EXISTS system_settings_tenant_idx ON system_settings (tenant_id);

ALTER TABLE system_settings ENABLE ROW LEVEL SECURITY;
ALTER TABLE system_settings FORCE ROW LEVEL SECURITY;

-- A tenant may read its own settings plus the system-wide defaults, and may
-- write only its own. A NULL tenant_id is not readable through the tenant
-- policies at all; it is reachable only through the `agrocore_auth`-style
-- bootstrap path, which is why the default rows are not exposed to tenants
-- directly but merged by the repository.
DROP POLICY IF EXISTS system_settings_select ON system_settings;
CREATE POLICY system_settings_select ON system_settings
    FOR SELECT
    USING (tenant_id IS NULL OR tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS system_settings_insert ON system_settings;
CREATE POLICY system_settings_insert ON system_settings
    FOR INSERT
    WITH CHECK (tenant_id IS NOT NULL AND tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS system_settings_update ON system_settings;
CREATE POLICY system_settings_update ON system_settings
    FOR UPDATE
    USING (tenant_id = get_current_tenant_id())
    WITH CHECK (tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS system_settings_delete ON system_settings;
CREATE POLICY system_settings_delete ON system_settings
    FOR DELETE
    USING (tenant_id = get_current_tenant_id());

GRANT SELECT, INSERT, UPDATE, DELETE ON system_settings TO agrocore_app;

COMMENT ON TABLE system_settings IS
    'Typed key/value settings. tenant_id NULL = system-wide default that tenants inherit.';
COMMENT ON COLUMN system_settings.value_type IS
    'Drives validation and which input the AdminUI renders. Does not change how value is stored.';

-- ---------------------------------------------------------------------------
-- Shipped defaults (tenant_id NULL).
--
-- Seeded here rather than in Rust so a fresh installation has a complete
-- settings page. `restore_defaults` deletes tenant overrides; these rows are
-- what everything falls back to.
-- ---------------------------------------------------------------------------
INSERT INTO system_settings (tenant_id, key, value, value_type, description, is_sensitive)
VALUES
    (NULL, 'company.name',        '"AgroCore Farm"',            'string',  'Company or farm name shown on documents', FALSE),
    (NULL, 'company.tax_id',      '""',                          'string',  'Tax identification number',             FALSE),
    (NULL, 'company.email',       '""',                          'string',  'Official contact email',                FALSE),
    (NULL, 'company.phone',       '""',                          'string',  'Contact phone',                         FALSE),
    (NULL, 'company.address',     '""',                          'string',  'Postal address',                        FALSE),
    (NULL, 'company.website',     '""',                          'string',  'Public website',                        FALSE),
    (NULL, 'company.country',     '"PT"',                        'string',  'ISO 3166-1 alpha-2 country code',      FALSE),

    (NULL, 'locale.default_language',  '"de"',                   'string',  'Language used when a user has none',    FALSE),
    (NULL, 'locale.supported_languages',
        '["de","en","es","fr","pt"]',                          'array',   'Languages offered in the UI',            FALSE),
    (NULL, 'locale.timezone',          '"Europe/Lisbon"',       'string',  'IANA timezone for dates and times',      FALSE),
    (NULL, 'locale.date_format',       '"%d.%m.%Y"',            'string',  'Date format pattern',                   FALSE),

    (NULL, 'backup.enabled',          'true',                  'boolean', 'Run scheduled backups',                 FALSE),
    (NULL, 'backup.schedule',         '"0 2 * * *"',            'string',  'Cron expression for backups',            FALSE),
    (NULL, 'backup.retention_days',   '30',                    'number',  'Days of backups to keep',                FALSE),
    (NULL, 'backup.targets',          '[]',                    'array',   'Configured backup destinations',         FALSE),
    (NULL, 'backup.verify',           'true',                  'boolean', 'Verify each backup after writing it',    FALSE),

    (NULL, 'notification.email.enabled', 'true',                'boolean', 'Send notifications by email',           FALSE),
    (NULL, 'notification.push.enabled',  'false',               'boolean', 'Send push notifications',               FALSE),

    (NULL, 'weather.provider',        '"open-meteo"',          'string',  'Weather data provider',                 FALSE),
    (NULL, 'weather.cache_ttl_seconds', '3600',                'number',  'Seconds a weather response is cached',   FALSE)
ON CONFLICT (COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid), key)
DO UPDATE SET value = EXCLUDED.value,
              value_type = EXCLUDED.value_type,
              description = EXCLUDED.description;
