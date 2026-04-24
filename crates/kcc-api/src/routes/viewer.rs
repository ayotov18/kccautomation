use axum::{
    Json, Router,
    body::Body,
    extract::{Extension, Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

/// Short-lived token minting route. Nested under the protected `/drawings` tree
/// so it inherits the existing Bearer-auth middleware and `user_id` extension.
pub fn viewer_token_routes() -> Router<AppState> {
    Router::new().route("/{id}/viewer-token", post(mint_viewer_token))
}

/// Public source-stream route. No auth middleware — trust comes from the
/// HMAC-signed JWT carried in the query string. The trailing `{filename}` is
/// informational; downstream viewers (mlightcad) sniff format from the URL's
/// last path segment, so we include an explicit `drawing.dxf` / `drawing.dwg`.
pub fn viewer_source_routes() -> Router<AppState> {
    Router::new().route("/viewer/source/{id}/{filename}", get(get_source))
}

#[derive(Serialize, Deserialize)]
struct ViewerClaims {
    sub: String,    // user_id (audit trail)
    drawing_id: String,
    exp: usize,
    token_type: String, // always "viewer"
}

#[derive(Serialize)]
struct ViewerTokenResponse {
    source_url: String,
    expires_in: u64,
}

const VIEWER_TOKEN_TTL_SECS: i64 = 300; // 5 minutes

async fn mint_viewer_token(
    State(state): State<AppState>,
    Path(drawing_id): Path<Uuid>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<ViewerTokenResponse>, ApiError> {
    // Ownership + format lookup in one round-trip. We pick the filename here
    // so the path segment mlightcad sniffs carries the right extension.
    let row: Option<(Uuid, Option<String>, String)> = sqlx::query_as(
        "SELECT id, s3_key_dxf, original_format FROM drawings WHERE id = $1 AND user_id = $2",
    )
    .bind(drawing_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;
    let (_, s3_key_dxf, original_format) =
        row.ok_or_else(|| ApiError::NotFound("Drawing not found".to_string()))?;

    let filename = match (s3_key_dxf.as_deref(), original_format.as_str()) {
        (Some(_), "dwg") => "drawing.dxf",
        (_, "dxf") => "drawing.dxf",
        (_, "pdf") => "drawing.pdf",
        _ => "drawing.dwg",
    };

    let now = chrono::Utc::now().timestamp();
    let claims = ViewerClaims {
        sub: user_id.to_string(),
        drawing_id: drawing_id.to_string(),
        exp: (now + VIEWER_TOKEN_TTL_SECS) as usize,
        token_type: "viewer".to_string(),
    };

    let key = jsonwebtoken::EncodingKey::from_secret(state.jwt_secret.as_bytes());
    let header = jsonwebtoken::Header::default();
    let token = jsonwebtoken::encode(&header, &claims, &key)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ViewerTokenResponse {
        source_url: format!("/api/v1/viewer/source/{drawing_id}/{filename}?token={token}"),
        expires_in: VIEWER_TOKEN_TTL_SECS as u64,
    }))
}

#[derive(Deserialize)]
struct SourceQuery {
    token: String,
}

async fn get_source(
    State(state): State<AppState>,
    Path((drawing_id, _filename)): Path<(Uuid, String)>,
    Query(q): Query<SourceQuery>,
) -> Result<Response, ApiError> {
    // Decode + validate token
    let key = jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes());
    let validation = jsonwebtoken::Validation::default();
    let claims = jsonwebtoken::decode::<ViewerClaims>(&q.token, &key, &validation)
        .map_err(|_| ApiError::Unauthorized)?
        .claims;

    if claims.token_type != "viewer" {
        return Err(ApiError::Unauthorized);
    }
    // Bind token to path — a leaked token can't be used to fetch a different drawing
    if claims.drawing_id != drawing_id.to_string() {
        return Err(ApiError::Unauthorized);
    }

    // Look up S3 keys + format. Prefer the converted DXF when available — the
    // mlightcad viewer handles DXF far more reliably than DWG (LibreDWG quirks).
    let row: Option<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT s3_key_original, s3_key_dxf, original_format FROM drawings WHERE id = $1",
    )
    .bind(drawing_id)
    .fetch_optional(&state.db)
    .await?;
    let (s3_key_original, s3_key_dxf, original_format) =
        row.ok_or_else(|| ApiError::NotFound("Drawing not found".to_string()))?;

    let (s3_key, fmt) = match (s3_key_dxf, original_format.as_str()) {
        (Some(dxf_key), "dwg") => (dxf_key, "dxf".to_string()),
        _ => (s3_key_original, original_format),
    };

    // Stream bytes from S3
    let obj = state
        .s3
        .get_object()
        .bucket(&state.s3_bucket)
        .key(&s3_key)
        .send()
        .await
        .map_err(|e| ApiError::StorageUnavailable(format!("S3 get failed: {e}")))?;

    let content_type = match fmt.as_str() {
        "dwg" => "application/acad",
        "dxf" => "application/dxf",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    };

    let bytes = obj
        .body
        .collect()
        .await
        .map_err(|e| ApiError::StorageUnavailable(format!("S3 read failed: {e}")))?
        .into_bytes();

    Ok(([
        (header::CONTENT_TYPE, content_type.to_string()),
        (header::CACHE_CONTROL, "private, max-age=300".to_string()),
    ], Body::from(bytes))
        .into_response())
}
