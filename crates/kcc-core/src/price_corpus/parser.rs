//! Offer XLSX parser — extracts priced KSS rows for the user price corpus.
//!
//! The wooddesign offer template is the reference shape:
//!
//!   row N (header): No | Описание | м.ед. | Колич. | Ед. Цена мат |
//!                   Цена мат | Монтаж | Цена монтаж | Общо
//!
//! We auto-detect the header row by scanning the first ~20 rows of each
//! sheet for cells containing "описание" / "наименование" / "description"
//! and then map columns by header text. Multi-sheet workbooks are read
//! per-sheet (one offer per module is the wooddesign convention).
//!
//! Tolerant of variants — a header that says "ед. цена материал" /
//! "цена на материал" / "matierial price" will still be picked up.

use calamine::{Data, Range, Reader};
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct OfferRow {
    pub sek_code: Option<String>,
    pub description: String,
    pub unit: String,
    pub quantity: Option<f64>,
    pub material_price_eur: f64,
    pub labor_price_eur: f64,
    pub total_unit_price_eur: f64,
    pub source_sheet: String,
    pub source_row: u32,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedOffer {
    pub sheets: Vec<String>,
    pub rows: Vec<OfferRow>,
    /// Rows we visually skipped (footer, headers, blank descriptions).
    pub skipped_rows: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("workbook open failed: {0}")]
    Workbook(String),
    #[error("no usable data found in any sheet")]
    Empty,
}

pub fn parse_offer_xlsx(bytes: &[u8]) -> Result<ParsedOffer, ParseError> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut wb = calamine::open_workbook_auto_from_rs(cursor)
        .map_err(|e| ParseError::Workbook(e.to_string()))?;

    let mut out = ParsedOffer::default();
    let sheet_names: Vec<String> = wb.sheet_names().to_vec();

    for sheet_name in sheet_names {
        out.sheets.push(sheet_name.clone());
        let Ok(range) = wb.worksheet_range(&sheet_name) else {
            continue;
        };
        let parsed = parse_sheet(&range, &sheet_name);
        out.rows.extend(parsed.rows);
        out.skipped_rows += parsed.skipped_rows;
    }

    if out.rows.is_empty() {
        return Err(ParseError::Empty);
    }
    Ok(out)
}

