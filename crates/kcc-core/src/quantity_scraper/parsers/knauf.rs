//! Parser for knauf.bg / Knauf Bulgaria Download Center.
//!
//! Knauf drywall catalogs tabulate per-m² material consumption for each wall
//! or ceiling system:
//!   "W111 стена — 2 × гипсокартон 12,5 мм, 2 × OSB 12 мм, ..."
//!   "Профил UW 75:           0,9 м/м²"
//!   "Профил CW 75:           2,0 м/м²"
//!   "Винтове TN 25:          25 бр./м²"

use regex::Regex;
use std::sync::OnceLock;

use super::{NormCategoryUrl, NormParseResult, NormParser};
use crate::quantity_scraper::{NormMaterial, ScrapedNorm};

pub struct KnaufParser;

/// System header like "W111 Метална конструкция" or "D112 Окачен таван".
static SYSTEM_RE: OnceLock<Regex> = OnceLock::new();
fn system_re() -> &'static Regex {
    SYSTEM_RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*(?P<code>[WDFKS]\s?\d{2,3})[^\n]{0,60}").unwrap()
    })
}

/// Consumption row inside a system table.
///   "Профил CW 75:           2,0 м/м²"
///   "Гипсокартон 12,5 мм:    2,1 м²/м²"
///   "Винтове TN 25:          25 бр./м²"
static ROW_RE: OnceLock<Regex> = OnceLock::new();
fn row_re() -> &'static Regex {
    ROW_RE.get_or_init(|| {
        // Right-hand side divisor accepts м², m², м2, m2. The left-hand unit may
        // itself carry ² (e.g. "2,1 м²/м²") — we accept that as a trailing glyph
        // and throw it away when normalising.
        Regex::new(r"(?m)^\s*(?P<label>[A-Za-zА-Яа-я][^:\n]{3,60}?)\s*:?\s+(?P<qty>\d+(?:[.,]\d+)?)\s*(?P<unit>м²|m²|м|кг|бр\.|m|kg|pcs)\s*/\s*(?:м|m)?\s*(?:2|²)").unwrap()
    })
}

impl NormParser for KnaufParser {
    fn site_name(&self) -> &str { "knauf.bg" }
    fn template_key(&self) -> &str { "knauf" }

    fn parse_page(&self, content: &str, url: &str) -> NormParseResult {
        let mut diagnostics = Vec::new();
        let mut norms: Vec<ScrapedNorm> = Vec::new();

        let systems: Vec<(usize, String)> = system_re()
            .captures_iter(content)
            .map(|c| {
                let start = c.get(0).unwrap().start();
                let code = c["code"].replace(char::is_whitespace, "");
                (start, code)
            })
            .collect();
        diagnostics.push(("system_headers", systems.len()));

        // For each system, collect rows until the next system header.
        for (i, (start, code)) in systems.iter().enumerate() {
            let end = systems.get(i + 1).map(|(s, _)| *s).unwrap_or(content.len());
            let window = &content[*start..end];

            let sek = sek_for_code(code);
            let trade = trade_for(sek);

            let mut materials: Vec<NormMaterial> = Vec::new();
            for c in row_re().captures_iter(window) {
                let label = c["label"].trim().to_string();
                if label.len() < 3 { continue; }
                let qty: f64 = c["qty"].replace(',', ".").parse().unwrap_or(0.0);
                let unit_raw = c["unit"].to_string();
                let unit = normalise_unit(&unit_raw);
                if qty <= 0.0 { continue; }
                materials.push(NormMaterial { name: label, qty, unit });
            }

            if materials.is_empty() { continue; }

            norms.push(ScrapedNorm {
                source_site: "knauf.bg".into(),
                source_url: url.into(),
                description_bg: format!("Knauf {code} — монтаж на 1 m² система"),
                work_unit: "m²".into(),
                labor_qualified_h: 0.55,
                labor_helper_h: 0.20,
                labor_trade: Some(trade.into()),
                materials,
                machinery: Vec::new(),
                sek_group_hint: Some(sek.into()),
                raw_snippet: Some(window.chars().take(240).collect()),
                extraction_confidence: 0.85,
            });
        }

        diagnostics.push(("systems_emitted", norms.len()));
        let count = norms.len();
        NormParseResult {
            norms,
            strategy_used: "knauf_dc_tables",
            candidates_before_filter: count,
            candidates_after_filter: count,
            diagnostics,
        }
    }

    fn category_urls(&self) -> Vec<NormCategoryUrl> {
        vec![
            NormCategoryUrl::pdf("https://knauf.com/bg-BG/system-brochures/W11-metal-walls.pdf",  "СЕК20", "Knauf W11 — метални преградни стени"),
            NormCategoryUrl::pdf("https://knauf.com/bg-BG/system-brochures/D11-metal-ceilings.pdf","СЕК20", "Knauf D11 — метални окачени тавани"),
            NormCategoryUrl::pdf("https://knauf.com/bg-BG/system-brochures/W62-shaft-walls.pdf",  "СЕК20", "Knauf W62 — шахтни стени"),
            NormCategoryUrl::pdf("https://knauf.com/bg-BG/system-brochures/F13-screeds.pdf",       "СЕК11", "Knauf F13 — замазки"),
            NormCategoryUrl::pdf("https://knauf.com/bg-BG/system-brochures/K21-exterior-facade.pdf","СЕК16", "Knauf K21 — фасадни системи"),
        ]
    }
}

fn sek_for_code(code: &str) -> &'static str {
    match code.chars().next() {
        Some('W') | Some('D') => "СЕК20", // walls & ceilings → drywall
        Some('F') => "СЕК11",              // floor screeds
        Some('K') => "СЕК16",              // exterior thermal
        Some('S') => "СЕК10",              // plasters
        _ => "СЕК20",
    }
}
fn trade_for(sek: &str) -> &'static str {
    match sek {
        "СЕК20" => "сухостроител",
        "СЕК11" => "подов майстор",
        "СЕК16" => "изолатор",
        "СЕК10" => "мазач",
        _ => "общ работник",
    }
}
fn normalise_unit(raw: &str) -> String {
    match raw {
        "м" | "m"      => "м".into(),
        "м²" | "m²"    => "m²".into(),
        "кг" | "kg"    => "кг".into(),
        "бр." | "pcs"  => "бр.".into(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_w111_system() {
        let text = "\
W111 Метална конструкция с двойна обшивка

Гипсокартон 12,5 мм:    2,1 м²/м²
Профил CW 75:            2,0 м/м²
Профил UW 75:            0,9 м/м²
Винтове TN 25:           25 бр./м²
";
        let r = KnaufParser.parse_page(text, "u");
        assert!(!r.norms.is_empty());
        let n = &r.norms[0];
        assert!(n.description_bg.starts_with("Knauf W111"));
        assert!(n.materials.iter().any(|m| m.name.starts_with("Профил CW")));
        assert!(n.materials.iter().any(|m| m.unit == "бр."));
    }
}
