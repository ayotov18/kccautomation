use super::types::{KssLineItem, KssReport, PriceList, QuantityItem};

/// Map extracted quantities to KSS line items using a price list.
pub fn generate_kss_report(
    drawing_name: &str,
    generated_at: &str,
    quantities: &[QuantityItem],
    price_list: &PriceList,
) -> KssReport {
    let mut items = Vec::new();

    for (idx, qty) in quantities.iter().enumerate() {
        // Look up price from user's price list
        let price_item = price_list.find_by_code(&qty.suggested_sek_code);

        let (labor, material, mechanization, overhead) = if let Some(p) = price_item {
            (p.labor_price, p.material_price, p.mechanization_price, p.overhead_price)
        } else {
            // No price found — zero prices (user can fill in later)
            (0.0, 0.0, 0.0, 0.0)
        };

        let unit_total = labor + material + mechanization + overhead;
        let total = unit_total * qty.quantity;

        items.push(KssLineItem {
            item_no: idx + 1,
            sek_code: qty.suggested_sek_code.clone(),
            description: qty.description.clone(),
            unit: qty.unit.clone(),
            quantity: qty.quantity,
            labor_price: labor,
            material_price: material,
            mechanization_price: mechanization,
            overhead_price: overhead,
            total_price: total,
            // Row-level confidence is the minimum of price certainty (always 0.9
            // when we find a price-list match) and the geometric confidence.
            confidence: 0.9_f64.min(qty.geometry_confidence),
            reasoning: format!("Layer geometry: {} ({})", qty.category, qty.extraction_method.as_str()),
            provenance: "rule_based".to_string(),
            source_entity_id: qty.source_entity_id.clone(),
            source_layer: qty.source_layer.clone(),
            centroid_x: qty.centroid.map(|(x, _)| x),
            centroid_y: qty.centroid.map(|(_, y)| y),
            extraction_method: Some(qty.extraction_method.as_str().to_string()),
            geometry_confidence: qty.geometry_confidence,
            needs_review: qty.needs_review,
        });
    }

    let totals = KssReport::compute_totals(&items);

    KssReport {
        drawing_name: drawing_name.to_string(),
        generated_at: generated_at.to_string(),
        items,
        totals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kss::types::PriceListItem;

    #[test]
    fn test_generate_kss_report() {
        let quantities = vec![
            QuantityItem { category: "Steel members".into(), description: "IPE 200 beams".into(), unit: "m".into(), quantity: 12.5, suggested_sek_code: "14.001".into(), ..Default::default() },
            QuantityItem { category: "Fasteners".into(), description: "M16 bolts".into(), unit: "pcs".into(), quantity: 24.0, suggested_sek_code: "14.015".into(), ..Default::default() },
        ];

        let price_list = PriceList {
            items: vec![
                PriceListItem {
                    sek_code: "14.001".into(),
                    description: "Steel beams".into(),
                    unit: "m".into(),
                    labor_price: 5.0,
                    material_price: 25.0,
                    mechanization_price: 3.0,
                    overhead_price: 1.0,
                },
                PriceListItem {
                    sek_code: "14.015".into(),
                    description: "Bolt assemblies".into(),
                    unit: "pcs".into(),
                    labor_price: 1.5,
                    material_price: 4.0,
                    mechanization_price: 0.0,
                    overhead_price: 0.5,
                },
            ],
        };

        let report = generate_kss_report("test.dwg", "2026-01-01", &quantities, &price_list);
        assert_eq!(report.items.len(), 2);
        // 12.5m * (5+25+3+1) = 12.5 * 34 = 425
        assert!((report.items[0].total_price - 425.0).abs() < 0.01);
        // 24 * (1.5+4+0+0.5) = 24 * 6 = 144
        assert!((report.items[1].total_price - 144.0).abs() < 0.01);
        assert!((report.totals.grand_total - 569.0).abs() < 0.01);
    }
}
