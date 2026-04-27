use axum::{
    Json, Router,
    extract::{Extension, Multipart, Path, State},
    http::header,
    response::IntoResponse,
    routing::{get, post, put},
};
use serde::Serialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn kss_routes() -> Router<AppState> {
    Router::new()
        .route("/drawings/{drawing_id}/generate-kss", post(generate_kss))
        .route("/drawings/{drawing_id}/generate-ai-kss", post(trigger_ai_kss_research))
        .route("/drawings/{drawing_id}/ai-kss/items", get(get_ai_kss_research_items))
        .route("/drawings/{drawing_id}/ai-kss/items/{item_id}", put(update_ai_kss_item))
        .route("/drawings/{drawing_id}/ai-kss/generate", post(trigger_ai_kss_generation))
        .route("/drawings/{drawing_id}/ai-kss/status", get(get_ai_kss_status))
        .route("/price-lists/upload", post(upload_price_list))
        .route("/price-lists", get(list_price_lists))
        .route("/reports/{drawing_id}/kss/excel", get(download_kss_excel))
        .route("/reports/{drawing_id}/kss/pdf", get(download_kss_pdf))
        .route("/reports/{drawing_id}/kss/data", get(get_kss_data))
        .route("/reports/{drawing_id}/kss/audit", get(get_kss_audit_trail))
        .route("/reports/{drawing_id}/kss/suggestions/{item_id}/accept", post(accept_suggestion))
        .route("/reports/{drawing_id}/kss/suggestions/{item_id}/reject", post(reject_suggestion))
        .route("/reports/{drawing_id}/kss/items", post(add_kss_item))
        .route("/reports/{drawing_id}/kss/finalize", post(finalize_kss))
        .route("/drawings/{drawing_id}/structures/{structure_id}", put(rename_structure))
        .route("/drawings/{drawing_id}/structures/merge", post(merge_structures))
        .route("/drawings/{drawing_id}/structures/{structure_id}/delete", post(delete_structure))
}

#[derive(Serialize)]
struct GenerateKssResponse {
    job_id: Uuid,
}

#[derive(serde::Deserialize)]
struct GenerateKssRequest {
    price_list_id: Option<Uuid>,
}

async fn generate_kss(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
    Json(body): Json<GenerateKssRequest>,
) -> Result<Json<GenerateKssResponse>, ApiError> {
    // Verify drawing belongs to user
    let exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM drawings WHERE id = $1 AND user_id = $2",
    )
    .bind(drawing_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    if exists.is_none() {
        return Err(ApiError::NotFound("Drawing not found".into()));
    }

    // Create job
    let job_id = Uuid::new_v4();
    sqlx::query("INSERT INTO jobs (id, drawing_id, status) VALUES ($1, $2, 'queued')")
        .bind(job_id)
        .bind(drawing_id)
        .execute(&state.db)
        .await?;

    // Enqueue KSS job
    let job_data = serde_json::json!({
        "job_id": job_id,
        "drawing_id": drawing_id,
        "price_list_id": body.price_list_id,
    });

    {
        let mut redis = state.redis.lock().await;
        redis::cmd("LPUSH")
            .arg("kcc:kss-jobs")
            .arg(serde_json::to_string(&job_data).unwrap())
            .exec_async(&mut *redis)
            .await
            .map_err(|e| ApiError::Internal(format!("Redis enqueue failed: {e}")))?;
    }

    Ok(Json(GenerateKssResponse { job_id }))
}

#[derive(Serialize)]
struct PriceListResponse {
    id: Uuid,
    name: String,
    item_count: i32,
    created_at: String,
}

