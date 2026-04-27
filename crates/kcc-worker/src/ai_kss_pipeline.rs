//! AI-Full KSS Pipeline — Perplexity research + Redis storage + Opus generation.
//!
//! Phase 1 (Research): Perplexity searches for prices → writes to Redis HASH per item
//! Phase 2 (Review): Frontend reads/edits items via Redis → no Postgres during review
//! Phase 3 (Generate): Opus reads reviewed items from Redis → generates KSS → writes final to Postgres

use anyhow::Result;
use uuid::Uuid;
use std::collections::HashMap;
use std::time::Instant;

use kcc_core::kss::audit::{self, KssAuditTrail, truncate_for_audit};

use crate::jobs::AiKssJob;
use crate::pipeline::WorkerContext;

const PERPLEXITY_MODEL: &str = "perplexity/sonar-pro";
const OPUS_MODEL: &str = "anthropic/claude-opus-4.6";
const SESSION_TTL_SECS: u64 = 86400; // 24 hours

/// Process an AI KSS job (either research or generate phase).
pub async fn process_ai_kss_job(job: AiKssJob, ctx: &WorkerContext) -> Result<()> {
    match job.phase.as_str() {
        "research" => run_research_phase(job, ctx).await,
        "generate" => run_generation_phase(job, ctx).await,
        _ => Err(anyhow::anyhow!("Unknown AI KSS phase: {}", job.phase)),
    }
}

