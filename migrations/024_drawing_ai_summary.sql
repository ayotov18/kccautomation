-- ═══════════════════════════════════════════════════════════
-- AI-generated bilingual drawing summary
--
-- Replaces the raw-stat widgets (entity distribution, annotation
-- chips, etc.) with a single comprehensive summary the operator can
-- redact before signing. EN and BG are independently editable so the
-- author keeps the bilingual integrity (no auto-translate on save).
-- ═══════════════════════════════════════════════════════════

ALTER TABLE drawings
    ADD COLUMN ai_summary_en TEXT,
    ADD COLUMN ai_summary_bg TEXT,
    ADD COLUMN ai_summary_generated_at TIMESTAMPTZ,
    ADD COLUMN ai_summary_edited_at TIMESTAMPTZ,
    ADD COLUMN ai_summary_model TEXT;
