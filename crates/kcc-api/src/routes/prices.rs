use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn price_routes() -> Router<AppState> {
    Router::new()
        .route("/prices/scrape", post(trigger_scrape))
        .route("/prices/scraped", get(list_scraped_prices))
        .route("/prices/rows", post(create_price_row))
        .route("/prices/rows/{id}", put(update_price_row))
        .route("/prices/rows/{id}", delete(archive_price_row))
        .route("/prices/runs", get(list_scrape_runs))
        .route("/prices/sources", get(list_sources))
        .route("/prices/sources", post(add_source))
        .route("/prices/sources/{id}", put(toggle_source))
        .route("/prices/sources/{id}", delete(delete_source))
        .route("/price-lists/{id}/default", put(set_default_price_list))
}

// ── Trigger scrape ──────────────────────────────────────────

#[derive(Deserialize)]
struct TriggerScrapeRequest {
    source_ids: Option<Vec<Uuid>>,
}

#[derive(Serialize)]
struct JobResponse {
    job_id: Uuid,
}

async fn trigger_scrape(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<TriggerScrapeRequest>,
) -> Result<Json<JobResponse>, ApiError> {
    let job_id = Uuid::new_v4();

    // Create job record (no drawing_id for scrape jobs)
    sqlx::query(
        "INSERT INTO jobs (id, drawing_id, status, progress) VALUES ($1, NULL, 'queued', 0)"
    )
    .bind(job_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Enqueue to Redis
    let job_data = serde_json::json!({
        "job_id": job_id,
        "user_id": user_id,
        "source_ids": body.source_ids.unwrap_or_default(),
    });

    let mut conn = state.redis.lock().await;
    redis::cmd("LPUSH")
        .arg("kcc:scrape-jobs")
        .arg(job_data.to_string())
        .query_async::<()>(&mut *conn)
        .await
        .map_err(|e| ApiError::Internal(format!("Redis error: {e}")))?;

    Ok(Json(JobResponse { job_id }))
}

// ── List scraped prices ─────────────────────────────────────

#[derive(Deserialize)]
struct ScrapedPricesQuery {
    site: Option<String>,
    category: Option<String>,
    search: Option<String>,
    source_type: Option<String>,  // "scraped" | "manual" | "edited"
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
struct ScrapedPriceItem {
    id: Uuid,
    site: String,
    sek_code: Option<String>,
    sek_group: Option<String>,
    item_name: String,
    unit: Option<String>,
    price_min_eur: Option<f64>,
    price_max_eur: Option<f64>,
    currency: Option<String>,
    mapping_confidence: Option<f64>,
    extraction_confidence: Option<f64>,
    extraction_strategy: Option<String>,
    is_manual: bool,
    is_user_edited: bool,
    notes: Option<String>,
    captured_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct ScrapedPricesResponse {
    items: Vec<ScrapedPriceItem>,
    total: i64,
}

async fn list_scraped_prices(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Query(q): Query<ScrapedPricesQuery>,
) -> Result<Json<ScrapedPricesResponse>, ApiError> {
    let limit = q.limit.unwrap_or(50).min(500);
    let offset = q.offset.unwrap_or(0);

    let items: Vec<ScrapedPriceItem> = sqlx::query_as(
        r#"SELECT id, site, sek_code, sek_group, item_name, unit, price_min_eur, price_max_eur, price_min_eur, price_max_eur, currency, mapping_confidence, extraction_confidence, extraction_strategy, is_manual, is_user_edited, notes, captured_at
           FROM scraped_price_rows
           WHERE user_id = $1
             AND archived_at IS NULL
             AND ($2::text IS NULL OR site = $2)
             AND ($3::text IS NULL OR sek_group = $3)
             AND ($4::text IS NULL OR item_name ILIKE '%' || $4 || '%' OR sek_code ILIKE '%' || $4 || '%')
             AND ($5::text IS NULL OR
                  ($5 = 'manual' AND is_manual = true) OR
                  ($5 = 'edited' AND is_user_edited = true) OR
                  ($5 = 'scraped' AND is_manual = false AND is_user_edited = false))
           ORDER BY captured_at DESC
           LIMIT $6 OFFSET $7"#,
    )
    .bind(user_id)
    .bind(&q.site)
    .bind(&q.category)
    .bind(&q.search)
    .bind(&q.source_type)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // COUNT with same filters (not global)
    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM scraped_price_rows
           WHERE user_id = $1
             AND archived_at IS NULL
             AND ($2::text IS NULL OR site = $2)
             AND ($3::text IS NULL OR sek_group = $3)
             AND ($4::text IS NULL OR item_name ILIKE '%' || $4 || '%' OR sek_code ILIKE '%' || $4 || '%')
             AND ($5::text IS NULL OR
                  ($5 = 'manual' AND is_manual = true) OR
                  ($5 = 'edited' AND is_user_edited = true) OR
                  ($5 = 'scraped' AND is_manual = false AND is_user_edited = false))"#,
    )
    .bind(user_id)
    .bind(&q.site)
    .bind(&q.category)
    .bind(&q.search)
    .bind(&q.source_type)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(ScrapedPricesResponse {
        items,
        total: total.0,
    }))
}

// ── Scrape runs ─────────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct ScrapeRunItem {
    id: Uuid,
    status: String,
    started_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    total_sources: i32,
    successful_sources: i32,
    failed_sources: i32,
    artifact_failures: i32,
}

