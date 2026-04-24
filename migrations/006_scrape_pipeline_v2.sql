-- Scrape run: one row per worker execution
CREATE TABLE scrape_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID REFERENCES jobs(id) ON DELETE SET NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'running',
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    total_sources INT NOT NULL DEFAULT 0,
    successful_sources INT NOT NULL DEFAULT 0,
    failed_sources INT NOT NULL DEFAULT 0,
    artifact_failures INT NOT NULL DEFAULT 0,
    notes JSONB
);

-- Per-source-URL scrape tracking
CREATE TABLE scrape_source_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scrape_run_id UUID NOT NULL REFERENCES scrape_runs(id) ON DELETE CASCADE,
    site TEXT NOT NULL,
    url TEXT NOT NULL,
    category_hint TEXT,
    fetch_status TEXT NOT NULL DEFAULT 'pending',
    parse_status TEXT NOT NULL DEFAULT 'pending',
    db_status TEXT NOT NULL DEFAULT 'pending',
    artifact_status TEXT NOT NULL DEFAULT 'skipped',
    http_status INT,
    elapsed_ms INT,
    html_len INT,
    parsed_count INT DEFAULT 0,
    error_message TEXT,
    fetched_at TIMESTAMPTZ
);

CREATE INDEX idx_ssr_run ON scrape_source_runs(scrape_run_id);

-- Normalized price rows — the actual product data table
CREATE TABLE scraped_price_rows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scrape_source_run_id UUID NOT NULL REFERENCES scrape_source_runs(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    site TEXT NOT NULL,
    source_url TEXT NOT NULL,
    category_slug TEXT,
    item_name TEXT NOT NULL,
    unit TEXT,
    price_min DOUBLE PRECISION,
    price_max DOUBLE PRECISION,
    price_avg DOUBLE PRECISION,
    currency TEXT DEFAULT 'EUR',
    raw_price_text TEXT,
    sek_code TEXT,
    sek_group TEXT,
    mapping_confidence DOUBLE PRECISION DEFAULT 0.0,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_spr_user ON scraped_price_rows(user_id);
CREATE INDEX idx_spr_sek ON scraped_price_rows(sek_code);
CREATE INDEX idx_spr_site ON scraped_price_rows(site);
CREATE INDEX idx_spr_source_run ON scraped_price_rows(scrape_source_run_id);