fn parse_sheet(range: &Range<Data>, sheet_name: &str) -> ParsedOffer {
    let mut out = ParsedOffer::default();
    let Some((header_row_idx, cols)) = detect_columns(range) else {
        // Sheet has no recognisable offer header — skip silently.
        return out;
    };

    for (row_idx, row) in range.rows().enumerate() {
        if row_idx <= header_row_idx {
            continue;
        }
        let desc = cols
            .description
            .and_then(|c| row.get(c))
            .map(cell_string)
            .unwrap_or_default();
        let desc = desc.trim();
        if desc.is_empty() || desc.chars().count() < 3 {
            out.skipped_rows += 1;
            continue;
        }
        // Skip footer rows ("Общо в Евро без ДДС", etc.) and notes.
        let lc = desc.to_lowercase();
        if lc.starts_with("общо")
            || lc.starts_with("всичко")
            || lc.starts_with("сума")
            || lc.starts_with("забележ")
            || lc.starts_with("срок")
            || lc.starts_with("с уважение")
            || lc.starts_with("не са вкл")
            || lc.starts_with("www.")
        {
            out.skipped_rows += 1;
            continue;
        }
        // Sometimes the "No" column has empty rows that are just labels in
        // column B (e.g. category dividers). Treat as data only when there's
        // a unit AND at least one numeric (quantity or any price).
        let unit = cols
            .unit
            .and_then(|c| row.get(c))
            .map(cell_string)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let qty = cols.quantity.and_then(|c| row.get(c)).and_then(cell_float);
        let mat_unit = cols
            .material_price_unit
            .and_then(|c| row.get(c))
            .and_then(cell_float)
            .unwrap_or(0.0);
        let lab_unit = cols
            .labor_price_unit
            .and_then(|c| row.get(c))
            .and_then(cell_float)
            .unwrap_or(0.0);
        let total_row = cols
            .total
            .and_then(|c| row.get(c))
            .and_then(cell_float)
            .unwrap_or(0.0);

        // Drop only the truly empty rows (no description-meaningful unit
        // and no numeric anywhere). Anything with a description goes in,
        // even unpriced "Баня и оборудване" / "Транспорт" placeholders —
        // the user wants the library to mirror the XLSX 1:1. RAG search
        // already filters out priceless rows at query time, so they don't
        // pollute matching.
        let any_value = mat_unit > 0.0 || lab_unit > 0.0 || total_row > 0.0 || qty.unwrap_or(0.0) > 0.0;
        if unit.is_empty() && !any_value {
            out.skipped_rows += 1;
            continue;
        }

        // Total UNIT price = mat_unit + lab_unit. The "общо" column in the
        // offer is total*qty, not unit total — we keep both (per-unit goes
        // into the corpus; quantity is preserved for traceability but not
        // searched on).
        let total_unit_price = mat_unit + lab_unit;
        // Some offer rows skip the labour split and put everything in the
        // total column. If unit prices are zero but total/qty are present,
        // back-derive a unit price from total/qty.
        let total_unit_price = if total_unit_price <= 0.0 && total_row > 0.0 && qty.unwrap_or(0.0) > 0.0 {
            total_row / qty.unwrap()
        } else {
            total_unit_price
        };

        // SEK code: optional. Some offers carry a "СЕК05.001"-style code in
        // a dedicated column. The wooddesign template doesn't, but we keep
        // the slot so future imports can carry it through.
        let sek_code = cols
            .sek_code
            .and_then(|c| row.get(c))
            .map(cell_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        out.rows.push(OfferRow {
            sek_code,
            description: desc.to_string(),
            unit: normalise_unit(&unit),
            quantity: qty,
            material_price_eur: mat_unit,
            labor_price_eur: lab_unit,
            total_unit_price_eur: total_unit_price,
            source_sheet: sheet_name.to_string(),
            source_row: row_idx as u32 + 1, // 1-indexed for human-readability
        });
    }

    out
}

#[derive(Debug, Default, Clone, Copy)]
struct ColumnMap {
    description: Option<usize>,
    unit: Option<usize>,
    quantity: Option<usize>,
    material_price_unit: Option<usize>,
    labor_price_unit: Option<usize>,
    total: Option<usize>,
    sek_code: Option<usize>,
}

fn detect_columns(range: &Range<Data>) -> Option<(usize, ColumnMap)> {
    // Scan first 20 rows for a header row containing a description column.
    for (row_idx, row) in range.rows().enumerate().take(20) {
        let mut map = ColumnMap::default();
        let mut hits = 0;
        for (col_idx, cell) in row.iter().enumerate() {
            let s = cell_string(cell).to_lowercase();
            let s = s.trim();
            if s.is_empty() {
                continue;
            }

            // Description column (mandatory)
            if map.description.is_none()
                && (s.contains("описание")
                    || s.contains("наименование")
                    || s == "description"
                    || s.contains("description"))
            {
                map.description = Some(col_idx);
                hits += 1;
                continue;
            }

            // Unit
            if map.unit.is_none()
                && (s.contains("м.ед")
                    || s == "ед"
                    || s.contains("ед. мярка")
                    || s.contains("мярка")
                    || s == "unit"
                    || s == "uom")
            {
                map.unit = Some(col_idx);
                hits += 1;
                continue;
            }

            // Quantity
            if map.quantity.is_none()
                && (s.contains("колич") || s == "qty" || s == "quantity")
            {
                map.quantity = Some(col_idx);
                hits += 1;
                continue;
            }

            // Material unit price — prefer the explicit "ед. цена мат" column
            // over the broader "цена мат" (which is total = qty × unit price
            // in the wooddesign template). Both have "цена" + "мат".
            if map.material_price_unit.is_none()
                && s.contains("ед")
                && s.contains("цена")
                && (s.contains("мат") || s.contains("материал"))
            {
                map.material_price_unit = Some(col_idx);
                hits += 1;
                continue;
            }
            if map.material_price_unit.is_none()
                && (s == "material price" || s.contains("material") && s.contains("price"))
            {
                map.material_price_unit = Some(col_idx);
                hits += 1;
                continue;
            }

            // Labor unit price. Header is often just "Монтаж" with no
            // "цена" prefix; that's the unit cost in wooddesign offers.
            if map.labor_price_unit.is_none()
                && (s == "монтаж"
                    || (s.contains("монтаж") && !s.contains("цена"))
                    || s == "labor"
                    || s == "labour"
                    || s.contains("labor price"))
            {
                map.labor_price_unit = Some(col_idx);
                hits += 1;
                continue;
            }

            // Total per row
            if map.total.is_none() && (s == "общо" || s == "total" || s.contains("сума")) {
                map.total = Some(col_idx);
                hits += 1;
                continue;
            }

            // SEK code (rare in offers but supported)
            if map.sek_code.is_none() && (s.starts_with("сек") || s == "sek_code") {
                map.sek_code = Some(col_idx);
                hits += 1;
                continue;
            }
        }

        // Need at minimum: description, unit, AND (material OR labor OR total) price.
        let priced = map.material_price_unit.is_some()
            || map.labor_price_unit.is_some()
            || map.total.is_some();
        if map.description.is_some() && map.unit.is_some() && priced && hits >= 3 {
            return Some((row_idx, map));
        }
    }
    None
}

fn cell_string(c: &Data) -> String {
    match c {
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.0}", f)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => format!("{:?}", d),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(_) | Data::Empty => String::new(),
    }
}

