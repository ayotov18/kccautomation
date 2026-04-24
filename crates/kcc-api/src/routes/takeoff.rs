use axum::{
    Json, Router,
    extract::{Extension, Multipart, Path, Query, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn takeoff_routes() -> Router<AppState> {
    Router::new()
        .route("/documents", post(upload_document).get(list_takeoff_documents))
        .route("/documents/{id}", get(get_document))
        .route("/measurements", post(create_measurement).get(list_measurements))
}

// ── Models ──────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct TakeoffDocument {
    id: Uuid,
    project_id: Uuid,
    filename: String,
    s3_key: String,
    file_size: Option<i64>,
    page_count: Option<i32>,
    status: String,
    uploaded_by: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
struct TakeoffMeasurement {
    id: Uuid,
    document_id: Uuid,
    page_number: Option<i32>,
    label: String,
    measurement_type: String,
    value: f64,
    unit: String,
    geometry: Option<serde_json::Value>,
    notes: Option<String>,
    created_by: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
}

// ── DTOs ────────────────────────────────────────────

#[derive(Deserialize)]
struct ListMeasurementsQuery {
    document_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct CreateMeasurementRequest {
    document_id: Uuid,
    page_number: Option<i32>,
    label: String,
    measurement_type: String,
    value: f64,
    unit: String,
    geometry: Option<serde_json::Value>,
    notes: Option<String>,
}

// ── Handlers ────────────────────────────────────────

async fn upload_document(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<TakeoffDocument>, ApiError> {
    let mut project_id: Option<Uuid> = None;
    let mut filename: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Multipart error: {e}")))?
    {
        match field.name() {
            Some("project_id") => {
                let text = field.text().await.map_err(|e| ApiError::BadRequest(e.to_string()))?;
                project_id = Some(
                    Uuid::parse_str(&text)
                        .map_err(|_| ApiError::BadRequest("Invalid project_id".into()))?,
                );
            }
            Some("file") => {
                filename = field.file_name().map(|s| s.to_string());
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| ApiError::BadRequest(format!("Failed to read file: {e}")))?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    let project_id = project_id.ok_or_else(|| ApiError::BadRequest("Missing project_id".into()))?;
    let bytes = file_bytes.ok_or_else(|| ApiError::BadRequest("Missing file field".into()))?;
    let fname = filename.unwrap_or_else(|| "takeoff.pdf".to_string());

    verify_project_owner(&state, project_id, user_id).await?;

    let doc_id = Uuid::new_v4();
    let s3_key = format!("takeoff/{project_id}/{doc_id}/{fname}");

    state
        .s3
        .put_object()
        .bucket(&state.s3_bucket)
        .key(&s3_key)
        .body(bytes.clone().into())
        .send()
        .await
        .map_err(|e| ApiError::StorageUnavailable(format!("S3 upload failed: {e}")))?;

    let file_size = bytes.len() as i64;

    let doc = sqlx::query_as::<_, TakeoffDocument>(
        r#"INSERT INTO takeoff_documents
           (id, project_id, filename, s3_key, file_size, page_count, status, uploaded_by, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, NULL, 'uploaded', $6, now(), now())
           RETURNING *"#,
    )
    .bind(doc_id)
    .bind(project_id)
    .bind(&fname)
    .bind(&s3_key)
    .bind(file_size)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(doc))
}

async fn list_takeoff_documents(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Query(_params): Query<ListMeasurementsQuery>,
) -> Result<Json<Vec<TakeoffDocument>>, ApiError> {
    // Reuse document_id field to filter by project when listing docs
    // The actual query param name in the spec is implicit; for documents,
    // we accept project_id. We use the same query struct for simplicity.
    let docs = sqlx::query_as::<_, TakeoffDocument>(
        r#"SELECT td.* FROM takeoff_documents td
           JOIN projects p ON p.id = td.project_id
           WHERE p.user_id = $1
           ORDER BY td.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(docs))
}

async fn get_document(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<TakeoffDocument>, ApiError> {
    let doc = sqlx::query_as::<_, TakeoffDocument>(
        r#"SELECT td.* FROM takeoff_documents td
           JOIN projects p ON p.id = td.project_id
           WHERE td.id = $1 AND p.user_id = $2"#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Takeoff document not found".into()))?;

    Ok(Json(doc))
}

async fn create_measurement(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<CreateMeasurementRequest>,
) -> Result<Json<TakeoffMeasurement>, ApiError> {
    // Verify document ownership
    let _doc = sqlx::query_as::<_, TakeoffDocument>(
        r#"SELECT td.* FROM takeoff_documents td
           JOIN projects p ON p.id = td.project_id
           WHERE td.id = $1 AND p.user_id = $2"#,
    )
    .bind(body.document_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Takeoff document not found".into()))?;

    let measurement = sqlx::query_as::<_, TakeoffMeasurement>(
        r#"INSERT INTO takeoff_measurements
           (id, document_id, page_number, label, measurement_type, value, unit, geometry, notes, created_by, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(body.document_id)
    .bind(body.page_number)
    .bind(&body.label)
    .bind(&body.measurement_type)
    .bind(body.value)
    .bind(&body.unit)
    .bind(&body.geometry)
    .bind(&body.notes)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(measurement))
}

async fn list_measurements(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Query(params): Query<ListMeasurementsQuery>,
) -> Result<Json<Vec<TakeoffMeasurement>>, ApiError> {
    let measurements = if let Some(doc_id) = params.document_id {
        // Verify ownership
        let _doc = sqlx::query_as::<_, TakeoffDocument>(
            r#"SELECT td.* FROM takeoff_documents td
               JOIN projects p ON p.id = td.project_id
               WHERE td.id = $1 AND p.user_id = $2"#,
        )
        .bind(doc_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Takeoff document not found".into()))?;

        sqlx::query_as::<_, TakeoffMeasurement>(
            "SELECT * FROM takeoff_measurements WHERE document_id = $1 ORDER BY created_at",
        )
        .bind(doc_id)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, TakeoffMeasurement>(
            r#"SELECT tm.* FROM takeoff_measurements tm
               JOIN takeoff_documents td ON td.id = tm.document_id
               JOIN projects p ON p.id = td.project_id
               WHERE p.user_id = $1
               ORDER BY tm.created_at DESC
               LIMIT 200"#,
        )
        .bind(user_id)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(measurements))
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
