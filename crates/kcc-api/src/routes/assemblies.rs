use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    routing::{get, post, put},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn assembly_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_assemblies).post(create_assembly))
        .route("/{id}", get(get_assembly))
        .route("/{id}/components", post(add_component))
        .route("/components/{id}", put(update_component).delete(delete_component))
        .route("/{id}/total-rate", get(total_rate))
}

#[derive(Deserialize)]
struct ListQuery {
    project_id: Option<Uuid>,
}

async fn list_assemblies(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<erp_assemblies::models::Assembly>>, ApiError> {
    let assemblies = erp_assemblies::service::list_assemblies(&state.db, params.project_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(assemblies))
}

async fn create_assembly(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Json(body): Json<erp_assemblies::models::CreateAssembly>,
) -> Result<Json<erp_assemblies::models::Assembly>, ApiError> {
    let assembly = erp_assemblies::service::create_assembly(&state.db, body)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(assembly))
}

async fn get_assembly(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<erp_assemblies::models::AssemblyWithComponents>, ApiError> {
    let assembly = erp_assemblies::service::get_assembly(&state.db, id)
        .await
        .map_err(|e| match e {
            erp_assemblies::service::AssemblyError::NotFound(_) => {
                ApiError::NotFound("Assembly not found".into())
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    Ok(Json(assembly))
}

async fn add_component(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(body): Json<erp_assemblies::models::CreateComponent>,
) -> Result<Json<erp_assemblies::models::Component>, ApiError> {
    let component = erp_assemblies::service::add_component(&state.db, id, body)
        .await
        .map_err(|e| match e {
            erp_assemblies::service::AssemblyError::NotFound(_) => {
                ApiError::NotFound("Assembly not found".into())
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    Ok(Json(component))
}

async fn update_component(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(body): Json<erp_assemblies::models::UpdateComponent>,
) -> Result<Json<erp_assemblies::models::Component>, ApiError> {
    let component = erp_assemblies::service::update_component(&state.db, id, body)
        .await
        .map_err(|e| match e {
            erp_assemblies::service::AssemblyError::ComponentNotFound(_) => {
                ApiError::NotFound("Component not found".into())
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    Ok(Json(component))
}

async fn delete_component(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    erp_assemblies::service::delete_component(&state.db, id)
        .await
        .map_err(|e| match e {
            erp_assemblies::service::AssemblyError::ComponentNotFound(_) => {
                ApiError::NotFound("Component not found".into())
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn total_rate(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let total = erp_assemblies::service::compute_total_rate(&state.db, id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "assembly_id": id, "total_rate": total })))
}
