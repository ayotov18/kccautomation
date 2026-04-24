use super::symbols;
use crate::geometry::model::{
    DatumReference, EntityId, FeatureControlFrame, MaterialCondition, Point2D,
};

/// Parse MTEXT content containing GD&T information.
///
/// MTEXT-based GD&T uses Unicode symbols or `{\fGDT;X}` font codes inline in text.
/// This is the secondary source of GD&T data (after TOLERANCE entities).
pub fn parse_gdt_mtext(
    text: &str,
    position: Point2D,
    entity_id: EntityId,
) -> Vec<FeatureControlFrame> {
    let mut frames = Vec::new();

    // Try to parse as a TOLERANCE-style text code first (some MTEXT uses the same format)
    let tolerance_frames = parse_tolerance_text_code(text, position, entity_id);
    if !tolerance_frames.is_empty() {
        return tolerance_frames;
    }

    // Fallback: try legacy single-frame parsing for simple MTEXT GD&T
    if let Some(frame) = parse_single_frame(text, position, entity_id) {
        frames.push(frame);
    }

    frames
}

/// Parse a TOLERANCE entity's display_text field.
///
/// TOLERANCE entities use a structured text code format:
/// - Rows separated by `^J` (literal caret-J, not newline)
/// - Each row has cells separated by `%%v`
/// - Cell 1: GD&T symbol as `{\Fgdt;X}`
/// - Cell 2: Tolerance value, optional `%%c` diameter prefix, optional modifier suffix
/// - Cells 3-5: Datum references with optional modifier suffix
///
/// Example: `{\Fgdt;j}%%v%%c0.05%%vm%%vA%%vB`
///   = Position, diameter zone 0.05, MMC, datums A, B
///
/// Composite example: `{\Fgdt;j}%%v%%c0.05%%vm%%vA%%vB^J{\Fgdt;j}%%v0.02%%vm`
///   = Two rows: position 0.05 dia MMC ref A B, then position 0.02 MMC (no datums)
pub fn parse_tolerance_entity(
    raw: &str,
    position: Point2D,
    entity_id: EntityId,
) -> Vec<FeatureControlFrame> {
    let frames = parse_tolerance_text_code(raw, position, entity_id);
    if !frames.is_empty() {
        return frames;
    }

    // Fallback to legacy parser for non-standard formats
    if let Some(frame) = parse_single_frame(raw, position, entity_id) {
        return vec![frame];
    }

    Vec::new()
}

/// Parse the structured TOLERANCE text code format.
fn parse_tolerance_text_code(
    raw: &str,
    position: Point2D,
    entity_id: EntityId,
) -> Vec<FeatureControlFrame> {
    let mut frames = Vec::new();

    // Split on ^J for composite (multi-row) frames
    // ^J appears as literal characters '^' followed by 'J' in the DXF text
    let rows: Vec<&str> = raw.split("^J").collect();

    for row in rows {
        let row = row.trim();
        if row.is_empty() {
            continue;
        }

        if let Some(frame) = parse_tolerance_row(row, position, entity_id) {
            frames.push(frame);
        }
    }

    frames
}

/// Parse a single row of a TOLERANCE text code.
///
/// Format: `{symbol_cell}%%v{tolerance_cell}%%v{datum1}%%v{datum2}%%v{datum3}`
fn parse_tolerance_row(
    row: &str,
    position: Point2D,
    entity_id: EntityId,
) -> Option<FeatureControlFrame> {
    let cells: Vec<&str> = row.split("%%v").collect();
    if cells.is_empty() {
        return None;
    }

    // Cell 0: GD&T symbol
    let symbol = symbols::identify_symbol(cells[0])?;

    // Cell 1: Tolerance value with optional diameter prefix and modifier
    let (tolerance_value, material_condition, is_diameter_zone, projected_tolerance) =
        if cells.len() > 1 {
            parse_tolerance_cell(cells[1])
        } else {
            return None;
        };

    // Cells 2-4: Datum references
    let mut datum_refs = Vec::new();
    for cell in cells.iter().skip(2).take(3) {
        if let Some(datum_ref) = parse_datum_cell(cell) {
            datum_refs.push(datum_ref);
        }
    }

    Some(FeatureControlFrame {
        id: entity_id,
        symbol,
        tolerance_value,
        material_condition,
        datum_refs,
        position,
        attached_entities: Vec::new(),
        projected_tolerance,
        is_diameter_zone,
    })
}

