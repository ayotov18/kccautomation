-- ═══════════════════════════════════════════════════════════
-- Link XLSX imports to drawings
--
-- A user testing the system uploads a known-good human KSS report
-- per drawing and expects RAG generation for that drawing to be
-- 1:1 with the reference offer. Without an explicit link, the worker
-- searches the whole user corpus and pairs sheets to modules by size,
-- which is fine when there's a single offer but ambiguous when there
-- are several.
--
-- Adding `drawing_id` lets the user pin an offer to a drawing. The
-- worker prefers linked imports when they exist; falls back to the
-- whole user corpus otherwise.
-- ═══════════════════════════════════════════════════════════

ALTER TABLE user_price_imports
    ADD COLUMN drawing_id UUID REFERENCES drawings(id) ON DELETE SET NULL;

CREATE INDEX idx_user_price_imports_drawing
    ON user_price_imports(user_id, drawing_id)
    WHERE drawing_id IS NOT NULL;