async fn list_scrape_runs(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<ScrapeRunItem>>, ApiError> {
    let runs: Vec<ScrapeRunItem> = sqlx::query_as(
        "SELECT id, status, started_at, completed_at, total_sources, successful_sources, failed_sources, artifact_failures
         FROM scrape_runs WHERE user_id = $1 ORDER BY started_at DESC LIMIT 20",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(runs))
}

// ── Scrape sources management ───────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct ScrapeSourceItem {
    id: Uuid,
    site_name: String,
    base_url: String,
    enabled: bool,
    is_builtin: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn list_sources(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<ScrapeSourceItem>>, ApiError> {
    let sources: Vec<ScrapeSourceItem> = sqlx::query_as(
        "SELECT id, site_name, base_url, enabled, is_builtin, created_at
         FROM scrape_sources WHERE user_id = $1 ORDER BY is_builtin DESC, created_at",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(sources))
}

#[derive(Deserialize)]
struct AddSourceRequest {
    site_name: String,
    base_url: String,
}

async fn add_source(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<AddSourceRequest>,
) -> Result<Json<ScrapeSourceItem>, ApiError> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO scrape_sources (id, user_id, site_name, base_url, enabled, is_builtin, created_at)
         VALUES ($1, $2, $3, $4, true, false, $5)"
    )
    .bind(id)
    .bind(user_id)
    .bind(&body.site_name)
    .bind(&body.base_url)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(ScrapeSourceItem {
        id,
        site_name: body.site_name,
        base_url: body.base_url,
        enabled: true,
        is_builtin: false,
        created_at: now,
    }))
}

#[derive(Deserialize)]
struct ToggleSourceRequest {
    enabled: bool,
}

async fn toggle_source(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(body): Json<ToggleSourceRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query(
        "UPDATE scrape_sources SET enabled = $1 WHERE id = $2 AND user_id = $3"
    )
    .bind(body.enabled)
    .bind(id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(serde_json::json!({"ok": true})))
}

async fn delete_source(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query(
        "DELETE FROM scrape_sources WHERE id = $1 AND user_id = $2 AND is_builtin = false"
    )
    .bind(id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(serde_json::json!({"ok": true})))
}

// ── Set default price list ──────────────────────────────────

async fn set_default_price_list(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Unset all defaults for this user
    sqlx::query("UPDATE price_lists SET is_default = false WHERE user_id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Set new default
    sqlx::query("UPDATE price_lists SET is_default = true WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(serde_json::json!({"ok": true})))
}

// ── CRUD: Create manual price row ───────────────────────────

#[derive(Deserialize)]
struct CreatePriceRequest {
    sek_code: Option<String>,
    item_name: String,
    category: Option<String>,
    unit: Option<String>,
    price_min_eur: Option<f64>,
    price_max_eur: Option<f64>,
    notes: Option<String>,
}

async fn create_price_row(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<CreatePriceRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = Uuid::new_v4();
    let sek_group = body.sek_code.as_deref()
        .and_then(|c| c.find('.').map(|i| c[..i].to_string()));

    sqlx::query(
        "INSERT INTO scraped_price_rows (id, user_id, site, source_url, item_name, unit, price_min_eur, price_max_eur, price_min, price_max, currency, sek_code, sek_group, category_slug, notes, is_manual, mapping_confidence)
         VALUES ($1, $2, 'manual', '', $3, $4, $5, $6, $5, $6, 'EUR', $7, $8, $9, $10, true, 1.0)",
    )
    .bind(id)
    .bind(user_id)
    .bind(&body.item_name)
    .bind(&body.unit)
    .bind(body.price_min_eur)
    .bind(body.price_max_eur)
    .bind(&body.sek_code)
    .bind(&sek_group)
    .bind(&body.category)
    .bind(&body.notes)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Record to DRM as price association
    if let Some(sek_code) = &body.sek_code {
        let _ = kcc_core::drm::recorder::record_user_correction(
            &state.db, user_id, Uuid::nil(),
            kcc_core::drm::ARTIFACT_PRICE,
            &body.item_name, sek_code,
            sek_group.as_deref().unwrap_or(""),
            &body.item_name,
            body.unit.as_deref().unwrap_or("М2"),
        ).await;
    }

    Ok(Json(serde_json::json!({"id": id, "ok": true})))
}

// ── CRUD: Update price row ──────────────────────────────────

#[derive(Deserialize)]
struct UpdatePriceRequest {
    sek_code: Option<String>,
    item_name: Option<String>,
    unit: Option<String>,
    price_min_eur: Option<f64>,
    price_max_eur: Option<f64>,
    notes: Option<String>,
}

async fn update_price_row(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePriceRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let original: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT item_name, sek_code FROM scraped_price_rows WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    if original.is_none() {
        return Err(ApiError::NotFound("Price row not found".to_string()));
    }

    sqlx::query(
        "UPDATE scraped_price_rows SET
            sek_code = COALESCE($1, sek_code),
            item_name = COALESCE($2, item_name),
            unit = COALESCE($3, unit),
            price_min_eur = COALESCE($4, price_min_eur),
            price_max_eur = COALESCE($5, price_max_eur),
            notes = COALESCE($6, notes),
            is_user_edited = true
         WHERE id = $7 AND user_id = $8",
    )
    .bind(&body.sek_code)
    .bind(&body.item_name)
    .bind(&body.unit)
    .bind(body.price_min_eur)
    .bind(body.price_max_eur)
    .bind(&body.notes)
    .bind(id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Record SEK code change to DRM
    if let (Some((orig_name, _orig_sek)), Some(new_sek)) = (&original, &body.sek_code) {
        let name = body.item_name.as_deref().unwrap_or(orig_name);
        let sek_group = new_sek.find('.').map(|i| &new_sek[..i]).unwrap_or(new_sek);
        let _ = kcc_core::drm::recorder::record_user_correction(
            &state.db, user_id, Uuid::nil(),
            kcc_core::drm::ARTIFACT_PRICE,
            name, new_sek, sek_group, name,
            body.unit.as_deref().unwrap_or("М2"),
        ).await;
    }

    Ok(Json(serde_json::json!({"ok": true})))
}

// ── CRUD: Archive (soft delete) price row ───────────────────

async fn archive_price_row(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query(
        "UPDATE scraped_price_rows SET archived_at = now() WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(serde_json::json!({"ok": true})))
}
