//! Layer-aware quantity calculator for Bulgarian KSS generation.
//!
//! Extracts quantities from DXF geometry by:
//! 1. Grouping entities by layer name
//! 2. Mapping layers to KSS sections via `layer_mapper`
//! 3. Computing quantities per method (wall area, volume, counts, etc.)
//! 4. Parsing room area annotations from TEXT entities
//! 5. Counting block INSERT references for doors/windows/fixtures

use std::collections::HashMap;

use crate::geometry::model::{Drawing, Entity, GeometryPrimitive, Point2D};
use crate::geometry::utils;
use crate::kss::layer_mapper::{self, QuantityMethod};
use crate::kss::types::{ExtractionMethod, QuantityItem};

/// Default wall height when not derivable from the drawing (in mm).
const DEFAULT_WALL_HEIGHT_MM: f64 = 2800.0;

/// Default concrete wall thickness (in mm).
const DEFAULT_CONCRETE_THICKNESS_MM: f64 = 200.0;

/// Configuration for quantity calculation.
#[derive(Debug, Clone)]
pub struct QuantityCalcConfig {
    pub wall_height_mm: f64,
    pub concrete_thickness_mm: f64,
    /// Scale factor to convert drawing coordinates to mm.
    /// Derived from drawing.units.to_mm_scale().
    pub unit_scale: f64,
}

impl Default for QuantityCalcConfig {
    fn default() -> Self {
        Self {
            wall_height_mm: DEFAULT_WALL_HEIGHT_MM,
            concrete_thickness_mm: DEFAULT_CONCRETE_THICKNESS_MM,
            unit_scale: 1.0,
        }
    }
}

impl QuantityCalcConfig {
    /// Create config from a drawing, auto-detecting unit scale AND a plausible
    /// wall height from the drawing's own dimensions. Two-signal heuristic for
    /// unitless drawings: (a) maximum coordinate extent, (b) median dimension
    /// value. If the two signals disagree, the larger-scale wins and both
    /// values are logged so the auditor knows we assumed.
    pub fn from_drawing(drawing: &Drawing) -> Self {
        let mut config = Self::default();
        config.unit_scale = drawing.units.to_mm_scale();

        if drawing.units == crate::geometry::model::Units::Unitless {
            config.unit_scale = infer_unitless_scale(drawing);
        }

        // Derive wall height from the drawing if a plausible ceiling-height
        // dimension exists (2300-3500 mm). Boosts geometry_confidence of
        // wall-volume rows from 0.65 → 0.80 downstream.
        if let Some(h_mm) = infer_wall_height_mm(drawing, config.unit_scale) {
            tracing::info!(
                height_mm = h_mm,
                default = DEFAULT_WALL_HEIGHT_MM,
                "Dimension miner: overriding DEFAULT wall height"
            );
            config.wall_height_mm = h_mm;
        }

        tracing::info!(
            units = ?drawing.units,
            scale = config.unit_scale,
            wall_height_mm = config.wall_height_mm,
            "Quantity calc config"
        );
        config
    }
}

/// Two-signal unitless unit inference: extents vs dimension values.
/// Returns the mm-per-drawing-unit scale.
fn infer_unitless_scale(drawing: &Drawing) -> f64 {
    // Signal A: max coordinate extent
    let max_coord = drawing.entities.iter()
        .filter_map(|e| match &e.geometry {
            GeometryPrimitive::Line { start, end } => {
                Some(start.x.abs().max(start.y.abs()).max(end.x.abs()).max(end.y.abs()))
            }
            GeometryPrimitive::Polyline { points, .. } if !points.is_empty() => {
                Some(points.iter()
                    .map(|p| p.x.abs().max(p.y.abs()))
                    .fold(0.0f64, f64::max))
            }
            _ => None,
        })
        .fold(0.0f64, f64::max);

    // Signal B: median value of real dimensions (value is usually in the
    // *authored* unit, not drawing coordinates — they can differ).
    let mut dim_values: Vec<f64> = drawing.dimensions.iter()
        .map(|d| d.nominal_value.abs())
        .filter(|v| *v > 0.01)
        .collect();
    dim_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let dim_median = if dim_values.is_empty() {
        0.0
    } else {
        dim_values[dim_values.len() / 2]
    };

    // Resolve.
    //   extents in [1, 200]   + dim_median [0.5, 10]      → meters (×1000)
    //   extents in [200, 20000] + dim_median [50, 1000]   → cm (×10)
    //   extents > 20000 + dim_median [500, 10000]         → mm (×1)
    //   mixed case (e.g. extents ≈ 99 000 + dim_median ≈ 250) → treat
    //   entities as mm but dimensions as cm; return mm scale, downstream
    //   `wall_height_mm` inference handles the dimension-side conversion.
    let scale = if max_coord > 0.0 && max_coord < 200.0 {
        1000.0 // meters
    } else if max_coord > 0.0 && max_coord < 20_000.0 {
        if dim_median > 50.0 && dim_median < 1000.0 { 10.0 } else { 1.0 }
    } else if max_coord >= 20_000.0 {
        1.0 // mm
    } else {
        1.0
    };

    tracing::info!(
        extent_max = max_coord,
        dim_median,
        resolved_scale = scale,
        "Unitless heuristic"
    );
    scale
}

