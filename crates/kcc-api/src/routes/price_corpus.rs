//! User price corpus — upload, list, delete.
//!
//! The corpus backs RAG retrieval in the KSS pipeline. Each user maintains
//! their own library of priced offer rows (typically the actual KSS files
//! they've shipped to clients) which the generator searches via pg_trgm.

use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn price_corpus_routes() -> Router<AppState> {
    Router::new()
        .route("/price-corpus/import", post(import_corpus))
        .route("/price-corpus", get(list_corpus))
        .route("/price-corpus/imports", get(list_imports))
        .route("/price-corpus/imports/{import_id}", delete(delete_import))
}

#[derive(Serialize)]
struct ImportSummary {
    import_id: Uuid,
    filename: String,
    sheet_count: i32,
    row_count: i32,
    skipped_count: i32,
    /// True when this exact file (by content hash) was uploaded before and
    /// we re-used the existing import rather than re-importing.
    deduped: bool,
}

async fn import_corpus(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<ImportSummary>, ApiError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .ok_or_else(|| ApiError::BadRequest("No file uploaded".into()))?;

    let filename = field.file_name().unwrap_or("offer.xlsx").to_string();
    let data = field
        .bytes()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if data.len() < 100 {
        return Err(ApiError::BadRequest("File too small to be an XLSX".into()));
    }
    let file_hash = format!("{:x}", Sha256::digest(&data));

    // Idempotency: same user + same content hash → return prior import.
    let existing: Option<(Uuid, String, i32, i32, i32)> = sqlx::query_as(
        "SELECT id, filename, sheet_count, row_count, skipped_count
         FROM user_price_imports WHERE user_id = $1 AND file_hash = $2",
    )
    .bind(user_id)
    .bind(&file_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
    if let Some((id, name, sheets, rows, skipped)) = existing {
        return Ok(Json(ImportSummary {
            import_id: id,
            filename: name,
            sheet_count: sheets,
            row_count: rows,
            skipped_count: skipped,
            deduped: true,
        }));
    }

    let parsed = kcc_core::price_corpus::parse_offer_xlsx(&data)
        .map_err(|e| ApiError::BadRequest(format!("XLSX parse failed: {e}")))?;

    if parsed.rows.is_empty() {
        return Err(ApiError::BadRequest(
            "No priced rows found in the workbook. Make sure the file has a header row with 'Описание', 'Ед. Цена мат', and 'Монтаж' columns."
                .into(),
        ));
    }

    let import_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO user_price_imports
            (id, user_id, filename, file_hash, sheet_count, row_count, skipped_count)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(import_id)
    .bind(user_id)
    .bind(&filename)
    .bind(&file_hash)
    .bind(parsed.sheets.len() as i32)
    .bind(parsed.rows.len() as i32)
    .bind(parsed.skipped_rows as i32)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Bulk insert corpus rows. We batch into chunks of 200 to keep
    // parameter count under Postgres's 65535 limit while still amortising
    // round-trips.
    for chunk in parsed.rows.chunks(200) {
        // Construct a single multi-row VALUES insert via QueryBuilder.
        let mut qb = sqlx::QueryBuilder::new(
            "INSERT INTO user_price_corpus
                (user_id, import_id, sek_code, description, unit, quantity,
                 material_price_lv, labor_price_lv, total_unit_price_lv,
                 currency, source_sheet, source_row) ",
        );
        qb.push_values(chunk.iter(), |mut b, row| {
            b.push_bind(user_id)
                .push_bind(import_id)
                .push_bind(row.sek_code.clone())
                .push_bind(&row.description)
                .push_bind(&row.unit)
                .push_bind(row.quantity)
                .push_bind(row.material_price_lv)
                .push_bind(row.labor_price_lv)
                .push_bind(row.total_unit_price_lv)
                .push_bind("EUR")
                .push_bind(&row.source_sheet)
                .push_bind(row.source_row as i32);
        });
        qb.build()
            .execute(&state.db)
            .await
            .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
    }

    Ok(Json(ImportSummary {
        import_id,
        filename,
        sheet_count: parsed.sheets.len() as i32,
        row_count: parsed.rows.len() as i32,
        skipped_count: parsed.skipped_rows as i32,
        deduped: false,
    }))
}

#[derive(Deserialize)]
struct ListCorpusQuery {
    /// Optional substring filter on description (post-fetch ILIKE match).
    q: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize)]
