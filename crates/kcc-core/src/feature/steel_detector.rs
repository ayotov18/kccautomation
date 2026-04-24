use super::types::{Feature, FeatureId, FeatureType};
use crate::geometry::model::{Drawing, EntityId, GeometryPrimitive, Point2D};
use crate::geometry::spatial::SpatialIndex;
use crate::geometry::utils;

/// Detect structural steel features: members, plates, bolt groups, weld preps.
///
/// Structural steel drawings use a different visual vocabulary than machined
/// parts: I-beams, channels, angles, gusset plates, bolt groups at specific
/// pitch/gauge. This detector identifies these from geometry patterns.
pub fn detect_steel_features(drawing: &Drawing, _index: &SpatialIndex) -> Vec<Feature> {
    let mut features = Vec::new();

    // 1. Detect steel members from parallel line pairs (flanges/webs)
    let members = detect_steel_members(drawing);
    features.extend(members);

    // 2. Detect gusset plates from closed polygonal regions
    let plates = detect_gusset_plates(drawing);
    features.extend(plates);

    // 3. Detect bolt groups from clustered holes
    let bolt_groups = detect_bolt_groups(drawing);
    features.extend(bolt_groups);

    // 4. Extract member dimensions from annotations
    let annotated = detect_annotated_members(drawing);
    features.extend(annotated);

    features
}

