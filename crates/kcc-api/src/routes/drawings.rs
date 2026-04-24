use axum::{
    Json, Router,
    extract::{Extension, Multipart, Path, State},
    routing::{get, post},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

type DrawingRow = (
    Uuid,
    String,
    String,
    Option<String>,
    Option<i32>,
    chrono::DateTime<chrono::Utc>,
);

pub fn drawing_routes() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload_drawing))
        .route("/", get(list_drawings))
        .route("/{id}", get(get_drawing).delete(delete_drawing))
        .route("/{id}/summary", get(get_drawing_summary))
}

#[derive(Serialize)]
struct UploadResponse {
    drawing_id: Uuid,
    job_id: Uuid,
    duplicate: bool,
}

#[derive(Serialize)]
struct DrawingResponse {
    id: Uuid,
    filename: String,
    original_format: String,
    units: Option<String>,
    entity_count: Option<i32>,
    created_at: String,
}

async fn upload_drawing(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, ApiError> {
    // Extract file from multipart
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .ok_or_else(|| ApiError::BadRequest("No file uploaded".to_string()))?;

    let filename = field.file_name().unwrap_or("upload.dxf").to_string();
    let data = field
        .bytes()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Validate extension
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    if ext != "dxf" && ext != "dwg" && ext != "pdf" {
        return Err(ApiError::BadRequest(
            "Only .dxf, .dwg, and .pdf files are accepted".to_string(),
        ));
    }

    // Reject DWG uploads if ODA File Converter is not available
    if ext == "dwg" && !state.dwg_conversion_available {
        return Err(ApiError::BadRequest(
            "DWG files require ODA File Converter which is not installed on this server. Please upload a DXF file instead.".to_string(),
        ));
    }

    // Compute SHA-256 hash for duplicate detection
    let file_hash = {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        format!("{:x}", hasher.finalize())
    };

    // Check for duplicate: same user + same file content
    let existing: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM drawings WHERE user_id = $1 AND file_hash = $2 LIMIT 1")
            .bind(user_id)
            .bind(&file_hash)
            .fetch_optional(&state.db)
            .await?;

    if let Some((existing_id,)) = existing {
        // Return the existing drawing — find its latest job
        let existing_job: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM jobs WHERE drawing_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(existing_id)
        .fetch_optional(&state.db)
        .await?;

        let job_id = existing_job.map(|(id,)| id).unwrap_or(existing_id);

        return Ok(Json(UploadResponse {
            drawing_id: existing_id,
            job_id,
            duplicate: true,
        }));
    }

    let drawing_id = Uuid::new_v4();
    let s3_key = format!("uploads/{drawing_id}/original.{ext}");

    // Upload to S3
    state
        .s3
        .put_object()
        .bucket(&state.s3_bucket)
        .key(&s3_key)
        .body(data.into())
        .send()
        .await
        .map_err(|e| {
            tracing::error!(
                error = ?e,
                bucket = %state.s3_bucket,
                key = %s3_key,
                "S3 upload failed — full error chain"
            );
            ApiError::StorageUnavailable(format!("Could not store file: {e}"))
        })?;

    // Insert drawing record with hash
    sqlx::query(
        "INSERT INTO drawings (id, user_id, filename, original_format, s3_key_original, file_hash) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(drawing_id)
    .bind(user_id)
    .bind(&filename)
    .bind(&ext)
    .bind(&s3_key)
    .bind(&file_hash)
    .execute(&state.db)
    .await?;

    // Create job
    let job_id = Uuid::new_v4();
    sqlx::query("INSERT INTO jobs (id, drawing_id) VALUES ($1, $2)")
        .bind(job_id)
        .bind(drawing_id)
        .execute(&state.db)
        .await?;

    // Enqueue job in Redis
    let job_data = serde_json::json!({
        "job_id": job_id,
        "drawing_id": drawing_id,
        "s3_key": s3_key,
    });

    {
        let mut redis = state.redis.lock().await;
        redis::cmd("LPUSH")
            .arg("kcc:jobs")
            .arg(serde_json::to_string(&job_data).unwrap())
            .exec_async(&mut *redis)
            .await
            .map_err(|e| ApiError::Internal(format!("Redis enqueue failed: {e}")))?;
    }

    Ok(Json(UploadResponse {
        drawing_id,
        job_id,
        duplicate: false,
    }))
}

async fn list_drawings(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<DrawingResponse>>, ApiError> {
    let rows: Vec<DrawingRow> =
        sqlx::query_as(
            "SELECT id, filename, original_format, units, entity_count, created_at FROM drawings WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&state.db)
        .await?;

    let drawings = rows
        .into_iter()
        .map(
            |(id, filename, fmt, units, count, created_at)| DrawingResponse {
                id,
                filename,
                original_format: fmt,
                units,
                entity_count: count,
                created_at: created_at.to_rfc3339(),
            },
        )
        .collect();

    Ok(Json(drawings))
}

async fn get_drawing(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<DrawingResponse>, ApiError> {
    let row: (Uuid, String, String, Option<String>, Option<i32>, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as(
            "SELECT id, filename, original_format, units, entity_count, created_at FROM drawings WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".to_string()))?;

    Ok(Json(DrawingResponse {
        id: row.0,
        filename: row.1,
        original_format: row.2,
        units: row.3,
        entity_count: row.4,
        created_at: row.5.to_rfc3339(),
    }))
}

async fn delete_drawing(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let result = sqlx::query("DELETE FROM drawings WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Drawing not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ── Drawing summary for overview page ───────────────────

async fn get_drawing_summary(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Drawing metadata
    let drawing: (Uuid, String, String, Option<String>, Option<i32>, bool, Option<f64>, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as(
            "SELECT id, filename, original_format, units, entity_count, kss_generated, kss_total_lv, created_at FROM drawings WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Drawing not found".to_string()))?;

    // Check if deep analysis exists
    let deep_analysis_report: Option<(String,)> = sqlx::query_as(
        "SELECT s3_key FROM reports WHERE drawing_id = $1 AND format = 'deep_analysis' LIMIT 1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    // Try to load deep analysis summary from S3
    let analysis_summary = if let Some((s3_key,)) = &deep_analysis_report {
        match state.s3.get_object().bucket(&state.s3_bucket).key(s3_key).send().await {
            Ok(result) => {
                let data = result.body.collect().await.map(|b| b.into_bytes().to_vec()).unwrap_or_default();
                let json: serde_json::Value = serde_json::from_slice(&data).unwrap_or_default();
                let stats = json.get("statistics").cloned().unwrap_or_default();
                let layers: Vec<serde_json::Value> = json.get("layers")
                    .and_then(|l| l.as_array())
                    .map(|arr| arr.iter().map(|l| serde_json::json!({
                        "name": l.get("name"),
                        "color": l.get("color"),
                    })).collect())
                    .unwrap_or_default();
                let annotations: Vec<String> = json.get("annotations")
                    .and_then(|a| a.as_array())
                    .map(|arr| arr.iter().filter_map(|a| a.get("value").and_then(|v| v.as_str()).map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                let blocks: Vec<serde_json::Value> = json.get("blocks")
                    .and_then(|b| b.as_array())
                    .map(|arr| arr.iter()
                        .filter(|b| b.get("entity_count").and_then(|c| c.as_i64()).unwrap_or(0) > 0
                            && !b.get("name").and_then(|n| n.as_str()).unwrap_or("*").starts_with('*'))
                        .map(|b| serde_json::json!({
                            "name": b.get("name"),
                            "entity_count": b.get("entity_count"),
                        }))
                        .collect())
                    .unwrap_or_default();
                let file_meta = json.get("file_metadata").cloned().unwrap_or_default();

                serde_json::json!({
                    "available": true,
                    "layer_count": stats.get("total_layers"),
                    "dimension_count": stats.get("total_entities").and_then(|_| {
                        stats.get("entity_type_counts").and_then(|tc| tc.get("DIMENSION_ROTATED"))
                    }),
                    "block_count": stats.get("total_blocks"),
                    "entity_type_counts": stats.get("entity_type_counts"),
                    "entities_per_layer": stats.get("entities_per_layer"),
                    "layers": layers,
                    "annotations": annotations,
                    "blocks": blocks,
                    "insert_units": file_meta.get("insert_units"),
                    "version": file_meta.get("version"),
                })
            }
            Err(_) => serde_json::json!({ "available": false }),
        }
    } else {
        serde_json::json!({ "available": false })
    };

    // KSS report info
    let kss_info: Option<(bool, Option<f64>, Option<f64>, Option<f64>, Option<i32>)> = sqlx::query_as(
        "SELECT ai_enhanced, subtotal_lv, vat_lv, total_with_vat_lv, item_count FROM kss_reports WHERE drawing_id = $1 LIMIT 1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let kss_status = if let Some((ai_enhanced, subtotal, vat, total, items)) = kss_info {
        serde_json::json!({
            "status": "generated",
            "ai_enhanced": ai_enhanced,
            "subtotal_lv": subtotal,
            "vat_lv": vat,
            "total_with_vat_lv": total,
            "item_count": items,
        })
    } else {
        serde_json::json!({ "status": "not_generated" })
    };

    Ok(Json(serde_json::json!({
        "drawing": {
            "id": drawing.0,
            "filename": drawing.1,
            "format": drawing.2,
            "units": drawing.3,
            "entity_count": drawing.4,
            "kss_generated": drawing.5,
            "kss_total_lv": drawing.6,
            "created_at": drawing.7.to_rfc3339(),
        },
        "analysis": analysis_summary,
        "kss": kss_status,
    })))
}
