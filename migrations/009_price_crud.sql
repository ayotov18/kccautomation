-- Add CRUD fields to scraped_price_rows for manual creation, editing, archiving
ALTER TABLE scraped_price_rows ADD COLUMN is_manual BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE scraped_price_rows ADD COLUMN is_user_edited BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE scraped_price_rows ADD COLUMN archived_at TIMESTAMPTZ;
ALTER TABLE scraped_price_rows ADD COLUMN notes TEXT;
ALTER TABLE scraped_price_rows ADD COLUMN title TEXT;
ALTER TABLE scraped_price_rows ADD COLUMN description TEXT;
ALTER TABLE scraped_price_rows ALTER COLUMN scrape_source_run_id DROP NOT NULL;

CREATE INDEX idx_spr_archived ON scraped_price_rows(archived_at) WHERE archived_at IS NULL;
CREATE INDEX idx_spr_manual ON scraped_price_rows(is_manual) WHERE is_manual = true;
