-- Migration 37: Type entry moderation
--
-- Adds fields to track user-created type entries that need admin review.
-- This enables the "Add new type" feature in SearchableTypeDropdown, where
-- regular users can create new type entries that are automatically marked
-- for review. Admins can then approve, rename, or merge these entries.

-- Field to track which user created the type entry
ALTER TABLE type_entries
    ADD COLUMN IF NOT EXISTS created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;

COMMENT ON COLUMN type_entries.created_by_user_id IS
'User who created this type entry. NULL means system-created (migration or admin). Non-NULL means user-created via the "Add new type" feature.';

-- Field to mark entries pending admin review
ALTER TABLE type_entries
    ADD COLUMN IF NOT EXISTS pending_review BOOLEAN NOT NULL DEFAULT false;

COMMENT ON COLUMN type_entries.pending_review IS
'True when a user-created type entry needs admin review. Admins can approve (set to false), rename, or merge with existing entries.';

-- Index for finding pending review entries
CREATE INDEX IF NOT EXISTS idx_type_entries_pending_review
    ON type_entries(pending_review) WHERE pending_review;

-- Index for finding user-created entries
CREATE INDEX IF NOT EXISTS idx_type_entries_created_by
    ON type_entries(created_by_user_id) WHERE created_by_user_id IS NOT NULL;

-- View to list all pending type entries with user info
CREATE OR REPLACE VIEW v_pending_type_entries AS
SELECT
    te.id,
    te.category_id,
    tc.name AS category_name,
    te.parent_id,
    te.tenant_id,
    te.name,
    te.code,
    te.description,
    te.is_active,
    te.sort_order,
    te.created_by_user_id,
    u.email AS created_by_email,
    u.display_name AS created_by_name,
    te.created_at,
    te.updated_at
FROM type_entries te
JOIN type_categories tc ON te.category_id = tc.id
LEFT JOIN users u ON te.created_by_user_id = u.id
WHERE te.pending_review = true
ORDER BY te.created_at DESC;

COMMENT ON VIEW v_pending_type_entries IS
'All type entries awaiting admin review, with user information';

-- Function to merge a user-created type entry into an existing one
-- Updates all entity references from the old entry to the new one
CREATE OR REPLACE FUNCTION merge_type_entry(
    old_entry_id UUID,
    new_entry_id UUID
)
RETURNS void AS $$
BEGIN
    -- Validate both entries exist and are in the same category
    IF NOT EXISTS (SELECT 1 FROM type_entries WHERE id = old_entry_id) THEN
        RAISE EXCEPTION 'Source type entry % does not exist', old_entry_id;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM type_entries WHERE id = new_entry_id) THEN
        RAISE EXCEPTION 'Target type entry % does not exist', new_entry_id;
    END IF;

    PERFORM 1 FROM type_entries
    WHERE id = old_entry_id
    AND category_id = (SELECT category_id FROM type_entries WHERE id = new_entry_id);
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Type entries must be in the same category to merge';
    END IF;

    -- Mark the old entry as inactive
    UPDATE type_entries SET is_active = false, updated_at = NOW() WHERE id = old_entry_id;

    -- Note: Entity references (sites.site_type_id, orders.order_type_id, etc.)
    -- must be updated by the application layer, not here, because the table
    -- names and column names vary. The API handler should provide a list of
    -- affected entities for the admin to confirm.
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION merge_type_entry IS
'Merge a user-created type entry into an existing one. Marks the old entry as inactive. Entity references must be updated by the application layer.';

-- Function to approve a pending type entry
CREATE OR REPLACE FUNCTION approve_type_entry(entry_id UUID)
RETURNS void AS $$
BEGIN
    UPDATE type_entries
    SET pending_review = false, updated_at = NOW()
    WHERE id = entry_id AND pending_review = true;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION approve_type_entry IS
'Approve a pending type entry by setting pending_review to false';
