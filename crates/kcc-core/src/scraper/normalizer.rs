//! Normalizes scraped prices into PriceListItem CSV format.
//!
//! Canonical currency: лв (BGN). EUR is derived.

use crate::kss::types::PriceListItem;
use crate::scraper::sek_mapper::MappedPrice;
use crate::scraper::price_utils;

/// Typical cost component ratios for Bulgarian construction work.
const LABOR_RATIO: f64 = 0.40;
const MATERIAL_RATIO: f64 = 0.35;
const MECHANIZATION_RATIO: f64 = 0.10;
const OVERHEAD_RATIO: f64 = 0.15;

/// Convert mapped scraped prices into PriceListItems (prices in лв).
pub fn normalize_to_price_list(mapped: &[MappedPrice]) -> Vec<PriceListItem> {
    mapped
        .iter()
        .filter(|m| m.sek_code.is_some())
        .filter(|m| m.scraped.price_avg_lv() > 0.0)
        .map(|m| {
            let price_lv = m.scraped.price_avg_lv();

            PriceListItem {
                sek_code: m.sek_code.clone().unwrap_or_default(),
                description: m.scraped.description_bg.clone(),
                unit: price_utils::normalize_unit(&m.scraped.unit),
                labor_price: price_lv * LABOR_RATIO,
                material_price: price_lv * MATERIAL_RATIO,
                mechanization_price: price_lv * MECHANIZATION_RATIO,
                overhead_price: price_lv * OVERHEAD_RATIO,
            }
        })
        .collect()
}

/// Generate CSV content from PriceListItems.
pub fn items_to_csv(items: &[PriceListItem]) -> String {
    let mut csv = String::from("sek_code,description,unit,labor,material,mechanization,overhead\n");
    for item in items {
        csv.push_str(&format!(
            "{},{},{},{:.2},{:.2},{:.2},{:.2}\n",
            escape_csv(&item.sek_code),
            escape_csv(&item.description),
            escape_csv(&item.unit),
            item.labor_price,
            item.material_price,
            item.mechanization_price,
            item.overhead_price,
        ));
    }
    csv
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraper::ScrapedPrice;
    use crate::scraper::sek_mapper::MappedPrice;

    #[test]
    fn test_lv_prices_used_directly() {
        let mapped = vec![MappedPrice {
            scraped: ScrapedPrice::from_lv(
                "test", "https://test.com", "Тухлена зидария", "М2",
                Some(80.0), Some(120.0), None, None, 0.9,
            ),
            sek_code: Some("СЕК05.002".into()),
            sek_group: "СЕК05".into(),
            confidence: 0.8,
        }];

        let items = normalize_to_price_list(&mapped);
        assert_eq!(items.len(), 1);
        let total = items[0].total_unit_price();
        assert!((total - 100.0).abs() < 0.01); // avg of 80-120 лв
    }

    #[test]
    fn test_eur_converted_to_lv() {
        let mapped = vec![MappedPrice {
            scraped: ScrapedPrice::from_eur(
                "test", "https://test.com", "Зидария", "М2",
                Some(50.0), Some(50.0), None, None, 0.9,
            ),
            sek_code: Some("СЕК05.002".into()),
            sek_group: "СЕК05".into(),
            confidence: 0.8,
        }];

        let items = normalize_to_price_list(&mapped);
        assert_eq!(items.len(), 1);
        let total = items[0].total_unit_price();
        let expected = 50.0 * crate::scraper::EUR_TO_BGN;
        assert!((total - expected).abs() < 0.01);
    }
}