struct CorpusRow {
    id: Uuid,
    sek_code: Option<String>,
    description: String,
    unit: String,
    quantity: Option<f64>,
    material_price_lv: Option<f64>,
    labor_price_lv: Option<f64>,
    total_unit_price_lv: Option<f64>,
    currency: String,
    source_sheet: Option<String>,
    source_row: Option<i32>,
    import_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn list_corpus(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Query(q): Query<ListCorpusQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let offset = q.offset.unwrap_or(0).max(0);
    let filter = q.q.as_deref().map(|s| format!("%{}%", s.trim()));

    let rows: Vec<CorpusRow> = sqlx::query_as::<_, (
        Uuid,
        Option<String>,
        String,
        String,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        String,
        Option<String>,
        Option<i32>,
        Option<Uuid>,
        chrono::DateTime<chrono::Utc>,
    )>(
        "SELECT id, sek_code, description, unit, quantity,
                material_price_lv, labor_price_lv, total_unit_price_lv,
                currency, source_sheet, source_row, import_id, created_at
         FROM user_price_corpus
         WHERE user_id = $1
           AND ($2::text IS NULL OR description ILIKE $2)
         ORDER BY created_at DESC
         LIMIT $3 OFFSET $4",
    )
    .bind(user_id)
    .bind(filter.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?
    .into_iter()
    .map(|r| CorpusRow {
        id: r.0,
        sek_code: r.1,
        description: r.2,
        unit: r.3,
        quantity: r.4,
        material_price_lv: r.5,
        labor_price_lv: r.6,
        total_unit_price_lv: r.7,
        currency: r.8,
        source_sheet: r.9,
        source_row: r.10,
        import_id: r.11,
        created_at: r.12,
    })
    .collect();

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM user_price_corpus WHERE user_id = $1
         AND ($2::text IS NULL OR description ILIKE $2)",
    )
    .bind(user_id)
    .bind(filter.as_deref())
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(serde_json::json!({
        "rows": rows,
        "total": total.0,
        "limit": limit,
        "offset": offset,
    })))
}

#[derive(Serialize)]
struct ImportRow {
    id: Uuid,
    filename: String,
    sheet_count: i32,
    row_count: i32,
    skipped_count: i32,
    imported_at: chrono::DateTime<chrono::Utc>,
}

async fn list_imports(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let rows: Vec<ImportRow> = sqlx::query_as::<_, (
        Uuid,
        String,
        i32,
        i32,
        i32,
        chrono::DateTime<chrono::Utc>,
    )>(
        "SELECT id, filename, sheet_count, row_count, skipped_count, imported_at
         FROM user_price_imports
         WHERE user_id = $1
         ORDER BY imported_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?
    .into_iter()
    .map(|r| ImportRow {
        id: r.0,
        filename: r.1,
        sheet_count: r.2,
        row_count: r.3,
        skipped_count: r.4,
        imported_at: r.5,
    })
    .collect();

    let total_corpus_rows = kcc_core::price_corpus::search::corpus_size(&state.db, user_id)
        .await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(serde_json::json!({
        "imports": rows,
        "total_corpus_rows": total_corpus_rows,
    })))
}

async fn delete_import(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(import_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Cascade: corpus rows reference import_id with ON DELETE CASCADE.
    let result = sqlx::query(
        "DELETE FROM user_price_imports WHERE id = $1 AND user_id = $2",
    )
    .bind(import_id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Import not found".into()));
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}
