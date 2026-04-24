//! Parser for ytong.bg — YTONG design handbook PDF.
//!
//! The handbook (ПРОЕКТИРАНЕ И СТРОИТЕЛСТВО С ПРОДУКТИ YTONG) has a dedicated
//! "РАЗХОДНИ НОРМИ" chapter with a table per product line
//! (THERMO / KOMFORT / AKUSTIK / SILKA / FORTE). Each table row carries per-m²
//! consumption for one wall thickness.
//!
//! Column layout (verified against ytong_design.pdf sections 15-16):
//!   mm | ч.ч. без отвори | ч.ч. с отвори | m³ блокове | kg лепилен | ℓ пясъчен разтвор | ℓ вода | мсм вдигачка
//!
//! Values in parentheses are "reduced-effort" alternatives — we ignore them and
//! keep the primary (outside-parens) numbers.

use regex::Regex;
use std::sync::OnceLock;

use super::{NormCategoryUrl, NormParseResult, NormParser};
use crate::quantity_scraper::{NormMachinery, NormMaterial, ScrapedNorm};

pub struct YtongParser;

/// Line shape:
///   `350             1.03 (0.89)   1.09 (0.96)       0.357       5.25 (3.85)        4.12     1.58 (1.16)     0.0305`
/// Leading number = thickness (mm). The 7 floats that follow are the consumption values.
static ROW_RE: OnceLock<Regex> = OnceLock::new();
fn row_re() -> &'static Regex {
    ROW_RE.get_or_init(|| {
        // thickness  labor1  labor2  blocks_m3  glue_kg  mortar_l  water_l  lift_msm
        // Each "col" may be `<num>` or `<num> (<alt>)`.
        Regex::new(
            r"(?m)^\s*(?P<mm>\d{2,3})\s+(?P<l1>\d+\.\d+)(?:\s*\([^)]*\))?\s+(?P<l2>\d+\.\d+)(?:\s*\([^)]*\))?\s+(?P<blocks>\d+\.\d+)\s+(?P<glue>\d+\.\d+)(?:\s*\([^)]*\))?\s+(?P<mortar>\d+\.\d+)\s+(?P<water>\d+\.\d+)(?:\s*\([^)]*\))?\s+(?P<lift>\d+\.\d+)\s*$"
        ).unwrap()
    })
}

/// Product-section headers we honour. Order matters: the first header we see
/// above a row is the product line that row belongs to.
const PRODUCT_HEADERS: &[(&str, &str)] = &[
    ("YTONG THERMO",  "YTONG THERMO"),
    ("YTONG KOMFORT", "YTONG KOMFORT"),
    ("YTONG AKUSTIK", "YTONG AKUSTIK"),
    ("YTONG SILKA",   "YTONG SILKA"),
    ("YTONG FORTE",   "YTONG FORTE"),
];

impl NormParser for YtongParser {
    fn site_name(&self) -> &str {
        "ytong.bg"
    }
    fn template_key(&self) -> &str {
        "ytong"
    }

