CREATE TABLE IF NOT EXISTS trees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plot_id UUID NOT NULL REFERENCES plot(id) ON DELETE CASCADE,
    group_id VARCHAR(255),
    tree_type VARCHAR(50) NOT NULL,
    count INTEGER NOT NULL DEFAULT 1,
    label VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX idx_trees_plot ON trees(plot_id);
CREATE INDEX idx_trees_group ON trees(group_id);