/// Find the most plausible ceiling height from the drawing's dimensions.
/// Accepts values already in drawing units OR in cm (if the drawing has
/// mixed-units). Returns the height in mm, or None if no plausible value.
fn infer_wall_height_mm(drawing: &Drawing, unit_scale: f64) -> Option<f64> {
    if drawing.dimensions.is_empty() { return None; }

    // Candidate pass: convert each dimension to mm using the DXF authored scale
    // AND, as an alternative, assume cm — then keep whichever hits the
    // 2300-3500 mm ceiling-height band.
    let mut candidates: Vec<f64> = Vec::new();
    for d in &drawing.dimensions {
        let raw = d.nominal_value.abs();
        if raw <= 0.0 { continue; }
        // Path A: value × unit_scale — treat dim value as drawing coordinates.
        let mm_a = raw * unit_scale;
        if (2300.0..=3500.0).contains(&mm_a) { candidates.push(mm_a); continue; }
        // Path B: value × 10 — treat as cm (common mixed-unit case).
        let mm_b = raw * 10.0;
        if (2300.0..=3500.0).contains(&mm_b) { candidates.push(mm_b); continue; }
        // Path C: value × 1 — treat as mm.
        if (2300.0..=3500.0).contains(&raw) { candidates.push(raw); }
    }
    if candidates.is_empty() { return None; }

    // Return the mode (most frequent, rounded to 10 mm), tiebreak by median.
    let rounded: Vec<i32> = candidates.iter().map(|v| (*v / 10.0).round() as i32 * 10).collect();
    let mut freq: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();
    for v in &rounded { *freq.entry(*v).or_default() += 1; }
    let (best, _) = freq.iter().max_by_key(|(_, c)| *c)?;
    Some(*best as f64)
}

/// Extract construction quantities from a drawing using layer-based classification.
pub fn extract_layer_quantities(drawing: &Drawing, config: &QuantityCalcConfig) -> Vec<QuantityItem> {
    let mut quantities = Vec::new();

    // 1. Group entities by layer and compute per-layer quantities
    let layer_quantities = compute_layer_quantities(drawing, config);
    quantities.extend(layer_quantities);

    // 2. Count block references for doors, windows, fixtures
    let block_quantities = count_block_references(drawing);
    quantities.extend(block_quantities);

    // 3. Parse room area annotations from TEXT entities
    let room_quantities = parse_room_annotations(drawing);
    quantities.extend(room_quantities);

    // 4. Derive secondary quantities (plaster, painting, screed from wall/floor areas)
    let derived = derive_secondary_quantities(&quantities, config);
    quantities.extend(derived);

    // 5. Subtract opening (door/window) areas from finishes, then emit the
    //    two opening rows (СЕК12 glazing + СЕК17 doors) so they appear on the
    //    final КСС. Multi-loop void subtraction per the research brief.
    let openings = crate::kss::opening_subtractor::compute_opening_totals(
        drawing,
        config.unit_scale,
    );
    let opening_rows = crate::kss::opening_subtractor::subtract_and_split(
        &mut quantities,
        openings,
    );
    if !opening_rows.is_empty() {
        tracing::info!(
            source = "opening_subtractor",
            added = opening_rows.len(),
            "Post-primary splitter added rows"
        );
    }
    quantities.extend(opening_rows);

    // 5b. Steel profile extractor (research's "extrude 2D metadata" logic).
    // Adds СЕК14 rows when the drawing has enough 4-vertex closed polylines
    // on structural layers — typical of steel-fabrication takeoffs.
    let (steel_rows, steel_stats) = crate::kss::steel_profile_extractor::extract_steel_profiles(
        drawing,
        config.unit_scale,
    );
    if !steel_rows.is_empty() {
        tracing::info!(
            source = "steel_profile_extractor",
            added = steel_rows.len(),
            total_kg = steel_stats.total_kg,
            candidates = steel_stats.candidates_considered,
            accepted = steel_stats.candidates_accepted,
            "Post-primary splitter added rows"
        );
    }
    quantities.extend(steel_rows);

    // 6. Schema-contract audit. Flags rows with bad units, non-positive
    //    quantities, invalid SEK codes, or out-of-range confidence. Does NOT
    //    drop rows — the human still sees every item via needs_review = true.
    let audit = crate::kss::validators::validate_schema(&mut quantities);
    tracing::info!(
        total = audit.total_rows,
        passed = audit.passed,
        needs_review = audit.needs_review,
        violations = audit.violations.len(),
        "Schema audit complete"
    );
    for warn in &audit.violations {
        tracing::debug!(check = %warn.check, message = %warn.message, "Schema violation");
    }

    quantities
}

