//! Parser for daibau.bg — Bulgarian construction services price guide.
//!
//! 4-layer fallback extraction:
//!   A) .citwrp containers (primary calculator blocks)
//!   B) Semantic price rows from content elements
//!   C) Table-based extraction
//!   D) Section heading + adjacent content
//!
//! Daibau shows EUR prices. We extract EUR and derive €.

use scraper::{Html, Selector, ElementRef};
use std::collections::HashSet;

use super::{CategoryUrl, ParseResult, PriceParser};
use crate::scraper::ScrapedPrice;
use crate::scraper::price_utils::{self, PriceCurrency};

pub struct DaibauParser;

impl PriceParser for DaibauParser {
    fn site_name(&self) -> &str {
        "daibau.bg"
    }

    fn parse_page(&self, html: &str, url: &str) -> ParseResult {
        let doc = Html::parse_document(html);
        let mut diagnostics: Vec<(&'static str, usize)> = Vec::new();
        let mut all_prices: Vec<ScrapedPrice> = Vec::new();
        let mut primary_strategy = "none";

        // Layer A: .citwrp containers
        let (prices_a, citwrp_count) = try_citwrp_parse(&doc, url);
        diagnostics.push((".citwrp", citwrp_count));
        if !prices_a.is_empty() { primary_strategy = "primary_blocks"; }
        all_prices.extend(prices_a);

        // Layer B: Semantic price rows — always run
        let (prices_b, semantic_count) = try_semantic_parse(&doc, url);
        diagnostics.push(("semantic_rows", semantic_count));
        if primary_strategy == "none" && !prices_b.is_empty() { primary_strategy = "semantic_rows"; }
        all_prices.extend(prices_b);

        // Layer C: Table-based extraction — always run
        let (prices_c, table_count) = try_table_parse(&doc, url);
        diagnostics.push(("tables", table_count));
        if primary_strategy == "none" && !prices_c.is_empty() { primary_strategy = "tables"; }
        all_prices.extend(prices_c);

        // Layer D: Section headings — always run
        let (prices_d, section_count) = try_section_parse(&doc, url);
        diagnostics.push(("section_scan", section_count));
        if primary_strategy == "none" && !prices_d.is_empty() { primary_strategy = "section_scan"; }
        all_prices.extend(prices_d);

        // Dedupe merged results by normalized description
        let candidates_before = all_prices.len();
        dedupe_prices(&mut all_prices);
        all_prices.retain(|p| price_utils::is_valid_description(&p.description_bg));

        ParseResult {
            candidates_before_filter: candidates_before,
            candidates_after_filter: all_prices.len(),
            strategy_used: primary_strategy,
            diagnostics,
            prices: all_prices,
        }
    }

    fn category_urls(&self) -> Vec<CategoryUrl> {
        vec![
            // Masonry / structural
            cat("https://www.daibau.bg/ceni/zidarski_raboti_zidar", "СЕК05", "Зидарски работи"),
            cat("https://www.daibau.bg/ceni/betonovi_raboti", "СЕК04", "Бетонови работи"),
            cat("https://www.daibau.bg/ceni/armatura", "СЕК03", "Арматура"),
            // Finishing
            cat("https://www.daibau.bg/ceni/boyadisvane", "СЕК13", "Бояджийски работи"),
            cat("https://www.daibau.bg/ceni/mazilka", "СЕК10", "Мазачески работи"),
            cat("https://www.daibau.bg/ceni/podovi_nastilki", "СЕК11", "Настилки"),
            cat("https://www.daibau.bg/ceni/polagane_na_plochki", "СЕК09", "Плочки"),
            cat("https://www.daibau.bg/ceni/suho_stroitelstvo", "СЕК20", "Сухо строителство"),
            cat("https://www.daibau.bg/ceni/dekorativni_boyadzhiyski_raboti", "СЕК13", "Декоративни бояджийски"),
            // Insulation / waterproofing
            cat("https://www.daibau.bg/ceni/hidroizolacia", "СЕК15", "Хидроизолации"),
            cat("https://www.daibau.bg/ceni/toploizolacia", "СЕК16", "Топлоизолации"),
            cat("https://www.daibau.bg/ceni/sanirane", "СЕК16", "Саниране"),
            // Roofing
            cat("https://www.daibau.bg/ceni/pokrivni_raboti", "СЕК06", "Покривни работи"),
            cat("https://www.daibau.bg/ceni/remont_na_pokriv", "СЕК06", "Ремонт покрив"),
            // Installations
            cat("https://www.daibau.bg/ceni/elektricheska_instalacia", "СЕК34", "Електрическа"),
            cat("https://www.daibau.bg/ceni/vodoprovodchik", "СЕК22", "ВиК"),
            cat("https://www.daibau.bg/ceni/otoplenie", "СЕК18", "Отопление"),
            cat("https://www.daibau.bg/ceni/klimatizacia", "СЕК18", "Климатизация"),
            // Doors / windows
            cat("https://www.daibau.bg/ceni/aluministka_dograma", "СЕК17", "Алум. дограма"),
            cat("https://www.daibau.bg/ceni/pvc_dograma", "СЕК17", "PVC дограма"),
            cat("https://www.daibau.bg/ceni/vrati", "СЕК17", "Врати"),
            // Exterior / landscaping
            cat("https://www.daibau.bg/ceni/ogradi", "СЕК14", "Огради"),
            cat("https://www.daibau.bg/ceni/fasadi", "СЕК10", "Фасади"),
            cat("https://www.daibau.bg/ceni/terasi", "СЕК11", "Тераси"),
            // Renovation
            cat("https://www.daibau.bg/ceni/remont_na_banya", "СЕК22", "Ремонт баня"),
            cat("https://www.daibau.bg/ceni/remont_na_kuhnya", "СЕК09", "Ремонт кухня"),
            cat("https://www.daibau.bg/ceni/remont_na_apartament", "СЕК10", "Ремонт апартамент"),
            // Demolition / earthwork
            cat("https://www.daibau.bg/ceni/izkopaване", "СЕК01", "Изкопни работи"),
            cat("https://www.daibau.bg/ceni/rushene", "СЕК49", "Събаряне"),
        ]
    }
}

fn cat(url: &str, sek: &str, name: &str) -> CategoryUrl {
    CategoryUrl {
        url: url.to_string(),
        sek_group_hint: sek.to_string(),
        category_name: name.to_string(),
    }
}

// ── Layer A: .citwrp containers ─────────────────────────────

fn try_citwrp_parse(doc: &Html, url: &str) -> (Vec<ScrapedPrice>, usize) {
    let container_sel = match Selector::parse(".citwrp") {
        Ok(s) => s,
        Err(_) => return (Vec::new(), 0),
    };

    let mut prices = Vec::new();
    let mut count = 0;

    for container in doc.select(&container_sel) {
        count += 1;
        if let Some(price) = parse_citwrp_container(&container, url) {
            prices.push(price);
        }
    }

    (prices, count)
}

fn parse_citwrp_container(el: &ElementRef, url: &str) -> Option<ScrapedPrice> {
    // Description from .grpttl
    let desc = Selector::parse(".grpttl").ok()
        .and_then(|sel| el.select(&sel).next())
        .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|d| !d.is_empty())?;

