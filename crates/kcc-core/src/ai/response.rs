//! Parse and validate AI response into KSS report structures.

use crate::ai::{AiKssItem, AiKssResponse};
use crate::kss::types::{KssLineItem, KssReport};

/// Validate and convert AI response into a KssReport for merging.
pub fn ai_response_to_kss_report(
    response: &AiKssResponse,
    drawing_name: &str,
    generated_at: &str,
) -> KssReport {
    let mut items = Vec::new();
    let mut item_no = 0;

    for section in &response.kss_sections {
        for ai_item in &section.items {
            if !is_valid_item(ai_item) {
                tracing::warn!(
                    sek = %ai_item.sek_code,
                    desc = %ai_item.description,
                    "AI item rejected: invalid"
                );
                continue;
            }

            item_no += 1;

            // Use real material/labor split if provided, otherwise fall back to price_lv
            let (mat, lab) = if ai_item.material_price_lv > 0.0 || ai_item.labor_price_lv > 0.0 {
                (ai_item.material_price_lv, ai_item.labor_price_lv)
            } else {
                // Legacy fallback: split price_lv into material 60% / labor 40%
                (ai_item.price_lv * 0.60, ai_item.price_lv * 0.40)
            };

            let unit_total = mat + lab;
            let total = unit_total * ai_item.quantity;

            // Honour the AI's self-reported traceability. If source_layer is
            // missing OR the basis is "assumed_typical", cap the geometry
            // confidence so the row routes to the review widget. `needs_review`
            // reflects the lower of AI-confidence and geometry-confidence.
            let basis = ai_item.extraction_basis.as_deref().unwrap_or("assumed_typical");
            let src_layer = ai_item.source_layer.clone().filter(|s| !s.is_empty() && s != "none");
            let geom_conf = if src_layer.is_none() || basis == "assumed_typical" {
                ai_item.confidence.min(0.50)
            } else {
                ai_item.confidence
            };
            let needs_review = geom_conf < 0.70;
            items.push(KssLineItem {
                item_no,
                sek_code: ai_item.sek_code.clone(),
                description: ai_item.description.clone(),
                unit: ai_item.unit.clone(),
                quantity: ai_item.quantity,
                material_price: mat,
                labor_price: lab,
                mechanization_price: 0.0,
                overhead_price: 0.0,
                total_price: total,
                confidence: ai_item.confidence,
                reasoning: ai_item.reasoning.clone(),
                provenance: "ai_generated".to_string(),
                source_layer: src_layer,
                extraction_method: Some("ai_inferred".to_string()),
                geometry_confidence: geom_conf,
                needs_review,
                ..Default::default()
            });
        }
    }

    let totals = KssReport::compute_totals(&items);

    KssReport {
        drawing_name: drawing_name.to_string(),
        generated_at: generated_at.to_string(),
        items,
        totals,
    }
}

/// Validate an AI-generated item.
fn is_valid_item(item: &AiKssItem) -> bool {
    // SEK code must be non-empty
    if item.sek_code.is_empty() {
        return false;
    }

    // Description must be non-empty and reasonable length
    if item.description.is_empty() || item.description.len() > 500 {
        return false;
    }

    // Unit must be from allowed set (case-insensitive, with common variants)
    let unit_lower = item.unit.to_lowercase().replace('.', "").replace(' ', "");
    let valid_units = [
        "м2", "м²", "m2", "кв.м", "квм",
        "м3", "м³", "m3", "куб.м", "кубм",
        "м", "m", "мл", "м.л", "лм", "л.м",
        "бр", "br", "pcs",
        "кг", "kg",
        "тон", "т",
        "компл", "комплект",
        "час", "ден",
    ];
    let unit_normalized = unit_lower.as_str();
    if !valid_units.iter().any(|u| unit_normalized == *u || unit_normalized.contains(u)) {
        tracing::debug!(unit = %item.unit, normalized = %unit_normalized, "Unit validation failed");
        return false;
    }

    // Quantity must be positive and reasonable
    if item.quantity <= 0.0 || item.quantity > 1_000_000.0 {
        return false;
    }

    // Price must be non-negative
    if item.price_lv < 0.0 {
        return false;
    }

    true
}

/// Extract AI warnings for frontend display.
pub fn extract_warnings(response: &AiKssResponse) -> Vec<String> {
    response.warnings.clone()
}

/// Extract per-item provenance info.
pub fn extract_provenance(response: &AiKssResponse) -> Vec<(String, f64, String)> {
    // (sek_code, confidence, reasoning)
    response
        .kss_sections
        .iter()
        .flat_map(|s| s.items.iter())
        .map(|item| {
            (
                item.sek_code.clone(),
                item.confidence,
                item.reasoning.clone(),
            )
        })
        .collect()
}
