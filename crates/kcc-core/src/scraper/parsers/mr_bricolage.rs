//! Parser for mr-bricolage.bg — Bulgarian building materials retailer.
//!
//! 3-layer fallback extraction:
//!   A) JSON-LD structured data
//!   B) Embedded JS product state
//!   C) Product card DOM extraction
//!
//! Mr.Bricolage shows prices in € (EUR). Canonical storage.

use scraper::{Html, Selector};

use super::{CategoryUrl, ParseResult, PriceParser};
use crate::scraper::ScrapedPrice;
use crate::scraper::price_utils;

pub struct MrBricolageParser;

impl PriceParser for MrBricolageParser {
    fn site_name(&self) -> &str {
        "mr-bricolage.bg"
    }

    fn parse_page(&self, html: &str, url: &str) -> ParseResult {
        let doc = Html::parse_document(html);
        let mut diagnostics: Vec<(&'static str, usize)> = Vec::new();

        // Strategy A: JSON-LD structured data
        let (mut prices, jsonld_count) = try_jsonld_parse(&doc, url);
        diagnostics.push(("json_ld", jsonld_count));
        if !prices.is_empty() {
            let before = prices.len();
            prices.retain(|p| price_utils::is_valid_description(&p.description_bg) || p.description_bg.len() > 3);
            return ParseResult {
                candidates_before_filter: before,
                candidates_after_filter: prices.len(),
                strategy_used: "json_ld",
                diagnostics,
                prices,
            };
        }

        // Strategy B: Embedded JS product data
        let (mut prices, js_count) = try_embedded_js_parse(html, url);
        diagnostics.push(("embedded_json", js_count));
        if !prices.is_empty() {
            let before = prices.len();
            prices.retain(|p| p.description_bg.len() > 3);
            return ParseResult {
                candidates_before_filter: before,
                candidates_after_filter: prices.len(),
                strategy_used: "embedded_json",
                diagnostics,
                prices,
            };
        }

        // Strategy C: Product card DOM
        let (mut prices, card_count) = try_product_card_parse(&doc, url);
        diagnostics.push(("product_cards", card_count));
        if !prices.is_empty() {
            let before = prices.len();
            prices.retain(|p| p.description_bg.len() > 3);
            return ParseResult {
                candidates_before_filter: before,
                candidates_after_filter: prices.len(),
                strategy_used: "product_cards",
                diagnostics,
                prices,
            };
        }

        ParseResult {
            prices: Vec::new(),
            strategy_used: "none",
            candidates_before_filter: 0,
            candidates_after_filter: 0,
            diagnostics,
        }
    }

    fn category_urls(&self) -> Vec<CategoryUrl> {
        vec![
            cat("https://mr-bricolage.bg/stroitelstvo/stroitelni-materiali/c/011004", "materials", "Строителни материали"),
            cat("https://mr-bricolage.bg/stroitelstvo/izolacii/c/011003", "СЕК15", "Изолации"),
            cat("https://mr-bricolage.bg/stroitelstvo/smesi-i-lepila/c/011001", "СЕК10", "Смеси и лепила"),
            cat("https://mr-bricolage.bg/stroitelstvo/boi-i-lakove/c/011002", "СЕК13", "Бои и лакове"),
            cat("https://mr-bricolage.bg/stroitelstvo/stroitelni-prinadlezhnosti/c/011003", "materials", "Стр. принадлежности"),
            cat("https://mr-bricolage.bg/vodosnabdyavane-i-otoplenie/c/007", "СЕК22", "ВиК и отопление"),
            cat("https://mr-bricolage.bg/elektrichestvo/c/005", "СЕК34", "Електричество"),
            cat("https://mr-bricolage.bg/podovi-nastilki/c/009", "СЕК11", "Подови настилки"),
            cat("https://mr-bricolage.bg/stroitelstvo/c/011", "materials", "Строителство общо"),
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

// ── Strategy A: JSON-LD ─────────────────────────────────────

fn try_jsonld_parse(doc: &Html, url: &str) -> (Vec<ScrapedPrice>, usize) {
    let script_sel = match Selector::parse("script[type='application/ld+json']") {
        Ok(s) => s,
        Err(_) => return (Vec::new(), 0),
    };

    let mut prices = Vec::new();
    let mut count = 0;

    for script in doc.select(&script_sel) {
        let text = script.text().collect::<Vec<_>>().join("");
        count += 1;

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            extract_jsonld_products(&json, url, &mut prices);
        }
    }

    (prices, count)
}

fn extract_jsonld_products(json: &serde_json::Value, url: &str, prices: &mut Vec<ScrapedPrice>) {
    match json {
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_jsonld_products(item, url, prices);
            }
        }
        serde_json::Value::Object(obj) => {
            let type_val = obj.get("@type").and_then(|v| v.as_str()).unwrap_or("");

            if type_val == "Product" {
                let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if name.is_empty() { return; }

                // Extract price from offers
                if let Some(offers) = obj.get("offers") {
                    let price = extract_offer_price(offers);
                    let currency = offers.get("priceCurrency")
                        .or_else(|| offers.get("currency"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("EUR");

                    if let Some(p) = price {
                        let is_eur = currency == "EUR" || currency == "€" || currency == "lv";
                        let sp = if is_eur {
                            ScrapedPrice::from_eur("mr-bricolage.bg", url, name, "бр.", Some(p), Some(p), None, None, 1.0)
                        } else {
                            ScrapedPrice::from_eur("mr-bricolage.bg", url, name, "бр.", Some(p), Some(p), None, None, 1.0)
                        };
                        prices.push(sp);
                    }
                }
            }

            // Recurse into ItemList
            if type_val == "ItemList" {
                if let Some(elements) = obj.get("itemListElement") {
                    extract_jsonld_products(elements, url, prices);
                }
            }

            // Recurse into ListItem
            if type_val == "ListItem" {
                if let Some(item) = obj.get("item") {
                    extract_jsonld_products(item, url, prices);
                }
            }
        }
        _ => {}
    }
}

fn extract_offer_price(offers: &serde_json::Value) -> Option<f64> {
    // Single offer
    if let Some(p) = offers.get("price") {
        return p.as_f64().or_else(|| p.as_str().and_then(|s| s.replace(',', ".").parse().ok()));
    }
    if let Some(p) = offers.get("lowPrice") {
        return p.as_f64().or_else(|| p.as_str().and_then(|s| s.replace(',', ".").parse().ok()));
    }
    // Array of offers
    if let Some(arr) = offers.as_array() {
        for offer in arr {
            if let Some(p) = offer.get("price") {
                return p.as_f64().or_else(|| p.as_str().and_then(|s| s.replace(',', ".").parse().ok()));
            }
        }
    }
    None
}

// ── Strategy B: Embedded JS product state ───────────────────

fn try_embedded_js_parse(html: &str, url: &str) -> (Vec<ScrapedPrice>, usize) {
    let mut prices = Vec::new();
    let mut json_found = 0;

    // Search for JSON arrays/objects in script tags containing product data
    // Common patterns: window.__data, productListData, catalogData, etc.
    let search_patterns = [
        "\"products\":", "\"items\":", "\"productList\":",
        "\"catalogEntries\":", "\"searchResults\":",
        "productData", "productList",
    ];

    let doc = Html::parse_document(html);
    let script_sel = match Selector::parse("script") {
        Ok(s) => s,
        Err(_) => return (Vec::new(), 0),
    };

    for script in doc.select(&script_sel) {
        let text = script.text().collect::<Vec<_>>().join("");
        if text.len() < 50 { continue; }

        let has_product_data = search_patterns.iter().any(|p| text.contains(p));
        if !has_product_data { continue; }

        json_found += 1;

        // Try to find JSON arrays within the script
        // Look for [...] patterns that might contain product objects
        for start in find_json_starts(&text) {
            let substr = &text[start..];
            if let Some(json_str) = extract_balanced_json(substr) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                    extract_products_from_json(&val, url, &mut prices);
                }
            }
        }
    }

    (prices, json_found)
}

fn find_json_starts(text: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    for (i, c) in text.char_indices() {
        if c == '[' || c == '{' {
            starts.push(i);
        }
        if starts.len() > 200 { break; } // scan more JSON candidates
    }
    starts
}

fn extract_balanced_json(text: &str) -> Option<&str> {
    let first = text.chars().next()?;
    let (open, close) = match first {
        '[' => ('[', ']'),
        '{' => ('{', '}'),
        _ => return None,
    };

    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, c) in text.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if c == '\\' && in_string {
            escape_next = true;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string { continue; }

        if c == open { depth += 1; }
        if c == close {
            depth -= 1;
            if depth == 0 {
                return Some(&text[..=i]);
            }
        }

        if i > 50000 { break; } // safety limit
    }
    None
}