/// Group entities by layer, map to SEK groups, compute quantities.
fn compute_layer_quantities(drawing: &Drawing, config: &QuantityCalcConfig) -> Vec<QuantityItem> {
    let mut quantities = Vec::new();

    // Group entities by layer
    let mut layers: HashMap<String, Vec<&Entity>> = HashMap::new();
    for entity in &drawing.entities {
        layers.entry(entity.layer.clone()).or_default().push(entity);
    }

    for (layer_name, entities) in &layers {
        let mapping = match layer_mapper::map_layer(layer_name) {
            Some(m) if !m.sek_group.is_empty() => m,
            _ => continue, // skip unmapped layers and empty-group skip layers
        };

        // Apply unit_scale: convert drawing units → mm before computing
        let s = config.unit_scale;

        let (quantity, method) = match mapping.quantity_method {
            QuantityMethod::WallArea => {
                let line_length_raw = sum_line_lengths(entities);
                if line_length_raw <= 0.0 { continue; }
                let line_length_mm = line_length_raw * s;
                let area_m2 = (line_length_mm * config.wall_height_mm) / 1_000_000.0;
                // Length is measured, height is an assumption → "from centerline".
                (area_m2, ExtractionMethod::WallAreaFromCenterline)
            }
            QuantityMethod::WallVolume => {
                let line_length_raw = sum_line_lengths(entities);
                if line_length_raw <= 0.0 { continue; }
                let line_length_mm = line_length_raw * s;
                let vol_m3 = (line_length_mm * config.wall_height_mm * config.concrete_thickness_mm)
                    / 1_000_000_000.0;
                // Length measured, height + thickness both assumed → lowest trust.
                (vol_m3, ExtractionMethod::WallVolumeFromCenterline)
            }
            QuantityMethod::FloorArea => {
                let area_raw = sum_closed_polyline_areas(entities);
                if area_raw <= 0.0 { continue; }
                let area_mm2 = area_raw * s * s;
                (area_mm2 / 1_000_000.0, ExtractionMethod::PolylineShoelace)
            }
            QuantityMethod::LinearLength => {
                let length_raw = sum_line_lengths(entities);
                if length_raw <= 0.0 { continue; }
                let length_mm = length_raw * s;
                (length_mm / 1000.0, ExtractionMethod::LinearPolyline)
            }
            QuantityMethod::DirectArea => {
                let area_raw = sum_closed_polyline_areas(entities);
                if area_raw <= 0.0 { continue; }
                let area_mm2 = area_raw * s * s;
                (area_mm2 / 1_000_000.0, ExtractionMethod::PolylineShoelace)
            }
            QuantityMethod::SteelWeight => {
                let length_raw = sum_line_lengths(entities);
                if length_raw <= 0.0 { continue; }
                let length_mm = length_raw * s;
                let length_m = length_mm / 1000.0;
                // Length measured, weight-per-meter is a hardcoded 20 kg/m fallback.
                (length_m * 20.0, ExtractionMethod::DerivedFromPrimary)
            }
            QuantityMethod::BlockCount => {
                (entities.len() as f64, ExtractionMethod::BlockInstanceCount)
            }
        };

        if quantity > 0.0 {
            // Pick a representative entity for back-reference (first in group).
            let first = entities.first();
            let centroid = first.and_then(|e| entity_centroid(e));
            let entity_id = first.map(|e| format!("{}", e.id.0));
            let mut item = QuantityItem::new(
                mapping.sek_group,
                format!("{} (слой: {})", mapping.work_description_bg, layer_name),
                mapping.unit,
                quantity,
                mapping.sek_group,
                method,
            );
            item.source_layer = Some(layer_name.clone());
            item.source_entity_id = entity_id;
            item.centroid = centroid;
            quantities.push(item);
        }
    }

    quantities
}

