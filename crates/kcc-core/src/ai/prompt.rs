//! Builds structured prompts for the AI agent from pipeline data.
//!
//! The prompt contains ONLY structured data extracted from the DXF parser —
//! layers, dimensions, annotations, blocks, features, DRM context, and prices.
//! The AI never sees raw geometry or DWG/DXF bytes.

use std::collections::HashMap;

use crate::drm::ContextBundle;
use crate::kss::types::{KssReport, PriceList};
use crate::AnalysisResult;

/// System prompt — defines the AI's role and output format.
pub const SYSTEM_PROMPT: &str = r#"You are a Bulgarian construction quantity surveyor (КСС specialist).
You receive structured data extracted from a DWG/DXF drawing and user-reviewed price data.
Produce a complete Количествено-Стойностна Сметка following real Bulgarian construction practice.

FORMAT RULES:
1. Each item must be "Доставка и монтаж [description]" format with SEPARATE material and labor prices
2. material_price_eur = material/delivery cost per unit in €
3. labor_price_eur = labor/installation cost per unit in €
4. price_eur = material_price_eur + labor_price_eur (unit total)
5. Group into standard KSS sections I–XXIII
6. Include TRANSPORT as an explicit line item
7. Include firm OVERHEAD section (admin, contingency, delivery/storage, profit, ДДС)
8. Output ONLY valid JSON — no markdown fences

QUANTITY ESTIMATION (CRITICAL):
1. WALL AREA: Line length on wall/vista layers × ceiling height.
   If drawing has multiple views (plan + sections), DIVIDE total by estimated view count (2-3).
   Look for "PLANTA BAIXA E CORTES", "план и разрези", or multiple Vista layers.
2. BLOCK COUNTS: Total INSERT references per block type.
   If same block appears in plan + section views, DIVIDE by view count.
   A typical bungalow: 1-2 toilets, 1-2 sinks, 1 shower — not 13 toilets.
3. ROOM AREAS: From "Xm2" annotations directly. If none, estimate from dimensions.
4. CEILING HEIGHT: Look for repeated dimension ~2.50-3.00m. Default 2.80m.
5. WALL THICKNESS: Look for dimension 0.15-0.25m. Default 0.20m.
6. TRANSPORT: Small (<100м2): 800 €. Medium (100-200м2): 1500 €. Large (>200м2): 2000 €.

CONFIDENCE SCORING (CRITICAL — be honest, users WILL verify against the drawing):
- 0.9: Quantity derived DIRECTLY from drawing geometry, dimensions, or block INSERT counts
- 0.7-0.8: Estimated from related drawing data with reasonable inference (wall area from layer lengths × height)
- 0.5: Assumed from building type — drawing has SOME indirect evidence (e.g., roof from floor area)
- 0.3: Assumed from building type — NO evidence in drawing data at all (e.g., electrical when no electrical layers exist)
Items with confidence < 0.7 will be shown as "AI Suggestions" requiring user approval — NOT in the main report.
Do NOT inflate confidence. If you have no drawing evidence, use 0.3.

SECTION ORDER:
I.ЗЕМНИ РАБОТИ, II.КОФРАЖНИ РАБОТИ, III.АРМИРОВЪЧНИ РАБОТИ,
IV.БЕТОНОВИ РАБОТИ, V.ЗИДАРСКИ РАБОТИ, VI.ПОКРИВНИ РАБОТИ,
VII.ТЕНЕКЕДЖИЙСКИ РАБОТИ, VIII.ДЪРВОДЕЛСКИ РАБОТИ,
IX.ОБЛИЦОВЪЧНИ РАБОТИ, X.МАЗАЧЕСКИ РАБОТИ,
XI.НАСТИЛКИ И ЗАМАЗКИ, XII.СТЪКЛАРСКИ РАБОТИ,
XIII.БОЯДЖИЙСКИ РАБОТИ, XIV.МЕТАЛНИ КОНСТРУКЦИИ,
XV.ХИДРОИЗОЛАЦИИ, XVI.ТОПЛОИЗОЛАЦИИ,
XVII.СТОЛАРСКИ РАБОТИ, XVIII.СУХО СТРОИТЕЛСТВО,
XIX.СГРАДНИ ВИК, XX.ВЪНШНИ ВИК,
XXI.ЕЛЕКТРИЧЕСКА, XXII.ОТОПЛИТЕЛНА, XXIII.ПЪТИЩА, XXIV.ТРАНСПОРТ

