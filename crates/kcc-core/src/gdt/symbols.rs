use crate::geometry::model::{GdtSymbol, MaterialCondition};

/// Map GDT font character code to symbol.
/// Based on the AutoCAD GDT font mapping ({\Fgdt;X} format).
pub fn gdt_font_char_to_symbol(c: char) -> Option<GdtSymbol> {
    match c {
        'b' => Some(GdtSymbol::Flatness),
        'u' => Some(GdtSymbol::Straightness),
        'e' => Some(GdtSymbol::Circularity),
        'g' => Some(GdtSymbol::Cylindricity),
        'f' => Some(GdtSymbol::Parallelism),
        'n' => Some(GdtSymbol::Perpendicularity),
        'a' => Some(GdtSymbol::Angularity),
        'j' => Some(GdtSymbol::Position),
        'r' => Some(GdtSymbol::Concentricity),
        'i' => Some(GdtSymbol::Symmetry),
        'h' => Some(GdtSymbol::RunoutCircular),
        't' => Some(GdtSymbol::RunoutTotal),
        'k' => Some(GdtSymbol::ProfileLine),
        'd' => Some(GdtSymbol::ProfileSurface),
        _ => None,
    }
}

/// Map Unicode character to GD&T symbol.
pub fn unicode_to_symbol(c: char) -> Option<GdtSymbol> {
    match c {
        '\u{23E5}' => Some(GdtSymbol::Flatness),         // ⏥
        '\u{23E4}' => Some(GdtSymbol::Straightness),     // ⏤
        '\u{25EF}' => Some(GdtSymbol::Circularity),      // ◯
        '\u{232D}' => Some(GdtSymbol::Cylindricity),     // ⌭
        '\u{2225}' => Some(GdtSymbol::Parallelism),      // ∥
        '\u{27C2}' => Some(GdtSymbol::Perpendicularity), // ⟂
        '\u{2220}' => Some(GdtSymbol::Angularity),       // ∠
        '\u{2316}' => Some(GdtSymbol::Position),         // ⌖
        '\u{25CE}' => Some(GdtSymbol::Concentricity),    // ◎
        '\u{232F}' => Some(GdtSymbol::Symmetry),
        '\u{2197}' => Some(GdtSymbol::RunoutCircular), // ↗ (approximation)
        '\u{2330}' => Some(GdtSymbol::RunoutTotal),
        '\u{2312}' => Some(GdtSymbol::ProfileLine), // ⌒
        '\u{2313}' => Some(GdtSymbol::ProfileSurface),
        _ => None,
    }
}

/// Parse a single-character modifier code from TOLERANCE entity text.
/// Used in cells where the modifier follows the tolerance value or datum letter.
pub fn parse_modifier_code(c: char) -> Option<MaterialCondition> {
    match c {
        'm' | 'M' => Some(MaterialCondition::MaximumMaterial),
        'l' | 'L' => Some(MaterialCondition::LeastMaterial),
        's' | 'S' => Some(MaterialCondition::RegardlessOfFeature),
        _ => None,
    }
}

/// Check if a character is the projected tolerance modifier.
pub fn is_projected_modifier(c: char) -> bool {
    c == 'p' || c == 'P'
}

/// Try to identify a GD&T symbol from text (font code or Unicode).
pub fn identify_symbol(text: &str) -> Option<GdtSymbol> {
    // Check for font-based encoding: {\Fgdt;X} or {\fGDT;X}
    if let Some(idx) = text.find("gdt;").or_else(|| text.find("GDT;")) {
        let char_idx = idx + 4;
        if let Some(c) = text.chars().nth(char_idx) {
            if let Some(sym) = gdt_font_char_to_symbol(c) {
                return Some(sym);
            }
        }
    }

    // Check Unicode characters
    for c in text.chars() {
        if let Some(sym) = unicode_to_symbol(c) {
            return Some(sym);
        }
    }

    // Check text keywords (fallback)
    let lower = text.to_lowercase();
    if lower.contains("position") {
        return Some(GdtSymbol::Position);
    }
    if lower.contains("flatness") {
        return Some(GdtSymbol::Flatness);
    }
    if lower.contains("perpendicular") {
        return Some(GdtSymbol::Perpendicularity);
    }
    if lower.contains("parallel") {
        return Some(GdtSymbol::Parallelism);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_char_mapping() {
        assert_eq!(gdt_font_char_to_symbol('j'), Some(GdtSymbol::Position));
        assert_eq!(gdt_font_char_to_symbol('b'), Some(GdtSymbol::Flatness));
        assert_eq!(gdt_font_char_to_symbol('z'), None);
    }

    #[test]
    fn test_identify_from_font() {
        let text = r"{\fGDT;j}";
        assert_eq!(identify_symbol(text), Some(GdtSymbol::Position));
    }

    #[test]
    fn test_parse_modifier_code() {
        assert_eq!(
            parse_modifier_code('m'),
            Some(MaterialCondition::MaximumMaterial)
        );
        assert_eq!(
            parse_modifier_code('l'),
            Some(MaterialCondition::LeastMaterial)
        );
        assert_eq!(
            parse_modifier_code('s'),
            Some(MaterialCondition::RegardlessOfFeature)
        );
        assert_eq!(parse_modifier_code('x'), None);
    }

    #[test]
    fn test_is_projected_modifier() {
        assert!(is_projected_modifier('p'));
        assert!(is_projected_modifier('P'));
        assert!(!is_projected_modifier('m'));
    }
}
