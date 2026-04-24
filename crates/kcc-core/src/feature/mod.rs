pub mod centerline_detector;
pub mod contour_builder;
pub mod edge_detector;
pub mod hole_detector;
pub mod pattern_detector;
pub mod slot_detector;
pub mod steel_detector;
pub mod thread_detector;
pub mod types;

use crate::geometry::model::Drawing;
use crate::geometry::spatial::SpatialIndex;
use crate::geometry::utils;
use types::FeatureSet;

/// Compute an adaptive search radius for linking dimensions to features.
/// Uses 2% of the drawing's diagonal extent so it works for both
/// mm-scale (typical: ~200mm) and large-coordinate drawings (~100,000 units).
fn compute_link_radius(drawing: &Drawing) -> f64 {
    if drawing.entities.is_empty() {
        return 15.0;
    }
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for e in &drawing.entities {
        let pts = match &e.geometry {
            crate::geometry::model::GeometryPrimitive::Line { start, end } => vec![*start, *end],
            crate::geometry::model::GeometryPrimitive::Circle { center, .. } => vec![*center],
            crate::geometry::model::GeometryPrimitive::Arc { center, .. } => vec![*center],
            crate::geometry::model::GeometryPrimitive::Point(p) => vec![*p],
            crate::geometry::model::GeometryPrimitive::Polyline { points, .. } => points.clone(),
            crate::geometry::model::GeometryPrimitive::Spline { control_points, .. } => control_points.clone(),
        };
        for p in pts {
            if p.x.is_finite() && p.y.is_finite() {
                min_x = min_x.min(p.x);
                min_y = min_y.min(p.y);
                max_x = max_x.max(p.x);
                max_y = max_y.max(p.y);
            }
        }
    }
    let dx = max_x - min_x;
    let dy = max_y - min_y;
    let diagonal = (dx * dx + dy * dy).sqrt();
    // 2% of diagonal, clamped between 5 and 500
    (diagonal * 0.02).clamp(5.0, 500.0)
}

/// Run all feature detectors on a parsed drawing.
pub fn extract_features(drawing: &Drawing, index: &SpatialIndex) -> FeatureSet {
    let mut features = FeatureSet::new();

    // Detect holes from circles
    let holes = hole_detector::detect_holes(drawing, index);
    for h in &holes {
        features.add(h.clone());
    }

    // Detect slots
    let slots = slot_detector::detect_slots(drawing, index);
    for s in &slots {
        features.add(s.clone());
    }

    // Detect edges and contours
    let contours = contour_builder::find_closed_contours(drawing);
    let edges = edge_detector::detect_edges(&contours, drawing);
    for e in &edges {
        features.add(e.clone());
    }

    // Detect bolt circle patterns from holes
    let bolt_circles = pattern_detector::detect_bolt_circles(&holes);
    for bc in &bolt_circles {
        features.add(bc.clone());
    }

    // Detect linear patterns
    let linear_patterns = pattern_detector::detect_linear_patterns(&features.features);
    for lp in &linear_patterns {
        features.add(lp.clone());
    }

    // Detect centerlines
    let centerlines = centerline_detector::detect_centerlines(drawing);
    for cl in &centerlines {
        features.add(cl.clone());
    }

    // Detect threads
    let threads = thread_detector::detect_threads(drawing, &holes);
    for t in &threads {
        features.add(t.clone());
    }

    // Detect structural steel features (members, plates, bolt groups)
    let steel = steel_detector::detect_steel_features(drawing, index);
    for s in &steel {
        features.add(s.clone());
    }

    // Post-extraction: link dimensions and GD&T to ALL features by spatial proximity.
    let link_radius = compute_link_radius(drawing);
    tracing::info!(link_radius = %format!("{:.1}", link_radius), "Computed adaptive link radius");
    link_dimensions_to_features(&mut features, drawing, link_radius);
    link_gdt_to_features(&mut features, drawing, link_radius);

    features
}

/// Link drawing dimensions to features whose geometry refs or centroids
/// are near the dimension's definition points or text position.
fn link_dimensions_to_features(features: &mut FeatureSet, drawing: &Drawing, link_radius: f64) {
    for feature in &mut features.features {
        if !feature.dimensions.is_empty() {
            continue;
        }
        for dim in &drawing.dimensions {
            let near_centroid = dim.definition_points.iter().any(|p| {
                utils::distance(p, &feature.centroid) < link_radius
            });
            let near_text = utils::distance(&dim.text_position, &feature.centroid) < link_radius * 2.0;
            let entity_overlap = dim.attached_entities.iter().any(|e| {
                feature.geometry_refs.contains(e)
            });

            if near_centroid || near_text || entity_overlap {
                feature.dimensions.push(dim.id);
            }
        }
    }
}

fn link_gdt_to_features(features: &mut FeatureSet, drawing: &Drawing, link_radius: f64) {
    for feature in &mut features.features {
        if !feature.gdt_frames.is_empty() {
            continue;
        }
        for frame in &drawing.gdt_frames {
            let near = utils::distance(&frame.position, &feature.centroid) < link_radius;
            let entity_overlap = frame.attached_entities.iter().any(|e| {
                feature.geometry_refs.contains(e)
            });
            if near || entity_overlap {
                feature.gdt_frames.push(frame.id);
                for dref in &frame.datum_refs {
                    if !feature.datum_refs.contains(&dref.label) {
                        feature.datum_refs.push(dref.label);
                    }
                }
            }
        }
    }
}
