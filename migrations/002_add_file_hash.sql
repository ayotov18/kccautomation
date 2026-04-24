-- Add file_hash column for duplicate detection (SHA-256 hex)
ALTER TABLE drawings ADD COLUMN file_hash TEXT;

-- Unique constraint per user: same user can't upload same file twice
-- Different users CAN upload the same file
CREATE UNIQUE INDEX idx_drawings_user_hash ON drawings(user_id, file_hash) WHERE file_hash IS NOT NULL;
