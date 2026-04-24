use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::Serialize;
use uuid::Uuid;

pub fn feature_routes() -> Router<AppState> {
    Router::new().route("/{drawing_id}", get(list_features))
}

#[derive(Serialize)]
struct FeatureResponse {
    id: Uuid,
    feature_type: String,
    description: String,
    centroid_x: f64,
    centroid_y: f64,
    properties: serde_json::Value,
}

async fn list_features(
    State(state): State<AppState>,
    Path(drawing_id): Path<Uuid>,
) -> Result<Json<Vec<FeatureResponse>>, ApiError> {
    let rows: Vec<(Uuid, String, String, f64, f64, serde_json::Value)> = sqlx::query_as(
        "SELECT id, feature_type, description, centroid_x, centroid_y, properties FROM features WHERE drawing_id = $1",
    )
    .bind(drawing_id)
    .fetch_all(&state.db)
    .await?;

    let features = rows
        .into_iter()
        .map(|(id, ft, desc, cx, cy, props)| FeatureResponse {
            id,
            feature_type: ft,
            description: desc,
            centroid_x: cx,
            centroid_y: cy,
            properties: props,
        })
        .collect();

    Ok(Json(features))
}