/// Detect steel members by finding parallel line pairs that form I-beam webs/flanges.
/// In 2D steel drawings, members appear as pairs of parallel lines (the flanges)
/// connected by shorter perpendicular lines (stiffeners/webs).
fn detect_steel_members(drawing: &Drawing) -> Vec<Feature> {
    let mut features = Vec::new();

    // Collect all lines with their lengths
    let lines: Vec<(EntityId, Point2D, Point2D, f64, String)> = drawing
        .entities
        .iter()
        .filter_map(|e| {
            if let GeometryPrimitive::Line { start, end } = &e.geometry {
                let len = utils::distance(start, end);
                if len > 50.0 {
                    // Only consider lines > 50 units as potential members
                    Some((e.id, *start, *end, len, e.layer.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Find pairs of approximately parallel lines with consistent spacing
    let mut used = std::collections::HashSet::new();
    for i in 0..lines.len() {
        if used.contains(&i) {
            continue;
        }
        for j in (i + 1)..lines.len() {
            if used.contains(&j) {
                continue;
            }

            let (id_a, a1, a2, len_a, ref layer_a) = lines[i];
            let (id_b, b1, b2, len_b, _) = lines[j];

            // Similar length (within 20%)
            let len_ratio = len_a.min(len_b) / len_a.max(len_b);
            if len_ratio < 0.8 {
                continue;
            }

            // Check parallelism
            if !utils::are_parallel(&a1, &a2, &b1, &b2) {
                continue;
            }

            // Check spacing (typical steel member depth: 100-1000mm)
            let spacing = utils::point_to_line_distance(&b1, &a1, &a2);
            if !(50.0..=1500.0).contains(&spacing) {
                continue;
            }

            // This looks like a steel member (two parallel flanges)
            let mid = Point2D::new(
                (a1.x + a2.x + b1.x + b2.x) / 4.0,
                (a1.y + a2.y + b1.y + b2.y) / 4.0,
            );

            features.push(Feature {
                id: FeatureId(0),
                feature_type: FeatureType::SteelMember {
                    length: len_a,
                    depth: spacing,
                    profile_hint: guess_profile(spacing, &drawing.annotations, &mid),
                },
                geometry_refs: vec![id_a, id_b],
                centroid: mid,
                dimensions: Vec::new(),
                gdt_frames: Vec::new(),
                datum_refs: Vec::new(),
                layer_hint: Some(layer_a.clone()),
            });

            used.insert(i);
            used.insert(j);
            break;
        }
    }

    features
}

/// Detect gusset plates from closed polylines (triangular or trapezoidal shapes).
fn detect_gusset_plates(drawing: &Drawing) -> Vec<Feature> {
    let mut features = Vec::new();

    for entity in &drawing.entities {
        if let GeometryPrimitive::Polyline {
            points,
            closed: true,
            ..
        } = &entity.geometry
        {
            // Closed polylines with 3-6 vertices are likely gusset plates
            if points.len() >= 3 && points.len() <= 6 {
                let area = compute_polygon_area(points);
                // Filter by reasonable plate area (> 100 mm², < 10 m²)
                if area > 100.0 && area < 10_000_000.0 {
                    let centroid = utils::centroid(points);
                    features.push(Feature {
                        id: FeatureId(0),
                        feature_type: FeatureType::GussetPlate {
                            area,
                            vertex_count: points.len(),
                        },
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

    features
}

/// Detect bolt groups: clusters of holes at regular spacing.
fn detect_bolt_groups(drawing: &Drawing) -> Vec<Feature> {
    let mut features = Vec::new();

    // Collect all circles (potential bolt holes)
    let holes: Vec<(EntityId, Point2D, f64)> = drawing
        .entities
        .iter()
        .filter_map(|e| {
            if let GeometryPrimitive::Circle { center, radius } = &e.geometry {
                if *radius >= 4.0 && *radius <= 30.0 {
                    // Typical bolt hole range: M8 (4mm) to M56 (30mm)
                    Some((e.id, *center, *radius))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if holes.len() < 2 {
        return features;
    }

    // Group holes by similar radius (same bolt size)
    let mut assigned = vec![false; holes.len()];

    for i in 0..holes.len() {
        if assigned[i] {
            continue;
        }
        let mut group = vec![i];
        assigned[i] = true;

        for j in (i + 1)..holes.len() {
            if assigned[j] {
                continue;
            }
            // Same radius (within 5%)
            let r_diff = (holes[i].2 - holes[j].2).abs();
            if r_diff < holes[i].2 * 0.05 {
                // Within reasonable bolt group distance (< 500mm between any two)
                let dist = utils::distance(&holes[i].1, &holes[j].1);
                if dist < 500.0 {
                    group.push(j);
                    assigned[j] = true;
                }
            }
        }

        if group.len() >= 2 {
            let ids: Vec<EntityId> = group.iter().map(|&idx| holes[idx].0).collect();
            let centers: Vec<Point2D> = group.iter().map(|&idx| holes[idx].1).collect();
            let centroid = utils::centroid(&centers);
            let diameter = holes[group[0]].2 * 2.0;

            features.push(Feature {
                id: FeatureId(0),
                feature_type: FeatureType::BoltGroup {
                    bolt_count: group.len(),
                    bolt_diameter: diameter,
                    group_width: bbox_width(&centers),
                    group_height: bbox_height(&centers),
                },
                geometry_refs: ids,
                centroid,
                dimensions: Vec::new(),
                gdt_frames: Vec::new(),
                datum_refs: Vec::new(),
                layer_hint: None,
            });
        }
    }

    features
}

/// Detect steel members from dimension annotations containing profile designations.
/// Uses 3-gate filter to avoid title block false positives:
/// 1. Excluded layers (title block, border, notes)
/// 2. Strict profile regex (must match standard designation pattern)
/// 3. Spatial proximity to detected geometry (must be near real drawing entities)
fn detect_annotated_members(drawing: &Drawing) -> Vec<Feature> {
    let mut features = Vec::new();

    // Compute drawing geometry centroid and bounds for proximity check
    let (geo_cx, geo_cy, geo_radius) = if !drawing.entities.is_empty() {
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut count = 0.0;
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        for e in &drawing.entities {
            let pts: Vec<Point2D> = match &e.geometry {
                GeometryPrimitive::Line { start, end } => vec![*start, *end],
                GeometryPrimitive::Circle { center, .. } => vec![*center],
                GeometryPrimitive::Arc { center, .. } => vec![*center],
                _ => continue,
            };
            for p in pts {
                if p.x.is_finite() && p.y.is_finite() {
                    sum_x += p.x;
                    sum_y += p.y;
                    count += 1.0;
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }
            }
        }
        if count > 0.0 {
            let cx = sum_x / count;
            let cy = sum_y / count;
            let dx = max_x - min_x;
            let dy = max_y - min_y;
            let radius = (dx * dx + dy * dy).sqrt() / 2.0;
            (cx, cy, radius)
        } else {
            (0.0, 0.0, 1000.0)
        }
    } else {
        (0.0, 0.0, 1000.0)
    };

    for ann in &drawing.annotations {
        // Gate 1: Exclude title block / border layers
        let layer_lower = ann.layer.to_lowercase();
        if layer_lower.contains("title")
            || layer_lower.contains("border")
            || layer_lower.contains("frame")
            || layer_lower.contains("notes")
            || layer_lower.contains("legend")
            || layer_lower.contains("defpoints")
        {
            continue;
        }

        // Gate 2: Strict profile regex — must be a plausible steel designation
        let text = ann.text.trim().to_uppercase();
        let is_profile = (text.starts_with("IPE") && text.len() <= 8 && text.chars().skip(3).all(|c| c.is_ascii_digit() || c == ' '))
            || (text.starts_with("HEA") && text.len() <= 8 && text.chars().skip(3).all(|c| c.is_ascii_digit() || c == ' '))
            || (text.starts_with("HEB") && text.len() <= 8 && text.chars().skip(3).all(|c| c.is_ascii_digit() || c == ' '))
            || (text.starts_with("HEM") && text.len() <= 8 && text.chars().skip(3).all(|c| c.is_ascii_digit() || c == ' '))
            || (text.starts_with("UPN") && text.len() <= 8 && text.chars().skip(3).all(|c| c.is_ascii_digit() || c == ' '))
            || (text.starts_with("L") && text.contains("X") && text.len() <= 15
                && text.chars().filter(|c| c.is_ascii_digit()).count() >= 2
                && !text.contains('|') && !text.contains(':'));

        if !is_profile {
            continue;
        }

        // Gate 3: Must be within the drawing geometry bounds (not off in title block corner)
        let dist = ((ann.position.x - geo_cx).powi(2) + (ann.position.y - geo_cy).powi(2)).sqrt();
        if dist > geo_radius * 1.5 {
            continue;
        }

        features.push(Feature {
            id: FeatureId(0),
            feature_type: FeatureType::SteelMember {
                length: 0.0,
                depth: 0.0,
                profile_hint: Some(ann.text.trim().to_string()),
            },
            geometry_refs: Vec::new(),
            centroid: ann.position,
            dimensions: Vec::new(),
            gdt_frames: Vec::new(),
            datum_refs: Vec::new(),
            layer_hint: Some(ann.layer.clone()),
        });
    }

    features
}

/// Guess a profile designation from nearby annotations.
fn guess_profile(
    _depth: f64,
    annotations: &[crate::geometry::model::Annotation],
    centroid: &Point2D,
) -> Option<String> {
    for ann in annotations {
        let dist = utils::distance(&ann.position, centroid);
        if dist < 200.0 {
            let text = ann.text.to_uppercase();
            if text.starts_with("IPE")
                || text.starts_with("HEA")
                || text.starts_with("HEB")
                || text.starts_with("UPN")
            {
                return Some(ann.text.clone());
            }
        }
    }
    None
}

fn compute_polygon_area(points: &[Point2D]) -> f64 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].x * points[j].y;
        area -= points[j].x * points[i].y;
    }
    area.abs() / 2.0
}

fn bbox_width(points: &[Point2D]) -> f64 {
    let min_x = points.iter().map(|p| p.x).fold(f64::MAX, f64::min);
    let max_x = points.iter().map(|p| p.x).fold(f64::MIN, f64::max);
    max_x - min_x
}

fn bbox_height(points: &[Point2D]) -> f64 {
    let min_y = points.iter().map(|p| p.y).fold(f64::MAX, f64::min);
    let max_y = points.iter().map(|p| p.y).fold(f64::MIN, f64::max);
    max_y - min_y
}
