use kcc_core::geometry::model::{Drawing, Entity, GeometryPrimitive, Point2D, Units};
use kcc_core::geometry::utils::clean_coordinate;
use std::collections::HashSet;

/// Apply all normalization steps to a parsed drawing.
pub fn normalize(drawing: &mut Drawing) {
    // 1. Unit conversion: convert inches to millimeters
    if drawing.units == Units::Inches {
        convert_to_mm(drawing);
        drawing.units = Units::Millimeters;
    }

    // 2. Clean floating-point noise
    clean_coordinates(drawing);

    // 3. Remove duplicate entities
    remove_duplicates(drawing);
}

/// Convert all coordinates from inches to millimeters.
fn convert_to_mm(drawing: &mut Drawing) {
    let factor = 25.4;

    for entity in &mut drawing.entities {
        scale_geometry(&mut entity.geometry, factor);
    }

    for dim in &mut drawing.dimensions {
        dim.nominal_value *= factor;
        if let Some(ref mut tol) = dim.tolerance {
            tol.upper *= factor;
            tol.lower *= factor;
        }
        for p in &mut dim.definition_points {
            p.x *= factor;
            p.y *= factor;
        }
        dim.text_position.x *= factor;
        dim.text_position.y *= factor;
    }

    for frame in &mut drawing.gdt_frames {
        frame.tolerance_value *= factor;
        frame.position.x *= factor;
        frame.position.y *= factor;
    }

    for ann in &mut drawing.annotations {
        ann.position.x *= factor;
        ann.position.y *= factor;
        ann.height *= factor;
    }

    for datum in &mut drawing.datums {
        datum.position.x *= factor;
        datum.position.y *= factor;
    }
}

fn scale_geometry(geom: &mut GeometryPrimitive, factor: f64) {
    match geom {
        GeometryPrimitive::Line { start, end } => {
            start.x *= factor;
            start.y *= factor;
            end.x *= factor;
            end.y *= factor;
        }
        GeometryPrimitive::Circle { center, radius } => {
            center.x *= factor;
            center.y *= factor;
            *radius *= factor;
        }
        GeometryPrimitive::Arc { center, radius, .. } => {
            center.x *= factor;
            center.y *= factor;
            *radius *= factor;
        }
        GeometryPrimitive::Polyline { points, .. } => {
            for p in points.iter_mut() {
                p.x *= factor;
                p.y *= factor;
            }
        }
        GeometryPrimitive::Spline { control_points, .. } => {
            for p in control_points.iter_mut() {
                p.x *= factor;
                p.y *= factor;
            }
        }
        GeometryPrimitive::Point(p) => {
            p.x *= factor;
            p.y *= factor;
        }
    }
}

/// Round all coordinates to 6 decimal places.
fn clean_coordinates(drawing: &mut Drawing) {
    for entity in &mut drawing.entities {
        clean_geometry(&mut entity.geometry);
    }
}

fn clean_geometry(geom: &mut GeometryPrimitive) {
    match geom {
        GeometryPrimitive::Line { start, end } => {
            *start = clean_point(start);
            *end = clean_point(end);
        }
        GeometryPrimitive::Circle { center, radius } => {
            *center = clean_point(center);
            *radius = clean_coordinate(*radius);
        }
        GeometryPrimitive::Arc { center, radius, .. } => {
            *center = clean_point(center);
            *radius = clean_coordinate(*radius);
        }
        GeometryPrimitive::Polyline { points, .. } => {
            for p in points.iter_mut() {
                *p = clean_point(p);
            }
        }
        GeometryPrimitive::Spline { control_points, .. } => {
            for p in control_points.iter_mut() {
                *p = clean_point(p);
            }
        }
        GeometryPrimitive::Point(p) => {
            *p = clean_point(p);
        }
    }
}

fn clean_point(p: &Point2D) -> Point2D {
    Point2D::new(clean_coordinate(p.x), clean_coordinate(p.y))
}

/// Remove geometrically identical entities (within epsilon).
fn remove_duplicates(drawing: &mut Drawing) {
    let mut seen = HashSet::new();
    drawing.entities.retain(|entity| {
        let key = entity_fingerprint(entity);
        seen.insert(key)
    });
}

/// Generate a string fingerprint for deduplication.
fn entity_fingerprint(entity: &Entity) -> String {
    match &entity.geometry {
        GeometryPrimitive::Line { start, end } => {
            format!("L:{:.4},{:.4},{:.4},{:.4}", start.x, start.y, end.x, end.y)
        }
        GeometryPrimitive::Circle { center, radius } => {
            format!("C:{:.4},{:.4},{:.4}", center.x, center.y, radius)
        }
        GeometryPrimitive::Arc {
            center,
            radius,
            start_angle,
            end_angle,
        } => {
            format!(
                "A:{:.4},{:.4},{:.4},{:.4},{:.4}",
                center.x, center.y, radius, start_angle, end_angle
            )
        }
        _ => format!("{:?}", entity.id),
    }
}
