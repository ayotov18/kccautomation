use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("storage unavailable: {0}")]
    StorageUnavailable(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Unauthorized".into(),
            ),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            ApiError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", msg.clone())
            }
            ApiError::StorageUnavailable(msg) => {
                tracing::error!("Storage unavailable: {msg}");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "STORAGE_UNAVAILABLE",
                    "File storage is temporarily unavailable. Please try again shortly.".into(),
                )
            }
            ApiError::Database(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DB_ERROR",
                    "Database error".into(),
                )
            }
        };

        let body = serde_json::json!({
            "error": { "code": code, "message": message }
        });

        (status, Json(body)).into_response()
    }
}
