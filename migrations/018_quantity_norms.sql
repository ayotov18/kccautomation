-- Phase 4+ cont'd: Quantity norms system — per-unit consumption tables,
-- project-type distributions, scrape history. Mirrors the pricing stack
-- structure (scrape_runs / scrape_source_runs / scraped_*_rows).

-- ── Per-unit consumption norms (like УСН) ────────────────────────────────
CREATE TABLE IF NOT EXISTS quantity_norms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sek_code TEXT NOT NULL,                -- e.g. "СЕК05.007"
    description_bg TEXT NOT NULL,
    work_unit TEXT NOT NULL,               -- "m²" / "m³" / "m" / "бр." / "кг"
    -- Labor consumption per 1 work_unit (hours)
    labor_qualified_h NUMERIC(10,4) NOT NULL DEFAULT 0,
    labor_helper_h NUMERIC(10,4) NOT NULL DEFAULT 0,
    labor_trade TEXT,                      -- "зидар" / "армировач" / "бояджия"
    -- Materials: [{ name, qty, unit, waste_pct }]
    materials JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- Machinery: [{ resource, hours }]
    machinery JSONB NOT NULL DEFAULT '[]'::jsonb,
    source TEXT NOT NULL,                  -- "УСН-05-007" / "Ytong-tech-sheet"
    source_url TEXT,
    confidence NUMERIC(3,2) NOT NULL DEFAULT 0.80,
    -- NULL user_id = global / seed dataset, visible to everyone
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (sek_code, source, user_id)
);

CREATE INDEX IF NOT EXISTS idx_quantity_norms_sek ON quantity_norms(sek_code);
CREATE INDEX IF NOT EXISTS idx_quantity_norms_user ON quantity_norms(user_id);
CREATE INDEX IF NOT EXISTS idx_quantity_norms_sek_group ON quantity_norms(substring(sek_code, 1, 5));

-- ── Typical project-level distributions (sanity heuristics) ──────────────
CREATE TABLE IF NOT EXISTS project_distributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_type TEXT NOT NULL,           -- "residential_apartment" / "bungalow" / "office" / "school" / "road"
    metric_key TEXT NOT NULL,              -- "concrete_m3_per_floor_m2"
    metric_label_bg TEXT NOT NULL,         -- "Бетон на m² подова площ"
    unit TEXT NOT NULL,                    -- "m³/m²"
    min_value NUMERIC(14,4),
    max_value NUMERIC(14,4),
    median_value NUMERIC(14,4) NOT NULL,
    sample_size INT NOT NULL DEFAULT 0,
    source TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (building_type, metric_key)
);

CREATE INDEX IF NOT EXISTS idx_distributions_type ON project_distributions(building_type);

-- ── Scrape runs history (mirrors scrape_runs) ───────────────────────────
CREATE TABLE IF NOT EXISTS scrape_quantity_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'running',
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    total_sources INT NOT NULL DEFAULT 0,
    successful_sources INT NOT NULL DEFAULT 0,
    failed_sources INT NOT NULL DEFAULT 0,
    norms_created INT NOT NULL DEFAULT 0,
    norms_updated INT NOT NULL DEFAULT 0,
    elapsed_ms INT,
    notes JSONB
);

CREATE INDEX IF NOT EXISTS idx_sqr_user ON scrape_quantity_runs(user_id, started_at DESC);

-- ── Sources config (mirrors scrape_sources) ─────────────────────────────
CREATE TABLE IF NOT EXISTS quantity_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site_name TEXT NOT NULL,               -- "ytong.bg"
    base_url TEXT NOT NULL,
    description TEXT,
    parser_template TEXT NOT NULL DEFAULT 'manual',  -- "ytong" / "wienerberger" / "manual"
    is_builtin BOOLEAN NOT NULL DEFAULT false,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_run_at TIMESTAMPTZ,
    last_success BOOLEAN,
    last_norms_count INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Seed: built-in sources ──────────────────────────────────────────────
