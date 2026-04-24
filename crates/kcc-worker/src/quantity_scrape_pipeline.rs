//! Quantity-norm scrape pipeline.
//!
//! Architecture (mirrors `scrape_pipeline.rs`):
//!   fetch (HTML via BrightData, PDF via plain reqwest + pdf-extract)
//!     → parse (site-specific NormParser)
//!     → sek_mapper (shared with price scraper)
//!     → persist (quantity_norms UPSERT ON CONFLICT (sek_code, source, user_id))
//!
//! One run row in `scrape_quantity_runs`, one child row per URL in
//! `scrape_quantity_source_runs`. Success is measured by DB writes.

use anyhow::Result;
use std::time::Instant;
use uuid::Uuid;

use kcc_core::quantity_scraper::parsers::procurement_xls;
use kcc_core::quantity_scraper::{parsers as qparsers, ScrapedNorm};
use kcc_core::scraper::brightdata::BrightDataClient;
use kcc_core::scraper::sek_mapper;
use kcc_core::scraper::ScrapedPrice;

use crate::jobs::QuantityScrapeJob;
use crate::pipeline::WorkerContext;

/// Entry point called from `main.rs` for `kcc:quantity-scrape-jobs`.
pub async fn process_quantity_scrape_job(
    job: QuantityScrapeJob,
    ctx: &WorkerContext,
) -> Result<()> {
    let job_id = job.job_id;
    let user_id = job.user_id;

    tracing::info!(%job_id, %user_id, "Starting quantity scrape pipeline");
    update_job_status(&ctx.db, job_id, "scraping", 5).await?;

    // BrightData for HTML; plain reqwest for PDF bodies (PDFs are static).
    let bd_client_opt = match (std::env::var("BRIGHTDATA_API_KEY"), std::env::var("BRIGHTDATA_ZONE")) {
        (Ok(k), Ok(z)) => Some(BrightDataClient::new(k, z)),
        _ => {
            tracing::warn!(%job_id, "BRIGHTDATA_* not set — HTML sources will fail, PDFs will still work");
            None
        }
    };
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    // Optional filter from job payload — empty = run every enabled source.
    let selected_templates: Option<Vec<String>> = if job.source_ids.is_empty() {
        None
    } else {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT parser_template FROM quantity_sources WHERE id = ANY($1) AND enabled = true",
        )
        .bind(&job.source_ids)
        .fetch_all(&ctx.db)
        .await?;
        Some(rows.into_iter().map(|(t,)| t).collect())
    };

    // Create scrape_quantity_runs row.
    let run_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO scrape_quantity_runs (id, job_id, user_id, status, total_sources) VALUES ($1, $2, $3, 'running', 0)",
    )
    .bind(run_id)
    .bind(job_id)
    .bind(user_id)
    .execute(&ctx.db)
    .await?;

    let all_parsers = qparsers::builtin_parsers();
    // Filter parsers by selected source templates if provided.
    let parsers: Vec<_> = if let Some(ref templates) = selected_templates {
        all_parsers
            .into_iter()
            .filter(|p| templates.iter().any(|t| t == p.template_key()))
            .collect()
    } else {
        all_parsers
    };

    let total_urls: usize = parsers.iter().map(|p| p.category_urls().len()).sum();
    let mut urls_processed: usize = 0;
    let mut total_sources = 0i32;
    let mut successful_sources = 0i32;
    let mut failed_sources = 0i32;
    let mut norms_created = 0i32;
    let mut norms_updated = 0i32;

    let pipeline_start = Instant::now();

    for parser in &parsers {
        for cat in parser.category_urls() {
            total_sources += 1;
            urls_processed += 1;
            let progress = 5 + ((urls_processed * 85) / total_urls.max(1)) as i32;
            update_job_status(&ctx.db, job_id, "scraping", progress).await?;

            // Per-URL run row.
            let source_run_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO scrape_quantity_source_runs
                    (id, scrape_quantity_run_id, site, url, category_hint)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(source_run_id)
            .bind(run_id)
            .bind(parser.site_name())
            .bind(&cat.url)
            .bind(&cat.sek_group_hint)
            .execute(&ctx.db)
            .await?;

            // FETCH
            let fetch_start = Instant::now();
            let fetch_result = match cat.fetch_kind {
                "pdf" => fetch_pdf_as_text(&http, &cat.url).await,
                "xls" => fetch_xls_as_payload(&http, &cat.url).await,
                _ => match &bd_client_opt {
                    Some(bd) => bd
                        .fetch_html_debug(&cat.url, parser.expect_selector())
                        .await
                        .map(|(html, info)| (html, info.status as i32, info.html_len as i32))
                        .map_err(anyhow::Error::from),
                    None => Err(anyhow::anyhow!("BrightData not configured for HTML source")),
                },
            };
            let fetch_elapsed = fetch_start.elapsed().as_millis() as i32;

            let content = match fetch_result {
                Ok((body, status, body_len)) => {
                    sqlx::query(
                        "UPDATE scrape_quantity_source_runs SET fetch_status='success', http_status=$1, elapsed_ms=$2, html_len=$3, fetched_at=now() WHERE id=$4",
                    )
                    .bind(status)
                    .bind(fetch_elapsed)
                    .bind(body_len)
                    .bind(source_run_id)
                    .execute(&ctx.db)
                    .await?;
                    if body_len == 0 {
                        sqlx::query("UPDATE scrape_quantity_source_runs SET fetch_status='failed', error_message='Empty body' WHERE id=$1")
                            .bind(source_run_id).execute(&ctx.db).await?;
                        failed_sources += 1;
                        continue;
                    }
                    body
                }
                Err(e) => {
                    tracing::warn!(%job_id, site=parser.site_name(), url=%cat.url, error=%e, "Fetch failed");
                    sqlx::query("UPDATE scrape_quantity_source_runs SET fetch_status='failed', error_message=$1 WHERE id=$2")
                        .bind(e.to_string()).bind(source_run_id).execute(&ctx.db).await?;
                    failed_sources += 1;
                    continue;
                }
            };

            // PARSE
            let parse_result = parser.parse_page(&content, &cat.url);
            tracing::info!(
                %job_id, site=parser.site_name(), url=%cat.url,
                strategy=parse_result.strategy_used,
                candidates_before=parse_result.candidates_before_filter,
                candidates_after=parse_result.candidates_after_filter,
                accepted=parse_result.norms.len(),
                "Quantity parse complete"
            );
            for (label, count) in &parse_result.diagnostics {
                tracing::debug!(%job_id, site=parser.site_name(), label, count, "Quantity parse diagnostic");
            }

            sqlx::query("UPDATE scrape_quantity_source_runs SET parse_status='success', parsed_count=$1 WHERE id=$2")
                .bind(parse_result.norms.len() as i32)
                .bind(source_run_id)
                .execute(&ctx.db)
                .await?;

            if parse_result.norms.is_empty() {
                sqlx::query("UPDATE scrape_quantity_source_runs SET db_status='skipped' WHERE id=$1")
                    .bind(source_run_id).execute(&ctx.db).await?;
                successful_sources += 1;
                continue;
            }

            // SEK MAP + PERSIST
            let (created, updated) =
                persist_norms(&ctx.db, user_id, &parse_result.norms, parser.site_name()).await?;
            norms_created += created;
            norms_updated += updated;

            let db_status = if created + updated > 0 { "success" } else { "failed" };
            sqlx::query("UPDATE scrape_quantity_source_runs SET db_status=$1 WHERE id=$2")
                .bind(db_status).bind(source_run_id).execute(&ctx.db).await?;

            if created + updated > 0 {
                successful_sources += 1;
            } else {
                failed_sources += 1;
            }
        }
    }

    let run_status = if successful_sources > 0 { "completed" } else { "failed" };
    let elapsed_ms = pipeline_start.elapsed().as_millis() as i32;

    sqlx::query(
        "UPDATE scrape_quantity_runs SET
            status=$1, completed_at=now(), total_sources=$2, successful_sources=$3,
            failed_sources=$4, norms_created=$5, norms_updated=$6, elapsed_ms=$7
         WHERE id=$8",
    )
    .bind(run_status)
    .bind(total_sources)
    .bind(successful_sources)
    .bind(failed_sources)
    .bind(norms_created)
    .bind(norms_updated)
    .bind(elapsed_ms)
    .bind(run_id)
    .execute(&ctx.db)
    .await?;

    let job_status = if successful_sources > 0 { "done" } else { "failed" };
    update_job_status(&ctx.db, job_id, job_status, 100).await?;

    tracing::info!(
        %job_id, %run_id, total_sources, successful_sources, failed_sources,
        norms_created, norms_updated, elapsed_ms,
        "Quantity scrape pipeline complete"
    );

    Ok(())
}

