use super::types::{Feature, FeatureId, FeatureType};
use crate::geometry::model::Point2D;
use crate::geometry::utils::{self, EPSILON};

/// Distance tolerance for pattern matching (mm).
const PATTERN_TOLERANCE: f64 = 0.5;

/// Angular tolerance for bolt circle spacing (radians).
const ANGULAR_TOLERANCE: f64 = 0.05;

/// Detect bolt circle patterns from a set of detected holes.
pub fn detect_bolt_circles(holes: &[Feature]) -> Vec<Feature> {
    let mut patterns = Vec::new();

    // Group holes by diameter
    let groups = group_by_diameter(holes);

    for group in &groups {
        if group.len() < 3 {
            continue;
        }

        // Try to fit a circle to the hole centers
        let centers: Vec<Point2D> = group.iter().map(|h| h.centroid).collect();

        if let Some((cx, cy, radius)) = fit_circle(&centers) {
            // Check how well the circle fits
            let max_error: f64 = centers
                .iter()
                .map(|c| (utils::distance(c, &Point2D::new(cx, cy)) - radius).abs())
                .fold(0.0_f64, f64::max);

            if max_error < PATTERN_TOLERANCE {
                // Check if holes are approximately equally spaced angularly
                if is_equally_spaced_angular(&centers, cx, cy) {
                    let hole_diameter = match &group[0].feature_type {
                        FeatureType::Hole { diameter, .. } => *diameter,
                        _ => 0.0,
                    };

                    let centroid = Point2D::new(cx, cy);
                    let geometry_refs: Vec<_> = group
                        .iter()
                        .flat_map(|h| h.geometry_refs.iter().copied())
                        .collect();

                    patterns.push(Feature {
                        id: FeatureId(0),
                        feature_type: FeatureType::BoltCircle {
                            hole_count: group.len(),
                            hole_diameter,
                            pattern_diameter: radius * 2.0,
                        },
                        geometry_refs,
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

    patterns
}

/// Detect linear patterns from features with equal spacing.
pub fn detect_linear_patterns(features: &[Feature]) -> Vec<Feature> {
    let mut patterns = Vec::new();

    // Group by feature type and approximate size
    let groups = group_by_type_and_size(features);

    for group in &groups {
        if group.len() < 3 {
            continue;
        }

        // Sort by x-coordinate first
        let mut sorted = group.clone();
        sorted.sort_by(|a, b| {
            a.centroid
                .x
                .partial_cmp(&b.centroid.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Check if collinear and equally spaced
        if sorted.len() >= 3 {
            let centers: Vec<Point2D> = sorted.iter().map(|f| f.centroid).collect();

            // Check collinearity
            let all_collinear = centers
                .windows(3)
                .all(|w| utils::are_collinear(&w[0], &w[1], &w[2]));

            if all_collinear {
                // Check equal spacing
                let spacings: Vec<f64> = centers
                    .windows(2)
                    .map(|w| utils::distance(&w[0], &w[1]))
                    .collect();

                let avg_spacing = spacings.iter().sum::<f64>() / spacings.len() as f64;
                let all_equal = spacings
                    .iter()
                    .all(|s| (s - avg_spacing).abs() < PATTERN_TOLERANCE);

                if all_equal && avg_spacing > EPSILON {
                    let dir_x = centers.last().unwrap().x - centers.first().unwrap().x;
                    let dir_y = centers.last().unwrap().y - centers.first().unwrap().y;
                    let dir_len = (dir_x * dir_x + dir_y * dir_y).sqrt();

                    let direction = if dir_len > EPSILON {
                        (dir_x / dir_len, dir_y / dir_len)
                    } else {
                        (1.0, 0.0)
                    };

                    let geometry_refs: Vec<_> = sorted
                        .iter()
                        .flat_map(|f| f.geometry_refs.iter().copied())
                        .collect();

                    patterns.push(Feature {
                        id: FeatureId(0),
                        feature_type: FeatureType::LinearPattern {
                            feature_count: sorted.len(),
                            spacing: avg_spacing,
                            direction,
                        },
                        geometry_refs,
                        centroid: utils::centroid(&centers),
                        dimensions: Vec::new(),
                        gdt_frames: Vec::new(),
                        datum_refs: Vec::new(),
                        layer_hint: None,
                    });
                }
            }
        }
    }

    patterns
}

/// Group holes by diameter (within tolerance).
fn group_by_diameter(holes: &[Feature]) -> Vec<Vec<&Feature>> {
    let mut groups: Vec<Vec<&Feature>> = Vec::new();

    for hole in holes {
        let dia = match &hole.feature_type {
            FeatureType::Hole { diameter, .. } => *diameter,
            _ => continue,
        };

        let mut added = false;
        for group in &mut groups {
            let group_dia = match &group[0].feature_type {
                FeatureType::Hole { diameter, .. } => *diameter,
                _ => continue,
            };

            if (dia - group_dia).abs() < PATTERN_TOLERANCE {
                group.push(hole);
                added = true;
                break;
            }
        }

        if !added {
            groups.push(vec![hole]);
        }
    }

    groups
}

/// Group features by type and approximate size.
fn group_by_type_and_size(features: &[Feature]) -> Vec<Vec<&Feature>> {
    let mut groups: Vec<Vec<&Feature>> = Vec::new();

    for feature in features {
        let key = feature.feature_type.name();

        let mut added = false;
        for group in &mut groups {
            if group[0].feature_type.name() == key {
                group.push(feature);
                added = true;
                break;
            }
        }

        if !added {
            groups.push(vec![feature]);
        }
    }

    groups
}

/// Algebraic least-squares circle fit. Returns (cx, cy, radius).
fn fit_circle(points: &[Point2D]) -> Option<(f64, f64, f64)> {
    let n = points.len();
    if n < 3 {
        return None;
    }

    // Use Kasa method: minimize algebraic distance
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_x3 = 0.0;
    let mut sum_y3 = 0.0;
    let mut sum_x2y = 0.0;
    let mut sum_xy2 = 0.0;

    for p in points {
        sum_x += p.x;
        sum_y += p.y;
        sum_x2 += p.x * p.x;
        sum_y2 += p.y * p.y;
        sum_xy += p.x * p.y;
        sum_x3 += p.x * p.x * p.x;
        sum_y3 += p.y * p.y * p.y;
        sum_x2y += p.x * p.x * p.y;
        sum_xy2 += p.x * p.y * p.y;
    }

    let nf = n as f64;
    let a = nf * sum_x2 - sum_x * sum_x;
    let b = nf * sum_xy - sum_x * sum_y;
    let c = nf * sum_y2 - sum_y * sum_y;
    let d = 0.5 * (nf * (sum_x3 + sum_xy2) - sum_x * (sum_x2 + sum_y2));
    let e = 0.5 * (nf * (sum_x2y + sum_y3) - sum_y * (sum_x2 + sum_y2));

    let denom = a * c - b * b;
    if denom.abs() < EPSILON {
        return None;
    }

    let cx = (d * c - b * e) / denom;
    let cy = (a * e - b * d) / denom;

    let radius = points
        .iter()
        .map(|p| utils::distance(p, &Point2D::new(cx, cy)))
        .sum::<f64>()
        / nf;

    if radius < EPSILON {
        return None;
    }

    Some((cx, cy, radius))
}

/// Check if points are approximately equally spaced around a circle.
fn is_equally_spaced_angular(centers: &[Point2D], cx: f64, cy: f64) -> bool {
    let n = centers.len();
    if n < 3 {
        return false;
    }

    let expected_spacing = 2.0 * std::f64::consts::PI / n as f64;

    let mut angles: Vec<f64> = centers
        .iter()
        .map(|c| (c.y - cy).atan2(c.x - cx))
        .map(utils::normalize_angle)
        .collect();

    angles.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Check spacing between consecutive angles
    for i in 0..n {
        let next = (i + 1) % n;
        let mut spacing = angles[next] - angles[i];
        if spacing < 0.0 {
            spacing += 2.0 * std::f64::consts::PI;
        }

        if (spacing - expected_spacing).abs() > ANGULAR_TOLERANCE {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::model::EntityId;

    fn make_hole(id: u64, cx: f64, cy: f64, dia: f64) -> Feature {
        Feature {
            id: FeatureId(id),
            feature_type: FeatureType::Hole {
                diameter: dia,
                is_through: false,
            },
            geometry_refs: vec![EntityId(id)],
            centroid: Point2D::new(cx, cy),
            dimensions: Vec::new(),
            gdt_frames: Vec::new(),
            datum_refs: Vec::new(),
            layer_hint: None,
        }
    }

    #[test]
    fn test_bolt_circle_detection() {
        let r = 40.0; // PCD radius
        let holes: Vec<Feature> = (0..6)
            .map(|i| {
                let angle = i as f64 * std::f64::consts::PI / 3.0;
                make_hole(i as u64, r * angle.cos(), r * angle.sin(), 10.0)
            })
            .collect();

        let patterns = detect_bolt_circles(&holes);
        assert_eq!(patterns.len(), 1);

        if let FeatureType::BoltCircle {
            hole_count,
            hole_diameter,
            pattern_diameter,
        } = &patterns[0].feature_type
        {
            assert_eq!(*hole_count, 6);
            assert!((hole_diameter - 10.0).abs() < 0.1);
            assert!((pattern_diameter - 80.0).abs() < 1.0);
        } else {
            panic!("Expected bolt circle");
        }
    }

    #[test]
    fn test_circle_fit() {
        let r = 50.0;
        let points: Vec<Point2D> = (0..8)
            .map(|i| {
                let angle = i as f64 * std::f64::consts::PI / 4.0;
                Point2D::new(r * angle.cos(), r * angle.sin())
            })
            .collect();

        let (cx, cy, radius) = fit_circle(&points).unwrap();
        assert!(cx.abs() < 0.1);
        assert!(cy.abs() < 0.1);
        assert!((radius - 50.0).abs() < 0.1);
    }
}
