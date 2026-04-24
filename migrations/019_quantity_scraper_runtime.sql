-- Per-source-URL scrape tracking for quantity scraper.
-- Mirrors scrape_source_runs (migration 006) but for the quantity pipeline.
CREATE TABLE IF NOT EXISTS scrape_quantity_source_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scrape_quantity_run_id UUID NOT NULL REFERENCES scrape_quantity_runs(id) ON DELETE CASCADE,
    site TEXT NOT NULL,
    url TEXT NOT NULL,
    category_hint TEXT,
    fetch_status TEXT NOT NULL DEFAULT 'pending',
    parse_status TEXT NOT NULL DEFAULT 'pending',
    db_status TEXT NOT NULL DEFAULT 'pending',
    http_status INT,
    elapsed_ms INT,
    html_len INT,
    parsed_count INT DEFAULT 0,
    error_message TEXT,
    fetched_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_sqsr_run ON scrape_quantity_source_runs(scrape_quantity_run_id);

-- Link the run row back to the enqueuing job, same as scrape_runs.job_id.
ALTER TABLE scrape_quantity_runs
    ADD COLUMN IF NOT EXISTS job_id UUID REFERENCES jobs(id) ON DELETE SET NULL;

-- Round-2 HIGH-value sources we just confirmed. parser_template is the
-- dispatch key used by the worker's builtin_parsers() registry.
INSERT INTO quantity_sources (site_name, base_url, description, parser_template, is_builtin)
VALUES
    ('wienerberger.bg',  'https://www.wienerberger.bg',                                     'Porotherm ceramic blocks — brick-per-m² + mortar dosage',     'wienerberger', true),
    ('ceresit.bg',       'https://www.ceresit.bg',                                          'Tile adhesives, grouts, plasters (kg/m²)',                    'ceresit',      true),
    ('globus-bg.com',    'https://www.globus-bg.com',                                       'Mortar dosage sheets',                                         'globus',       true),
    ('bgr.sika.com',     'https://bgr.sika.com',                                            '~200 TDS PDFs: mortars, waterproofing, floors, ETICS',        'sika',         true),
    ('mapei.bg',         'https://www.mapei.com/bg/bg',                                     'Tile adhesives, grouts, FRP, self-leveling (kg/m²)',          'mapei',        true),
    ('weber.bg',         'https://www.bg.weber',                                            'Mortars, plasters, floor screeds (kg/m²)',                    'weber',        true),
    ('fibran.bg',        'https://fibran.bg',                                               'ETICS handbook — full system consumption',                    'fibran',       true),
    ('knauf.bg',         'https://knauf.com/bg-BG/tools/download-center',                   'Drywall / ceilings / fire protection / ETICS (~500 docs)',    'knauf',        true),
    ('rail-infra.bg',    'https://www.rail-infra.bg',                                       'НКЖИ railway-infrastructure tender КСС archive',              'procurement_xls', true),
    ('api.government.bg','https://www.api.government.bg',                                   'АПИ road-infrastructure norms & tender КСС',                  'api_road',     true),
    ('op.plovdiv.bg',    'http://op.plovdiv.bg',                                            'Plovdiv buyer-profile legacy КСС Excel archive',              'procurement_xls', true),
    ('aop.bg',           'https://www.aop.bg',                                              'ЦАИС ЕОП legacy document viewer (case2.php?doc_id=)',         'procurement_xls', true)
ON CONFLICT DO NOTHING;
