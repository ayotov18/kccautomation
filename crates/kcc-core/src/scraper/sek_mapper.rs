//! Maps scraped Bulgarian work descriptions to SEK codes.
//!
//! Uses keyword matching against a hardcoded table of Bulgarian construction terms.
//! Falls back to the `sek_code_mappings` DB table for user-added patterns.

use crate::scraper::ScrapedPrice;

/// Result of mapping a scraped price to a SEK code.
#[derive(Debug, Clone)]
pub struct MappedPrice {
    pub scraped: ScrapedPrice,
    pub sek_code: Option<String>,
    pub sek_group: String,
    pub confidence: f64,
}

/// Keyword rules: (lowercased stems in ANY language we've seen, SEK group,
/// default SEK code). Checked via substring-contains, so we cover BG + EN +
/// PT + DE + transliterated spellings that actually show up in the wild.
///
/// Kept deliberately permissive — keyword matches drive an `extraction_method`
/// tag, not a final quantity, so an occasional false-positive is harmless
/// (the row still goes through geometry + AI review). False-negatives are the
/// expensive ones, because they silently drop rows from the deterministic
/// lane.
static KEYWORD_RULES: &[(&[&str], &str, &str)] = &[
    // СЕК01 — Earthwork
    (&[
        "изкоп", "земни работи", "хумус", "засипване",
        "escavacao", "escavação", "aterro",
        "excavation", "earthwork", "backfill",
        "erdarbeit", "aushub",
    ], "СЕК01", "СЕК01.013"),

    // СЕК02 — Formwork
    (&[
        "кофраж",
        "cofragem", "forma",
        "formwork", "shutter",
        "schalung",
    ], "СЕК02", "СЕК02.010"),

    // СЕК03 — Reinforcement
    (&[
        "армировка", "арматур", "армиров",
        "arm1", "arm2", "arm ", "arm-", "strem",
        "armacao", "armação", "ferragem",
        "rebar", "reinforcement",
        "bewehrung",
    ], "СЕК03", "СЕК03.012"),

    // СЕК04 — Concrete
    (&[
        "бетон", "фундамент", "бетонов",
        "beton", "betão", "concreto",
        "concrete", "foundation",
    ], "СЕК04", "СЕК04.068"),

    // СЕК05 — Masonry (note: "блок" removed — too generic)
    (&[
        "зидария", "зидане", "тухл", "газобетон", "ytong", "блокчет",
        "steni-tuhla", "steni-beton", "steni-gazobeton", "steni ", "steni-",
        "alvenaria", "tijolo", "bloco",
        "masonry", "brick", "wall",
        "mauerwerk", "ziegel",
    ], "СЕК05", "СЕК05.002"),

    // СЕК06 — Roofing
    (&[
        "покрив", "керемид", "битумн",
        "pokriv", "pokrivi", "pokriv-",
        "telhado", "telha",
        "roof",
        "dach",
    ], "СЕК06", "СЕК06.010"),

    // СЕК07 — Gutters / tinsmith
    (&[
        "улук", "водосточ", "тенекеджий",
        "gutter", "downspout",
    ], "СЕК07", "СЕК07.010"),

    // СЕК08 — Carpentry
    (&[
        "дърводел", "дървен", "скеле",
        "darveni", "darvo",
        "madeira", "madeirame",
        "carpentry", "timber",
        "zimmerei",
    ], "СЕК08", "СЕК08.010"),

    // СЕК09 — Tiling / cladding
    (&[
        "облицовк", "фаянс", "теракот", "плоч", "гранитогрес",
        "ladrilho", "cerâmica", "azulejo",
        "tile", "ceramic",
        "fliese",
    ], "СЕК09", "СЕК09.010"),

    // СЕК10 — Plastering
    (&[
        "мазилка", "шпаклов", "мазаческ",
        "reboco", "emboco",
        "plaster", "render",
        "putz",
    ], "СЕК10", "СЕК10.011"),

    // СЕК11 — Flooring
    (&[
        "настилк", "паркет", "ламинат", "замазк", "мозайк",
        "hatch nastilki", "nastilka",
        "piso", "pavimento",
        "floor", "screed", "laminate",
        "estrich", "bodenbelag",
    ], "СЕК11", "СЕК11.020"),

    // СЕК12 — Glazing
    (&[
        "стъклар", "стъклопакет", "остъклявне",
        "vidro", "vidraça",
        "glaz", "glass", "glazing",
        "verglasung",
    ], "СЕК12", "СЕК12.010"),

    // СЕК13 — Painting
    (&[
        "боядис", "латекс", "грунд", "лакир", "бояджийск", "варосване",
        "pintura", "tinta",
        "paint",
        "anstrich",
    ], "СЕК13", "СЕК13.025"),

    // СЕК14 — Steel structures (substring-match on profile names: IPE / HEB / UPN / HEA)
    (&[
        "стоман", "метал", "заваряване", "конструкци",
        "kon", "koloni", "metal", "bolt", "befestigung", "stahl",
        "ipe", "heb", "hea", "upn", "ipn",
        "steel", "weld",
    ], "СЕК14", "СЕК14.010"),

    // СЕК15 — Waterproofing
    (&[
        "хидроизолаци", "битумн мембран", "водоплътн",
        "impermea",
        "waterproof",
        "abdichtung",
    ], "СЕК15", "СЕК15.010"),

    // СЕК16 — Thermal insulation
    (&[
        "топлоизолаци", "eps", "xps", "минерална вата", "стиропор",
        "isolat", "isolation", "isolamento",
        "thermal",
        "dämmung",
    ], "СЕК16", "СЕК16.020"),

    // СЕК17 — Joinery (doors / windows)
    (&[
        "дограм", "врат", "прозор", "pvc", "алуминиев",
        "dograma",
        "janela", "porta",
        "window", "door",
        "fenster", "tür",
    ], "СЕК17", "СЕК17.020"),

    // СЕК20 — Drywall
    (&[
        "гипсокартон", "сухо строит", "окачен таван",
        "gipskarton",
        "drywall", "plasterboard",
        "trockenbau",
    ], "СЕК20", "СЕК20.010"),

    // СЕК22 — Plumbing
    (&[
        "вик", "водопров", "канализаци", "мивк", "тоалетн", "вана", "душ", "сифон",
        "hidraulica", "hidráulica", "encanamento",
        "plumb", "sanit", "water-supply",
        "sanitär",
    ], "СЕК22", "СЕК22.050"),

    // СЕК34 — Electrical
    (&[
        "електрическ", "кабел", "контакт", "ключ", "осветл", "табло",
        "eletric", "elétric",
        "electric", "cable",
        "elektr",
    ], "СЕК34", "СЕК34.011"),

    // СЕК49 — Demolition
    (&[
        "разрушаване", "демонтаж", "разбиване", "разваляне",
        "demolicao", "demolição",
        "demolition",
        "abbruch",
    ], "СЕК49", "СЕК49.025"),
];

