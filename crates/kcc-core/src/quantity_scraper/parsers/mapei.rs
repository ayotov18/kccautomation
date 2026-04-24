//! Parser for mapei.bg — Mapei Bulgaria TDS PDFs.

use regex::Regex;
use std::sync::OnceLock;

use super::generic_tds;
use super::{NormCategoryUrl, NormParseResult, NormParser};
use crate::quantity_scraper::ScrapedNorm;

pub struct MapeiParser;

static PRODUCT_RE: OnceLock<Regex> = OnceLock::new();
fn product_re() -> &'static Regex {
    PRODUCT_RE.get_or_init(|| Regex::new(r"(?i)(Mape[a-z0-9\- ]{1,40}|Ultra[A-Za-z0-9\- ]{1,30}|Keraflex|Kerabond|Planitop|Keracolor|Adesilex)").unwrap())
}

impl NormParser for MapeiParser {
    fn site_name(&self) -> &str { "mapei.bg" }
    fn template_key(&self) -> &str { "mapei" }

    fn parse_page(&self, content: &str, url: &str) -> NormParseResult {
        let product = product_re()
            .find(content)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| "Mapei".to_string());
        let sek = sek_for_product(&product);
        let trade = trade_for(sek);

        let mut norms: Vec<ScrapedNorm> = Vec::new();
        for hit in generic_tds::scan_dosage(content) {
            let (lq, lh) = if hit.per_mm { (0.05, 0.02) } else { (0.12, 0.05) };
            let description = if hit.per_mm {
                format!("{product} — нанасяне, на мм дебелина")
            } else {
                format!("{product} — нанасяне, 1 m²")
            };
            norms.push(generic_tds::make_norm(
                "mapei.bg", url, &product, &description, sek, trade, lq, lh, &hit, content,
            ));
        }

        let count = norms.len();
        NormParseResult {
            norms,
            strategy_used: "mapei_tds",
            candidates_before_filter: count,
            candidates_after_filter: count,
            diagnostics: vec![("dosage_hits", count)],
        }
    }

    fn category_urls(&self) -> Vec<NormCategoryUrl> {
        vec![
            NormCategoryUrl::pdf("https://cdnmedia.mapei.com/docs/librariesprovider65/products-documents/ultracolor-plus-bg.pdf",   "СЕК09", "Ultracolor Plus"),
            NormCategoryUrl::pdf("https://cdnmedia.mapei.com/docs/librariesprovider65/products-documents/keraflex-maxi-s1-bg.pdf", "СЕК09", "Keraflex Maxi S1"),
            NormCategoryUrl::pdf("https://cdnmedia.mapei.com/docs/librariesprovider65/products-documents/mapewrap-12-bg.pdf",       "СЕК14", "MapeWrap 12"),
            NormCategoryUrl::pdf("https://cdnmedia.mapei.com/docs/librariesprovider65/products-documents/planitop-400-bg.pdf",      "СЕК04", "Planitop 400"),
            NormCategoryUrl::pdf("https://cdnmedia.mapei.com/docs/librariesprovider65/products-documents/mapetherm-ar-1-bg.pdf",    "СЕК16", "Mapetherm AR1"),
        ]
    }
}

fn sek_for_product(p: &str) -> &'static str {
    let s = p.to_lowercase();
    if s.contains("keraflex") || s.contains("kerabond") || s.contains("keracolor") || s.contains("ultracolor") { "СЕК09" }
    else if s.contains("planitop") { "СЕК04" }
    else if s.contains("wrap") || s.contains("fiber") { "СЕК14" }
    else if s.contains("therm") { "СЕК16" }
    else if s.contains("floor") || s.contains("ultraplan") { "СЕК11" }
    else { "СЕК09" }
}
fn trade_for(sek: &str) -> &'static str {
    match sek {
        "СЕК04" => "бетонджия",
        "СЕК09" => "плочкар",
        "СЕК11" => "подов майстор",
        "СЕК14" => "стоманов монтажник",
        "СЕК16" => "изолатор",
        _ => "плочкар",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mapewrap_per_mm() {
        let text = "MapeWrap 12\nРазход: 1,55 kg/m²/mm";
        let r = MapeiParser.parse_page(text, "u");
        assert!(!r.norms.is_empty());
        assert!(r.norms[0].work_unit.contains("mm"));
        assert_eq!(r.norms[0].sek_group_hint.as_deref(), Some("СЕК14"));
    }
}
