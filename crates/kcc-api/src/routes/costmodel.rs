use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn costmodel_routes() -> Router<AppState> {
    Router::new()
        .route("/{project_id}/snapshots", get(list_evm_snapshots).post(create_evm_snapshot))
        .route("/{project_id}/evm", get(calculate_evm))
}

// ── Models ──────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct EvmSnapshotRow {
    id: Uuid,
    project_id: Uuid,
    period: String,
    bcws: f64,
    bcwp: f64,
    acwp: f64,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

// ── DTOs ────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateEvmSnapshotRequest {
    period: String,
    bcws: f64,
    bcwp: f64,
    acwp: f64,
    notes: Option<String>,
}

// ── Handlers ────────────────────────────────────────

async fn list_evm_snapshots(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<EvmSnapshotRow>>, ApiError> {
    verify_project_owner(&state, project_id, user_id).await?;

    let snapshots = sqlx::query_as::<_, EvmSnapshotRow>(
        "SELECT * FROM evm_snapshots WHERE project_id = $1 ORDER BY period DESC",
    )
    .bind(project_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(snapshots))
}

async fn create_evm_snapshot(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<CreateEvmSnapshotRequest>,
) -> Result<Json<EvmSnapshotRow>, ApiError> {
    verify_project_owner(&state, project_id, user_id).await?;

    let snapshot = sqlx::query_as::<_, EvmSnapshotRow>(
        r#"INSERT INTO evm_snapshots (id, project_id, period, bcws, bcwp, acwp, notes, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(project_id)
    .bind(&body.period)
    .bind(body.bcws)
    .bind(body.bcwp)
    .bind(body.acwp)
    .bind(&body.notes)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(snapshot))
}

async fn calculate_evm(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    verify_project_owner(&state, project_id, user_id).await?;

    // Get the latest snapshot
    let latest = sqlx::query_as::<_, EvmSnapshotRow>(
        "SELECT * FROM evm_snapshots WHERE project_id = $1 ORDER BY period DESC LIMIT 1",
    )
    .bind(project_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("No EVM snapshots found for this project".into()))?;

    // Get BAC from project budget
    let bac: f64 = sqlx::query_scalar(
        "SELECT COALESCE(budget, 0) FROM projects WHERE id = $1",
    )
    .bind(project_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0.0);

    if bac <= 0.0 {
        return Err(ApiError::BadRequest(
            "Project budget (BAC) must be set and greater than 0".into(),
        ));
    }

    let snapshot = erp_core::evm::EvmSnapshot {
        period: latest.period.clone(),
        bcws: latest.bcws,
        bcwp: latest.bcwp,
        acwp: latest.acwp,
    };

    let metrics = erp_core::evm::calculate_evm(bac, &snapshot);

    Ok(Json(serde_json::json!({
        "project_id": project_id,
        "period": latest.period,
        "bac": bac,
        "snapshot": {
            "bcws": latest.bcws,
            "bcwp": latest.bcwp,
            "acwp": latest.acwp,
        },
        "metrics": metrics,
    })))
}

// ── Helpers ─────────────────────────────────────────

async fn verify_project_owner(state: &AppState, project_id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
    let exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM projects WHERE id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    if exists.is_none() {
        return Err(ApiError::NotFound("Project not found".into()));
    }

    Ok(())
}