/// Parse a tolerance value cell.
///
/// Examples:
/// - `0.05` → value 0.05, no modifier
/// - `%%c0.05` → value 0.05, diameter zone
/// - `0.05m` → value 0.05, MMC
/// - `%%c0.05m` → value 0.05, diameter zone, MMC
/// - `0.05p` → value 0.05, projected tolerance
fn parse_tolerance_cell(cell: &str) -> (f64, MaterialCondition, bool, bool) {
    let mut text = cell.trim();
    let mut is_diameter = false;
    let mut projected = false;
    let mut mc = MaterialCondition::None;

    // Check for diameter prefix %%c
    if let Some(rest) = text.strip_prefix("%%c").or_else(|| text.strip_prefix("%%C")) {
        is_diameter = true;
        text = rest;
    }
    // Also check for Unicode diameter symbol
    if let Some(rest) = text.strip_prefix('\u{2300}').or_else(|| text.strip_prefix('\u{00D8}')) {
        is_diameter = true;
        text = rest;
    }

    // Check for trailing modifier character
    if let Some(last) = text.chars().last() {
        if let Some(condition) = symbols::parse_modifier_code(last) {
            mc = condition;
            text = &text[..text.len() - last.len_utf8()];
        } else if symbols::is_projected_modifier(last) {
            projected = true;
            text = &text[..text.len() - last.len_utf8()];
        }
    }

    // Parse the numeric value
    let value = extract_number_from_str(text).unwrap_or(0.0);

    (value, mc, is_diameter, projected)
}

/// Parse a datum reference cell.
///
/// Examples:
/// - `A` → datum A, no modifier
/// - `Am` → datum A, MMC
/// - `Bl` → datum B, LMC
/// - `` (empty) → None
fn parse_datum_cell(cell: &str) -> Option<DatumReference> {
    let text = cell.trim();
    if text.is_empty() {
        return None;
    }

    let mut chars = text.chars();
    let label = chars.next()?;

    // Must be an uppercase letter A-Z
    if !label.is_ascii_uppercase() {
        return None;
    }

    // Check for trailing modifier
    let material_condition = if let Some(modifier_char) = chars.next() {
        symbols::parse_modifier_code(modifier_char).unwrap_or(MaterialCondition::None)
    } else {
        MaterialCondition::None
    };

    Some(DatumReference {
        label,
        material_condition,
    })
}

/// Extract a floating-point number from a string, skipping non-numeric prefix.
fn extract_number_from_str(text: &str) -> Option<f64> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    // Find the first digit or decimal point
    let start = text.find(|c: char| c.is_ascii_digit() || c == '.')?;
    let rest = &text[start..];

    // Take digits and decimal points
    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());

    rest[..end].parse::<f64>().ok()
}

/// Parse a single feature control frame from free-form text (legacy MTEXT format).
///
/// Handles patterns like: `{\fGDT;j} 0.05 (M) A B`
fn parse_single_frame(
    text: &str,
    position: Point2D,
    entity_id: EntityId,
) -> Option<FeatureControlFrame> {
    let symbol = symbols::identify_symbol(text)?;
    let tolerance_value = extract_tolerance_value(text)?;
    let material_condition = extract_material_condition(text);
    let datum_refs = extract_datum_refs(text);

    Some(FeatureControlFrame {
        id: entity_id,
        symbol,
        tolerance_value,
        material_condition,
        datum_refs,
        position,
        attached_entities: Vec::new(),
        projected_tolerance: false,
        is_diameter_zone: false,
    })
}

/// Extract the tolerance value (first floating point number after the symbol).
fn extract_tolerance_value(text: &str) -> Option<f64> {
    let mut in_brace = false;
    let mut num_start = None;
    let mut num_end = 0;

    for (i, c) in text.chars().enumerate() {
        if c == '{' {
            in_brace = true;
            continue;
        }
        if c == '}' {
            in_brace = false;
            continue;
        }
        if in_brace {
            continue;
        }

        if c.is_ascii_digit() || c == '.' {
            if num_start.is_none() {
                num_start = Some(i);
            }
            num_end = i + c.len_utf8();
        } else if num_start.is_some() {
            break;
        }
    }

    if let Some(start) = num_start {
        text[start..num_end].parse::<f64>().ok()
    } else {
        None
    }
}

/// Extract material condition modifier from text (legacy MTEXT format).
fn extract_material_condition(text: &str) -> MaterialCondition {
    if text.contains("(M)") || text.contains("\u{24C2}") {
        MaterialCondition::MaximumMaterial
    } else if text.contains("(L)") || text.contains("\u{24C1}") {
        MaterialCondition::LeastMaterial
    } else if text.contains("(S)") {
        MaterialCondition::RegardlessOfFeature
    } else {
        MaterialCondition::None
    }
}

