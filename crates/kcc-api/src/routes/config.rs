use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    Json, Router,
    extract::{Extension, State},
    routing::get,
};
use kcc_core::kcc::config::KccConfig;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn config_routes() -> Router<AppState> {
    Router::new()
        .route("/thresholds", get(get_thresholds).put(update_thresholds))
        .route(
            "/pricing-defaults",
            get(get_pricing_defaults).put(put_pricing_defaults),
        )
}

async fn get_thresholds(State(_state): State<AppState>) -> Result<Json<KccConfig>, ApiError> {
    Ok(Json(KccConfig::default()))
}

async fn update_thresholds(
    State(_state): State<AppState>,
    Json(config): Json<KccConfig>,
) -> Result<Json<KccConfig>, ApiError> {
    Ok(Json(config))
}

// ── Pricing defaults ────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
#[serde(rename_all = "snake_case")]
struct PricingDefaultsRow {
    currency: String,
    vat_rate_pct: f64,
    dr_labor_pct: f64,
    dr_light_machinery_pct: f64,
    dr_heavy_machinery_pct: f64,
    dr_materials_pct: f64,
    contingency_pct: f64,
    profit_pct: f64,
    transport_slab_eur: f64,
    rate_mason_low: f64,
    rate_mason_high: f64,
    rate_formwork_low: f64,
    rate_formwork_high: f64,
    rate_rebar_low: f64,
    rate_rebar_high: f64,
    rate_painter_low: f64,
    rate_painter_high: f64,
    rate_electrician_low: f64,
    rate_electrician_high: f64,
    rate_plumber_low: f64,
    rate_plumber_high: f64,
    rate_welder_low: f64,
    rate_welder_high: f64,
    rate_helper_low: f64,
    rate_helper_high: f64,
    active_preset: Option<String>,
}

/// Shape returned to the frontend — labor rates nested into a struct that
/// matches the TypeScript `PricingDefaults` interface exactly.
#[derive(Serialize, Deserialize)]
struct LaborBand {
    low: f64,
    high: f64,
}

#[derive(Serialize, Deserialize)]
struct LaborRates {
    mason: LaborBand,
    formwork: LaborBand,
    rebar: LaborBand,
    painter: LaborBand,
    electrician: LaborBand,
    plumber: LaborBand,
    welder: LaborBand,
    helper: LaborBand,
}

#[derive(Serialize, Deserialize)]
struct PricingDefaultsDto {
    currency: String,
    vat_rate_pct: f64,
    dr_labor_pct: f64,
    dr_light_machinery_pct: f64,
    dr_heavy_machinery_pct: f64,
    dr_materials_pct: f64,
    contingency_pct: f64,
    profit_pct: f64,
    transport_slab_eur: f64,
    labor_rates: LaborRates,
    active_preset: Option<String>,
}

impl From<PricingDefaultsRow> for PricingDefaultsDto {
    fn from(r: PricingDefaultsRow) -> Self {
        PricingDefaultsDto {
            currency: r.currency,
            vat_rate_pct: r.vat_rate_pct,
            dr_labor_pct: r.dr_labor_pct,
            dr_light_machinery_pct: r.dr_light_machinery_pct,
            dr_heavy_machinery_pct: r.dr_heavy_machinery_pct,
            dr_materials_pct: r.dr_materials_pct,
            contingency_pct: r.contingency_pct,
            profit_pct: r.profit_pct,
            transport_slab_eur: r.transport_slab_eur,
            labor_rates: LaborRates {
                mason: LaborBand { low: r.rate_mason_low, high: r.rate_mason_high },
                formwork: LaborBand { low: r.rate_formwork_low, high: r.rate_formwork_high },
                rebar: LaborBand { low: r.rate_rebar_low, high: r.rate_rebar_high },
                painter: LaborBand { low: r.rate_painter_low, high: r.rate_painter_high },
                electrician: LaborBand { low: r.rate_electrician_low, high: r.rate_electrician_high },
                plumber: LaborBand { low: r.rate_plumber_low, high: r.rate_plumber_high },
                welder: LaborBand { low: r.rate_welder_low, high: r.rate_welder_high },
                helper: LaborBand { low: r.rate_helper_low, high: r.rate_helper_high },
            },
            active_preset: r.active_preset,
        }
    }
}

/// Default row when a user hasn't configured anything yet. Values match the
/// Bulgarian industry research (2026 EUR market).
fn default_dto() -> PricingDefaultsDto {
    PricingDefaultsDto {
        currency: "EUR".into(),
        vat_rate_pct: 20.0,
        dr_labor_pct: 110.0,
        dr_light_machinery_pct: 100.0,
        dr_heavy_machinery_pct: 30.0,
        dr_materials_pct: 12.0,
        contingency_pct: 10.0,
        profit_pct: 10.0,
        transport_slab_eur: 800.0,
        labor_rates: LaborRates {
            mason: LaborBand { low: 9.0, high: 14.0 },
            formwork: LaborBand { low: 9.0, high: 14.0 },
            rebar: LaborBand { low: 9.0, high: 15.0 },
            painter: LaborBand { low: 8.0, high: 13.0 },
            electrician: LaborBand { low: 10.0, high: 18.0 },
            plumber: LaborBand { low: 10.0, high: 18.0 },
            welder: LaborBand { low: 11.0, high: 20.0 },
            helper: LaborBand { low: 4.0, high: 7.0 },
        },
        active_preset: Some("public_tender".into()),
    }
}

