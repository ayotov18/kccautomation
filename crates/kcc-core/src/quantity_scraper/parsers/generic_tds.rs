//! Shared extraction primitives used by the multi-vendor TDS parsers
//! (Sika / Mapei / Weber / Fibran / Knauf).
//!
//! They all publish Bulgarian "Техническа карта" PDFs sharing the same
//! dosage-line shape: `Разход[…]: <N>[-<M>] kg|кг/m²|/m²/mm`. We factor out
//! the regex + unit normalisation here so each vendor parser only owns its
//! product-name heuristics and URL seed list.

use regex::Regex;
use std::sync::OnceLock;

use crate::quantity_scraper::{NormMaterial, ScrapedNorm};

/// "Разход: 2,8 kg/m²" / "Разход около 0,3-0,5 кг/м2" / "… 1,94 kg/m²/mm".
/// Note: `/mm` means per-mm-thickness — captured separately and the caller
/// decides how to present it (we emit `unit = "m²·mm"` in that case).
static DOSAGE_RE: OnceLock<Regex> = OnceLock::new();

fn dosage_re() -> &'static Regex {
    DOSAGE_RE.get_or_init(|| {
        // Unit: kg|кг  /  (m|м) (2|²)  optional "/mm" or "/мм" tail.
        Regex::new(
            r"(?i)(?:разход|consumption)[^:\n]{0,30}?[:\-]?\s*(?:около\s+)?(?P<lo>\d+(?:[.,]\d+)?)(?:\s*[-–]\s*(?P<hi>\d+(?:[.,]\d+)?))?\s*(?:kg|кг)\s*/\s*(?:m|м)\s*(?:2|²)(?P<permm>\s*/\s*(?:mm|мм))?"
        ).unwrap()
    })
}

/// Result of a single dosage match.
pub struct DosageHit {
    pub kg_per_m2: f64,
    /// If true, value is per mm of applied-layer thickness.
    pub per_mm: bool,
    /// Span in the source text for audit snippets.
    pub span: (usize, usize),
}

pub fn scan_dosage(text: &str) -> Vec<DosageHit> {
    let mut out = Vec::new();
    for c in dosage_re().captures_iter(text) {
        let lo: f64 = c["lo"].replace(',', ".").parse().unwrap_or(0.0);
        let hi: f64 = c.name("hi").and_then(|m| m.as_str().replace(',', ".").parse().ok()).unwrap_or(lo);
        let mid = (lo + hi) / 2.0;
        if mid <= 0.0 { continue; }
        let full = c.get(0).unwrap();
        out.push(DosageHit {
            kg_per_m2: (mid * 1000.0).round() / 1000.0,
            per_mm: c.name("permm").is_some(),
            span: (full.start(), full.end()),
        });
    }
    out
}

/// Build a single ScrapedNorm from a dosage hit + caller-supplied metadata.
pub fn make_norm(
    site: &str, url: &str, product_label: &str,
    description: &str, sek_hint: &str, trade: &str,
    labor_q: f64, labor_h: f64,
    hit: &DosageHit,
    source_text: &str,
) -> ScrapedNorm {
    let unit = if hit.per_mm { "m²·mm".to_string() } else { "m²".to_string() };
    let mut snippet_start = hit.span.0.saturating_sub(40);
    while snippet_start > 0 && !source_text.is_char_boundary(snippet_start) {
        snippet_start -= 1;
    }
    let mut snippet_end = (hit.span.1 + 60).min(source_text.len());
    while snippet_end < source_text.len() && !source_text.is_char_boundary(snippet_end) {
        snippet_end += 1;
    }
    let snippet: String = source_text[snippet_start..snippet_end].chars().take(240).collect();

    ScrapedNorm {
        source_site: site.to_string(),
        source_url: url.to_string(),
        description_bg: description.to_string(),
        work_unit: unit,
        labor_qualified_h: labor_q,
        labor_helper_h: labor_h,
        labor_trade: Some(trade.to_string()),
        materials: vec![NormMaterial {
            name: product_label.to_string(),
            qty: hit.kg_per_m2,
            unit: "кг".into(),
        }],
        machinery: Vec::new(),
        sek_group_hint: Some(sek_hint.to_string()),
        raw_snippet: Some(snippet),
        extraction_confidence: 0.85,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_sika_style() {
        let hits = scan_dosage("Разход: 2,8 kg/m² ±5%");
        assert_eq!(hits.len(), 1);
        assert!((hits[0].kg_per_m2 - 2.8).abs() < 0.01);
        assert!(!hits[0].per_mm);
    }

    #[test]
    fn matches_per_mm() {
        let hits = scan_dosage("Разход около 1,94 kg/m²/mm");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].per_mm);
    }

    #[test]
    fn matches_range_cyrillic() {
        let hits = scan_dosage("Разход: 0,1-0,3 кг/м2");
        assert_eq!(hits.len(), 1);
        assert!((hits[0].kg_per_m2 - 0.2).abs() < 0.01);
    }
}
