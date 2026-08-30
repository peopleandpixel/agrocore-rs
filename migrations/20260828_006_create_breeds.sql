CREATE TABLE IF NOT EXISTS breeds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    species VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    origin VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX idx_breeds_species ON breeds(species);
