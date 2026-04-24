//! Steel-profile length + weight extractor for structural drawings.
//!
//! Research reference: the research's "Volumetric Extraction from 2D
//! Metadata" section — "extrude 2D areas based on metadata or textual
//! parameters". For steel fabrication we do the equivalent with 4-vertex
//! closed LWPOLYLINEs on structural layers (typical H/I/U cross-section
//! footprints) × a canonical profile-weight lookup.
//!
//! This is deliberately conservative: we do NOT try to reconstruct the
//! profile name from the drawing (that needs OCR). Instead we classify each
//! rectangle by its short-edge width bucket and pick the class-average weight
//! per metre for that bucket. The result is flagged as
//! `ExtractionMethod::BlockInstanceCount` with `geometry_confidence = 0.70`
//! so the AI and the human reviewer both understand: the count is real, the
//! per-metre weight is derived.
//!
//! The extractor runs only when:
//!   - There are at least 3 closed 4-vertex polylines on structural layers.
//!   - Those polylines' short-edges cluster around ≥ 1 known profile bucket.
//! Otherwise it returns an empty vec and yields to the rule-based path.

use crate::geometry::model::{Drawing, Entity, GeometryPrimitive};
use crate::kss::types::{ExtractionMethod, QuantityItem};

/// Layer-name prefixes that indicate structural/steel content.
/// Case-insensitive `contains` match on the layer name.
const STRUCTURAL_LAYER_MARKERS: &[&str] = &[
    "kon", "koloni", "metal", "steel", "stoman",
    "beam", "column", "ipe", "heb", "hea", "upn", "ipn",
    "s-", "s_", "a-steel", "a-metal",
];

/// One bucket of profile-width → canonical class + average kg/m.
struct ProfileBucket {
    label: &'static str,
    min_short_mm: f64,
    max_short_mm: f64,
    kg_per_m: f64,
}

/// Bulgarian steel fabrication is dominated by H/I/U profiles 100-300 mm wide.
/// Averages computed across HEB/IPE/HEA/UPN profile catalogues so a single
/// number per bucket is a reasonable "unknown profile in this size range".
const BUCKETS: &[ProfileBucket] = &[
    ProfileBucket { label: "I/HEB 100-140", min_short_mm: 90.0,  max_short_mm: 140.0, kg_per_m: 20.0 },
    ProfileBucket { label: "I/HEB 140-180", min_short_mm: 140.0, max_short_mm: 180.0, kg_per_m: 32.0 },
    ProfileBucket { label: "I/HEB 180-220", min_short_mm: 180.0, max_short_mm: 220.0, kg_per_m: 50.0 },
    ProfileBucket { label: "I/HEB 220-260", min_short_mm: 220.0, max_short_mm: 260.0, kg_per_m: 70.0 },
    ProfileBucket { label: "I/HEB 260-320", min_short_mm: 260.0, max_short_mm: 320.0, kg_per_m: 95.0 },
];

#[derive(Debug, Default)]
pub struct SteelExtractionStats {
    pub candidates_considered: usize,
    pub candidates_accepted: usize,
    pub total_kg: f64,
}

/// Extract steel profile quantities from a drawing. The returned items are
/// already configured with appropriate traceability + confidence.
pub fn extract_steel_profiles(
    drawing: &Drawing,
    unit_scale_mm: f64,
) -> (Vec<QuantityItem>, SteelExtractionStats) {
    let mut stats = SteelExtractionStats::default();

    // Find candidate rectangles on structural layers.
    let mut candidates: Vec<(&Entity, f64, f64)> = Vec::new();
    for entity in &drawing.entities {
        if !is_structural_layer(&entity.layer) { continue; }
        stats.candidates_considered += 1;
        if let Some((short_mm, long_mm)) = rectangle_short_long_mm(entity, unit_scale_mm) {
            candidates.push((entity, short_mm, long_mm));
        }
    }
    if candidates.len() < 3 {
        return (Vec::new(), stats);
    }

    // Group candidates by bucket. Rectangles outside every bucket are ignored.
    let mut grouped: std::collections::HashMap<&'static str, (f64, f64, usize)> =
        std::collections::HashMap::new(); // label → (kg_per_m, total_length_m, count)
    for (entity, short_mm, long_mm) in &candidates {
        if let Some(bucket) = BUCKETS.iter().find(|b| *short_mm >= b.min_short_mm && *short_mm < b.max_short_mm) {
            let length_m = long_mm / 1000.0;
            let entry = grouped.entry(bucket.label).or_insert((bucket.kg_per_m, 0.0, 0));
            entry.1 += length_m;
            entry.2 += 1;
            stats.candidates_accepted += 1;
            let _ = entity;
        }
    }
    if grouped.is_empty() {
        return (Vec::new(), stats);
    }

    // Emit one QuantityItem per bucket.
    let mut items = Vec::new();
    for (label, (kg_per_m, total_length_m, count)) in grouped {
        let kg = total_length_m * kg_per_m;
        if kg <= 0.0 { continue; }
        stats.total_kg += kg;
        let desc = format!(
            "Стоманени профили {label} ({count} елемента, ~{length:.1} м обща дължина)",
            length = total_length_m,
        );
        let mut item = QuantityItem::new(
            "СЕК14",
            desc,
            "кг",
            kg.round(),
            "СЕК14.001",
            ExtractionMethod::BlockInstanceCount,
        );
        // Count is real (polyline count), weight is derived from a bucket
        // average. Downgrade confidence to 0.70 to reflect that the per-metre
        // weight is a bucket assumption, not a profile-name lookup.
        item.geometry_confidence = 0.70;
        item.needs_review = false; // 0.70 is above the 0.60 review threshold
        items.push(item);
    }
    (items, stats)
}

