-- KSS report storage for frontend display + editing
CREATE TABLE kss_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ai_enhanced BOOLEAN NOT NULL DEFAULT false,
    report_data JSONB NOT NULL,
    subtotal_lv DOUBLE PRECISION,
    vat_lv DOUBLE PRECISION,
    total_with_vat_lv DOUBLE PRECISION,
    item_count INT,
    s3_key_excel TEXT,
    s3_key_pdf TEXT
);

CREATE UNIQUE INDEX idx_kss_reports_drawing ON kss_reports(drawing_id);

-- Quick lookup for KSS status on drawings list
ALTER TABLE drawings ADD COLUMN kss_generated BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE drawings ADD COLUMN kss_total_lv DOUBLE PRECISION;