/// Extract datum references from free-form text (legacy MTEXT format).
fn extract_datum_refs(text: &str) -> Vec<DatumReference> {
    let mut refs = Vec::new();
    let mut in_brace = false;
    let mut chars = text.chars().peekable();

    let mut found_number = false;
    let mut past_number = false;

    while let Some(c) = chars.next() {
        if c == '{' {
            in_brace = true;
            continue;
        }
        if c == '}' {
            in_brace = false;
            continue;
        }
        if in_brace {
            continue;
        }

        if c.is_ascii_digit() || c == '.' {
            found_number = true;
        } else if found_number && !c.is_ascii_digit() && c != '.' {
            past_number = true;
        }

        if past_number && c.is_ascii_uppercase() && c.is_alphabetic() {
            let mc = if let Some(&next) = chars.peek() {
                if next == 'M' || next == '(' {
                    MaterialCondition::MaximumMaterial
                } else {
                    MaterialCondition::None
                }
            } else {
                MaterialCondition::None
            };

            refs.push(DatumReference {
                label: c,
                material_condition: mc,
            });
        }
    }

    refs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::model::GdtSymbol;

    // === Legacy MTEXT format tests ===

    #[test]
    fn test_parse_position_frame() {
        let text = r"{\fGDT;j} 0.05 (M) A B";
        let frame = parse_single_frame(text, Point2D::new(0.0, 0.0), EntityId(1)).unwrap();

        assert_eq!(frame.symbol, GdtSymbol::Position);
        assert!((frame.tolerance_value - 0.05).abs() < 1e-6);
        assert_eq!(frame.material_condition, MaterialCondition::MaximumMaterial);
        assert!(frame.datum_refs.len() >= 1);
    }

    #[test]
    fn test_extract_tolerance_value() {
        assert!((extract_tolerance_value(r"{\fGDT;j} 0.05").unwrap() - 0.05).abs() < 1e-6);
        assert!((extract_tolerance_value("0.1 A B").unwrap() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_extract_material_condition() {
        assert_eq!(
            extract_material_condition("0.05 (M) A"),
            MaterialCondition::MaximumMaterial
        );
        assert_eq!(
            extract_material_condition("0.05 A"),
            MaterialCondition::None
        );
    }

    // === TOLERANCE entity text code format tests ===

    #[test]
    fn test_tolerance_entity_simple() {
        // Position, 0.05 tolerance, datum A
        let text = r"{\Fgdt;j}%%v0.05%%vA";
        let frames = parse_tolerance_entity(text, Point2D::new(10.0, 20.0), EntityId(1));
        assert_eq!(frames.len(), 1);
        let f = &frames[0];
        assert_eq!(f.symbol, GdtSymbol::Position);
        assert!((f.tolerance_value - 0.05).abs() < 1e-6);
        assert_eq!(f.material_condition, MaterialCondition::None);
        assert!(!f.is_diameter_zone);
        assert!(!f.projected_tolerance);
        assert_eq!(f.datum_refs.len(), 1);
        assert_eq!(f.datum_refs[0].label, 'A');
    }

    #[test]
    fn test_tolerance_entity_diameter_zone_mmc() {
        // Position, diameter zone 0.05, MMC, datums A and B
        let text = r"{\Fgdt;j}%%v%%c0.05m%%vA%%vB";
        let frames = parse_tolerance_entity(text, Point2D::new(0.0, 0.0), EntityId(2));
        assert_eq!(frames.len(), 1);
        let f = &frames[0];
        assert_eq!(f.symbol, GdtSymbol::Position);
        assert!((f.tolerance_value - 0.05).abs() < 1e-6);
        assert_eq!(f.material_condition, MaterialCondition::MaximumMaterial);
        assert!(f.is_diameter_zone);
        assert_eq!(f.datum_refs.len(), 2);
        assert_eq!(f.datum_refs[0].label, 'A');
        assert_eq!(f.datum_refs[1].label, 'B');
    }

    #[test]
    fn test_tolerance_entity_composite_frame() {
        // Composite: two rows separated by ^J
        let text = r"{\Fgdt;j}%%v%%c0.05m%%vA%%vB^J{\Fgdt;j}%%v0.02m";
        let frames = parse_tolerance_entity(text, Point2D::new(0.0, 0.0), EntityId(3));
        assert_eq!(frames.len(), 2);

        // First row: position, dia 0.05, MMC, datums A B
        assert_eq!(frames[0].symbol, GdtSymbol::Position);
        assert!((frames[0].tolerance_value - 0.05).abs() < 1e-6);
        assert!(frames[0].is_diameter_zone);
        assert_eq!(frames[0].datum_refs.len(), 2);

        // Second row: position, 0.02, MMC, no datums
        assert_eq!(frames[1].symbol, GdtSymbol::Position);
        assert!((frames[1].tolerance_value - 0.02).abs() < 1e-6);
        assert!(!frames[1].is_diameter_zone);
        assert!(frames[1].datum_refs.is_empty());
    }

    #[test]
    fn test_tolerance_entity_projected() {
        // Position with projected tolerance zone
        let text = r"{\Fgdt;j}%%v0.10p%%vA";
        let frames = parse_tolerance_entity(text, Point2D::new(0.0, 0.0), EntityId(4));
        assert_eq!(frames.len(), 1);
        assert!(frames[0].projected_tolerance);
        assert!((frames[0].tolerance_value - 0.10).abs() < 1e-6);
    }

    #[test]
    fn test_tolerance_entity_datum_modifiers() {
        // Position with datum A at MMC, datum B at LMC
        let text = r"{\Fgdt;j}%%v0.05%%vAm%%vBl";
        let frames = parse_tolerance_entity(text, Point2D::new(0.0, 0.0), EntityId(5));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].datum_refs.len(), 2);
        assert_eq!(frames[0].datum_refs[0].label, 'A');
        assert_eq!(
            frames[0].datum_refs[0].material_condition,
            MaterialCondition::MaximumMaterial
        );
        assert_eq!(frames[0].datum_refs[1].label, 'B');
        assert_eq!(
            frames[0].datum_refs[1].material_condition,
            MaterialCondition::LeastMaterial
        );
    }

    #[test]
    fn test_tolerance_entity_flatness() {
        // Flatness 0.01 — no datums (form tolerance)
        let text = r"{\Fgdt;b}%%v0.01";
        let frames = parse_tolerance_entity(text, Point2D::new(0.0, 0.0), EntityId(6));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].symbol, GdtSymbol::Flatness);
        assert!((frames[0].tolerance_value - 0.01).abs() < 1e-6);
        assert!(frames[0].datum_refs.is_empty());
    }

    #[test]
    fn test_tolerance_entity_perpendicularity() {
        let text = r"{\Fgdt;n}%%v0.03%%vA";
        let frames = parse_tolerance_entity(text, Point2D::new(0.0, 0.0), EntityId(7));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].symbol, GdtSymbol::Perpendicularity);
    }

    #[test]
    fn test_tolerance_entity_empty_datum_cells() {
        // Position with only datum A (cells 3-4 are empty)
        let text = r"{\Fgdt;j}%%v0.05%%vA%%v%%v";
        let frames = parse_tolerance_entity(text, Point2D::new(0.0, 0.0), EntityId(8));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].datum_refs.len(), 1);
        assert_eq!(frames[0].datum_refs[0].label, 'A');
    }

    #[test]
    fn test_parse_datum_cell() {
        assert_eq!(parse_datum_cell("A").unwrap().label, 'A');
        assert_eq!(
            parse_datum_cell("Am").unwrap().material_condition,
            MaterialCondition::MaximumMaterial
        );
        assert_eq!(
            parse_datum_cell("Bl").unwrap().material_condition,
            MaterialCondition::LeastMaterial
        );
        assert!(parse_datum_cell("").is_none());
        assert!(parse_datum_cell("  ").is_none());
    }

    #[test]
    fn test_parse_tolerance_cell() {
        let (val, mc, dia, proj) = parse_tolerance_cell("0.05");
        assert!((val - 0.05).abs() < 1e-6);
        assert_eq!(mc, MaterialCondition::None);
        assert!(!dia);
        assert!(!proj);

        let (val, mc, dia, _) = parse_tolerance_cell("%%c0.05m");
        assert!((val - 0.05).abs() < 1e-6);
        assert_eq!(mc, MaterialCondition::MaximumMaterial);
        assert!(dia);

        let (val, _, _, proj) = parse_tolerance_cell("0.10p");
        assert!((val - 0.10).abs() < 1e-6);
        assert!(proj);
    }

    #[test]
    fn test_mtext_gdt_delegates_to_tolerance_parser() {
        // If MTEXT contains TOLERANCE-style format, use that parser
        let text = r"{\Fgdt;j}%%v0.05%%vA";
        let frames = parse_gdt_mtext(text, Point2D::new(0.0, 0.0), EntityId(1));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].symbol, GdtSymbol::Position);
    }
}
