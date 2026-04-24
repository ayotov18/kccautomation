//! Parser for ceresit.bg — Henkel product datasheets.
//!
//! Two dosage shapes:
//!   A) inline primer/paint: "Разход: X[-Y] кг/м2"
//!   B) tile-grout table: rows of "tile_size | joint_width | ~kg/m2"

use regex::Regex;
use std::sync::OnceLock;

use super::{NormCategoryUrl, NormParseResult, NormParser};
use crate::quantity_scraper::{NormMaterial, ScrapedNorm};

pub struct CeresitParser;

static PRODUCT_RE: OnceLock<Regex> = OnceLock::new();
static INLINE_RE: OnceLock<Regex> = OnceLock::new();
static GROUT_ROW_RE: OnceLock<Regex> = OnceLock::new();

fn product_re() -> &'static Regex {
    PRODUCT_RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*(?P<code>CT\s*\d{1,3}|CE\s*\d{1,3}|CM\s*\d{1,3}|CX\s*\d{1,3}|CN\s*\d{1,3})\b").unwrap()
    })
}
fn inline_re() -> &'static Regex {
    INLINE_RE.get_or_init(|| Regex::new(r"Разход[^:]*:\s*(?P<lo>\d+(?:[.,]\d+)?)(?:\s*[-–]\s*(?P<hi>\d+(?:[.,]\d+)?))?\s*кг/м2").unwrap())
}
fn grout_row_re() -> &'static Regex {
    GROUT_ROW_RE.get_or_init(|| {
        // "плочки    10,8 / 10,8        3            около 0,35"
        Regex::new(r"(?m)^\s*(?P<kind>[A-Za-zА-Яа-я]{3,30})\s+(?P<sz>\d+(?:[.,]\d+)?\s*/\s*\d+(?:[.,]\d+)?)\s+(?P<joint>\d+(?:[.,]\d+)?)\s+около\s+(?P<kg>\d+(?:[.,]\d+)?)").unwrap()
    })
}

impl NormParser for CeresitParser {
    fn site_name(&self) -> &str { "ceresit.bg" }
    fn template_key(&self) -> &str { "ceresit" }

    fn parse_page(&self, content: &str, url: &str) -> NormParseResult {
        let product_code = product_re()
            .captures(content)
            .and_then(|c| c.name("code").map(|m| m.as_str().trim().to_string()))
            .unwrap_or_else(|| "Ceresit".to_string());
        let product_label = format!("Ceresit {}", product_code.replace(char::is_whitespace, " "));

        let mut norms = Vec::new();
        let mut diagnostics = Vec::new();

        // Shape A — inline single-value consumption.
        if let Some(c) = inline_re().captures(content) {
            let lo: f64 = c["lo"].replace(',', ".").parse().unwrap_or(0.0);
            let hi: f64 = c.name("hi").and_then(|m| m.as_str().replace(',', ".").parse().ok()).unwrap_or(lo);
            let mid = (lo + hi) / 2.0;
            let sek = sek_for_code(&product_code);
            norms.push(ScrapedNorm {
                source_site: "ceresit.bg".into(),
                source_url: url.into(),
                description_bg: format!("{product_label} — грундиране / нанасяне"),
                work_unit: "m²".into(),
                labor_qualified_h: 0.08,
                labor_helper_h: 0.03,
                labor_trade: Some(trade_for_sek(sek).into()),
                materials: vec![
                    NormMaterial { name: product_label.clone(), qty: (mid * 100.0).round() / 100.0, unit: "кг".into() },
                ],
                machinery: Vec::new(),
                sek_group_hint: Some(sek.into()),
                raw_snippet: Some(c.get(0).unwrap().as_str().chars().take(180).collect()),
                extraction_confidence: 0.85,
            });
        }

        // Shape B — tile-grout table (rows per tile size + joint width).
        let mut grout_rows = 0usize;
        for c in grout_row_re().captures_iter(content) {
            grout_rows += 1;
            let kind = c["kind"].trim().to_string();
            let size = c["sz"].trim().to_string();
            let joint: f64 = c["joint"].replace(',', ".").parse().unwrap_or(0.0);
            let kg: f64 = c["kg"].replace(',', ".").parse().unwrap_or(0.0);
            if kg == 0.0 { continue; }
            norms.push(ScrapedNorm {
                source_site: "ceresit.bg".into(),
                source_url: url.into(),
                description_bg: format!("{product_label} — фугиране {kind} {size} см, фуга {joint:.0} мм"),
                work_unit: "m²".into(),
                labor_qualified_h: 0.12,
                labor_helper_h: 0.04,
                labor_trade: Some("плочкар".into()),
                materials: vec![
                    NormMaterial { name: product_label.clone(), qty: kg, unit: "кг".into() },
                ],
                machinery: Vec::new(),
                sek_group_hint: Some("СЕК09".into()),
                raw_snippet: Some(c.get(0).unwrap().as_str().chars().take(180).collect()),
                extraction_confidence: 0.9,
            });
        }

        diagnostics.push(("inline_matched", if inline_re().is_match(content) { 1 } else { 0 }));
        diagnostics.push(("grout_rows", grout_rows));
        diagnostics.push(("total", norms.len()));

        let count = norms.len();
        NormParseResult {
            norms,
            strategy_used: "ceresit_tds",
            candidates_before_filter: count,
            candidates_after_filter: count,
            diagnostics,
        }
    }