/// Count block INSERT references for doors, windows, plumbing fixtures.
fn count_block_references(drawing: &Drawing) -> Vec<QuantityItem> {
    let mut counts: HashMap<(&str, &str), usize> = HashMap::new();

    for entity in &drawing.entities {
        if let Some(ref block_name) = entity.block_ref {
            if let Some((sek_group, desc)) = layer_mapper::map_block(block_name) {
                *counts.entry((sek_group, desc)).or_insert(0) += 1;
            }
        }
    }

    counts
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|((sek_group, desc), count)| {
            QuantityItem::new(
                sek_group, desc, "бр.", count as f64, sek_group,
                ExtractionMethod::BlockInstanceCount,
            )
        })
        .collect()
}

/// Parse TEXT annotations that contain room area values (e.g., "16m2", "19m²").
fn parse_room_annotations(drawing: &Drawing) -> Vec<QuantityItem> {
    let mut total_area = 0.0f64;
    let mut room_count = 0usize;

    for annotation in &drawing.annotations {
        if let Some(area) = parse_area_text(&annotation.text) {
            total_area += area;
            room_count += 1;
        }
    }

    let mut quantities = Vec::new();
    if total_area > 0.0 {
        quantities.push(QuantityItem::new(
            "СЕК11",
            format!("Подови настилки ({} помещения, {:.1} м² общо)", room_count, total_area),
            "М2", total_area, "СЕК11.020",
            ExtractionMethod::TextAnnotation,
        ));
    }

    quantities
}

/// Derive secondary quantities from primary ones.
/// E.g., wall area → internal plaster, painting, priming.
fn derive_secondary_quantities(primary: &[QuantityItem], _config: &QuantityCalcConfig) -> Vec<QuantityItem> {
    let mut derived = Vec::new();

    // Sum up masonry wall areas (both sides need plaster + paint)
    let masonry_area: f64 = primary
        .iter()
        .filter(|q| q.category == "СЕК05" && q.unit == "М2")
        .map(|q| q.quantity)
        .sum();

    if masonry_area > 0.0 {
        // Internal plaster = both sides of walls — DerivedFromPrimary keeps the
        // confidence slightly below the source wall row (0.70 vs 0.90).
        let plaster_area = masonry_area * 2.0;
        derived.push(QuantityItem::new(
            "СЕК10", "Вътрешна варова мазилка по стени (двустранно)",
            "М2", plaster_area, "СЕК10.011",
            ExtractionMethod::DerivedFromPrimary,
        ));
        derived.push(QuantityItem::new(
            "СЕК13", "Грундиране по стени и тавани",
            "М2", plaster_area, "СЕК13.025",
            ExtractionMethod::DerivedFromPrimary,
        ));
        derived.push(QuantityItem::new(
            "СЕК13", "Латексово боядисване по стени и тавани (2 слоя)",
            "М2", plaster_area, "СЕК13.030",
            ExtractionMethod::DerivedFromPrimary,
        ));
    }

    // Floor area → cement screed
    let floor_area: f64 = primary
        .iter()
        .filter(|q| q.category == "СЕК11")
        .map(|q| q.quantity)
        .sum();

    if floor_area > 0.0 {
        derived.push(QuantityItem::new(
            "СЕК11", "Циментова замазка по подове",
            "М2", floor_area, "СЕК11.020",
            ExtractionMethod::DerivedFromPrimary,
        ));
    }

    // Drywall area → also needs painting
    let drywall_area: f64 = primary
        .iter()
        .filter(|q| q.category == "СЕК20")
        .map(|q| q.quantity)
        .sum();

    if drywall_area > 0.0 {
        derived.push(QuantityItem::new(
            "СЕК13", "Шпакловка и боядисване на гипсокартон",
            "М2", drywall_area * 2.0, "СЕК13.025",
            ExtractionMethod::DerivedFromPrimary,
        ));
    }

    derived
}

