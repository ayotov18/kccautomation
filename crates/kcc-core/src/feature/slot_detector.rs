use super::types::{Feature, FeatureId, FeatureType};
use crate::geometry::model::{Drawing, GeometryPrimitive, Point2D};
use crate::geometry::spatial::SpatialIndex;
use crate::geometry::utils::{self, EPSILON};

/// Detect slot features from arc pairs connected by parallel lines.
pub fn detect_slots(drawing: &Drawing, _index: &SpatialIndex) -> Vec<Feature> {
    let mut slots = Vec::new();

    // Collect all arcs
    let arcs: Vec<_> = drawing
        .entities
        .iter()
        .filter_map(|e| {
            if let GeometryPrimitive::Arc {
                center,
                radius,
                start_angle,
                end_angle,
            } = &e.geometry
            {
                Some((e.id, *center, *radius, *start_angle, *end_angle))
            } else {
                None
            }
        })
        .collect();

    // Check polylines for slot shapes (2 arcs + 2 lines in a closed polyline)
    for entity in &drawing.entities {
        if let GeometryPrimitive::Polyline {
            points,
            bulges,
            closed,
        } = &entity.geometry
        {
            if *closed && points.len() >= 4 {
                if let Some((width, length)) = check_polyline_slot(points, bulges) {
                    let centroid = utils::centroid(points);
                    slots.push(Feature {
                        id: FeatureId(0),
                        feature_type: FeatureType::Slot { width, length },
                        geometry_refs: vec![entity.id],
                        centroid,
                        dimensions: Vec::new(),
                        gdt_frames: Vec::new(),
                        datum_refs: Vec::new(),
                        layer_hint: Some(entity.layer.clone()),
                    });
                }
            }
        }
    }

    // Check arc pairs with connecting lines
    for i in 0..arcs.len() {
        for j in (i + 1)..arcs.len() {
            let (id_a, center_a, radius_a, _, _) = &arcs[i];
            let (id_b, center_b, radius_b, _, _) = &arcs[j];

            // Arcs must have equal radius
            if !utils::float_eq(*radius_a, *radius_b) {
                continue;
            }

            let center_dist = utils::distance(center_a, center_b);

            // Centers must be separated (not the same circle)
            if center_dist < EPSILON {
                continue;
            }

            // Distance should be > 2*radius (otherwise it's more like a circle)
            if center_dist <= *radius_a * 1.5 {
                continue;
            }

            // Check for connecting parallel lines
            if has_connecting_lines(drawing, center_a, center_b, *radius_a) {
                let width = radius_a * 2.0;
                let length = center_dist + width;
                let centroid = utils::midpoint(center_a, center_b);

                slots.push(Feature {
                    id: FeatureId(0),
                    feature_type: FeatureType::Slot { width, length },
                    geometry_refs: vec![*id_a, *id_b],
                    centroid,
                    dimensions: Vec::new(),
                    gdt_frames: Vec::new(),
                    datum_refs: Vec::new(),
                    layer_hint: None,
                });
            }
        }
    }

    slots
}

/// Check if a closed polyline forms a slot shape.
fn check_polyline_slot(points: &[Point2D], bulges: &[f64]) -> Option<(f64, f64)> {
    if points.len() != 4 || bulges.len() != 4 {
        return None;
    }

    // A slot polyline has 2 line segments (bulge = 0) and 2 arc segments (bulge != 0)
    let arc_indices: Vec<usize> = bulges
        .iter()
        .enumerate()
        .filter(|(_, b)| b.abs() > EPSILON)
        .map(|(i, _)| i)
        .collect();

    let line_indices: Vec<usize> = bulges
        .iter()
        .enumerate()
        .filter(|(_, b)| b.abs() < EPSILON)
        .map(|(i, _)| i)
        .collect();

    if arc_indices.len() != 2 || line_indices.len() != 2 {
        return None;
    }

    // Arc bulges should be equal (same radius semicircles)
    if !utils::float_eq(bulges[arc_indices[0]].abs(), bulges[arc_indices[1]].abs()) {
        return None;
    }

    // Line segments should be parallel
    let l1_start = &points[line_indices[0]];
    let l1_end = &points[(line_indices[0] + 1) % points.len()];
    let l2_start = &points[line_indices[1]];
    let l2_end = &points[(line_indices[1] + 1) % points.len()];

    if !utils::are_parallel(l1_start, l1_end, l2_start, l2_end) {
        return None;
    }

    let line_length = utils::distance(l1_start, l1_end);
    let line_spacing = utils::point_to_line_distance(l2_start, l1_start, l1_end);

    // Width = distance between parallel lines, Length = line_length + width
    Some((line_spacing, line_length + line_spacing))
}

/// Check if there are two parallel lines connecting the endpoints of two arcs.
fn has_connecting_lines(
    drawing: &Drawing,
    center_a: &Point2D,
    center_b: &Point2D,
    radius: f64,
) -> bool {
    let lines: Vec<_> = drawing
        .entities
        .iter()
        .filter_map(|e| {
            if let GeometryPrimitive::Line { start, end } = &e.geometry {
                Some((*start, *end))
            } else {
                None
            }
        })
        .collect();

    // Look for two lines that are tangent to both arcs
    let mut connecting_count = 0;
    for (start, end) in &lines {
        let dist_to_a = utils::point_to_line_distance(center_a, start, end);
        let dist_to_b = utils::point_to_line_distance(center_b, start, end);

        if utils::float_eq_tol(dist_to_a, radius, 0.5)
            && utils::float_eq_tol(dist_to_b, radius, 0.5)
        {
            connecting_count += 1;
        }
    }

    connecting_count >= 2
}
