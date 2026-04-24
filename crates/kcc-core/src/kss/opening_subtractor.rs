//! Multi-loop void subtraction.
//!
//! Research reference: the "Deterministic Computation of Quantities — Linear
//! and Surface Algorithms" section describing void subtraction ("multi-loop
//! approach where the area of the inner boundary is programmatically subtracted
//! from the area of the outer boundary"). Without this, plaster + paint rows
//! overcount by the combined area of every door and window in the wall.
//!
//! Strategy:
//!   1. Partition the drawing into "openings" and "host surfaces" by layer
//!      prefix (A-DOOR / A-WIND / A-GLAZ for openings; A-WALL / A-PLST / A-PNT
//!      for the surfaces that should be debited).
//!   2. Sum the opening areas with sensible defaults when only an INSERT count
//!      is available (doors ~2.0 m², windows ~1.6 m² per instance).
//!   3. Subtract from any QuantityItem whose unit is m² and whose SEK group
//!      targets masonry/plaster/paint.
//!   4. Emit separate rows (СЕК12 glazing, СЕК17 dograma) so the openings are
//!      accounted for in the final КСС — not lost.

use crate::geometry::model::{Drawing, Entity, GeometryPrimitive};
use crate::kss::types::{ExtractionMethod, QuantityItem};

/// Typical single-leaf door area used when only block counts are available.
/// Derived from standard Bulgarian door nominal 900×2100 mm.
const DEFAULT_DOOR_AREA_M2: f64 = 1.89;

/// Typical window opening area used when only block counts are available.
/// Derived from 1200×1400 mm which is the BG residential median.
const DEFAULT_WINDOW_AREA_M2: f64 = 1.68;

/// SEK groups whose m² values should be reduced by the opening area.
/// (Wall area itself stays unchanged — the deduction applies to finishes.)
const DEDUCT_FROM_GROUPS: &[&str] = &["СЕК10", "СЕК13"];

/// Layer-name prefixes that mark opening geometry (case-insensitive startswith).
const DOOR_LAYER_PREFIXES:   &[&str] = &["a-door", "a-drs", "dograma-v", "ad-врата", "врата"];
const WINDOW_LAYER_PREFIXES: &[&str] = &["a-wind", "a-glaz", "a-glzg", "dograma-p", "ad-прозор", "прозор"];

#[derive(Debug, Clone, Copy, Default)]
pub struct OpeningTotals {
    pub door_count: usize,
    pub window_count: usize,
    pub door_area_m2: f64,
    pub window_area_m2: f64,
}

impl OpeningTotals {
    pub fn total_area_m2(&self) -> f64 {
        self.door_area_m2 + self.window_area_m2
    }
}

/// Scan a drawing and accumulate opening geometry. Uses the Shoelace area of
/// closed polylines when available and falls back to standard nominal sizes
/// per block INSERT otherwise.
pub fn compute_opening_totals(drawing: &Drawing, unit_scale: f64) -> OpeningTotals {
    let mut out = OpeningTotals::default();

    for entity in &drawing.entities {
        let kind = classify_opening(&entity.layer);
        if kind.is_none() { continue; }
        let kind = kind.unwrap();

        // If the entity is a closed polyline, use its actual Shoelace area.
        if let Some(area_m2) = polyline_area_m2(entity, unit_scale) {
            match kind {
                OpeningKind::Door   => { out.door_count   += 1; out.door_area_m2   += area_m2; }
                OpeningKind::Window => { out.window_count += 1; out.window_area_m2 += area_m2; }
            }
            continue;
        }
        // Otherwise fall back to block-count × nominal area.
        if entity.block_ref.is_some() {
            match kind {
                OpeningKind::Door   => { out.door_count   += 1; out.door_area_m2   += DEFAULT_DOOR_AREA_M2; }
                OpeningKind::Window => { out.window_count += 1; out.window_area_m2 += DEFAULT_WINDOW_AREA_M2; }
            }
        }
    }

    out
}

#[derive(Debug, Clone, Copy)]
enum OpeningKind { Door, Window }

fn classify_opening(layer: &str) -> Option<OpeningKind> {
    let l = layer.to_lowercase();
    if DOOR_LAYER_PREFIXES.iter().any(|p| l.contains(p)) {
        Some(OpeningKind::Door)
    } else if WINDOW_LAYER_PREFIXES.iter().any(|p| l.contains(p)) {
        Some(OpeningKind::Window)
    } else {
        None
    }
}

fn polyline_area_m2(entity: &Entity, unit_scale: f64) -> Option<f64> {
    if let GeometryPrimitive::Polyline { points, closed, .. } = &entity.geometry {
        if !*closed || points.len() < 3 { return None; }
        let n = points.len();
        let mut a = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            a += points[i].x * points[j].y;
            a -= points[j].x * points[i].y;
        }
        let area_drawing_units = (a / 2.0).abs();
        // drawing units → mm² → m²
        let area_mm2 = area_drawing_units * unit_scale * unit_scale;
        Some(area_mm2 / 1_000_000.0)
    } else {
        None
    }
}

