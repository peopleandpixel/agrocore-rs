CREATE TABLE IF NOT EXISTS livestock (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plot_id UUID NOT NULL REFERENCES plot(id) ON DELETE CASCADE,
    herd_id VARCHAR(255),
    livestock_type VARCHAR(50) NOT NULL,
    count INTEGER NOT NULL DEFAULT 1,
    label VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX idx_livestock_plot ON livestock(plot_id);
CREATE INDEX idx_livestock_herd ON livestock(herd_id);
