use crate::error::ApiError;
use crate::state::AppState;
use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Hash password with argon2
    use argon2::PasswordHasher;
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let password_hash = argon2::Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e: argon2::password_hash::Error| ApiError::Internal(e.to_string()))?
        .to_string();

    // Insert user
    let user_id: uuid::Uuid =
        sqlx::query_scalar("INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id")
            .bind(&body.email)
            .bind(&password_hash)
            .fetch_one(&state.db)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(ref db_err)
                    if db_err.constraint() == Some("users_email_key") =>
                {
                    ApiError::BadRequest("Email already registered".to_string())
                }
                _ => ApiError::Database(e),
            })?;

    // Generate tokens
    let (access_token, refresh_token) = generate_tokens(&state.jwt_secret, user_id)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id: user_id.to_string(),
    }))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Find user
    let row: Option<(uuid::Uuid, String)> =
        sqlx::query_as("SELECT id, password_hash FROM users WHERE email = $1")
            .bind(&body.email)
            .fetch_optional(&state.db)
            .await?;

    let (user_id, password_hash) = row.ok_or(ApiError::Unauthorized)?;

    // Verify password
    use argon2::password_hash::PasswordVerifier;
    let parsed_hash =
        argon2::PasswordHash::new(&password_hash).map_err(|e| ApiError::Internal(e.to_string()))?;
    argon2::Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::Unauthorized)?;

    let (access_token, refresh_token) = generate_tokens(&state.jwt_secret, user_id)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id: user_id.to_string(),
    }))
}

async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Decode refresh token
    let claims = decode_token(&state.jwt_secret, &body.refresh_token)?;
    if claims.token_type != "refresh" {
        return Err(ApiError::BadRequest("Not a refresh token".to_string()));
    }

    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::Internal("Invalid token subject".to_string()))?;

    let (access_token, refresh_token) = generate_tokens(&state.jwt_secret, user_id)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id: user_id.to_string(),
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub token_type: String,
}

fn generate_tokens(secret: &str, user_id: uuid::Uuid) -> Result<(String, String), ApiError> {
    let now = chrono::Utc::now().timestamp() as usize;

    let access_claims = Claims {
        sub: user_id.to_string(),
        exp: now + 900, // 15 minutes
        token_type: "access".to_string(),
    };

    let refresh_claims = Claims {
        sub: user_id.to_string(),
        exp: now + 604800, // 7 days
        token_type: "refresh".to_string(),
    };

    let key = jsonwebtoken::EncodingKey::from_secret(secret.as_bytes());
    let header = jsonwebtoken::Header::default();

    let access = jsonwebtoken::encode(&header, &access_claims, &key)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let refresh = jsonwebtoken::encode(&header, &refresh_claims, &key)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok((access, refresh))
}

pub fn decode_token(secret: &str, token: &str) -> Result<Claims, ApiError> {
    let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
    let validation = jsonwebtoken::Validation::default();
    let data = jsonwebtoken::decode::<Claims>(token, &key, &validation)
        .map_err(|_| ApiError::Unauthorized)?;
    Ok(data.claims)
}
