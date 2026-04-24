-- KSS Suggestions: track accept/reject lifecycle for AI-generated items
-- Items with confidence < 0.7 are "suggestions" requiring user review.

ALTER TABLE kss_line_items ADD COLUMN IF NOT EXISTS suggestion_status TEXT;
-- Values: NULL (main report item), 'pending' (needs review), 'accepted', 'rejected'

-- Fast lookup for suggestion queries (confidence < 0.7)
CREATE INDEX IF NOT EXISTS idx_kss_items_suggestions
  ON kss_line_items(report_id, confidence) WHERE confidence < 0.7;

-- Section ordering for add-item feature
CREATE INDEX IF NOT EXISTS idx_kss_items_section_order
  ON kss_line_items(report_id, section_number, item_no);
