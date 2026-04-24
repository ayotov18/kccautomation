use sqlx::postgres::PgPoolOptions;

mod ai_kss_pipeline;
mod analyze_pipeline;
mod jobs;
mod kss_pipeline;
mod pdf_pipeline;
mod pipeline;
mod pricing_defaults;
mod quantity_scrape_pipeline;
mod scrape_pipeline;

use jobs::{AiKssJob, AnalyzeDrawingJob, DeepAnalyzeJob, GenerateKssJob, QuantityScrapeJob, ScrapeJob};
use pipeline::WorkerContext;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,kcc_worker=debug,kcc_core=debug".into()),
        )
        .init();

    tracing::info!("Starting KCC Worker");

    // Database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://kcc:kcc_dev_password@localhost:5432/kcc".to_string());
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&db).await?;

    // S3 client — supports MinIO via AWS_ENDPOINT_URL; force_path_style only for local dev
    let endpoint_override = std::env::var("AWS_ENDPOINT_URL").ok();
    let s3_region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let mut aws_config_loader =
        aws_config::from_env().region(aws_config::meta::region::RegionProviderChain::first_try(
            aws_sdk_s3::config::Region::new(s3_region),
        ));
    if let Some(ref endpoint) = endpoint_override {
        aws_config_loader = aws_config_loader.endpoint_url(endpoint.as_str());
    }
    let aws_config = aws_config_loader.load().await;
    let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
        .force_path_style(endpoint_override.is_some()) // only for MinIO/local
        .build();
    let s3 = aws_sdk_s3::Client::from_conf(s3_config);

    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kcc-files-prod".to_string());
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let ctx = WorkerContext { db, s3, bucket };

    tracing::info!("Worker ready, polling for jobs...");

    // Simple Redis-based job polling loop
    let redis_client = redis::Client::open(redis_url)?;
    let mut conn = redis_client.get_multiplexed_async_connection().await?;

    loop {
        // BRPOP blocks on both queues until a job is available (5s timeout)
        let result: Option<(String, String)> = redis::cmd("BRPOP")
            .arg("kcc:jobs")
            .arg("kcc:kss-jobs")
            .arg("kcc:analyze-jobs")
            .arg("kcc:scrape-jobs")
            .arg("kcc:quantity-scrape-jobs")
            .arg("kcc:ai-kss-jobs")
            .arg(5)
            .query_async(&mut conn)
            .await?;

        if let Some((queue, job_data)) = result {
            match queue.as_str() {
                "kcc:jobs" => {
                    match serde_json::from_str::<AnalyzeDrawingJob>(&job_data) {
                        Ok(job) => {
                            tracing::info!(job_id = %job.job_id, "Processing analysis job");
                            if let Err(e) = pipeline::process_job(job.clone(), &ctx).await {
                                tracing::error!(job_id = %job.job_id, error = %e, "Analysis job failed");
                                let _ = sqlx::query(
                                    "UPDATE jobs SET status = 'failed', error_message = $1 WHERE id = $2",
                                )
                                .bind(e.to_string())
                                .bind(job.job_id)
                                .execute(&ctx.db)
                                .await;
                            }
                        }
                        Err(e) => tracing::error!(error = %e, "Failed to deserialize analysis job"),
                    }
                }
                "kcc:kss-jobs" => {
                    match serde_json::from_str::<GenerateKssJob>(&job_data) {
                        Ok(job) => {
                            tracing::info!(job_id = %job.job_id, "Processing KSS job");
                            if let Err(e) = kss_pipeline::process_kss_job(job.clone(), &ctx).await {
                                tracing::error!(job_id = %job.job_id, error = %e, "KSS job failed");
                                let _ = sqlx::query(
                                    "UPDATE jobs SET status = 'failed', error_message = $1 WHERE id = $2",
                                )
                                .bind(e.to_string())
                                .bind(job.job_id)
                                .execute(&ctx.db)
                                .await;
                            }
                        }
                        Err(e) => tracing::error!(error = %e, "Failed to deserialize KSS job"),
                    }
                }
                "kcc:analyze-jobs" => {
                    match serde_json::from_str::<DeepAnalyzeJob>(&job_data) {
                        Ok(job) => {
                            tracing::info!(job_id = %job.job_id, "Processing deep analyze job");
                            if let Err(e) = analyze_pipeline::process_deep_analyze_job(job.clone(), &ctx).await {
                                tracing::error!(job_id = %job.job_id, error = %e, "Deep analyze job failed");
                                let _ = sqlx::query(
                                    "UPDATE jobs SET status = 'failed', error_message = $1 WHERE id = $2",
                                )
                                .bind(e.to_string())
                                .bind(job.job_id)
                                .execute(&ctx.db)
                                .await;
                            }
                        }
                        Err(e) => tracing::error!(error = %e, "Failed to deserialize deep analyze job"),
                    }
                }
                "kcc:scrape-jobs" => {
                    match serde_json::from_str::<ScrapeJob>(&job_data) {
                        Ok(job) => {
                            tracing::info!(job_id = %job.job_id, "Processing scrape job");
                            if let Err(e) = scrape_pipeline::process_scrape_job(job.clone(), &ctx).await {
                                tracing::error!(job_id = %job.job_id, error = %e, "Scrape job failed");
                                let _ = sqlx::query(
                                    "UPDATE jobs SET status = 'failed', error_message = $1 WHERE id = $2",
                                )
                                .bind(e.to_string())
                                .bind(job.job_id)
                                .execute(&ctx.db)
                                .await;
                            }
                        }
                        Err(e) => tracing::error!(error = %e, "Failed to deserialize scrape job"),
                    }
                }
                "kcc:quantity-scrape-jobs" => {
                    match serde_json::from_str::<QuantityScrapeJob>(&job_data) {
                        Ok(job) => {
                            tracing::info!(job_id = %job.job_id, "Processing quantity scrape job");
                            if let Err(e) = quantity_scrape_pipeline::process_quantity_scrape_job(job.clone(), &ctx).await {
                                tracing::error!(job_id = %job.job_id, error = %e, "Quantity scrape job failed");
                                let _ = sqlx::query(
                                    "UPDATE jobs SET status = 'failed', error_message = $1 WHERE id = $2",
                                )
                                .bind(e.to_string())
                                .bind(job.job_id)
                                .execute(&ctx.db)
                                .await;
                            }
                        }
                        Err(e) => tracing::error!(error = %e, "Failed to deserialize quantity scrape job"),
                    }
                }
                "kcc:ai-kss-jobs" => {
                    match serde_json::from_str::<AiKssJob>(&job_data) {
                        Ok(job) => {
                            tracing::info!(job_id = %job.job_id, phase = %job.phase, "Processing AI KSS job");
                            if let Err(e) = ai_kss_pipeline::process_ai_kss_job(job.clone(), &ctx).await {
                                tracing::error!(job_id = %job.job_id, error = %e, "AI KSS job failed");
                                let _ = sqlx::query(
                                    "UPDATE jobs SET status = 'failed', error_message = $1 WHERE id = $2",
                                )
                                .bind(e.to_string())
                                .bind(job.job_id)
                                .execute(&ctx.db)
                                .await;
                            }
                        }
                        Err(e) => tracing::error!(error = %e, "Failed to deserialize AI KSS job"),
                    }
                }
                _ => tracing::warn!(queue = %queue, "Unknown job queue"),
            }
        }
    }
}
