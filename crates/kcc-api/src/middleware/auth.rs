use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};

/// Auth middleware: extracts user_id from JWT Bearer token and adds it to request extensions.
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(ApiError::Unauthorized)?;

    let claims = crate::routes::auth::decode_token(&state.jwt_secret, token)?;

    if claims.token_type != "access" {
        return Err(ApiError::Unauthorized);
    }

    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| ApiError::Unauthorized)?;

    // Verify user still exists in database (catches stale tokens after DB reset)
    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| ApiError::Unauthorized)?;

    if exists.is_none() {
        return Err(ApiError::Unauthorized);
    }

    request.extensions_mut().insert(user_id);

    Ok(next.run(request).await)
}
