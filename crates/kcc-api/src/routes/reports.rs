use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::header,
    response::IntoResponse,
    routing::get,
};
use serde::Serialize;
use uuid::Uuid;

pub fn report_routes() -> Router<AppState> {
    Router::new()
        .route("/{drawing_id}", get(get_report_json))
        .route("/{drawing_id}/pdf", get(get_report_pdf))
        .route("/{drawing_id}/csv", get(get_report_csv))
        .route("/{drawing_id}/kcc", get(get_kcc_results))
}

async fn get_report_json(
    State(state): State<AppState>,
    Path(drawing_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let data = download_report(&state, drawing_id, "json").await?;
    Ok(([(header::CONTENT_TYPE, "application/json")], data))
}

async fn get_report_pdf(
    State(state): State<AppState>,
    Path(drawing_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let data = download_report(&state, drawing_id, "pdf").await?;
    let disposition = format!("attachment; filename=\"kcc-report-{drawing_id}.pdf\"");
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        data,
    ))
}

async fn get_report_csv(
    State(state): State<AppState>,
    Path(drawing_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let data = download_report(&state, drawing_id, "csv").await?;
    let disposition = format!("attachment; filename=\"kcc-report-{drawing_id}.csv\"");
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv".to_string()),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        data,
    ))
}

async fn download_report(
    state: &AppState,
    drawing_id: Uuid,
    format: &str,
) -> Result<Vec<u8>, ApiError> {
    let s3_key: String = sqlx::query_scalar(
        "SELECT s3_key FROM reports WHERE drawing_id = $1 AND format = $2 ORDER BY generated_at DESC LIMIT 1",
    )
    .bind(drawing_id)
    .bind(format)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("No {format} report found for this drawing")))?;

    let result = state
        .s3
        .get_object()
        .bucket(&state.s3_bucket)
        .key(&s3_key)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("S3 download failed: {e}")))?;

    let data = result
        .body
        .collect()
        .await
        .map_err(|e| ApiError::Internal(format!("S3 read failed: {e}")))?
        .into_bytes()
        .to_vec();

    Ok(data)
}

#[derive(Serialize)]
struct KccResultResponse {
    feature_id: Uuid,
    classification: String,
    score: i32,
    factors: serde_json::Value,
    tolerance_chain: Option<serde_json::Value>,
}

async fn get_kcc_results(
    State(state): State<AppState>,
    Path(drawing_id): Path<Uuid>,
) -> Result<Json<Vec<KccResultResponse>>, ApiError> {
    let rows: Vec<(Uuid, String, i32, serde_json::Value, Option<serde_json::Value>)> =
        sqlx::query_as(
            "SELECT feature_id, classification, score, factors, tolerance_chain \
             FROM kcc_results WHERE drawing_id = $1 ORDER BY score DESC",
        )
        .bind(drawing_id)
        .fetch_all(&state.db)
        .await?;

    let results = rows
        .into_iter()
        .map(
            |(feature_id, classification, score, factors, tolerance_chain)| KccResultResponse {
                feature_id,
                classification,
                score,
                factors,
                tolerance_chain,
            },
        )
        .collect();

    Ok(Json(results))
}
