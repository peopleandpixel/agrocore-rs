CREATE TABLE IF NOT EXISTS buildings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plot_id UUID NOT NULL REFERENCES plot(id) ON DELETE CASCADE,
    building_type VARCHAR(50) NOT NULL,
    label VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX idx_buildings_plot ON buildings(plot_id);