    // Prices from .calc-cell
    let mut min_val = None;
    let mut max_val = None;
    let mut unit = None;
    let mut raw_parts = Vec::new();

    if let Ok(cell_sel) = Selector::parse(".calc-cell") {
        for cell in el.select(&cell_sel) {
            let text = cell.text().collect::<Vec<_>>().join(" ");
            let text = text.trim();
            raw_parts.push(text.to_string());

            let lower = text.to_lowercase()
                .replace("oт", "от")
                .replace("ot", "от");

            if lower.contains("от") {
                min_val = price_utils::extract_number(&lower);
            } else if lower.contains("до") {
                max_val = price_utils::extract_number(&lower);
            }

            if unit.is_none() {
                unit = price_utils::extract_unit(text);
            }
        }
    }

    // Fallback: parse all text in container
    if min_val.is_none() && max_val.is_none() {
        let all_text = el.text().collect::<Vec<_>>().join(" ");
        if let Some(parsed) = price_utils::parse_price_text(&all_text) {
            min_val = parsed.min;
            max_val = parsed.max;
        }
        if unit.is_none() {
            unit = price_utils::extract_unit(&all_text);
        }
    }

    if min_val.is_none() && max_val.is_none() {
        return None;
    }

    let raw_text = raw_parts.join(" | ");
    let unit_str = unit.unwrap_or_else(|| "М2".to_string());

