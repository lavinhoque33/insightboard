-- Create dashboards table
CREATE TABLE IF NOT EXISTS dashboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    layout_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    settings_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on user_id for faster user dashboard lookups
CREATE INDEX IF NOT EXISTS idx_dashboards_user_id ON dashboards(user_id);

-- Create index on updated_at for sorting
CREATE INDEX IF NOT EXISTS idx_dashboards_updated_at ON dashboards(updated_at DESC);