fn extract_products_from_json(val: &serde_json::Value, url: &str, prices: &mut Vec<ScrapedPrice>) {
    match val {
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_products_from_json(item, url, prices);
            }
        }
        serde_json::Value::Object(obj) => {
            // Check if this object looks like a product
            let name = obj.get("name").or_else(|| obj.get("productName"))
                .or_else(|| obj.get("title"))
                .and_then(|v| v.as_str());
            let price = obj.get("price").or_else(|| obj.get("currentPrice"))
                .or_else(|| obj.get("salePrice"))
                .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.replace(',', ".").parse().ok())));

            if let (Some(name), Some(price)) = (name, price) {
                if name.len() > 3 && price > 0.0 {
                    prices.push(ScrapedPrice::from_eur(
                        "mr-bricolage.bg", url, name, "бр.",
                        Some(price), Some(price),
                        None, None, 0.8,
                    ));
                    // DO NOT return — continue recursing to find more products
                }
            }

            // Always recurse into nested values that might contain products
            for key in ["products", "items", "results", "entries", "data", "content", "productList"] {
                if let Some(nested) = obj.get(key) {
                    extract_products_from_json(nested, url, prices);
                }
            }
        }
        _ => {}
    }
}

// ── Strategy C: Product card DOM ────────────────────────────