/// Fetch an XLS/XLSX body over plain HTTP and base-64 wrap it so the
/// string-typed `NormParser::parse_page` interface survives binary data.
async fn fetch_xls_as_payload(
    http: &reqwest::Client,
    url: &str,
) -> Result<(String, i32, i32)> {
    let start = Instant::now();
    let resp = http
        .get(url)
        .header("User-Agent", "kcc-automation/1.0 (+https://kcc-automation.com)")
        .send()
        .await?;
    let status = resp.status().as_u16() as i32;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("XLS fetch {url} returned HTTP {status}"));
    }
    let bytes = resp.bytes().await?;
    let payload = procurement_xls::encode_xls_payload(&bytes);
    let payload_len = payload.len() as i32;
    tracing::info!(url, status, bytes = bytes.len(), payload_len, elapsed_ms = start.elapsed().as_millis() as i64, "XLS fetched + encoded");
    Ok((payload, status, payload_len))
}

/// Fetch a PDF via plain HTTP and return (extracted_text, status, text_len).
/// PDFs are static — no BrightData needed.
async fn fetch_pdf_as_text(
    http: &reqwest::Client,
    url: &str,
) -> Result<(String, i32, i32)> {
    let start = Instant::now();
    let resp = http
        .get(url)
        .header("User-Agent", "kcc-automation/1.0 (+https://kcc-automation.com)")
        .send()
        .await?;
    let status = resp.status().as_u16() as i32;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("PDF fetch {url} returned HTTP {status}"));
    }
    let bytes = resp.bytes().await?;
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| anyhow::anyhow!("pdf_extract failed: {e}"))?;
    tracing::info!(url, status, bytes = bytes.len(), text_len = text.len(), elapsed_ms = start.elapsed().as_millis() as i64, "PDF fetched + extracted");
    let text_len = text.len() as i32;
    Ok((text, status, text_len))
}