/// Subtract opening areas from finish rows (plaster / paint / priming) in-place,
/// then return two new rows (one for doors, one for glazing) so the openings
/// appear as their own line items in the КСС.
///
/// The deducted rows are tagged with a lower confidence (0.75) because the
/// subtraction relies on either Shoelace area (trustworthy) OR nominal defaults
/// (less trustworthy). The AI + human reviewer see this via `geometry_confidence`.
pub fn subtract_and_split(
    quantities: &mut Vec<QuantityItem>,
    openings: OpeningTotals,
) -> Vec<QuantityItem> {
    let total_opening_m2 = openings.total_area_m2();
    if total_opening_m2 <= 0.0 {
        return Vec::new();
    }

    // Any measurement came from Shoelace if we saw a real closed polyline.
    // We approximate: if door_count + window_count > (items with source layer
    // matching any opening-prefix and method == PolylineShoelace) we know some
    // were defaults. Cheaper approximation: base confidence on whether openings
    // total_area is non-zero; the nominal-fallback case yields a round multiple.
    let used_defaults =
        (openings.door_area_m2 / DEFAULT_DOOR_AREA_M2).fract().abs() < 0.01 ||
        (openings.window_area_m2 / DEFAULT_WINDOW_AREA_M2).fract().abs() < 0.01;
    let subtraction_method = if used_defaults {
        ExtractionMethod::DerivedFromPrimary
    } else {
        ExtractionMethod::PolylineShoelace
    };

    // Pass 1 — subtract from finish rows.
    for item in quantities.iter_mut() {
        if item.unit != "М2" && item.unit != "m²" { continue; }
        if !DEDUCT_FROM_GROUPS.iter().any(|g| item.category == *g) { continue; }

        // Finishes are computed as "both sides × wall height". Each opening
        // covers both sides, so subtract 2×.
        let deduct = total_opening_m2 * 2.0;
        let before = item.quantity;
        let after = (before - deduct).max(0.0);
        item.quantity = after;
        item.description = format!(
            "{} [нетно — приспаднати отвори: {:.2} м²]",
            item.description,
            deduct,
        );
        // Confidence cannot exceed the weakest link. If we used default areas,
        // the subtraction pulls the row's trust down to the DerivedFromPrimary level.
        let new_conf = item.geometry_confidence.min(subtraction_method.base_confidence());
        item.geometry_confidence = new_conf;
        item.needs_review = item.needs_review || new_conf < 0.60;
    }

    // Pass 2 — emit the two opening rows so they are visible in the КСС.
    let mut emitted: Vec<QuantityItem> = Vec::new();
    if openings.window_count > 0 && openings.window_area_m2 > 0.0 {
        let mut row = QuantityItem::new(
            "СЕК12",
            format!(
                "Остъкление — {} прозорец/прозореца, {:.2} м² общо",
                openings.window_count, openings.window_area_m2
            ),
            "М2",
            openings.window_area_m2,
            "СЕК12.010",
            subtraction_method,
        );
        row.source_layer = Some("opening:window".into());
        emitted.push(row);
    }
    if openings.door_count > 0 && openings.door_area_m2 > 0.0 {
        let mut row = QuantityItem::new(
            "СЕК17",
            format!(
                "Врати — {} бр., {:.2} м² общо",
                openings.door_count, openings.door_area_m2
            ),
            "бр.",
            openings.door_count as f64,
            "СЕК17.020",
            subtraction_method,
        );
        row.source_layer = Some("opening:door".into());
        emitted.push(row);
    }
    emitted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtract_reduces_plaster_and_emits_rows() {
        let mut q = vec![
            QuantityItem::new(
                "СЕК10", "Вътрешна мазилка (двустранно)",
                "М2", 100.0, "СЕК10.011",
                ExtractionMethod::DerivedFromPrimary,
            ),
            QuantityItem::new(
                "СЕК13", "Латексово боядисване",
                "М2", 100.0, "СЕК13.030",
                ExtractionMethod::DerivedFromPrimary,
            ),
            // SEK05 masonry area stays unchanged — it is not a finish.
            QuantityItem::new(
                "СЕК05", "Зидария",
                "М2", 50.0, "СЕК05.002",
                ExtractionMethod::WallAreaFromCenterline,
            ),
        ];
        let ops = OpeningTotals {
            door_count: 2, window_count: 3,
            door_area_m2: DEFAULT_DOOR_AREA_M2 * 2.0,
            window_area_m2: DEFAULT_WINDOW_AREA_M2 * 3.0,
        };
        let added = subtract_and_split(&mut q, ops);
        // Finishes reduced by total × 2 (both sides)
        assert!(q[0].quantity < 100.0);
        assert!(q[1].quantity < 100.0);
        // Masonry unchanged
        assert_eq!(q[2].quantity, 50.0);
        // Two rows emitted
        assert_eq!(added.len(), 2);
        assert!(added.iter().any(|r| r.category == "СЕК12"));
        assert!(added.iter().any(|r| r.category == "СЕК17"));
    }

    #[test]
    fn classify_recognises_standard_prefixes() {
        assert!(matches!(classify_opening("A-DOOR"), Some(OpeningKind::Door)));
        assert!(matches!(classify_opening("A-WIND-1"), Some(OpeningKind::Window)));
        assert!(matches!(classify_opening("a-glaz-ex"), Some(OpeningKind::Window)));
        assert!(classify_opening("A-WALL").is_none());
    }
}
