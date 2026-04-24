-- Extend price_lists with source tracking and default flag
ALTER TABLE price_lists ADD COLUMN source TEXT NOT NULL DEFAULT 'upload';
ALTER TABLE price_lists ADD COLUMN scrape_metadata JSONB;
ALTER TABLE price_lists ADD COLUMN is_default BOOLEAN NOT NULL DEFAULT false;

-- Per-user configured scrape sources (built-in sites + custom URLs)
CREATE TABLE scrape_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    site_name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    is_builtin BOOLEAN NOT NULL DEFAULT false,
    category_urls JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_scrape_sources_user ON scrape_sources(user_id);

-- Scraped price items with per-user provenance
CREATE TABLE scraped_prices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    source_id UUID REFERENCES scrape_sources(id) ON DELETE SET NULL,
    source_site TEXT NOT NULL,
    source_url TEXT NOT NULL,
    sek_code TEXT,
    description_bg TEXT NOT NULL,
    unit TEXT NOT NULL,
    price_min DOUBLE PRECISION,
    price_max DOUBLE PRECISION,
    price_avg DOUBLE PRECISION,
    currency TEXT NOT NULL DEFAULT 'EUR',
    category TEXT,
    scraped_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_scraped_user ON scraped_prices(user_id);
CREATE INDEX idx_scraped_sek ON scraped_prices(sek_code);
CREATE INDEX idx_scraped_date ON scraped_prices(scraped_at);

-- System-wide SEK code mapping table (Bulgarian keyword patterns -> SEK codes)
CREATE TABLE sek_code_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern TEXT NOT NULL,
    sek_code TEXT NOT NULL,
    sek_group TEXT NOT NULL,
    description_bg TEXT NOT NULL,
    unit TEXT NOT NULL,
    confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0
);

-- Seed common SEK mappings
INSERT INTO sek_code_mappings (pattern, sek_code, sek_group, description_bg, unit, confidence) VALUES
('тухлена зидария', 'СЕК05.002', 'СЕК05', 'Тухлена зидария', 'М2', 1.0),
('газобетон', 'СЕК05.050', 'СЕК05', 'Зидария от газобетонни блокчета', 'М2', 1.0),
('латексово боядисване', 'СЕК13.030', 'СЕК13', 'Латексово боядисване по стени и тавани', 'М2', 1.0),
('грундиране', 'СЕК13.025', 'СЕК13', 'Грундиране с готов грунд върху мазилка', 'М2', 1.0),
('циментова замазка', 'СЕК11.020', 'СЕК11', 'Циментова замазка', 'М2', 1.0),
('гипсова мазилка', 'СЕК10.053', 'СЕК10', 'Гипсова машинно полагана мазилка', 'М2', 1.0),
('вътрешна мазилка', 'СЕК10.011', 'СЕК10', 'Вътрешна варова мазилка по стени', 'М2', 1.0),
('хидроизолация', 'СЕК15.010', 'СЕК15', 'Хидроизолация с битумна мембрана', 'М2', 1.0),
('топлоизолация EPS', 'СЕК16.020', 'СЕК16', 'Топлоизолация с EPS по стени', 'М2', 1.0),
('гипсокартон', 'СЕК20.010', 'СЕК20', 'Монтаж на гипсокартон по метална конструкция', 'М2', 1.0),
('ламиниран паркет', 'СЕК11.040', 'СЕК11', 'Доставка и монтаж на ламиниран паркет', 'М2', 1.0),
('PVC дограма', 'СЕК17.030', 'СЕК17', 'Доставка и монтаж на PVC дограма', 'бр.', 1.0),
('интериорна врата', 'СЕК17.020', 'СЕК17', 'Доставка и монтаж на МДФ интериорна врата', 'бр.', 1.0),
('тоалетна моноблок', 'СЕК22.050', 'СЕК22', 'Доставка и монтаж на тоалетна моноблок', 'бр.', 1.0),
('мивка', 'СЕК22.055', 'СЕК22', 'Доставка и монтаж на мивка', 'бр.', 1.0),
('вана', 'СЕК22.060', 'СЕК22', 'Доставка и монтаж на вана', 'бр.', 1.0),
('бетон фундамент', 'СЕК04.068', 'СЕК04', 'Доставка и полагане на армиран бетон за основи', 'М3', 1.0),
('кофраж плочи', 'СЕК02.010', 'СЕК02', 'Кофраж за стоманобетонни плочи', 'М2', 1.0),
('армировка', 'СЕК03.012', 'СЕК03', 'Изработка и монтаж на армировка', 'кг', 1.0);