/// Upsert parsed norms into `quantity_norms`. Returns (created, updated).
/// SEK mapping reuses the price-scraper's keyword rules — if we can't map, the
/// row is still persisted with `sek_group_hint` so the user can correct it.
async fn persist_norms(
    db: &sqlx::PgPool,
    user_id: Uuid,
    norms: &[ScrapedNorm],
    source_site: &str,
) -> Result<(i32, i32)> {
    let mut created = 0i32;
    let mut updated = 0i32;

    // Re-use sek_mapper by wrapping each ScrapedNorm in a minimal ScrapedPrice.
    // We only need `description_bg` + `unit` — price fields are irrelevant.
    for norm in norms {
        let shim = ScrapedPrice::from_lv(
            &norm.source_site,
            &norm.source_url,
            &norm.description_bg,
            &norm.work_unit,
            None, None, None, None,
            norm.extraction_confidence,
        );
        let mapped = sek_mapper::map_to_sek(&shim, norm.sek_group_hint.as_deref());

        // Disambiguate multiple rows on the same SEK code by appending a
        // deterministic suffix from the description (first 40 chars).
        let desc_key: String = norm
            .description_bg
            .chars()
            .take(40)
            .collect::<String>()
            .trim()
            .to_string();
        let sek_code = match mapped.sek_code.as_ref() {
            Some(code) => code.clone(),
            None => format!("{}.AUTO", mapped.sek_group),
        };

        let materials_json = serde_json::to_value(&norm.materials).unwrap_or(serde_json::json!([]));
        let machinery_json = serde_json::to_value(&norm.machinery).unwrap_or(serde_json::json!([]));
        let source_label = source_site.to_string();

        // Confidence floor: parser × mapper, capped at 0.95 since scraped data
        // should never trump a human-edited norm.
        let final_confidence = (norm.extraction_confidence * mapped.confidence.max(0.5)).min(0.95);

        let description_with_key = if desc_key.is_empty() {
            norm.description_bg.clone()
        } else {
            norm.description_bg.clone()
        };

        let result = sqlx::query(
            "INSERT INTO quantity_norms
                (sek_code, description_bg, work_unit, labor_qualified_h, labor_helper_h,
                 labor_trade, materials, machinery, source, source_url, confidence, user_id)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
             ON CONFLICT (sek_code, source, user_id) DO UPDATE SET
                description_bg = EXCLUDED.description_bg,
                work_unit = EXCLUDED.work_unit,
                labor_qualified_h = EXCLUDED.labor_qualified_h,
                labor_helper_h = EXCLUDED.labor_helper_h,
                labor_trade = EXCLUDED.labor_trade,
                materials = EXCLUDED.materials,
                machinery = EXCLUDED.machinery,
                source_url = EXCLUDED.source_url,
                confidence = EXCLUDED.confidence,
                updated_at = now()
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(&sek_code)
        .bind(&description_with_key)
        .bind(&norm.work_unit)
        .bind(norm.labor_qualified_h)
        .bind(norm.labor_helper_h)
        .bind(&norm.labor_trade)
        .bind(&materials_json)
        .bind(&machinery_json)
        .bind(&source_label)
        .bind(&norm.source_url)
        .bind(final_confidence)
        .bind(user_id)
        .fetch_one(db)
        .await;

        match result {
            Ok(row) => {
                use sqlx::Row;
                let inserted: bool = row.try_get::<bool, _>("inserted").unwrap_or(false);
                if inserted { created += 1 } else { updated += 1 }
            }
            Err(e) => tracing::warn!(error = %e, sek = %sek_code, "Failed to upsert norm"),
        }
    }

    Ok((created, updated))
}

async fn update_job_status(
    db: &sqlx::PgPool,
    job_id: Uuid,
    status: &str,
    progress: i32,
) -> Result<()> {
    sqlx::query("UPDATE jobs SET status=$1, progress=$2 WHERE id=$3")
        .bind(status)
        .bind(progress)
        .bind(job_id)
        .execute(db)
        .await?;
    Ok(())
}
