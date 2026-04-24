use crate::kss::types::{KssLineItem, KssReport, PriceList};
use super::bill_graph::FabricationBillGraph;

/// Transform a FabricationBillGraph into a KSS report (Образец 9.1).
pub fn bill_graph_to_kss(
    bill: &FabricationBillGraph,
    drawing_name: &str,
    generated_at: &str,
    price_list: &PriceList,
) -> KssReport {
    let mut items = Vec::new();

    for (idx, fab_item) in bill.items.iter().enumerate() {
        let price = price_list.find_by_code(&fab_item.sek_code);

        let (labor, material, mechanization, overhead) = if let Some(p) = price {
            (p.labor_price, p.material_price, p.mechanization_price, p.overhead_price)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        let unit_total = labor + material + mechanization + overhead;
        let total = unit_total * fab_item.quantity;

        items.push(KssLineItem {
            item_no: idx + 1,
            sek_code: fab_item.sek_code.clone(),
            description: fab_item.description.clone(),
            unit: fab_item.unit.clone(),
            quantity: fab_item.quantity,
            labor_price: labor,
            material_price: material,
            mechanization_price: mechanization,
            overhead_price: overhead,
            total_price: total,
            confidence: 0.9,
            reasoning: format!("Fabrication bill: {}", fab_item.description),
            provenance: "rule_based".to_string(),
            ..Default::default()
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