    fn category_urls(&self) -> Vec<NormCategoryUrl> {
        vec![
            NormCategoryUrl::pdf("https://www.ceresit.bg/content/dam/ceresit/bg/products/ct/ct19/TDS_CT19.pdf", "СЕК10", "Ceresit CT 19 — грунд"),
            NormCategoryUrl::pdf("https://www.ceresit.bg/content/dam/ceresit/bg/products/ce/ce33/TDS_CE33.pdf", "СЕК09", "Ceresit CE 33 — фугираща смес"),
            NormCategoryUrl::pdf("https://www.ceresit.bg/content/dam/ceresit/bg/products/cm/cm11/TDS_CM11.pdf", "СЕК09", "Ceresit CM 11 — лепило за плочки"),
        ]
    }
}

fn sek_for_code(code: &str) -> &'static str {
    let c = code.replace(' ', "").to_uppercase();
    if c.starts_with("CM") || c.starts_with("CE") { "СЕК09" }
    else if c.starts_with("CT19") || c.starts_with("CT17") { "СЕК10" }
    else if c.starts_with("CT") { "СЕК10" }
    else if c.starts_with("CX") || c.starts_with("CN") { "СЕК15" }
    else { "СЕК10" }
}

fn trade_for_sek(sek: &str) -> &'static str {
    match sek {
        "СЕК09" => "плочкар",
        "СЕК15" => "изолатор",
        _ => "мазач",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ct19_inline() {
        let text = "CT 19\nГрунд Бетонконтакт\nРазход: 0,1-0,3 кг/м2 в зависимост от основата";
        let r = CeresitParser.parse_page(text, "u");
        assert!(!r.norms.is_empty());
        assert!(r.norms[0].description_bg.contains("CT 19"));
        assert_eq!(r.norms[0].materials[0].unit, "кг");
    }

    #[test]
    fn parses_ce33_grout_table() {
        let text = "\
CE 33
Ориентировъчен разход
Вид на               Размер       Ширина на         Количество
облицовката          (в cm)        фугите              СE 33
                                     (в mm)            (в kg/m2)
мозайка                5/5           1,5-2          около 0,50
плочки                 5/5             2            около 0,67
плочки              10,8 / 10,8        3            около 0,35
плочки                15 / 15          3            около 0,38
";
        let r = CeresitParser.parse_page(text, "u");
        assert!(r.norms.len() >= 3);
        assert!(r.norms.iter().any(|n| n.description_bg.contains("фугиране")));
    }
}
