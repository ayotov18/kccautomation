//! Quantity norms + project distributions API.
//!
//! Mirrors the `/prices` routes but for per-unit consumption norms. The AI KSS
//! pipeline reads from these tables to produce defensible quantities instead
//! of hallucinating from drawing data alone.

use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

pub fn quantity_routes() -> Router<AppState> {
    Router::new()
        .route("/quantities/norms", get(list_norms).post(create_norm))
        .route(
            "/quantities/norms/{id}",
            put(update_norm).delete(delete_norm),
        )
        .route("/quantities/distributions", get(list_distributions).post(upsert_distribution))
        .route("/quantities/distributions/{id}", delete(delete_distribution))
        .route("/quantities/sources", get(list_sources).post(create_source))
        .route("/quantities/sources/{id}", delete(delete_source))
        .route("/quantities/runs", get(list_runs))
        .route("/quantities/bulk-import", post(bulk_import_norms))
        .route("/quantities/scrape", post(trigger_quantity_scrape))
}

// ── SCRAPE TRIGGER ────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TriggerQuantityScrapeRequest {
    #[serde(default)]
    source_ids: Vec<Uuid>,
}

async fn trigger_quantity_scrape(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<TriggerQuantityScrapeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let job_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO jobs (id, drawing_id, status, progress) VALUES ($1, NULL, 'queued', 0)",
    )
    .bind(job_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let payload = serde_json::json!({
        "job_id": job_id,
        "user_id": user_id,
        "source_ids": body.source_ids,
    });

    let mut conn = state.redis.lock().await;
    redis::cmd("LPUSH")
        .arg("kcc:quantity-scrape-jobs")
        .arg(payload.to_string())
        .query_async::<()>(&mut *conn)
        .await
        .map_err(|e| ApiError::Internal(format!("Redis error: {e}")))?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

// ── NORMS ──────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct NormDto {
    id: Option<Uuid>,
    sek_code: String,
    description_bg: String,
    work_unit: String,
    labor_qualified_h: f64,
    labor_helper_h: f64,
    labor_trade: Option<String>,
    materials: serde_json::Value,
    machinery: serde_json::Value,
    source: String,
    source_url: Option<String>,
    confidence: f64,
    user_id: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize)]
