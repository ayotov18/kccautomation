use super::types::{Feature, FeatureId, FeatureType};
use crate::geometry::model::{Drawing, EntityId, GeometryPrimitive, Point2D};
use crate::geometry::spatial::SpatialIndex;
use crate::geometry::utils;

/// Minimum hole radius in mm (filter out construction geometry).
const MIN_HOLE_RADIUS: f64 = 0.25;
/// Maximum hole radius in mm (filter out large construction circles).
const MAX_HOLE_RADIUS: f64 = 250.0;
/// Search radius for nearby dimensions (mm).
const DIM_SEARCH_RADIUS: f64 = 5.0;

/// Detect holes from circle entities AND arc-pairs in the drawing.
pub fn detect_holes(drawing: &Drawing, index: &SpatialIndex) -> Vec<Feature> {
    let mut holes = Vec::new();

    // 1. Detect holes from explicit Circle entities
    for entity in &drawing.entities {
        if let GeometryPrimitive::Circle { center, radius } = &entity.geometry {
            if *radius < MIN_HOLE_RADIUS || *radius > MAX_HOLE_RADIUS {
                continue;
            }

            let diameter = radius * 2.0;
            let is_through = check_through_hole(drawing, center, *radius, index);

            holes.push(Feature {
                id: FeatureId(0),
                feature_type: FeatureType::Hole {
                    diameter,
                    is_through,
                },
                geometry_refs: vec![entity.id],
                centroid: *center,
                dimensions: find_related_dimensions(drawing, center, *radius),
                gdt_frames: Vec::new(),
                datum_refs: Vec::new(),
                layer_hint: Some(entity.layer.clone()),
            });
        }
    }

    // 2. Detect holes from arc-pairs that form complete circles
    // (common in structural steel drawings where holes come from polyline decomposition)
    let arcs: Vec<_> = drawing.entities.iter().filter(|e| {
        matches!(e.geometry, GeometryPrimitive::Arc { radius, .. } if radius > MIN_HOLE_RADIUS && radius < MAX_HOLE_RADIUS)
    }).collect();

    for i in 0..arcs.len() {
        for j in (i + 1)..arcs.len() {
            if let (
                GeometryPrimitive::Arc { center: c1, radius: r1, start_angle: s1, end_angle: e1 },
                GeometryPrimitive::Arc { center: c2, radius: r2, start_angle: s2, end_angle: e2 },
            ) = (&arcs[i].geometry, &arcs[j].geometry) {
                // Same center (within tolerance) and same radius
                let center_dist = utils::distance(c1, c2);
                let radius_diff = (r1 - r2).abs();
                let radius_tol = r1 * 0.01; // 1% tolerance

                if center_dist < radius_tol && radius_diff < radius_tol {
                    // Check if the two arcs together span ~360 degrees
                    let span1 = utils::arc_span(*s1, *e1);
                    let span2 = utils::arc_span(*s2, *e2);
                    let total_span = span1 + span2;

                    if (total_span - std::f64::consts::TAU).abs() < 0.2 {
                        // These two arcs form a circle — it's a hole
                        let diameter = r1 * 2.0;
                        let already_found = holes.iter().any(|h| {
                            utils::distance(&h.centroid, c1) < radius_tol
                        });
                        if !already_found {
                            holes.push(Feature {
                                id: FeatureId(0),
                                feature_type: FeatureType::Hole {
                                    diameter,
                                    is_through: false,
                                },
                                geometry_refs: vec![arcs[i].id, arcs[j].id],
                                centroid: *c1,
                                dimensions: find_related_dimensions(drawing, c1, *r1),
                                gdt_frames: Vec::new(),
                                datum_refs: Vec::new(),
                                layer_hint: Some(arcs[i].layer.clone()),
                            });
                        }
                    }
                }
            }
        }
    }

    holes
}

/// Check if a hole is a through-hole by looking for cross-hatch or annotations.
fn check_through_hole(
    drawing: &Drawing,
    center: &Point2D,
    radius: f64,
    _index: &SpatialIndex,
) -> bool {
    // Check annotations for "THRU" or "THROUGH"
    for ann in &drawing.annotations {
        if utils::distance(&ann.position, center) < radius * 3.0 {
            let text_upper = ann.text.to_uppercase();
            if text_upper.contains("THRU") || text_upper.contains("THROUGH") {
                return true;
            }
        }
    }

    // Check for cross-hatch lines inside the circle (common through-hole indicator)
    let mut internal_lines = 0;
    for entity in &drawing.entities {
        if let GeometryPrimitive::Line { start, end } = &entity.geometry {
            let mid = utils::midpoint(start, end);
            let len = utils::distance(start, end);
            if utils::distance(&mid, center) < radius && len < radius * 2.0 {
                internal_lines += 1;
            }
        }
    }

    // Multiple short lines inside = likely cross-hatch = through hole
    internal_lines >= 2
}

/// Find dimension entities related to a hole.
fn find_related_dimensions(drawing: &Drawing, center: &Point2D, radius: f64) -> Vec<EntityId> {
    drawing
        .dimensions
        .iter()
        .filter(|dim| {
            // Check if any definition point is near the circle
            dim.definition_points
                .iter()
                .any(|p| utils::distance(p, center) < radius + DIM_SEARCH_RADIUS)
                || utils::distance(&dim.text_position, center) < radius + DIM_SEARCH_RADIUS * 2.0
        })
        .map(|dim| dim.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::model::{Drawing, Entity, EntityId, GeometryPrimitive};

    fn make_test_drawing_with_holes() -> Drawing {
        let mut drawing = Drawing::new("test.dxf".to_string());
        // Add 4 circles (holes)
        for i in 0..4 {
            drawing.entities.push(Entity {
                id: EntityId(i),
                geometry: GeometryPrimitive::Circle {
                    center: Point2D::new(i as f64 * 25.0, 0.0),
                    radius: 5.0,
                },
                layer: "0".to_string(),
                color: None,
                lineweight: None,
                linetype: None,
                block_ref: None,
            });
        }
        drawing
    }

    #[test]
    fn test_detect_holes() {
        let drawing = make_test_drawing_with_holes();
        let index = SpatialIndex::build(&drawing.entities);
        let holes = detect_holes(&drawing, &index);
        assert_eq!(holes.len(), 4);
        for hole in &holes {
            match &hole.feature_type {
                FeatureType::Hole { diameter, .. } => {
                    assert!((diameter - 10.0).abs() < 1e-6);
                }
                _ => panic!("Expected hole feature"),
            }
        }
    }
}
