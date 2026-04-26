use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use kcc_core::kcc::config::KccConfig;
use kcc_dxf::{dwg_converter, renderer};
use kcc_report::{csv as csv_report, json as json_report, pdf as pdf_report};

use crate::jobs::AnalyzeDrawingJob;

#[derive(Clone)]
pub struct WorkerContext {
    pub db: PgPool,
    pub s3: aws_sdk_s3::Client,
    pub bucket: String,
}

/// Process a single analysis job through the full pipeline.
pub async fn process_job(job: AnalyzeDrawingJob, ctx: &WorkerContext) -> Result<()> {
    let job_id = job.job_id;
    let drawing_id = job.drawing_id;

    tracing::info!(%job_id, %drawing_id, "Starting job pipeline");

    // ===================================================================
    // Stage 1: PARSE — Download, convert DWG if needed, parse DXF
    // ===================================================================
    update_status(&ctx.db, job_id, "parsing", 0).await?;

    let file_bytes = download_from_s3(&ctx.s3, &ctx.bucket, &job.s3_key).await?;
    let filename = job
        .s3_key
        .rsplit('/')
        .next()
        .unwrap_or("drawing.dxf")
        .to_string();

    let dxf_bytes = if dwg_converter::is_dwg_bytes(&file_bytes) {
        tracing::info!(%job_id, "Detected DWG format, converting to DXF");
        let converter = dwg_converter::DwgConverter::auto_detect()?;
        let fname = filename.clone();
        let bytes = file_bytes.clone();
        let converted =
            tokio::task::spawn_blocking(move || converter.convert_bytes(&bytes, &fname))
                .await
                .map_err(|e| anyhow::anyhow!("DWG conversion task panicked: {e}"))??;

        let dxf_key = format!("uploads/{drawing_id}/converted.dxf");
        upload_to_s3(&ctx.s3, &ctx.bucket, &dxf_key, &converted).await?;
        sqlx::query("UPDATE drawings SET s3_key_dxf = $1 WHERE id = $2")
            .bind(&dxf_key)
            .bind(drawing_id)
            .execute(&ctx.db)
            .await?;

        tracing::info!(%job_id, dxf_size = converted.len(), "DWG conversion complete");
        converted
    } else {
        file_bytes
    };

    let dxf_bytes_for_deep = dxf_bytes.clone(); // Keep a copy for deep analysis
    let fname_for_parse = filename.clone();
    let drawing = tokio::task::spawn_blocking(move || {
        kcc_dxf::parser::parse_dxf_bytes(&dxf_bytes, fname_for_parse)
    })
    .await
    .map_err(|e| anyhow::anyhow!("DXF parse task panicked: {e}"))?
    .map_err(|e| {
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
        if ext == "dwg" {
            anyhow::anyhow!(
                "This file appears to be in DWG format. DWG conversion requires ODA File Converter \
                 which is not installed. Please upload a DXF file instead."
            )
        } else {
            anyhow::anyhow!("Failed to parse DXF file: {e}")
        }
    })?;

    // Update drawing metadata
    sqlx::query("UPDATE drawings SET units = $1, entity_count = $2 WHERE id = $3")
        .bind(format!("{:?}", drawing.units))
        .bind(drawing.entities.len() as i32)
        .bind(drawing_id)
        .execute(&ctx.db)
        .await?;

    update_status(&ctx.db, job_id, "parsing", 25).await?;

    // === TELEMETRY: Parse stage ===
    let mut type_counts: HashMap<&str, usize> = HashMap::new();
    for e in &drawing.entities {
        let name = match &e.geometry {
            kcc_core::geometry::model::GeometryPrimitive::Line { .. } => "Line",
            kcc_core::geometry::model::GeometryPrimitive::Arc { .. } => "Arc",
            kcc_core::geometry::model::GeometryPrimitive::Circle { .. } => "Circle",
            kcc_core::geometry::model::GeometryPrimitive::Polyline { .. } => "Polyline",
            kcc_core::geometry::model::GeometryPrimitive::Spline { .. } => "Spline",
            kcc_core::geometry::model::GeometryPrimitive::Point(_) => "Point",
        };
        *type_counts.entry(name).or_default() += 1;
    }
    let block_entities = drawing.entities.iter().filter(|e| e.block_ref.is_some()).count();
    tracing::info!(
        %job_id,
        total_entities = drawing.entities.len(),
        from_blocks = block_entities,
        dimensions = drawing.dimensions.len(),
        annotations = drawing.annotations.len(),
        gdt_frames = drawing.gdt_frames.len(),
        datums_in_drawing = drawing.datums.len(),
        entity_types = ?type_counts,
        "TELEMETRY [parse]: entity breakdown"
    );

    // ===================================================================
    // Stage 2: EXTRACT + CLASSIFY — Feature detection, scoring
    // ===================================================================
    update_status(&ctx.db, job_id, "extracting", 25).await?;

    let config = KccConfig::default();
    let drawing_clone = drawing.clone();
    let config_clone = config.clone();

    let analysis = tokio::task::spawn_blocking(move || {
        kcc_core::analyze_drawing(drawing_clone, &config_clone)
    })
    .await
    .map_err(|e| anyhow::anyhow!("spawn_blocking panicked: {e}"))?;

    // === TELEMETRY: Feature extraction breakdown ===
    let mut feature_type_counts: HashMap<&str, usize> = HashMap::new();
    for f in &analysis.features {
        *feature_type_counts.entry(f.feature_type.name()).or_default() += 1;
    }
    let features_with_dims = analysis.features.iter().filter(|f| !f.dimensions.is_empty()).count();
    let features_with_gdt = analysis.features.iter().filter(|f| !f.gdt_frames.is_empty()).count();
    let features_with_datums = analysis.features.iter().filter(|f| !f.datum_refs.is_empty()).count();
    tracing::info!(
        %job_id,
        total_features = analysis.features.len(),
        with_dimensions = features_with_dims,
        with_gdt = features_with_gdt,
        with_datums = features_with_datums,
        feature_types = ?feature_type_counts,
        tolerance_chains = analysis.tolerance_chains.len(),
        datums_found = analysis.datums.len(),
        "TELEMETRY [extract]: feature breakdown"
    );

    // Store features in DB
    let mut feature_id_map: HashMap<u64, Uuid> = HashMap::new();
    for feature in &analysis.features {
        let db_id: Uuid = sqlx::query_scalar(
            "INSERT INTO features (drawing_id, feature_type, description, centroid_x, centroid_y, geometry_refs, properties) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        )
        .bind(drawing_id)
        .bind(feature.feature_type.name())
        .bind(feature.description())
        .bind(feature.centroid.x)
        .bind(feature.centroid.y)
        .bind(serde_json::to_value(&feature.geometry_refs)?)
        .bind(serde_json::to_value(&feature.feature_type)?)
        .fetch_one(&ctx.db)
        .await?;

        feature_id_map.insert(feature.id.0, db_id);
    }

    update_status(&ctx.db, job_id, "classifying", 60).await?;

    // === TELEMETRY: Classification distribution ===
    let kcc_count = analysis.kcc_results.iter().filter(|(_, s)| s.classification == kcc_core::kcc::types::KccClassification::Kcc).count();
    let imp_count = analysis.kcc_results.iter().filter(|(_, s)| s.classification == kcc_core::kcc::types::KccClassification::Important).count();
    let std_count = analysis.kcc_results.iter().filter(|(_, s)| s.classification == kcc_core::kcc::types::KccClassification::Standard).count();

    // Log per-feature scoring detail for debugging
    for (fid, score) in &analysis.kcc_results {
        if score.total > 0 {
            let factor_names: Vec<&str> = score.factors.iter().map(|f| f.name.as_str()).collect();
            tracing::debug!(
                %job_id,
                feature_id = fid,
                score = score.total,
                classification = score.classification.as_str(),
                factors = ?factor_names,
                "TELEMETRY [classify]: scored feature"
            );
        }
    }
    tracing::info!(
        %job_id,
        kcc = kcc_count, important = imp_count, standard = std_count,
        total_scored = analysis.kcc_results.len(),
        nonzero_scores = analysis.kcc_results.iter().filter(|(_, s)| s.total > 0).count(),
        "TELEMETRY [classify]: distribution"
    );

    // Store KCC results
    for (feature_id_num, score) in &analysis.kcc_results {
        if let Some(db_feature_id) = feature_id_map.get(feature_id_num) {
            sqlx::query(
                "INSERT INTO kcc_results (feature_id, drawing_id, classification, score, factors) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(db_feature_id)
            .bind(drawing_id)
            .bind(score.classification.as_str())
            .bind(score.total as i32)
            .bind(serde_json::to_value(&score.factors)?)
            .execute(&ctx.db)
            .await?;
        }
    }

    update_status(&ctx.db, job_id, "classifying", 85).await?;

    // ===================================================================
    // Stage 3: PERSIST CANONICAL SNAPSHOT — Single source of truth
    // ===================================================================
    // This is the key architectural artifact: the full AnalysisResult (drawing +
    // features + scores + chains + datums) serialized once, consumed by everything
    // downstream (viewer, KSS, reports) without lossy reconstruction from DB rows.
    let snapshot_bytes = serde_json::to_vec(&analysis)?;
    let snapshot_key = format!("reports/{drawing_id}/analysis.json");
    upload_to_s3(&ctx.s3, &ctx.bucket, &snapshot_key, &snapshot_bytes).await?;
    tracing::info!(
        %job_id,
        snapshot_size = snapshot_bytes.len(),
        snapshot_key = %snapshot_key,
        "TELEMETRY [snapshot]: canonical analysis persisted"
    );

    // ===================================================================
    // Stage 4: GENERATE REPORTS — All consume the same analysis
    // ===================================================================
    update_status(&ctx.db, job_id, "reporting", 85).await?;

    // JSON report
    let json_bytes = json_report::generate_json_report(&analysis, drawing_id)?;
    let json_key = format!("reports/{drawing_id}/report.json");
    upload_to_s3(&ctx.s3, &ctx.bucket, &json_key, &json_bytes).await?;
    store_report_record(&ctx.db, drawing_id, "json", &json_key).await?;

    // CSV report
    let csv_bytes = csv_report::generate_csv_report(&analysis)?;
    let csv_key = format!("reports/{drawing_id}/report.csv");
    upload_to_s3(&ctx.s3, &ctx.bucket, &csv_key, &csv_bytes).await?;
    store_report_record(&ctx.db, drawing_id, "csv", &csv_key).await?;

    // PDF report
    let pdf_bytes = pdf_report::generate_pdf_report(&analysis, drawing_id)?;
    let pdf_key = format!("reports/{drawing_id}/report.pdf");
    upload_to_s3(&ctx.s3, &ctx.bucket, &pdf_key, &pdf_bytes).await?;
    store_report_record(&ctx.db, drawing_id, "pdf", &pdf_key).await?;

    // Render packet — replace internal feature IDs with DB UUIDs
    let mut render_packet = renderer::generate_render_packet(&drawing, &analysis);
    for rf in &mut render_packet.features {
        if let Some(internal_id) = rf.id.strip_prefix("F-").and_then(|s| s.parse::<u64>().ok()) {
            if let Some(db_uuid) = feature_id_map.get(&internal_id) {
                rf.id = db_uuid.to_string();
            }
        }
    }
    let render_bytes = serde_json::to_vec(&render_packet)?;
    let render_key = format!("reports/{drawing_id}/render.json");
    upload_to_s3(&ctx.s3, &ctx.bucket, &render_key, &render_bytes).await?;

    // ===================================================================
    // Stage 5: AUTO DEEP ANALYSIS — extract everything for overview page
    // ===================================================================
    update_status(&ctx.db, job_id, "reporting", 95).await?;

    match run_deep_analysis(ctx, job_id, drawing_id, &dxf_bytes_for_deep).await {
        Ok(json_size) => {
            tracing::info!(%job_id, json_size, "Deep analysis auto-completed");
        }
        Err(e) => {
            // Non-fatal — overview page will show "deep analysis not available"
            tracing::warn!(%job_id, error = %e, "Deep analysis failed (non-fatal)");
        }
    }

    // ===================================================================
    // DONE
    // ===================================================================
    update_status(&ctx.db, job_id, "done", 100).await?;

    sqlx::query("UPDATE jobs SET completed_at = now() WHERE id = $1")
        .bind(job_id)
        .execute(&ctx.db)
        .await?;

    tracing::info!(%job_id, %drawing_id, "Job complete");

    Ok(())
}

/// Load the canonical analysis snapshot from S3 for a given drawing.
pub async fn load_analysis_snapshot(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    drawing_id: Uuid,
) -> Result<kcc_core::AnalysisResult> {
    let key = format!("reports/{drawing_id}/analysis.json");
    let data = download_from_s3(s3, bucket, &key).await?;
    let analysis: kcc_core::AnalysisResult = serde_json::from_slice(&data)?;
    Ok(analysis)
}

async fn update_status(db: &PgPool, job_id: Uuid, status: &str, progress: i32) -> Result<()> {
    sqlx::query("UPDATE jobs SET status = $1, progress = $2, started_at = COALESCE(started_at, now()) WHERE id = $3")
        .bind(status)
        .bind(progress)
        .bind(job_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn download_from_s3(s3: &aws_sdk_s3::Client, bucket: &str, key: &str) -> Result<Vec<u8>> {
    let result = s3.get_object().bucket(bucket).key(key).send().await?;
    let data = result.body.collect().await?.into_bytes().to_vec();
    Ok(data)
}

pub async fn upload_to_s3(s3: &aws_sdk_s3::Client, bucket: &str, key: &str, data: &[u8]) -> Result<()> {
    s3.put_object()
        .bucket(bucket)
        .key(key)
        .body(data.to_vec().into())
        .send()
        .await?;
    Ok(())
}

async fn store_report_record(
    db: &PgPool,
    drawing_id: Uuid,
    format: &str,
    s3_key: &str,
) -> Result<()> {
    sqlx::query("INSERT INTO reports (drawing_id, format, s3_key) VALUES ($1, $2, $3)")
        .bind(drawing_id)
        .bind(format)
        .bind(s3_key)
        .execute(db)
        .await?;
    Ok(())
}

/// Run deep analysis and persist to S3 (file download) + Postgres (normalized tables).
async fn run_deep_analysis(
    ctx: &WorkerContext,
    job_id: Uuid,
    drawing_id: Uuid,
    dxf_bytes: &[u8],
) -> Result<usize> {
    let deep_json = kcc_dxf::deep_analyze::deep_analyze(dxf_bytes, "original.dwg")
        .map_err(|e| anyhow::anyhow!("Deep analysis failed: {e}"))?;
    let json_bytes = serde_json::to_vec_pretty(&deep_json)?;
    let json_size = json_bytes.len();

    // Upload to S3 (for file download only)
    let s3_key = format!("reports/{drawing_id}/deep-analysis.json");
    upload_to_s3(&ctx.s3, &ctx.bucket, &s3_key, &json_bytes).await?;
    store_report_record(&ctx.db, drawing_id, "deep_analysis", &s3_key).await?;

    // ── Purge stale rows so re-analysis is idempotent ──────
    // Deep-analysis used to be append-only — re-running on the same
    // drawing duplicated drawing_layers/blocks/dimensions/annotations
    // and silently kept old rows around forever. Now we delete first,
    // so triggering /drawings/{id}/deep-analyze always reflects the
    // latest parser output.
    for table in ["drawing_layers", "drawing_blocks", "drawing_dimensions", "drawing_annotations"] {
        let _ = sqlx::query(&format!("DELETE FROM {table} WHERE drawing_id = $1"))
            .bind(drawing_id)
            .execute(&ctx.db)
            .await;
    }

    // ── Persist to normalized Postgres tables ──────────────
    // This is the PRIMARY data source for AI pipelines — not S3.

    // Extract statistics
    let stats = deep_json.get("statistics").cloned().unwrap_or_default();
    let file_meta = deep_json.get("file_metadata").cloned().unwrap_or_default();

    // Update drawings metadata columns. Also populate `entity_count` + `units`
    // from the deep-analysis JSON so the AI KSS pipeline's Phase 1 audit has
    // real numbers (previously it ran the ai-full flow without ever calling
    // the rule-based `pipeline::process_job`, so `entity_count` stayed NULL
    // and the audit reported 0 entities even on 1856-entity files).
    let total_entities = stats
        .get("total_entities")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    let insert_units_code = file_meta
        .get("insert_units")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    let units_label = match insert_units_code {
        Some(1) => Some("Inches"),
        Some(2) => Some("Feet"),
        Some(4) => Some("Millimeters"),
        Some(5) => Some("Centimeters"),
        Some(6) => Some("Meters"),
        Some(0) | None => Some("Unitless"),
        _ => Some("Unknown"),
    };
    let _ = sqlx::query(
        "UPDATE drawings SET
            total_layers       = $1,
            total_blocks       = $2,
            total_dimensions   = $3,
            total_annotations  = $4,
            insert_units_raw   = $5,
            dwg_version        = $6,
            entity_count       = COALESCE($7, entity_count),
            units              = COALESCE($8, units)
         WHERE id = $9",
    )
    .bind(stats.get("total_layers").and_then(|v| v.as_i64()).map(|v| v as i32))
    .bind(stats.get("total_blocks").and_then(|v| v.as_i64()).map(|v| v as i32))
    .bind(deep_json.get("dimensions").and_then(|d| d.as_array()).map(|a| a.len() as i32))
    .bind(deep_json.get("annotations").and_then(|a| a.as_array()).map(|a| a.len() as i32))
    .bind(insert_units_code)
    .bind(file_meta.get("version").and_then(|v| v.as_str()))
    .bind(total_entities)
    .bind(units_label)
    .bind(drawing_id)
    .execute(&ctx.db)
    .await;

    // Insert layers
    if let Some(layers) = deep_json.get("layers").and_then(|l| l.as_array()) {
        let entities_per_layer = stats.get("entities_per_layer").cloned().unwrap_or_default();
        for layer in layers {
            let name = layer.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let color = layer.get("color").and_then(|c| c.as_i64()).map(|c| c as i32);
            let count = entities_per_layer.get(name).and_then(|c| c.as_i64()).unwrap_or(0) as i32;
            let _ = sqlx::query(
                "INSERT INTO drawing_layers (drawing_id, name, color, entity_count) VALUES ($1, $2, $3, $4)"
            )
            .bind(drawing_id).bind(name).bind(color).bind(count)
            .execute(&ctx.db).await;
        }
    }

    // Insert blocks (named, non-system)
    if let Some(blocks) = deep_json.get("blocks").and_then(|b| b.as_array()) {
        for block in blocks {
            let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.starts_with('*') { continue; }
            let entity_count = block.get("entity_count").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
            if entity_count == 0 { continue; }
            let _ = sqlx::query(
                "INSERT INTO drawing_blocks (drawing_id, name, entity_count) VALUES ($1, $2, $3)"
            )
            .bind(drawing_id).bind(name).bind(entity_count)
            .execute(&ctx.db).await;
        }
    }

    // Insert dimensions
    if let Some(dims) = deep_json.get("dimensions").and_then(|d| d.as_array()) {
        for dim in dims {
            let dim_type = dim.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
            let value = dim.get("actual_measurement").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let layer = dim.get("layer").and_then(|l| l.as_str());
            let _ = sqlx::query(
                "INSERT INTO drawing_dimensions (drawing_id, dim_type, value, layer) VALUES ($1, $2, $3, $4)"
            )
            .bind(drawing_id).bind(dim_type).bind(value).bind(layer)
            .execute(&ctx.db).await;
        }
    }

    // Insert annotations
    if let Some(anns) = deep_json.get("annotations").and_then(|a| a.as_array()) {
        for ann in anns {
            let text = ann.get("value").and_then(|v| v.as_str()).unwrap_or("");
            if text.is_empty() || text == "None" { continue; }
            let layer = ann.get("layer").and_then(|l| l.as_str());
            let ann_type = ann.get("type").and_then(|t| t.as_str());
            let _ = sqlx::query(
                "INSERT INTO drawing_annotations (drawing_id, text, layer, ann_type) VALUES ($1, $2, $3, $4)"
            )
            .bind(drawing_id).bind(text).bind(layer).bind(ann_type)
            .execute(&ctx.db).await;
        }
    }

    tracing::info!(%job_id, json_size, "Deep analysis persisted to S3 + Postgres");
    Ok(json_size)
}
