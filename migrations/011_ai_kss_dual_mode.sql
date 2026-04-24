-- ═══════════════════════════════════════════════════════════
-- Drawing Analysis — Normalized tables (replaces S3-only storage)
-- ═══════════════════════════════════════════════════════════

CREATE TABLE drawing_layers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color INT,
    entity_count INT NOT NULL DEFAULT 0,
    linetype TEXT
);
CREATE INDEX idx_drawing_layers_drawing ON drawing_layers(drawing_id);

CREATE TABLE drawing_blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    entity_count INT NOT NULL DEFAULT 0,
    insert_count INT NOT NULL DEFAULT 0
);
CREATE INDEX idx_drawing_blocks_drawing ON drawing_blocks(drawing_id);

CREATE TABLE drawing_dimensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    dim_type TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    layer TEXT
);
CREATE INDEX idx_drawing_dims_drawing ON drawing_dimensions(drawing_id);

CREATE TABLE drawing_annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    layer TEXT,
    ann_type TEXT
);
CREATE INDEX idx_drawing_anns_drawing ON drawing_annotations(drawing_id);

-- Drawing-level analysis metadata
ALTER TABLE drawings ADD COLUMN total_layers INT;
ALTER TABLE drawings ADD COLUMN total_blocks INT;
ALTER TABLE drawings ADD COLUMN total_dimensions INT;
ALTER TABLE drawings ADD COLUMN total_annotations INT;
ALTER TABLE drawings ADD COLUMN detected_drawing_type TEXT;
ALTER TABLE drawings ADD COLUMN detected_language TEXT;
ALTER TABLE drawings ADD COLUMN insert_units_raw INT;
ALTER TABLE drawings ADD COLUMN dwg_version TEXT;

-- ═══════════════════════════════════════════════════════════
-- AI KSS Sessions + Research Items
-- ═══════════════════════════════════════════════════════════

CREATE TABLE ai_kss_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    job_id UUID REFERENCES jobs(id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'researching',
    research_model TEXT,
    generation_model TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX idx_ai_kss_sessions_drawing ON ai_kss_sessions(drawing_id);

CREATE TABLE ai_kss_research_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES ai_kss_sessions(id) ON DELETE CASCADE,
    sek_group TEXT NOT NULL,
    sek_code TEXT,
    description TEXT NOT NULL,
    unit TEXT,
    price_min_lv DOUBLE PRECISION,
    price_max_lv DOUBLE PRECISION,
    source_url TEXT,
    confidence DOUBLE PRECISION DEFAULT 0.5,
    reasoning TEXT,
    user_approved BOOLEAN NOT NULL DEFAULT true,
    user_edited BOOLEAN NOT NULL DEFAULT false,
    edited_description TEXT,
    edited_price_min_lv DOUBLE PRECISION,
    edited_price_max_lv DOUBLE PRECISION,
    edited_unit TEXT
);
CREATE INDEX idx_ai_research_items_session ON ai_kss_research_items(session_id);

-- ═══════════════════════════════════════════════════════════
-- KSS Reports — dual mode + normalized line items
-- ═══════════════════════════════════════════════════════════

ALTER TABLE kss_reports ADD COLUMN mode TEXT NOT NULL DEFAULT 'rule_based';
DROP INDEX IF EXISTS idx_kss_reports_drawing;
CREATE INDEX idx_kss_reports_drawing_mode ON kss_reports(drawing_id, mode);

CREATE TABLE kss_line_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id UUID NOT NULL REFERENCES kss_reports(id) ON DELETE CASCADE,
    section_number TEXT NOT NULL,
    section_title TEXT NOT NULL,
    item_no INT NOT NULL,
    sek_code TEXT NOT NULL,
    description TEXT NOT NULL,
    unit TEXT NOT NULL,
    quantity DOUBLE PRECISION NOT NULL,
    unit_price_lv DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_lv DOUBLE PRECISION NOT NULL DEFAULT 0,
    labor_price DOUBLE PRECISION DEFAULT 0,
    material_price DOUBLE PRECISION DEFAULT 0,
    mechanization_price DOUBLE PRECISION DEFAULT 0,
    overhead_price DOUBLE PRECISION DEFAULT 0,
    confidence DOUBLE PRECISION DEFAULT 0.5,
    reasoning TEXT,
    provenance TEXT DEFAULT 'rule_based',
    source_layer TEXT,
    source_block TEXT
);
CREATE INDEX idx_kss_items_report ON kss_line_items(report_id);
CREATE INDEX idx_kss_items_section ON kss_line_items(report_id, section_number);