fn try_product_card_parse(doc: &Html, url: &str) -> (Vec<ScrapedPrice>, usize) {
    // Try progressively broader selectors
    let card_selectors = [
        ".product-item",
        ".product-card",
        ".product-box",
        ".product-tile",
        ".product__item",
        ".product-grid-item",
        "[class*='product-list'] > *",
        "[class*='productList'] > *",
        "[data-product]",
        "[class*='product']",
    ];

    let name_selectors = [
        ".product-name a", ".product-name", ".product-title a", ".product-title",
        ".product__name a", ".product__name",
        "a.name", "h3 a", "h4 a", "h3", "h4",
        "[class*='name'] a", "[class*='name']",
        "[class*='title'] a", "[class*='title']",
    ];

    let price_selectors = [
        ".product-price .price", ".product-price", ".price",
        ".current-price", ".sale-price", ".product__price",
        "[class*='price']", "span[class*='price']",
    ];

    let mut best_prices = Vec::new();
    let mut total_cards = 0;

    for card_sel_str in &card_selectors {
        let card_sel = match Selector::parse(card_sel_str) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let cards: Vec<_> = doc.select(&card_sel).collect();
        if cards.is_empty() { continue; }

        let mut prices = Vec::new();
        total_cards += cards.len();

        for card in &cards {
            let name = try_extract_text(card, &name_selectors);
            let price_text = try_extract_text(card, &price_selectors);

            if name.is_empty() || price_text.is_empty() { continue; }

            let price_val = extract_bgn_price(&price_text);
            if let Some(p) = price_val {
                if p > 0.0 && name.len() > 3 {
                    prices.push(ScrapedPrice::from_eur(
                        "mr-bricolage.bg", url,
                        &name, "бр.",
                        Some(p), Some(p),
                        Some(&price_text), None,
                        0.6,
                    ));
                }
            }
        }

        if !prices.is_empty() {
            best_prices = prices;
            break; // use first selector that produces results
        }
    }

    (best_prices, total_cards)
}

fn try_extract_text(parent: &scraper::ElementRef, selectors: &[&str]) -> String {
    for sel_str in selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = parent.select(&sel).next() {
                let text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !text.is_empty() {
                    return text;
                }
            }
        }
    }
    String::new()
}

/// Extract a EUR price from text like "12.50 €.", "12,50 €", "12.50".
fn extract_bgn_price(text: &str) -> Option<f64> {
    let cleaned = text
        .replace("€.", "")
        .replace("€", "")
        .replace("EUR", "")
        .replace('\u{a0}', "")
        .replace(' ', "")
        .replace(',', ".")
        .trim()
        .to_string();

    // Try direct parse first
    if let Ok(val) = cleaned.parse::<f64>() {
        return Some(val);
    }

    // Fallback: extract first number
    price_utils::extract_number(&cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bgn_price() {
        assert_eq!(extract_bgn_price("12.50 €."), Some(12.50));
        assert_eq!(extract_bgn_price("12,50 €"), Some(12.50));
        assert_eq!(extract_bgn_price("3.82€."), Some(3.82));
        assert_eq!(extract_bgn_price("1\u{a0}234.56 €"), Some(1234.56));
    }

    #[test]
    fn test_jsonld_product_extraction() {
        let json = serde_json::json!({
            "@type": "Product",
            "name": "Цимент ЦЕМ I 42.5R 25кг",
            "offers": {
                "price": "8.99",
                "priceCurrency": "EUR"
            }
        });
        let mut prices = Vec::new();
        extract_jsonld_products(&json, "https://test.com", &mut prices);
        assert_eq!(prices.len(), 1);
        assert_eq!(prices[0].description_bg, "Цимент ЦЕМ I 42.5R 25кг");
        assert_eq!(prices[0].price_min_eur, Some(8.99));
        assert_eq!(prices[0].currency, "lv");
    }
}
