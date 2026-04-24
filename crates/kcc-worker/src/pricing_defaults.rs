//! Per-user pricing defaults loaded from `user_pricing_defaults`.
//!
//! Used by the AI KSS pipeline to inject the user's configured ДР, profit,
//! contingency, transport slab and labor-rate bands into both the Perplexity
//! research prompt and the Opus generation prompt. The same values also drive
//! the final overhead stack so the user sees what they configured.

use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LaborBand {
    pub low: f64,
    pub high: f64,
}

impl LaborBand {
    pub fn format(&self, unit: &str) -> String {
        format!("{:.1}-{:.1} {}", self.low, self.high, unit)
    }
}

#[derive(Debug, Clone)]
pub struct PricingDefaults {
    pub currency: String, // "EUR" or "BGN"
    pub vat_rate_pct: f64,

    pub dr_labor_pct: f64,
    pub dr_light_machinery_pct: f64,
    pub dr_heavy_machinery_pct: f64,
    pub dr_materials_pct: f64,

    pub contingency_pct: f64,
    pub profit_pct: f64,
    pub transport_slab_eur: f64,

    pub rate_mason: LaborBand,
    pub rate_formwork: LaborBand,
    pub rate_rebar: LaborBand,
    pub rate_painter: LaborBand,
    pub rate_electrician: LaborBand,
    pub rate_plumber: LaborBand,
    pub rate_welder: LaborBand,
    pub rate_helper: LaborBand,
}

impl PricingDefaults {
    /// Canonical defaults sourced from 2026 Bulgarian industry research.
    /// Used when the user has not yet configured anything.
    pub fn canonical() -> Self {
        Self {
            currency: "EUR".into(),
            vat_rate_pct: 20.0,
            dr_labor_pct: 110.0,
            dr_light_machinery_pct: 100.0,
            dr_heavy_machinery_pct: 30.0,
            dr_materials_pct: 12.0,
            contingency_pct: 10.0,
            profit_pct: 10.0,
            transport_slab_eur: 800.0,
            rate_mason: LaborBand { low: 9.0, high: 14.0 },
            rate_formwork: LaborBand { low: 9.0, high: 14.0 },
            rate_rebar: LaborBand { low: 9.0, high: 15.0 },
            rate_painter: LaborBand { low: 8.0, high: 13.0 },
            rate_electrician: LaborBand { low: 10.0, high: 18.0 },
            rate_plumber: LaborBand { low: 10.0, high: 18.0 },
            rate_welder: LaborBand { low: 11.0, high: 20.0 },
            rate_helper: LaborBand { low: 4.0, high: 7.0 },
        }
    }

    /// Load the user's row if any; fall back to canonical defaults.
    /// Uses raw sqlx::query with try_get to avoid tuple-arity limits on FromRow.
    pub async fn load_for_user(db: &PgPool, user_id: Uuid) -> Self {
        use sqlx::Row;
        let row_opt = sqlx::query(
            "SELECT currency,
                    vat_rate_pct::float8 AS vat_rate_pct,
                    dr_labor_pct::float8 AS dr_labor_pct,
                    dr_light_machinery_pct::float8 AS dr_light_machinery_pct,
                    dr_heavy_machinery_pct::float8 AS dr_heavy_machinery_pct,
                    dr_materials_pct::float8 AS dr_materials_pct,
                    contingency_pct::float8 AS contingency_pct,
                    profit_pct::float8 AS profit_pct,
                    transport_slab_eur::float8 AS transport_slab_eur,
                    rate_mason_low::float8 AS rate_mason_low,
                    rate_mason_high::float8 AS rate_mason_high,
                    rate_formwork_low::float8 AS rate_formwork_low,
                    rate_formwork_high::float8 AS rate_formwork_high,
                    rate_rebar_low::float8 AS rate_rebar_low,
                    rate_rebar_high::float8 AS rate_rebar_high,
                    rate_painter_low::float8 AS rate_painter_low,
                    rate_painter_high::float8 AS rate_painter_high,
                    rate_electrician_low::float8 AS rate_electrician_low,
                    rate_electrician_high::float8 AS rate_electrician_high,
                    rate_plumber_low::float8 AS rate_plumber_low,
                    rate_plumber_high::float8 AS rate_plumber_high,
                    rate_welder_low::float8 AS rate_welder_low,
                    rate_welder_high::float8 AS rate_welder_high,
                    rate_helper_low::float8 AS rate_helper_low,
                    rate_helper_high::float8 AS rate_helper_high
             FROM user_pricing_defaults WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

        let Some(row) = row_opt else { return Self::canonical(); };
        let get = |k: &str| row.try_get::<f64, _>(k).unwrap_or(0.0);
        Self {
            currency: row.try_get::<String, _>("currency").unwrap_or_else(|_| "EUR".into()),
            vat_rate_pct: get("vat_rate_pct"),
            dr_labor_pct: get("dr_labor_pct"),
            dr_light_machinery_pct: get("dr_light_machinery_pct"),
            dr_heavy_machinery_pct: get("dr_heavy_machinery_pct"),
            dr_materials_pct: get("dr_materials_pct"),
            contingency_pct: get("contingency_pct"),
            profit_pct: get("profit_pct"),
            transport_slab_eur: get("transport_slab_eur"),
            rate_mason: LaborBand { low: get("rate_mason_low"), high: get("rate_mason_high") },
            rate_formwork: LaborBand { low: get("rate_formwork_low"), high: get("rate_formwork_high") },
            rate_rebar: LaborBand { low: get("rate_rebar_low"), high: get("rate_rebar_high") },
            rate_painter: LaborBand { low: get("rate_painter_low"), high: get("rate_painter_high") },
            rate_electrician: LaborBand { low: get("rate_electrician_low"), high: get("rate_electrician_high") },
            rate_plumber: LaborBand { low: get("rate_plumber_low"), high: get("rate_plumber_high") },
            rate_welder: LaborBand { low: get("rate_welder_low"), high: get("rate_welder_high") },
            rate_helper: LaborBand { low: get("rate_helper_low"), high: get("rate_helper_high") },
        }
    }

    pub fn currency_symbol(&self) -> &'static str {
        if self.currency == "EUR" { "€" } else { "лв" }
    }

    /// Build the Bulgarian labor-rate anchor block for injection into the AI
    /// prompt. Gives the model concrete hourly rates to reason from.
    pub fn labor_anchors_bg(&self) -> String {
        let unit = format!("{}/час", self.currency_symbol());
        format!(
            "Ставки на труда за 2026 ({}):\n\
             - Зидар: {}\n\
             - Кофражист: {}\n\
             - Армировач: {}\n\
             - Бояджия: {}\n\
             - Електротехник: {}\n\
             - Водопроводчик: {}\n\
             - Заварчик: {}\n\
             - Помощник: {}",
            unit,
            self.rate_mason.format(&unit),
            self.rate_formwork.format(&unit),
            self.rate_rebar.format(&unit),
            self.rate_painter.format(&unit),
            self.rate_electrician.format(&unit),
            self.rate_plumber.format(&unit),
            self.rate_welder.format(&unit),
            self.rate_helper.format(&unit),
        )
    }
}