/// Map a scraped price to a SEK code using keyword matching.
pub fn map_to_sek(price: &ScrapedPrice, category_hint: Option<&str>) -> MappedPrice {
    let desc_lower = price.description_bg.to_lowercase();

    // Try keyword matching first
    for (keywords, sek_group, default_code) in KEYWORD_RULES {
        for kw in *keywords {
            if desc_lower.contains(kw) {
                return MappedPrice {
                    scraped: price.clone(),
                    sek_code: Some(default_code.to_string()),
                    sek_group: sek_group.to_string(),
                    confidence: 0.8,
                };
            }
        }
    }

    // Fall back to category hint from the URL
    if let Some(hint) = category_hint {
        if hint.starts_with("СЕК") {
            return MappedPrice {
                scraped: price.clone(),
                sek_code: None,
                sek_group: hint.to_string(),
                confidence: 0.5,
            };
        }
    }

    // Unmapped
    MappedPrice {
        scraped: price.clone(),
        sek_code: None,
        sek_group: "unknown".to_string(),
        confidence: 0.0,
    }
}

/// Map a batch of scraped prices, using the category hint for fallback.
pub fn map_batch(prices: &[ScrapedPrice], category_hint: Option<&str>) -> Vec<MappedPrice> {
    prices.iter().map(|p| map_to_sek(p, category_hint)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_price(desc: &str) -> ScrapedPrice {
        ScrapedPrice::from_eur(
            "test", "https://test.com", desc, "М2",
            Some(10.0), Some(20.0), None, None, 0.8,
        )
    }

    #[test]
    fn test_masonry_mapping() {
        let result = map_to_sek(&make_price("Тухлена зидария 29 см"), None);
        assert_eq!(result.sek_group, "СЕК05");
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_painting_mapping() {
        let result = map_to_sek(&make_price("Латексово боядисване 2 слоя"), None);
        assert_eq!(result.sek_group, "СЕК13");
    }

    #[test]
    fn test_plumbing_mapping() {
        let result = map_to_sek(&make_price("Монтаж на тоалетна моноблок"), None);
        assert_eq!(result.sek_group, "СЕК22");
    }

    #[test]
    fn test_unknown_with_hint() {
        let result = map_to_sek(&make_price("Some unknown item"), Some("СЕК10"));
        assert_eq!(result.sek_group, "СЕК10");
        assert_eq!(result.confidence, 0.5);
    }

    #[test]
    fn test_completely_unknown() {
        let result = map_to_sek(&make_price("xyz abc"), None);
        assert_eq!(result.sek_group, "unknown");
        assert_eq!(result.confidence, 0.0);
    }
}
