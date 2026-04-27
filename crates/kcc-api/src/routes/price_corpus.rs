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
        .route(
            "/price-corpus/imports/{import_id}",
            delete(delete_import).put(update_import_link),
        )
}

#[derive(Serialize)]
struct ImportSummary {
    import_id: Uuid,
    filename: String,
    sheet_count: i32,
    row_count: i32,
    skipped_count: i32,
    drawing_id: Option<Uuid>,
    /// True when this exact file (by content hash) was uploaded before and
    /// we re-used the existing import rather than re-importing.
    deduped: bool,
    /// True when content-overlap warnings exist (the user proceeded by
    /// passing on_conflict=add). Empty Vec on a clean import.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    overlap_warnings: Vec<OverlapMatch>,
}

#[derive(Serialize, Debug, Clone)]
struct OverlapMatch {
    import_id: Uuid,
    filename: String,
    drawing_id: Option<Uuid>,
    overlapping_rows: i64,
    total_rows: i32,
    overlap_pct: f64,
    imported_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
struct ImportQuery {
    /// Optional drawing to pin this import to. Empty / missing means
    /// the offer is not associated with any drawing — RAG will use it
    /// only when the whole-corpus fallback path runs.
    drawing_id: Option<Uuid>,
    /// Behavior when content-overlap is detected with an existing import:
    /// - `warn` (default): return 409 with overlap details, no insert
    /// - `add`: insert anyway alongside existing (user accepted)
    /// - `replace`: delete the most-overlapping prior import + cascade,
    ///   then insert this one
    /// - `skip`: don't insert; return the most-overlapping import as-is
    on_conflict: Option<String>,
}

async fn import_corpus(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Query(qry): Query<ImportQuery>,
    mut multipart: Multipart,
) -> Result<axum::response::Response, ApiError> {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let on_conflict = qry
        .on_conflict
        .as_deref()
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_else(|| "warn".to_string());

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
    let existing: Option<(Uuid, String, i32, i32, i32, Option<Uuid>)> = sqlx::query_as(
        "SELECT id, filename, sheet_count, row_count, skipped_count, drawing_id
         FROM user_price_imports WHERE user_id = $1 AND file_hash = $2",
    )
    .bind(user_id)
    .bind(&file_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
    if let Some((id, name, sheets, rows, skipped, prior_drawing)) = existing {
        // If the user is now linking to a drawing and the prior import has
        // no link (or a different one), update the link silently.
        if let Some(new_drawing) = qry.drawing_id {
            if prior_drawing != Some(new_drawing) {
                let _ = sqlx::query(
                    "UPDATE user_price_imports SET drawing_id = $1 WHERE id = $2",
                )
                .bind(new_drawing)
                .bind(id)
                .execute(&state.db)
                .await;
            }
        }
        return Ok(Json(ImportSummary {
            import_id: id,
            filename: name,
            sheet_count: sheets,
            row_count: rows,
            skipped_count: skipped,
            drawing_id: qry.drawing_id.or(prior_drawing),
            deduped: true,
            overlap_warnings: Vec::new(),
        })
        .into_response());
    }

    let parsed = kcc_core::price_corpus::parse_offer_xlsx(&data)
        .map_err(|e| ApiError::BadRequest(format!("XLSX parse failed: {e}")))?;

    if parsed.rows.is_empty() {
        return Err(ApiError::BadRequest(
            "No priced rows found in the workbook. Make sure the file has a header row with 'Описание', 'Ед. Цена мат', and 'Монтаж' columns."
                .into(),
        ));
    }

    // Content-overlap detection: how many existing imports share lots of
    // descriptions with this new file? Any prior import whose corpus rows
    // match >= 50% of the new file's descriptions is a candidate
    // duplicate. We compare on lowercase + trimmed description text.
    let new_descriptions: Vec<String> = parsed
        .rows
        .iter()
        .map(|r| r.description.trim().to_lowercase())
        .collect();
    let overlap_matches: Vec<OverlapMatch> = if new_descriptions.is_empty() {
        Vec::new()
    } else {
        let rows: Vec<(Uuid, String, Option<Uuid>, i32, chrono::DateTime<chrono::Utc>, i64)> =
            sqlx::query_as(
                "SELECT i.id, i.filename, i.drawing_id, i.row_count, i.imported_at,
                        COUNT(DISTINCT lower(trim(c.description))) FILTER (
                            WHERE lower(trim(c.description)) = ANY($2::text[])
                        ) AS overlap
                 FROM user_price_imports i
                 LEFT JOIN user_price_corpus c ON c.import_id = i.id
                 WHERE i.user_id = $1
                 GROUP BY i.id
                 HAVING COUNT(DISTINCT lower(trim(c.description))) FILTER (
                     WHERE lower(trim(c.description)) = ANY($2::text[])
                 ) > 0
                 ORDER BY overlap DESC, i.imported_at DESC
                 LIMIT 5",
            )
            .bind(user_id)
            .bind(&new_descriptions)
            .fetch_all(&state.db)
            .await
            .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
        let total_new = new_descriptions.len() as f64;
        rows.into_iter()
            .map(|(id, fname, did, total_rows, imp, overlap)| OverlapMatch {
                import_id: id,
                filename: fname,
                drawing_id: did,
                overlapping_rows: overlap,
                total_rows,
                overlap_pct: (overlap as f64 / total_new) * 100.0,
                imported_at: imp,
            })
            .filter(|m| m.overlap_pct >= 50.0)
            .collect()
    };

    // Conflict resolution policies.
    if !overlap_matches.is_empty() {
        match on_conflict.as_str() {
            "warn" => {
                // 409 with details — frontend prompts the user.
                return Ok((
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "conflict": "content_overlap",
                        "matches": overlap_matches,
                        "options": ["skip", "replace", "add"],
                        "summary": format!(
                            "{} priced rows match {} existing import(s) by description ≥ 50%. Choose 'skip' to keep what's there, 'replace' to swap out the closest match, or 'add' to keep both side-by-side.",
                            overlap_matches.iter().map(|m| m.overlapping_rows).sum::<i64>(),
                            overlap_matches.len(),
                        ),
                    })),
                )
                    .into_response());
            }
            "skip" => {
                // Return the closest prior import without re-importing.
                let m = &overlap_matches[0];
                return Ok(Json(ImportSummary {
                    import_id: m.import_id,
                    filename: m.filename.clone(),
                    sheet_count: 0,
                    row_count: m.total_rows,
                    skipped_count: 0,
                    drawing_id: m.drawing_id,
                    deduped: true,
                    overlap_warnings: overlap_matches,
                })
                .into_response());
            }
            "replace" => {
                let m = &overlap_matches[0];
                // CASCADE wipes corpus rows. We carry forward the previous
                // drawing_id when the caller didn't specify one.
                let _ = sqlx::query(
                    "DELETE FROM user_price_imports WHERE id = $1 AND user_id = $2",
                )
                .bind(m.import_id)
                .bind(user_id)
                .execute(&state.db)
                .await;
            }
            "add" => { /* fall through to insert */ }
            _ => {
                return Err(ApiError::BadRequest(format!(
                    "on_conflict must be one of warn|skip|replace|add (got '{on_conflict}')"
                )));
            }
        }
    }

    // Carry forward the prior drawing link if user didn't specify and we
    // just replaced a linked import.
    let resolved_drawing_id: Option<Uuid> = qry.drawing_id.or_else(|| {
        if on_conflict == "replace" {
            overlap_matches.first().and_then(|m| m.drawing_id)
        } else {
            None
        }
    });

    let import_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO user_price_imports
            (id, user_id, filename, file_hash, sheet_count, row_count, skipped_count, drawing_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(import_id)
    .bind(user_id)
    .bind(&filename)
    .bind(&file_hash)
    .bind(parsed.sheets.len() as i32)
    .bind(parsed.rows.len() as i32)
    .bind(parsed.skipped_rows as i32)
    .bind(resolved_drawing_id)
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
                 material_price_eur, labor_price_eur, total_unit_price_eur,
                 currency, source_sheet, source_row) ",
        );
        qb.push_values(chunk.iter(), |mut b, row| {
            b.push_bind(user_id)
                .push_bind(import_id)
                .push_bind(row.sek_code.clone())
                .push_bind(&row.description)
                .push_bind(&row.unit)
                .push_bind(row.quantity)
                .push_bind(row.material_price_eur)
                .push_bind(row.labor_price_eur)
                .push_bind(row.total_unit_price_eur)
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
        drawing_id: resolved_drawing_id,
        deduped: false,
        overlap_warnings: if on_conflict == "add" || on_conflict == "replace" {
            overlap_matches
        } else {
            Vec::new()
        },
    })
    .into_response())
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
    material_price_eur: Option<f64>,
    labor_price_eur: Option<f64>,
    total_unit_price_eur: Option<f64>,
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
                material_price_eur, labor_price_eur, total_unit_price_eur,
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
        material_price_eur: r.5,
        labor_price_eur: r.6,
        total_unit_price_eur: r.7,
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
    drawing_id: Option<Uuid>,
    drawing_filename: Option<String>,
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
        Option<Uuid>,
        Option<String>,
    )>(
        "SELECT i.id, i.filename, i.sheet_count, i.row_count, i.skipped_count,
                i.imported_at, i.drawing_id, d.filename
         FROM user_price_imports i
         LEFT JOIN drawings d ON d.id = i.drawing_id
         WHERE i.user_id = $1
         ORDER BY i.imported_at DESC",
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
        drawing_id: r.6,
        drawing_filename: r.7,
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

#[derive(Deserialize)]
struct UpdateLinkBody {
    /// Drawing to link this import to. Pass null to clear the link.
    drawing_id: Option<Uuid>,
}

async fn update_import_link(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(import_id): Path<Uuid>,
    Json(body): Json<UpdateLinkBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let result = sqlx::query(
        "UPDATE user_price_imports SET drawing_id = $1 WHERE id = $2 AND user_id = $3",
    )
    .bind(body.drawing_id)
    .bind(import_id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Import not found".into()));
    }
    Ok(Json(serde_json::json!({ "drawing_id": body.drawing_id })))
}