    // Daibau shows EUR — derive lv
    Some(ScrapedPrice::from_eur(
        "daibau.bg", url,
        &price_utils::clean_description(&desc),
        &unit_str,
        min_val, max_val,
        Some(&raw_text), None,
        0.9,
    ))
}

// ── Layer B: Semantic price rows from content elements ──────

fn try_semantic_parse(doc: &Html, url: &str) -> (Vec<ScrapedPrice>, usize) {
    let content_selectors = ["h2", "h3", "h4", "p", "li", "td", "span", "div.price", "div.cena"];
    let mut prices = Vec::new();
    let mut total_candidates = 0;
    let mut seen_texts = HashSet::new();

    for sel_str in &content_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            for el in doc.select(&sel) {
                let text = el.text().collect::<Vec<_>>().join(" ");
                let text = text.trim();

                if text.len() < 5 || text.len() > 500 {
                    continue;
                }

                // Must contain a price indicator
                if !has_price_indicator(text) {
                    continue;
                }

                total_candidates += 1;

                if let Some(parsed) = price_utils::parse_price_text(text) {
                    // Extract description: text before the first digit
                    let desc = extract_description_from_text(text);
                    if desc.is_empty() || !price_utils::is_valid_description(&desc) {
                        continue;
                    }

                    // Dedupe by description
                    let key = desc.to_lowercase();
                    if seen_texts.contains(&key) {
                        continue;
                    }
                    seen_texts.insert(key);

                    let unit = price_utils::extract_unit(text).unwrap_or_else(|| "М2".to_string());
                    let price = match parsed.currency {
                        PriceCurrency::Eur => ScrapedPrice::from_eur(
                            "daibau.bg", url, &desc, &unit,
                            parsed.min, parsed.max,
                            Some(text), None, 0.7,
                        ),
                        _ => ScrapedPrice::from_eur(
                            "daibau.bg", url, &desc, &unit,
                            parsed.min, parsed.max,
                            Some(text), None, 0.7,
                        ),
                    };
                    prices.push(price);
                }
            }
        }
    }

    (prices, total_candidates)
}

// ── Layer C: Table extraction ───────────────────────────────