struct ListFilters {
    sek_group: Option<String>,
    search: Option<String>,
    source: Option<String>,
    #[serde(default)]
    only_mine: bool,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn list_norms(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Query(f): Query<ListFilters>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut sql = String::from(
        "SELECT id, sek_code, description_bg, work_unit,
                labor_qualified_h::float8, labor_helper_h::float8, labor_trade,
                materials, machinery, source, source_url,
                confidence::float8, user_id, created_at, updated_at
         FROM quantity_norms WHERE 1=1",
    );
    let mut binds: Vec<String> = Vec::new();
    let mut next_idx = 1usize;

    if f.only_mine {
        sql.push_str(&format!(" AND user_id = ${}", next_idx));
        binds.push(user_id.to_string());
        next_idx += 1;
    } else {
        sql.push_str(&format!(" AND (user_id IS NULL OR user_id = ${})", next_idx));
        binds.push(user_id.to_string());
        next_idx += 1;
    }
    if let Some(g) = &f.sek_group {
        sql.push_str(&format!(" AND sek_code LIKE ${} || '%'", next_idx));
        binds.push(g.clone());
        next_idx += 1;
    }
    if let Some(s) = &f.search {
        sql.push_str(&format!(
            " AND (description_bg ILIKE '%' || ${} || '%' OR sek_code ILIKE '%' || ${} || '%')",
            next_idx, next_idx
        ));
        binds.push(s.clone());
        next_idx += 1;
    }
    if let Some(src) = &f.source {
        sql.push_str(&format!(" AND source = ${}", next_idx));
        binds.push(src.clone());
        next_idx += 1;
    }
    sql.push_str(" ORDER BY sek_code, source");
    let limit = f.limit.unwrap_or(200).clamp(1, 1000);
    let offset = f.offset.unwrap_or(0).max(0);
    sql.push_str(&format!(" LIMIT {limit} OFFSET {offset}"));

    let mut q = sqlx::query(&sql);
    for b in &binds {
        // First bind is always a UUID
        if let Ok(u) = Uuid::parse_str(b) {
            q = q.bind(u);
        } else {
            q = q.bind(b.clone());
        }
    }

    let rows = q
        .fetch_all(&state.db)
        .await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                "sek_code": r.try_get::<String, _>("sek_code").unwrap_or_default(),
                "description_bg": r.try_get::<String, _>("description_bg").unwrap_or_default(),
                "work_unit": r.try_get::<String, _>("work_unit").unwrap_or_default(),
                "labor_qualified_h": r.try_get::<f64, _>("labor_qualified_h").unwrap_or(0.0),
                "labor_helper_h": r.try_get::<f64, _>("labor_helper_h").unwrap_or(0.0),
                "labor_trade": r.try_get::<Option<String>, _>("labor_trade").unwrap_or(None),
                "materials": r.try_get::<serde_json::Value, _>("materials").unwrap_or_default(),
                "machinery": r.try_get::<serde_json::Value, _>("machinery").unwrap_or_default(),
                "source": r.try_get::<String, _>("source").unwrap_or_default(),
                "source_url": r.try_get::<Option<String>, _>("source_url").unwrap_or(None),
                "confidence": r.try_get::<f64, _>("confidence").unwrap_or(0.0),
                "user_id": r.try_get::<Option<Uuid>, _>("user_id").unwrap_or(None),
                "created_at": r.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at").ok(),
                "updated_at": r.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at").ok(),
            })
        })
        .collect();

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM quantity_norms WHERE user_id IS NULL OR user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(serde_json::json!({
        "items": items,
        "total": count.0,
    })))
}

async fn create_norm(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(dto): Json<NormDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO quantity_norms
           (id, sek_code, description_bg, work_unit,
            labor_qualified_h, labor_helper_h, labor_trade,
            materials, machinery, source, source_url, confidence, user_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
    )
    .bind(id)
    .bind(&dto.sek_code)
    .bind(&dto.description_bg)
    .bind(&dto.work_unit)
    .bind(dto.labor_qualified_h)
    .bind(dto.labor_helper_h)
    .bind(&dto.labor_trade)
    .bind(&dto.materials)
    .bind(&dto.machinery)
    .bind(&dto.source)
    .bind(&dto.source_url)
    .bind(dto.confidence)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(serde_json::json!({ "id": id })))
}

