ALTER TABLE vineyards
    ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE;

CREATE INDEX IF NOT EXISTS idx_vineyards_active ON vineyards(is_active);