fn is_structural_layer(layer: &str) -> bool {
    let l = layer.to_lowercase();
    STRUCTURAL_LAYER_MARKERS.iter().any(|m| l.contains(m))
}

/// If the entity is a 4-vertex polyline AND its vertices form a right-ish
/// rectangle, return (short_edge_mm, long_edge_mm). Otherwise None.
fn rectangle_short_long_mm(entity: &Entity, unit_scale_mm: f64) -> Option<(f64, f64)> {
    let points = match &entity.geometry {
        GeometryPrimitive::Polyline { points, .. } if points.len() == 4 => points,
        _ => return None,
    };
    // Edge lengths.
    let e: Vec<f64> = (0..4)
        .map(|i| {
            let a = &points[i];
            let b = &points[(i + 1) % 4];
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            (dx * dx + dy * dy).sqrt() * unit_scale_mm
        })
        .collect();

    // Must have 2 pairs of equal edges (rectangle) within 8% tolerance.
    // (Cheap structural check — skips arbitrary quadrilaterals.)
    let (e0, e1, e2, e3) = (e[0], e[1], e[2], e[3]);
    if e0.min(e2) <= 0.0 || e1.min(e3) <= 0.0 { return None; }
    if (e0 - e2).abs() / e0.max(e2) > 0.08 { return None; }
    if (e1 - e3).abs() / e1.max(e3) > 0.08 { return None; }

    let short = e0.min(e1);
    let long = e0.max(e1);
    // Skip degenerate shapes (short < 40 mm) and square-ish blocks (long/short < 1.2).
    if short < 40.0 { return None; }
    if long / short < 1.2 { return None; }
    Some((short, long))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::model::{Entity, EntityId, Point2D};

    fn make_poly(layer: &str, w: f64, h: f64, id: u64) -> Entity {
        let pts = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(w, 0.0),
            Point2D::new(w, h),
            Point2D::new(0.0, h),
        ];
        Entity {
            id: EntityId(id),
            geometry: GeometryPrimitive::Polyline {
                points: pts,
                bulges: vec![0.0; 4],
                closed: true,
            },
            layer: layer.into(),
            color: None,
            lineweight: None,
            linetype: None,
            block_ref: None,
        }
    }

    #[test]
    fn structural_marker_recognises_kon_layer() {
        assert!(is_structural_layer("KON"));
        assert!(is_structural_layer("kon"));
        assert!(is_structural_layer("Koloni-1"));
        assert!(is_structural_layer("Metal"));
        assert!(is_structural_layer("ipe200"));
        assert!(!is_structural_layer("A-WALL"));
    }

    fn make_drawing(entities: Vec<Entity>) -> crate::geometry::model::Drawing {
        let mut d = crate::geometry::model::Drawing::new("test".to_string());
        d.entities = entities;
        d
    }

    #[test]
    fn extracts_three_heb160_columns() {
        let entities = vec![
            make_poly("KON", 160.0, 3000.0, 1),
            make_poly("KON", 160.0, 3000.0, 2),
            make_poly("KON", 160.0, 3000.0, 3),
        ];
        let drawing = make_drawing(entities);
        let (items, stats) = extract_steel_profiles(&drawing, 1.0);
        assert_eq!(stats.candidates_accepted, 3);
        assert_eq!(items.len(), 1);
        // 3 × 3.0 m × 32 kg/m = 288 kg
        let kg: f64 = items[0].quantity;
        assert!(kg >= 250.0 && kg <= 320.0, "expected ~288 kg, got {kg}");
        assert_eq!(items[0].category, "СЕК14");
        assert_eq!(items[0].extraction_method.as_str(), "block_instance_count");
    }

    #[test]
    fn ignores_non_structural_layers() {
        let entities = vec![make_poly("A-WALL", 200.0, 3000.0, 1); 4];
        let drawing = make_drawing(entities);
        let (items, _) = extract_steel_profiles(&drawing, 1.0);
        assert!(items.is_empty());
    }

    #[test]
    fn rejects_non_rectangles() {
        let pts = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(200.0, 0.0),
            Point2D::new(250.0, 3000.0),
            Point2D::new(0.0, 3000.0),
        ];
        let e = Entity {
            id: EntityId(1),
            geometry: GeometryPrimitive::Polyline { points: pts, bulges: vec![0.0; 4], closed: true },
            layer: "KON".into(),
            color: None, lineweight: None, linetype: None, block_ref: None,
        };
        let drawing = make_drawing(vec![e; 3]);
        let (items, _) = extract_steel_profiles(&drawing, 1.0);
        assert!(items.is_empty());
    }
}