async fn update_norm(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(dto): Json<NormDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let result = sqlx::query(
        "UPDATE quantity_norms SET
           sek_code = $1, description_bg = $2, work_unit = $3,
           labor_qualified_h = $4, labor_helper_h = $5, labor_trade = $6,
           materials = $7, machinery = $8, source = $9, source_url = $10,
           confidence = $11, updated_at = NOW()
         WHERE id = $12 AND (user_id = $13 OR user_id IS NULL)",
    )
    .bind(&dto.sek_code)
    .bind(&dto.description_bg)
    .bind(&dto.work_unit)
    .bind(dto.labor_qualified_h)
    .bind(dto.labor_helper_h)
    .bind(&dto.labor_trade)
    .bind(&dto.materials)
    .bind(&dto.machinery)
    .bind(&dto.source)
    .bind(&dto.source_url)
    .bind(dto.confidence)
    .bind(id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Norm not found".into()));
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn delete_norm(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let result = sqlx::query("DELETE FROM quantity_norms WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Norm not found or read-only".into()));
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ── BULK IMPORT ──

#[derive(Deserialize)]
struct BulkImportReq {
    norms: Vec<NormDto>,
    #[serde(default)]
    on_duplicate: String, // "skip" | "replace"
}

async fn bulk_import_norms(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<BulkImportReq>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut created = 0;
    let mut updated = 0;
    let replace = body.on_duplicate == "replace";

    for dto in body.norms {
        let conflict = if replace {
            "ON CONFLICT (sek_code, source, user_id) DO UPDATE SET
                description_bg = EXCLUDED.description_bg,
                work_unit = EXCLUDED.work_unit,
                labor_qualified_h = EXCLUDED.labor_qualified_h,
                labor_helper_h = EXCLUDED.labor_helper_h,
                materials = EXCLUDED.materials,
                machinery = EXCLUDED.machinery,
                updated_at = NOW()"
        } else {
            "ON CONFLICT DO NOTHING"
        };
        let sql = format!(
            "INSERT INTO quantity_norms
               (sek_code, description_bg, work_unit, labor_qualified_h, labor_helper_h,
                labor_trade, materials, machinery, source, source_url, confidence, user_id)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) {conflict}"
        );
        let res = sqlx::query(&sql)
            .bind(&dto.sek_code)
            .bind(&dto.description_bg)
            .bind(&dto.work_unit)
            .bind(dto.labor_qualified_h)
            .bind(dto.labor_helper_h)
            .bind(&dto.labor_trade)
            .bind(&dto.materials)
            .bind(&dto.machinery)
            .bind(&dto.source)
            .bind(&dto.source_url)
            .bind(dto.confidence)
            .bind(user_id)
            .execute(&state.db)
            .await
            .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
        if res.rows_affected() > 0 {
            if replace { updated += 1; } else { created += 1; }
        }
    }

    Ok(Json(serde_json::json!({ "created": created, "updated": updated })))
}

// ── DISTRIBUTIONS ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct DistributionDto {
    id: Option<Uuid>,
    building_type: String,
    metric_key: String,
    metric_label_bg: String,
    unit: String,
    min_value: Option<f64>,
    max_value: Option<f64>,
    median_value: f64,
    sample_size: i32,
    source: Option<String>,
    notes: Option<String>,
}

async fn list_distributions(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let rows = sqlx::query(
        "SELECT id, building_type, metric_key, metric_label_bg, unit,
                min_value::float8, max_value::float8, median_value::float8,
                sample_size, source, notes
         FROM project_distributions
         ORDER BY building_type, metric_key",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                "building_type": r.try_get::<String, _>("building_type").unwrap_or_default(),
                "metric_key": r.try_get::<String, _>("metric_key").unwrap_or_default(),
                "metric_label_bg": r.try_get::<String, _>("metric_label_bg").unwrap_or_default(),
                "unit": r.try_get::<String, _>("unit").unwrap_or_default(),
                "min_value": r.try_get::<Option<f64>, _>("min_value").unwrap_or(None),
                "max_value": r.try_get::<Option<f64>, _>("max_value").unwrap_or(None),
                "median_value": r.try_get::<f64, _>("median_value").unwrap_or(0.0),
                "sample_size": r.try_get::<i32, _>("sample_size").unwrap_or(0),
                "source": r.try_get::<Option<String>, _>("source").unwrap_or(None),
                "notes": r.try_get::<Option<String>, _>("notes").unwrap_or(None),
            })
        })
        .collect();
    Ok(Json(items))
}

