use crate::feature::types::{Feature, FeatureType};
use crate::geometry::model::{Drawing, GeometryPrimitive};
use crate::geometry::utils;

use super::types::{ExtractionMethod, QuantityItem};

/// Convenience constructor that stamps `extraction_method` + derived
/// confidence flags consistently across every row emitted from feature data.
fn qi(
    category: &str,
    description: String,
    unit: &str,
    quantity: f64,
    sek: &str,
    method: ExtractionMethod,
) -> QuantityItem {
    QuantityItem::new(category, description, unit, quantity, sek, method)
}

/// Steel density in kg/m³ (structural steel). Reserved for future weight calculation.
#[allow(dead_code)]
const STEEL_DENSITY: f64 = 7850.0;

/// Extract quantities from parsed drawing features for KSS generation.
pub fn extract_quantities(features: &[Feature], drawing: &Drawing) -> Vec<QuantityItem> {
    let mut quantities = Vec::new();

    // Group and count features by type
    let mut hole_count = 0usize;
    let mut bolt_count = 0usize;
    let mut thread_count = 0usize;
    let mut total_edge_length = 0.0f64;
    let mut total_surface_area = 0.0f64;
    let mut total_line_length = 0.0f64;

    let mut steel_member_count = 0usize;
    let mut steel_member_length = 0.0f64;
    let mut gusset_count = 0usize;
    let mut gusset_area = 0.0f64;
    let mut bolt_group_total = 0usize;

    for feature in features {
        match &feature.feature_type {
            FeatureType::Hole { diameter, .. } => {
                hole_count += 1;
                let _ = diameter;
            }
            FeatureType::BoltCircle { hole_count: count, hole_diameter, .. } => {
                bolt_count += count;
                let _ = hole_diameter;
            }
            FeatureType::Thread { designation, .. } => {
                thread_count += 1;
                let _ = designation;
            }
            FeatureType::Surface { area, .. } => {
                total_surface_area += area;
            }
            FeatureType::Edge { .. } => {
                let perimeter = compute_feature_perimeter(feature, drawing);
                total_edge_length += perimeter;
            }
            FeatureType::Slot { width, length } => {
                // Counted from a real feature (0.95 confidence).
                quantities.push(qi(
                    "Slots",
                    format!("Slot {:.1}x{:.1}mm", width, length),
                    "pcs", 1.0, "14.010",
                    ExtractionMethod::BlockInstanceCount,
                ));
            }
            FeatureType::SteelMember { length, profile_hint, .. } => {
                steel_member_count += 1;
                if *length > 0.0 {
                    steel_member_length += length;
                }
                if let Some(profile) = profile_hint {
                    quantities.push(qi(
                        "Steel members",
                        format!("Profile: {}", profile),
                        "pcs", 1.0, "14.001",
                        ExtractionMethod::BlockInstanceCount,
                    ));
                }
            }
            FeatureType::GussetPlate { area, .. } => {
                gusset_count += 1;
                gusset_area += area;
            }
            FeatureType::BoltGroup { bolt_count: count, bolt_diameter, .. } => {
                bolt_group_total += count;
                quantities.push(qi(
                    "Bolt groups",
                    format!("Bolt group: {}x \u{2300}{:.1}mm", count, bolt_diameter),
                    "pcs", *count as f64, "14.015",
                    ExtractionMethod::BlockInstanceCount,
                ));
            }
            _ => {}
        }
    }

    // Compute total steel member length from Line entities
    for entity in &drawing.entities {
        if let GeometryPrimitive::Line { start, end } = &entity.geometry {
            total_line_length += utils::distance(start, end);
        }
    }

    // Extract dimension values for member sizes
    let mut dimension_values: Vec<(String, f64)> = Vec::new();
    for dim in &drawing.dimensions {
        if dim.nominal_value > 0.0 {
            let text = dim.text_override.as_deref().unwrap_or("");
            dimension_values.push((text.to_string(), dim.nominal_value));
        }
    }

    // Emit quantity items

    // Steel members — total line length converted to approximate weight
    if total_line_length > 0.0 {
        // Line-length is a real measurement from polylines.
        let length_m = total_line_length / 1000.0;
        quantities.push(qi(
            "Steel members",
            format!("Structural steel members ({:.1}m total length)", length_m),
            "m", length_m, "14.001",
            ExtractionMethod::LinearPolyline,
        ));
    }

    // Holes / bolt assemblies — direct counts.
    let total_fasteners = hole_count + bolt_count + thread_count;
    if total_fasteners > 0 {
        quantities.push(qi(
            "Fasteners",
            format!("Bolt/hole assemblies ({hole_count} holes, {bolt_count} bolt patterns, {thread_count} threads)"),
            "pcs", total_fasteners as f64, "14.015",
            ExtractionMethod::BlockInstanceCount,
        ));
    }

    // Steel plates (from edge features — approximate area).
    if total_edge_length > 0.0 {
        let perimeter_m = total_edge_length / 1000.0;
        quantities.push(qi(
            "Plates/Gussets",
            format!("Steel plates and gussets ({:.1}m total perimeter)", perimeter_m),
            "m", perimeter_m, "14.003",
            ExtractionMethod::LinearPolyline,
        ));
    }

    // Surface treatment — derived from measured surface area.
    if total_surface_area > 0.0 {
        let area_m2 = total_surface_area / 1_000_000.0;
        quantities.push(qi(
            "Surface treatment",
            format!("Anti-corrosion treatment ({:.2}m\u{00B2})", area_m2),
            "m\u{00B2}", area_m2, "14.020",
            ExtractionMethod::DerivedFromPrimary,
        ));
    }

    // Welding — estimate from total joint length (60% ratio is an assumption).
    if total_edge_length > 0.0 {
        let weld_m = total_edge_length / 1000.0 * 0.6;
        if weld_m > 0.0 {
            quantities.push(qi(
                "Welding",
                format!("Weld joints ({:.1}m estimated)", weld_m),
                "m", weld_m, "14.025",
                ExtractionMethod::DerivedFromPrimary,
            ));
        }
    }

    // Steel members (detected via parallel line pairs) — counted from features.
    if steel_member_count > 0 && steel_member_length > 0.0 {
        let length_m = steel_member_length / 1000.0;
        quantities.push(qi(
            "Steel members",
            format!("{} steel members ({:.1}m total)", steel_member_count, length_m),
            "m", length_m, "14.001",
            ExtractionMethod::LinearPolyline,
        ));
    }

    // Gusset plates — measured area.
    if gusset_count > 0 {
        let area_m2 = gusset_area / 1_000_000.0;
        quantities.push(qi(
            "Gusset plates",
            format!("{} connection plates ({:.2}m\u{00B2})", gusset_count, area_m2),
            "pcs", gusset_count as f64, "14.003",
            ExtractionMethod::BlockInstanceCount,
        ));
    }

    // Bolt groups (aggregated)
    if bolt_group_total > 0 {
        // Already added per-group above, add aggregate
    }

    // Include dimension-derived items — parsed from drawing text annotation.
    for (text, value) in &dimension_values {
        if !text.is_empty() && text.contains('x') {
            quantities.push(qi(
                "Profile",
                format!("Profile designation: {}", text),
                "pcs", 1.0, "14.001",
                ExtractionMethod::TextAnnotation,
            ));
        }
        let _ = value;
    }

    quantities
}

