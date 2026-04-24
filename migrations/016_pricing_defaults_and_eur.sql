-- Phase 0+1: EUR migration + per-user pricing defaults.
--
-- Bulgaria switched to the euro on 2026-01-01 at the locked rate
-- 1 EUR = 1.95583 BGN. We keep monetary amounts as numbers and tag them with
-- a currency code so old BGN reports stay valid while new ones are EUR.

-- ── 1. Currency column on every monetary table ──────────────────────────

ALTER TABLE kss_reports
    ADD COLUMN IF NOT EXISTS currency TEXT NOT NULL DEFAULT 'EUR';

ALTER TABLE kss_line_items
    ADD COLUMN IF NOT EXISTS currency TEXT NOT NULL DEFAULT 'EUR';

ALTER TABLE scraped_price_rows
    ADD COLUMN IF NOT EXISTS currency TEXT;  -- NULL = inherit from row semantics

ALTER TABLE price_lists
    ADD COLUMN IF NOT EXISTS currency TEXT NOT NULL DEFAULT 'EUR';

-- Older rows created before the cutover stay in BGN until re-generated.
UPDATE kss_reports
SET currency = 'BGN'
WHERE generated_at < '2026-01-01' AND currency = 'EUR';

UPDATE kss_line_items
SET currency = 'BGN'
FROM kss_reports r
WHERE kss_line_items.report_id = r.id
  AND r.generated_at < '2026-01-01'
  AND kss_line_items.currency = 'EUR';

-- ── 2. Per-user pricing defaults ────────────────────────────────────────

CREATE TABLE IF NOT EXISTS user_pricing_defaults (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,

    -- Currency + VAT
    currency TEXT NOT NULL DEFAULT 'EUR',
    vat_rate_pct NUMERIC(5,2) NOT NULL DEFAULT 20.00,

    -- ДР (допълнителни разходи) — applied per cost-factor
    dr_labor_pct NUMERIC(6,2) NOT NULL DEFAULT 110.00,
    dr_light_machinery_pct NUMERIC(6,2) NOT NULL DEFAULT 100.00,
    dr_heavy_machinery_pct NUMERIC(6,2) NOT NULL DEFAULT 30.00,
    dr_materials_pct NUMERIC(6,2) NOT NULL DEFAULT 12.00,

    -- Надбавки
    contingency_pct NUMERIC(5,2) NOT NULL DEFAULT 10.00,
    profit_pct NUMERIC(5,2) NOT NULL DEFAULT 10.00,
    transport_slab_eur NUMERIC(10,2) NOT NULL DEFAULT 800.00,

    -- Ставки на труда (EUR/hour) — ranges by trade, used as anchors by the AI prompt
    rate_mason_low NUMERIC(6,2) NOT NULL DEFAULT 9.00,
    rate_mason_high NUMERIC(6,2) NOT NULL DEFAULT 14.00,
    rate_formwork_low NUMERIC(6,2) NOT NULL DEFAULT 9.00,
    rate_formwork_high NUMERIC(6,2) NOT NULL DEFAULT 14.00,
    rate_rebar_low NUMERIC(6,2) NOT NULL DEFAULT 9.00,
    rate_rebar_high NUMERIC(6,2) NOT NULL DEFAULT 15.00,
    rate_painter_low NUMERIC(6,2) NOT NULL DEFAULT 8.00,
    rate_painter_high NUMERIC(6,2) NOT NULL DEFAULT 13.00,
    rate_electrician_low NUMERIC(6,2) NOT NULL DEFAULT 10.00,
    rate_electrician_high NUMERIC(6,2) NOT NULL DEFAULT 18.00,
    rate_plumber_low NUMERIC(6,2) NOT NULL DEFAULT 10.00,
    rate_plumber_high NUMERIC(6,2) NOT NULL DEFAULT 18.00,
    rate_welder_low NUMERIC(6,2) NOT NULL DEFAULT 11.00,
    rate_welder_high NUMERIC(6,2) NOT NULL DEFAULT 20.00,
    rate_helper_low NUMERIC(6,2) NOT NULL DEFAULT 4.00,
    rate_helper_high NUMERIC(6,2) NOT NULL DEFAULT 7.00,

    -- Which preset the user last applied (display-only, not a constraint)
    active_preset TEXT DEFAULT 'public_tender',

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE user_pricing_defaults IS
    'Per-user defaults injected into the AI price-research prompt. '
    'Percentages derived from Bulgarian industry research (СЕК, УСН). '
    'Labor-rate bands are EUR/hour for 2026 market.';