/// Phase 1: Perplexity researches prices → writes each item to Redis HASH.
async fn run_research_phase(job: AiKssJob, ctx: &WorkerContext) -> Result<()> {
    let session_id = job.session_id;
    let drawing_id = job.drawing_id;
    let user_id = job.user_id;
    let pipeline_start = Instant::now();

    // Load the user's configured pricing defaults — these get injected into
    // the Perplexity prompt as authoritative constraints (no more hardcoded
    // 30% profit). Falls back to canonical BG 2026 values if unconfigured.
    let defaults = crate::pricing_defaults::PricingDefaults::load_for_user(&ctx.db, user_id).await;
    tracing::info!(
        %session_id, %user_id, currency = %defaults.currency,
        profit_pct = defaults.profit_pct, contingency_pct = defaults.contingency_pct,
        "Loaded user pricing defaults"
    );

    tracing::info!(%session_id, %drawing_id, "Starting AI KSS research phase");

    // Connect to Redis
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = redis::Client::open(redis_url)?;
    let mut conn = redis_client.get_multiplexed_async_connection().await?;

    // Set session status in Redis
    set_redis_field(&mut conn, &session_id, "status", "researching").await?;
    set_redis_field(&mut conn, &session_id, "progress", "10").await?;
    set_redis_field(&mut conn, &session_id, "model", PERPLEXITY_MODEL).await?;
    set_redis_field(&mut conn, &session_id, "drawing_id", &drawing_id.to_string()).await?;

    // Load drawing data from POSTGRES (not S3)
    let layers: Vec<(String, i32)> = sqlx::query_as(
        "SELECT name, entity_count FROM drawing_layers WHERE drawing_id = $1 ORDER BY entity_count DESC"
    ).bind(drawing_id).fetch_all(&ctx.db).await?;

    let annotations: Vec<(String,)> = sqlx::query_as(
        "SELECT text FROM drawing_annotations WHERE drawing_id = $1"
    ).bind(drawing_id).fetch_all(&ctx.db).await?;

    let dimensions: Vec<(f64,)> = sqlx::query_as(
        "SELECT value FROM drawing_dimensions WHERE drawing_id = $1"
    ).bind(drawing_id).fetch_all(&ctx.db).await?;

    let blocks: Vec<(String, i32)> = sqlx::query_as(
        "SELECT name, entity_count FROM drawing_blocks WHERE drawing_id = $1 ORDER BY entity_count DESC"
    ).bind(drawing_id).fetch_all(&ctx.db).await?;

    let drawing_meta: Option<(String, Option<String>, Option<i32>)> = sqlx::query_as(
        "SELECT filename, units, entity_count FROM drawings WHERE id = $1"
    ).bind(drawing_id).fetch_optional(&ctx.db).await?;

    let (filename, _units, entity_count) = drawing_meta.unwrap_or(("unknown".into(), None, None));

    set_redis_field(&mut conn, &session_id, "progress", "20").await?;

    // Build Perplexity research prompt from DB data
    let layer_summary: String = layers.iter()
        .filter(|(_, count)| *count > 0)
        .take(20)
        .map(|(name, count)| format!("  {} ({} entities)", name, count))
        .collect::<Vec<_>>()
        .join("\n");

    let ann_summary: String = annotations.iter()
        .filter(|(t,)| !t.is_empty() && t != "None")
        .take(40)
        .map(|(t,)| format!("  \"{}\"", t))
        .collect::<Vec<_>>()
        .join("\n");

    let dim_summary: String = dimensions.iter()
        .take(40)
        .map(|(v,)| format!("{:.2}", v))
        .collect::<Vec<_>>()
        .join(", ");

    let block_summary: String = blocks.iter()
        .take(30)
        .map(|(name, count)| format!("  {} ({} entities)", name, count))
        .collect::<Vec<_>>()
        .join("\n");

    // Detect drawing type from layer / block / annotation TEXT — not from
    // feature counts. The geometric feature extractor produced 250 false
    // "Steel Members" on a wooden cabin floor plan because every wall is
    // a parallel-line pair; using its output to pick the price branch sent
    // Perplexity hunting for IPE/HEB prices on a KVH/BSH timber drawing.
    let layer_names: Vec<String> = layers.iter().map(|(n, _)| n.clone()).collect();
    let block_names: Vec<String> = blocks.iter().map(|(n, _)| n.clone()).collect();
    let ann_strings: Vec<String> = annotations.iter().map(|(t,)| t.clone()).collect();
    let drawing_type = kcc_core::drawing_type::classify_from_text(
        &layer_names, &block_names, &ann_strings,
    );
    tracing::info!(
        %session_id, drawing_type = drawing_type.as_str(),
        layer_n = layers.len(), block_n = blocks.len(), ann_n = annotations.len(),
        "Detected drawing type from text signals"
    );

    // Fail-fast: refuse to ship a fabricated KSS for a drawing whose
    // parser output is too sparse to support a real takeoff. The user is
    // better served by an explicit "drawing data insufficient" error than
    // by Opus filling in plausible numbers.
    let populated_layer_count = layers.iter().filter(|(_, c)| *c > 0).count();
    let total_signal = populated_layer_count
        .saturating_add(blocks.len())
        .saturating_add(annotations.len());
    if populated_layer_count <= 2 && annotations.is_empty() && total_signal < 6 {
        let msg = format!(
            "Drawing data is too sparse to generate a meaningful KSS \
             ({populated_layer_count} populated layers, {} blocks, {} annotations, \
             {} dimensions). The DXF parse may have lost geometry, or the source DWG \
             is dimensions-only — re-upload a working drawing with walls/doors/windows on named layers.",
            blocks.len(), annotations.len(), dimensions.len(),
        );
        tracing::warn!(%session_id, %msg, "AI KSS aborting — geometry-poor drawing");
        return Err(anyhow::anyhow!(msg));
    }

    use kcc_core::drawing_type::DrawingType;
    let price_categories = match drawing_type {
        DrawingType::Steel => "This is a STEEL FABRICATION drawing. Search for these Bulgarian prices:\n\
         1. Стоманени профили IPE/HEB/UPN — цена на кг доставка + монтаж\n\
         2. Заваръчни работи — цена на м.л.\n\
         3. Болтови съединения — цена на бр.\n\
         4. Антикорозионна защита — цена на М2\n\
         5. Кофраж за стоманобетонни елементи — цена на М2\n\
         6. Бетон клас C25/30 — цена на М3\n\
         7. Армировка стомана B500B — цена на кг\n\
         8. Стоманени плочи/ламарина — цена на кг\n\
         9. Монтаж на стоманена конструкция с кран — цена на тон\n\
         10. Транспорт на стоманена конструкция",
        DrawingType::Timber => "This is a WOODEN / TIMBER FRAME construction drawing (KVH / BSH columns and beams, OSB / plywood sheathing, mineral-wool insulation, sheet-metal façade & roof). Search for these Bulgarian prices:\n\
         1. КVH конструкционна дървесина (10×12, 10×16, 8×10, 10×20) — цена на м³ доставка + монтаж\n\
         2. BSH ламелирана дървесина — цена на м³ доставка + монтаж\n\
         3. OSB плочи 10/12/18 mm — цена на м² доставка + монтаж\n\
         4. Шперплат 18 mm — цена на м² доставка + монтаж\n\
         5. Минерална / каменна вата 10–12 cm — цена на м² доставка + монтаж\n\
         6. Топлоизолация неопор (EPS) 10–12 cm — цена на м² доставка + монтаж\n\
         7. Ламарина за фасада и покрив (поцинкована, прахово боядисана) — цена на м² доставка + монтаж\n\
         8. PVC 5-камерна дограма — цена на м² доставка + монтаж\n\
         9. Вътрешни врати — цена на бр. доставка + монтаж\n\
         10. Лепило за фасадни панели — цена на м²\n\
         11. Летвена скара (фасада) — цена на м²\n\
         12. Дъски 25 mm обшивка — цена на м³\n\
         13. Паропропусклива фолио (~125 г/м²) — цена на м²\n\
         14. Конструктивни планки и крепежни елементи — цена на бр.\n\
         15. Стълба и второ ниво — цена на бр./комплект\n\
         16. Транспорт",
        DrawingType::Architectural | DrawingType::Mechanical | DrawingType::Unknown =>
            "This is an ARCHITECTURAL drawing. Search for these Bulgarian prices:\n\
         1. Тухлена зидария / газобетон — цена на М2\n\
         2. Вътрешна мазилка — цена на М2\n\
         3. Латексово боядисване — цена на М2\n\
         4. Подови настилки (ламинат/плочки) — цена на М2\n\
         5. Покривни работи — цена на М2\n\
         6. Хидроизолация — цена на М2\n\
         7. Топлоизолация EPS/XPS — цена на М2\n\
         8. PVC/алуминиева дограма — цена на М2 или бр.\n\
         9. ВиК — мивки, тоалетни, вани — цена на бр.\n\
         10. Транспорт",
    };

    let currency_symbol = defaults.currency_symbol();
    let labor_anchors = defaults.labor_anchors_bg();
    let currency_code = defaults.currency.clone();
    let entity_n = entity_count.unwrap_or(0);
    // Currency line used to tell Perplexity which unit to return.
    let currency_header = format!(
        "Валута: {currency_symbol} ({currency_code}).\n{labor_anchors}\n"
    );

    // Load quantity norms for the SEK groups relevant to this drawing type.
    // Built-ins (user_id IS NULL) + the user's personal norms are both considered.
    // Format as prompt anchors so Perplexity / Opus has concrete consumption
    // numbers instead of hallucinating (e.g. "1 м2 zidariya needs 25 tuhli
    // + 0.02 m3 разтвор", not a guess).
    let relevant_groups: &[&str] = match drawing_type {
        DrawingType::Steel => &["СЕК03", "СЕК04", "СЕК14", "СЕК15", "СЕК16"],
        DrawingType::Timber => &["СЕК18", "СЕК03", "СЕК09", "СЕК10", "СЕК11", "СЕК12", "СЕК13"],
        DrawingType::Architectural | DrawingType::Mechanical | DrawingType::Unknown =>
            &["СЕК01", "СЕК02", "СЕК03", "СЕК04", "СЕК05", "СЕК06", "СЕК07", "СЕК08", "СЕК09", "СЕК10"],
    };
    let quantity_norms_block = load_quantity_norms_block(&ctx.db, user_id, relevant_groups).await;
    let research_prompt = format!(
        "{price_categories}\n\n\
         {currency_header}\n\
         ===== DRAWING EVIDENCE (use this to refine quantities and item selection) =====\n\
         File: {filename}  ·  {entity_n} entities  ·  detected type: {dtype}\n\
         Active layers (top 20):\n{layer_summary}\n\
         Block instances (top 30 — block names usually identify wall/door/window/furniture/profile types):\n{block_summary}\n\
         Annotations (first 40 — these are the literal text labels on the drawing):\n{ann_summary}\n\
         Dimension values present (sample):\n{dim_summary}\n\n\
         Use the layer/block/annotation text above to identify the actual building system before pricing — \
         e.g. layers like 'A-CONSTR-Wood' or blocks containing 'KVH'/'OSB' mean a timber-frame cabin, \
         not concrete or masonry.\n\n\
         ===== RULES FOR EVERY PRICE =====\n\
         1. Every price MUST be the FULL unit cost including BOTH material AND labor — \
         the single number a Bulgarian contractor would quote to a client on Образец 9.1.\n\
         2. Do NOT return labor-only or material-only prices. If a source gives only one, \
         combine it with typical market labor/material for that work.\n\
         3. Provide FOUR numeric prices per item: material_price_lv, labor_price_lv, \
         price_min_lv (market floor), price_max_lv (market ceiling). \
         Invariant: price_min_lv ≤ material_price_lv + labor_price_lv ≤ price_max_lv. \
         price_min_lv must be strictly less than price_max_lv (a real range, not a single value).\n\
         4. Provide source_url (direct link to the page where this price was confirmed). \
         Prefer 2+ sources averaged; cite the most representative one.\n\
         5. All prices in {currency_code} without ДДС. If source is in a different currency, convert (1 EUR = 1.95583 лв).\n\
         6. Current 2025-2026 prices only. Reject outdated data.\n\n\
         ===== QUANTITY / CONSUMPTION NORMS (use these to validate your material & labor per unit) =====\n\
         {quantity_norms_block}\n\
         Rules:\n\
         - If the user's norm specifies X kg cement per м2 plaster, your per-unit material cost MUST reflect that (do not halve it, do not double it).\n\
         - Labor hours below are from Bulgarian УСН / АТС Прес. Convert to labor cost using the labor band for that trade.\n\
         - If you lack a norm for an item, state confidence ≤ 0.65 and derive conservatively from market tender averages.\n\n\
         ===== SANITY ANCHORS (reject any item whose total falls outside these bands) =====\n\
         (Bands below are in EUR per unit; scale proportionally if currency is BGN.)\n\
         - Газобетонна зидария 25см: 18–33 €/М2 total (material 10–18, labor 8–13)\n\
         - Тухлена зидария 25см: 20–36 €/М2 total (material 11–20, labor 9–15)\n\
         - Вътрешна варо-циментова мазилка: 5–11 €/М2 total\n\
         - Външна фасадна мазилка (бяла): 7–14 €/М2 total\n\
         - Латексово боядисване двукратно: 3.5–7 €/М2 total\n\
         - Ламинат клас 32, монтаж: 18–36 €/М2 total\n\
         - Гранитогрес 60x60 + монтаж: 28–56 €/М2 total\n\
         - Хидроизолация битумна мембрана 4мм: 9–18 €/М2 total\n\
         - EPS 10см фасадна топлоизолация с мрежа и лепило: 18–31 €/М2 total\n\
         - PVC 5-камерна дограма с монтаж: 112–194 €/М2 total\n\
         - Стоманени профили IPE доставка + монтаж: 1.8–2.8 €/кг total\n\
         - КVH конструкционна дървесина (10×12, 10×16, 8×10): 1500–1900 €/м³ total (material 600–800, labor 800–1100)\n\
         - BSH ламелирана дървесина (10×20): 1700–2200 €/м³ total\n\
         - OSB плочи 18 mm доставка + монтаж: 11–17 €/м² total\n\
         - OSB плочи 10 mm доставка + монтаж: 8–13 €/м² total\n\
         - Шперплат мебелен 18 mm доставка + монтаж: 30–46 €/м² total\n\
         - Минерална/каменна вата 10–12 cm: 11–22 €/м² total\n\
         - Топлоизолация неопор (EPS) 12 cm: 13–22 €/м² total\n\
         - Лепило за фасадни панели: 22–34 €/м² total\n\
         - Летвена скара 3×4 cm: 5–7 €/м² total\n\
         - Дъски 2.5 cm обшивка: 250–320 €/м³ total\n\
         - Паропропусклива фолио (~125 г/м²): 3–5 €/м² total\n\
         - Ламарина за фасада/покрив (поцинкована, прахово боядисана): 70–95 €/м² total\n\
         - Конструктивни планки/крепежни елементи: 10–14 €/бр.\n\n\
         ===== OUTPUT JSON — EXACT SHAPE =====\n\
         {{\n\
           \"categories\": [\n\
             {{\n\
               \"sek_group\": \"СЕК05\",\n\
               \"items\": [\n\
                 {{\n\
                   \"sek_code\": \"СЕК05.007\",\n\
                   \"description\": \"Доставка и монтаж газобетонна зидария 25см\",\n\
                   \"unit\": \"М2\",\n\
                   \"material_price_lv\": 14.0,\n\
                   \"labor_price_lv\": 10.0,\n\
                   \"price_min_lv\": 20.0,\n\
                   \"price_max_lv\": 30.0,\n\
                   \"source_url\": \"https://daibau.bg/ceni/gazobeton\",\n\
                   \"confidence\": 0.85,\n\
                   \"notes\": \"Средна пазарна цена от 3 източника\"\n\
                 }}\n\
               ]\n\
             }}\n\
           ],\n\
           \"overhead\": {{\n\
             \"admin_rate_pct\": 0.0,\n\
             \"contingency_rate_pct\": {contg},\n\
             \"delivery_storage_rate_pct\": {delivery},\n\
             \"profit_rate_pct\": {profit}\n\
           }},\n\
           \"transport_lv\": {transport}\n\
         }}\n\n\
         Return 10-20 items minimum, grouped by СЕК group. Every item MUST have all four price fields AND a source_url.",
        contg = defaults.contingency_pct,
        delivery = defaults.dr_materials_pct,
        profit = defaults.profit_pct,
        transport = defaults.transport_slab_eur,
        quantity_norms_block = quantity_norms_block,
        dtype = drawing_type.as_str(),
        block_summary = block_summary,
        ann_summary = ann_summary,
        dim_summary = dim_summary,
    );

    set_redis_field(&mut conn, &session_id, "progress", "30").await?;

    // Call Perplexity DIRECTLY via OpenRouter — no :online plugin, Perplexity has native search
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .map_err(|_| anyhow::anyhow!("OPENROUTER_API_KEY not set"))?;

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let system_prompt = "You are a Bulgarian construction price researcher. Search the web for current construction work prices in Bulgaria (лв/BGN). Output ONLY valid JSON — no markdown fences, no explanation outside JSON.";

    let perplexity_body = serde_json::json!({
        "model": PERPLEXITY_MODEL,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": research_prompt }
        ],
        "temperature": 0.1,
        "max_tokens": 16384
    });

    tracing::info!(%session_id, model = PERPLEXITY_MODEL, "Calling Perplexity directly (native search)");

    // Call with retry — Perplexity sometimes returns truncated responses
    let mut content = String::new();
    for attempt in 1..=2u32 {
        let resp = http_client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://kcc-automation.com")
            .header("X-OpenRouter-Title", "KCC Price Research")
            .json(&perplexity_body)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if status != 200 {
            let body = resp.text().await.unwrap_or_default();
            if attempt < 2 {
                tracing::warn!(%session_id, status, attempt, "Perplexity returned non-200, retrying");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
            return Err(anyhow::anyhow!("Perplexity API error (status {}): {}", status, &body[..body.len().min(500)]));
        }

        let raw_body = resp.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&raw_body)
            .map_err(|e| anyhow::anyhow!("Perplexity response JSON parse failed: {e}"))?;

        content = response_json
            .get("choices").and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let model_used = response_json.get("model").and_then(|m| m.as_str()).unwrap_or("unknown");
        tracing::info!(%session_id, model_used, content_len = content.len(), attempt, "Perplexity response received");

        // Validate minimum response length — retry if truncated
        if content.len() >= 500 {
            break;
        }
        if attempt < 2 {
            tracing::warn!(%session_id, content_len = content.len(), "Perplexity response too short, retrying");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    if content.len() < 100 {
        return Err(anyhow::anyhow!("Perplexity returned insufficient data ({} bytes)", content.len()));
    }

    set_redis_field(&mut conn, &session_id, "progress", "70").await?;

    // Parse response — strip reasoning tags + markdown fences, try parse, repair if needed
    let json_str = kcc_core::ai::extract_json_payload(&content);
    let preview: String = content.chars().take(500).collect();
    let parsed: serde_json::Value = match serde_json::Deserializer::from_str(&json_str)
        .into_iter::<serde_json::Value>()
        .next()
    {
        Some(Ok(val)) => val,
        Some(Err(e)) => {
            tracing::warn!(%session_id, error = %e, raw_preview = %preview, "JSON parse failed, attempting repair");
            // Repair: close unclosed strings/arrays/objects, remove trailing commas
            let repaired = kcc_core::ai::repair_truncated_json(&json_str);
            serde_json::from_str(&repaired).map_err(|e2| {
                anyhow::anyhow!(
                    "Perplexity JSON repair also failed: {e2}\n--- raw content (first 500 chars) ---\n{preview}"
                )
            })?
        }
        None => {
            return Err(anyhow::anyhow!(
                "Perplexity returned no JSON\n--- raw content (first 500 chars) ---\n{preview}"
            ));
        }
    };

    let mut item_count = 0;

    // Debug: log what Perplexity actually returned so we can see the structure
    tracing::info!(%session_id, keys = ?parsed.as_object().map(|o| o.keys().collect::<Vec<_>>()), "Perplexity response structure");

    if let Some(categories) = parsed.get("categories").and_then(|c| c.as_array()) {
        for category in categories {
            let sek_group = category.get("sek_group").and_then(|s| s.as_str()).unwrap_or("");
            let sek_order = sek_group_order(sek_group);

            if let Some(items) = category.get("items").and_then(|i| i.as_array()) {
                for item in items {
                    let item_id = Uuid::new_v4().to_string();
                    let key = format!("kcc:ai:{}:item:{}", session_id, item_id);

                    // Read each priced field. Missing min/max become a ±15% synthetic
                    // range around the total so the UI never shows Min == Max.
                    let mat_price = item.get("material_price_lv").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let lab_price = item.get("labor_price_lv").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let total = mat_price + lab_price;
                    let price_min_raw = item.get("price_min_lv").and_then(|v| v.as_f64());
                    let price_max_raw = item.get("price_max_lv").and_then(|v| v.as_f64());
                    let (price_min, price_max) = match (price_min_raw, price_max_raw) {
                        (Some(lo), Some(hi)) if lo < hi => (lo, hi),
                        (Some(lo), Some(hi)) if lo >= hi && total > 0.0 => {
                            // Degenerate (min==max or inverted) — synthesize a spread around total
                            (total * 0.85, total * 1.15)
                        }
                        (Some(lo), None) if total > 0.0 => (lo, total * 1.15),
                        (None, Some(hi)) if total > 0.0 => (total * 0.85, hi),
                        _ if total > 0.0 => (total * 0.85, total * 1.15),
                        _ => (0.0, 0.0),
                    };
                    // Final guard: price_min_raw == price_max_raw fallthrough case already handled above.
                    let notes = item.get("notes").and_then(|v| v.as_str()).unwrap_or("");

                    redis::cmd("HSET").arg(&key)
                        .arg("sek_group").arg(sek_group)
                        .arg("sek_code").arg(item.get("sek_code").and_then(|s| s.as_str()).unwrap_or(""))
                        .arg("description").arg(item.get("description").and_then(|s| s.as_str()).unwrap_or(""))
                        .arg("unit").arg(item.get("unit").and_then(|s| s.as_str()).unwrap_or("М2"))
                        .arg("material_price_lv").arg(mat_price.to_string())
                        .arg("labor_price_lv").arg(lab_price.to_string())
                        .arg("price_lv").arg(total.to_string())
                        .arg("price_min_lv").arg(price_min.to_string())
                        .arg("price_max_lv").arg(price_max.to_string())
                        .arg("source_url").arg(item.get("source_url").and_then(|s| s.as_str()).unwrap_or(""))
                        .arg("notes").arg(notes)
                        .arg("confidence").arg(item.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5).to_string())
                        .arg("approved").arg("true")
                        .arg("edited").arg("false")
                        .query_async::<()>(&mut conn).await?;

                    // Add to sorted set for ordering
                    redis::cmd("ZADD")
                        .arg(format!("kcc:ai:{}:items", session_id))
                        .arg(sek_order as f64)
                        .arg(&item_id)
                        .query_async::<()>(&mut conn).await?;

                    // Set TTL
                    redis::cmd("EXPIRE").arg(&key).arg(SESSION_TTL_SECS)
                        .query_async::<()>(&mut conn).await?;

                    item_count += 1;
                }
            }
        }
    }

    if item_count == 0 {
        // Log the first 500 chars of parsed JSON to diagnose empty results
        let preview = serde_json::to_string(&parsed).unwrap_or_default();
        let safe_len = {
            let target = preview.len().min(500);
            let mut end = target;
            while end > 0 && !preview.is_char_boundary(end) { end -= 1; }
            end
        };
        tracing::warn!(%session_id, json_preview = &preview[..safe_len], "Perplexity returned 0 parseable items — check JSON structure");
    }

    // Set sorted set TTL
    redis::cmd("EXPIRE")
        .arg(format!("kcc:ai:{}:items", session_id))
        .arg(SESSION_TTL_SECS)
        .query_async::<()>(&mut conn).await?;

    set_redis_field(&mut conn, &session_id, "progress", "100").await?;
    set_redis_field(&mut conn, &session_id, "status", "ready").await?;

    // Also update Postgres session record
    sqlx::query("UPDATE ai_kss_sessions SET status = 'ready', research_model = $1, updated_at = now() WHERE id = $2")
        .bind(PERPLEXITY_MODEL)
        .bind(session_id)
        .execute(&ctx.db)
        .await?;

    // ── Store AUDIT TRAIL for research phase ────────────────
    let research_elapsed = pipeline_start.elapsed().as_millis() as u64;
    let mut audit = KssAuditTrail::default();

    // Phase 1: Upload/Parse data from DB
    audit.phase1_upload.original_filename = filename.clone();
    audit.phase1_upload.total_entities = entity_count.unwrap_or(0) as usize;
    audit.phase1_upload.layer_count = layers.len();
    audit.phase1_upload.layer_list = layers.iter().map(|(n, _)| n.clone()).collect();
    audit.phase1_upload.dimension_count = dimensions.len();
    audit.phase1_upload.annotation_count = annotations.len();
    audit.phase1_upload.units_detected = "from_db".into();

    // Phase 2: Analysis — drawing type detection (now from layer / block /
    // annotation text via kcc_core::drawing_type, not from extracted features).
    audit.phase2_analysis.drawing_type_classification = drawing_type.as_str().into();
    audit.phase2_analysis.drawing_type_reasoning = audit::DrawingTypeReasoning {
        has_steel_features: matches!(drawing_type, DrawingType::Steel),
        ..Default::default()
    };
    let feature_counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT feature_type, COUNT(*) FROM features WHERE drawing_id = $1 GROUP BY feature_type"
    ).bind(drawing_id).fetch_all(&ctx.db).await.unwrap_or_default();
    for (ft, count) in &feature_counts {
        audit.phase2_analysis.features_detected.push(audit::FeatureAuditEntry {
            feature_type: ft.clone(),
            count: *count as usize,
        });
    }

    // Phase 4: Price Research — the main payload
    let mut price_audit = audit::PriceResearchAudit::default();
    price_audit.model_used = PERPLEXITY_MODEL.to_string();
    price_audit.prompt_preview = truncate_for_audit(&research_prompt, 10000);
    price_audit.response_preview = truncate_for_audit(&content, 10000);
    price_audit.latency_ms = research_elapsed;
    price_audit.items_parsed = item_count;
    price_audit.items_approved = item_count; // all start approved
    // Extract source URLs from parsed items
    if let Some(cats) = parsed.get("categories").and_then(|c| c.as_array()) {
        for cat in cats {
            if let Some(items) = cat.get("items").and_then(|i| i.as_array()) {
                for item in items {
                    if let Some(url) = item.get("source_url").and_then(|s| s.as_str()) {
                        if !url.is_empty() && !price_audit.sources_cited.contains(&url.to_string()) {
                            price_audit.sources_cited.push(url.to_string());
                        }
                    }
                }
            }
        }
    }
    price_audit.price_source_breakdown.insert("perplexity_research".into(), item_count);
    audit.phase4_prices = Some(price_audit);
    audit.phase5_generation.mode = "ai_full_research".into();
    audit.record_timing("research", research_elapsed);

    // Store audit
    let user_summary = serde_json::to_value(&audit.to_user_summary()).ok();
    let audit_json = serde_json::to_value(&audit).unwrap_or_else(|_| serde_json::json!({}));

    // Get job_id for the audit record
    let audit_job_id = job.job_id;
    if let Err(e) = sqlx::query(
        "INSERT INTO kss_audit_trails (drawing_id, job_id, pipeline_mode, total_duration_ms, total_warnings, total_errors, audit_data, user_summary)
         VALUES ($1, $2, 'ai_full_research', $3, $4, $5, $6, $7)"
    )
    .bind(drawing_id)
    .bind(audit_job_id)
    .bind(research_elapsed as i64)
    .bind(audit.warnings.len() as i32)
    .bind(audit.errors.len() as i32)
    .bind(&audit_json)
    .bind(&user_summary)
    .execute(&ctx.db)
    .await {
        tracing::warn!(%session_id, error = %e, "Research audit trail storage failed (non-fatal)");
    }

    tracing::info!(%session_id, item_count, audit_ms = research_elapsed, "AI KSS research complete — items in Redis, audit stored");
    Ok(())
}

