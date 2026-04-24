use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn tendering_routes() -> Router<AppState> {
    Router::new()
        .route("/packages", post(create_package).get(list_packages))
        .route("/packages/{id}", get(get_package))
        .route("/packages/{id}/bids", post(submit_bid).get(list_bids))
}

// ── Models ──────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct TenderPackage {
    id: Uuid,
    project_id: Uuid,
    name: String,
    description: Option<String>,
    scope: Option<serde_json::Value>,
    status: String,
    due_date: Option<chrono::NaiveDate>,
    created_by: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
struct TenderBid {
    id: Uuid,
    package_id: Uuid,
    bidder_name: String,
    bidder_email: Option<String>,
    amount: f64,
    currency: String,
    notes: Option<String>,
    attachments: Option<serde_json::Value>,
    status: String,
    submitted_at: chrono::DateTime<chrono::Utc>,
}

// ── DTOs ────────────────────────────────────────────

#[derive(Deserialize)]
struct CreatePackageRequest {
    project_id: Uuid,
    name: String,
    description: Option<String>,
    scope: Option<serde_json::Value>,
    due_date: Option<chrono::NaiveDate>,
}

#[derive(Deserialize)]
struct ListPackagesQuery {
    project_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct SubmitBidRequest {
    bidder_name: String,
    bidder_email: Option<String>,
    amount: f64,
    currency: Option<String>,
    notes: Option<String>,
    attachments: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct PackageWithBids {
    package: TenderPackage,
    bids: Vec<TenderBid>,
}

// ── Handlers ────────────────────────────────────────

async fn create_package(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<CreatePackageRequest>,
) -> Result<Json<TenderPackage>, ApiError> {
    verify_project_owner(&state, body.project_id, user_id).await?;

    let package = sqlx::query_as::<_, TenderPackage>(
        r#"INSERT INTO tender_packages
           (id, project_id, name, description, scope, status, due_date, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, 'open', $6, $7, now(), now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(body.project_id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.scope)
    .bind(body.due_date)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(package))
}

async fn list_packages(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Query(params): Query<ListPackagesQuery>,
) -> Result<Json<Vec<TenderPackage>>, ApiError> {
    let packages = if let Some(project_id) = params.project_id {
        verify_project_owner(&state, project_id, user_id).await?;

        sqlx::query_as::<_, TenderPackage>(
            "SELECT * FROM tender_packages WHERE project_id = $1 ORDER BY created_at DESC",
        )
        .bind(project_id)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, TenderPackage>(
            r#"SELECT tp.* FROM tender_packages tp
               JOIN projects p ON p.id = tp.project_id
               WHERE p.user_id = $1
               ORDER BY tp.created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(packages))
}

async fn get_package(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<PackageWithBids>, ApiError> {
    let package = sqlx::query_as::<_, TenderPackage>(
        r#"SELECT tp.* FROM tender_packages tp
           JOIN projects p ON p.id = tp.project_id
           WHERE tp.id = $1 AND p.user_id = $2"#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Tender package not found".into()))?;

    let bids = sqlx::query_as::<_, TenderBid>(
        "SELECT * FROM tender_bids WHERE package_id = $1 ORDER BY submitted_at DESC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(PackageWithBids { package, bids }))
}

async fn submit_bid(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(package_id): Path<Uuid>,
    Json(body): Json<SubmitBidRequest>,
) -> Result<Json<TenderBid>, ApiError> {
    // Verify package exists and user owns the project
    let _package = sqlx::query_as::<_, TenderPackage>(
        r#"SELECT tp.* FROM tender_packages tp
           JOIN projects p ON p.id = tp.project_id
           WHERE tp.id = $1 AND p.user_id = $2"#,
    )
    .bind(package_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Tender package not found".into()))?;

    let currency = body.currency.unwrap_or_else(|| "BGN".to_string());

    let bid = sqlx::query_as::<_, TenderBid>(
        r#"INSERT INTO tender_bids
           (id, package_id, bidder_name, bidder_email, amount, currency, notes, attachments, status, submitted_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'submitted', now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(package_id)
    .bind(&body.bidder_name)
    .bind(&body.bidder_email)
    .bind(body.amount)
    .bind(&currency)
    .bind(&body.notes)
    .bind(&body.attachments)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(bid))
}

async fn list_bids(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(package_id): Path<Uuid>,
) -> Result<Json<Vec<TenderBid>>, ApiError> {
    // Verify ownership
    let _package = sqlx::query_as::<_, TenderPackage>(
        r#"SELECT tp.* FROM tender_packages tp
           JOIN projects p ON p.id = tp.project_id
           WHERE tp.id = $1 AND p.user_id = $2"#,
    )
    .bind(package_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Tender package not found".into()))?;

    let bids = sqlx::query_as::<_, TenderBid>(
        "SELECT * FROM tender_bids WHERE package_id = $1 ORDER BY submitted_at DESC",
    )
    .bind(package_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(bids))
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
