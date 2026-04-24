//! Parser for bg.weber — Weber Saint-Gobain Bulgaria.

use regex::Regex;
use std::sync::OnceLock;

use super::generic_tds;
use super::{NormCategoryUrl, NormParseResult, NormParser};
use crate::quantity_scraper::ScrapedNorm;

pub struct WeberParser;

static PRODUCT_RE: OnceLock<Regex> = OnceLock::new();
fn product_re() -> &'static Regex {
    PRODUCT_RE.get_or_init(|| Regex::new(r"(?i)(weber[a-z0-9\.\- ]{0,40})").unwrap())
}

/// Weber frequently publishes ranges like "18-20 kg/m² per 10 mm". We catch
/// both the kg and the per-N-mm thickness so the emitted row preserves it.
static PER_MM_RE: OnceLock<Regex> = OnceLock::new();
fn per_mm_re() -> &'static Regex {
    PER_MM_RE.get_or_init(|| Regex::new(r"(?i)(?P<lo>\d+(?:[.,]\d+)?)(?:\s*[-–]\s*(?P<hi>\d+(?:[.,]\d+)?))?\s*(?:kg|кг)\s*/\s*m(?:2|²)\s*(?:на|per)\s+(?P<mm>\d+(?:[.,]\d+)?)\s*mm").unwrap())
}

impl NormParser for WeberParser {
    fn site_name(&self) -> &str { "weber.bg" }
    fn template_key(&self) -> &str { "weber" }

    fn parse_page(&self, content: &str, url: &str) -> NormParseResult {
        let product = product_re()
            .find(content)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| "Weber".to_string());
        let sek = sek_for_product(&product);
        let trade = trade_for(sek);
        let mut norms: Vec<ScrapedNorm> = Vec::new();
        let mut diagnostics = Vec::new();

        // Shape A — explicit "X kg/m² per Y mm".
        let mut per_mm_hits = 0usize;
        for c in per_mm_re().captures_iter(content) {
            per_mm_hits += 1;
            let lo: f64 = c["lo"].replace(',', ".").parse().unwrap_or(0.0);
            let hi: f64 = c.name("hi").and_then(|m| m.as_str().replace(',', ".").parse().ok()).unwrap_or(lo);
            let mm: f64 = c["mm"].replace(',', ".").parse().unwrap_or(1.0);
            if mm <= 0.0 { continue; }
            let per_mm_kg = ((lo + hi) / 2.0) / mm;
            let hit = generic_tds::DosageHit {
                kg_per_m2: (per_mm_kg * 1000.0).round() / 1000.0,
                per_mm: true,
                span: (c.get(0).unwrap().start(), c.get(0).unwrap().end()),
            };
            norms.push(generic_tds::make_norm(
                "weber.bg", url, &product,
                &format!("{product} — нанасяне, на мм дебелина"),
                sek, trade, 0.04, 0.02, &hit, content,
            ));
        }

        // Shape B — generic dosage fallback for lines without explicit mm.
        for hit in generic_tds::scan_dosage(content) {
            let (lq, lh) = if hit.per_mm { (0.05, 0.02) } else { (0.12, 0.05) };
            let description = if hit.per_mm {
                format!("{product} — нанасяне, на мм дебелина")
            } else {
                format!("{product} — нанасяне, 1 m²")
            };
            norms.push(generic_tds::make_norm(
                "weber.bg", url, &product, &description, sek, trade, lq, lh, &hit, content,
            ));
        }

        // Dedupe by (description + qty).
        let mut seen = std::collections::HashSet::new();
        norms.retain(|n| {
            let k = format!("{}|{:.2}", n.description_bg, n.materials.first().map(|m| m.qty).unwrap_or(0.0));
            seen.insert(k)
        });

        diagnostics.push(("per_mm_hits", per_mm_hits));
        diagnostics.push(("total_rows", norms.len()));
        let count = norms.len();
        NormParseResult {
            norms,
            strategy_used: "weber_tds",
            candidates_before_filter: count,
            candidates_after_filter: count,
            diagnostics,
        }
    }

    fn category_urls(&self) -> Vec<NormCategoryUrl> {
        vec![
            NormCategoryUrl::pdf("https://www.bg.weber/files/bg/2022-06/weberfloor-basic.pdf",      "СЕК11", "weberfloor basic"),
            NormCategoryUrl::pdf("https://www.bg.weber/files/bg/2022-06/weber-110-G.pdf",            "СЕК05", "weber 110G — зидарски хоросан"),
            NormCategoryUrl::pdf("https://www.bg.weber/files/bg/2022-06/weber-top-202.pdf",          "СЕК09", "webertop 202 — плочки"),
            NormCategoryUrl::pdf("https://www.bg.weber/files/bg/2022-06/weber-therm-universal.pdf",  "СЕК16", "webertherm universal"),
        ]
    }
}

fn sek_for_product(p: &str) -> &'static str {
    let s = p.to_lowercase();
    if s.contains("floor") { "СЕК11" }
    else if s.contains("top") || s.contains("tile") { "СЕК09" }
    else if s.contains("therm") { "СЕК16" }
    else if s.contains("110") || s.contains("base") || s.contains("рус") { "СЕК05" }
    else { "СЕК10" }
}
fn trade_for(sek: &str) -> &'static str {
    match sek {
        "СЕК05" => "зидар",
        "СЕК09" => "плочкар",
        "СЕК11" => "подов майстор",
        "СЕК16" => "изолатор",
        _ => "мазач",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_weberfloor_range_per_mm() {
        let text = "weberfloor basic = 18-20 kg/m² per 10 mm";
        let r = WeberParser.parse_page(text, "u");
        assert!(!r.norms.is_empty());
        // 19 kg / 10 mm ≈ 1.9 kg/m²/mm
        assert!((r.norms[0].materials[0].qty - 1.9).abs() < 0.1);
    }
}
