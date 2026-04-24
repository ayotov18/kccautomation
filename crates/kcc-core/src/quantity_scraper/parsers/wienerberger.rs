//! Parser for wienerberger.bg — Porotherm ceramic-block handbook.
//!
//! Wienerberger publishes two PDF series: the technical handbook (mortar-layer
//! thickness, block-per-m² counts) and per-product datasheets. We scrape the
//! handbook for dosage and the datasheets for per-block consumption.

use regex::Regex;
use std::sync::OnceLock;

use super::{NormCategoryUrl, NormParseResult, NormParser};
use crate::quantity_scraper::{NormMachinery, NormMaterial, ScrapedNorm};

pub struct WienerbergerParser;

/// Catches "Porotherm 25 Light plus" and similar "Porotherm NN[…]" variants,
/// then the next "X бр./м²" or "X-Y мм" mortar layer pattern.
static PRODUCT_RE: OnceLock<Regex> = OnceLock::new();
static BRICKS_PER_M2_RE: OnceLock<Regex> = OnceLock::new();
static MORTAR_THICK_RE: OnceLock<Regex> = OnceLock::new();

fn product_re() -> &'static Regex {
    PRODUCT_RE.get_or_init(|| {
        Regex::new(r"Porotherm\s+(?P<thick>\d{2})(?:\s+(?P<variant>[A-Za-z][A-Za-z ]*))?").unwrap()
    })
}
fn bricks_per_m2_re() -> &'static Regex {
    BRICKS_PER_M2_RE.get_or_init(|| Regex::new(r"(?P<n>\d{1,3}(?:[.,]\d)?)\s*бр\./?\s*м?2").unwrap())
}
fn mortar_thick_re() -> &'static Regex {
    MORTAR_THICK_RE.get_or_init(|| Regex::new(r"(?P<lo>\d{1,2})\s*[-–]\s*(?P<hi>\d{1,2})\s*мм").unwrap())
}

impl NormParser for WienerbergerParser {
    fn site_name(&self) -> &str { "wienerberger.bg" }
    fn template_key(&self) -> &str { "wienerberger" }

    fn parse_page(&self, content: &str, url: &str) -> NormParseResult {
        let mut norms: Vec<ScrapedNorm> = Vec::new();
        let mut diagnostics = Vec::new();
        let mut product_hits = 0usize;
        let mut bricks_hits = 0usize;

        // Window-scan: for each Porotherm product mention, look at the following
        // ±600 chars for a bricks-per-m² count and mortar thickness.
        for m in product_re().captures_iter(content) {
            product_hits += 1;
            let full = m.get(0).unwrap();
            let thick: i32 = m.name("thick").and_then(|t| t.as_str().parse().ok()).unwrap_or(0);
            if !(8..=50).contains(&thick) { continue; }
            let variant = m.name("variant").map(|v| v.as_str().trim()).unwrap_or("");

            let start = full.start();
            let end = (full.end() + 800).min(content.len());
            let window = &content[start..end];

            // Bricks-per-m² count (typical Porotherm 25 = 10-11 бр./м²).
            let bricks_per_m2: f64 = bricks_per_m2_re()
                .captures(window)
                .and_then(|c| c.name("n"))
                .and_then(|n| n.as_str().replace(',', ".").parse().ok())
                .unwrap_or_else(|| default_bricks_per_m2(thick));
            if bricks_per_m2_re().is_match(window) { bricks_hits += 1; }

            // Mortar layer thickness "10-12 мм" → midpoint mm.
            let mortar_mm = mortar_thick_re()
                .captures(window)
                .and_then(|c| {
                    let lo: f64 = c.name("lo")?.as_str().parse().ok()?;
                    let hi: f64 = c.name("hi")?.as_str().parse().ok()?;
                    Some((lo + hi) / 2.0)
                })
                .unwrap_or(10.0);

            // Mortar volume per m² wall = thickness_cm × 0.001 × mortar_mm (approx).
            let mortar_l_per_m2 = (thick as f64) * (mortar_mm / 1000.0) * 10.0;

            let product_label = if variant.is_empty() {
                format!("Porotherm {thick}")
            } else {
                format!("Porotherm {thick} {variant}")
            };

            let norm = ScrapedNorm {
                source_site: "wienerberger.bg".into(),
                source_url: url.into(),
                description_bg: format!("Зидария от керамични блокове {product_label}, дебелина {thick} см"),
                work_unit: "m²".into(),
                labor_qualified_h: labor_per_m2_for_thickness(thick).0,
                labor_helper_h:    labor_per_m2_for_thickness(thick).1,
                labor_trade: Some("зидар".into()),
                materials: vec![
                    NormMaterial { name: format!("{} блок {} см", product_label, thick), qty: bricks_per_m2, unit: "бр.".into() },
                    NormMaterial { name: "Вароциментов разтвор М5".into(),                  qty: mortar_l_per_m2, unit: "л".into() },
                ],
                machinery: vec![
                    NormMachinery { name: "Бъркалка".into(), qty: 0.02, unit: "маш.-ч".into() },
                ],
                sek_group_hint: Some("СЕК05".into()),
                raw_snippet: Some(window.chars().take(180).collect()),
                extraction_confidence: 0.8,
            };
            norms.push(norm);
        }

        diagnostics.push(("product_mentions", product_hits));
        diagnostics.push(("bricks_matches", bricks_hits));
        diagnostics.push(("rows", norms.len()));

        // Dedupe by (thickness, variant) — PDF repeats product names in TOC.
        let mut seen = std::collections::HashSet::new();
        norms.retain(|n| seen.insert(n.description_bg.clone()));

        let count = norms.len();
        NormParseResult {
            norms,
            strategy_used: "wienerberger_handbook",
            candidates_before_filter: count,
            candidates_after_filter: count,
            diagnostics,
        }
    }

    fn category_urls(&self) -> Vec<NormCategoryUrl> {
        vec![
            NormCategoryUrl::pdf(
                "https://www.wienerberger.bg/content/dam/wienerberger/bulgaria/marketing/documents-magazines/building-manual/porotherm-tehnichesko-ruk-vodstvo.pdf",
                "СЕК05",
                "Porotherm Техническо ръководство",
            ),
        ]
    }
}

/// УСН-derived defaults for labor hours per m² ceramic-block wall, as a function
/// of wall thickness (cm). Returns (qualified_h, helper_h).
fn labor_per_m2_for_thickness(thick_cm: i32) -> (f64, f64) {
    match thick_cm {
        ..=12 => (0.45, 0.20),
        13..=20 => (0.60, 0.25),
        21..=30 => (0.80, 0.30),
        _ => (1.05, 0.40),
    }
}

/// Empirical block-per-m² defaults when the PDF window doesn't carry the count.
fn default_bricks_per_m2(thick_cm: i32) -> f64 {
    match thick_cm {
        12 => 12.0,
        25 => 10.7,
        30 => 10.7,
        38 => 16.0,
        _  => 11.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_porotherm_25_light_plus() {
        let text = "\
Специални указания за Porotherm 25 Light plus

Porotherm 25 Light plus е с по-големи вертикални отвори. 10.7 бр./м2. Полагане в слой 10-12 мм.
";
        let parser = WienerbergerParser;
        let r = parser.parse_page(text, "u");
        assert!(!r.norms.is_empty());
        assert!(r.norms[0].description_bg.contains("Porotherm 25"));
        assert!(r.norms[0].materials.iter().any(|m| m.unit == "бр."));
    }
}
