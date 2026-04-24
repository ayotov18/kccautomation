//! Parser for globus-bg.com — "Globus G" mortar/grout series datasheets.
//!
//! Shape: "Приблизително N торби за 1 м3 тухлена зидария
//!        (… - около X л/м3 тухлена зидария)."

use regex::Regex;
use std::sync::OnceLock;

use super::{NormCategoryUrl, NormParseResult, NormParser};
use crate::quantity_scraper::{NormMaterial, ScrapedNorm};

pub struct GlobusParser;

static PRODUCT_RE: OnceLock<Regex> = OnceLock::new();
static DOSAGE_RE: OnceLock<Regex> = OnceLock::new();
static BAG_SIZE_RE: OnceLock<Regex> = OnceLock::new();

fn product_re() -> &'static Regex {
    PRODUCT_RE.get_or_init(|| Regex::new(r"Globus\s+G\d+\s*\d{0,3}").unwrap())
}
fn dosage_re() -> &'static Regex {
    DOSAGE_RE.get_or_init(|| Regex::new(r"около\s+(?P<lpm3>\d{2,3})\s*л\s*/?\s*м3\s+тухл").unwrap())
}
fn bag_size_re() -> &'static Regex {
    BAG_SIZE_RE.get_or_init(|| Regex::new(r"(?P<bag>\d{1,3})\s*кг\s*/\s*торба|торб[а-я]*\s+от\s+(?P<alt>\d{1,3})\s*кг").unwrap())
}

impl NormParser for GlobusParser {
    fn site_name(&self) -> &str { "globus-bg.com" }
    fn template_key(&self) -> &str { "globus" }

    fn parse_page(&self, content: &str, url: &str) -> NormParseResult {
        let product = product_re()
            .find(content)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| "Globus хоросан".to_string());

        let mut norms = Vec::new();
        let mut diagnostics = Vec::new();

        if let Some(c) = dosage_re().captures(content) {
            let lpm3: f64 = c["lpm3"].parse().unwrap_or(0.0);
            // Convert per-m³-of-brickwork to per-m²-of-25cm-wall (canonical).
            // With 25 cm thickness: 1 m² wall ~ 0.25 m³ brickwork.
            let lpm2 = (lpm3 * 0.25).round();

            let bag_kg: f64 = bag_size_re().captures(content).and_then(|c| {
                c.name("bag").and_then(|m| m.as_str().parse().ok())
                    .or_else(|| c.name("alt").and_then(|m| m.as_str().parse().ok()))
            }).unwrap_or(25.0);
            let density: f64 = 1.22; // kg/л, from datasheet
            let kg_per_m2 = (lpm2 * density).round();

            norms.push(ScrapedNorm {
                source_site: "globus-bg.com".into(),
                source_url: url.into(),
                description_bg: format!("{product} — зидарски разтвор, стена 25 см"),
                work_unit: "m²".into(),
                labor_qualified_h: 0.50,
                labor_helper_h: 0.20,
                labor_trade: Some("зидар".into()),
                materials: vec![
                    NormMaterial { name: product.clone(), qty: kg_per_m2, unit: "кг".into() },
                    NormMaterial { name: "Вода".into(),   qty: (kg_per_m2 * 0.25).round(), unit: "л".into() },
                ],
                machinery: Vec::new(),
                sek_group_hint: Some("СЕК05".into()),
                raw_snippet: Some(c.get(0).unwrap().as_str().chars().take(180).collect()),
                extraction_confidence: 0.8,
            });

            // Also emit the per-m³ figure as an auxiliary row for concrete/grout cases.
            norms.push(ScrapedNorm {
                source_site: "globus-bg.com".into(),
                source_url: url.into(),
                description_bg: format!("{product} — хоросан за тухлена зидария"),
                work_unit: "m³".into(),
                labor_qualified_h: 0.0,
                labor_helper_h: 0.0,
                labor_trade: None,
                materials: vec![
                    NormMaterial { name: product.clone(), qty: (lpm3 * density).round(), unit: "кг".into() },
                ],
                machinery: Vec::new(),
                sek_group_hint: Some("СЕК05".into()),
                raw_snippet: Some(c.get(0).unwrap().as_str().chars().take(180).collect()),
                extraction_confidence: 0.8,
            });

            diagnostics.push(("bag_size_kg", bag_kg as usize));
        }

        diagnostics.push(("rows", norms.len()));
        let count = norms.len();
        NormParseResult {
            norms,
            strategy_used: "globus_tds",
            candidates_before_filter: count,
            candidates_after_filter: count,
            diagnostics,
        }
    }

    fn category_urls(&self) -> Vec<NormCategoryUrl> {
        vec![
            NormCategoryUrl::pdf("https://www.globus-bg.com/downloads/tds/G1_23.pdf", "СЕК05", "Globus G1 23 — варо-циментов хоросан"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_globus_g1_23() {
        let text = "\
Globus G1 23 е варо-циментов хоросан.
Насипно тегло 1,22 кг/л
разходна норма
Приблизително 2 торби за 1 м3 тухлена зидария (при средна дебелина на стената 25 см и 10 мм
дебелина на слоя - около 55 л/м3 тухлена зидария).
";
        let r = GlobusParser.parse_page(text, "u");
        assert!(!r.norms.is_empty());
        assert!(r.norms[0].description_bg.contains("Globus G1"));
        assert!(r.norms.iter().any(|n| n.work_unit == "m²"));
        assert!(r.norms.iter().any(|n| n.work_unit == "m³"));
    }
}
