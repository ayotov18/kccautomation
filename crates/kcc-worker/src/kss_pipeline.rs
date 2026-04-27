use anyhow::Result;
use sqlx::PgPool;
use std::time::Instant;
use uuid::Uuid;

use kcc_core::fabrication::bill_graph::{self, FabricationParams};
use kcc_core::fabrication::quantity_builder;
use kcc_core::kss::audit::{self, KssAuditTrail, truncate_for_audit};
use kcc_core::kss::layer_mapper;
use kcc_core::kss::quantity_calc::{self, QuantityCalcConfig};
use kcc_core::kss::sek_mapper;
use kcc_core::kss::types::PriceList;
use kcc_report::{kss_excel, kss_pdf};

use crate::jobs::GenerateKssJob;
use crate::pipeline::{WorkerContext, download_from_s3, upload_to_s3, load_analysis_snapshot};

/// Process a KSS (Bill of Quantities) generation job.
///
/// Pipeline: load snapshot → DRM retrieve → build bill → generate KSS → DRM record
pub async fn process_kss_job(job: GenerateKssJob, ctx: &WorkerContext) -> Result<()> {
    let job_id = job.job_id;
    let drawing_id = job.drawing_id;
    let pipeline_start = Instant::now();
    let mut audit = KssAuditTrail::default();

    tracing::info!(%job_id, %drawing_id, "Starting KSS pipeline");
    update_status(&ctx.db, job_id, "extracting", 10).await?;

    // Look up user_id from drawing (needed for DRM scoping)
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM drawings WHERE id = $1")
        .bind(drawing_id)
        .fetch_one(&ctx.db)
        .await
        .map_err(|e| anyhow::anyhow!("Cannot find drawing {drawing_id}: {e}"))?;

    // Load the canonical analysis snapshot from S3
    let analysis = load_analysis_snapshot(&ctx.s3, &ctx.bucket, drawing_id)
        .await
        .map_err(|e| anyhow::anyhow!(
            "Cannot generate KSS: analysis snapshot not found for drawing {drawing_id}. \
             The drawing must be analyzed first. Error: {e}"
        ))?;

    tracing::info!(
        %job_id,
        features = analysis.features.len(),
        entities = analysis.drawing.entities.len(),
        dimensions = analysis.drawing.dimensions.len(),
        "Loaded canonical analysis snapshot"
    );

    // ── Audit: Phase 1 — Upload/Parse data ────────────────
    {
        let p1 = &mut audit.phase1_upload;
        p1.original_filename = analysis.drawing.metadata.filename.clone();
        p1.total_entities = analysis.drawing.entities.len();
        p1.dimension_count = analysis.drawing.dimensions.len();
        p1.annotation_count = analysis.drawing.annotations.len();
        p1.units_detected = format!("{:?}", analysis.drawing.units);
        p1.units_detection_method = "dxf_header".into();
        // Count entities by geometry type
        let mut type_counts = std::collections::HashMap::new();
        for e in &analysis.drawing.entities {
            let tn = match &e.geometry {
                kcc_core::geometry::model::GeometryPrimitive::Line { .. } => "Line",
                kcc_core::geometry::model::GeometryPrimitive::Arc { .. } => "Arc",
                kcc_core::geometry::model::GeometryPrimitive::Circle { .. } => "Circle",
                kcc_core::geometry::model::GeometryPrimitive::Polyline { .. } => "Polyline",
                kcc_core::geometry::model::GeometryPrimitive::Spline { .. } => "Spline",
                kcc_core::geometry::model::GeometryPrimitive::Point(_) => "Point",
            };
            *type_counts.entry(tn.to_string()).or_insert(0usize) += 1;
        }
        p1.entity_counts_by_type = type_counts;
        // Populated layers: group entities by layer, filter out zero-count.
        // The drawing metadata often lists dozens of template layers that
        // carry no geometry — don't mislead the auditor.
        let mut per_layer: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for e in &analysis.drawing.entities {
            *per_layer.entry(e.layer.clone()).or_default() += 1;
        }
        let mut populated: Vec<audit::PopulatedLayerAudit> = per_layer
            .into_iter()
            .map(|(name, entity_count)| audit::PopulatedLayerAudit { name, entity_count })
            .collect();
        populated.sort_by(|a, b| b.entity_count.cmp(&a.entity_count));
        p1.layer_count = populated.len();
        p1.layer_count_populated = populated.len();
        p1.layer_list = populated.iter().map(|pl| pl.name.clone()).collect();
        p1.populated_layers = populated;
    }

    update_status(&ctx.db, job_id, "classifying", 25).await?;

    // ── DRM Retrieval ──────────────────────────────────────
    let drm_context = match kcc_core::drm::retrieval::retrieve_context(
        &ctx.db, user_id, &analysis.drawing,
    ).await {
        Ok(ctx) => {
            tracing::info!(
                %job_id,
                total = ctx.total_matches(),
                auto_overrides = ctx.auto_override_count(),
                "DRM context retrieved"
            );
            ctx
        }
        Err(e) => {
            // DRM failure is non-fatal — fall back to rigid rules
            tracing::warn!(%job_id, error = %e, "DRM retrieval failed, using rigid rules only");
            kcc_core::drm::ContextBundle::default()
        }
    };

    update_status(&ctx.db, job_id, "classifying", 35).await?;

    // Load price list: explicit CSV list → scraped prices from DB → empty
    let price_list = if let Some(pl_id) = job.price_list_id {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT s3_key FROM price_lists WHERE id = $1"
        )
        .bind(pl_id)
        .fetch_optional(&ctx.db)
        .await?;

        if let Some((s3_key,)) = row {
            let data = download_from_s3(&ctx.s3, &ctx.bucket, &s3_key).await?;
            PriceList::from_csv(&data).unwrap_or_else(|e| {
                tracing::warn!(%job_id, error = %e, "Failed to parse price list, using empty");
                PriceList::empty()
            })
        } else {
            PriceList::empty()
        }
    } else {
        // No explicit price list — try to build one from scraped_price_rows
        build_price_list_from_scraped(&ctx.db, user_id).await
    };

    tracing::info!(%job_id, price_items = price_list.items.len(), "Price list loaded");

    // ── Audit: capture price list data ────────────────────
    if !price_list.items.is_empty() {
        let mut price_audit = audit::PriceResearchAudit::default();
        price_audit.items_parsed = price_list.items.len();
        price_audit.items_approved = price_list.items.len();
        price_audit.model_used = "price_list".into();
        price_audit.price_source_breakdown.insert(
            if job.price_list_id.is_some() { "user_csv" } else { "scraped_db" }.into(),
            price_list.items.len(),
        );
        audit.phase4_prices = Some(price_audit);
    }

    update_status(&ctx.db, job_id, "reporting", 50).await?;

    // ── Auto-detect drawing type ───────────────────────────
    // INVERTED LOGIC: default to architectural, only steel when confirmed
    let architectural_layer_count = analysis.drawing.entities.iter()
        .filter(|e| {
            let m = layer_mapper::map_layer(&e.layer);
            // Only count layers that actually map to real SEK groups (not empty skip layers)
            m.map(|l| !l.sek_group.is_empty()).unwrap_or(false)
        })
        .count();

    let has_fixture_blocks = analysis.drawing.entities.iter()
        .any(|e| e.block_ref.as_ref().map(|b| layer_mapper::map_block(b).is_some()).unwrap_or(false));

    let has_area_annotations = analysis.drawing.annotations.iter().any(|a| {
        let lower = a.text.to_lowercase();
        lower.contains("m2") || lower.contains("м2") || lower.contains("м²")
    });

    let has_steel_features = analysis.features.iter()
        .any(|f| matches!(f.feature_type, kcc_core::feature::types::FeatureType::SteelMember { .. }));

    // Steel fabrication ONLY when: steel features detected AND no architectural signals
    let is_steel_fabrication = has_steel_features
        && architectural_layer_count == 0
        && !has_fixture_blocks
        && !has_area_annotations;

    tracing::info!(
        %job_id,
        architectural_layers = architectural_layer_count,
        has_fixture_blocks,
        has_area_annotations,
        has_steel_features,
        is_steel = is_steel_fabrication,
        "Drawing type detection"
    );

    // ── Audit: Phase 2 — Analysis & Features ──────────────
    {
        let p2 = &mut audit.phase2_analysis;
        p2.drawing_type_classification = if is_steel_fabrication { "steel_fabrication" } else { "architectural" }.into();
        p2.drawing_type_reasoning = audit::DrawingTypeReasoning {
            architectural_layer_count,
            has_fixture_blocks,
            has_area_annotations,
            has_steel_features,
            is_steel_fabrication,
        };
        p2.drm_total_matches = drm_context.total_matches();
        p2.drm_auto_overrides = drm_context.auto_override_count();
        // DRM match details
        for m in drm_context.layer_mappings.iter().chain(drm_context.block_mappings.iter()) {
            p2.drm_matches.push(audit::DrmMatchAudit {
                input_key: m.input_key.clone(),
                sek_code: m.sek_code.clone().unwrap_or_default(),
                similarity: m.similarity as f32,
                confidence: m.confidence,
                action: format!("{:?}", m.action),
            });
        }
        // Feature summary
        let mut feat_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for f in &analysis.features {
            let name = format!("{:?}", std::mem::discriminant(&f.feature_type));
            *feat_counts.entry(name).or_insert(0) += 1;
        }
        p2.features_detected = feat_counts.into_iter()
            .map(|(t, c)| audit::FeatureAuditEntry { feature_type: t, count: c })
            .collect();
        // Layer→SEK mappings
        let mut seen_layers = std::collections::HashSet::new();
        for e in &analysis.drawing.entities {
            if seen_layers.insert(e.layer.clone()) {
                if let Some(m) = layer_mapper::map_layer(&e.layer) {
                    let count = analysis.drawing.entities.iter().filter(|x| x.layer == e.layer).count();
                    p2.layer_sek_mappings.push(audit::LayerSekMappingAudit {
                        layer_name: e.layer.clone(),
                        sek_group: m.sek_group.to_string(),
                        description: m.work_description_bg.to_string(),
                        entity_count: count,
                    });
                }
            }
        }
    }

    let generated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();

    let kss_report = if !is_steel_fabrication {
        // ── Architectural path (DEFAULT): layer-based quantity extraction ──
        tracing::info!(
            %job_id,
            architectural_layers = architectural_layer_count,
            "Using ARCHITECTURAL KSS path (layer-based)"
        );

        let config = QuantityCalcConfig::from_drawing(&analysis.drawing);
        let qty_start = Instant::now();
        let quantities = quantity_calc::extract_layer_quantities(&analysis.drawing, &config);
        let qty_elapsed = qty_start.elapsed().as_millis() as u64;

        // Pre-AI gate: verify the deterministic pipeline produced enough
        // trustworthy rows. If not, the job finishes with a clear status and
        // the frontend routes the user to the AI-KSS flow instead of silently
        // emitting a report full of assumed defaults.
        if !kcc_core::kss::validators::has_sufficient_geometry(&quantities) {
            tracing::warn!(
                %job_id,
                rows = quantities.len(),
                "Geometry insufficient for deterministic KSS — user should switch to AI-KSS flow"
            );
        }
        let flagged = quantities.iter().filter(|q| q.needs_review).count();
        tracing::info!(
            %job_id, total = quantities.len(), flagged,
            "Post-schema-audit row tally"
        );

        tracing::info!(
            %job_id,
            quantity_items = quantities.len(),
            "Layer-based quantities extracted"
        );
        for q in &quantities {
            tracing::info!(
                %job_id,
                sek = %q.suggested_sek_code,
                desc = %q.description,
                qty = format!("{:.2}", q.quantity),
                unit = %q.unit,
                "Quantity item"
            );
        }

        // ── Audit: Phase 3 — Quantities ───────────────────
        {
            let p3 = &mut audit.phase3_quantities;
            p3.unit_scale_factor = config.unit_scale;
            p3.unit_scale_source = if config.unit_scale == 0.001 { "mm_to_m" } else if config.unit_scale == 1.0 { "meters" } else { "custom" }.into();
            for q in &quantities {
                p3.items.push(audit::QuantityItemAudit {
                    sek_code: q.suggested_sek_code.clone(),
                    description: q.description.clone(),
                    unit: q.unit.clone(),
                    calculated_quantity: q.quantity,
                    formula_used: "layer_aggregate".into(),
                    source_layer: q.category.clone(),
                });
            }
            audit.record_timing("quantities", qty_elapsed);
        }

        sek_mapper::generate_kss_report(
            &analysis.drawing.metadata.filename,
            &generated_at,
            &quantities,
            &price_list,
        )
    } else {
        // ── Steel fabrication path: fabrication bill graph ───────
        tracing::info!(
            %job_id,
            architectural_layers = architectural_layer_count,
            "Detected FABRICATION drawing — using steel bill graph path"
        );

        let fab_params = FabricationParams::default();
        let bill = bill_graph::build_fabrication_bill(
            &analysis.features,
            &analysis.drawing,
            &fab_params,
        );

        tracing::info!(
            %job_id,
            bill_items = bill.items.len(),
            total_weight_kg = format!("{:.1}", bill.total_weight_kg),
            "Fabrication bill built"
        );

        quantity_builder::bill_graph_to_kss(
            &bill,
            &analysis.drawing.metadata.filename,
            &generated_at,
            &price_list,
        )
    };

    // ── Audit: Phase 5 baseline — rule-based stats ──────
    audit.phase5_generation.rule_based_items = kss_report.items.len();
    audit.phase5_generation.rule_based_total_eur = kss_report.totals.grand_total;

    // ── AI Agent (OpenRouter) ─────────────────────────────
    update_status(&ctx.db, job_id, "reporting", 65).await?;

    let kss_report = {
        let ai_config = kcc_core::ai::AiConfig::from_env();
        if ai_config.enabled {
            audit.phase5_generation.ai_enabled = true;
            audit.phase5_generation.mode = "ai_merged".into();
            audit.phase5_generation.ai_model = ai_config.model.clone();
            tracing::info!(%job_id, model = %ai_config.model, "AI agent enabled — calling OpenRouter");

            match kcc_core::ai::OpenRouterClient::new(&ai_config) {
                Ok(ai_client) => {
                    let user_prompt = kcc_core::ai::prompt::build_user_prompt(
                        &analysis, &drm_context, &price_list, &kss_report,
                    );

                    // Audit: capture prompts (truncated)
                    audit.phase5_generation.ai_prompt_system_preview = truncate_for_audit(
                        kcc_core::ai::prompt::SYSTEM_PROMPT, 5000,
                    );
                    audit.phase5_generation.ai_prompt_user_preview = truncate_for_audit(
                        &user_prompt, 10000,
                    );

                    let ai_start = Instant::now();
                    match ai_client.generate_kss(
                        kcc_core::ai::prompt::SYSTEM_PROMPT,
                        &user_prompt,
                    ).await {
                        Ok(ai_response) => {
                            let ai_elapsed = ai_start.elapsed().as_millis() as u64;
                            audit.phase5_generation.ai_latency_ms = ai_elapsed;
                            audit.phase5_generation.ai_items_generated = ai_response.total_items;

                            tracing::info!(
                                %job_id,
                                ai_items = ai_response.total_items,
                                ai_total = format!("{:.2}", ai_response.total_eur),
                                drawing_type = %ai_response.drawing_type,
                                warnings = ai_response.warnings.len(),
                                "AI KSS draft received"
                            );

                            // Log AI warnings
                            for w in &ai_response.warnings {
                                tracing::info!(%job_id, warning = %w, "AI warning");
                                audit.warn("generation", format!("AI: {}", w));
                            }

                            // Convert AI response to KSS report and merge
                            let ai_kss = kcc_core::ai::response::ai_response_to_kss_report(
                                &ai_response,
                                &analysis.drawing.metadata.filename,
                                &generated_at,
                            );

                            audit.phase5_generation.ai_items_validated = ai_kss.items.len();

                            let merged = kcc_core::ai::merger::merge_kss(&kss_report, &ai_kss);
                            tracing::info!(
                                %job_id,
                                rule_items = kss_report.items.len(),
                                ai_items = ai_kss.items.len(),
                                merged_items = merged.items.len(),
                                merged_total = format!("{:.2}", merged.totals.grand_total),
                                "KSS merged (rule + AI)"
                            );

                            // Log AI decisions to DRM audit
                            for item in &ai_kss.items {
                                let _ = sqlx::query(
                                    "INSERT INTO drm_audit_log (drawing_id, job_id, action, input_key, matched_sek_code, new_confidence)
                                     VALUES ($1, $2, 'ai_generated', $3, $4, $5)"
                                )
                                .bind(drawing_id)
                                .bind(job_id)
                                .bind(&item.description)
                                .bind(&item.sek_code)
                                .bind(item.total_price)
                                .execute(&ctx.db)
                                .await;
                            }

                            audit.record_timing("generation", ai_elapsed);
                            merged
                        }
                        Err(e) => {
                            tracing::warn!(%job_id, error = %e, "AI agent failed (non-fatal) — using rule-based KSS only");
                            audit.error("generation", format!("AI failed: {e}"));
                            audit.phase5_generation.mode = "rule_based".into();
                            kss_report
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(%job_id, error = %e, "AI client init failed — using rule-based KSS only");
                    audit.error("generation", format!("AI init failed: {e}"));
                    audit.phase5_generation.mode = "rule_based".into();
                    kss_report
                }
            }
        } else {
            tracing::info!(%job_id, "AI agent disabled (OPENROUTER_API_KEY not set)");
            audit.phase5_generation.mode = "rule_based".into();
            kss_report
        }
    };

    update_status(&ctx.db, job_id, "reporting", 70).await?;

    // Generate Excel
    let excel_bytes = kss_excel::generate_kss_excel(&kss_report)
        .map_err(|e| anyhow::anyhow!("Excel generation failed: {e}"))?;
    let excel_key = format!("reports/{drawing_id}/kss.xlsx");
    upload_to_s3(&ctx.s3, &ctx.bucket, &excel_key, &excel_bytes).await?;
    store_report_record(&ctx.db, drawing_id, "kss_excel", &excel_key).await?;

    update_status(&ctx.db, job_id, "reporting", 85).await?;

    // Generate PDF summary
    let pdf_bytes = kss_pdf::generate_kss_pdf(&kss_report)
        .map_err(|e| anyhow::anyhow!("KSS PDF generation failed: {e}"))?;
    let pdf_key = format!("reports/{drawing_id}/kss.pdf");
    upload_to_s3(&ctx.s3, &ctx.bucket, &pdf_key, &pdf_bytes).await?;
    store_report_record(&ctx.db, drawing_id, "kss_pdf", &pdf_key).await?;

    // ── Store KSS report data in DB for frontend display ───
    let sectioned = kcc_core::kss::types::SectionedKssReport::from_items(
        &kss_report.drawing_name, &kss_report.generated_at,
        kss_report.items.clone(), 0.20,
    );
    let report_data = serde_json::to_value(&sectioned)
        .unwrap_or_else(|_| serde_json::json!({}));

    let ai_enhanced = kcc_core::ai::AiConfig::from_env().enabled;

    let _ = sqlx::query(
        "INSERT INTO kss_reports (drawing_id, user_id, ai_enhanced, report_data, subtotal_eur, vat_eur, total_with_vat_eur, item_count, s3_key_excel, s3_key_pdf)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (drawing_id) DO UPDATE SET report_data = $4, subtotal_eur = $5, vat_eur = $6, total_with_vat_eur = $7, item_count = $8, ai_enhanced = $3, generated_at = now(), s3_key_excel = $9, s3_key_pdf = $10"
    )
    .bind(drawing_id)
    .bind(user_id)
    .bind(ai_enhanced)
    .bind(&report_data)
    .bind(sectioned.subtotal_eur)
    .bind(sectioned.vat_eur)
    .bind(sectioned.total_with_vat_eur)
    .bind(kss_report.items.len() as i32)
    .bind(&excel_key)
    .bind(&pdf_key)
    .execute(&ctx.db)
    .await;

    // Update drawing kss status
    let _ = sqlx::query(
        "UPDATE drawings SET kss_generated = true, kss_total_eur = $1 WHERE id = $2"
    )
    .bind(sectioned.total_with_vat_eur)
    .bind(drawing_id)
    .execute(&ctx.db)
    .await;

    update_status(&ctx.db, job_id, "reporting", 92).await?;

    // ── DRM Recording (passive learning) ───────────────────
    // Extract all mappings from this KSS run and store as artifacts
    let layer_mappings: Vec<(String, String, String)> = analysis.drawing.entities.iter()
        .filter_map(|e| {
            kcc_core::kss::layer_mapper::map_layer(&e.layer).map(|m| {
                (e.layer.clone(), m.sek_group.to_string(), m.work_description_bg.to_string())
            })
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let block_mappings: Vec<(String, String, String)> = analysis.drawing.entities.iter()
        .filter_map(|e| {
            e.block_ref.as_ref().and_then(|b| {
                kcc_core::kss::layer_mapper::map_block(b).map(|(sek, desc)| {
                    (b.clone(), sek.to_string(), desc.to_string())
                })
            })
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let kss_items: Vec<(String, String, String, String)> = kss_report.items.iter()
        .filter(|item| !item.sek_code.is_empty())
        .map(|item| {
            (item.description.clone(), item.sek_code.clone(), String::new(), item.unit.clone())
        })
        .collect();

    match kcc_core::drm::recorder::record_kss_artifacts(
        &ctx.db, user_id, drawing_id,
        &layer_mappings, &block_mappings, &kss_items,
    ).await {
        Ok(count) => {
            tracing::info!(%job_id, artifacts_recorded = count, "DRM artifacts saved");
            audit.phase6_report.drm_artifacts_recorded = count;
        }
        Err(e) => tracing::warn!(%job_id, error = %e, "DRM recording failed (non-fatal)"),
    }

    // ── Log DRM audit entries for any auto-overrides ───────
    for drm_match in drm_context.layer_mappings.iter()
        .chain(drm_context.block_mappings.iter())
        .filter(|m| m.action == kcc_core::drm::DrmAction::AutoOverride)
    {
        let _ = sqlx::query(
            "INSERT INTO drm_audit_log (drawing_id, job_id, action, artifact_id, input_key, matched_sek_code, similarity_score, times_confirmed, new_confidence)
             VALUES ($1, $2, 'auto_override', $3, $4, $5, $6, $7, $8)",
        )
        .bind(drawing_id)
        .bind(job_id)
        .bind(drm_match.artifact_id)
        .bind(&drm_match.input_key)
        .bind(&drm_match.sek_code)
        .bind(drm_match.similarity)
        .bind(drm_match.times_confirmed)
        .bind(drm_match.confidence)
        .execute(&ctx.db)
        .await;
    }

    // ── Audit: Phase 6 — Final Report ────────────────────
    {
        let p6 = &mut audit.phase6_report;
        p6.total_items = kss_report.items.len();
        p6.total_sections = sectioned.sections.len();
        p6.subtotal_eur = sectioned.subtotal_eur;
        p6.vat_eur = sectioned.vat_eur;
        p6.total_with_vat_eur = sectioned.total_with_vat_eur;
        p6.reports_generated = vec!["excel".into(), "pdf".into()];
    }

    // ── Store audit trail (non-fatal) ─────────────────────
    let total_elapsed = pipeline_start.elapsed().as_millis() as u64;
    audit.record_timing("total", total_elapsed);

    let user_summary = serde_json::to_value(&audit.to_user_summary()).ok();
    let audit_json = serde_json::to_value(&audit).unwrap_or_else(|_| serde_json::json!({}));

    if let Err(e) = sqlx::query(
        "INSERT INTO kss_audit_trails (drawing_id, job_id, pipeline_mode, total_duration_ms, total_warnings, total_errors, audit_data, user_summary)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(drawing_id)
    .bind(job_id)
    .bind(&audit.phase5_generation.mode)
    .bind(total_elapsed as i64)
    .bind(audit.warnings.len() as i32)
    .bind(audit.errors.len() as i32)
    .bind(&audit_json)
    .bind(&user_summary)
    .execute(&ctx.db)
    .await {
        tracing::warn!(%job_id, error = %e, "Audit trail storage failed (non-fatal)");
    }

    // Done
    update_status(&ctx.db, job_id, "done", 100).await?;
    sqlx::query("UPDATE jobs SET completed_at = now() WHERE id = $1")
        .bind(job_id)
        .execute(&ctx.db)
        .await?;

    tracing::info!(
        %job_id, %drawing_id,
        items = kss_report.items.len(),
        total = format!("{:.2}", kss_report.totals.grand_total),
        drm_matches = drm_context.total_matches(),
        drm_overrides = drm_context.auto_override_count(),
        audit_duration_ms = total_elapsed,
        "KSS generation complete"
    );

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

async fn store_report_record(db: &PgPool, drawing_id: Uuid, format: &str, s3_key: &str) -> Result<()> {
    sqlx::query("INSERT INTO reports (drawing_id, format, s3_key) VALUES ($1, $2, $3)")
        .bind(drawing_id)
        .bind(format)
        .bind(s3_key)
        .execute(db)
        .await?;
    Ok(())
}

/// Build a PriceList from scraped_price_rows in DB (user's latest scraped data).
/// Falls back to empty if no scraped prices exist.
async fn build_price_list_from_scraped(db: &PgPool, user_id: Uuid) -> PriceList {
    use kcc_core::kss::types::PriceListItem;

    let rows: Vec<(Option<String>, String, Option<String>, Option<f64>, Option<f64>)> = match sqlx::query_as(
        "SELECT sek_code, item_name, unit, price_min_eur, price_max_eur FROM scraped_price_rows WHERE user_id = $1 AND archived_at IS NULL AND sek_code IS NOT NULL ORDER BY mapping_confidence DESC"
    )
    .bind(user_id)
    .fetch_all(db)
    .await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to load scraped prices, using empty price list");
            return PriceList::empty();
        }
    };

    if rows.is_empty() {
        tracing::info!("No scraped prices found, using empty price list");
        return PriceList::empty();
    }

    let items: Vec<PriceListItem> = rows.iter().filter_map(|(sek_code, name, unit, min_eur, max_eur)| {
        let code = sek_code.as_ref()?;
        let avg = match (min_eur, max_eur) {
            (Some(min), Some(max)) => (min + max) / 2.0,
            (Some(v), None) | (None, Some(v)) => *v,
            _ => return None,
        };
        // Split into cost components (40/35/10/15 ratio)
        Some(PriceListItem {
            sek_code: code.clone(),
            description: name.clone(),
            unit: unit.clone().unwrap_or_else(|| "М2".to_string()),
            labor_price: avg * 0.40,
            material_price: avg * 0.35,
            mechanization_price: avg * 0.10,
            overhead_price: avg * 0.15,
        })
    }).collect();

    tracing::info!(scraped_prices = items.len(), "Built price list from scraped DB prices");
    PriceList { items }
}
