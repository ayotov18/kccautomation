//! Shared price parsing utilities for Bulgarian construction pricing.
//!
//! Canonical currency: лв (BGN). EUR is derived.

/// Parsed price result from text.
#[derive(Debug, Clone)]
pub struct ParsedPrice {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub currency: PriceCurrency,
    pub raw_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PriceCurrency {
    Lv,
    Eur,
    Unknown,
}

/// Parse a price range from Bulgarian construction text.
///
/// Supports:
/// - `12 - 18 лв`  /  `12–18 лв`  /  `12—18 лв`
/// - `от 12 лв до 18 лв`  /  `от 12 до 18`
/// - `12 лв`  (single price)
/// - `12,50 лв`  /  `12.50 лв`
/// - `12 лв/м2`  /  `12,50 лв / м2`
/// - Same patterns with `€` / `EUR`
pub fn parse_price_text(text: &str) -> Option<ParsedPrice> {
    let raw = text.trim().to_string();
    if raw.is_empty() {
        return None;
    }

    let currency = detect_currency(&raw);

    // Normalize Latin/Cyrillic mixed "от"/"до"
    let normalized = raw
        .replace("Oт", "от")
        .replace("OT", "от")
        .replace("Дo", "до");
    let lower = normalized.to_lowercase();

    // Try "от X до Y" pattern
    if let Some(result) = try_ot_do_pattern(&lower, currency, &raw) {
        return Some(result);
    }

    // Try "X - Y" pattern (with currency marker required)
    if let Some(result) = try_dash_range_pattern(&lower, currency, &raw) {
        return Some(result);
    }

    // Try single value
    if let Some(result) = try_single_price(&lower, currency, &raw) {
        return Some(result);
    }

    None
}

fn try_ot_do_pattern(lower: &str, currency: PriceCurrency, raw: &str) -> Option<ParsedPrice> {
    let from_val = extract_number_after(lower, "от");
    let to_val = extract_number_after(lower, "до");

    match (from_val, to_val) {
        (Some(min), Some(max)) if min > 0.0 && max > 0.0 && max >= min => Some(ParsedPrice {
            min: Some(min),
            max: Some(max),
            currency,
            raw_text: raw.to_string(),
        }),
        (Some(val), None) => Some(ParsedPrice {
            min: Some(val),
            max: Some(val),
            currency,
            raw_text: raw.to_string(),
        }),
        (None, Some(val)) => Some(ParsedPrice {
            min: Some(val),
            max: Some(val),
            currency,
            raw_text: raw.to_string(),
        }),
        _ => None,
    }
}

fn try_dash_range_pattern(lower: &str, currency: PriceCurrency, raw: &str) -> Option<ParsedPrice> {
    // Only accept dash range if currency is detected (prevents JS garbage)
    if currency == PriceCurrency::Unknown {
        return None;
    }

    let parts: Vec<&str> = lower.split(|c: char| c == '-' || c == '–' || c == '—').collect();
    if parts.len() == 2 {
        let min = extract_number(parts[0]);
        let max = extract_number(parts[1]);
        if let (Some(min), Some(max)) = (min, max) {
            if min > 0.0 && max > 0.0 && max >= min {
                return Some(ParsedPrice {
                    min: Some(min),
                    max: Some(max),
                    currency,
                    raw_text: raw.to_string(),
                });
            }
        }
    }
    None
}

fn try_single_price(lower: &str, currency: PriceCurrency, raw: &str) -> Option<ParsedPrice> {
    if currency == PriceCurrency::Unknown {
        return None;
    }
    let val = extract_number(lower)?;
    if val > 0.0 {
        Some(ParsedPrice {
            min: Some(val),
            max: Some(val),
            currency,
            raw_text: raw.to_string(),
        })
    } else {
        None
    }
}

/// Detect currency from text.
pub fn detect_currency(text: &str) -> PriceCurrency {
    let lower = text.to_lowercase();
    if lower.contains("лв") || lower.contains("лев") || lower.contains("bgn") {
        PriceCurrency::Lv
    } else if lower.contains('€') || lower.contains("eur") {
        PriceCurrency::Eur
    } else {
        PriceCurrency::Unknown
    }
}

/// Extract the first number appearing after a keyword.
pub fn extract_number_after(text: &str, keyword: &str) -> Option<f64> {
    let pos = text.find(keyword)?;
    let after = &text[pos + keyword.len()..];
    extract_number(after)
}

/// Extract the first decimal number from a string.
/// Handles both `.` and `,` as decimal separators.
pub fn extract_number(text: &str) -> Option<f64> {
    let mut num_str = String::new();
    let mut found_digit = false;
    for c in text.chars() {
        if c.is_ascii_digit() {
            num_str.push(c);
            found_digit = true;
        } else if c == '.' && found_digit && !num_str.contains('.') {
            num_str.push(c);
        } else if c == ',' && found_digit && !num_str.contains('.') {
            num_str.push('.'); // European decimal comma
        } else if found_digit {
            break;
        }
    }
    num_str.parse().ok()
}

/// Extract unit from text containing unit markers.
pub fn extract_unit(text: &str) -> Option<String> {
    let lower = text.to_lowercase();

    // Check for explicit unit patterns (most specific first)
    let patterns = [
        (&["м²", "м2", "m2", "кв.м", "кв.м.", "/м2", "/m2"][..], "М2"),
        (&["м³", "м3", "m3", "куб.м", "куб.м.", "/м3", "/m3"][..], "М3"),
        (&["бр.", "бр", "/бр"][..], "бр."),
        (&["кг", "kg", "/кг", "/kg"][..], "кг"),
        (&["тон", "/тон"][..], "тон"),
        (&["компл", "/компл"][..], "компл."),
        (&["час", "/час"][..], "час"),
        (&["ден", "/ден"][..], "ден"),
        // Linear meter must be last (substring of м2/м3)
        (&["м.л.", "л.м.", "/м", "/m"][..], "м"),
    ];

    for (markers, unit) in &patterns {
        for marker in *markers {
            if lower.contains(marker) {
                return Some(unit.to_string());
            }
        }
    }

    // Check for standalone "м" but NOT if "м2" or "м3" is present
    if (lower.contains("м") || lower.contains("m")) && !lower.contains("м2") && !lower.contains("м3") && !lower.contains("m2") && !lower.contains("m3") {
        // Only if it's clearly a linear meter context
        if lower.contains("/м") || lower.contains("/m") {
            return Some("м".to_string());
        }
    }

    None
}

/// Normalize a unit string to canonical form.
pub fn normalize_unit(text: &str) -> String {
    let lower = text.to_lowercase().trim().to_string();
    match lower.as_str() {
        "м2" | "м²" | "m2" | "кв.м" | "кв.м." => "М2".to_string(),
        "м3" | "м³" | "m3" | "куб.м" | "куб.м." => "М3".to_string(),
        "м" | "m" | "м.л." | "л.м." => "м".to_string(),
        "бр." | "бр" | "pcs" => "бр.".to_string(),
        "кг" | "kg" => "кг".to_string(),
        "тон" | "т" => "тон".to_string(),
        "компл." | "компл" => "компл.".to_string(),
        "час" => "час".to_string(),
        "ден" => "ден".to_string(),
        _ => text.trim().to_string(),
    }
}

/// Clean a description string: strip price/unit noise, normalize whitespace.
pub fn clean_description(desc: &str) -> String {
    let cleaned = desc
        .trim()
        // Strip common Bulgarian price page prefixes
        .trim_start_matches("Цени на ")
        .trim_start_matches("Цена на ")
        .trim_start_matches("Цени за ")
        .trim_start_matches("Цена за ")
        .trim_start_matches("Цена, ")
        .trim_start_matches("Цена ")
        // Strip trailing price/unit text that leaked into description
        .trim_end_matches("лв.")
        .trim_end_matches("лв")
        .trim_end_matches("лева")
        .trim();

    // Collapse multiple whitespace
    let collapsed: String = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

    // Capitalize first letter
    let mut chars = collapsed.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Check if a description is valid (not garbage).
pub fn is_valid_description(desc: &str) -> bool {
    if desc.len() < 3 || desc.len() > 300 {
        return false;
    }

    let garbage_markers = [
        "document.", "getelementbyid", "math.", "function", "return ",
        "margin:", "padding:", "calc(", ".search", ".srwrp", "display:",
        "var(", "window.", "console.", "addeventlistener", "{", "}", "=>",
        "queryselector", "innerhtml", "classname", "onclick", "style=",
        "javascript:", "http://", "https://", "www.", "<script", "<div",
    ];

    let lower = desc.to_lowercase();
    for marker in &garbage_markers {
        if lower.contains(marker) {
            return false;
        }
    }

    // Accept if: has Cyrillic, or is a known construction term, or is long enough to be a real product name
    let has_cyrillic = desc.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
    let is_known_latin = [
        "eps", "xps", "pvc", "ytong", "knauf", "rigips", "weber", "ceresit",
        "baumit", "henkel", "hilti", "bosch", "makita", "dewalt", "sika",
        "mapei", "tytan", "isover", "rockwool", "styrofoam", "cem", "osb",
    ]
    .iter()
    .any(|term| lower.contains(term));

    // Allow 5+ char descriptions without Cyrillic — real product names can be Latin
    let is_long_enough = desc.len() >= 5;

    has_cyrillic || is_known_latin || is_long_enough
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_price_range_lv() {
        let p = parse_price_text("12 - 18 лв").unwrap();
        assert_eq!(p.min, Some(12.0));
        assert_eq!(p.max, Some(18.0));
        assert_eq!(p.currency, PriceCurrency::Lv);
    }

    #[test]
    fn test_parse_price_ot_do() {
        let p = parse_price_text("от 5.10 до 10.20 лв/м2").unwrap();
        assert_eq!(p.min, Some(5.10));
        assert_eq!(p.max, Some(10.20));
        assert_eq!(p.currency, PriceCurrency::Lv);
    }

    #[test]
    fn test_parse_price_eur() {
        let p = parse_price_text("Oт 40.80 €/м2 До 61.20 €/м2").unwrap();
        assert_eq!(p.min, Some(40.80));
        assert_eq!(p.max, Some(61.20));
        assert_eq!(p.currency, PriceCurrency::Eur);
    }

    #[test]
    fn test_parse_single_price_lv() {
        let p = parse_price_text("12,50 лв").unwrap();
        assert_eq!(p.min, Some(12.50));
        assert_eq!(p.max, Some(12.50));
    }

    #[test]
    fn test_reject_no_currency() {
        assert!(parse_price_text("0.85 - 1.15").is_none());
    }

    #[test]
    fn test_extract_unit() {
        assert_eq!(extract_unit("12 лв/м2"), Some("М2".to_string()));
        assert_eq!(extract_unit("100 лв/бр"), Some("бр.".to_string()));
        assert_eq!(extract_unit("50 лв/кг"), Some("кг".to_string()));
    }

    #[test]
    fn test_clean_description() {
        assert_eq!(clean_description("Цени на тухлена зидария"), "Тухлена зидария");
        assert_eq!(clean_description("  Бетон   работи  "), "Бетон работи");
    }

    #[test]
    fn test_valid_description() {
        assert!(is_valid_description("Тухлена зидария"));
        assert!(is_valid_description("EPS топлоизолация"));
        assert!(!is_valid_description("document.getElementById('x')"));
        assert!(!is_valid_description(".cx"));
        assert!(!is_valid_description("https://example.com"));
    }
}
