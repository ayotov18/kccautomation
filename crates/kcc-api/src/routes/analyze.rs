use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::header,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Serialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn analyze_routes() -> Router<AppState> {
    Router::new()
        .route("/drawings/{drawing_id}/deep-analyze", post(trigger_deep_analyze))
        .route("/reports/{drawing_id}/analysis", get(download_analysis))
}

#[derive(Serialize)]
struct AnalyzeResponse {
    job_id: Uuid,
}

async fn trigger_deep_analyze(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
) -> Result<Json<AnalyzeResponse>, ApiError> {
    // Verify drawing belongs to user
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id).bind(user_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    let job_id = Uuid::new_v4();
    sqlx::query("INSERT INTO jobs (id, drawing_id, status) VALUES ($1, $2, 'queued')")
        .bind(job_id).bind(drawing_id)
        .execute(&state.db).await?;

    let job_data = serde_json::json!({
        "job_id": job_id,
        "drawing_id": drawing_id,
    });

    {
        let mut redis = state.redis.lock().await;
        redis::cmd("LPUSH")
            .arg("kcc:analyze-jobs")
            .arg(serde_json::to_string(&job_data).unwrap())
            .exec_async(&mut *redis)
            .await
            .map_err(|e| ApiError::Internal(format!("Redis enqueue failed: {e}")))?;
    }

    Ok(Json(AnalyzeResponse { job_id }))
}

async fn download_analysis(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id).bind(user_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT s3_key FROM reports WHERE drawing_id = $1 AND format = 'deep_analysis' ORDER BY generated_at DESC LIMIT 1",
    )
    .bind(drawing_id)
    .fetch_optional(&state.db).await?;

    let s3_key = row.ok_or_else(|| ApiError::NotFound("Deep analysis not found. Run 'Analyze' first.".into()))?.0;

    // Retry S3 download (same pattern as KSS)
    let data = crate::routes::kss::download_s3_with_retry(&state.s3, &state.s3_bucket, &s3_key).await?;

    Ok((
        [(header::CONTENT_TYPE, "application/json"),
         (header::CONTENT_DISPOSITION, "attachment; filename=\"deep-analysis.json\"")],
        data,
    ))
}
