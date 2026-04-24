//! Parser for bgr.sika.com — Sika Bulgaria TDS PDFs.

use regex::Regex;
use std::sync::OnceLock;

use super::generic_tds;
use super::{NormCategoryUrl, NormParseResult, NormParser};
use crate::quantity_scraper::ScrapedNorm;

pub struct SikaParser;

static PRODUCT_RE: OnceLock<Regex> = OnceLock::new();
fn product_re() -> &'static Regex {
    // "Sika ThermoCoat Acryl EPS", "Sikadur-31 CF", "SikaTop Seal-107", …
    PRODUCT_RE.get_or_init(|| Regex::new(r"(?i)(Sika(?:flex|dur|bond|Top|Plan|Tack|Swell|Grout|Fiber|Coat|Fill|Monotop|Cem|Latex|Wrap|Fire)?[A-Za-z0-9\- ]{0,40})").unwrap())
}

impl NormParser for SikaParser {
    fn site_name(&self) -> &str { "bgr.sika.com" }
    fn template_key(&self) -> &str { "sika" }

    fn parse_page(&self, content: &str, url: &str) -> NormParseResult {
        let product = product_re()
            .find(content)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| "Sika".to_string());

        let sek = sek_for_product(&product);
        let trade = trade_for_sek(sek);

        let mut norms: Vec<ScrapedNorm> = Vec::new();
        for hit in generic_tds::scan_dosage(content) {
            let (lq, lh) = labor_for_sek(sek, hit.per_mm);
            let description = if hit.per_mm {
                format!("{product} — нанасяне, на мм дебелина")
            } else {
                format!("{product} — нанасяне, 1 m²")
            };
            norms.push(generic_tds::make_norm(
                "bgr.sika.com", url, &product, &description, sek, trade, lq, lh, &hit, content,
            ));
        }

        let count = norms.len();
        NormParseResult {
            norms,
            strategy_used: "sika_tds",
            candidates_before_filter: count,
            candidates_after_filter: count,
            diagnostics: vec![("dosage_hits", count)],
        }
    }

    fn category_urls(&self) -> Vec<NormCategoryUrl> {
        vec![
            NormCategoryUrl::pdf("https://bgr.sika.com/dms/getdocument.get/tds/sika-thermocoat-acryl-eps.pdf", "СЕК16", "Sika ThermoCoat Acryl EPS"),
            NormCategoryUrl::pdf("https://bgr.sika.com/dms/getdocument.get/tds/sikadur-31-cf-rapid.pdf",       "СЕК14", "Sikadur 31 CF Rapid"),
            NormCategoryUrl::pdf("https://bgr.sika.com/dms/getdocument.get/tds/sikatop-seal-107.pdf",          "СЕК15", "SikaTop Seal 107"),
            NormCategoryUrl::pdf("https://bgr.sika.com/dms/getdocument.get/tds/sika-monotop-412.pdf",          "СЕК04", "Sika MonoTop 412"),
            NormCategoryUrl::pdf("https://bgr.sika.com/dms/getdocument.get/tds/sikafloor-263-sl.pdf",          "СЕК11", "Sikafloor 263 SL"),
        ]
    }
}

fn sek_for_product(p: &str) -> &'static str {
    let s = p.to_lowercase();
    if s.contains("thermo")   { "СЕК16" }
    else if s.contains("seal") || s.contains("top")         { "СЕК15" }
    else if s.contains("floor") || s.contains("level")     { "СЕК11" }
    else if s.contains("dur") || s.contains("grout") || s.contains("monotop") { "СЕК04" }
    else if s.contains("wrap") || s.contains("fiber")       { "СЕК14" }
    else if s.contains("bond") || s.contains("flex")        { "СЕК15" }
    else { "СЕК15" }
}

fn trade_for_sek(sek: &str) -> &'static str {
    match sek {
        "СЕК04" => "бетонджия",
        "СЕК09" => "плочкар",
        "СЕК11" => "подов майстор",
        "СЕК13" => "бояджия",
        "СЕК14" => "стоманов монтажник",
        "СЕК15" => "хидроизолатор",
        "СЕК16" => "изолатор",
        _ => "мазач",
    }
}

fn labor_for_sek(sek: &str, per_mm: bool) -> (f64, f64) {
    // Per-mm applications are thinner → less labor per invocation.
    let base = match sek {
        "СЕК15" => (0.15, 0.05),
        "СЕК16" => (0.20, 0.07),
        "СЕК11" => (0.15, 0.05),
        "СЕК04" => (0.10, 0.05),
        "СЕК14" => (0.25, 0.10),
        _ => (0.10, 0.04),
    };
    if per_mm {
        (base.0 * 0.4, base.1 * 0.4)
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sika_thermocoat() {
        let text = "Sika ThermoCoat Acryl EPS\nРазход: 2,8 kg/m² ±5%";
        let r = SikaParser.parse_page(text, "u");
        assert!(!r.norms.is_empty());
        assert_eq!(r.norms[0].sek_group_hint.as_deref(), Some("СЕК16"));
    }

    #[test]
    fn parses_sikadur_per_mm() {
        let text = "Sikadur-31 CF Rapid\nРазход около 1,94 kg/m²/mm";
        let r = SikaParser.parse_page(text, "u");
        assert!(!r.norms.is_empty());
        assert!(r.norms[0].work_unit.contains("mm"));
    }
}
