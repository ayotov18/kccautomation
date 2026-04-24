-- Allow jobs without a drawing (e.g., scrape jobs)
ALTER TABLE jobs ALTER COLUMN drawing_id DROP NOT NULL;
