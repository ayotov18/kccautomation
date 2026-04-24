use super::contour_builder::Contour;
use super::types::{EdgeType, Feature, FeatureId, FeatureType};
use crate::geometry::model::{Drawing, GeometryPrimitive, Point2D};
use crate::geometry::utils;

/// Detect edge features from closed contours.
pub fn detect_edges(contours: &[Contour], drawing: &Drawing) -> Vec<Feature> {
    let mut features = Vec::new();

    for contour in contours {
        let edge_type = if contour.is_outer {
            EdgeType::Outer
        } else {
            EdgeType::Inner
        };

        features.push(Feature {
            id: FeatureId(0),
            feature_type: FeatureType::Edge { edge_type },
            geometry_refs: contour.entity_ids.clone(),
            centroid: utils::centroid(&contour.points),
            dimensions: Vec::new(),
            gdt_frames: Vec::new(),
            datum_refs: Vec::new(),
            layer_hint: None,
        });

        // Detect chamfers and fillets within this contour
        let chamfers = detect_chamfers(contour, drawing);
        features.extend(chamfers);

        let fillets = detect_fillets(contour, drawing);
        features.extend(fillets);
    }

    features
}

/// Detect chamfers: short line segments at corners between longer edges.
fn detect_chamfers(contour: &Contour, drawing: &Drawing) -> Vec<Feature> {
    let mut chamfers = Vec::new();

    // Look at consecutive segments in the contour
    let n = contour.entity_ids.len();
    if n < 3 {
        return chamfers;
    }

    for i in 0..n {
        let prev = if i == 0 { n - 1 } else { i - 1 };
        let next = (i + 1) % n;

        // Get lengths
        if let (Some(prev_len), Some(curr_len), Some(next_len)) = (
            entity_length(drawing, &contour.entity_ids[prev]),
            entity_length(drawing, &contour.entity_ids[i]),
            entity_length(drawing, &contour.entity_ids[next]),
        ) {
            // Chamfer: short line between two longer lines
            if curr_len < prev_len * 0.3 && curr_len < next_len * 0.3 && curr_len > 0.1 {
                // Check if adjacent entities are lines (chamfer connects two lines)
                if is_line_entity(drawing, &contour.entity_ids[prev])
                    && is_line_entity(drawing, &contour.entity_ids[next])
                    && is_line_entity(drawing, &contour.entity_ids[i])
                {
                    let centroid = if i < contour.points.len() {
                        contour.points[i]
                    } else {
                        Point2D::origin()
                    };

                    chamfers.push(Feature {
                        id: FeatureId(0),
                        feature_type: FeatureType::Edge {
                            edge_type: EdgeType::Chamfer,
                        },
                        geometry_refs: vec![contour.entity_ids[i]],
                        centroid,
                        dimensions: Vec::new(),
                        gdt_frames: Vec::new(),
                        datum_refs: Vec::new(),
                        layer_hint: None,
                    });
                }
            }
        }
    }

    chamfers
}

/// Detect fillets: arcs at corners connecting two edges tangentially.
fn detect_fillets(contour: &Contour, drawing: &Drawing) -> Vec<Feature> {
    let mut fillets = Vec::new();

    for &entity_id in &contour.entity_ids {
        if let Some(entity) = drawing.entities.iter().find(|e| e.id == entity_id) {
            if let GeometryPrimitive::Arc { center, radius, .. } = &entity.geometry {
                // Small arcs in contours are likely fillets
                if *radius < 20.0 {
                    fillets.push(Feature {
                        id: FeatureId(0),
                        feature_type: FeatureType::Edge {
                            edge_type: EdgeType::Fillet,
                        },
                        geometry_refs: vec![entity_id],
                        centroid: *center,
                        dimensions: Vec::new(),
                        gdt_frames: Vec::new(),
                        datum_refs: Vec::new(),
                        layer_hint: Some(entity.layer.clone()),
                    });
                }
            }
        }
    }

    fillets
}

fn entity_length(drawing: &Drawing, id: &crate::geometry::model::EntityId) -> Option<f64> {
    let entity = drawing.entities.iter().find(|e| e.id == *id)?;
    match &entity.geometry {
        GeometryPrimitive::Line { start, end } => Some(utils::distance(start, end)),
        GeometryPrimitive::Arc {
            radius,
            start_angle,
            end_angle,
            ..
        } => Some(utils::arc_length(*radius, *start_angle, *end_angle)),
        _ => None,
    }
}

fn is_line_entity(drawing: &Drawing, id: &crate::geometry::model::EntityId) -> bool {
    drawing
        .entities
        .iter()
        .find(|e| e.id == *id)
        .map(|e| matches!(e.geometry, GeometryPrimitive::Line { .. }))
        .unwrap_or(false)
}
