use axum::{
    Json, Router,
    extract::{Extension, Multipart, Path, Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn cost_routes() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_costs))
        .route("/items/{id}", get(get_cost_item))
        .route("/items", post(create_cost_item))
        .route("/import", post(import_csv))
        .route("/regions", get(list_regions))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    region: Option<String>,
    limit: Option<i64>,
}

async fn search_costs(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<erp_costs::models::CostItem>>, ApiError> {
    let limit = params.limit.unwrap_or(50).min(200);

    let items = erp_costs::service::search_costs(
        &state.db,
        &params.q,
        params.region.as_deref(),
        limit,
    )
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(items))
}

async fn get_cost_item(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<erp_costs::models::CostItem>, ApiError> {
    let item = erp_costs::service::get_cost_item(&state.db, id)
        .await
        .map_err(|e| match e {
            erp_costs::service::CostError::NotFound(_) => {
                ApiError::NotFound("Cost item not found".into())
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    Ok(Json(item))
}

async fn create_cost_item(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Json(body): Json<erp_costs::models::CreateCostItem>,
) -> Result<Json<erp_costs::models::CostItem>, ApiError> {
    let item = erp_costs::service::create_cost_item(&state.db, body)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(item))
}

#[derive(Deserialize)]
struct ImportQuery {
    region: Option<String>,
    source: Option<String>,
}

async fn import_csv(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Query(params): Query<ImportQuery>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, ApiError> {
    let region = params.region.unwrap_or_else(|| "BG".to_string());
    let source = params.source.unwrap_or_else(|| "csv_import".to_string());

    let mut csv_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Multipart error: {e}")))?
    {
        if field.name() == Some("file") {
            csv_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Failed to read file: {e}")))?
                    .to_vec(),
            );
            break;
        }
    }

    let bytes = csv_bytes.ok_or_else(|| ApiError::BadRequest("Missing 'file' field".into()))?;

    let count = erp_costs::service::import_csv(&state.db, &bytes, &region, &source)
        .await
        .map_err(|e| match e {
            erp_costs::service::CostError::CsvParse(msg) => {
                ApiError::BadRequest(format!("CSV parse error: {msg}"))
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    Ok(Json(serde_json::json!({ "imported": count })))
}

async fn list_regions(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
) -> Result<Json<Vec<String>>, ApiError> {
    let regions = erp_costs::service::list_regions(&state.db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(regions))
}
