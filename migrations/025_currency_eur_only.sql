-- ═══════════════════════════════════════════════════════════
-- Currency cleanup: BGN → EUR everywhere
--
-- The platform always priced in EUR. Columns and Rust types named
-- `*_lv` (short for лв = BGN) were a leftover from earlier ambition
-- to support both currencies; in practice every value already came
-- in as EUR. The mismatch surfaced as "85,472 лв" displayed in the UI
-- when the underlying numbers were really €85,472.
--
-- This migration renames every `_lv` column to `_eur` and removes the
-- redundant `currency` column from the user_price_corpus.
-- ═══════════════════════════════════════════════════════════

-- kss_reports
ALTER TABLE kss_reports RENAME COLUMN subtotal_lv TO subtotal_eur;
ALTER TABLE kss_reports RENAME COLUMN vat_lv TO vat_eur;
ALTER TABLE kss_reports RENAME COLUMN total_with_vat_lv TO total_with_vat_eur;
ALTER TABLE kss_reports RENAME COLUMN smr_subtotal_lv TO smr_subtotal_eur;
ALTER TABLE kss_reports RENAME COLUMN contingency_lv TO contingency_eur;
ALTER TABLE kss_reports RENAME COLUMN delivery_storage_lv TO delivery_storage_eur;
ALTER TABLE kss_reports RENAME COLUMN profit_lv TO profit_eur;
ALTER TABLE kss_reports RENAME COLUMN pre_vat_total_lv TO pre_vat_total_eur;
ALTER TABLE kss_reports RENAME COLUMN final_total_lv TO final_total_eur;

-- kss_line_items
ALTER TABLE kss_line_items RENAME COLUMN unit_price_lv TO unit_price_eur;
ALTER TABLE kss_line_items RENAME COLUMN total_lv TO total_eur;

-- user_price_corpus — drop redundant currency column.
ALTER TABLE user_price_corpus RENAME COLUMN material_price_lv TO material_price_eur;
ALTER TABLE user_price_corpus RENAME COLUMN labor_price_lv TO labor_price_eur;
ALTER TABLE user_price_corpus RENAME COLUMN total_unit_price_lv TO total_unit_price_eur;
ALTER TABLE user_price_corpus DROP COLUMN IF EXISTS currency;

-- ai_kss_research_items
ALTER TABLE ai_kss_research_items RENAME COLUMN price_min_lv TO price_min_eur;
ALTER TABLE ai_kss_research_items RENAME COLUMN price_max_lv TO price_max_eur;
ALTER TABLE ai_kss_research_items RENAME COLUMN edited_price_min_lv TO edited_price_min_eur;
ALTER TABLE ai_kss_research_items RENAME COLUMN edited_price_max_lv TO edited_price_max_eur;

-- drawings.kss_total_lv
ALTER TABLE drawings RENAME COLUMN kss_total_lv TO kss_total_eur;

-- scraped_price_rows had BOTH price_min_lv AND price_min_eur from the
-- legacy dual-storage design. Drop the _lv duplicates; _eur is canonical.
ALTER TABLE scraped_price_rows DROP COLUMN IF EXISTS price_min_lv;
ALTER TABLE scraped_price_rows DROP COLUMN IF EXISTS price_max_lv;