fn try_table_parse(doc: &Html, url: &str) -> (Vec<ScrapedPrice>, usize) {
    let table_sel = match Selector::parse("table") {
        Ok(s) => s,
        Err(_) => return (Vec::new(), 0),
    };
    let tr_sel = match Selector::parse("tr") {
        Ok(s) => s,
        Err(_) => return (Vec::new(), 0),
    };
    let td_sel = match Selector::parse("td, th") {
        Ok(s) => s,
        Err(_) => return (Vec::new(), 0),
    };

    let mut prices = Vec::new();
    let mut row_count = 0;

    for table in doc.select(&table_sel) {
        for row in table.select(&tr_sel) {
            let cells: Vec<String> = row
                .select(&td_sel)
                .map(|td| td.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            if cells.len() < 2 {
                continue;
            }
            row_count += 1;

            // Find description (longest non-price cell) and price cell
            let mut desc = String::new();
            let mut parsed_price = None;
            let mut unit = None;

            for cell in &cells {
                if let Some(p) = price_utils::parse_price_text(cell) {
                    if parsed_price.is_none() {
                        parsed_price = Some(p);
                    }
                } else if cell.len() > desc.len() && price_utils::is_valid_description(cell) {
                    desc = price_utils::clean_description(cell);
                }
                if unit.is_none() {
                    unit = price_utils::extract_unit(cell);
                }
            }

            if let (Some(parsed), true) = (parsed_price, !desc.is_empty()) {
                let unit_str = unit.unwrap_or_else(|| "М2".to_string());
                let price = match parsed.currency {
                    PriceCurrency::Eur => ScrapedPrice::from_eur(
                        "daibau.bg", url, &desc, &unit_str,
                        parsed.min, parsed.max,
                        Some(&parsed.raw_text), None, 0.6,
                    ),
                    _ => ScrapedPrice::from_eur(
                        "daibau.bg", url, &desc, &unit_str,
                        parsed.min, parsed.max,
                        Some(&parsed.raw_text), None, 0.6,
                    ),
                };
                prices.push(price);
            }
        }
    }

    (prices, row_count)
}

// ── Layer D: Section heading scan ───────────────────────────

fn try_section_parse(doc: &Html, url: &str) -> (Vec<ScrapedPrice>, usize) {
    let heading_sel = match Selector::parse("h2, h3, h4") {
        Ok(s) => s,
        Err(_) => return (Vec::new(), 0),
    };

    let mut prices = Vec::new();
    let mut heading_count = 0;

    for heading in doc.select(&heading_sel) {
        let heading_text = heading.text().collect::<Vec<_>>().join(" ");
        let heading_text = heading_text.trim();

        // Only process headings that look like construction work descriptions
        if !has_construction_keyword(heading_text) {
            continue;
        }
        heading_count += 1;

        // Look at sibling/following content for prices
        // Since scraper doesn't have next_sibling easily, check parent's children
        if let Some(parent) = heading.parent() {
            for child in parent.children() {
                if let Some(el) = ElementRef::wrap(child) {
                    let tag = el.value().name();
                    if tag == "p" || tag == "ul" || tag == "ol" || tag == "div" {
                        let text = el.text().collect::<Vec<_>>().join(" ");
                        if let Some(parsed) = price_utils::parse_price_text(&text) {
                            let desc = price_utils::clean_description(heading_text);
                            if price_utils::is_valid_description(&desc) {
                                let unit = price_utils::extract_unit(&text).unwrap_or_else(|| "М2".to_string());
                                let price = match parsed.currency {
                                    PriceCurrency::Eur => ScrapedPrice::from_eur(
                                        "daibau.bg", url, &desc, &unit,
                                        parsed.min, parsed.max,
                                        Some(text.trim()), None, 0.5,
                                    ),
                                    _ => ScrapedPrice::from_eur(
                                        "daibau.bg", url, &desc, &unit,
                                        parsed.min, parsed.max,
                                        Some(text.trim()), None, 0.5,
                                    ),
                                };
                                prices.push(price);
                                break; // one price per heading
                            }
                        }
                    }
                }
            }
        }
    }

    (prices, heading_count)
}

// ── Helpers ─────────────────────────────────────────────────

/// Dedupe prices by normalized description (case-insensitive, whitespace-collapsed).
fn dedupe_prices(prices: &mut Vec<ScrapedPrice>) {
    let mut seen = HashSet::new();
    prices.retain(|p| {
        let key = p.description_bg.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
        seen.insert(key)
    });
}

fn has_price_indicator(text: &str) -> bool {
    text.contains("€") || text.contains("€") || text.contains("EUR")
        || text.contains("лева") || text.contains("EUR")
}

fn has_construction_keyword(text: &str) -> bool {
    let lower = text.to_lowercase();
    let keywords = [
        "зидария", "мазилка", "боядисване", "настилк", "покрив", "изолаци",
        "бетон", "кофраж", "арматур", "дограм", "ВиК", "вик", "електр",
        "облицов", "гипсокартон", "тухл", "плоч", "паркет", "ламинат",
        "хидроизолаци", "топлоизолаци", "стоман", "метал", "врат", "прозор",
        "тенекеджий", "дърводел", "стъклар", "демонтаж", "разрушаване",
        "цена", "цени", "ремонт", "строител",
    ];
    keywords.iter().any(|kw| lower.contains(kw))
}

fn extract_description_from_text(text: &str) -> String {
    // Everything before the first digit that's part of a price
    let mut desc_end = text.len();
    for (i, c) in text.char_indices() {
        if c.is_ascii_digit() {
            // Check if this digit is part of a price (followed by more digits, then currency)
            let rest = &text[i..];
            if price_utils::parse_price_text(rest).is_some() {
                desc_end = i;
                break;
            }
        }
    }

    let desc = text[..desc_end].trim();
    price_utils::clean_description(desc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_price_indicator() {
        assert!(has_price_indicator("12 €/м2"));
        assert!(has_price_indicator("40.80 €"));
        assert!(!has_price_indicator("just text"));
    }

    #[test]
    fn test_has_construction_keyword() {
        assert!(has_construction_keyword("Цени на зидария"));
        assert!(has_construction_keyword("Мазилка и шпакловка"));
        assert!(!has_construction_keyword("Random heading"));
    }

    #[test]
    fn test_extract_description() {
        assert_eq!(
            extract_description_from_text("Тухлена зидария 40.80 - 61.20 €"),
            "Тухлена зидария"
        );
    }
}
