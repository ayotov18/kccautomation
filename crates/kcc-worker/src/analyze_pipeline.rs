use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::jobs::DeepAnalyzeJob;
use crate::pipeline::WorkerContext;

/// Process a deep analysis job — extracts everything from the DXF.
pub async fn process_deep_analyze_job(job: DeepAnalyzeJob, ctx: &WorkerContext) -> Result<()> {
    let job_id = job.job_id;
    let drawing_id = job.drawing_id;

    tracing::info!(%job_id, %drawing_id, "Starting deep analyze pipeline");
    update_status(&ctx.db, job_id, "parsing", 10).await?;

    // Look up drawing to get S3 keys
    let row: (String, Option<String>, String) = sqlx::query_as(
        "SELECT s3_key_original, s3_key_dxf, original_format FROM drawings WHERE id = $1"
    )
    .bind(drawing_id)
    .fetch_one(&ctx.db)
    .await?;

    let (s3_key_original, s3_key_dxf, format) = row;

    // Get DXF bytes — prefer already-converted DXF, fall back to original
    let dxf_bytes = if let Some(ref dxf_key) = s3_key_dxf {
        tracing::info!(%job_id, "Using pre-converted DXF from S3");
        download_from_s3(&ctx.s3, &ctx.bucket, dxf_key).await?
    } else if format == "dwg" {
        // Need to convert DWG → DXF
        tracing::info!(%job_id, "Converting DWG to DXF via ODA");
        let dwg_bytes = download_from_s3(&ctx.s3, &ctx.bucket, &s3_key_original).await?;
        let converter = kcc_dxf::dwg_converter::DwgConverter::auto_detect()?;
        let fname = s3_key_original.rsplit('/').next().unwrap_or("file.dwg").to_string();
        let bytes = dwg_bytes;
        tokio::task::spawn_blocking(move || converter.convert_bytes(&bytes, &fname))
            .await
            .map_err(|e| anyhow::anyhow!("DWG conversion panicked: {e}"))??
    } else {
        // Already DXF
        download_from_s3(&ctx.s3, &ctx.bucket, &s3_key_original).await?
    };

    update_status(&ctx.db, job_id, "extracting", 30).await?;

    let filename = s3_key_original.rsplit('/').next().unwrap_or("drawing").to_string();
    let bytes = dxf_bytes;
    let fname = filename.clone();

    // Run deep analysis (CPU-bound)
    let analysis_json = tokio::task::spawn_blocking(move || {
        kcc_dxf::deep_analyze::deep_analyze(&bytes, &fname)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Deep analyze panicked: {e}"))?
    .map_err(|e| anyhow::anyhow!("Deep analyze failed: {e}"))?;

    update_status(&ctx.db, job_id, "reporting", 70).await?;

    let json_bytes = serde_json::to_vec_pretty(&analysis_json)?;

    tracing::info!(
        %job_id,
        json_size = json_bytes.len(),
        "Deep analysis complete"
    );

    // Upload to S3
    let s3_key = format!("reports/{drawing_id}/deep-analysis.json");
    upload_to_s3(&ctx.s3, &ctx.bucket, &s3_key, &json_bytes).await?;

    // Store report record
    sqlx::query("INSERT INTO reports (drawing_id, format, s3_key) VALUES ($1, $2, $3)")
        .bind(drawing_id)
        .bind("deep_analysis")
        .bind(&s3_key)
        .execute(&ctx.db)
        .await?;

    // Also write to local analysis/ folder for development inspection
    let local_path = format!("analysis/{drawing_id}.json");
    if let Err(e) = tokio::fs::write(&local_path, &json_bytes).await {
        tracing::warn!(%job_id, error = %e, path = %local_path, "Failed to write local analysis file (non-fatal)");
    } else {
        tracing::info!(%job_id, path = %local_path, "Written local analysis file");
    }

    // Done
    update_status(&ctx.db, job_id, "done", 100).await?;
    sqlx::query("UPDATE jobs SET completed_at = now() WHERE id = $1")
        .bind(job_id)
        .execute(&ctx.db)
        .await?;

    tracing::info!(%job_id, %drawing_id, "Deep analyze job complete");

    Ok(())
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

async fn download_from_s3(s3: &aws_sdk_s3::Client, bucket: &str, key: &str) -> Result<Vec<u8>> {
    let result = s3.get_object().bucket(bucket).key(key).send().await?;
    let data = result.body.collect().await?.into_bytes().to_vec();
    Ok(data)
}

async fn upload_to_s3(s3: &aws_sdk_s3::Client, bucket: &str, key: &str, data: &[u8]) -> Result<()> {
    s3.put_object().bucket(bucket).key(key).body(data.to_vec().into()).send().await?;
    Ok(())
}
