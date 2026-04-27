use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn project_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_project).get(list_projects))
        .route("/{id}", get(get_project).put(update_project).delete(delete_project))
}

#[derive(Deserialize)]
struct CreateProjectRequest {
    name: String,
    description: Option<String>,
    client: Option<String>,
    location: Option<String>,
    currency: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct ProjectRow {
    id: Uuid,
    user_id: Uuid,
    name: String,
    description: Option<String>,
    client: Option<String>,
    location: Option<String>,
    currency: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

async fn create_project(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<Json<ProjectRow>, ApiError> {
    let currency = body.currency.unwrap_or_else(|| "EUR".to_string());

    let project = sqlx::query_as::<_, ProjectRow>(
        r#"INSERT INTO projects (id, user_id, name, description, client, location, currency, status, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', now(), now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.client)
    .bind(&body.location)
    .bind(&currency)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(project))
}

async fn list_projects(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<ProjectRow>>, ApiError> {
    let projects = sqlx::query_as::<_, ProjectRow>(
        "SELECT * FROM projects WHERE user_id = $1 ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(projects))
}

async fn get_project(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectRow>, ApiError> {
    let project = sqlx::query_as::<_, ProjectRow>(
        "SELECT * FROM projects WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Project not found".into()))?;

    Ok(Json(project))
}

#[derive(Deserialize)]
struct UpdateProjectRequest {
    name: Option<String>,
    description: Option<String>,
    client: Option<String>,
    location: Option<String>,
    currency: Option<String>,
    status: Option<String>,
}

async fn update_project(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectRow>, ApiError> {
    let current = sqlx::query_as::<_, ProjectRow>(
        "SELECT * FROM projects WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Project not found".into()))?;

    let name = body.name.unwrap_or(current.name);
    let description = body.description.or(current.description);
    let client = body.client.or(current.client);
    let location = body.location.or(current.location);
    let currency = body.currency.unwrap_or(current.currency);
    let status = body.status.unwrap_or(current.status);

    let project = sqlx::query_as::<_, ProjectRow>(
        r#"UPDATE projects
           SET name = $1, description = $2, client = $3, location = $4,
               currency = $5, status = $6, updated_at = now()
           WHERE id = $7 AND user_id = $8
           RETURNING *"#,
    )
    .bind(&name)
    .bind(&description)
    .bind(&client)
    .bind(&location)
    .bind(&currency)
    .bind(&status)
    .bind(id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(project))
}

async fn delete_project(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let result = sqlx::query("DELETE FROM projects WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Project not found".into()));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}