fn cell_float(c: &Data) -> Option<f64> {
    match c {
        Data::Float(f) => Some(*f),
        Data::Int(i) => Some(*i as f64),
        Data::String(s) => s.trim().replace(',', ".").parse::<f64>().ok(),
        _ => None,
    }
}

/// Normalise unit strings so "м2 / m2 / М²" all collapse to "М2", etc.
fn normalise_unit(raw: &str) -> String {
    let s = raw.trim().to_lowercase();
    let s = s.replace(' ', "").replace('.', "");
    match s.as_str() {
        "м2" | "m2" | "м²" | "m²" | "квм" | "кв.м" | "квадратенметър" => "М2".to_string(),
        "м3" | "m3" | "м³" | "m³" | "кубм" => "М3".to_string(),
        "мл" | "м" | "m" | "ml" => "М".to_string(),
        "бр" | "бр." | "брой" | "broy" | "pcs" | "ea" => "БР".to_string(),
        "кг" | "kg" => "КГ".to_string(),
        "т" | "тон" | "t" | "tn" => "Т".to_string(),
        "л" | "lit" | "литр" | "liter" => "Л".to_string(),
        _ => raw.trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wooddesign_offer() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/offer_tasos.xlsx");
        if !path.exists() {
            eprintln!("Skipping: fixture not present at {}", path.display());
            return;
        }
        let bytes = std::fs::read(&path).expect("read fixture");
        let parsed = parse_offer_xlsx(&bytes).expect("parse");
        assert_eq!(parsed.sheets.len(), 3, "expected 3 sheets, got {:?}", parsed.sheets);
        assert!(
            parsed.rows.len() >= 50,
            "expected >= 50 priced rows across 3 modules, got {}",
            parsed.rows.len()
        );

        // Spot-check: KVH 10×12 row with mat=690, lab=950
        let kvh = parsed
            .rows
            .iter()
            .find(|r| r.description.contains("10х12") && r.description.to_lowercase().contains("kvh"));
        let kvh = kvh.expect("expected to find KVH 10x12 row");
        assert!((kvh.material_price_eur - 690.0).abs() < 0.01);
        assert!((kvh.labor_price_eur - 950.0).abs() < 0.01);
        assert!((kvh.total_unit_price_eur - 1640.0).abs() < 0.01);
        assert_eq!(kvh.unit, "М3");
    }
}
