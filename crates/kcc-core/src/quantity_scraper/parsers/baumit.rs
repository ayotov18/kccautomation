//! Parser for baumit.bg — product datasheets (Техническа карта) with
//! "Зърнометрия и разходни норми" tables. Each product PDF typically has
//! per-layer kg/m² consumption plus mm layer thickness.

use regex::Regex;
use std::sync::OnceLock;

use super::{NormCategoryUrl, NormParseResult, NormParser};
use crate::quantity_scraper::{NormMaterial, ScrapedNorm};

pub struct BaumitParser;

/// "Пердашена мазилка еднослойна:          около 4,5 – 5 kg/m2     3 – 4 mm"
/// Captures: label, lo-hi kg/m², lo-hi mm.
static ROW_RE: OnceLock<Regex> = OnceLock::new();
fn row_re() -> &'static Regex {
    ROW_RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*(?P<label>[A-Za-zА-Яа-я][^\n]{3,60}?)\s+около\s+(?P<klo>\d+(?:[.,]\d+)?)(?:\s*[-–]\s*(?P<khi>\d+(?:[.,]\d+)?))?\s*kg/m2\s+(?P<mlo>\d+(?:[.,]\d+)?)(?:\s*[-–]\s*(?P<mhi>\d+(?:[.,]\d+)?))?\s*mm").unwrap()
    })
}

static PRODUCT_HEADER_RE: OnceLock<Regex> = OnceLock::new();
fn product_header_re() -> &'static Regex {
    PRODUCT_HEADER_RE.get_or_init(|| Regex::new(r"(?i)(Баумит\s+[A-Za-zА-Яа-я][^\n]{2,40})").unwrap())
}

impl NormParser for BaumitParser {
    fn site_name(&self) -> &str { "baumit.bg" }
    fn template_key(&self) -> &str { "baumit" }

    fn parse_page(&self, content: &str, url: &str) -> NormParseResult {
        let product = product_header_re()
            .captures(content)
            .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
            .unwrap_or_else(|| "Баумит продукт".to_string());

        let mut norms: Vec<ScrapedNorm> = Vec::new();
        let mut diagnostics = Vec::new();
        let mut rows = 0usize;

        for c in row_re().captures_iter(content) {
            rows += 1;
            let label = c["label"].trim().to_string();
            let klo: f64 = c["klo"].replace(',', ".").parse().unwrap_or(0.0);
            let khi: f64 = c.name("khi").and_then(|m| m.as_str().replace(',', ".").parse().ok()).unwrap_or(klo);
            let mlo: f64 = c["mlo"].replace(',', ".").parse().unwrap_or(0.0);
            let mhi: f64 = c.name("mhi").and_then(|m| m.as_str().replace(',', ".").parse().ok()).unwrap_or(mlo);

            let k_mid = (klo + khi) / 2.0;
            let m_mid = (mlo + mhi) / 2.0;

            let sek_hint = sek_for_label(&label, &product);

            let norm = ScrapedNorm {
                source_site: "baumit.bg".into(),
                source_url: url.into(),
                description_bg: format!("{product} — {label} (~{m_mid:.1} мм)"),
                work_unit: "m²".into(),
                labor_qualified_h: labor_for_layer(m_mid),
                labor_helper_h: 0.05,
                labor_trade: Some(labor_trade_for(&sek_hint).into()),
                materials: vec![
                    NormMaterial { name: product.clone(), qty: (k_mid * 100.0).round() / 100.0, unit: "кг".into() },
                ],
                machinery: Vec::new(),
                sek_group_hint: Some(sek_hint.to_string()),
                raw_snippet: Some(c.get(0).unwrap().as_str().chars().take(180).collect()),
                extraction_confidence: 0.85,
            };
            norms.push(norm);
        }

        diagnostics.push(("rows", rows));
        let count = norms.len();
        NormParseResult {
            norms,
            strategy_used: "baumit_tk_table",
            candidates_before_filter: count,
            candidates_after_filter: count,
            diagnostics,
        }
    }

    fn category_urls(&self) -> Vec<NormCategoryUrl> {
        vec![
            NormCategoryUrl::pdf("https://baumit.bg/files/bulgaria/products/Classico.pdf",    "СЕК10", "Baumit Classico — благородна мазилка"),
            NormCategoryUrl::pdf("https://baumit.bg/files/bulgaria/products/Nivello.pdf",     "СЕК11", "Baumit Nivello — саморазливна замазка"),
            NormCategoryUrl::pdf("https://baumit.bg/files/bulgaria/products/Beton_Lepilo.pdf","СЕК09", "Baumit Beton Lepilo — плочкарско лепило"),
        ]
    }
}

fn sek_for_label(label: &str, product: &str) -> &'static str {
    let s = format!("{} {}", label, product).to_lowercase();
    if s.contains("мазилка") || s.contains("шпакл") { "СЕК10" }
    else if s.contains("замазк") || s.contains("под") || s.contains("nivello") { "СЕК11" }
    else if s.contains("лепило") || s.contains("плочк") { "СЕК09" }
    else if s.contains("боя") { "СЕК13" }
    else if s.contains("топло") || s.contains("eps") { "СЕК16" }
    else { "СЕК10" }
}

fn labor_trade_for(sek: &str) -> &'static str {
    match sek {
        "СЕК10" => "мазач",
        "СЕК11" => "подов майстор",
        "СЕК09" => "плочкар",
        "СЕК13" => "бояджия",
        "СЕК16" => "изолатор",
        _ => "общ работник",
    }
}

fn labor_for_layer(mm: f64) -> f64 {
    // Rough rule: 0.08 h/m² + 0.01 h per mm of layer thickness.
    ((0.08 + 0.01 * mm) * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_classico_rows() {
        let text = "\
Баумит Класико – благородна мазилка 2 мм
Зърнометрия и разходни норми:
Пердашена мазилка еднослойна:          около 4,5 – 5 kg/m2     3 – 4 mm
       I слой                          около 4,0 kg/m2          3 mm
       II слой                          около 3,0 kg/m2          2 mm
";
        let parser = BaumitParser;
        let r = parser.parse_page(text, "u");
        assert!(r.norms.len() >= 3);
        assert!(r.norms[0].description_bg.contains("Класико"));
        assert_eq!(r.norms[0].materials[0].unit, "кг");
    }
}
