//! Static lookup table: СЕК code → Образец 9.1 cross-reference.
//!
//! Tender committees reference line items by their position in the canonical
//! Образец 9.1 template (e.g. "XIII-т.63"). We emit this as a second code on
//! every line so the contract administrator can verify against the master list.
//!
//! Coverage is deliberately partial — we cover the most common items encountered
//! in the 21-file corpus. Unmapped items get a NULL obrazec_ref and are noted
//! in the audit trail.

use std::collections::HashMap;
use std::sync::OnceLock;

static CATALOG: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

/// SEK code → Образец 9.1 reference.
/// Format of value: `<Roman>-т.<int>` (position within the section).
fn catalog() -> &'static HashMap<&'static str, &'static str> {
    CATALOG.get_or_init(|| {
        let mut m = HashMap::new();
        // I Земни работи
        m.insert("СЕК01.001", "I-т.1");
        m.insert("СЕК01.002", "I-т.2");
        m.insert("СЕК01.003", "I-т.3");
        m.insert("СЕК01.009", "I-т.9");
        m.insert("СЕК01.010", "I-т.10");
        // II Кофражни
        m.insert("СЕК02.001", "II-т.1");
        m.insert("СЕК02.002", "II-т.2");
        // III Армировъчни
        m.insert("СЕК03.001", "III-т.1");
        m.insert("СЕК03.002", "III-т.2");
        m.insert("СЕК03.003", "III-т.3");
        m.insert("СЕК03.004", "III-т.4");
        // IV Бетонови
        m.insert("СЕК04.001", "IV-т.1");
        m.insert("СЕК04.007", "IV-т.7");
        m.insert("СЕК04.010", "IV-т.10");
        m.insert("СЕК04.011", "IV-т.11");
        // V Зидарски
        m.insert("СЕК05.007", "V-т.7");
        m.insert("СЕК05.009", "V-т.9");
        m.insert("СЕК05.013", "V-т.13");
        m.insert("СЕК05.014", "V-т.14");
        // VI Покривни
        m.insert("СЕК06.001", "VI-т.1");
        m.insert("СЕК06.004", "VI-т.4");
        // VII Тенекеджийски
        m.insert("СЕК07.001", "VII-т.1");
        m.insert("СЕК07.003", "VII-т.3");
        // IX Облицовъчни
        m.insert("СЕК09.003", "IX-т.3");
        m.insert("СЕК09.004", "IX-т.4");
        // X Мазачески
        m.insert("СЕК10.001", "X-т.1");
        m.insert("СЕК10.006", "X-т.6");
        m.insert("СЕК10.020", "X-т.19");
        // XI Настилки
        m.insert("СЕК11.008", "XI-т.8");
        m.insert("СЕК11.013", "XI-т.13");
        m.insert("СЕК11.025", "XI-т.25");
        // XIII Бояджийски
        m.insert("СЕК13.007", "XIII-т.7");
        m.insert("СЕК13.009", "XIII-т.9");
        // XIV Метална дограма
        m.insert("СЕК14.003", "XIV-т.3");
        // XV Хидроизолации
        m.insert("СЕК15.004", "XV-т.4");
        m.insert("СЕК15.006", "XV-т.6");
        // XVI Топлоизолации
        m.insert("СЕК16.001", "XVI-т.1");
        m.insert("СЕК16.013", "XVI-т.13");
        // XVII Столарски
        m.insert("СЕК17.029", "XVII-т.29");
        m.insert("СЕК17.030", "XVII-т.30");
        // XVIII Сухо строителство
        m.insert("СЕК20.001", "XVIII-т.1");
        m.insert("СЕК20.006", "XVIII-т.6");
        // XIX Сградни ВиК
        m.insert("СЕК22.001", "XIX-т.1");
        m.insert("СЕК22.031", "XIX-т.31");
        // XXI Електрическа
        m.insert("СЕК34.111", "XXI-1.1.1");
        m.insert("СЕК34.311", "XXI-3.1");
        m.insert("СЕК34.411", "XXI-4.1");
        m
    })
}

/// Exact or prefix match. Returns None if no mapping.
pub fn lookup_obrazec_ref(sek_code: &str) -> Option<&'static str> {
    if let Some(v) = catalog().get(sek_code) {
        return Some(*v);
    }
    // Also try stripping trailing digits — СЕК05.817 renovation repair maps
    // to the same Образец 9.1 position as СЕК05.017 new-build.
    if let Some(dot) = sek_code.find('.') {
        let group = &sek_code[..dot];
        let num_str = &sek_code[dot + 1..];
        if let Ok(num) = num_str.parse::<u32>() {
            let base_num = if num >= 800 { num - 800 } else { num };
            let key = format!("{}.{:03}", group, base_num);
            // Lookup with a leaked static — only happens once per item.
            let leaked: &'static str = Box::leak(key.into_boxed_str());
            if let Some(v) = catalog().get(leaked) {
                return Some(*v);
            }
        }
    }
    None
}

/// Classify a SEK code as renovation/repair (800+ sub-number).
pub fn is_renovation_code(sek_code: &str) -> bool {
    if let Some(dot) = sek_code.find('.') {
        if let Ok(num) = sek_code[dot + 1..].parse::<u32>() {
            return num >= 800;
        }
    }
    false
}

/// EWC (European Waste Classification) code for demolition/renovation waste.
/// Based on the Bulgarian Наредба № 2 transposition of Directive 2008/98/EC.
pub fn ewc_code_for_sek(sek_group: &str) -> Option<&'static str> {
    match sek_group {
        "СЕК04" => Some("17 01 01"), // concrete
        "СЕК05" => Some("17 01 02"), // bricks
        "СЕК09" => Some("17 01 03"), // tiles and ceramics
        "СЕК06" | "СЕК08" | "СЕК17" => Some("17 02 01"), // wood
        "СЕК12" => Some("17 02 02"), // glass
        "СЕК13" | "СЕК15" | "СЕК16" => Some("17 02 03"), // plastic / paint / insulation
        "СЕК14" => Some("17 04 05"), // iron / steel
        "СЕК34" => Some("17 04 11"), // cables
        "СЕК22" | "СЕК23" => Some("17 04 07"), // mixed metals (pipes)
        "СЕК49" => Some("17 09 04"), // mixed construction & demolition
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_lookup() {
        assert_eq!(lookup_obrazec_ref("СЕК05.007"), Some("V-т.7"));
    }

    #[test]
    fn test_renovation_mapping() {
        // 800+ sub-number should map to the base position
        assert_eq!(lookup_obrazec_ref("СЕК05.807"), Some("V-т.7"));
    }

    #[test]
    fn test_is_renovation() {
        assert!(is_renovation_code("СЕК05.801"));
        assert!(!is_renovation_code("СЕК05.001"));
    }

    #[test]
    fn test_ewc_for_demolition() {
        assert_eq!(ewc_code_for_sek("СЕК04"), Some("17 01 01"));
        assert_eq!(ewc_code_for_sek("СЕК49"), Some("17 09 04"));
    }
}