OUTPUT JSON SCHEMA:
{
  "kss_sections": [{
    "number": "V",
    "title": "ЗИДАРСКИ РАБОТИ",
    "items": [{
      "sek_code": "СЕК05.002",
      "description": "Доставка и монтаж на тухлена зидария 29см",
      "unit": "М2",
      "quantity": 68.3,
      "material_price_eur": 55.0,
      "labor_price_eur": 70.0,
      "price_eur": 125.0,
      "confidence": 0.85,
      "reasoning": "Wall area from Vista layers ÷ 2 views = 68.3 М2"
    }]
  }],
  "overhead": {
    "admin_rate_pct": 10.0,
    "contingency_rate_pct": 10.0,
    "delivery_storage_rate_pct": 8.0,
    "profit_rate_pct": 30.0,
    "vat_rate_pct": 20.0
  },
  "total_items": 15,
  "construction_subtotal_eur": 85000.0,
  "total_eur": 165000.0,
  "drawing_type": "architectural_floor_plan",
  "language_detected": "bulgarian",
  "warnings": []
}"#;

/// Build the user prompt from pipeline data.
pub fn build_user_prompt(
    analysis: &AnalysisResult,
    drm_context: &ContextBundle,
    price_list: &PriceList,
    rule_kss: &KssReport,
) -> String {
    let drawing = &analysis.drawing;

    // Layer summary: name + entity count
    let mut layer_counts: HashMap<String, usize> = HashMap::new();
    for entity in &drawing.entities {
        *layer_counts.entry(entity.layer.clone()).or_insert(0) += 1;
    }
    let mut layers: Vec<serde_json::Value> = layer_counts
        .iter()
        .map(|(name, count)| serde_json::json!({ "name": name, "entity_count": count }))
        .collect();
    layers.sort_by(|a, b| {
        b["entity_count"].as_u64().cmp(&a["entity_count"].as_u64())
    });

    // Dimension values
    let dimensions: Vec<serde_json::Value> = drawing
        .dimensions
        .iter()
        .map(|d| serde_json::json!({
            "type": format!("{:?}", d.dim_type),
            "value": d.nominal_value,
        }))
        .collect();

    // Annotations
    let annotations: Vec<serde_json::Value> = drawing
        .annotations
        .iter()
        .map(|a| serde_json::json!({
            "text": a.text,
            "layer": a.layer,
        }))
        .collect();

    // Block INSERT counts
    let mut block_counts: HashMap<String, usize> = HashMap::new();
    for entity in &drawing.entities {
        if let Some(ref block_ref) = entity.block_ref {
            *block_counts.entry(block_ref.clone()).or_insert(0) += 1;
        }
    }
    let blocks: Vec<serde_json::Value> = block_counts
        .iter()
        .map(|(name, count)| serde_json::json!({ "name": name, "insert_count": count }))
        .collect();

    // Feature type summary
    let mut feature_types: HashMap<String, usize> = HashMap::new();
    for feature in &analysis.features {
        let type_name = feature.feature_type.name();
        *feature_types.entry(type_name.to_string()).or_insert(0) += 1;
    }

    // DRM historical mappings (top matches)
    let drm_mappings: Vec<serde_json::Value> = drm_context
        .layer_mappings
        .iter()
        .chain(drm_context.block_mappings.iter())
        .take(20)
        .map(|m| serde_json::json!({
            "input": m.input_key,
            "sek_group": m.sek_group,
            "confidence": m.confidence,
            "times_confirmed": m.times_confirmed,
            "source": m.source,
        }))
        .collect();

    // Available prices (top 50)
    let prices: Vec<serde_json::Value> = price_list
        .items
        .iter()
        .take(50)
        .map(|p| serde_json::json!({
            "sek_code": p.sek_code,
            "description": p.description,
            "unit": p.unit,
            "price_avg_eur": p.total_unit_price(),
        }))
        .collect();

    // Rule engine result
    let rule_items: Vec<serde_json::Value> = rule_kss
        .items
        .iter()
        .map(|item| serde_json::json!({
            "sek_code": item.sek_code,
            "description": item.description,
            "unit": item.unit,
            "quantity": item.quantity,
            "total_price": item.total_price,
        }))
        .collect();

    let prompt = serde_json::json!({
        "task": "generate_kss",
        "drawing": {
            "filename": drawing.metadata.filename,
            "units": format!("{:?}", drawing.units),
            "total_entities": drawing.entities.len(),
            "total_layers": layer_counts.len(),
            "total_dimensions": drawing.dimensions.len(),
            "total_annotations": drawing.annotations.len(),
        },
        "layers": layers,
        "dimensions": dimensions,
        "annotations": annotations,
        "blocks_used": blocks,
        "features_summary": feature_types,
        "drm_context": {
            "historical_mappings": drm_mappings,
        },
        "available_prices": prices,
        "rule_engine_result": {
            "items": rule_items,
            "total": rule_kss.totals.grand_total,
        },
    });

    serde_json::to_string_pretty(&prompt).unwrap_or_default()
}
