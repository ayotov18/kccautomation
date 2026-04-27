//! AI bilingual drawing-summary pipeline.
//!
//! Replaces the raw-stat widgets (entity distribution, annotation chips,
//! active layers grid) with a single comprehensive summary in EN and BG
//! that the operator can redact before signing.
//!
//! One Opus call returns BOTH languages in a single JSON response — this
//! is cheaper than two calls AND keeps the BG/EN content synchronized
//! (same numeric facts, same module layout).

use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::time::Instant;

use crate::jobs::AiSummaryJob;
use crate::pipeline::WorkerContext;

const SUMMARY_MODEL: &str = "anthropic/claude-opus-4.6";

#[derive(Debug, Deserialize)]
struct AiSummaryResponse {
    summary_en: String,
    summary_bg: String,
}

pub async fn process_ai_summary_job(job: AiSummaryJob, ctx: &WorkerContext) -> Result<()> {
    let drawing_id = job.drawing_id;
    let user_id = job.user_id;
    let started = Instant::now();

    tracing::info!(%drawing_id, %user_id, "Starting AI summary generation");

    // Verify ownership + load drawing meta
    let drawing: Option<(String, Option<String>, Option<i32>)> = sqlx::query_as(
        "SELECT filename, units, entity_count FROM drawings WHERE id = $1 AND user_id = $2",
    )
    .bind(drawing_id)
    .bind(user_id)
    .fetch_optional(&ctx.db)
    .await?;
    let (filename, units, entity_count) = drawing.ok_or_else(|| anyhow!("Drawing not found"))?;

    // Pull the per-drawing extracted data (same source the AI KSS pipeline reads)
    let layers: Vec<(String, i32)> = sqlx::query_as(
        "SELECT name, entity_count FROM drawing_layers
         WHERE drawing_id = $1 AND entity_count > 0
         ORDER BY entity_count DESC LIMIT 30",
    )
    .bind(drawing_id)
    .fetch_all(&ctx.db)
    .await?;

    let blocks: Vec<(String, i32)> = sqlx::query_as(
        "SELECT name, entity_count FROM drawing_blocks
         WHERE drawing_id = $1 AND entity_count > 0
         ORDER BY entity_count DESC LIMIT 25",
    )
    .bind(drawing_id)
    .fetch_all(&ctx.db)
    .await?;

    let dimensions: Vec<(f64,)> = sqlx::query_as(
        "SELECT value FROM drawing_dimensions WHERE drawing_id = $1 ORDER BY value DESC LIMIT 30",
    )
    .bind(drawing_id)
    .fetch_all(&ctx.db)
    .await?;

    let annotations: Vec<(String,)> = sqlx::query_as(
        "SELECT text FROM drawing_annotations WHERE drawing_id = $1 LIMIT 40",
    )
    .bind(drawing_id)
    .fetch_all(&ctx.db)
    .await?;

    let structures: Vec<(String, f64, f64, f64, f64, i32, i32)> = sqlx::query_as(
        "SELECT label, bbox_min_x, bbox_min_y, bbox_max_x, bbox_max_y,
                dimension_count, annotation_count
         FROM drawing_structures WHERE drawing_id = $1 ORDER BY structure_index",
    )
    .bind(drawing_id)
    .fetch_all(&ctx.db)
    .await?;

    // Detect drawing type (timber / arch / steel) from text signals
    let layer_names: Vec<String> = layers.iter().map(|(n, _)| n.clone()).collect();
    let block_names: Vec<String> = blocks.iter().map(|(n, _)| n.clone()).collect();
    let ann_strings: Vec<String> = annotations.iter().map(|(t,)| t.clone()).collect();
    let drawing_type =
        kcc_core::drawing_type::classify_from_text(&layer_names, &block_names, &ann_strings);

    // Build the prompt context (compact)
    let layer_summary = layers
        .iter()
        .map(|(n, c)| format!("{n} ({c})"))
        .collect::<Vec<_>>()
        .join(", ");
    let block_summary = blocks
        .iter()
        .map(|(n, c)| format!("{n} ({c})"))
        .collect::<Vec<_>>()
        .join(", ");
    let dim_summary = dimensions
        .iter()
        .map(|(v,)| format!("{:.2}", v))
        .collect::<Vec<_>>()
        .join(", ");
    let ann_summary = annotations
        .iter()
        .filter(|(t,)| !t.is_empty() && t != "None")
        .take(20)
        .map(|(t,)| t.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let structures_summary = if structures.is_empty() {
        "Single-module drawing.".to_string()
    } else {
        let lines: Vec<String> = structures
            .iter()
            .map(|(label, x0, y0, x1, y1, dn, an)| {
                let w = (x1 - x0).abs();
                let h = (y1 - y0).abs();
                format!(
                    "  - \"{label}\": bbox {w:.0} × {h:.0}, {dn} dims, {an} annotations"
                )
            })
            .collect();
        format!(
            "Detected {} independent module(s):\n{}",
            structures.len(),
            lines.join("\n")
        )
    };

    let system_prompt = "You are a senior Bulgarian construction engineer reviewing a freshly-uploaded CAD drawing. Produce two SHORT (~180–260 words each) prose summaries — one in English, one in Bulgarian — covering: drawing type & purpose, structural system, key dimensions, modules detected, and what's missing or worth flagging. Write naturally, like a brief written for the project owner. No bullet headers, no emoji, no SaaS marketing tone. Use Markdown only for **bold** of critical numbers and module names. Output ONLY a JSON object: {\"summary_en\": \"...\", \"summary_bg\": \"...\"}.";

    let user_prompt = format!(
        "DRAWING METADATA\n\
        File: {filename}\n\
        Detected type: {dtype}\n\
        Units: {units}\n\
        Total entities: {entity_count}\n\n\
        STRUCTURES\n{structures_summary}\n\n\
        LAYERS (top 30, with entity counts)\n{layer_summary}\n\n\
        BLOCKS (top 25)\n{block_summary}\n\n\
        DIMENSION VALUES (top 30)\n{dim_summary}\n\n\
        ANNOTATIONS (sample)\n{ann_summary}\n\n\
        Write the summaries now.",
        dtype = drawing_type.as_str(),
        units = units.as_deref().unwrap_or("Unknown"),
        entity_count = entity_count.unwrap_or(0),
    );

    let ai_config = kcc_core::ai::AiConfig::from_env();
    let mut config = ai_config;
    config.model = SUMMARY_MODEL.to_string();
    config.timeout_secs = 120;
    let client = kcc_core::ai::OpenRouterClient::new(&config)?;

    let raw = client.complete_json(system_prompt, &user_prompt).await?;
    // Extract a single JSON value (model may emit fences or extra prose)
    let json_str = kcc_core::ai::extract_json_payload(&raw);
    let parsed: AiSummaryResponse = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => {
            let repaired = kcc_core::ai::repair_truncated_json(&json_str);
            serde_json::from_str(&repaired)
                .map_err(|e| anyhow!("AI summary JSON parse failed: {e}"))?
        }
    };

    sqlx::query(
        "UPDATE drawings
         SET ai_summary_en = $1,
             ai_summary_bg = $2,
             ai_summary_generated_at = now(),
             ai_summary_edited_at = NULL,
             ai_summary_model = $3
         WHERE id = $4",
    )
    .bind(&parsed.summary_en)
    .bind(&parsed.summary_bg)
    .bind(SUMMARY_MODEL)
    .bind(drawing_id)
    .execute(&ctx.db)
    .await?;

    sqlx::query("UPDATE jobs SET status = 'done', completed_at = now() WHERE id = $1")
        .bind(job.job_id)
        .execute(&ctx.db)
        .await?;

    let elapsed = started.elapsed().as_millis();
    tracing::info!(
        %drawing_id,
        en_len = parsed.summary_en.len(),
        bg_len = parsed.summary_bg.len(),
        elapsed_ms = elapsed as u64,
        "AI summary stored",
    );

    Ok(())
}