/// Phase 3: Read reviewed items from Redis → Opus generates KSS → write final to Postgres.
async fn run_generation_phase(job: AiKssJob, ctx: &WorkerContext) -> Result<()> {
    let session_id = job.session_id;
    let drawing_id = job.drawing_id;
    let user_id = job.user_id;

    let gen_start = Instant::now();
    tracing::info!(%session_id, %drawing_id, "Starting AI KSS generation phase (Opus)");

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = redis::Client::open(redis_url)?;
    let mut conn = redis_client.get_multiplexed_async_connection().await?;

    set_redis_field(&mut conn, &session_id, "status", "generating").await?;
    set_redis_field(&mut conn, &session_id, "model", OPUS_MODEL).await?;
    set_redis_field(&mut conn, &session_id, "progress", "10").await?;

    // Read all APPROVED items from Redis
    let item_ids: Vec<String> = redis::cmd("ZRANGE")
        .arg(format!("kcc:ai:{}:items", session_id))
        .arg(0i64).arg(-1i64)
        .query_async(&mut conn).await?;

    let mut reviewed_items = Vec::new();
    for item_id in &item_ids {
        let fields: HashMap<String, String> = redis::cmd("HGETALL")
            .arg(format!("kcc:ai:{}:item:{}", session_id, item_id))
            .query_async(&mut conn).await?;

        if fields.get("approved").map(|v| v == "true").unwrap_or(false) {
            reviewed_items.push(fields);
        }
    }

    tracing::info!(%session_id, approved_items = reviewed_items.len(), "Read reviewed items from Redis");

    // Load detected structures (modules). Multi-module sheets get one Opus
    // call per structure; single-module drawings (or pre-structure-detection
    // legacy drawings) collapse to one virtual "whole drawing" structure.
    let structures: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, label FROM drawing_structures WHERE drawing_id = $1 ORDER BY structure_index ASC"
    ).bind(drawing_id).fetch_all(&ctx.db).await?;

    let target_structures: Vec<(Option<Uuid>, String)> = if structures.is_empty() {
        vec![(None, "Whole drawing".to_string())]
    } else {
        structures.iter().map(|(id, label)| (Some(*id), label.clone())).collect()
    };
    tracing::info!(
        %session_id, n_structures = target_structures.len(),
        labels = ?target_structures.iter().map(|(_, l)| l.clone()).collect::<Vec<_>>(),
        "Generating KSS per structure",
    );

    // Reused: drawing-wide layers/dims/annotations as a fallback for
    // single-module drawings or legacy data.
    let all_layers: Vec<(String, i32)> = sqlx::query_as(
        "SELECT name, entity_count FROM drawing_layers WHERE drawing_id = $1 ORDER BY entity_count DESC"
    ).bind(drawing_id).fetch_all(&ctx.db).await?;
    let all_dimensions: Vec<(f64,)> = sqlx::query_as(
        "SELECT value FROM drawing_dimensions WHERE drawing_id = $1"
    ).bind(drawing_id).fetch_all(&ctx.db).await?;
    let all_annotations: Vec<(String,)> = sqlx::query_as(
        "SELECT text FROM drawing_annotations WHERE drawing_id = $1"
    ).bind(drawing_id).fetch_all(&ctx.db).await?;

    set_redis_field(&mut conn, &session_id, "progress", "25").await?;

    // Build Opus prompt with reviewed price data (material + labor split)
    let price_data: String = reviewed_items.iter()
        .map(|item| format!(
            "- {} [{}]: материал={} лв, труд={} лв, общо={}-{} лв/{} (source: {})",
            item.get("description").unwrap_or(&String::new()),
            item.get("sek_group").unwrap_or(&String::new()),
            item.get("material_price_lv").unwrap_or(&"0".to_string()),
            item.get("labor_price_lv").unwrap_or(&"0".to_string()),
            item.get("price_min_lv").unwrap_or(&"0".to_string()),
            item.get("price_max_lv").unwrap_or(&"0".to_string()),
            item.get("unit").unwrap_or(&"М2".to_string()),
            item.get("source_url").unwrap_or(&String::new()),
        ))
        .collect::<Vec<_>>()
        .join("\n");

    set_redis_field(&mut conn, &session_id, "progress", "40").await?;

    // Generation mode: ai (default) | rag | hybrid. Drives the per-structure
    // dispatch below. We default to "ai" to preserve legacy behaviour for
    // jobs queued before this field was added.
    let mode = job
        .mode
        .as_deref()
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_else(|| "ai".to_string());
    let mode = match mode.as_str() {
        "rag" | "hybrid" | "ai" => mode,
        _ => "ai".to_string(),
    };
    tracing::info!(%session_id, %mode, "Generation mode selected");

    // Re-detect drawing type from text signals — drives the canonical RAG
    // category list per structure. Same logic as research phase.
    let layer_names: Vec<String> = all_layers.iter().map(|(n, _)| n.clone()).collect();
    let block_names: Vec<String> = Vec::new(); // not loaded in generation
    let ann_strings: Vec<String> = all_annotations.iter().map(|(t,)| t.clone()).collect();
    let drawing_type = kcc_core::drawing_type::classify_from_text(
        &layer_names,
        &block_names,
        &ann_strings,
    );

    // Call Opus via OpenRouter — use longer timeout for full KSS generation
    let ai_config = kcc_core::ai::AiConfig::from_env();
    let mut opus_config = ai_config;
    opus_config.model = OPUS_MODEL.to_string();
    opus_config.timeout_secs = 180; // 3 minutes — Opus needs time for large structured output
    let client = kcc_core::ai::OpenRouterClient::new(&opus_config)?;
    let system_prompt = kcc_core::ai::prompt::SYSTEM_PROMPT;

    // Per-structure generation. We collect each structure's items into a flat
    // tagged list and merge the warnings. The aggregate report is then built
    // and persisted as a single kss_report whose line items each carry the
    // structure_id they came from.
    type TaggedItem = (Option<Uuid>, String, kcc_core::kss::types::KssLineItem);
    let mut all_tagged_items: Vec<TaggedItem> = Vec::new();
    let mut all_warnings: Vec<String> = Vec::new();
    let mut detected_drawing_type = String::new();
    let mut detected_language = String::new();
    let n_structs = target_structures.len();

    for (struct_idx, (sid_opt, label)) in target_structures.iter().enumerate() {
        // Per-structure data filtering. When sid_opt is None, fall back to the
        // drawing-wide aggregates.
        let (layer_info, dim_info, ann_info) = if let Some(sid) = sid_opt {
            // Layers themselves are drawing-wide, but dim/annotation rows ARE
            // tagged. We emit drawing-wide layers (the AI uses them as a
            // selection menu) and per-structure dimensions + annotations.
            let dims: Vec<(f64,)> = sqlx::query_as(
                "SELECT value FROM drawing_dimensions WHERE drawing_id = $1 AND structure_id = $2"
            ).bind(drawing_id).bind(*sid).fetch_all(&ctx.db).await?;
            let anns: Vec<(String,)> = sqlx::query_as(
                "SELECT text FROM drawing_annotations WHERE drawing_id = $1 AND structure_id = $2"
            ).bind(drawing_id).bind(*sid).fetch_all(&ctx.db).await?;
            let layer_info = all_layers.iter()
                .filter(|(_, c)| *c > 0).take(15)
                .map(|(n, c)| format!("{}: {} entities", n, c))
                .collect::<Vec<_>>().join(", ");
            let dim_info = dims.iter().take(15)
                .map(|(v,)| format!("{:.2}", v))
                .collect::<Vec<_>>().join(", ");
            let ann_info = anns.iter()
                .filter(|(t,)| !t.is_empty() && t != "None").take(10)
                .map(|(t,)| t.clone())
                .collect::<Vec<_>>().join(", ");
            (layer_info, dim_info, ann_info)
        } else {
            let layer_info = all_layers.iter()
                .filter(|(_, c)| *c > 0).take(15)
                .map(|(n, c)| format!("{}: {} entities", n, c))
                .collect::<Vec<_>>().join(", ");
            let dim_info = all_dimensions.iter().take(15)
                .map(|(v,)| format!("{:.2}", v))
                .collect::<Vec<_>>().join(", ");
            let ann_info = all_annotations.iter()
                .filter(|(t,)| !t.is_empty() && t != "None").take(10)
                .map(|(t,)| t.clone())
                .collect::<Vec<_>>().join(", ");
            (layer_info, dim_info, ann_info)
        };

        // Per-structure header tells Opus exactly which module it's pricing.
        let module_header = if n_structs > 1 {
            format!(
                "===== MODULE {idx} of {total}: {label} =====\n\
                 This drawing contains {total} independent floor-plan modules laid out side-by-side. \
                 You are now generating the КСС for module \"{label}\". \
                 Use ONLY the dimensions and annotations listed below — they belong to this module. \
                 Do NOT invent quantities for other modules: each module gets its own KSS pass.\n\n",
                idx = struct_idx + 1, total = n_structs, label = label,
            )
        } else {
            String::new()
        };

        let opus_prompt = format!(
            "{module_header}Generate a complete Bulgarian КСС (Количествено-Стойностна Сметка) in Образец 9.1 format.\n\n\
             Drawing layers: {layer_info}\n\
             Dimensions: {dim_info}\n\
             Annotations: {ann_info}\n\n\
             USER-REVIEWED PRICE DATA (use these as authoritative):\n{price_data}\n\n\
             Generate all applicable KSS sections (I-XXIII) with quantities estimated from the drawing geometry and prices from the reviewed data.\n\n\
             ===== GEOMETRY CONFIDENCE — STRICT HALLUCINATION CONTROLS =====\n\
             Every quantity row carries `geometry_confidence` (0.0–1.0) and `extraction_method` derived from the deterministic extractor. You MUST respect them:\n\
             - geometry_confidence >= 0.80  (polyline_shoelace | block_instance_count | linear_polyline | text_annotation):\n\
                 Numbers came from measured polylines or counted blocks. TRUST the quantity AS-IS. \
                 Your only job is pricing. Set item.confidence = geometry_confidence.\n\
             - geometry_confidence 0.60-0.80 (wall_area_from_centerline | derived_from_primary):\n\
                 Length/count is real, height or ratio is an assumption. Keep quantity AS-IS; \
                 set item.confidence = 0.65 and note the assumption in `reasoning` \
                 (e.g. \"wall area assumes 2.8 m height\").\n\
             - geometry_confidence < 0.60  (wall_volume_from_centerline | assumed_default | ai_inferred):\n\
                 Do NOT fabricate a \"better\" number. Preserve the original quantity, set \
                 needs_review = true, set item.confidence = geometry_confidence, and write \
                 a short suggestion in `reasoning`. Do NOT guess concrete volumes from floor area. Do NOT \
                 hallucinate rebar or formwork totals.\n\
             - extraction_method == \"assumed_default\": same rules as confidence < 0.60 — treat \
                 as a flag, not a fact.\n\
             If an item has no geometric source at all, do NOT emit a fresh row. Fifteen \
             trustworthy items are better than thirty plausible-looking ones.\n\n\
             ===== REQUIRED PER-ITEM TRACEABILITY FIELDS =====\n\
             For EVERY item you emit, include:\n\
               \"source_layer\": <one of the drawing layers you used — must appear in the layers list above, or the literal string \"none\" if purely inferred>\n\
               \"source_annotation\": <optional, e.g. \"21m2\" if derived from a text annotation>\n\
               \"extraction_basis\": one of \"layer_geometry\" | \"annotation\" | \"assumed_typical\"\n\
             If source_layer == \"none\" AND extraction_basis == \"assumed_typical\", confidence MUST be ≤ 0.5.\n\n\
             CONFIDENCE SCORING FALLBACK (when the extractor did NOT provide a confidence):\n\
             - 0.9: Quantity from drawing geometry/dimensions/blocks directly\n\
             - 0.7: Estimated from related drawing data with inference\n\
             - 0.5: Assumed from building type with SOME indirect evidence\n\
             - 0.3: Assumed from building type with NO evidence in drawing\n\
             Items with confidence < 0.7 are 'suggestions' needing user approval.\n\n\
             Output JSON matching: {{ \"kss_sections\": [...], \"total_items\": N, \"total_lv\": N, \"drawing_type\": \"...\", \"language_detected\": \"...\", \"warnings\": [...] }}"
        );

        let progress = 40 + (struct_idx as i32 * 40 / n_structs.max(1) as i32);
        set_redis_field(&mut conn, &session_id, "progress", &progress.to_string()).await?;

        // RAG / hybrid path: pull line items from the user's price corpus
        // before (or instead of) calling Opus. RAG-only never calls Opus;
        // hybrid uses corpus matches first and lets Opus fill the gaps.
        let mut rag_items: Vec<kcc_core::kss::types::KssLineItem> = Vec::new();
        let mut covered_descriptions: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        if mode == "rag" || mode == "hybrid" {
            rag_items = generate_rag_items_for_structure(&ctx.db, user_id, drawing_type)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(%session_id, error = %e, "RAG search failed, falling back to AI");
                    Vec::new()
                });
            for it in &rag_items {
                covered_descriptions.insert(canonical_desc_key(&it.description));
            }
            tracing::info!(
                %session_id, structure = %label, mode = %mode,
                rag_matched = rag_items.len(),
                "RAG corpus retrieval complete",
            );
        }

        // RAG-only mode skips Opus entirely. We still need a synthetic
        // ai_response so the audit trail records the structure.
        let ai_response = if mode == "rag" {
            kcc_core::ai::AiKssResponse {
                kss_sections: Vec::new(),
                overhead: kcc_core::ai::AiKssOverhead::default(),
                total_items: rag_items.len(),
                construction_subtotal_lv: rag_items.iter().map(|i| i.total_price).sum(),
                total_lv: rag_items.iter().map(|i| i.total_price).sum(),
                drawing_type: drawing_type.as_str().to_string(),
                language_detected: "bg".to_string(),
                warnings: if rag_items.is_empty() {
                    vec!["RAG mode produced no items — your price library does not yet contain matches for this drawing's typical work. Re-run in 'AI' or 'Both' mode, or upload a more representative offer first.".into()]
                } else {
                    Vec::new()
                },
            }
        } else {
            tracing::info!(
                %session_id, structure = %label, idx = struct_idx + 1, total = n_structs, %mode,
                "Calling Opus for structure",
            );
            client.generate_kss(system_prompt, &opus_prompt).await?
        };

        tracing::info!(
            %session_id, structure = %label, %mode,
            ai_items = ai_response.total_items,
            total_lv = format!("{:.2}", ai_response.total_lv),
            warnings = ai_response.warnings.len(),
            "KSS generation complete for structure",
        );

        if detected_drawing_type.is_empty() {
            detected_drawing_type = ai_response.drawing_type.clone();
        }
        if detected_language.is_empty() {
            detected_language = ai_response.language_detected.clone();
        }
        for w in &ai_response.warnings {
            all_warnings.push(if n_structs > 1 {
                format!("[{}] {}", label, w)
            } else {
                w.clone()
            });
        }

        let per_struct_kss = kcc_core::ai::response::ai_response_to_kss_report(
            &ai_response,
            &format!("AI KSS - drawing {} - {}", drawing_id, label),
            &chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        );

        // Hybrid: keep RAG items as the authoritative price source when their
        // description matches an AI-emitted item; otherwise add the AI item.
        // RAG-only: discard whatever per_struct_kss produced (it was a
        // synthetic empty AiResponse anyway) and use rag_items directly.
        let mut struct_items: Vec<kcc_core::kss::types::KssLineItem> = match mode.as_str() {
            "rag" => rag_items.clone(),
            "hybrid" => {
                let mut combined = rag_items.clone();
                for ai_item in per_struct_kss.items {
                    let key = canonical_desc_key(&ai_item.description);
                    if !covered_descriptions.contains(&key) {
                        let mut it = ai_item.clone();
                        it.provenance = "rag+ai_fallback".to_string();
                        combined.push(it);
                    }
                }
                combined
            }
            _ => per_struct_kss.items, // mode == "ai"
        };

        // Renumber within structure
        for (i, it) in struct_items.iter_mut().enumerate() {
            it.item_no = i + 1;
        }
        for item in struct_items {
            all_tagged_items.push((*sid_opt, label.clone(), item));
        }
    }

    set_redis_field(&mut conn, &session_id, "progress", "80").await?;

    // Synthesize an aggregate AiResponse from all per-structure items so the
    // existing report-building / validation / persistence path keeps working.
    let aggregate_items: Vec<kcc_core::kss::types::KssLineItem> = all_tagged_items
        .iter().map(|(_, _, it)| it.clone()).collect();
    let total_lv: f64 = aggregate_items.iter().map(|i| i.total_price).sum();
    let ai_response = kcc_core::ai::AiKssResponse {
        kss_sections: Vec::new(), // not used downstream; we use ai_kss directly
        overhead: kcc_core::ai::AiKssOverhead::default(),
        total_items: aggregate_items.len(),
        construction_subtotal_lv: total_lv,
        total_lv,
        drawing_type: detected_drawing_type,
        language_detected: detected_language,
        warnings: all_warnings.clone(),
    };

    tracing::info!(
        %session_id,
        n_structures = n_structs,
        ai_items = ai_response.total_items,
        total_lv = format!("{:.2}", ai_response.total_lv),
        warnings = ai_response.warnings.len(),
        "All structures complete — aggregating",
    );

    // Write warnings to Redis LIST
    for warning in &ai_response.warnings {
        redis::cmd("RPUSH")
            .arg(format!("kcc:ai:{}:warnings", session_id))
            .arg(warning)
            .query_async::<()>(&mut conn).await?;
    }
    redis::cmd("EXPIRE")
        .arg(format!("kcc:ai:{}:warnings", session_id))
        .arg(SESSION_TTL_SECS)
        .query_async::<()>(&mut conn).await?;

    // ── FINALIZE: Write to Postgres (permanent storage) ────
    // Convert AI response to KSS report and store as normalized rows

    let ai_kss_items = aggregate_items.clone();
    let ai_kss_totals = kcc_core::kss::types::KssReport::compute_totals(&ai_kss_items);
    let ai_kss = kcc_core::kss::types::KssReport {
        drawing_name: format!("AI KSS - drawing {}", drawing_id),
        generated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        items: ai_kss_items,
        totals: ai_kss_totals,
    };

    // Create kss_reports record (mode = 'ai_full').
    // Phase 4: apply the user's configured VAT rate instead of hardcoded 20 %,
    // run sanity validators, and tag renovation reports.
    let gen_defaults = crate::pricing_defaults::PricingDefaults::load_for_user(&ctx.db, user_id).await;
    let vat_ratio = gen_defaults.vat_rate_pct / 100.0;
    let overheads = kcc_core::kss::types::KssOverheads {
        contingency_pct:        gen_defaults.contingency_pct,
        delivery_storage_pct:   gen_defaults.dr_materials_pct,
        profit_pct:             gen_defaults.profit_pct,
    };
    let report_id = Uuid::new_v4();
    let sectioned = kcc_core::kss::types::SectionedKssReport::from_items_full(
        &ai_kss.drawing_name, &ai_kss.generated_at,
        ai_kss.items.clone(), vat_ratio, overheads,
    );

    // Run sanity cross-checks (rebar/concrete ratio, missing mandatory
    // sections, zero-quantity items). Stored on the report so the frontend
    // can surface them as warning banners.
    let warnings = kcc_core::kss::validators::validate(&sectioned);
    let warnings_json = serde_json::to_value(&warnings).unwrap_or_default();
    let is_renovation = ai_kss
        .items
        .iter()
        .any(|i| kcc_core::kss::obrazec91_catalog::is_renovation_code(&i.sek_code));

    sqlx::query(
        "INSERT INTO kss_reports
           (id, drawing_id, user_id, ai_enhanced, report_data,
            subtotal_lv, vat_lv, total_with_vat_lv, item_count,
            mode, status, validation_warnings, is_renovation,
            smr_subtotal_lv, contingency_lv, delivery_storage_lv, profit_lv,
            pre_vat_total_lv, final_total_lv, totals_formula_version)
         VALUES
           ($1, $2, $3, true, $4,
            $5, $6, $7, $8,
            'ai_full', 'draft', $9, $10,
            $11, $12, $13, $14,
            $15, $16, 'v1')"
    )
    .bind(report_id)
    .bind(drawing_id)
    .bind(user_id)
    .bind(serde_json::to_value(&sectioned).unwrap_or_default())
    .bind(sectioned.cost_ladder.smr_subtotal)
    .bind(sectioned.cost_ladder.vat)
    .bind(sectioned.cost_ladder.final_total)
    .bind(ai_kss.items.len() as i32)
    .bind(&warnings_json)
    .bind(is_renovation)
    .bind(sectioned.cost_ladder.smr_subtotal)
    .bind(sectioned.cost_ladder.contingency)
    .bind(sectioned.cost_ladder.delivery_storage)
    .bind(sectioned.cost_ladder.profit)
    .bind(sectioned.cost_ladder.pre_vat_total)
    .bind(sectioned.cost_ladder.final_total)
    .execute(&ctx.db)
    .await?;

    tracing::info!(
        %report_id, warning_count = warnings.len(), %is_renovation,
        "KSS report validated"
    );

    // Pre-commit schema audit on the AI payload. Rows that fail a contract
    // test are flagged but kept — the human then sees them in the
    // "needs review" widget. This is the ai_full equivalent of the pre-AI
    // schema gate that runs on the rule-based lane.
    let mut ai_items_as_quantity: Vec<kcc_core::kss::types::QuantityItem> = ai_kss
        .items
        .iter()
        .map(|li| kcc_core::kss::types::QuantityItem {
            category: li.sek_code.clone(),
            description: li.description.clone(),
            unit: li.unit.clone(),
            quantity: li.quantity,
            suggested_sek_code: li.sek_code.clone(),
            source_entity_id: None,
            source_layer: None,
            centroid: None,
            extraction_method: kcc_core::kss::types::ExtractionMethod::AiInferred,
            geometry_confidence: li.confidence,
            needs_review: li.confidence < 0.70,
        })
        .collect();
    let schema_audit = kcc_core::kss::validators::validate_schema(&mut ai_items_as_quantity);
    tracing::info!(
        %session_id,
        ai_rows = schema_audit.total_rows,
        passed = schema_audit.passed,
        flagged = schema_audit.needs_review,
        "Post-AI schema audit"
    );
    for w in &schema_audit.violations {
        tracing::warn!(%session_id, check = %w.check, msg = %w.message, "Post-AI schema violation");
    }

    // Write each KSS item as a normalized row in kss_line_items.
    // Each row is tagged with its structure_id/structure_label so the frontend
    // can render one tab per module and the recap rolls up by structure.
    for (idx, item) in ai_kss.items.iter().enumerate() {
        let qi_flags = ai_items_as_quantity.get(idx); // carries needs_review + confidence
        let unit_price = if item.quantity > 0.0 { item.total_price / item.quantity } else { 0.0 };
        let obrazec_ref = kcc_core::kss::obrazec91_catalog::lookup_obrazec_ref(&item.sek_code);
        let item_is_renovation = kcc_core::kss::obrazec91_catalog::is_renovation_code(&item.sek_code);
        let sek_group = if let Some(dot) = item.sek_code.find('.') {
            &item.sek_code[..dot]
        } else {
            item.sek_code.as_str()
        };
        let ewc = if item_is_renovation {
            kcc_core::kss::obrazec91_catalog::ewc_code_for_sek(sek_group)
        } else {
            None
        };
        let (struct_id, struct_label): (Option<Uuid>, Option<String>) = all_tagged_items
            .get(idx)
            .map(|(sid, label, _)| (*sid, Some(label.clone())))
            .unwrap_or((None, None));
        let audit = serde_json::json!({
            "provenance": item.provenance,
            "confidence": item.confidence,
            "reasoning": item.reasoning,
            "build_up": {
                "material_price": item.material_price,
                "labor_price": item.labor_price,
                "mechanization_price": item.mechanization_price,
                "overhead_price": item.overhead_price,
                "total_price": item.total_price,
            },
            "sek_code": item.sek_code,
            "obrazec_ref": obrazec_ref,
            "structure_id": struct_id.map(|u| u.to_string()),
            "structure_label": struct_label,
            "generated_at": chrono::Utc::now().to_rfc3339(),
        });
        sqlx::query(
            "INSERT INTO kss_line_items (report_id, section_number, section_title, item_no, sek_code, description, unit, quantity, unit_price_lv, total_lv, labor_price, material_price, mechanization_price, overhead_price, confidence, reasoning, provenance, obrazec_ref, is_renovation, ewc_waste_code, audit_trail, source_entity_id, source_layer, centroid_x, centroid_y, extraction_method, geometry_confidence, needs_review, structure_id, structure_label)
             VALUES ($1, '', '', $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28)"
        )
        .bind(report_id)
        .bind(item.item_no as i32)
        .bind(&item.sek_code)
        .bind(&item.description)
        .bind(&item.unit)
        .bind(item.quantity)
        .bind(unit_price)
        .bind(item.total_price)
        .bind(item.labor_price)
        .bind(item.material_price)
        .bind(item.mechanization_price)
        .bind(item.overhead_price)
        .bind(item.confidence)
        .bind(&item.reasoning)
        .bind(&item.provenance)
        .bind(obrazec_ref)
        .bind(item_is_renovation)
        .bind(ewc)
        .bind(&audit)
        .bind(&item.source_entity_id)
        .bind(&item.source_layer)
        .bind(item.centroid_x)
        .bind(item.centroid_y)
        // Every AI-only row gets stamped ai_inferred so the frontend pill
        // and the suggestions widget can surface them uniformly.
        .bind("ai_inferred")
        // Geometry-side confidence mirrors the AI's self-reported confidence
        // since there IS no geometry for the ai_full lane.
        .bind(qi_flags.map(|q| q.geometry_confidence).unwrap_or(item.confidence))
        // needs_review = true whenever confidence < 0.70 OR the schema audit
        // flagged the row.
        .bind(qi_flags.map(|q| q.needs_review).unwrap_or(item.confidence < 0.70))
        .bind(struct_id)
        .bind(struct_label)
        .execute(&ctx.db)
        .await?;
    }

    // Update session status
    set_redis_field(&mut conn, &session_id, "status", "complete").await?;
    set_redis_field(&mut conn, &session_id, "progress", "100").await?;

    sqlx::query("UPDATE ai_kss_sessions SET status = 'complete', generation_model = $1, updated_at = now() WHERE id = $2")
        .bind(OPUS_MODEL)
        .bind(session_id)
        .execute(&ctx.db)
        .await?;

    // Record DRM artifacts
    let kss_items: Vec<(String, String, String, String)> = ai_kss.items.iter()
        .filter(|i| !i.sek_code.is_empty())
        .map(|i| (i.description.clone(), i.sek_code.clone(), String::new(), i.unit.clone()))
        .collect();

    let _ = kcc_core::drm::recorder::record_kss_artifacts(
        &ctx.db, user_id, drawing_id,
        &[], &[], &kss_items,
    ).await;

    // ── Store AUDIT TRAIL for generation phase ──────────────
    let gen_elapsed = gen_start.elapsed().as_millis() as u64;
    let mut audit = KssAuditTrail::default();

    // Phase 1: drawing data used — pulled from the canonical `drawings` row
    // and the normalised child tables, so the audit reflects the real parse.
    let drawing_row: Option<(String, Option<String>, Option<i32>, Option<String>, Option<i32>, Option<String>)> = sqlx::query_as(
        "SELECT filename, units, entity_count, dwg_version, insert_units_raw, original_format
         FROM drawings WHERE id = $1",
    )
    .bind(drawing_id)
    .fetch_optional(&ctx.db)
    .await
    .unwrap_or(None);

    if let Some((filename, units, entity_count, dwg_version, insert_units_raw, original_format)) = drawing_row {
        audit.phase1_upload.original_filename = filename;
        audit.phase1_upload.total_entities = entity_count.unwrap_or(0) as usize;
        audit.phase1_upload.dxf_version = dwg_version.unwrap_or_default();
        audit.phase1_upload.original_format = original_format.unwrap_or_default();
        audit.phase1_upload.units_detected = units.unwrap_or_else(|| "unknown".into());
        audit.phase1_upload.units_detection_method = match insert_units_raw {
            Some(0) => "heuristic (INSUNITS=0, unitless)".into(),
            Some(code) => format!("dxf_header (INSUNITS={code})"),
            None => "no_header".into(),
        };
    } else {
        audit.phase1_upload.original_filename = format!("drawing-{}", drawing_id);
    }

    // `all_layers` came from drawing_layers. Each row already carries its
    // entity_count — so layer_count_total == all_layers.len() and
    // layer_count_populated = count where entity_count > 0.
    let populated: Vec<kcc_core::kss::audit::PopulatedLayerAudit> = all_layers
        .iter()
        .filter(|(_, c)| *c > 0)
        .map(|(name, c): &(String, i32)| kcc_core::kss::audit::PopulatedLayerAudit {
            name: name.clone(),
            entity_count: *c as usize,
        })
        .collect();
    audit.phase1_upload.layer_count = all_layers.len();
    audit.phase1_upload.layer_count_populated = populated.len();
    audit.phase1_upload.layer_list = all_layers
        .iter()
        .map(|(n, _): &(String, i32)| n.clone())
        .collect();
    audit.phase1_upload.populated_layers = populated;
    audit.phase1_upload.dimension_count = all_dimensions.len();
    audit.phase1_upload.annotation_count = all_annotations.len();

    // Per-structure summaries: bbox + dim/ann counts + KSS subtotals so the
    // frontend's audit drawer can render one row per detected module.
    if !structures.is_empty() {
        let bboxes: Vec<(Uuid, f64, f64, f64, f64)> = sqlx::query_as(
            "SELECT id, bbox_min_x, bbox_min_y, bbox_max_x, bbox_max_y
             FROM drawing_structures WHERE drawing_id = $1 ORDER BY structure_index ASC",
        )
        .bind(drawing_id)
        .fetch_all(&ctx.db)
        .await
        .unwrap_or_default();
        let bbox_lookup: HashMap<Uuid, (f64, f64, f64, f64)> = bboxes
            .iter()
            .map(|(id, x0, y0, x1, y1)| (*id, (*x0, *y0, *x1, *y1)))
            .collect();

        for (idx, (sid, label)) in structures.iter().enumerate() {
            let dim_n: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM drawing_dimensions WHERE drawing_id = $1 AND structure_id = $2"
            ).bind(drawing_id).bind(*sid).fetch_one(&ctx.db).await.unwrap_or(0);
            let ann_n: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM drawing_annotations WHERE drawing_id = $1 AND structure_id = $2"
            ).bind(drawing_id).bind(*sid).fetch_one(&ctx.db).await.unwrap_or(0);
            let subtotal: Option<f64> = sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_lv), 0) FROM kss_line_items
                 WHERE report_id = $1 AND structure_id = $2",
            )
            .bind(report_id)
            .bind(*sid)
            .fetch_one(&ctx.db)
            .await
            .ok();
            let line_n: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM kss_line_items WHERE report_id = $1 AND structure_id = $2"
            ).bind(report_id).bind(*sid).fetch_one(&ctx.db).await.unwrap_or(0);
            let bbox = bbox_lookup.get(sid).copied().unwrap_or((0.0, 0.0, 0.0, 0.0));
            audit.structures.push(kcc_core::kss::audit::StructureAudit {
                structure_id: Some(sid.to_string()),
                structure_index: idx,
                label: label.clone(),
                bbox_min_x: bbox.0,
                bbox_min_y: bbox.1,
                bbox_max_x: bbox.2,
                bbox_max_y: bbox.3,
                entity_count: 0,
                dimension_count: dim_n as usize,
                annotation_count: ann_n as usize,
                line_item_count: line_n as usize,
                subtotal_lv: subtotal.unwrap_or(0.0),
            });
        }
    }

    // Semantic feature-type counts (extracted features, not raw DXF entity
    // types — the ai_full path doesn't load the raw entity list). These are
    // more useful than an empty map: the auditor sees which feature kinds
    // (Hole, SteelMember, Surface) survived extraction.
    if let Ok(rows) = sqlx::query_as::<_, (String, i64)>(
        "SELECT feature_type, COUNT(*)::bigint
         FROM features
         WHERE drawing_id = $1
         GROUP BY feature_type",
    )
    .bind(drawing_id)
    .fetch_all(&ctx.db)
    .await
    {
        for (ty, count) in rows {
            audit.phase1_upload
                .entity_counts_by_type
                .insert(ty, count as usize);
        }
    }

    // Phase 4: price data from Redis review
    let mut price_audit = audit::PriceResearchAudit::default();
    price_audit.items_parsed = item_ids.len();
    price_audit.items_approved = reviewed_items.len();
    price_audit.items_rejected = item_ids.len() - reviewed_items.len();
    price_audit.price_source_breakdown.insert("user_reviewed".into(), reviewed_items.len());
    // Store actual reviewed price data for traceability
    for item in &reviewed_items {
        let desc = item.get("description").cloned().unwrap_or_default();
        let src = item.get("source_url").cloned().unwrap_or_default();
        if !src.is_empty() && !price_audit.sources_cited.contains(&src) {
            price_audit.sources_cited.push(src);
        }
        let _ = desc; // used for logging above
    }
    audit.phase4_prices = Some(price_audit);

    // Phase 5: Generation — the core data
    let p5 = &mut audit.phase5_generation;
    p5.mode = "ai_full".into();
    p5.ai_enabled = true;
    p5.ai_model = OPUS_MODEL.to_string();
    p5.ai_prompt_system_preview = truncate_for_audit(system_prompt, 5000);
    // Per-structure prompts vary; keep a placeholder summary mentioning the
    // module count rather than dumping any one structure's prompt.
    p5.ai_prompt_user_preview = truncate_for_audit(
        &format!(
            "Generated KSS across {} structure(s): {}",
            n_structs,
            target_structures
                .iter()
                .map(|(_, l)| l.clone())
                .collect::<Vec<_>>()
                .join(", "),
        ),
        2000,
    );
    p5.ai_latency_ms = gen_elapsed;
    p5.ai_items_generated = ai_response.total_items;
    p5.ai_items_validated = ai_kss.items.len();

    // Splitter contribution log — explains "+1 / +6 row" deltas so the
    // auditor knows what produced the extra validated rows vs what the AI
    // emitted raw.
    let generated = ai_response.total_items as i64;
    let validated = ai_kss.items.len() as i64;
    let delta = validated - generated;
    if delta != 0 {
        let sek_groups: Vec<String> = ai_kss.items.iter()
            .map(|i| {
                let dot = i.sek_code.find('.').unwrap_or(i.sek_code.len());
                i.sek_code[..dot].to_string()
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        p5.row_splitter_log.push(kcc_core::kss::audit::SplitterContributionAudit {
            source: "ai_response_normaliser".into(),
            added_rows: delta.unsigned_abs() as usize,
            sek_groups,
        });
    }

    // Post-AI schema audit findings summarised on the audit record.
    for w in &schema_audit.violations {
        p5.post_ai_schema_violations.push(format!("{}: {}", w.check, w.message));
    }

    // Capture AI warnings
    for w in &ai_response.warnings {
        audit.warn("generation", w.clone());
    }

    // Phase 6: Final report — read from the canonical cost ladder so these
    // numbers match whatever the UI displays, byte for byte.
    let p6 = &mut audit.phase6_report;
    p6.total_items = ai_kss.items.len();
    p6.total_sections = sectioned.sections.len();
    p6.subtotal_bgn = sectioned.cost_ladder.smr_subtotal;
    p6.vat_bgn = sectioned.cost_ladder.vat;
    p6.total_with_vat_bgn = sectioned.cost_ladder.final_total;
    p6.reports_generated = vec!["postgres_kss_reports".into(), "postgres_kss_line_items".into()];

    // Phase 3: Quantity data — from reviewed items, capture what went in
    for item in &reviewed_items {
        audit.phase3_quantities.items.push(audit::QuantityItemAudit {
            sek_code: item.get("sek_code").cloned().unwrap_or_default(),
            description: item.get("description").cloned().unwrap_or_default(),
            unit: item.get("unit").cloned().unwrap_or("М2".into()),
            calculated_quantity: 0.0, // AI determines quantities
            formula_used: "ai_estimated".into(),
            source_layer: item.get("sek_group").cloned().unwrap_or_default(),
        });
    }

    audit.record_timing("generation", gen_elapsed);

    // Store audit
    let user_summary = serde_json::to_value(&audit.to_user_summary()).ok();
    let audit_json = serde_json::to_value(&audit).unwrap_or_else(|_| serde_json::json!({}));

    if let Err(e) = sqlx::query(
        "INSERT INTO kss_audit_trails (drawing_id, job_id, pipeline_mode, total_duration_ms, total_warnings, total_errors, audit_data, user_summary)
         VALUES ($1, $2, 'ai_full', $3, $4, $5, $6, $7)"
    )
    .bind(drawing_id)
    .bind(job.job_id)
    .bind(gen_elapsed as i64)
    .bind(audit.warnings.len() as i32)
    .bind(audit.errors.len() as i32)
    .bind(&audit_json)
    .bind(&user_summary)
    .execute(&ctx.db)
    .await {
        tracing::warn!(%session_id, error = %e, "Generation audit trail storage failed (non-fatal)");
    }

    tracing::info!(
        %session_id, %drawing_id,
        items = ai_kss.items.len(),
        total = format!("{:.2}", ai_kss.totals.grand_total),
        audit_ms = gen_elapsed,
        "AI KSS pipeline complete — stored in Postgres with audit"
    );

    Ok(())
}

// ── Redis helpers ───────────────────────────────────────────

async fn set_redis_field(
    conn: &mut redis::aio::MultiplexedConnection,
    session_id: &Uuid,
    field: &str,
    value: &str,
) -> Result<()> {
    let key = format!("kcc:ai:{}:{}", session_id, field);
    redis::cmd("SET").arg(&key).arg(value)
        .query_async::<()>(conn).await?;
    redis::cmd("EXPIRE").arg(&key).arg(SESSION_TTL_SECS)
        .query_async::<()>(conn).await?;
    Ok(())
}

fn sek_group_order(sek_group: &str) -> i32 {
    match sek_group {
        "СЕК01" => 1, "СЕК02" => 2, "СЕК03" => 3, "СЕК04" => 4,
        "СЕК05" => 5, "СЕК06" => 6, "СЕК07" => 7, "СЕК08" => 8,
        "СЕК09" => 9, "СЕК10" => 10, "СЕК11" => 11, "СЕК12" => 12,
        "СЕК13" => 13, "СЕК14" => 14, "СЕК15" => 15, "СЕК16" => 16,
        "СЕК17" => 17, "СЕК20" => 18, "СЕК22" => 19, "СЕК23" => 20,
        "СЕК34" => 21, "СЕК18" => 22, "СЕК26" => 23,
        _ => 99,
    }
}

// repair_truncated_json moved to kcc_core::ai::repair_truncated_json

/// Loads quantity norms for the relevant SEK groups (built-in + this user's
/// personal norms) and formats them as a compact block suitable for injection
/// into the Perplexity research prompt. Keeps at most ~40 rows to stay under
/// the 16k max_tokens ceiling while still giving the AI strong anchors.
async fn load_quantity_norms_block(
    db: &sqlx::PgPool,
    user_id: Uuid,
    sek_groups: &[&str],
) -> String {
    if sek_groups.is_empty() {
        return "(no quantity norms configured for this drawing type)".to_string();
    }
    // Build `OR` of `sek_code LIKE 'СЕК05%'` clauses, 1 bind per group.
    let mut sql = String::from(
        "SELECT sek_code, description_bg, work_unit,
                labor_qualified_h::float8, labor_helper_h::float8, labor_trade,
                materials, source
         FROM quantity_norms
         WHERE (user_id IS NULL OR user_id = $1) AND (",
    );
    for i in 0..sek_groups.len() {
        if i > 0 { sql.push_str(" OR "); }
        sql.push_str(&format!("sek_code LIKE ${} || '%'", i + 2));
    }
    sql.push_str(") ORDER BY sek_code, user_id NULLS LAST LIMIT 40");

    let mut q = sqlx::query(&sql).bind(user_id);
    for g in sek_groups {
        q = q.bind(*g);
    }

    let rows = match q.fetch_all(db).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to load quantity norms — prompt will run without anchors");
            return "(quantity norms unavailable)".to_string();
        }
    };

    if rows.is_empty() {
        return "(no norms seeded for this drawing's SEK groups)".to_string();
    }

    use sqlx::Row;
    let mut lines: Vec<String> = Vec::with_capacity(rows.len());
    for r in rows {
        let code: String = r.try_get("sek_code").unwrap_or_default();
        let desc: String = r.try_get("description_bg").unwrap_or_default();
        let unit: String = r.try_get("work_unit").unwrap_or_default();
        let lq: f64 = r.try_get("labor_qualified_h").unwrap_or(0.0);
        let lh: f64 = r.try_get("labor_helper_h").unwrap_or(0.0);
        let trade: Option<String> = r.try_get("labor_trade").ok();
        let mats_json: serde_json::Value = r.try_get("materials").unwrap_or(serde_json::Value::Null);
        let source: String = r.try_get("source").unwrap_or_default();

        let mats_str = format_materials_inline(&mats_json);
        let trade_str = trade.as_deref().unwrap_or("—");
        lines.push(format!(
            "- {code} | {desc} | per {unit} → труд: {q:.2}h кв. + {h:.2}h пом. ({trade_str}); материали: {mats_str} [src: {source}]",
            q = lq, h = lh,
        ));
    }
    lines.join("\n")
}

