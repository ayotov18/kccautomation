-- Research's "1-10-100" rule: fixing a money-total discrepancy after a bid is
-- the most expensive class of bug. Both audit runs showed the UI displaying
-- "ОБЩО ЗА ОБЕКТА 65,685 €" while the DB held "total 59,714 €" — the markups
-- (contingency / delivery-storage / profit) were applied in the rendering
-- layer but never persisted alongside the VAT-inclusive total. Any screen
-- reading the DB saw one number, any screen computing on-the-fly saw another.
--
-- This migration adds the full cost ladder as explicit columns so every UI
-- and every audit reads the same authoritative numbers.

ALTER TABLE kss_reports
    ADD COLUMN IF NOT EXISTS smr_subtotal_lv          DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS contingency_lv           DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS delivery_storage_lv      DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS profit_lv                DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS pre_vat_total_lv         DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS final_total_lv           DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS totals_formula_version   TEXT DEFAULT 'v1';

-- Backfill existing reports using the legacy 2-column totals so downstream
-- queries never see NULLs. Markups default to 0 when we don't know them;
-- the final_total_lv then equals the current total_with_vat_lv.
UPDATE kss_reports
   SET smr_subtotal_lv     = COALESCE(smr_subtotal_lv, subtotal_lv),
       contingency_lv      = COALESCE(contingency_lv, 0),
       delivery_storage_lv = COALESCE(delivery_storage_lv, 0),
       profit_lv           = COALESCE(profit_lv, 0),
       pre_vat_total_lv    = COALESCE(pre_vat_total_lv, subtotal_lv),
       final_total_lv      = COALESCE(final_total_lv, total_with_vat_lv)
 WHERE smr_subtotal_lv IS NULL OR final_total_lv IS NULL;
