//! Parser for fibran.bg — ETICS (External Thermal Insulation Composite
//! System) handbook. The single 60-page PDF enumerates per-m² consumption
//! for every system-layer (adhesive, mesh, dowel, reinforcement mortar,
//! primer, finish).

use regex::Regex;
use std::sync::OnceLock;

use super::generic_tds;
use super::{NormCategoryUrl, NormParseResult, NormParser};
use crate::quantity_scraper::{NormMaterial, ScrapedNorm};

pub struct FibranParser;

/// Header line that begins a per-layer section:
///   "Лепило за EPS плочи" / "Армираща смес" / "Стъклофибърна мрежа" / …
static LAYER_RE: OnceLock<Regex> = OnceLock::new();
fn layer_re() -> &'static Regex {
    LAYER_RE.get_or_init(|| {
        Regex::new(r"(?im)^\s*(?P<label>(?:Лепило|Армираща\s+смес|Стъклофибърна\s+мрежа|Грунд|Декоративна\s+мазилка|Шпакловка|Дюбели|EPS|XPS|MW)[^\n]{0,60})").unwrap()
    })
}

/// "3,5 бр./м2" / "1 бр./м2" for dowels + mesh.
static PIECE_PER_M2_RE: OnceLock<Regex> = OnceLock::new();
fn piece_per_m2_re() -> &'static Regex {
    PIECE_PER_M2_RE.get_or_init(|| Regex::new(r"(?P<n>\d+(?:[.,]\d+)?)\s*(?:бр|pcs)\.?\s*/\s*м?2").unwrap())
}

impl NormParser for FibranParser {
    fn site_name(&self) -> &str { "fibran.bg" }
    fn template_key(&self) -> &str { "fibran" }

    fn parse_page(&self, content: &str, url: &str) -> NormParseResult {
        let mut norms: Vec<ScrapedNorm> = Vec::new();
        let mut diagnostics = Vec::new();
        let mut layer_hits = 0usize;

        // For each layer heading, extract dosage figures appearing in the next
        // ±400 chars. Multiple hits per layer are allowed (e.g. adhesive ribbon
        // vs. full-surface).
        for cap in layer_re().captures_iter(content) {
            layer_hits += 1;
            let label = cap["label"].trim().to_string();
            let start = cap.get(0).unwrap().end();
            let window_end = (start + 500).min(content.len());
            let window = &content[start..window_end];

            // kg/m² dosage
            for hit in generic_tds::scan_dosage(window) {
                let (lq, lh) = labor_for_layer(&label);
                norms.push(generic_tds::make_norm(
                    "fibran.bg", url, &label,
                    &format!("ETICS система — {label}, 1 m² стена"),
                    "СЕК16", "изолатор", lq, lh, &hit, window,
                ));
            }
            // pieces/m² (dowels + mesh)
            if let Some(pc) = piece_per_m2_re().captures(window) {
                let n: f64 = pc["n"].replace(',', ".").parse().unwrap_or(0.0);
                if n > 0.0 {
                    norms.push(ScrapedNorm {
                        source_site: "fibran.bg".into(),
                        source_url: url.into(),
                        description_bg: format!("ETICS система — {label}, 1 m² стена"),
                        work_unit: "m²".into(),
                        labor_qualified_h: 0.10, labor_helper_h: 0.03,
                        labor_trade: Some("изолатор".into()),
                        materials: vec![NormMaterial { name: label.clone(), qty: n, unit: "бр.".into() }],
                        machinery: Vec::new(),
                        sek_group_hint: Some("СЕК16".into()),
                        raw_snippet: Some(pc.get(0).unwrap().as_str().chars().take(180).collect()),
                        extraction_confidence: 0.85,
                    });
                }
            }
        }

        // Dedupe per (label + qty) — the handbook repeats tables in TOC.
        let mut seen = std::collections::HashSet::new();
        norms.retain(|n| {
            let k = format!("{}|{:.2}|{}", n.description_bg, n.materials.first().map(|m| m.qty).unwrap_or(0.0), n.work_unit);
            seen.insert(k)
        });

        diagnostics.push(("layer_headers", layer_hits));
        diagnostics.push(("total_rows", norms.len()));
        let count = norms.len();
        NormParseResult {
            norms,
            strategy_used: "fibran_etics",
            candidates_before_filter: count,
            candidates_after_filter: count,
            diagnostics,
        }
    }

    fn category_urls(&self) -> Vec<NormCategoryUrl> {
        vec![
            NormCategoryUrl::pdf(
                "https://fibran.bg/wp-content/uploads/sites/8/2020/03/ETICS-Външни-Топлоизолационни-Комбинирани-Системи.pdf",
                "СЕК16",
                "FIBRAN ETICS handbook",
            ),
        ]
    }
}

fn labor_for_layer(label: &str) -> (f64, f64) {
    let l = label.to_lowercase();
    if l.contains("лепило")     { (0.15, 0.05) }
    else if l.contains("мрежа") { (0.10, 0.05) }
    else if l.contains("грунд") { (0.06, 0.02) }
    else if l.contains("мазилка") { (0.20, 0.08) }
    else if l.contains("дюбел") { (0.08, 0.03) }
    else { (0.12, 0.05) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_etics_sections() {
        let text = "\
Лепило за EPS плочи
Разход: 5,0 kg/m²

Армираща смес
Разход около 4,5 kg/m²

Стъклофибърна мрежа
1,1 бр./м2

Дюбели
6 бр./м2
";
        let r = FibranParser.parse_page(text, "u");
        assert!(r.norms.len() >= 4);
        assert!(r.norms.iter().any(|n| n.description_bg.contains("Лепило")));
        assert!(r.norms.iter().any(|n| n.materials.first().map(|m| m.unit.as_str()) == Some("бр.")));
    }
}
