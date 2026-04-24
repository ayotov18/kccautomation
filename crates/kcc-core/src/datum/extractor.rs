use super::types::DatumInfo;
use crate::geometry::model::Drawing;
use crate::geometry::spatial::SpatialIndex;
use crate::geometry::utils;

/// Extract datum features from the drawing.
pub fn extract_datums(drawing: &Drawing, _index: &SpatialIndex) -> Vec<DatumInfo> {
    let mut datums = Vec::new();

    // 1. Check existing datums in the drawing (from DXF parsing)
    for datum in &drawing.datums {
        datums.push(DatumInfo {
            label: datum.label,
            position: datum.position,
            attached_feature_id: None,
        });
    }

    // 2. Search annotations for datum labels (single uppercase letter near datum symbols)
    for annotation in &drawing.annotations {
        let text = annotation.text.trim();

        // Datum labels are single uppercase letters
        if text.len() == 1 {
            if let Some(c) = text.chars().next() {
                if c.is_ascii_uppercase() {
                    // Check if this label already exists
                    if !datums.iter().any(|d| d.label == c) {
                        datums.push(DatumInfo {
                            label: c,
                            position: annotation.position,
                            attached_feature_id: None,
                        });
                    }
                }
            }
        }

        // Also check for "DATUM A", "DATUM B" patterns
        if text.starts_with("DATUM ") || text.starts_with("datum ") {
            if let Some(c) = text.chars().last() {
                if c.is_ascii_uppercase() && !datums.iter().any(|d| d.label == c) {
                    datums.push(DatumInfo {
                        label: c,
                        position: annotation.position,
                        attached_feature_id: None,
                    });
                }
            }
        }
    }

    // 3. Search block references for datum symbol blocks
    for entity in &drawing.entities {
        if let Some(ref block_name) = entity.block_ref {
            let name_upper = block_name.to_uppercase();
            if name_upper.contains("DATUM") || name_upper.contains("GDT_DATUM") {
                // Try to find the datum label from nearby annotations
                if let Some(label) = find_nearby_datum_label(drawing, &entity_centroid(entity)) {
                    if !datums.iter().any(|d| d.label == label) {
                        datums.push(DatumInfo {
                            label,
                            position: entity_centroid(entity),
                            attached_feature_id: None,
                        });
                    }
                }
            }
        }
    }

    // 4. Extract datums from GD&T frames (datum references tell us which datums exist)
    for frame in &drawing.gdt_frames {
        for datum_ref in &frame.datum_refs {
            if !datums.iter().any(|d| d.label == datum_ref.label) {
                // We know this datum exists but don't know its position
                // Use the frame's position as an approximation
                datums.push(DatumInfo {
                    label: datum_ref.label,
                    position: frame.position,
                    attached_feature_id: None,
                });
            }
        }
    }

    datums
}

fn entity_centroid(entity: &crate::geometry::model::Entity) -> crate::geometry::model::Point2D {
    use crate::geometry::model::GeometryPrimitive;
    match &entity.geometry {
        GeometryPrimitive::Circle { center, .. } => *center,
        GeometryPrimitive::Arc { center, .. } => *center,
        GeometryPrimitive::Line { start, end } => utils::midpoint(start, end),
        GeometryPrimitive::Point(p) => *p,
        GeometryPrimitive::Polyline { points, .. } => utils::centroid(points),
        GeometryPrimitive::Spline { control_points, .. } => utils::centroid(control_points),
    }
}

fn find_nearby_datum_label(
    drawing: &Drawing,
    position: &crate::geometry::model::Point2D,
) -> Option<char> {
    for annotation in &drawing.annotations {
        if utils::distance(&annotation.position, position) < 10.0 {
            let text = annotation.text.trim();
            if text.len() == 1 {
                if let Some(c) = text.chars().next() {
                    if c.is_ascii_uppercase() {
                        return Some(c);
                    }
                }
            }
        }
    }
    None
}