INSERT INTO quantity_sources (site_name, base_url, description, parser_template, is_builtin)
VALUES
    ('ytong.bg', 'https://www.ytong.bg', 'Технически листове за газобетон', 'ytong', true),
    ('wienerberger.bg', 'https://www.wienerberger.bg', 'Технически листове за керамични тухли', 'wienerberger', true),
    ('baumit.bg', 'https://baumit.bg', 'Лепила, мазилки, фасадни системи', 'baumit', true),
    ('smr.sek-bg.com', 'https://smr.sek-bg.com', 'Стройексперт-СЕК норми (free items)', 'smr_sek', true),
    ('atspress.net', 'https://atspress.net/library/usn/', 'УСН електронна библиотека', 'usn_pdf', true)
ON CONFLICT DO NOTHING;

-- ── Seed: quantity_norms for the 8 top СЕК groups ───────────────────────
-- Values hand-curated from БГ industry research (УСН-equivalent).
-- All are global (user_id = NULL) so every user sees them on day 1.

INSERT INTO quantity_norms
    (sek_code, description_bg, work_unit, labor_qualified_h, labor_helper_h, labor_trade, materials, machinery, source, confidence)
VALUES
    -- СЕК01 Земни работи
    ('СЕК01.001', 'Отстраняване на хумус ръчно, дебелина до 10 см', 'm²',
     0.15, 0.00, 'общ работник',
     '[]'::jsonb,
     '[]'::jsonb,
     'УСН-01-001', 0.90),

    ('СЕК01.003', 'Механизиран изкоп за основи', 'm³',
     0.05, 0.02, 'общ работник',
     '[]'::jsonb,
     '[{"resource":"багер","hours":0.08}]'::jsonb,
     'УСН-01-003', 0.90),

    ('СЕК01.010', 'Превоз на земни маси до 10 км', 'm³',
     0.00, 0.00, NULL,
     '[]'::jsonb,
     '[{"resource":"самосвал","hours":0.12}]'::jsonb,
     'УСН-01-010', 0.85),

    -- СЕК02 Кофражни
    ('СЕК02.001', 'Кофраж за плочи', 'm²',
     0.80, 0.40, 'кофражист',
     '[{"name":"кофражни дъски","qty":1.05,"unit":"m²","waste_pct":5}]'::jsonb,
     '[]'::jsonb,
     'УСН-02-001', 0.88),

    ('СЕК02.002', 'Кофраж за колони/стени', 'm²',
     1.00, 0.50, 'кофражист',
     '[{"name":"кофражни платна","qty":1.1,"unit":"m²","waste_pct":10}]'::jsonb,
     '[]'::jsonb,
     'УСН-02-002', 0.88),

    -- СЕК03 Армировъчни
    ('СЕК03.001', 'Подготовка и монтаж армировка Ø6–Ø12', 'кг',
     0.025, 0.01, 'армировач',
     '[{"name":"арматура A-III Ø8","qty":1.04,"unit":"кг","waste_pct":4}]'::jsonb,
     '[]'::jsonb,
     'УСН-03-001', 0.90),

    ('СЕК03.002', 'Заварена армиран мрежа', 'кг',
     0.015, 0.005, 'армировач',
     '[{"name":"заварена мрежа Ø5","qty":1.03,"unit":"кг","waste_pct":3}]'::jsonb,
     '[]'::jsonb,
     'УСН-03-002', 0.88),

    -- СЕК04 Бетонови
    ('СЕК04.001', 'Доставка и полагане бетон B20', 'm³',
     0.35, 0.25, 'бетонджия',
     '[{"name":"готов бетон B20","qty":1.02,"unit":"m³","waste_pct":2}]'::jsonb,
     '[{"resource":"вибратор","hours":0.15}]'::jsonb,
     'УСН-04-001', 0.92),

    ('СЕК04.007', 'Бетон B25 за плочи/стени', 'm³',
     0.35, 0.25, 'бетонджия',
     '[{"name":"готов бетон B25","qty":1.02,"unit":"m³","waste_pct":2}]'::jsonb,
     '[{"resource":"вибратор","hours":0.15}]'::jsonb,
     'УСН-04-007', 0.92),

    -- СЕК05 Зидарски
    ('СЕК05.007', 'Газобетонна зидария 25 см', 'm²',
     0.55, 0.30, 'зидар',
     '[{"name":"газобетон блок 625×240×250","qty":6.67,"unit":"бр.","waste_pct":3},
       {"name":"лепило за газобетон","qty":3.5,"unit":"кг","waste_pct":5},
       {"name":"арм. мрежа","qty":0.35,"unit":"кг","waste_pct":5}]'::jsonb,
     '[{"resource":"бъркачка","hours":0.10}]'::jsonb,
     'Ytong-tech-sheet', 0.92),

    ('СЕК05.013', 'Тухлена зидария 25 см', 'm²',
     0.65, 0.35, 'зидар',
     '[{"name":"тухла 250×120×65","qty":72,"unit":"бр.","waste_pct":4},
       {"name":"варо-циментов разтвор","qty":0.055,"unit":"m³","waste_pct":5}]'::jsonb,
     '[{"resource":"бъркачка","hours":0.10}]'::jsonb,
     'Wienerberger-tech-sheet', 0.90),

    ('СЕК05.009', 'Газобетонна зидария 10 см (преградна)', 'm²',
     0.40, 0.20, 'зидар',
     '[{"name":"газобетон блок 625×100×250","qty":6.67,"unit":"бр.","waste_pct":3},
       {"name":"лепило за газобетон","qty":1.4,"unit":"кг","waste_pct":5}]'::jsonb,
     '[]'::jsonb,
     'Ytong-tech-sheet', 0.90),

    -- СЕК10 Мазачески
    ('СЕК10.001', 'Вътрешна варо-циментова мазилка', 'm²',
     0.35, 0.20, 'мазач',
     '[{"name":"варо-цим. разтвор","qty":0.022,"unit":"m³","waste_pct":10}]'::jsonb,
     '[]'::jsonb,
     'УСН-10-001', 0.88),

    ('СЕК10.006', 'Външна фасадна мазилка', 'm²',
     0.45, 0.25, 'мазач',
     '[{"name":"готова мазилна смес","qty":4.5,"unit":"кг","waste_pct":8}]'::jsonb,
     '[]'::jsonb,
     'Baumit-tech-sheet', 0.88),

    -- СЕК11 Настилки
    ('СЕК11.008', 'Настилка от гранитогрес 60×60', 'm²',
     0.45, 0.20, 'плочкаджия',
     '[{"name":"гранитогрес","qty":1.05,"unit":"m²","waste_pct":5},
       {"name":"лепило","qty":4.5,"unit":"кг","waste_pct":5},
       {"name":"фугиране","qty":0.35,"unit":"кг","waste_pct":5}]'::jsonb,
     '[]'::jsonb,
     'УСН-11-008', 0.88),

    ('СЕК11.013', 'Ламинат клас 32 с монтаж', 'm²',
     0.25, 0.10, 'настилач',
     '[{"name":"ламинат","qty":1.05,"unit":"m²","waste_pct":5},
       {"name":"мембрана","qty":1.00,"unit":"m²","waste_pct":3}]'::jsonb,
     '[]'::jsonb,
     'manufacturer', 0.85),

    -- СЕК13 Бояджийски
    ('СЕК13.007', 'Латексово боядисване двукратно', 'm²',
     0.12, 0.05, 'бояджия',
     '[{"name":"латекс бял","qty":0.18,"unit":"л","waste_pct":5},
       {"name":"грунд","qty":0.08,"unit":"л","waste_pct":5}]'::jsonb,
     '[]'::jsonb,
     'УСН-13-007', 0.88),

    -- СЕК14 Стоманени
    ('СЕК14.003', 'Алуминиева дограма', 'm²',
     1.20, 0.60, 'монтажник',
     '[{"name":"алуминиев профил","qty":1.0,"unit":"m²","waste_pct":3}]'::jsonb,
     '[]'::jsonb,
     'manufacturer', 0.80),

    -- СЕК15 Хидроизолации
    ('СЕК15.004', 'Хидроизолация битумна мембрана 4 мм', 'm²',
     0.18, 0.10, 'хидроизолатор',
     '[{"name":"битумна мембрана 4мм","qty":1.10,"unit":"m²","waste_pct":10},
       {"name":"битумен грунд","qty":0.30,"unit":"кг","waste_pct":5}]'::jsonb,
     '[{"resource":"пропан горелка","hours":0.08}]'::jsonb,
     'УСН-15-004', 0.88),

    -- СЕК16 Топлоизолации
    ('СЕК16.001', 'Топлоизолация с EPS 10 см (фасада)', 'm²',
     0.35, 0.20, 'топлоизолатор',
     '[{"name":"EPS 100 мм","qty":1.05,"unit":"m²","waste_pct":5},
       {"name":"лепило","qty":4.5,"unit":"кг","waste_pct":5},
       {"name":"дюбели","qty":6,"unit":"бр.","waste_pct":0},
       {"name":"армираща мрежа","qty":1.10,"unit":"m²","waste_pct":10}]'::jsonb,
     '[]'::jsonb,
     'Baumit-tech-sheet', 0.90),

    -- СЕК17 Столарски
    ('СЕК17.029', 'PVC 5-камерна дограма с монтаж', 'm²',
     1.10, 0.55, 'монтажник',
     '[{"name":"PVC профил 5-камерен","qty":1.0,"unit":"m²","waste_pct":2},
       {"name":"стъклопакет","qty":1.0,"unit":"m²","waste_pct":1}]'::jsonb,
     '[]'::jsonb,
     'manufacturer', 0.88),

    -- СЕК22 ВиК
    ('СЕК22.001', 'Вътрешна ВиК инсталация PPR тръба Ø20', 'm',
     0.15, 0.05, 'водопроводчик',
     '[{"name":"PPR тръба Ø20","qty":1.02,"unit":"m","waste_pct":2},
       {"name":"фитинги","qty":0.30,"unit":"бр.","waste_pct":5}]'::jsonb,
     '[]'::jsonb,
     'УСН-22-001', 0.85),

    -- СЕК34 Електро
    ('СЕК34.111', 'Кабел СВТ 3×1.5мм² в тръба', 'm',
     0.08, 0.04, 'електротехник',
     '[{"name":"кабел СВТ 3×1.5","qty":1.03,"unit":"m","waste_pct":3},
       {"name":"PVC тръба Ø16","qty":1.02,"unit":"m","waste_pct":2}]'::jsonb,
     '[]'::jsonb,
     'УСН-34-111', 0.85),

    ('СЕК34.311', 'Ключ / контакт шуко монтаж', 'бр.',
     0.20, 0.05, 'електротехник',
     '[{"name":"ключ / контакт","qty":1,"unit":"бр.","waste_pct":0},
       {"name":"разклонителна кутия","qty":1,"unit":"бр.","waste_pct":0}]'::jsonb,
     '[]'::jsonb,
     'УСН-34-311', 0.85)
