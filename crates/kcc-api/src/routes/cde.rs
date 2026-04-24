use axum::{
    Json, Router,
    extract::{Extension, Multipart, Path, Query, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn cde_routes() -> Router<AppState> {
    Router::new()
        .route("/documents", post(upload_document).get(list_documents))
        .route("/documents/{id}", get(get_document).put(update_document_status))
}

// ── Models ──────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct CdeDocument {
    id: Uuid,
    project_id: Uuid,
    filename: String,
    title: Option<String>,
    revision: Option<String>,
    status: String,
    doc_type: Option<String>,
    s3_key: String,
    file_size: Option<i64>,
    uploaded_by: Uuid,
    metadata: Option<serde_json::Value>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

// ── DTOs ────────────────────────────────────────────

#[derive(Deserialize)]
struct ListDocumentsQuery {
    project_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct UpdateDocumentStatusRequest {
    status: String,
    revision: Option<String>,
    title: Option<String>,
    metadata: Option<serde_json::Value>,
}

// ── Handlers ────────────────────────────────────────

async fn upload_document(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<CdeDocument>, ApiError> {
    let mut project_id: Option<Uuid> = None;
    let mut title: Option<String> = None;
    let mut doc_type: Option<String> = None;
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
            Some("title") => {
                title = Some(field.text().await.map_err(|e| ApiError::BadRequest(e.to_string()))?);
            }
            Some("doc_type") => {
                doc_type = Some(field.text().await.map_err(|e| ApiError::BadRequest(e.to_string()))?);
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
    let fname = filename.unwrap_or_else(|| "document".to_string());

    // Verify ownership
    verify_project_owner(&state, project_id, user_id).await?;

    // Upload to S3
    let doc_id = Uuid::new_v4();
    let s3_key = format!("cde/{project_id}/{doc_id}/{fname}");

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

    let doc = sqlx::query_as::<_, CdeDocument>(
        r#"INSERT INTO cde_documents
           (id, project_id, filename, title, revision, status, doc_type, s3_key, file_size,
            uploaded_by, metadata, created_at, updated_at)
           VALUES ($1, $2, $3, $4, '1', 'uploaded', $5, $6, $7, $8, NULL, now(), now())
           RETURNING *"#,
    )
    .bind(doc_id)
    .bind(project_id)
    .bind(&fname)
    .bind(&title)
    .bind(&doc_type)
    .bind(&s3_key)
    .bind(file_size)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(doc))
}

async fn list_documents(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Query(params): Query<ListDocumentsQuery>,
) -> Result<Json<Vec<CdeDocument>>, ApiError> {
    let docs = if let Some(project_id) = params.project_id {
        verify_project_owner(&state, project_id, user_id).await?;

        sqlx::query_as::<_, CdeDocument>(
            "SELECT * FROM cde_documents WHERE project_id = $1 ORDER BY updated_at DESC",
        )
        .bind(project_id)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, CdeDocument>(
            r#"SELECT d.* FROM cde_documents d
               JOIN projects p ON p.id = d.project_id
               WHERE p.user_id = $1
               ORDER BY d.updated_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(docs))
}

async fn get_document(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<CdeDocument>, ApiError> {
    let doc = sqlx::query_as::<_, CdeDocument>(
        r#"SELECT d.* FROM cde_documents d
           JOIN projects p ON p.id = d.project_id
           WHERE d.id = $1 AND p.user_id = $2"#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Document not found".into()))?;

    Ok(Json(doc))
}

async fn update_document_status(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateDocumentStatusRequest>,
) -> Result<Json<CdeDocument>, ApiError> {
    // Verify ownership
    let _existing = sqlx::query_as::<_, CdeDocument>(
        r#"SELECT d.* FROM cde_documents d
           JOIN projects p ON p.id = d.project_id
           WHERE d.id = $1 AND p.user_id = $2"#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Document not found".into()))?;

    let doc = sqlx::query_as::<_, CdeDocument>(
        r#"UPDATE cde_documents
           SET status = $1,
               revision = COALESCE($2, revision),
               title = COALESCE($3, title),
               metadata = COALESCE($4, metadata),
               updated_at = now()
           WHERE id = $5
           RETURNING *"#,
    )
    .bind(&body.status)
    .bind(&body.revision)
    .bind(&body.title)
    .bind(&body.metadata)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(doc))
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
