use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    Router,
    extract::{Path, State},
    http::header,
    response::IntoResponse,
    routing::get,
};
use uuid::Uuid;

pub fn render_routes() -> Router<AppState> {
    Router::new().route("/{drawing_id}", get(get_render_packet))
}

async fn get_render_packet(
    State(state): State<AppState>,
    Path(drawing_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let s3_key = format!("reports/{drawing_id}/render.json");

    let result = state
        .s3
        .get_object()
        .bucket(&state.s3_bucket)
        .key(&s3_key)
        .send()
        .await
        .map_err(|e| ApiError::NotFound(format!("Render packet not found: {e}")))?;

    let data = result
        .body
        .collect()
        .await
        .map_err(|e| ApiError::Internal(format!("S3 read failed: {e}")))?
        .into_bytes()
        .to_vec();

    Ok(([(header::CONTENT_TYPE, "application/json")], data))
}