async fn get_pricing_defaults(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<PricingDefaultsDto>, ApiError> {
    let row: Option<PricingDefaultsRow> = sqlx::query_as(
        "SELECT currency, vat_rate_pct::float8, \
                dr_labor_pct::float8, dr_light_machinery_pct::float8, \
                dr_heavy_machinery_pct::float8, dr_materials_pct::float8, \
                contingency_pct::float8, profit_pct::float8, transport_slab_eur::float8, \
                rate_mason_low::float8, rate_mason_high::float8, \
                rate_formwork_low::float8, rate_formwork_high::float8, \
                rate_rebar_low::float8, rate_rebar_high::float8, \
                rate_painter_low::float8, rate_painter_high::float8, \
                rate_electrician_low::float8, rate_electrician_high::float8, \
                rate_plumber_low::float8, rate_plumber_high::float8, \
                rate_welder_low::float8, rate_welder_high::float8, \
                rate_helper_low::float8, rate_helper_high::float8, \
                active_preset \
         FROM user_pricing_defaults WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    Ok(Json(row.map(Into::into).unwrap_or_else(default_dto)))
}

async fn put_pricing_defaults(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(dto): Json<PricingDefaultsDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query(
        "INSERT INTO user_pricing_defaults (
            user_id, currency, vat_rate_pct,
            dr_labor_pct, dr_light_machinery_pct, dr_heavy_machinery_pct, dr_materials_pct,
            contingency_pct, profit_pct, transport_slab_eur,
            rate_mason_low, rate_mason_high,
            rate_formwork_low, rate_formwork_high,
            rate_rebar_low, rate_rebar_high,
            rate_painter_low, rate_painter_high,
            rate_electrician_low, rate_electrician_high,
            rate_plumber_low, rate_plumber_high,
            rate_welder_low, rate_welder_high,
            rate_helper_low, rate_helper_high,
            active_preset, updated_at
        ) VALUES ($1,$2,$3, $4,$5,$6,$7, $8,$9,$10,
                  $11,$12, $13,$14, $15,$16, $17,$18,
                  $19,$20, $21,$22, $23,$24, $25,$26,
                  $27, NOW())
        ON CONFLICT (user_id) DO UPDATE SET
            currency = EXCLUDED.currency,
            vat_rate_pct = EXCLUDED.vat_rate_pct,
            dr_labor_pct = EXCLUDED.dr_labor_pct,
            dr_light_machinery_pct = EXCLUDED.dr_light_machinery_pct,
            dr_heavy_machinery_pct = EXCLUDED.dr_heavy_machinery_pct,
            dr_materials_pct = EXCLUDED.dr_materials_pct,
            contingency_pct = EXCLUDED.contingency_pct,
            profit_pct = EXCLUDED.profit_pct,
            transport_slab_eur = EXCLUDED.transport_slab_eur,
            rate_mason_low = EXCLUDED.rate_mason_low,
            rate_mason_high = EXCLUDED.rate_mason_high,
            rate_formwork_low = EXCLUDED.rate_formwork_low,
            rate_formwork_high = EXCLUDED.rate_formwork_high,
            rate_rebar_low = EXCLUDED.rate_rebar_low,
            rate_rebar_high = EXCLUDED.rate_rebar_high,
            rate_painter_low = EXCLUDED.rate_painter_low,
            rate_painter_high = EXCLUDED.rate_painter_high,
            rate_electrician_low = EXCLUDED.rate_electrician_low,
            rate_electrician_high = EXCLUDED.rate_electrician_high,
            rate_plumber_low = EXCLUDED.rate_plumber_low,
            rate_plumber_high = EXCLUDED.rate_plumber_high,
            rate_welder_low = EXCLUDED.rate_welder_low,
            rate_welder_high = EXCLUDED.rate_welder_high,
            rate_helper_low = EXCLUDED.rate_helper_low,
            rate_helper_high = EXCLUDED.rate_helper_high,
            active_preset = EXCLUDED.active_preset,
            updated_at = NOW()
        ",
    )
    .bind(user_id)
    .bind(&dto.currency)
    .bind(dto.vat_rate_pct)
    .bind(dto.dr_labor_pct)
    .bind(dto.dr_light_machinery_pct)
    .bind(dto.dr_heavy_machinery_pct)
    .bind(dto.dr_materials_pct)
    .bind(dto.contingency_pct)
    .bind(dto.profit_pct)
    .bind(dto.transport_slab_eur)
    .bind(dto.labor_rates.mason.low)
    .bind(dto.labor_rates.mason.high)
    .bind(dto.labor_rates.formwork.low)
    .bind(dto.labor_rates.formwork.high)
    .bind(dto.labor_rates.rebar.low)
    .bind(dto.labor_rates.rebar.high)
    .bind(dto.labor_rates.painter.low)
    .bind(dto.labor_rates.painter.high)
    .bind(dto.labor_rates.electrician.low)
    .bind(dto.labor_rates.electrician.high)
    .bind(dto.labor_rates.plumber.low)
    .bind(dto.labor_rates.plumber.high)
    .bind(dto.labor_rates.welder.low)
    .bind(dto.labor_rates.welder.high)
    .bind(dto.labor_rates.helper.low)
    .bind(dto.labor_rates.helper.high)
    .bind(&dto.active_preset)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"ok": true})))
}
