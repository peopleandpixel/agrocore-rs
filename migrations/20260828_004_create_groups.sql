CREATE TABLE IF NOT EXISTS groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plot_id UUID NOT NULL REFERENCES plot(id) ON DELETE CASCADE,
    parent_group_id UUID REFERENCES groups(id) ON DELETE SET NULL,
    group_type VARCHAR(50) NOT NULL,
    label VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX idx_groups_plot ON groups(plot_id);
CREATE INDEX idx_groups_parent ON groups(parent_group_id);
