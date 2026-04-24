use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::{get, post, put},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn boq_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_boq))
        .route("/{boq_id}", get(get_boq).put(update_boq).delete(delete_boq))
        .route("/{boq_id}/positions", post(create_position))
        .route("/positions/{position_id}", put(update_position).delete(delete_position))
        .route("/{boq_id}/markups", get(get_markups).post(create_markup))
        .route("/markups/{markup_id}", put(update_markup).delete(delete_markup))
        .route("/{boq_id}/markups/apply-defaults", post(apply_default_markups))
        .route("/{boq_id}/grand-total", get(grand_total))
        .route("/{boq_id}/snapshots", post(create_snapshot).get(list_snapshots))
        .route("/{boq_id}/snapshots/{snap_id}/restore", post(restore_snapshot))
        .route("/{boq_id}/validate", post(validate_boq))
        .route("/{boq_id}/activity-log", get(activity_log))
}

// ── DTOs ────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateBoqRequest {
    project_id: Uuid,
    name: String,
}

#[derive(Deserialize)]
struct UpdateBoqRequest {
    name: Option<String>,
    description: Option<String>,
    currency: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct ApplyDefaultsRequest {
    region: String,
}

#[derive(Deserialize)]
struct CreateSnapshotRequest {
    name: String,
}

// ── Handlers ────────────────────────────────────────

async fn create_boq(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<CreateBoqRequest>,
) -> Result<Json<erp_boq::models::Boq>, ApiError> {
    // Verify user owns the project
    verify_project_owner(&state, body.project_id, user_id).await?;

    let boq = erp_boq::service::create_boq(&state.db, body.project_id, &body.name, user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(boq))
}

async fn get_boq(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
) -> Result<Json<erp_boq::models::BoqWithPositions>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let boq = erp_boq::service::get_boq(&state.db, boq_id)
        .await
        .map_err(|e| match e {
            erp_boq::service::BoqError::NotFound(_) => ApiError::NotFound("BOQ not found".into()),
            other => ApiError::Internal(other.to_string()),
        })?;

    Ok(Json(boq))
}

async fn update_boq(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
    Json(body): Json<UpdateBoqRequest>,
) -> Result<Json<erp_boq::models::Boq>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let current = sqlx::query_as::<_, erp_boq::models::Boq>(
        "SELECT * FROM boqs WHERE id = $1",
    )
    .bind(boq_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("BOQ not found".into()))?;

    let name = body.name.unwrap_or(current.name);
    let description = body.description.or(current.description);
    let currency = body.currency.unwrap_or(current.currency);
    let status = body.status.unwrap_or(current.status);

    let boq = sqlx::query_as::<_, erp_boq::models::Boq>(
        r#"UPDATE boqs SET name = $1, description = $2, currency = $3, status = $4, updated_at = now()
           WHERE id = $5 RETURNING *"#,
    )
    .bind(&name)
    .bind(&description)
    .bind(&currency)
    .bind(&status)
    .bind(boq_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(boq))
}

async fn delete_boq(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let result = sqlx::query("DELETE FROM boqs WHERE id = $1")
        .bind(boq_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("BOQ not found".into()));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn create_position(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
    Json(body): Json<erp_boq::models::CreatePosition>,
) -> Result<Json<erp_boq::models::Position>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let position = erp_boq::service::create_position(&state.db, boq_id, body)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    erp_boq::service::log_activity(
        &state.db, boq_id, user_id, "create_position",
        Some("position"), Some(position.id), "Created position", None,
    ).await.ok();

    Ok(Json(position))
}

