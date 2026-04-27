use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn correction_routes() -> Router<AppState> {
    Router::new()
        .route("/reports/{drawing_id}/kss/corrections", post(submit_corrections))
        .route("/reports/{drawing_id}/kss/corrections", get(list_corrections))
        .route("/drm/stats", get(drm_stats))
}

// ── Submit corrections ──────────────────────────────────────

#[derive(Deserialize)]
struct CorrectionItem {
    /// Direct pointer to the kss_line_items row to update. When present, the
    /// endpoint writes corrected fields to the DB row deterministically.
    item_id: Option<Uuid>,
    original_sek_code: Option<String>,
    original_description: Option<String>,
    original_quantity: Option<f64>,
    original_unit: Option<String>,
    corrected_sek_code: Option<String>,
    corrected_description: Option<String>,
    corrected_quantity: Option<f64>,
    corrected_unit: Option<String>,
    /// Optional price overrides — user-approved numbers replace AI numbers as-is.
    corrected_material_price: Option<f64>,
    corrected_labor_price: Option<f64>,
    correction_type: String,
    source_layer: Option<String>,
    source_block: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct SubmitCorrectionsRequest {
    items: Vec<CorrectionItem>,
}

#[derive(Serialize)]
struct SubmitCorrectionsResponse {
    corrections_saved: usize,
    drm_artifacts_updated: usize,
}

async fn submit_corrections(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
    Json(body): Json<SubmitCorrectionsRequest>,
) -> Result<Json<SubmitCorrectionsResponse>, ApiError> {
    // Verify drawing belongs to user
    let exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM drawings WHERE id = $1 AND user_id = $2",
    )
    .bind(drawing_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    if exists.is_none() {
        return Err(ApiError::NotFound("Drawing not found".to_string()));
    }

    let mut corrections_saved = 0;
    let mut drm_updated = 0;
    let mut line_items_updated = 0;

    for item in &body.items {
        // (1) Deterministic mutation of the report row — the user's values are
        // written to kss_line_items exactly as typed. Zero AI involvement.
        if let Some(item_id) = item.item_id {
            // Pull current values so we only touch fields the user corrected.
            let cur: Option<(String, String, String, f64, f64, f64, Uuid)> = sqlx::query_as(
                "SELECT sek_code, description, unit, quantity, labor_price, material_price, report_id
                 FROM kss_line_items li
                 JOIN kss_reports r ON r.id = li.report_id
                 WHERE li.id = $1 AND r.drawing_id = $2"
            )
            .bind(item_id)
            .bind(drawing_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

            if let Some((cur_sek, cur_desc, cur_unit, cur_qty, cur_lab, cur_mat, report_id)) = cur {
                let new_sek = item.corrected_sek_code.clone().unwrap_or(cur_sek);
                let new_desc = item.corrected_description.clone().unwrap_or(cur_desc);
                let new_unit = item.corrected_unit.clone().unwrap_or(cur_unit);
                let new_qty = item.corrected_quantity.unwrap_or(cur_qty);
                let new_lab = item.corrected_labor_price.unwrap_or(cur_lab);
                let new_mat = item.corrected_material_price.unwrap_or(cur_mat);
                let new_unit_price = new_lab + new_mat;
                let new_total = new_qty * new_unit_price;

                sqlx::query(
                    "UPDATE kss_line_items SET
                        sek_code = $2, description = $3, unit = $4, quantity = $5,
                        labor_price = $6, material_price = $7,
                        unit_price_eur = $8, total_eur = $9,
                        provenance = 'user_correction', confidence = 1.0
                     WHERE id = $1"
                )
                .bind(item_id)
                .bind(&new_sek)
                .bind(&new_desc)
                .bind(&new_unit)
                .bind(new_qty)
                .bind(new_lab)
                .bind(new_mat)
                .bind(new_unit_price)
                .bind(new_total)
                .execute(&state.db)
                .await
                .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

                line_items_updated += 1;

                // Rebuild report_data JSONB from the authoritative line_items
                // so subsequent reads (Excel download, page refresh) see the
                // user's values. No AI re-invocation here.
                rebuild_report_snapshot(&state.db, report_id).await?;
            }
        }

        // (2) Audit record — unchanged, still useful for history + DRM.
        sqlx::query(
            "INSERT INTO kss_corrections (user_id, drawing_id, original_sek_code, original_description, original_quantity, original_unit, corrected_sek_code, corrected_description, corrected_quantity, corrected_unit, correction_type, source_layer, source_block, notes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        )
        .bind(user_id)
        .bind(drawing_id)
        .bind(&item.original_sek_code)
        .bind(&item.original_description)
        .bind(item.original_quantity)
        .bind(&item.original_unit)
        .bind(&item.corrected_sek_code)
        .bind(&item.corrected_description)
        .bind(item.corrected_quantity)
        .bind(&item.corrected_unit)
        .bind(&item.correction_type)
        .bind(&item.source_layer)
        .bind(&item.source_block)
        .bind(&item.notes)
        .execute(&state.db)
        .await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

        corrections_saved += 1;

        // Feed correction into DRM
        if let (Some(corrected_sek), Some(desc)) = (&item.corrected_sek_code, &item.corrected_description) {
            let input_key = item.source_layer.as_deref()
                .or(item.source_block.as_deref())
                .or(item.original_description.as_deref())
                .unwrap_or(desc);

            let artifact_type = if item.source_layer.is_some() {
                "layer_mapping"
            } else if item.source_block.is_some() {
                "block_mapping"
            } else {
                "feature_sek"
            };

            // Extract SEK group from code (e.g., "СЕК05.002" → "СЕК05")
            let sek_group = if let Some(dot) = corrected_sek.find('.') {
                &corrected_sek[..dot]
            } else {
                corrected_sek.as_str()
            };

            let unit = item.corrected_unit.as_deref().unwrap_or("М2");

            if let Err(e) = kcc_core::drm::recorder::record_user_correction(
                &state.db,
                user_id,
                drawing_id,
                artifact_type,
                input_key,
                corrected_sek,
                sek_group,
                desc,
                unit,
            ).await {
                tracing::warn!(error = %e, "Failed to record DRM correction");
            } else {
                drm_updated += 1;
            }
        }
    }

    tracing::info!(
        corrections_saved,
        line_items_updated,
        drm_artifacts_updated = drm_updated,
        "submit_corrections complete"
    );

    Ok(Json(SubmitCorrectionsResponse {
        corrections_saved,
        drm_artifacts_updated: drm_updated,
    }))
}

/// Recompute `kss_reports.report_data` JSONB from the authoritative
/// `kss_line_items` rows. Called after every direct update so downstream
/// consumers (page reloads, Excel export, audit trail) see the same truth.
async fn rebuild_report_snapshot(
    db: &sqlx::PgPool,
    report_id: Uuid,
) -> Result<(), ApiError> {
    use kcc_core::kss::types::{KssLineItem, SectionedKssReport};

    let rows: Vec<(String, String, String, f64, f64, f64, f64, f64, String, String)> = sqlx::query_as(
        "SELECT sek_code, description, unit, quantity, labor_price, material_price,
                COALESCE(unit_price_eur, 0), COALESCE(total_eur, 0),
                COALESCE(reasoning, ''), COALESCE(provenance, 'rule_based')
         FROM kss_line_items
         WHERE report_id = $1 AND (suggestion_status IS NULL OR suggestion_status != 'rejected')
         ORDER BY sek_code, description"
    )
    .bind(report_id)
    .fetch_all(db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let items: Vec<KssLineItem> = rows
        .into_iter()
        .enumerate()
        .map(|(i, (sek, desc, unit, qty, lab, mat, _up, total, reason, prov))| KssLineItem {
            item_no: i + 1,
            sek_code: sek,
            description: desc,
            unit,
            quantity: qty,
            labor_price: lab,
            material_price: mat,
            mechanization_price: 0.0,
            overhead_price: 0.0,
            total_price: total,
            confidence: 1.0,
            reasoning: reason,
            provenance: prov,
            ..Default::default()
        })
        .collect();

    let sectioned = SectionedKssReport::from_items(
        "",
        &chrono::Utc::now().to_rfc3339(),
        items,
        0.20,
    );

    let json = serde_json::to_value(&sectioned)
        .map_err(|e| ApiError::Internal(format!("Serialize error: {e}")))?;

    sqlx::query(
        "UPDATE kss_reports
         SET report_data = $1,
             subtotal_eur = $2,
             vat_eur = $3,
             total_with_vat_eur = $4,
             item_count = $5
         WHERE id = $6"
    )
    .bind(&json)
    .bind(sectioned.subtotal_eur)
    .bind(sectioned.vat_eur)
    .bind(sectioned.total_with_vat_eur)
    .bind(sectioned.sections.iter().map(|s| s.items.len() as i32).sum::<i32>())
    .bind(report_id)
    .execute(db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(())
}

// ── List corrections ────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct CorrectionRecord {
    id: Uuid,
    original_sek_code: Option<String>,
    original_description: Option<String>,
    corrected_sek_code: Option<String>,
    corrected_description: Option<String>,
    corrected_quantity: Option<f64>,
    corrected_unit: Option<String>,
    correction_type: String,
    source_layer: Option<String>,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn list_corrections(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(drawing_id): Path<Uuid>,
) -> Result<Json<Vec<CorrectionRecord>>, ApiError> {
    let corrections: Vec<CorrectionRecord> = sqlx::query_as(
        "SELECT id, original_sek_code, original_description, corrected_sek_code, corrected_description, corrected_quantity, corrected_unit, correction_type, source_layer, notes, created_at
         FROM kss_corrections WHERE user_id = $1 AND drawing_id = $2 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .bind(drawing_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(corrections))
}

// ── DRM Stats ───────────────────────────────────────────────

#[derive(Serialize)]
struct DrmStats {
    total_artifacts: i64,
    auto_generated: i64,
    user_corrected: i64,
    total_corrections: i64,
    avg_confidence: f64,
    top_confirmed: Vec<TopArtifact>,
}

#[derive(Serialize, sqlx::FromRow)]
struct TopArtifact {
    input_key: String,
    sek_code: Option<String>,
    times_confirmed: i32,
    confidence: f64,
    source: String,
}

async fn drm_stats(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<DrmStats>, ApiError> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM drm_artifacts WHERE user_id = $1")
        .bind(user_id).fetch_one(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let auto: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM drm_artifacts WHERE user_id = $1 AND source = 'auto'")
        .bind(user_id).fetch_one(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let corrected: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM drm_artifacts WHERE user_id = $1 AND source = 'user_correction'")
        .bind(user_id).fetch_one(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let corrections: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM kss_corrections WHERE user_id = $1")
        .bind(user_id).fetch_one(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let avg_conf: (Option<f64>,) = sqlx::query_as("SELECT AVG(confidence) FROM drm_artifacts WHERE user_id = $1")
        .bind(user_id).fetch_one(&state.db).await
        .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    let top: Vec<TopArtifact> = sqlx::query_as(
        "SELECT input_key, sek_code, times_confirmed, confidence, source FROM drm_artifacts WHERE user_id = $1 ORDER BY times_confirmed DESC LIMIT 10",
    )
    .bind(user_id).fetch_all(&state.db).await
    .map_err(|e| ApiError::Internal(format!("DB error: {e}")))?;

    Ok(Json(DrmStats {
        total_artifacts: total.0,
        auto_generated: auto.0,
        user_corrected: corrected.0,
        total_corrections: corrections.0,
        avg_confidence: avg_conf.0.unwrap_or(0.0),
        top_confirmed: top,
    }))
}
