use axum::{
    Json, Router,
    extract::{Extension, Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn validation_routes() -> Router<AppState> {
    Router::new()
        .route("/reports", get(list_reports))
        .route("/rules", get(list_rules))
}

// ── Models ──────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct ValidationReportRow {
    id: Uuid,
    target_type: String,
    target_id: Uuid,
    status: Option<String>,
    score: f64,
    results: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

// ── DTOs ────────────────────────────────────────────

#[derive(Deserialize)]
struct ListReportsQuery {
    target_type: Option<String>,
    target_id: Option<Uuid>,
}

#[derive(Serialize)]
struct RuleInfo {
    rule_id: &'static str,
    severity: &'static str,
    category: &'static str,
    description: &'static str,
}

// ── Handlers ────────────────────────────────────────

async fn list_reports(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Query(params): Query<ListReportsQuery>,
) -> Result<Json<Vec<ValidationReportRow>>, ApiError> {
    let reports = match (params.target_type.as_deref(), params.target_id) {
        (Some(tt), Some(tid)) => {
            sqlx::query_as::<_, ValidationReportRow>(
                "SELECT * FROM validation_reports WHERE target_type = $1 AND target_id = $2 ORDER BY created_at DESC LIMIT 50",
            )
            .bind(tt)
            .bind(tid)
            .fetch_all(&state.db)
            .await?
        }
        (Some(tt), None) => {
            sqlx::query_as::<_, ValidationReportRow>(
                "SELECT * FROM validation_reports WHERE target_type = $1 ORDER BY created_at DESC LIMIT 50",
            )
            .bind(tt)
            .fetch_all(&state.db)
            .await?
        }
        (None, Some(tid)) => {
            sqlx::query_as::<_, ValidationReportRow>(
                "SELECT * FROM validation_reports WHERE target_id = $1 ORDER BY created_at DESC LIMIT 50",
            )
            .bind(tid)
            .fetch_all(&state.db)
            .await?
        }
        (None, None) => {
            sqlx::query_as::<_, ValidationReportRow>(
                "SELECT * FROM validation_reports ORDER BY created_at DESC LIMIT 50",
            )
            .fetch_all(&state.db)
            .await?
        }
    };

    Ok(Json(reports))
}

async fn list_rules(
    Extension(_user_id): Extension<Uuid>,
) -> Result<Json<Vec<RuleInfo>>, ApiError> {
    let rules = vec![
        RuleInfo {
            rule_id: "position_has_quantity",
            severity: "error",
            category: "completeness",
            description: "Every position must have a quantity greater than 0",
        },
        RuleInfo {
            rule_id: "position_has_unit_rate",
            severity: "error",
            category: "completeness",
            description: "Every position must have a unit rate greater than 0",
        },
        RuleInfo {
            rule_id: "position_has_description",
            severity: "error",
            category: "completeness",
            description: "Every position must have a non-empty description",
        },
        RuleInfo {
            rule_id: "no_duplicate_ordinals",
            severity: "error",
            category: "structure",
            description: "No duplicate ordinals within a BOQ",
        },
        RuleInfo {
            rule_id: "unit_rate_in_range",
            severity: "warning",
            category: "quality",
            description: "Unit rate outlier detection (greater than 5x median triggers a warning)",
        },
        RuleInfo {
            rule_id: "no_negative_values",
            severity: "error",
            category: "consistency",
            description: "No negative values for quantity or unit rate",
        },
        RuleInfo {
            rule_id: "section_structure",
            severity: "warning",
            category: "structure",
            description: "Positions should have a section assignment",
        },
        RuleInfo {
            rule_id: "total_cost_benchmarks",
            severity: "warning",
            category: "quality",
            description: "No single position should exceed 50% of the total BOQ cost",
        },
    ];

    Ok(Json(rules))
}