/// Parse area from annotation text like "16m2", "2x6.5=13m2", "19m²", "21 м2".
fn parse_area_text(text: &str) -> Option<f64> {
    let trimmed = text.trim().to_string();

    // Normalize: replace all area markers with a canonical "M2" token
    let normalized = trimmed
        .replace("м²", "M2")
        .replace("m²", "M2")
        .replace("м2", "M2")
        .replace("m2", "M2");

    if !normalized.contains("M2") {
        return None;
    }

    // If it contains "=", take the value after "="
    let value_part = if let Some(eq_pos) = normalized.find('=') {
        &normalized[eq_pos + 1..]
    } else {
        &normalized
    };

    // Extract the number before "M2"
    let num_end = value_part.find("M2")?;
    let num_str = value_part[..num_end].trim();
    num_str.parse::<f64>().ok().filter(|v| *v > 0.0 && *v < 10000.0)
}

/// Sum line lengths of all LINE entities in a set.
fn sum_line_lengths(entities: &[&Entity]) -> f64 {
    let mut total = 0.0;
    for entity in entities {
        match &entity.geometry {
            GeometryPrimitive::Line { start, end } => {
                total += utils::distance(start, end);
            }
            GeometryPrimitive::Arc { radius, start_angle, end_angle, .. } => {
                total += utils::arc_length(*radius, *start_angle, *end_angle);
            }
            GeometryPrimitive::Polyline { points, closed, .. } => {
                for pair in points.windows(2) {
                    total += utils::distance(&pair[0], &pair[1]);
                }
                if *closed && points.len() >= 2 {
                    total += utils::distance(points.last().unwrap(), &points[0]);
                }
            }
            _ => {}
        }
    }
    total
}

/// Sum areas of closed polylines (Shoelace formula).
fn sum_closed_polyline_areas(entities: &[&Entity]) -> f64 {
    let mut total = 0.0;
    for entity in entities {
        if let GeometryPrimitive::Polyline { points, closed: true, .. } = &entity.geometry {
            if points.len() >= 3 {
                total += shoelace_area(points);
            }
        }
    }
    total
}

/// Approximate centroid of a single entity in drawing units. Used for
/// back-reference so the frontend can highlight the source geometry when the
/// user clicks a KSS row.
fn entity_centroid(entity: &Entity) -> Option<(f64, f64)> {
    match &entity.geometry {
        GeometryPrimitive::Line { start, end } => {
            Some(((start.x + end.x) / 2.0, (start.y + end.y) / 2.0))
        }
        GeometryPrimitive::Arc { center, .. } | GeometryPrimitive::Circle { center, .. } => {
            Some((center.x, center.y))
        }
        GeometryPrimitive::Polyline { points, .. } if !points.is_empty() => {
            let n = points.len() as f64;
            let sx: f64 = points.iter().map(|p| p.x).sum();
            let sy: f64 = points.iter().map(|p| p.y).sum();
            Some((sx / n, sy / n))
        }
        _ => None,
    }
}

/// Compute area of a polygon using the Shoelace formula (result in mm²).
fn shoelace_area(points: &[Point2D]) -> f64 {
    let n = points.len();
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].x * points[j].y;
        area -= points[j].x * points[i].y;
    }
    (area / 2.0).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_area_text() {
        assert_eq!(parse_area_text("16m2"), Some(16.0));
        assert_eq!(parse_area_text("19m²"), Some(19.0));
        assert_eq!(parse_area_text("21 м2"), Some(21.0));
        assert_eq!(parse_area_text("2x6.5=13m2"), Some(13.0));
        assert_eq!(parse_area_text("no area here"), None);
        assert_eq!(parse_area_text("abc"), None);
    }

    #[test]
    fn test_shoelace_area() {
        // 10x10 square = 100
        let pts = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
        ];
        assert!((shoelace_area(&pts) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_room_annotations() {
        use crate::geometry::model::{Annotation, Drawing};

        let mut drawing = Drawing::new("test.dxf".into());
        drawing.annotations.push(Annotation {
            id: crate::geometry::model::EntityId(1),
            text: "16m2".into(),
            position: Point2D::new(0.0, 0.0),
            height: 10.0,
            rotation: 0.0,
            layer: "0".into(),
        });
        drawing.annotations.push(Annotation {
            id: crate::geometry::model::EntityId(2),
            text: "2x6.5=13m2".into(),
            position: Point2D::new(100.0, 0.0),
            height: 10.0,
            rotation: 0.0,
            layer: "0".into(),
        });

        let quantities = parse_room_annotations(&drawing);
        assert_eq!(quantities.len(), 1);
        assert!((quantities[0].quantity - 29.0).abs() < 0.01); // 16 + 13
    }
}