/// Compacts the materials JSON array [{name, qty, unit}, …] into one inline
/// string like "цимент 8кг, пясък 0.02м3". Returns "—" if the shape is empty
/// or unexpected.
fn format_materials_inline(v: &serde_json::Value) -> String {
    let arr = match v.as_array() {
        Some(a) if !a.is_empty() => a,
        _ => return "—".to_string(),
    };
    arr.iter()
        .take(5)
        .filter_map(|m| {
            let name = m.get("name")?.as_str()?;
            let qty = m.get("qty").and_then(|x| x.as_f64()).unwrap_or(0.0);
            let unit = m.get("unit").and_then(|x| x.as_str()).unwrap_or("");
            Some(format!("{name} {qty}{unit}"))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Canonical KSS work categories per drawing type. The RAG search uses
/// these as queries against the user's price corpus — one query per
/// category per structure. Mirrors the Perplexity research-phase categories
/// so RAG and AI cover the same surface.
fn rag_categories_for(drawing_type: kcc_core::drawing_type::DrawingType) -> Vec<&'static str> {
    use kcc_core::drawing_type::DrawingType;
    match drawing_type {
        DrawingType::Timber => vec![
            "KVH конструкционна дървесина",
            "BSH ламелирана дървесина",
            "OSB плочи 18 mm",
            "OSB плочи 10 mm",
            "Шперплат мебелен 18 mm",
            "Минерална каменна вата",
            "Топлоизолация неопор EPS",
            "Ламарина за фасада и покрив",
            "PVC 5-камерна дограма",
            "Вътрешни врати",
            "Лепило за фасадни панели",
            "Летвена скара",
            "Дъски 25 mm обшивка",
            "Паропропусклива фолио",
            "Конструктивни планки крепежни елементи",
            "Стълба и второ ниво",
            "Транспорт",
        ],
        DrawingType::Steel => vec![
            "Стоманени профили IPE HEB UPN",
            "Заваръчни работи",
            "Болтови съединения",
            "Антикорозионна защита",
            "Кофраж за стоманобетонни елементи",
            "Бетон C25/30",
            "Армировка стомана B500B",
            "Стоманени плочи ламарина",
            "Монтаж на стоманена конструкция кран",
            "Транспорт стоманена конструкция",
        ],
        DrawingType::Architectural | DrawingType::Mechanical | DrawingType::Unknown => vec![
            "Тухлена зидария",
            "Газобетонна зидария",
            "Вътрешна мазилка",
            "Латексово боядисване",
            "Подови настилки ламинат",
            "Подови настилки плочки",
            "Хидроизолация",
            "Топлоизолация EPS",
            "PVC дограма",
            "ВиК тоалетна",
            "ВиК мивка",
            "Транспорт",
        ],
    }
}

/// Lower-cased, stripped key for de-duping items between RAG and AI.
fn canonical_desc_key(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// RAG retrieval per structure. For each canonical category, search the
/// user's price corpus and emit a KssLineItem from the top match (when
/// similarity ≥ 0.30). Quantity is taken from the corpus row as a starting
/// point — the user reviews and adjusts in the KSS UI before signing.
async fn generate_rag_items_for_structure(
    db: &sqlx::PgPool,
    user_id: Uuid,
    drawing_type: kcc_core::drawing_type::DrawingType,
) -> anyhow::Result<Vec<kcc_core::kss::types::KssLineItem>> {
    use kcc_core::price_corpus::search::{search_corpus, SearchOptions};

    let opts = SearchOptions {
        min_similarity: 0.30,
        top_k: 1,
    };

    let mut items: Vec<kcc_core::kss::types::KssLineItem> = Vec::new();
    let mut seen_corpus_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    let mut item_no = 0usize;
    for (cat_idx, query) in rag_categories_for(drawing_type).iter().enumerate() {
        let matches = search_corpus(db, user_id, query, opts).await?;
        let Some(top) = matches.into_iter().next() else {
            continue;
        };
        // De-dup: a single corpus row could match two categories (e.g.
        // "OSB 18" and "OSB"). Skip if we've already used it.
        if !seen_corpus_ids.insert(top.id) {
            continue;
        }
        let qty = 0.0; // forces user to fill in quantity per the new drawing
        let total = top.total_unit_price_lv * qty;
        let sek_code = top
            .sek_code
            .clone()
            .unwrap_or_else(|| sek_code_for_query(query).to_string());
        item_no += 1;
        items.push(kcc_core::kss::types::KssLineItem {
            item_no,
            sek_code,
            description: top.description.clone(),
            unit: top.unit.clone(),
            quantity: qty,
            material_price: top.material_price_lv,
            labor_price: top.labor_price_lv,
            mechanization_price: 0.0,
            overhead_price: 0.0,
            total_price: total,
            confidence: top.similarity,
            reasoning: format!(
                "Matched corpus row (similarity {:.2}) — quantity needs to be set for this drawing.",
                top.similarity
            ),
            provenance: "rag".to_string(),
            source_layer: None,
            extraction_method: Some("user_corpus".to_string()),
            geometry_confidence: top.similarity,
            needs_review: true,
            ..Default::default()
        });
        let _ = cat_idx;
    }
    Ok(items)
}

/// Best-effort mapping from a category query to a SEK group code, used
/// when the corpus row carries no explicit sek_code.
fn sek_code_for_query(q: &str) -> &'static str {
    let lc = q.to_lowercase();
    if lc.contains("kvh") || lc.contains("bsh") || lc.contains("дървесина") {
        "СЕК05.001"
    } else if lc.contains("osb") || lc.contains("шперплат") || lc.contains("обшивка") {
        "СЕК06.001"
    } else if lc.contains("вата") || lc.contains("eps") || lc.contains("неопор") || lc.contains("топлоизолация") {
        "СЕК16.001"
    } else if lc.contains("ламарина") || lc.contains("фасада") {
        "СЕК07.001"
    } else if lc.contains("дограма") {
        "СЕК17.001"
    } else if lc.contains("врат") {
        "СЕК17.020"
    } else if lc.contains("транспорт") {
        "СЕК24.001"
    } else if lc.contains("зидария") {
        "СЕК05.002"
    } else if lc.contains("мазилка") {
        "СЕК10.011"
    } else if lc.contains("боядисване") {
        "СЕК13.030"
    } else if lc.contains("стомана") || lc.contains("ipe") || lc.contains("heb") {
        "СЕК14.001"
    } else {
        "СЕК99.999"
    }
}
