use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::Serialize;
use uuid::Uuid;

pub fn job_routes() -> Router<AppState> {
    Router::new().route("/{id}", get(get_job_status))
}

#[derive(Serialize)]
struct JobResponse {
    id: Uuid,
    drawing_id: Option<Uuid>,
    status: String,
    progress: i32,
    error_message: Option<String>,
    created_at: String,
}

async fn get_job_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JobResponse>, ApiError> {
    let row: (Uuid, Option<Uuid>, String, i32, Option<String>, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as(
            "SELECT id, drawing_id, status, progress, error_message, created_at FROM jobs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Job not found".to_string()))?;

    Ok(Json(JobResponse {
        id: row.0,
        drawing_id: row.1,
        status: row.2,
        progress: row.3,
        error_message: row.4,
        created_at: row.5.to_rfc3339(),
    }))
}
