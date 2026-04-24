-- Phase 4: Per-line audit trail, dual-code (СЕК + Образец 9.1), renovation
-- metadata, EN 1090 facets, floor attribution, and report-level validation.

ALTER TABLE kss_line_items
    ADD COLUMN IF NOT EXISTS obrazec_ref TEXT,           -- e.g. "XIII-т.63"
    ADD COLUMN IF NOT EXISTS is_renovation BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS ewc_waste_code TEXT,        -- e.g. "17 01 01"
    ADD COLUMN IF NOT EXISTS floor_label TEXT,           -- "Сутерен" / "Партер" / "Ет. 1"
    -- EN 1090 facets for steel structures (СЕК14)
    ADD COLUMN IF NOT EXISTS en1090_exc TEXT,            -- "EXC2" / "EXC3" / "EXC4"
    ADD COLUMN IF NOT EXISTS steel_grade TEXT,           -- "S235" / "S275" / "S355"
    ADD COLUMN IF NOT EXISTS coating_system TEXT,        -- "C3" / "C4" per EN ISO 12944
    -- Full defensible build-up — source URL, component breakdown, AI/СЕК provenance
    ADD COLUMN IF NOT EXISTS audit_trail JSONB;

CREATE INDEX IF NOT EXISTS idx_kss_line_items_obrazec_ref
    ON kss_line_items(report_id, obrazec_ref);

ALTER TABLE kss_reports
    ADD COLUMN IF NOT EXISTS validation_warnings JSONB,  -- [{check, severity, message, expected, actual}]
    ADD COLUMN IF NOT EXISTS is_renovation BOOLEAN NOT NULL DEFAULT FALSE;
