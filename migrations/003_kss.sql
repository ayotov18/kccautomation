-- Price lists for KSS generation
CREATE TABLE IF NOT EXISTS price_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    s3_key TEXT NOT NULL,
    item_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_price_lists_user ON price_lists(user_id);
