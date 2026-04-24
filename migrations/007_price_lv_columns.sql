-- Add canonical lv and derived eur price columns
ALTER TABLE scraped_price_rows ADD COLUMN price_min_lv DOUBLE PRECISION;
ALTER TABLE scraped_price_rows ADD COLUMN price_max_lv DOUBLE PRECISION;
ALTER TABLE scraped_price_rows ADD COLUMN price_min_eur DOUBLE PRECISION;
ALTER TABLE scraped_price_rows ADD COLUMN price_max_eur DOUBLE PRECISION;
ALTER TABLE scraped_price_rows ADD COLUMN extraction_confidence DOUBLE PRECISION DEFAULT 0.0;
ALTER TABLE scraped_price_rows ADD COLUMN extraction_strategy TEXT;
ALTER TABLE scraped_price_rows ALTER COLUMN currency SET DEFAULT 'lv';
