-- ═══════════════════════════════════════════════════════════
-- Self-hosted price RAG (no embeddings, no model server)
--
-- Users upload their actual project offers / KSS files (Excel) and the
-- contents become a per-user price corpus that the KSS generator can
-- "scatter" through to find prices for items it's about to quote — instead
-- of re-discovering prices via Perplexity each time.
--
-- Retrieval is pg_trgm + tsvector ranking, no embeddings:
--   Bulgarian KSS descriptions are highly stylized templates ("Доставка
--   и монтаж на ..."). Trigram similarity catches near-matches like
--   "Шперплат 18мм" vs "Шперплат мебелен 18 мм" without an embedding model.
--
-- Per-user only. We do NOT pool corpora across tenants — that's a privacy
-- and quality boundary; one user's overpriced offer shouldn't quietly drift
-- another user's quotes.
-- ═══════════════════════════════════════════════════════════

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE user_price_imports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    sheet_count INT NOT NULL DEFAULT 0,
    row_count INT NOT NULL DEFAULT 0,
    skipped_count INT NOT NULL DEFAULT 0,
    imported_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Idempotency: re-uploading the same file (by content hash) is a no-op
    -- rather than a duplicate corpus.
    UNIQUE (user_id, file_hash)
);
CREATE INDEX idx_user_price_imports_user ON user_price_imports(user_id);

CREATE TABLE user_price_corpus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    import_id UUID REFERENCES user_price_imports(id) ON DELETE CASCADE,
    sek_code TEXT,
    description TEXT NOT NULL,
    unit TEXT NOT NULL,
    quantity DOUBLE PRECISION,
    material_price_lv DOUBLE PRECISION DEFAULT 0,
    labor_price_lv DOUBLE PRECISION DEFAULT 0,
    total_unit_price_lv DOUBLE PRECISION DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'EUR',
    source_sheet TEXT,
    source_row INT,
    -- Bulgarian text-search vector. The simple_unaccent config is good
    -- enough — we don't need stemming, just unaccented + lowercased tokens
    -- for ts_rank. Maintained automatically by trigger.
    description_tsv tsvector,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Trigram fuzzy-match index — the workhorse of RAG retrieval.
CREATE INDEX idx_user_price_corpus_desc_trgm
    ON user_price_corpus USING GIN (description gin_trgm_ops);

-- Full-text search for tie-breaking when trigram similarity is close.
CREATE INDEX idx_user_price_corpus_desc_tsv
    ON user_price_corpus USING GIN (description_tsv);

CREATE INDEX idx_user_price_corpus_user ON user_price_corpus(user_id);
CREATE INDEX idx_user_price_corpus_sek  ON user_price_corpus(user_id, sek_code);
CREATE INDEX idx_user_price_corpus_import ON user_price_corpus(import_id);

-- Auto-maintain the tsvector. Use 'simple' config (built-in, no extra
-- packages). For Bulgarian we'd want 'pg_catalog.bulgarian' if the host
-- has it; 'simple' works on every distribution and still produces useful
-- token matches when combined with pg_trgm.
CREATE OR REPLACE FUNCTION user_price_corpus_tsv_update()
RETURNS trigger AS $$
BEGIN
    NEW.description_tsv := to_tsvector('simple', COALESCE(NEW.description, ''));
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_user_price_corpus_tsv
BEFORE INSERT OR UPDATE OF description ON user_price_corpus
FOR EACH ROW EXECUTE FUNCTION user_price_corpus_tsv_update();