async fn upsert_distribution(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Json(dto): Json<DistributionDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query(
        "INSERT INTO project_distributions
           (building_type, metric_key, metric_label_bg, unit,
            min_value, max_value, median_value, sample_size, source, notes)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
         ON CONFLICT (building_type, metric_key) DO UPDATE SET
            metric_label_bg = EXCLUDED.metric_label_bg,
            unit = EXCLUDED.unit,
            min_value = EXCLUDED.min_value,
            max_value = EXCLUDED.max_value,
            median_value = EXCLUDED.median_value,
            sample_size = EXCLUDED.sample_size,
            source = EXCLUDED.source,
            notes = EXCLUDED.notes",
    )
    .bind(&dto.building_type)
    .bind(&dto.metric_key)
    .bind(&dto.metric_label_bg)
    .bind(&dto.unit)
    .bind(dto.min_value)
    .bind(dto.max_value)
    .bind(dto.median_value)
    .bind(dto.sample_size)
    .bind(&dto.source)
    .bind(&dto.notes)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn delete_distribution(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query("DELETE FROM project_distributions WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ── SOURCES ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct SourceDto {
    id: Option<Uuid>,
    site_name: String,
    base_url: String,
    description: Option<String>,
    parser_template: String,
    is_builtin: Option<bool>,
    enabled: Option<bool>,
    last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    last_success: Option<bool>,
    last_norms_count: Option<i32>,
}

async fn list_sources(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let rows = sqlx::query(
        "SELECT id, site_name, base_url, description, parser_template,
                is_builtin, enabled, last_run_at, last_success, last_norms_count
         FROM quantity_sources ORDER BY is_builtin DESC, site_name",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                "site_name": r.try_get::<String, _>("site_name").unwrap_or_default(),
                "base_url": r.try_get::<String, _>("base_url").unwrap_or_default(),
                "description": r.try_get::<Option<String>, _>("description").unwrap_or(None),
                "parser_template": r.try_get::<String, _>("parser_template").unwrap_or_default(),
                "is_builtin": r.try_get::<bool, _>("is_builtin").unwrap_or(false),
                "enabled": r.try_get::<bool, _>("enabled").unwrap_or(true),
                "last_run_at": r.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_run_at").ok().flatten(),
                "last_success": r.try_get::<Option<bool>, _>("last_success").unwrap_or(None),
                "last_norms_count": r.try_get::<Option<i32>, _>("last_norms_count").unwrap_or(None),
            })
        })
        .collect();
    Ok(Json(items))
}

async fn create_source(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Json(dto): Json<SourceDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query(
        "INSERT INTO quantity_sources (site_name, base_url, description, parser_template, is_builtin, enabled)
         VALUES ($1, $2, $3, $4, false, true)",
    )
    .bind(&dto.site_name)
    .bind(&dto.base_url)
    .bind(&dto.description)
    .bind(&dto.parser_template)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn delete_source(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let result = sqlx::query(
        "DELETE FROM quantity_sources WHERE id = $1 AND is_builtin = false",
    )
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
    if result.rows_affected() == 0 {
        return Err(ApiError::BadRequest("Built-in sources cannot be deleted".into()));
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ── RUN HISTORY ───────────────────────────────────────────────────────────

async fn list_runs(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let rows = sqlx::query(
        "SELECT id, status, started_at, completed_at, total_sources, successful_sources,
                failed_sources, norms_created, norms_updated, elapsed_ms, notes
         FROM scrape_quantity_runs
         WHERE user_id = $1 OR user_id IS NULL
         ORDER BY started_at DESC
         LIMIT 50",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                "status": r.try_get::<String, _>("status").unwrap_or_default(),
                "started_at": r.try_get::<chrono::DateTime<chrono::Utc>, _>("started_at").ok(),
                "completed_at": r.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at").ok().flatten(),
                "total_sources": r.try_get::<i32, _>("total_sources").unwrap_or(0),
                "successful_sources": r.try_get::<i32, _>("successful_sources").unwrap_or(0),
                "failed_sources": r.try_get::<i32, _>("failed_sources").unwrap_or(0),
                "norms_created": r.try_get::<i32, _>("norms_created").unwrap_or(0),
                "norms_updated": r.try_get::<i32, _>("norms_updated").unwrap_or(0),
                "elapsed_ms": r.try_get::<Option<i32>, _>("elapsed_ms").unwrap_or(None),
                "notes": r.try_get::<Option<serde_json::Value>, _>("notes").unwrap_or(None),
            })
        })
        .collect();
    Ok(Json(items))
}