/// Compute the total perimeter of a feature from its geometry refs.
fn compute_feature_perimeter(feature: &Feature, drawing: &Drawing) -> f64 {
    let mut total = 0.0;
    for entity_id in &feature.geometry_refs {
        if let Some(entity) = drawing.entities.iter().find(|e| e.id == *entity_id) {
            match &entity.geometry {
                GeometryPrimitive::Line { start, end } => {
                    total += utils::distance(start, end);
                }
                GeometryPrimitive::Arc { radius, start_angle, end_angle, .. } => {
                    total += utils::arc_length(*radius, *start_angle, *end_angle);
                }
                _ => {}
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::types::FeatureId;
    use crate::geometry::model::{Entity, EntityId, Point2D};

    #[test]
    fn test_extract_quantities_with_holes() {
        let drawing = Drawing::new("test.dxf".into());
        let features = vec![
            Feature {
                id: FeatureId(1),
                feature_type: FeatureType::Hole { diameter: 16.0, is_through: true },
                geometry_refs: vec![EntityId(1)],
                centroid: Point2D::new(0.0, 0.0),
                dimensions: vec![],
                gdt_frames: vec![],
                datum_refs: vec![],
                layer_hint: None,
            },
            Feature {
                id: FeatureId(2),
                feature_type: FeatureType::Hole { diameter: 16.0, is_through: true },
                geometry_refs: vec![EntityId(2)],
                centroid: Point2D::new(50.0, 0.0),
                dimensions: vec![],
                gdt_frames: vec![],
                datum_refs: vec![],
                layer_hint: None,
            },
        ];

        let quantities = extract_quantities(&features, &drawing);
        let fasteners = quantities.iter().find(|q| q.category == "Fasteners");
        assert!(fasteners.is_some());
        assert!((fasteners.unwrap().quantity - 2.0).abs() < 0.01);
    }
}
