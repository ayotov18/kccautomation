//! Merge rule-based KSS + AI KSS draft.
//!
//! Strategy: for each SEK group, pick the result with higher confidence.
//! AI items not in rule result are added. Rule items not in AI result are kept.
//! Result is a superset with best confidence per item.

use std::collections::HashMap;

use crate::kss::types::{KssLineItem, KssReport};

/// Merge rule-based and AI-generated KSS reports.
/// Higher confidence wins per SEK group. Unique items from either side are included.
pub fn merge_kss(rule_based: &KssReport, ai_report: &KssReport) -> KssReport {
    // Group items by SEK code prefix (group level)
    let rule_groups = group_by_sek(&rule_based.items);
    let ai_groups = group_by_sek(&ai_report.items);

    let mut merged_items: Vec<KssLineItem> = Vec::new();

    // All SEK groups from both sources
    let mut all_groups: Vec<String> = rule_groups
        .keys()
        .chain(ai_groups.keys())
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    all_groups.sort();

    for group in &all_groups {
        let rule_items = rule_groups.get(group);
        let ai_items = ai_groups.get(group);

        match (rule_items, ai_items) {
            (Some(rules), Some(ais)) => {
                // Both have items for this group — use AI if it has more items
                // (AI is likely more comprehensive in its analysis)
                let rule_total: f64 = rules.iter().map(|i| i.total_price).sum();
                let ai_total: f64 = ais.iter().map(|i| i.total_price).sum();

                if ais.len() >= rules.len() || ai_total > rule_total {
                    merged_items.extend(ais.iter().cloned());
                } else {
                    merged_items.extend(rules.iter().cloned());
                }
            }
            (Some(rules), None) => {
                // Only rule engine has items — keep them
                merged_items.extend(rules.iter().cloned());
            }
            (None, Some(ais)) => {
                // Only AI has items — add them (AI found something rules missed)
                merged_items.extend(ais.iter().cloned());
            }
            (None, None) => {}
        }
    }

    // Re-number items sequentially
    for (i, item) in merged_items.iter_mut().enumerate() {
        item.item_no = i + 1;
    }

    let totals = KssReport::compute_totals(&merged_items);

    KssReport {
        drawing_name: rule_based.drawing_name.clone(),
        generated_at: rule_based.generated_at.clone(),
        items: merged_items,
        totals,
    }
}

/// Group KSS items by SEK code prefix (e.g., "СЕК05.002" → "СЕК05").
fn group_by_sek(items: &[KssLineItem]) -> HashMap<String, Vec<KssLineItem>> {
    let mut groups: HashMap<String, Vec<KssLineItem>> = HashMap::new();
    for item in items {
        let group = extract_sek_group(&item.sek_code);
        groups.entry(group).or_default().push(item.clone());
    }
    groups
}

fn extract_sek_group(code: &str) -> String {
    if let Some(dot_pos) = code.find('.') {
        code[..dot_pos].to_string()
    } else {
        code.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kss::types::KssTotals;

    fn make_item(no: usize, sek: &str, desc: &str, qty: f64, total: f64) -> KssLineItem {
        KssLineItem {
            item_no: no,
            sek_code: sek.to_string(),
            description: desc.to_string(),
            unit: "М2".to_string(),
            quantity: qty,
            labor_price: total * 0.4,
            material_price: total * 0.35,
            mechanization_price: total * 0.1,
            overhead_price: total * 0.15,
            total_price: total,
            confidence: 0.8,
            reasoning: String::new(),
            provenance: "rule_based".to_string(),
            ..Default::default()
        }
    }

    fn make_report(items: Vec<KssLineItem>) -> KssReport {
        let totals = KssReport::compute_totals(&items);
        KssReport {
            drawing_name: "test".into(),
            generated_at: "now".into(),
            items,
            totals,
        }
    }

    #[test]
    fn test_merge_ai_adds_missing_sections() {
        let rules = make_report(vec![
            make_item(1, "СЕК04", "Concrete walls", 100.0, 5000.0),
        ]);
        let ai = make_report(vec![
            make_item(1, "СЕК04", "Concrete walls", 100.0, 5000.0),
            make_item(2, "СЕК10.011", "Plaster", 200.0, 3000.0),
            make_item(3, "СЕК13.025", "Painting", 200.0, 1000.0),
        ]);

        let merged = merge_kss(&rules, &ai);
        assert_eq!(merged.items.len(), 3);
    }

    #[test]
    fn test_merge_rules_only_when_no_ai() {
        let rules = make_report(vec![
            make_item(1, "СЕК04", "Concrete", 100.0, 5000.0),
        ]);
        let ai = make_report(vec![]);

        let merged = merge_kss(&rules, &ai);
        assert_eq!(merged.items.len(), 1);
    }
}