ON CONFLICT (sek_code, source, user_id) DO NOTHING;

-- ── Seed: project-level distributions ────────────────────────────────────
INSERT INTO project_distributions
    (building_type, metric_key, metric_label_bg, unit, min_value, max_value, median_value, sample_size, source)
VALUES
    -- Residential apartment block
    ('residential_apartment', 'concrete_m3_per_floor_m2', 'Бетон / m² подова площ', 'm³/m²',
     0.08, 0.18, 0.12, 23, 'AOP tender corpus'),
    ('residential_apartment', 'rebar_kg_per_concrete_m3', 'Армировка / m³ бетон', 'кг/m³',
     80, 200, 140, 23, 'industry benchmark'),
    ('residential_apartment', 'formwork_m2_per_concrete_m3', 'Кофраж / m³ бетон', 'm²/m³',
     6, 12, 8.5, 18, 'industry benchmark'),
    ('residential_apartment', 'plaster_m2_per_floor_m2', 'Мазилка / m² подова площ', 'm²/m²',
     2.2, 3.6, 2.8, 15, 'AOP tender corpus'),
    ('residential_apartment', 'tile_m2_per_floor_m2', 'Плочки / m² подова площ', 'm²/m²',
     0.15, 0.35, 0.22, 12, 'AOP tender corpus'),
    ('residential_apartment', 'paint_m2_per_floor_m2', 'Боядисване / m² подова площ', 'm²/m²',
     1.8, 3.2, 2.4, 14, 'AOP tender corpus'),
    ('residential_apartment', 'dograma_m2_per_floor_m2', 'Дограма / m² подова площ', 'm²/m²',
     0.12, 0.22, 0.16, 11, 'AOP tender corpus'),
    ('residential_apartment', 'masonry_m2_per_floor_m2', 'Зидария (фасада + преградни) / m² подова площ', 'm²/m²',
     0.60, 1.20, 0.85, 14, 'AOP tender corpus'),

    -- Bungalow / detached house
    ('bungalow', 'concrete_m3_per_floor_m2', 'Бетон / m² подова площ', 'm³/m²',
     0.15, 0.28, 0.20, 8, 'real project corpus'),
    ('bungalow', 'rebar_kg_per_concrete_m3', 'Армировка / m³ бетон', 'кг/m³',
     70, 170, 120, 8, 'real project corpus'),
    ('bungalow', 'plaster_m2_per_floor_m2', 'Мазилка / m² подова площ', 'm²/m²',
     3.0, 4.2, 3.6, 6, 'real project corpus'),
    ('bungalow', 'dograma_m2_per_floor_m2', 'Дограма / m² подова площ', 'm²/m²',
     0.18, 0.28, 0.22, 7, 'real project corpus'),
    ('bungalow', 'masonry_m2_per_floor_m2', 'Зидария / m² подова площ', 'm²/m²',
     1.2, 1.8, 1.5, 8, 'real project corpus'),
    ('bungalow', 'roof_m2_per_floor_m2', 'Покрив / m² подова площ', 'm²/m²',
     1.10, 1.30, 1.20, 8, 'real project corpus'),

    -- Office
    ('office', 'concrete_m3_per_floor_m2', 'Бетон / m² подова площ', 'm³/m²',
     0.10, 0.20, 0.15, 5, 'industry benchmark'),
    ('office', 'dograma_m2_per_floor_m2', 'Дограма / m² подова площ', 'm²/m²',
     0.25, 0.45, 0.35, 4, 'industry benchmark'),
    ('office', 'paint_m2_per_floor_m2', 'Боядисване / m² подова площ', 'm²/m²',
     2.0, 3.4, 2.6, 4, 'industry benchmark'),
    ('office', 'cable_m_per_floor_m2', 'Кабел / m² подова площ', 'm/m²',
     3.5, 6.0, 4.5, 3, 'industry benchmark'),

    -- School / public
    ('school', 'concrete_m3_per_floor_m2', 'Бетон / m² подова площ', 'm³/m²',
     0.12, 0.22, 0.16, 4, 'AOP tender corpus'),
    ('school', 'plaster_m2_per_floor_m2', 'Мазилка / m² подова площ', 'm²/m²',
     2.4, 3.6, 3.0, 4, 'AOP tender corpus'),
    ('school', 'paint_m2_per_floor_m2', 'Боядисване / m² подова площ', 'm²/m²',
     2.2, 3.4, 2.8, 4, 'AOP tender corpus'),

    -- Road / infrastructure
    ('road', 'asphalt_t_per_m2', 'Асфалт (тегло) / m²', 'т/m²',
     0.20, 0.30, 0.25, 6, 'AOP road tenders'),
    ('road', 'aggregate_m3_per_m2', 'Трошен камък основа / m²', 'm³/m²',
     0.20, 0.35, 0.28, 6, 'AOP road tenders'),
    ('road', 'curb_m_per_road_m', 'Бордюри / линеен метър', 'm/m',
     0, 2, 2, 5, 'AOP road tenders')
ON CONFLICT (building_type, metric_key) DO NOTHING;