async fn update_position(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(position_id): Path<Uuid>,
    Json(body): Json<erp_boq::models::UpdatePosition>,
) -> Result<Json<erp_boq::models::Position>, ApiError> {
    let boq_id = get_boq_id_for_position(&state, position_id).await?;
    verify_boq_owner(&state, boq_id, user_id).await?;

    let position = erp_boq::service::update_position(&state.db, position_id, body)
        .await
        .map_err(|e| match e {
            erp_boq::service::BoqError::PositionNotFound(_) => {
                ApiError::NotFound("Position not found".into())
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    erp_boq::service::log_activity(
        &state.db, boq_id, user_id, "update_position",
        Some("position"), Some(position_id), "Updated position", None,
    ).await.ok();

    Ok(Json(position))
}

async fn delete_position(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(position_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let boq_id = get_boq_id_for_position(&state, position_id).await?;
    verify_boq_owner(&state, boq_id, user_id).await?;

    erp_boq::service::delete_position(&state.db, position_id)
        .await
        .map_err(|e| match e {
            erp_boq::service::BoqError::PositionNotFound(_) => {
                ApiError::NotFound("Position not found".into())
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    erp_boq::service::log_activity(
        &state.db, boq_id, user_id, "delete_position",
        Some("position"), Some(position_id), "Deleted position", None,
    ).await.ok();

    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn get_markups(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
) -> Result<Json<Vec<erp_boq::models::BoqMarkup>>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let markups = erp_boq::service::get_markups(&state.db, boq_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(markups))
}

async fn create_markup(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
    Json(body): Json<erp_boq::models::CreateMarkup>,
) -> Result<Json<erp_boq::models::BoqMarkup>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let markup = erp_boq::service::create_markup(&state.db, boq_id, body)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(markup))
}

async fn update_markup(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(markup_id): Path<Uuid>,
    Json(body): Json<erp_boq::models::UpdateMarkup>,
) -> Result<Json<erp_boq::models::BoqMarkup>, ApiError> {
    let boq_id = get_boq_id_for_markup(&state, markup_id).await?;
    verify_boq_owner(&state, boq_id, user_id).await?;

    let markup = erp_boq::service::update_markup(&state.db, markup_id, body)
        .await
        .map_err(|e| match e {
            erp_boq::service::BoqError::MarkupNotFound(_) => {
                ApiError::NotFound("Markup not found".into())
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    Ok(Json(markup))
}

async fn delete_markup(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(markup_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let boq_id = get_boq_id_for_markup(&state, markup_id).await?;
    verify_boq_owner(&state, boq_id, user_id).await?;

    erp_boq::service::delete_markup(&state.db, markup_id)
        .await
        .map_err(|e| match e {
            erp_boq::service::BoqError::MarkupNotFound(_) => {
                ApiError::NotFound("Markup not found".into())
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn apply_default_markups(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
    Json(body): Json<ApplyDefaultsRequest>,
) -> Result<Json<Vec<erp_boq::models::BoqMarkup>>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let markups = erp_boq::service::apply_default_markups(&state.db, boq_id, &body.region)
        .await
        .map_err(|e| match e {
            erp_boq::service::BoqError::UnknownRegion(r) => {
                ApiError::BadRequest(format!("Unknown region: {r}"))
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    Ok(Json(markups))
}

async fn grand_total(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
) -> Result<Json<erp_boq::models::GrandTotal>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let total = erp_boq::service::compute_grand_total(&state.db, boq_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(total))
}

async fn create_snapshot(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
    Json(body): Json<CreateSnapshotRequest>,
) -> Result<Json<erp_boq::models::Snapshot>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let snapshot = erp_boq::service::create_snapshot(&state.db, boq_id, &body.name, user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(snapshot))
}

async fn list_snapshots(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
) -> Result<Json<Vec<erp_boq::models::Snapshot>>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let snapshots = erp_boq::service::list_snapshots(&state.db, boq_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(snapshots))
}

async fn restore_snapshot(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path((boq_id, snap_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    erp_boq::service::restore_snapshot(&state.db, boq_id, snap_id)
        .await
        .map_err(|e| match e {
            erp_boq::service::BoqError::SnapshotNotFound(_) => {
                ApiError::NotFound("Snapshot not found".into())
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    erp_boq::service::log_activity(
        &state.db, boq_id, user_id, "restore_snapshot",
        Some("snapshot"), Some(snap_id), "Restored from snapshot", None,
    ).await.ok();

    Ok(Json(serde_json::json!({ "restored": true })))
}

async fn validate_boq(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
) -> Result<Json<erp_core::validation::ValidationReport>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let boq_data = erp_boq::service::get_boq(&state.db, boq_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let positions: Vec<serde_json::Value> = boq_data
        .positions
        .iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect();

    let ctx = erp_core::validation::ValidationContext {
        positions,
        metadata: serde_json::to_value(&boq_data.boq).unwrap_or_default(),
    };

    let engine = erp_core::boq_rules::default_boq_engine();
    let report = engine.validate(&ctx);

    // Store validation report
    sqlx::query(
        r#"INSERT INTO validation_reports (id, target_type, target_id, status, score, results, created_at)
           VALUES ($1, 'boq', $2, $3, $4, $5, now())"#,
    )
    .bind(Uuid::new_v4())
    .bind(boq_id)
    .bind(serde_json::to_value(&report.status).unwrap_or_default().as_str())
    .bind(report.score)
    .bind(serde_json::to_value(&report).unwrap_or_default())
    .execute(&state.db)
    .await
    .ok();

    Ok(Json(report))
}

async fn activity_log(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(boq_id): Path<Uuid>,
) -> Result<Json<Vec<erp_boq::models::ActivityLog>>, ApiError> {
    verify_boq_owner(&state, boq_id, user_id).await?;

    let logs = sqlx::query_as::<_, erp_boq::models::ActivityLog>(
        "SELECT * FROM boq_activity_log WHERE boq_id = $1 ORDER BY created_at DESC LIMIT 200",
    )
    .bind(boq_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(logs))
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

async fn verify_boq_owner(state: &AppState, boq_id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
    let exists: Option<(Uuid,)> = sqlx::query_as(
        r#"SELECT b.id FROM boqs b
           JOIN projects p ON p.id = b.project_id
           WHERE b.id = $1 AND p.user_id = $2"#,
    )
    .bind(boq_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    if exists.is_none() {
        return Err(ApiError::NotFound("BOQ not found".into()));
    }

    Ok(())
}

async fn get_boq_id_for_position(state: &AppState, position_id: Uuid) -> Result<Uuid, ApiError> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT boq_id FROM boq_positions WHERE id = $1",
    )
    .bind(position_id)
    .fetch_optional(&state.db)
    .await?;

    row.map(|r| r.0)
        .ok_or_else(|| ApiError::NotFound("Position not found".into()))
}

async fn get_boq_id_for_markup(state: &AppState, markup_id: Uuid) -> Result<Uuid, ApiError> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT boq_id FROM boq_markups WHERE id = $1",
    )
    .bind(markup_id)
    .fetch_optional(&state.db)
    .await?;

    row.map(|r| r.0)
        .ok_or_else(|| ApiError::NotFound("Markup not found".into()))
}
