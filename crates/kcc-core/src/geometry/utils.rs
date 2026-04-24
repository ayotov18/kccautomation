use super::model::Point2D;

/// Epsilon for floating-point comparisons (1 nanometer in mm).
pub const EPSILON: f64 = 1e-6;

/// Compare two floats within epsilon tolerance.
pub fn float_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

/// Compare two floats within a custom tolerance.
pub fn float_eq_tol(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

/// Round a coordinate to 6 decimal places (eliminates DXF floating-point noise).
pub fn clean_coordinate(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

/// Clean a Point2D by rounding both coordinates.
pub fn clean_point(p: &Point2D) -> Point2D {
    Point2D::new(clean_coordinate(p.x), clean_coordinate(p.y))
}

/// Euclidean distance between two points.
pub fn distance(a: &Point2D, b: &Point2D) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}

/// Squared distance (avoids sqrt for comparison purposes).
pub fn distance_squared(a: &Point2D, b: &Point2D) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    dx * dx + dy * dy
}

/// Midpoint between two points.
pub fn midpoint(a: &Point2D, b: &Point2D) -> Point2D {
    Point2D::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}

/// Angle from point a to point b in radians.
pub fn angle_between(a: &Point2D, b: &Point2D) -> f64 {
    (b.y - a.y).atan2(b.x - a.x)
}

/// Check if three points are collinear using the cross product (robust).
pub fn are_collinear(a: &Point2D, b: &Point2D, c: &Point2D) -> bool {
    let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    cross.abs() < EPSILON
}

/// Check if a point lies on a line segment (within epsilon).
pub fn point_on_line_segment(p: &Point2D, a: &Point2D, b: &Point2D) -> bool {
    let d_ap = distance(a, p);
    let d_pb = distance(p, b);
    let d_ab = distance(a, b);
    (d_ap + d_pb - d_ab).abs() < EPSILON
}

/// Normalize angle to [0, 2*PI) range.
pub fn normalize_angle(angle: f64) -> f64 {
    let two_pi = 2.0 * std::f64::consts::PI;
    let mut a = angle % two_pi;
    if a < 0.0 {
        a += two_pi;
    }
    a
}

/// Angular span of an arc from start to end angle (always positive, in [0, 2*PI]).
pub fn arc_span(start_angle: f64, end_angle: f64) -> f64 {
    normalize_angle(end_angle - start_angle)
}

/// Convert degrees to radians.
pub fn deg_to_rad(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

/// Convert radians to degrees.
pub fn rad_to_deg(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}

/// Compute the centroid of a set of points.
pub fn centroid(points: &[Point2D]) -> Point2D {
    if points.is_empty() {
        return Point2D::origin();
    }
    let n = points.len() as f64;
    let sum_x: f64 = points.iter().map(|p| p.x).sum();
    let sum_y: f64 = points.iter().map(|p| p.y).sum();
    Point2D::new(sum_x / n, sum_y / n)
}

/// Line length from start to end.
pub fn line_length(start: &Point2D, end: &Point2D) -> f64 {
    distance(start, end)
}

/// Arc length given radius and angular span.
pub fn arc_length(radius: f64, start_angle: f64, end_angle: f64) -> f64 {
    let span = normalize_angle(end_angle - start_angle);
    radius * span
}

/// Check if two line segments are parallel.
pub fn are_parallel(a1: &Point2D, a2: &Point2D, b1: &Point2D, b2: &Point2D) -> bool {
    let dx_a = a2.x - a1.x;
    let dy_a = a2.y - a1.y;
    let dx_b = b2.x - b1.x;
    let dy_b = b2.y - b1.y;
    let cross = dx_a * dy_b - dy_a * dx_b;
    cross.abs() < EPSILON
}

/// Perpendicular distance from a point to a line defined by two points.
pub fn point_to_line_distance(p: &Point2D, a: &Point2D, b: &Point2D) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < EPSILON {
        return distance(p, a);
    }
    ((p.x - a.x) * dy - (p.y - a.y) * dx).abs() / len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_eq() {
        assert!(float_eq(1.0, 1.0));
        assert!(float_eq(1.0, 1.0 + 1e-7));
        assert!(!float_eq(1.0, 1.001));
    }

    #[test]
    fn test_clean_coordinate() {
        let v = 25.000_000_123_456;
        let cleaned = clean_coordinate(v);
        assert!((cleaned - 25.0).abs() < 1e-6);
    }

    #[test]
    fn test_distance() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(3.0, 4.0);
        assert!((distance(&a, &b) - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_midpoint() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(10.0, 20.0);
        let m = midpoint(&a, &b);
        assert!(float_eq(m.x, 5.0));
        assert!(float_eq(m.y, 10.0));
    }

    #[test]
    fn test_collinear() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(5.0, 5.0);
        let c = Point2D::new(10.0, 10.0);
        assert!(are_collinear(&a, &b, &c));

        let d = Point2D::new(10.0, 11.0);
        assert!(!are_collinear(&a, &b, &d));
    }

    #[test]
    fn test_parallel_lines() {
        let a1 = Point2D::new(0.0, 0.0);
        let a2 = Point2D::new(10.0, 0.0);
        let b1 = Point2D::new(0.0, 5.0);
        let b2 = Point2D::new(10.0, 5.0);
        assert!(are_parallel(&a1, &a2, &b1, &b2));
    }

    #[test]
    fn test_point_to_line_distance() {
        let p = Point2D::new(0.0, 5.0);
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(10.0, 0.0);
        assert!((point_to_line_distance(&p, &a, &b) - 5.0).abs() < EPSILON);
    }
}