async fn upload_price_list(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<PriceListResponse>, ApiError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .ok_or_else(|| ApiError::BadRequest("No file uploaded".into()))?;

    let filename = field.file_name().unwrap_or("pricelist.csv").to_string();
    let data = field.bytes().await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Parse to count items
    let price_list = kcc_core::kss::types::PriceList::from_csv(&data)
        .map_err(|e| ApiError::BadRequest(format!("Invalid price list: {e}")))?;

    let id = Uuid::new_v4();
    let s3_key = format!("price-lists/{user_id}/{id}.csv");

    state.s3.put_object()
        .bucket(&state.s3_bucket)
        .key(&s3_key)
        .body(data.into())
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("S3 upload failed: {e}")))?;

    sqlx::query(
        "INSERT INTO price_lists (id, user_id, name, s3_key, item_count) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(user_id)
    .bind(&filename)
    .bind(&s3_key)
    .bind(price_list.items.len() as i32)
    .execute(&state.db)
    .await?;

    Ok(Json(PriceListResponse {
        id,
        name: filename,
        item_count: price_list.items.len() as i32,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

async fn list_price_lists(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<PriceListResponse>>, ApiError> {
    let rows: Vec<(Uuid, String, i32, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT id, name, item_count, created_at FROM price_lists WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let lists = rows.into_iter().map(|(id, name, count, created_at)| {
        PriceListResponse { id, name, item_count: count, created_at: created_at.to_rfc3339() }
    }).collect();

    Ok(Json(lists))
}

pub async fn download_s3_with_retry(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
) -> Result<Vec<u8>, ApiError> {
    for attempt in 0..3 {
        match s3.get_object().bucket(bucket).key(key).send().await {
            Ok(result) => {
                let data = result.body.collect().await
                    .map_err(|e| ApiError::Internal(format!("S3 read failed: {e}")))?;
                return Ok(data.into_bytes().to_vec());
            }
            Err(e) if attempt < 2 => {
                tracing::warn!(attempt, error = %e, "S3 download failed, retrying");
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            Err(e) => return Err(ApiError::Internal(format!("S3 download failed after retries: {e}"))),
        }
    }
    unreachable!()
}

async fn download_kss_excel(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id).bind(user_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    // Strategy 1: Try pre-generated S3 file
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT s3_key FROM reports WHERE drawing_id = $1 AND format = 'kss_excel' ORDER BY generated_at DESC LIMIT 1",
    )
    .bind(drawing_id)
    .fetch_optional(&state.db)
    .await?;

    if let Some((s3_key,)) = row {
        if let Ok(data) = download_s3_with_retry(&state.s3, &state.s3_bucket, &s3_key).await {
            return Ok((
                [(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                 (header::CONTENT_DISPOSITION, "attachment; filename=\"kss-report.xlsx\"")],
                data,
            ));
        }
    }

    // Strategy 2: On-demand generation from kss_reports JSONB
    let report_row: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT report_data FROM kss_reports WHERE drawing_id = $1 ORDER BY generated_at DESC LIMIT 1",
    )
    .bind(drawing_id)
    .fetch_optional(&state.db)
    .await?;

    let (report_data,) = report_row
        .ok_or_else(|| ApiError::NotFound("No KSS report found for this drawing".into()))?;

    let sectioned: kcc_core::kss::types::SectionedKssReport = serde_json::from_value(report_data)
        .map_err(|e| ApiError::Internal(format!("Failed to deserialize KSS report: {e}")))?;

    let excel_bytes = kcc_report::kss_excel::generate_sectioned_kss_excel(&sectioned)
        .map_err(|e| ApiError::Internal(format!("Excel generation failed: {e}")))?;

    Ok((
        [(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
         (header::CONTENT_DISPOSITION, "attachment; filename=\"kss-report.xlsx\"")],
        excel_bytes,
    ))
}

async fn download_kss_pdf(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id).bind(user_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    // Strategy 1: Try pre-generated S3 file
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT s3_key FROM reports WHERE drawing_id = $1 AND format = 'kss_pdf' ORDER BY generated_at DESC LIMIT 1",
    )
    .bind(drawing_id)
    .fetch_optional(&state.db)
    .await?;

    if let Some((s3_key,)) = row {
        if let Ok(data) = download_s3_with_retry(&state.s3, &state.s3_bucket, &s3_key).await {
            return Ok((
                [(header::CONTENT_TYPE, "application/pdf"),
                 (header::CONTENT_DISPOSITION, "attachment; filename=\"kss-report.pdf\"")],
                data,
            ));
        }
    }

    // Strategy 2: On-demand generation from kss_reports JSONB
    let report_row: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT report_data FROM kss_reports WHERE drawing_id = $1 ORDER BY generated_at DESC LIMIT 1",
    )
    .bind(drawing_id)
    .fetch_optional(&state.db)
    .await?;

    let (report_data,) = report_row
        .ok_or_else(|| ApiError::NotFound("No KSS report found for this drawing".into()))?;

    let sectioned: kcc_core::kss::types::SectionedKssReport = serde_json::from_value(report_data)
        .map_err(|e| ApiError::Internal(format!("Failed to deserialize KSS report: {e}")))?;

    // Convert sectioned to flat KssReport for PDF generator
    let flat_report = kcc_core::kss::types::KssReport {
        drawing_name: sectioned.project_name.clone(),
        generated_at: sectioned.generated_at.clone(),
        items: sectioned.sections.iter().flat_map(|s| s.items.clone()).collect(),
        totals: kcc_core::kss::types::KssReport::compute_totals(
            &sectioned.sections.iter().flat_map(|s| s.items.clone()).collect::<Vec<_>>(),
        ),
    };

    let pdf_bytes = kcc_report::kss_pdf::generate_kss_pdf(&flat_report)
        .map_err(|e| ApiError::Internal(format!("PDF generation failed: {e}")))?;

    Ok((
        [(header::CONTENT_TYPE, "application/pdf"),
         (header::CONTENT_DISPOSITION, "attachment; filename=\"kss-report.pdf\"")],
        pdf_bytes,
    ))
}

/// Get KSS report data as structured JSON for the frontend table.
async fn get_kss_data(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify ownership
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id).bind(user_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    // Load from kss_reports table. We pull the full cost ladder columns so
    // the UI reads the same numbers the audit trail reads — no more "UI says
    // 65,685 лв, audit says 49,761 лв" drift.
    use sqlx::Row as _;
    let row_opt = sqlx::query(
        "SELECT report_data, ai_enhanced,
                subtotal_lv, vat_lv, total_with_vat_lv,
                smr_subtotal_lv, contingency_lv, delivery_storage_lv, profit_lv,
                pre_vat_total_lv, final_total_lv,
                item_count, generated_at, status
         FROM kss_reports WHERE drawing_id = $1
         ORDER BY generated_at DESC LIMIT 1",
    )
    .bind(drawing_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
    let row = row_opt.ok_or_else(|| ApiError::NotFound("KSS report not generated yet".into()))?;

    let mut report_data: serde_json::Value = row.try_get("report_data").unwrap_or_default();
    let ai_enhanced: bool = row.try_get("ai_enhanced").unwrap_or(false);
    let subtotal: Option<f64> = row.try_get("subtotal_lv").ok();
    let vat: Option<f64> = row.try_get("vat_lv").ok();
    let total: Option<f64> = row.try_get("total_with_vat_lv").ok();
    let smr_subtotal: Option<f64> = row.try_get("smr_subtotal_lv").ok();
    let contingency_lv: Option<f64> = row.try_get("contingency_lv").ok();
    let delivery_storage_lv: Option<f64> = row.try_get("delivery_storage_lv").ok();
    let profit_lv: Option<f64> = row.try_get("profit_lv").ok();
    let pre_vat_total_lv: Option<f64> = row.try_get("pre_vat_total_lv").ok();
    let final_total_lv: Option<f64> = row.try_get("final_total_lv").ok();
    let items: Option<i32> = row.try_get("item_count").ok();
    let generated_at: chrono::DateTime<chrono::Utc> = row
        .try_get("generated_at")
        .unwrap_or_else(|_| chrono::Utc::now());
    let report_status: Option<String> = row.try_get("status").ok();

    // Get corrections count
    let corrections: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM kss_corrections WHERE drawing_id = $1 AND user_id = $2"
    )
    .bind(drawing_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Get suggestions (low-confidence AI items awaiting review)
    let report_id_row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM kss_reports WHERE drawing_id = $1 ORDER BY generated_at DESC LIMIT 1"
    ).bind(drawing_id).fetch_optional(&state.db).await.ok().flatten();

    // Annotate each item in report_data with its DB id, structure_id, and
    // structure_label so the frontend can render per-module tabs and reference
    // specific rows on save. Match by (sek_code, description) within a report.
    if let Some((report_id,)) = report_id_row {
        let id_rows: Vec<(Uuid, String, String, Option<Uuid>, Option<String>)> = sqlx::query_as(
            "SELECT id, sek_code, description, structure_id, structure_label
             FROM kss_line_items WHERE report_id = $1"
        ).bind(report_id).fetch_all(&state.db).await.unwrap_or_default();
        let id_map: std::collections::HashMap<(String, String), (Uuid, Option<Uuid>, Option<String>)> = id_rows
            .into_iter()
            .map(|(id, sek, desc, sid, slabel)| ((sek, desc), (id, sid, slabel)))
            .collect();
        if let Some(sections) = report_data.get_mut("sections").and_then(|v| v.as_array_mut()) {
            for sec in sections {
                if let Some(items) = sec.get_mut("items").and_then(|v| v.as_array_mut()) {
                    for item in items {
                        let sek = item.get("sek_code").and_then(|s| s.as_str()).unwrap_or("").to_string();
                        let desc = item.get("description").and_then(|s| s.as_str()).unwrap_or("").to_string();
                        if let Some((id, sid, slabel)) = id_map.get(&(sek, desc)) {
                            if let Some(obj) = item.as_object_mut() {
                                obj.insert("id".into(), serde_json::Value::String(id.to_string()));
                                if let Some(sid) = sid {
                                    obj.insert("structure_id".into(), serde_json::Value::String(sid.to_string()));
                                }
                                if let Some(label) = slabel {
                                    obj.insert("structure_label".into(), serde_json::Value::String(label.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Detected structures (modules) for this drawing — bbox + per-module
    // subtotal. The frontend renders a tab per entry. Empty list when the
    // drawing is single-module.
    let structures: Vec<serde_json::Value> = if let Some((report_id,)) = report_id_row {
        let rows: Vec<(Uuid, i32, String, f64, f64, f64, f64)> = sqlx::query_as(
            "SELECT id, structure_index, label, bbox_min_x, bbox_min_y, bbox_max_x, bbox_max_y
             FROM drawing_structures WHERE drawing_id = $1 ORDER BY structure_index ASC"
        ).bind(drawing_id).fetch_all(&state.db).await.unwrap_or_default();
        let mut out = Vec::with_capacity(rows.len());
        for (sid, idx, label, x0, y0, x1, y1) in rows {
            let subtotal: Option<f64> = sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_lv), 0)::float8 FROM kss_line_items
                 WHERE report_id = $1 AND structure_id = $2",
            ).bind(report_id).bind(sid).fetch_one(&state.db).await.ok();
            let line_count: Option<i64> = sqlx::query_scalar(
                "SELECT COUNT(*)::bigint FROM kss_line_items WHERE report_id = $1 AND structure_id = $2"
            ).bind(report_id).bind(sid).fetch_one(&state.db).await.ok();
            out.push(serde_json::json!({
                "id": sid.to_string(),
                "index": idx,
                "label": label,
                "bbox": [x0, y0, x1, y1],
                "subtotal_lv": subtotal.unwrap_or(0.0),
                "line_count": line_count.unwrap_or(0),
            }));
        }
        out
    } else {
        Vec::new()
    };

    let suggestions = if let Some((report_id,)) = report_id_row {
        // Include rows flagged by confidence < 0.7 OR by needs_review = true.
        // Using `sqlx::query` + try_get lets us avoid a 17-field tuple literal.
        use sqlx::Row;
        let rows = sqlx::query(
            "SELECT id, sek_code, description, unit, quantity, unit_price_lv, total_lv,
                    labor_price, material_price, confidence, reasoning, provenance,
                    source_entity_id, source_layer, centroid_x, centroid_y,
                    extraction_method, geometry_confidence, needs_review
             FROM kss_line_items
             WHERE report_id = $1
               AND (confidence < 0.7 OR needs_review = true)
               AND (suggestion_status IS NULL OR suggestion_status = 'pending')
             ORDER BY needs_review DESC, confidence ASC",
        )
        .bind(report_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        rows.iter().map(|r| {
            serde_json::json!({
                "id":             r.try_get::<Uuid, _>("id").unwrap_or_default(),
                "sek_code":       r.try_get::<String, _>("sek_code").unwrap_or_default(),
                "description":    r.try_get::<String, _>("description").unwrap_or_default(),
                "unit":           r.try_get::<String, _>("unit").unwrap_or_default(),
                "quantity":       r.try_get::<f64, _>("quantity").unwrap_or(0.0),
                "unit_price_lv":  r.try_get::<f64, _>("unit_price_lv").unwrap_or(0.0),
                "total_lv":       r.try_get::<f64, _>("total_lv").unwrap_or(0.0),
                "labor_price":    r.try_get::<Option<f64>, _>("labor_price").unwrap_or(None),
                "material_price": r.try_get::<Option<f64>, _>("material_price").unwrap_or(None),
                "confidence":     r.try_get::<f64, _>("confidence").unwrap_or(0.0),
                "reasoning":      r.try_get::<Option<String>, _>("reasoning").unwrap_or(None),
                "provenance":     r.try_get::<Option<String>, _>("provenance").unwrap_or(None),
                "source_entity_id":     r.try_get::<Option<String>, _>("source_entity_id").unwrap_or(None),
                "source_layer":         r.try_get::<Option<String>, _>("source_layer").unwrap_or(None),
                "centroid_x":           r.try_get::<Option<f64>, _>("centroid_x").unwrap_or(None),
                "centroid_y":           r.try_get::<Option<f64>, _>("centroid_y").unwrap_or(None),
                "extraction_method":    r.try_get::<Option<String>, _>("extraction_method").unwrap_or(None),
                "geometry_confidence":  r.try_get::<Option<f64>, _>("geometry_confidence").unwrap_or(None),
                "needs_review":         r.try_get::<Option<bool>, _>("needs_review").unwrap_or(Some(false)).unwrap_or(false),
            })
        }).collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    Ok(Json(serde_json::json!({
        "report": report_data,
        "ai_enhanced": ai_enhanced,
        "subtotal_lv": subtotal,
        "vat_lv": vat,
        "total_with_vat_lv": total,
        // Canonical cost ladder — UI reads these; no more on-the-fly
        // recomputation that could drift from the DB.
        "cost_ladder": {
            "smr_subtotal":      smr_subtotal.or(subtotal),
            "contingency":       contingency_lv,
            "delivery_storage":  delivery_storage_lv,
            "profit":            profit_lv,
            "pre_vat_total":     pre_vat_total_lv.or(subtotal),
            "vat":               vat,
            "final_total":       final_total_lv.or(total),
        },
        "item_count": items,
        "generated_at": generated_at.to_rfc3339(),
        "corrections_count": corrections.0,
        "suggestions": suggestions,
        "structures": structures,
        "status": report_status.unwrap_or_else(|| "final".to_string()),
    })))
}

// ═══════════════════════════════════════════════════════════
// AI KSS Endpoints — Redis-backed during review, Postgres for final
// ═══════════════════════════════════════════════════════════

/// Trigger Phase 1: Perplexity price research → results in Redis.
async fn trigger_ai_kss_research(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let job_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    // Create job record
    sqlx::query("INSERT INTO jobs (id, drawing_id, status, progress) VALUES ($1, $2, 'queued', 0)")
        .bind(job_id).bind(drawing_id)
        .execute(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Create AI KSS session record
    sqlx::query("INSERT INTO ai_kss_sessions (id, drawing_id, user_id, job_id, status) VALUES ($1, $2, $3, $4, 'researching') ON CONFLICT (drawing_id) DO UPDATE SET status = 'researching', job_id = $4, updated_at = now()")
        .bind(session_id).bind(drawing_id).bind(user_id).bind(job_id)
        .execute(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Enqueue to Redis
    let job_data = serde_json::json!({
        "job_id": job_id,
        "drawing_id": drawing_id,
        "user_id": user_id,
        "session_id": session_id,
        "phase": "research",
    });

    let mut conn = state.redis.lock().await;
    redis::cmd("LPUSH").arg("kcc:ai-kss-jobs").arg(job_data.to_string())
        .query_async::<()>(&mut *conn).await
        .map_err(|e| ApiError::Internal(format!("Redis error: {e}")))?;

    Ok(Json(serde_json::json!({
        "job_id": job_id,
        "session_id": session_id,
    })))
}

/// Get AI KSS session status from Redis.
async fn get_ai_kss_status(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Find session ID from Postgres
    let session: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, status FROM ai_kss_sessions WHERE drawing_id = $1 LIMIT 1"
    ).bind(drawing_id).fetch_optional(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let (session_id, db_status) = session
        .ok_or_else(|| ApiError::NotFound("No AI KSS session found".into()))?;

    // Read real-time status from Redis
    let mut conn = state.redis.lock().await;
    let redis_status: Option<String> = redis::cmd("GET")
        .arg(format!("kcc:ai:{}:status", session_id))
        .query_async(&mut *conn).await.ok();
    let progress: Option<String> = redis::cmd("GET")
        .arg(format!("kcc:ai:{}:progress", session_id))
        .query_async(&mut *conn).await.ok();
    let model: Option<String> = redis::cmd("GET")
        .arg(format!("kcc:ai:{}:model", session_id))
        .query_async(&mut *conn).await.ok();
    let error: Option<String> = redis::cmd("GET")
        .arg(format!("kcc:ai:{}:error", session_id))
        .query_async(&mut *conn).await.ok();

    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "status": redis_status.unwrap_or(db_status),
        "progress": progress.and_then(|p| p.parse::<i32>().ok()).unwrap_or(0),
        "model": model,
        "error": error,
    })))
}

/// Get research items from Redis (during review phase).
async fn get_ai_kss_research_items(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let session: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM ai_kss_sessions WHERE drawing_id = $1 LIMIT 1"
    ).bind(drawing_id).fetch_optional(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let (session_id,) = session
        .ok_or_else(|| ApiError::NotFound("No AI KSS session found".into()))?;

    let mut conn = state.redis.lock().await;

    // Get ordered item IDs from sorted set
    let item_ids: Vec<String> = redis::cmd("ZRANGE")
        .arg(format!("kcc:ai:{}:items", session_id))
        .arg(0i64).arg(-1i64)
        .query_async(&mut *conn).await
        .map_err(|e| ApiError::Internal(format!("Redis error: {e}")))?;

    let mut items = Vec::new();
    for item_id in &item_ids {
        let fields: std::collections::HashMap<String, String> = redis::cmd("HGETALL")
            .arg(format!("kcc:ai:{}:item:{}", session_id, item_id))
            .query_async(&mut *conn).await
            .map_err(|e| ApiError::Internal(format!("Redis error: {e}")))?;

        if fields.is_empty() { continue; }

        let mat = fields.get("material_price_lv").and_then(|v| v.parse::<f64>().ok());
        let lab = fields.get("labor_price_lv").and_then(|v| v.parse::<f64>().ok());
        let total = fields.get("price_lv").and_then(|v| v.parse::<f64>().ok())
            .or_else(|| match (mat, lab) {
                (Some(m), Some(l)) => Some(m + l),
                _ => None,
            });
        items.push(serde_json::json!({
            "id": item_id,
            "sek_group": fields.get("sek_group").unwrap_or(&String::new()),
            "sek_code": fields.get("sek_code").unwrap_or(&String::new()),
            "description": fields.get("description").unwrap_or(&String::new()),
            "unit": fields.get("unit").unwrap_or(&"М2".to_string()),
            "material_price_lv": mat,
            "labor_price_lv": lab,
            "price_lv": total,
            "price_min_lv": fields.get("price_min_lv").and_then(|v| v.parse::<f64>().ok()),
            "price_max_lv": fields.get("price_max_lv").and_then(|v| v.parse::<f64>().ok()),
            "source_url": fields.get("source_url").unwrap_or(&String::new()),
            "notes": fields.get("notes").unwrap_or(&String::new()),
            "confidence": fields.get("confidence").and_then(|v| v.parse::<f64>().ok()),
            "approved": fields.get("approved").map(|v| v == "true").unwrap_or(true),
            "edited": fields.get("edited").map(|v| v == "true").unwrap_or(false),
        }));
    }

    Ok(Json(items))
}

/// Update a single research item field in Redis (during review).
async fn update_ai_kss_item(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path((drawing_id, item_id)): Path<(Uuid, String)>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let session: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM ai_kss_sessions WHERE drawing_id = $1 LIMIT 1"
    ).bind(drawing_id).fetch_optional(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let (session_id,) = session
        .ok_or_else(|| ApiError::NotFound("No AI KSS session found".into()))?;

    let mut conn = state.redis.lock().await;
    let key = format!("kcc:ai:{}:item:{}", session_id, item_id);

    // Update individual fields from request body
    if let Some(obj) = body.as_object() {
        for (field, value) in obj {
            let val_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => continue,
            };
            redis::cmd("HSET").arg(&key).arg(field).arg(&val_str)
                .query_async::<()>(&mut *conn).await
                .map_err(|e| ApiError::Internal(format!("Redis error: {e}")))?;
        }
        // Mark as edited
        redis::cmd("HSET").arg(&key).arg("edited").arg("true")
            .query_async::<()>(&mut *conn).await
            .map_err(|e| ApiError::Internal(format!("Redis error: {e}")))?;
    }

    Ok(Json(serde_json::json!({"ok": true})))
}

/// Trigger Phase 3: Opus generation from reviewed Redis data.
#[derive(serde::Deserialize, Default)]
struct GenerateBody {
    /// "ai" (default) | "rag" | "hybrid". Drives the worker dispatch.
    #[serde(default)]
    mode: Option<String>,
}

async fn trigger_ai_kss_generation(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
    body: Option<Json<GenerateBody>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mode = body
        .and_then(|Json(b)| b.mode)
        .map(|s| s.to_ascii_lowercase())
        .filter(|s| matches!(s.as_str(), "ai" | "rag" | "hybrid"))
        .unwrap_or_else(|| "ai".to_string());

    let session: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM ai_kss_sessions WHERE drawing_id = $1 LIMIT 1"
    ).bind(drawing_id).fetch_optional(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let (session_id,) = session
        .ok_or_else(|| ApiError::NotFound("No AI KSS session — run research first".into()))?;

    let job_id = Uuid::new_v4();

    sqlx::query("INSERT INTO jobs (id, drawing_id, status, progress) VALUES ($1, $2, 'queued', 0)")
        .bind(job_id).bind(drawing_id)
        .execute(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    sqlx::query("UPDATE ai_kss_sessions SET status = 'generating', job_id = $1, updated_at = now() WHERE id = $2")
        .bind(job_id).bind(session_id)
        .execute(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let job_data = serde_json::json!({
        "job_id": job_id,
        "drawing_id": drawing_id,
        "user_id": user_id,
        "session_id": session_id,
        "phase": "generate",
        "mode": mode,
    });

    let mut conn = state.redis.lock().await;
    redis::cmd("LPUSH").arg("kcc:ai-kss-jobs").arg(job_data.to_string())
        .query_async::<()>(&mut *conn).await
        .map_err(|e| ApiError::Internal(format!("Redis error: {e}")))?;

    Ok(Json(serde_json::json!({
        "job_id": job_id,
        "session_id": session_id,
        "mode": mode,
    })))
}

/// Get KSS audit trail for a drawing (DEV/USER debug log).
async fn get_kss_audit_trail(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify ownership
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM drawings WHERE id = $1 AND user_id = $2)"
    )
    .bind(drawing_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    if !exists {
        return Err(ApiError::NotFound("Drawing not found".into()));
    }

    let rows: Vec<(uuid::Uuid, String, i64, i32, i32, Option<f64>, serde_json::Value, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT id, pipeline_mode, total_duration_ms, total_warnings, total_errors, overall_confidence, audit_data, user_summary
         FROM kss_audit_trails
         WHERE drawing_id = $1
         ORDER BY created_at DESC
         LIMIT 10"
    )
    .bind(drawing_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let audits: Vec<serde_json::Value> = rows.into_iter().map(|(id, mode, dur, warns, errs, conf, data, summary)| {
        serde_json::json!({
            "id": id,
            "pipeline_mode": mode,
            "total_duration_ms": dur,
            "total_warnings": warns,
            "total_errors": errs,
            "overall_confidence": conf,
            "audit_data": data,
            "user_summary": summary,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "audits": audits })))
}

// ═══════════════════════════════════════════════════════════
// Suggestions: Accept/Reject AI-generated low-confidence items
// ═══════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
struct AcceptSuggestionBody {
    edited_sek_code: Option<String>,
    edited_description: Option<String>,
    edited_quantity: Option<f64>,
    edited_unit_price: Option<f64>,
}

async fn accept_suggestion(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path((drawing_id, item_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<AcceptSuggestionBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify ownership
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id).bind(user_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    // Load the suggestion item
    let item: Option<(String, String, String, f64, f64, f64, Option<f64>, Option<f64>, Uuid)> = sqlx::query_as(
        "SELECT sek_code, description, unit, quantity, unit_price_lv, total_lv, labor_price, material_price, report_id
         FROM kss_line_items WHERE id = $1"
    ).bind(item_id).fetch_optional(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let (mut sek_code, mut description, unit, mut quantity, mut unit_price, _total, labor, material, report_id) =
        item.ok_or_else(|| ApiError::NotFound("Suggestion item not found".into()))?;

    // Apply user edits
    if let Some(ref code) = body.edited_sek_code { sek_code = code.clone(); }
    if let Some(ref desc) = body.edited_description { description = desc.clone(); }
    if let Some(qty) = body.edited_quantity { quantity = qty; }
    if let Some(up) = body.edited_unit_price { unit_price = up; }
    let total_lv = quantity * unit_price;

    // Update the item: accepted, high confidence
    sqlx::query(
        "UPDATE kss_line_items SET suggestion_status = 'accepted', confidence = 1.0, provenance = 'user_accepted',
         sek_code = $2, description = $3, unit = $4, quantity = $5, unit_price_lv = $6, total_lv = $7
         WHERE id = $1"
    )
    .bind(item_id).bind(&sek_code).bind(&description).bind(&unit)
    .bind(quantity).bind(unit_price).bind(total_lv)
    .execute(&state.db).await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Add to report_data JSONB
    let report_row: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT report_data FROM kss_reports WHERE id = $1"
    ).bind(report_id).fetch_optional(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    if let Some((report_data,)) = report_row {
        if let Ok(mut sectioned) = serde_json::from_value::<kcc_core::kss::types::SectionedKssReport>(report_data) {
            // Find or create the target section
            let sek_group = extract_sek_group_from_code(&sek_code);
            let section = sectioned.sections.iter_mut().find(|s| s.sek_group == sek_group);

            let new_item = kcc_core::kss::types::KssLineItem {
                item_no: 0, // will be renumbered
                sek_code: sek_code.clone(),
                description: description.clone(),
                unit: unit.clone(),
                quantity,
                labor_price: labor.unwrap_or(0.0),
                material_price: material.unwrap_or(0.0),
                mechanization_price: 0.0,
                overhead_price: 0.0,
                total_price: total_lv,
                confidence: 1.0,
                reasoning: "User accepted from AI suggestions".into(),
                provenance: "user_accepted".into(),
                ..Default::default()
            };

            if let Some(sec) = section {
                // Dedup: check if item already exists in section by description
                let existing = sec.items.iter_mut().find(|i| i.description == description);
                if let Some(existing_item) = existing {
                    // Update existing item instead of duplicating
                    existing_item.confidence = 1.0;
                    existing_item.provenance = "user_accepted".to_string();
                    existing_item.quantity = quantity;
                    existing_item.total_price = total_lv;
                    existing_item.labor_price = labor.unwrap_or(existing_item.labor_price);
                    existing_item.material_price = material.unwrap_or(existing_item.material_price);
                } else {
                    sec.items.push(new_item);
                }
                for (i, item) in sec.items.iter_mut().enumerate() {
                    item.item_no = i + 1;
                }
                sec.section_total_bgn = sec.items.iter().map(|i| i.total_price).sum();
            } else {
                // Create new section
                sectioned.sections.push(kcc_core::kss::types::KssSection {
                    number: "—".into(),
                    title_bg: format!("ДОБАВЕНИ ({})", sek_group),
                    sek_group: sek_group.clone(),
                    items: vec![kcc_core::kss::types::KssLineItem { item_no: 1, ..new_item }],
                    section_total_bgn: total_lv,
                });
            }

            // Recalculate + persist the FULL cost ladder (subtotal + markups
            // + VAT + final). Prior code only touched subtotal/vat/total_lv,
            // leaving the ladder columns stale — user saw no price change
            // after accepting suggestions because the UI reads the ladder.
            recompute_and_persist_ladder(&state.db, report_id, user_id, &mut sectioned).await?;
        }
    }

    // DRM: record user acceptance as audit log entry
    let _ = sqlx::query(
        "INSERT INTO drm_audit_log (drawing_id, action, input_key, matched_sek_code, new_confidence)
         VALUES ($1, 'user_accepted_suggestion', $2, $3, 1.0)"
    ).bind(drawing_id).bind(&description).bind(&sek_code)
    .execute(&state.db).await;

    Ok(Json(serde_json::json!({ "status": "accepted", "item_id": item_id })))
}

async fn reject_suggestion(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path((drawing_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id).bind(user_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    sqlx::query("UPDATE kss_line_items SET suggestion_status = 'rejected' WHERE id = $1")
        .bind(item_id).execute(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(serde_json::json!({ "status": "rejected", "item_id": item_id })))
}

// ═══════════════════════════════════════════════════════════
// Add Item: User manually adds a new line item to the report
// ═══════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
struct AddItemBody {
    sek_code: String,
    description: String,
    unit: String,
    quantity: f64,
    unit_price_lv: f64,
}

async fn add_kss_item(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
    Json(body): Json<AddItemBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id).bind(user_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    let report_row: Option<(Uuid, serde_json::Value)> = sqlx::query_as(
        "SELECT id, report_data FROM kss_reports WHERE drawing_id = $1 ORDER BY generated_at DESC LIMIT 1"
    ).bind(drawing_id).fetch_optional(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let (report_id, report_data) = report_row
        .ok_or_else(|| ApiError::NotFound("No KSS report found".into()))?;

    let total_lv = body.quantity * body.unit_price_lv;
    let item_id = Uuid::new_v4();

    // Insert into kss_line_items
    let sek_group = extract_sek_group_from_code(&body.sek_code);
    sqlx::query(
        "INSERT INTO kss_line_items (id, report_id, section_number, section_title, item_no, sek_code, description, unit, quantity, unit_price_lv, total_lv, confidence, reasoning, provenance)
         VALUES ($1, $2, $3, '', 0, $4, $5, $6, $7, $8, $9, 1.0, 'User added manually', 'user_added')"
    )
    .bind(item_id).bind(report_id).bind(&sek_group)
    .bind(&body.sek_code).bind(&body.description).bind(&body.unit)
    .bind(body.quantity).bind(body.unit_price_lv).bind(total_lv)
    .execute(&state.db).await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Update report_data JSONB
    if let Ok(mut sectioned) = serde_json::from_value::<kcc_core::kss::types::SectionedKssReport>(report_data) {
        let new_item = kcc_core::kss::types::KssLineItem {
            item_no: 0,
            sek_code: body.sek_code.clone(),
            description: body.description.clone(),
            unit: body.unit.clone(),
            quantity: body.quantity,
            labor_price: 0.0,
            material_price: body.unit_price_lv,
            mechanization_price: 0.0,
            overhead_price: 0.0,
            total_price: total_lv,
            confidence: 1.0,
            reasoning: "User added manually".into(),
            provenance: "user_added".into(),
            ..Default::default()
        };

        let section = sectioned.sections.iter_mut().find(|s| s.sek_group == sek_group);
        if let Some(sec) = section {
            sec.items.push(new_item);
            for (i, item) in sec.items.iter_mut().enumerate() { item.item_no = i + 1; }
            sec.section_total_bgn = sec.items.iter().map(|i| i.total_price).sum();
        } else {
            sectioned.sections.push(kcc_core::kss::types::KssSection {
                number: "—".into(),
                title_bg: format!("ДОБАВЕНИ ({})", sek_group),
                sek_group: sek_group.clone(),
                items: vec![kcc_core::kss::types::KssLineItem { item_no: 1, ..new_item }],
                section_total_bgn: total_lv,
            });
        }

        // Recalculate + persist the FULL cost ladder — see accept_suggestion.
        recompute_and_persist_ladder(&state.db, report_id, user_id, &mut sectioned).await?;
    }

    // DRM: record as user-added item in audit log
    let _ = sqlx::query(
        "INSERT INTO drm_audit_log (drawing_id, action, input_key, matched_sek_code, new_confidence)
         VALUES ($1, 'user_added_item', $2, $3, 1.0)"
    ).bind(drawing_id).bind(&body.description).bind(&body.sek_code)
    .execute(&state.db).await;

    Ok(Json(serde_json::json!({ "status": "added", "item_id": item_id })))
}

/// Finalize a draft KSS report — removes rejected suggestions, recalculates totals, sets status='final'.
async fn finalize_kss(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id).bind(user_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    // Load the current report
    let row: Option<(Uuid, serde_json::Value)> = sqlx::query_as(
        "SELECT id, report_data FROM kss_reports WHERE drawing_id = $1 ORDER BY generated_at DESC LIMIT 1"
    ).bind(drawing_id).fetch_optional(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let (report_id, report_data) = row
        .ok_or_else(|| ApiError::NotFound("No KSS report found".into()))?;

    // Get rejected item descriptions to filter them out
    let rejected: Vec<(String,)> = sqlx::query_as(
        "SELECT description FROM kss_line_items WHERE report_id = $1 AND suggestion_status = 'rejected'"
    ).bind(report_id).fetch_all(&state.db).await.unwrap_or_default();

    let rejected_descs: std::collections::HashSet<String> = rejected.into_iter().map(|(d,)| d).collect();

    // Rebuild the report without rejected items
    if let Ok(mut sectioned) = serde_json::from_value::<kcc_core::kss::types::SectionedKssReport>(report_data) {
        for section in &mut sectioned.sections {
            section.items.retain(|item| !rejected_descs.contains(&item.description));
            for (i, item) in section.items.iter_mut().enumerate() {
                item.item_no = i + 1;
            }
            section.section_total_bgn = section.items.iter().map(|i| i.total_price).sum();
        }
        // Remove empty sections
        sectioned.sections.retain(|s| !s.items.is_empty());

        // Recalculate + persist the FULL cost ladder AND flip status to final.
        recompute_and_persist_ladder(&state.db, report_id, user_id, &mut sectioned).await?;
        sqlx::query("UPDATE kss_reports SET status = 'final' WHERE id = $1")
            .bind(report_id)
            .execute(&state.db)
            .await
            .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

        let item_count: i32 = sectioned.sections.iter().map(|s| s.items.len() as i32).sum();
        Ok(Json(serde_json::json!({
            "status": "finalized",
            "item_count": item_count,
            "subtotal_bgn": sectioned.cost_ladder.smr_subtotal,
            "total_with_vat_bgn": sectioned.cost_ladder.final_total,
        })))
    } else {
        Err(ApiError::Internal("Failed to parse report data".into()))
    }
}

fn extract_sek_group_from_code(code: &str) -> String {
    if let Some(dot_pos) = code.find('.') {
        code[..dot_pos].to_string()
    } else {
        code.trim().to_string()
    }
}

/// Pull the user's pricing defaults (markups + VAT) from the DB. Returns the
/// canonical fallback values when no row exists — same defaults the worker
/// uses in `pricing_defaults::PricingDefaults::load_for_user`.
async fn load_user_overheads(
    db: &sqlx::PgPool,
    user_id: Uuid,
) -> (kcc_core::kss::types::KssOverheads, f64) {
    use sqlx::Row as _;
    let row = sqlx::query(
        "SELECT
            contingency_pct::float8    AS contingency_pct,
            dr_materials_pct::float8   AS dr_materials_pct,
            profit_pct::float8         AS profit_pct,
            vat_rate_pct::float8       AS vat_rate_pct
         FROM pricing_defaults WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    let (contingency_pct, dr_materials_pct, profit_pct, vat_rate_pct) = match row {
        Some(r) => (
            r.try_get::<f64, _>("contingency_pct").unwrap_or(10.0),
            r.try_get::<f64, _>("dr_materials_pct").unwrap_or(12.0),
            r.try_get::<f64, _>("profit_pct").unwrap_or(10.0),
            r.try_get::<f64, _>("vat_rate_pct").unwrap_or(20.0),
        ),
        None => (10.0, 12.0, 10.0, 20.0),
    };

    (
        kcc_core::kss::types::KssOverheads {
            contingency_pct,
            delivery_storage_pct: dr_materials_pct,
            profit_pct,
        },
        vat_rate_pct / 100.0,
    )
}

/// After ANY mutation to a KSS report (accept/reject suggestion, add line,
/// finalize), recompute the full cost ladder from the current rows and write
/// every ladder column atomically. This is the single source of truth that
/// `get_kss_data` reads — without it, the UI showed stale pre-markup totals.
async fn recompute_and_persist_ladder(
    db: &sqlx::PgPool,
    report_id: Uuid,
    user_id: Uuid,
    sectioned: &mut kcc_core::kss::types::SectionedKssReport,
) -> Result<(), ApiError> {
    let (overheads, vat_rate) = load_user_overheads(db, user_id).await;
    sectioned.overheads = overheads;
    sectioned.vat_rate = vat_rate;

    // Recompute from the current sections (items may have changed).
    let smr: f64 = sectioned.sections.iter().map(|s| s.section_total_bgn).sum();
    let ladder = kcc_core::kss::types::KssCostLadder::compute(smr, overheads, vat_rate);
    sectioned.subtotal_bgn = ladder.smr_subtotal;
    sectioned.vat_bgn = ladder.vat;
    sectioned.total_with_vat_bgn = ladder.final_total;
    sectioned.cost_ladder = ladder;

    let item_count: i32 = sectioned.sections.iter().map(|s| s.items.len() as i32).sum();
    sqlx::query(
        "UPDATE kss_reports SET
            report_data           = $1,
            subtotal_lv           = $2,
            vat_lv                = $3,
            total_with_vat_lv     = $4,
            item_count            = $5,
            smr_subtotal_lv       = $6,
            contingency_lv        = $7,
            delivery_storage_lv   = $8,
            profit_lv             = $9,
            pre_vat_total_lv      = $10,
            final_total_lv        = $11
         WHERE id = $12",
    )
    .bind(serde_json::to_value(sectioned).unwrap_or_default())
    .bind(ladder.smr_subtotal)
    .bind(ladder.vat)
    .bind(ladder.final_total)
    .bind(item_count)
    .bind(ladder.smr_subtotal)
    .bind(ladder.contingency)
    .bind(ladder.delivery_storage)
    .bind(ladder.profit)
    .bind(ladder.pre_vat_total)
    .bind(ladder.final_total)
    .bind(report_id)
    .execute(db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error recomputing ladder: {e}")))?;

    Ok(())
}

#[derive(serde::Deserialize)]
struct RenameStructureBody {
    label: String,
}

/// Rename a detected module. The drawing's title text rarely matches the way
/// engineers name modules in their KSS; let the user override "Module 1" with
/// "ТАСОС" inline. Updates `drawing_structures.label` AND propagates to every
/// `kss_line_items.structure_label` for that structure so the frontend tab
/// strip and the Excel export both reflect the new name without re-running
/// the AI pipeline.
async fn rename_structure(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path((drawing_id, structure_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<RenameStructureBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let trimmed = body.label.trim();
    if trimmed.is_empty() || trimmed.len() > 80 {
        return Err(ApiError::BadRequest("Label must be 1-80 chars".into()));
    }

    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    let updated = sqlx::query(
        "UPDATE drawing_structures SET label = $1
         WHERE id = $2 AND drawing_id = $3",
    )
    .bind(trimmed)
    .bind(structure_id)
    .bind(drawing_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    if updated.rows_affected() == 0 {
        return Err(ApiError::NotFound("Structure not found".into()));
    }

    sqlx::query(
        "UPDATE kss_line_items SET structure_label = $1 WHERE structure_id = $2",
    )
    .bind(trimmed)
    .bind(structure_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(serde_json::json!({ "label": trimmed })))
}

#[derive(serde::Deserialize)]
struct MergeStructuresBody {
    /// Module ids to merge INTO `target_id`. Their dim/annotation/line-item
    /// rows get reassigned and the rows themselves are deleted.
    source_ids: Vec<Uuid>,
    target_id: Uuid,
}

/// Merge N detected modules into a single one. Used when the auto-detector
/// over-split (e.g. one large floor plan with an internal courtyard appears
/// as two clusters that should really be one). The user keeps the target
/// module's label and bbox is expanded to cover everything.
async fn merge_structures(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
    Json(body): Json<MergeStructuresBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if body.source_ids.is_empty() {
        return Err(ApiError::BadRequest("source_ids must be non-empty".into()));
    }
    if body.source_ids.contains(&body.target_id) {
        return Err(ApiError::BadRequest(
            "target_id must not be in source_ids".into(),
        ));
    }

    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    // Expand target bbox to cover sources.
    let target: Option<(f64, f64, f64, f64)> = sqlx::query_as(
        "SELECT bbox_min_x, bbox_min_y, bbox_max_x, bbox_max_y
         FROM drawing_structures WHERE id = $1 AND drawing_id = $2",
    )
    .bind(body.target_id)
    .bind(drawing_id)
    .fetch_optional(&state.db)
    .await?;
    let (mut tx0, mut ty0, mut tx1, mut ty1) =
        target.ok_or_else(|| ApiError::NotFound("Target structure not found".into()))?;

    let sources: Vec<(f64, f64, f64, f64, String)> = sqlx::query_as(
        "SELECT bbox_min_x, bbox_min_y, bbox_max_x, bbox_max_y, label
         FROM drawing_structures
         WHERE drawing_id = $1 AND id = ANY($2::uuid[])",
    )
    .bind(drawing_id)
    .bind(&body.source_ids)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
    if sources.len() != body.source_ids.len() {
        return Err(ApiError::BadRequest(
            "One or more source_ids did not belong to this drawing".into(),
        ));
    }
    for (sx0, sy0, sx1, sy1, _) in &sources {
        tx0 = tx0.min(*sx0);
        ty0 = ty0.min(*sy0);
        tx1 = tx1.max(*sx1);
        ty1 = ty1.max(*sy1);
    }

    // Reassign all rows referencing the sources to the target.
    for tbl in ["drawing_layers", "drawing_blocks", "drawing_dimensions", "drawing_annotations"] {
        let q = format!(
            "UPDATE {tbl} SET structure_id = $1
             WHERE structure_id = ANY($2::uuid[])"
        );
        sqlx::query(&q)
            .bind(body.target_id)
            .bind(&body.source_ids)
            .execute(&state.db)
            .await
            .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;
    }
    sqlx::query(
        "UPDATE kss_line_items SET structure_id = $1, structure_label = (
             SELECT label FROM drawing_structures WHERE id = $1
         )
         WHERE structure_id = ANY($2::uuid[])",
    )
    .bind(body.target_id)
    .bind(&body.source_ids)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Update target bbox.
    sqlx::query(
        "UPDATE drawing_structures
         SET bbox_min_x = $1, bbox_min_y = $2, bbox_max_x = $3, bbox_max_y = $4
         WHERE id = $5",
    )
    .bind(tx0)
    .bind(ty0)
    .bind(tx1)
    .bind(ty1)
    .bind(body.target_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    // Delete source structures.
    sqlx::query(
        "DELETE FROM drawing_structures
         WHERE drawing_id = $1 AND id = ANY($2::uuid[])",
    )
    .bind(drawing_id)
    .bind(&body.source_ids)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(serde_json::json!({
        "merged_into": body.target_id,
        "removed": body.source_ids,
        "new_bbox": [tx0, ty0, tx1, ty1],
    })))
}

/// Delete a detected module entirely. Reassigns its line items to no
/// structure (NULL) so they appear under the recap-only view, and deletes
/// the structure row. Used to discard a false-positive cluster.
async fn delete_structure(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path((drawing_id, structure_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(drawing_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".into()))?;

    // ON DELETE SET NULL handles the layer/dim/annotation FKs; for KSS line
    // items we explicitly null both fields so the recap stays meaningful.
    sqlx::query(
        "UPDATE kss_line_items SET structure_id = NULL, structure_label = NULL
         WHERE structure_id = $1",
    )
    .bind(structure_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let result = sqlx::query(
        "DELETE FROM drawing_structures WHERE id = $1 AND drawing_id = $2",
    )
    .bind(structure_id)
    .bind(drawing_id)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Structure not found".into()));
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}