    fn parse_page(&self, content: &str, url: &str) -> NormParseResult {
        // Only run on the "РАЗХОДНИ НОРМИ" block — the rest of the handbook has
        // prose that mustn't be regex-scanned or we'd false-positive on any 3
        // numeric tokens in a row.
        let start_idx = content.find("РАЗХОДНИ НОРМИ");
        let stop_idx = content.rfind("ОПТИМИЗИРАНЕ НА ПРОЕКТНИТЕ РЕШЕНИЯ")
            .or_else(|| content.rfind("ИЗБЯГВАНЕ НА ГРЕШКИ"));

        let mut diagnostics: Vec<(&'static str, usize)> = Vec::new();

        let slice = match (start_idx, stop_idx) {
            (Some(s), Some(e)) if e > s => &content[s..e],
            (Some(s), _) => &content[s..],
            _ => {
                diagnostics.push(("section_missing", 0));
                return NormParseResult {
                    norms: Vec::new(),
                    strategy_used: "none",
                    candidates_before_filter: 0,
                    candidates_after_filter: 0,
                    diagnostics,
                };
            }
        };

        let mut current_product: &str = "YTONG THERMO";
        let mut norms: Vec<ScrapedNorm> = Vec::new();
        let mut product_matches: usize = 0;

        for raw_line in slice.lines() {
            // Product header — update cursor.
            for (needle, label) in PRODUCT_HEADERS {
                if raw_line.contains(needle) {
                    current_product = label;
                    product_matches += 1;
                    break;
                }
            }

            // Row.
            if let Some(caps) = row_re().captures(raw_line) {
                let mm: i32 = caps["mm"].parse().unwrap_or(0);
                if !(50..=500).contains(&mm) {
                    continue; // not a wall thickness, stray match
                }
                let labor_full: f64 = caps["l1"].parse().unwrap_or(0.0);
                let blocks_m3: f64 = caps["blocks"].parse().unwrap_or(0.0);
                let glue_kg: f64 = caps["glue"].parse().unwrap_or(0.0);
                let mortar_l: f64 = caps["mortar"].parse().unwrap_or(0.0);
                let water_l: f64 = caps["water"].parse().unwrap_or(0.0);
                let lift_msm: f64 = caps["lift"].parse().unwrap_or(0.0);

                let description = format!(
                    "Зидария от {product} блокове, дебелина {mm} мм",
                    product = current_product,
                    mm = mm,
                );

                let norm = ScrapedNorm {
                    source_site: "ytong.bg".to_string(),
                    source_url: url.to_string(),
                    description_bg: description,
                    work_unit: "m²".to_string(),
                    labor_qualified_h: (labor_full * 2.0 / 3.0 * 100.0).round() / 100.0,
                    labor_helper_h:    (labor_full      / 3.0 * 100.0).round() / 100.0,
                    labor_trade: Some("зидар".to_string()),
                    materials: vec![
                        NormMaterial { name: format!("{} блок {} мм", current_product, mm), qty: blocks_m3, unit: "m³".into() },
                        NormMaterial { name: "Сух лепилен разтвор".into(),                  qty: glue_kg,    unit: "кг".into() },
                        NormMaterial { name: "Цименто-пясъчен разтвор 1:3".into(),         qty: mortar_l,   unit: "л".into() },
                        NormMaterial { name: "Вода".into(),                                 qty: water_l,    unit: "л".into() },
                    ],
                    machinery: vec![
                        NormMachinery { name: "Асансьорна вдигачка".into(), qty: lift_msm, unit: "маш.-см".into() },
                    ],
                    sek_group_hint: Some("СЕК05".into()),
                    raw_snippet: Some(raw_line.trim().chars().take(200).collect()),
                    extraction_confidence: 0.9, // directly from the manufacturer's handbook
                };
                norms.push(norm);
            }
        }

        diagnostics.push(("product_headers", product_matches));
        diagnostics.push(("rows_matched", norms.len()));

        let candidates = norms.len();
        NormParseResult {
            norms,
            strategy_used: "ytong_handbook_tables",
            candidates_before_filter: candidates,
            candidates_after_filter: candidates,
            diagnostics,
        }
    }

    fn category_urls(&self) -> Vec<NormCategoryUrl> {
        vec![
            NormCategoryUrl::pdf(
                "https://www.ytong.bg/files/9_Design_manual_Ytong_A4_FULL_2021.pdf",
                "СЕК05",
                "YTONG Design Handbook — РАЗХОДНИ НОРМИ",
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "\
РАЗХОДНИ НОРМИ
YTONG THERMO
       mm                ч.ч.           ч.ч.           m3             kg               ℓ           ℓ           мсм
       350             1.03 (0.89)   1.09 (0.96)       0.357       5.25 (3.85)        4.12     1.58 (1.16)     0.0305
       250             0.68 (0.59)   0.74 (0.66)       0.255       3.75 (2.75)        2.94     1.13 (0.83)     0.0215

YTONG KOMFORT
       mm                ч.ч.           ч.ч.           m3             kg               ℓ           ℓ           мсм
       100             0.66 (0.57)   0.71 (0.62)       0.102       1.50 (1.10)        1.18     0.45 (0.33)     0.0085
ОПТИМИЗИРАНЕ НА ПРОЕКТНИТЕ РЕШЕНИЯ
";

    #[test]
    fn parses_thermo_and_komfort_rows() {
        let parser = YtongParser;
        let result = parser.parse_page(FIXTURE, "https://example/ytong.pdf");
        assert_eq!(result.norms.len(), 3);
        assert!(result.norms[0].description_bg.contains("THERMO"));
        assert!(result.norms[0].description_bg.contains("350"));
        assert!(result.norms[2].description_bg.contains("KOMFORT"));
        // Material unit for glue should be кг.
        assert!(result.norms[0].materials.iter().any(|m| m.name.contains("Сух лепилен") && m.unit == "кг"));
        // Total labor hours round-trip preserved.
        assert!((result.norms[0].total_labor_h() - 1.03).abs() < 0.05);
    }

    #[test]
    fn ignores_section_when_missing() {
        let parser = YtongParser;
        let result = parser.parse_page("some unrelated PDF text", "u");
        assert!(result.norms.is_empty());
        assert_eq!(result.strategy_used, "none");
    }
}
