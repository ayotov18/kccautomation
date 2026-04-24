//! DB-first scrape pipeline.
//!
//! Architecture: fetch → parse → persist-to-DB → done
//! Artifact upload (S3 CSV) is optional and non-blocking.
//! Job success = DB rows written. S3 failure = warning, not error.

use anyhow::Result;
use uuid::Uuid;

use crate::jobs::ScrapeJob;
use crate::pipeline::{upload_to_s3, WorkerContext};

/// Process a price scraping job with DB-first persistence.
pub async fn process_scrape_job(job: ScrapeJob, ctx: &WorkerContext) -> Result<()> {
    let job_id = job.job_id;
    let user_id = job.user_id;

    tracing::info!(%job_id, %user_id, "Starting scrape pipeline (DB-first)");
    update_job_status(&ctx.db, job_id, "scraping", 5).await?;

    // Load BrightData config
    let api_key = std::env::var("BRIGHTDATA_API_KEY")
        .map_err(|_| anyhow::anyhow!("BRIGHTDATA_API_KEY not set"))?;
    let zone = std::env::var("BRIGHTDATA_ZONE")
        .map_err(|_| anyhow::anyhow!("BRIGHTDATA_ZONE not set"))?;
    let bd_client = kcc_core::scraper::brightdata::BrightDataClient::new(api_key, zone);

    // Phase 0: Create scrape_run record
    let run_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO scrape_runs (id, job_id, user_id, status) VALUES ($1, $2, $3, 'running')",
    )
    .bind(run_id)
    .bind(job_id)
    .bind(user_id)
    .execute(&ctx.db)
    .await?;

    let parsers = kcc_core::scraper::parsers::builtin_parsers();
    let mut total_sources: i32 = 0;
    let mut successful_sources: i32 = 0;
    let mut failed_sources: i32 = 0;
    let mut total_rows_persisted: i32 = 0;

    let total_urls: usize = parsers.iter().map(|p| p.category_urls().len()).sum();
    let mut urls_processed: usize = 0;

    // Phase 1+2+3: Fetch → Parse → Persist (per source URL)
    for parser in &parsers {
        for cat_url in parser.category_urls() {
            total_sources += 1;
            urls_processed += 1;
            let progress = 5 + ((urls_processed * 85) / total_urls.max(1)) as i32;
            update_job_status(&ctx.db, job_id, "scraping", progress).await?;

            // Create source run record
            let source_run_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO scrape_source_runs (id, scrape_run_id, site, url, category_hint)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(source_run_id)
            .bind(run_id)
            .bind(parser.site_name())
            .bind(&cat_url.url)
            .bind(&cat_url.sek_group_hint)
            .execute(&ctx.db)
            .await?;

            // FETCH
            tracing::info!(
                %job_id, site = parser.site_name(), url = %cat_url.url, "Fetching"
            );

            let fetch_result = bd_client
                .fetch_html_debug(&cat_url.url, parser.expect_selector())
                .await;

            let html = match fetch_result {
                Ok((html, info)) => {
                    sqlx::query(
                        "UPDATE scrape_source_runs SET fetch_status = 'success', http_status = $1, elapsed_ms = $2, html_len = $3, fetched_at = now() WHERE id = $4",
                    )
                    .bind(info.status as i32)
                    .bind(info.elapsed_ms as i32)
                    .bind(info.html_len as i32)
                    .bind(source_run_id)
                    .execute(&ctx.db)
                    .await?;

                    if info.html_len == 0 || html.trim().is_empty() {
                        tracing::warn!(%job_id, site = parser.site_name(), url = %cat_url.url, "Empty HTML response — treating as fetch failure");
                        sqlx::query("UPDATE scrape_source_runs SET fetch_status = 'failed', parse_status = 'skipped', db_status = 'skipped', error_message = 'Empty HTML body' WHERE id = $1")
                            .bind(source_run_id).execute(&ctx.db).await?;
                        failed_sources += 1;
                        continue;
                    }

                    html
                }
                Err(e) => {
                    tracing::warn!(%job_id, site = parser.site_name(), url = %cat_url.url, error = %e, "Fetch failed");
                    sqlx::query("UPDATE scrape_source_runs SET fetch_status = 'failed', error_message = $1 WHERE id = $2")
                        .bind(e.to_string()).bind(source_run_id).execute(&ctx.db).await?;
                    failed_sources += 1;
                    continue;
                }
            };

            // PARSE with diagnostics
            let parse_result = parser.parse_page(&html, &cat_url.url);

            // Log diagnostics
            tracing::info!(
                %job_id, site = parser.site_name(), url = %cat_url.url,
                strategy = parse_result.strategy_used,
                candidates_before = parse_result.candidates_before_filter,
                candidates_after = parse_result.candidates_after_filter,
                accepted = parse_result.prices.len(),
                "Parse complete"
            );
            for (selector, count) in &parse_result.diagnostics {
                tracing::debug!(%job_id, site = parser.site_name(), selector, count, "Selector match");
            }

            // Map to SEK codes
            let mapped = kcc_core::scraper::sek_mapper::map_batch(
                &parse_result.prices,
                Some(&cat_url.sek_group_hint),
            );

            sqlx::query(
                "UPDATE scrape_source_runs SET parse_status = 'success', parsed_count = $1 WHERE id = $2",
            )
            .bind(parse_result.prices.len() as i32)
            .bind(source_run_id)
            .execute(&ctx.db)
            .await?;

            if mapped.is_empty() {
                sqlx::query("UPDATE scrape_source_runs SET db_status = 'skipped' WHERE id = $1")
                    .bind(source_run_id)
                    .execute(&ctx.db)
                    .await?;
                successful_sources += 1;
                continue;
            }

            // Deduplicate by normalized key before insert
            let mut seen_keys = std::collections::HashSet::new();
            let deduped: Vec<_> = mapped.into_iter().filter(|m| {
                let key = format!(
                    "{}|{}|{:?}|{:?}",
                    m.scraped.description_bg.to_lowercase().trim(),
                    m.scraped.unit.to_lowercase(),
                    m.scraped.price_min_lv.map(|v| (v * 100.0) as i64),
                    m.scraped.price_max_lv.map(|v| (v * 100.0) as i64),
                );
                seen_keys.insert(key)
            }).collect();

            // PERSIST TO DB with lv/eur columns
            let mut rows_written = 0i32;
            for m in &deduped {
                let result = sqlx::query(
                    "INSERT INTO scraped_price_rows (scrape_source_run_id, user_id, site, source_url, category_slug, item_name, unit, price_min, price_max, price_avg, price_min_lv, price_max_lv, price_min_eur, price_max_eur, currency, raw_price_text, sek_code, sek_group, mapping_confidence, extraction_confidence, extraction_strategy)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)",
                )
                .bind(source_run_id)
                .bind(user_id)
                .bind(&m.scraped.source_site)
                .bind(&m.scraped.source_url)
                .bind(&m.scraped.category)
                .bind(&m.scraped.description_bg)
                .bind(&m.scraped.unit)
                .bind(m.scraped.price_min_lv) // legacy price_min = lv
                .bind(m.scraped.price_max_lv) // legacy price_max = lv
                .bind(m.scraped.price_avg_lv()) // legacy price_avg = lv
                .bind(m.scraped.price_min_lv)
                .bind(m.scraped.price_max_lv)
                .bind(m.scraped.price_min_eur)
                .bind(m.scraped.price_max_eur)
                .bind(&m.scraped.currency)
                .bind(&m.scraped.raw_price_text)
                .bind(&m.sek_code)
                .bind(&m.sek_group)
                .bind(m.confidence)
                .bind(m.scraped.extraction_confidence)
                .bind(parse_result.strategy_used)
                .execute(&ctx.db)
                .await;

                match result {
                    Ok(_) => rows_written += 1,
                    Err(e) => tracing::warn!(%job_id, error = %e, "Failed to persist price row"),
                }
            }

            total_rows_persisted += rows_written;

            let db_status = if rows_written > 0 { "success" } else { "failed" };
            sqlx::query("UPDATE scrape_source_runs SET db_status = $1 WHERE id = $2")
                .bind(db_status)
                .bind(source_run_id)
                .execute(&ctx.db)
                .await?;

            if rows_written > 0 {
                successful_sources += 1;
            } else {
                failed_sources += 1;
            }

            tracing::info!(
                %job_id, site = parser.site_name(), url = %cat_url.url,
                rows_written, "DB persist complete"
            );
        }
    }

    // Phase 4: AI Price Research (fills gaps in scraped data)
    let ai_config = kcc_core::ai::AiConfig::from_env();
    if ai_config.enabled {
        update_job_status(&ctx.db, job_id, "scraping", 88).await?;
        tracing::info!(%job_id, "Starting AI price research phase");

        match kcc_core::ai::OpenRouterClient::new(&ai_config) {
            Ok(ai_client) => {
                let researcher = kcc_core::ai::researcher::PriceResearchAgent::new(&ai_client);
                match researcher.research_gaps(&ctx.db, user_id).await {
                    Ok(ai_prices) => {
                        tracing::info!(%job_id, found = ai_prices.len(), "AI research completed");
                        for price in &ai_prices {
                            let sek_group = price.category.as_deref().unwrap_or("");
                            let _ = sqlx::query(
                                "INSERT INTO scraped_price_rows (user_id, site, source_url, item_name, unit, price_min, price_max, price_avg, price_min_lv, price_max_lv, price_min_eur, price_max_eur, currency, sek_group, mapping_confidence, extraction_confidence, extraction_strategy)
                                 VALUES ($1, 'ai_research', $2, $3, $4, $5, $6, $7, $5, $6, $8, $9, 'lv', $10, 0.8, $11, 'ai_web_search')"
                            )
                            .bind(user_id)
                            .bind(&price.source_url)
                            .bind(&price.description_bg)
                            .bind(&price.unit)
                            .bind(price.price_min_lv)
                            .bind(price.price_max_lv)
                            .bind(price.price_avg_lv())
                            .bind(price.price_min_eur)
                            .bind(price.price_max_eur)
                            .bind(sek_group)
                            .bind(price.extraction_confidence)
                            .execute(&ctx.db)
                            .await;
                        }
                        total_rows_persisted += ai_prices.len() as i32;
                        successful_sources += 1;
                    }
                    Err(e) => tracing::warn!(%job_id, error = %e, "AI research failed (non-fatal)"),
                }
            }
            Err(e) => tracing::warn!(%job_id, error = %e, "AI client init failed"),
        }
    } else {
        tracing::info!(%job_id, "AI research skipped (OPENROUTER_API_KEY not set)");
    }

    // Phase 6: Optional artifact upload (non-blocking)
    let mut artifact_failures = 0i32;
    let artifact_enabled = std::env::var("SCRAPE_ARTIFACT_UPLOAD_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if artifact_enabled && total_rows_persisted > 0 {
        update_job_status(&ctx.db, job_id, "reporting", 92).await?;
        match try_upload_csv_artifact(ctx, user_id, run_id).await {
            Ok(_) => tracing::info!(%job_id, "CSV artifact uploaded"),
            Err(e) => {
                tracing::warn!(%job_id, error = %e, "Artifact upload failed (non-fatal)");
                artifact_failures += 1;
            }
        }
    }

    // Phase 7: Finalize run — success based on DB writes
    let run_status = if successful_sources > 0 { "completed" } else { "failed" };
    sqlx::query(
        "UPDATE scrape_runs SET status = $1, completed_at = now(), total_sources = $2, successful_sources = $3, failed_sources = $4, artifact_failures = $5 WHERE id = $6",
    )
    .bind(run_status)
    .bind(total_sources)
    .bind(successful_sources)
    .bind(failed_sources)
    .bind(artifact_failures)
    .bind(run_id)
    .execute(&ctx.db)
    .await?;

    let job_status = if successful_sources > 0 { "done" } else { "failed" };
    update_job_status(&ctx.db, job_id, job_status, 100).await?;

    tracing::info!(
        %job_id, %run_id,
        total_sources, successful_sources, failed_sources,
        total_rows_persisted, artifact_failures,
        "Scrape pipeline complete"
    );

    Ok(())
}

/// Try to build a CSV from persisted rows and upload to S3.
/// This is non-fatal — caller catches errors.
async fn try_upload_csv_artifact(
    ctx: &WorkerContext,
    user_id: Uuid,
    run_id: Uuid,
) -> Result<()> {
    // Read persisted rows from DB
    let rows: Vec<(Option<String>, String, Option<String>, Option<f64>, Option<f64>, Option<String>)> = sqlx::query_as(
        "SELECT sek_code, item_name, unit, price_min, price_max, currency FROM scraped_price_rows WHERE scrape_source_run_id IN (SELECT id FROM scrape_source_runs WHERE scrape_run_id = $1) ORDER BY site, item_name",
    )
    .bind(run_id)
    .fetch_all(&ctx.db)
    .await?;

    if rows.is_empty() {
        return Ok(());
    }

    // Build CSV
    let mut csv = String::from("sek_code,description,unit,price_min,price_max,currency\n");
    for (sek_code, name, unit, pmin, pmax, currency) in &rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            sek_code.as_deref().unwrap_or(""),
            escape_csv(name),
            unit.as_deref().unwrap_or(""),
            pmin.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            pmax.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            currency.as_deref().unwrap_or("EUR"),
        ));
    }

    // Fresh S3 client to avoid stale connection
    let fresh_s3 = build_fresh_s3_client().await;
    let list_id = Uuid::new_v4();
    let s3_key = format!("price-lists/{}/{}.csv", user_id, list_id);

    for attempt in 1..=3u32 {
        match upload_to_s3(&fresh_s3, &ctx.bucket, &s3_key, csv.as_bytes()).await {
            Ok(()) => {
                // Create price_lists record
                let name = format!(
                    "Market Prices (scraped {})",
                    chrono::Utc::now().format("%Y-%m-%d")
                );
                sqlx::query(
                    "INSERT INTO price_lists (id, user_id, name, s3_key, item_count, source, scrape_metadata) VALUES ($1, $2, $3, $4, $5, 'brightdata', $6)",
                )
                .bind(list_id)
                .bind(user_id)
                .bind(&name)
                .bind(&s3_key)
                .bind(rows.len() as i32)
                .bind(serde_json::json!({"run_id": run_id}))
                .execute(&ctx.db)
                .await?;
                return Ok(());
            }
            Err(e) if attempt < 3 => {
                tracing::warn!(attempt, error = %e, "S3 upload retry");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

async fn build_fresh_s3_client() -> aws_sdk_s3::Client {
    let endpoint_override = std::env::var("AWS_ENDPOINT_URL").ok();
    let s3_region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let mut loader =
        aws_config::from_env().region(aws_config::meta::region::RegionProviderChain::first_try(
            aws_sdk_s3::config::Region::new(s3_region),
        ));
    if let Some(ref endpoint) = endpoint_override {
        loader = loader.endpoint_url(endpoint.as_str());
    }
    let aws_config = loader.load().await;
    let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
        .force_path_style(endpoint_override.is_some())
        .build();
    aws_sdk_s3::Client::from_conf(s3_config)
}

async fn update_job_status(
    db: &sqlx::PgPool,
    job_id: Uuid,
    status: &str,
    progress: i32,
) -> Result<()> {
    sqlx::query("UPDATE jobs SET status = $1, progress = $2 WHERE id = $3")
        .bind(status)
        .bind(progress)
        .bind(job_id)
        .execute(db)
        .await?;
    Ok(())
}
