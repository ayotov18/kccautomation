use crate::geometry::model::{Drawing, EntityId, GeometryPrimitive, Point2D};
use crate::geometry::utils;
use std::collections::{HashMap, HashSet};

/// A closed contour found in the drawing.
#[derive(Debug, Clone)]
pub struct Contour {
    pub entity_ids: Vec<EntityId>,
    pub points: Vec<Point2D>,
    pub is_outer: bool,
    pub area: f64,
}

/// Endpoint connectivity tolerance for building contour graph.
const CONNECT_TOLERANCE: f64 = 0.1; // mm

/// Max segments before we skip contour detection entirely (O(n) guard).
const MAX_SEGMENTS: usize = 5_000;

/// Max DFS iterations before aborting cycle search.
const MAX_CYCLE_ITERATIONS: usize = 200_000;

/// Find closed contours by building an endpoint graph and finding cycles.
pub fn find_closed_contours(drawing: &Drawing) -> Vec<Contour> {
    let segments = extract_segments(drawing);

    if segments.is_empty() {
        return Vec::new();
    }

    // Guard: skip contour detection on very large drawings to avoid hangs
    if segments.len() > MAX_SEGMENTS {
        tracing::warn!(
            segment_count = segments.len(),
            "Skipping contour detection: too many segments (limit: {MAX_SEGMENTS})"
        );
        return Vec::new();
    }

    // Build adjacency graph using spatial hashing (O(n) average, not O(n²))
    let graph = build_adjacency_graph_spatial(&segments);

    // Find closed cycles using bounded DFS
    let cycles = find_cycles_bounded(&graph, &segments);

    // Convert cycles to contours with area computation
    let mut contours: Vec<Contour> = cycles
        .into_iter()
        .filter_map(|cycle| {
            let points: Vec<Point2D> = cycle
                .iter()
                .filter_map(|idx| segments.get(*idx).map(|s| s.start))
                .collect();

            if points.len() < 3 {
                return None;
            }

            let area = compute_signed_area(&points);
            let entity_ids: Vec<EntityId> = cycle
                .iter()
                .filter_map(|idx| segments.get(*idx).map(|s| s.entity_id))
                .collect();

            Some(Contour {
                entity_ids,
                points,
                is_outer: area > 0.0,
                area: area.abs(),
            })
        })
        .collect();

    // Sort by area descending — largest is the outer boundary
    contours.sort_by(|a, b| {
        b.area
            .partial_cmp(&a.area)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Mark the largest as outer
    if let Some(first) = contours.first_mut() {
        first.is_outer = true;
    }
    for contour in contours.iter_mut().skip(1) {
        contour.is_outer = false;
    }

    contours
}

struct Segment {
    entity_id: EntityId,
    start: Point2D,
    end: Point2D,
}

fn extract_segments(drawing: &Drawing) -> Vec<Segment> {
    let mut segments = Vec::new();

    for entity in &drawing.entities {
        match &entity.geometry {
            GeometryPrimitive::Line { start, end } => {
                segments.push(Segment {
                    entity_id: entity.id,
                    start: *start,
                    end: *end,
                });
            }
            GeometryPrimitive::Arc {
                center,
                radius,
                start_angle,
                end_angle,
            } => {
                let start = Point2D::new(
                    center.x + radius * start_angle.cos(),
                    center.y + radius * start_angle.sin(),
                );
                let end = Point2D::new(
                    center.x + radius * end_angle.cos(),
                    center.y + radius * end_angle.sin(),
                );
                segments.push(Segment {
                    entity_id: entity.id,
                    start,
                    end,
                });
            }
            _ => {}
        }
    }

    segments
}

/// Hash a point into a grid cell for spatial lookup.
fn grid_key(p: &Point2D, cell_size: f64) -> (i64, i64) {
    (
        (p.x / cell_size).floor() as i64,
        (p.y / cell_size).floor() as i64,
    )
}

/// Build adjacency graph using a spatial hash grid.
/// O(n) average instead of O(n²) brute force.
fn build_adjacency_graph_spatial(segments: &[Segment]) -> HashMap<usize, Vec<usize>> {
    let cell_size = CONNECT_TOLERANCE * 2.0;
    let mut graph: HashMap<usize, Vec<usize>> = HashMap::new();

    // Index: grid cell → list of (segment_index, is_start_point)
    let mut grid: HashMap<(i64, i64), Vec<(usize, bool)>> = HashMap::new();

    // Insert all segment endpoints into the grid
    for (i, seg) in segments.iter().enumerate() {
        for is_start in [true, false] {
            let pt = if is_start { &seg.start } else { &seg.end };
            let key = grid_key(pt, cell_size);
            // Insert into this cell and all 8 neighbors to handle boundary cases
            for dx in -1..=1 {
                for dy in -1..=1 {
                    grid.entry((key.0 + dx, key.1 + dy))
                        .or_default()
                        .push((i, is_start));
                }
            }
        }
    }

    // For each segment endpoint, find nearby endpoints from other segments
    for (i, seg) in segments.iter().enumerate() {
        for is_start in [true, false] {
            let pt = if is_start { &seg.start } else { &seg.end };
            let key = grid_key(pt, cell_size);

            if let Some(candidates) = grid.get(&key) {
                for &(j, _j_is_start) in candidates {
                    if j <= i {
                        continue; // avoid duplicates and self
                    }
                    let other = &segments[j];
                    let connects = utils::distance(pt, &other.start) < CONNECT_TOLERANCE
                        || utils::distance(pt, &other.end) < CONNECT_TOLERANCE;

                    if connects {
                        graph.entry(i).or_default().push(j);
                        graph.entry(j).or_default().push(i);
                    }
                }
            }
        }
    }

    // Deduplicate neighbor lists
    for neighbors in graph.values_mut() {
        neighbors.sort_unstable();
        neighbors.dedup();
    }

    graph
}

/// Find cycles with a hard iteration limit to prevent combinatorial explosion.
fn find_cycles_bounded(
    graph: &HashMap<usize, Vec<usize>>,
    _segments: &[Segment],
) -> Vec<Vec<usize>> {
    let mut cycles = Vec::new();
    let mut visited_global: HashSet<usize> = HashSet::new();
    let mut total_iterations: usize = 0;

    let mut sorted_keys: Vec<usize> = graph.keys().copied().collect();
    sorted_keys.sort_unstable();

    for start_node in sorted_keys {
        if visited_global.contains(&start_node) {
            continue;
        }

        if total_iterations >= MAX_CYCLE_ITERATIONS {
            tracing::warn!(
                iterations = total_iterations,
                cycles_found = cycles.len(),
                "Cycle search hit iteration limit, returning partial results"
            );
            break;
        }

        // Simple DFS to find one cycle from this node
        let mut stack = vec![(start_node, vec![start_node], HashSet::from([start_node]))];
        let mut found_cycle = false;

        while let Some((current, path, visited)) = stack.pop() {
            total_iterations += 1;

            if total_iterations >= MAX_CYCLE_ITERATIONS {
                break;
            }

            if path.len() > 20 {
                continue; // Limit cycle length
            }

            if let Some(neighbors) = graph.get(&current) {
                for &next in neighbors {
                    if next == start_node && path.len() >= 3 {
                        // Found a cycle
                        cycles.push(path.clone());
                        for &node in &path {
                            visited_global.insert(node);
                        }
                        found_cycle = true;
                        break;
                    }

                    if !visited.contains(&next) {
                        let mut new_path = path.clone();
                        new_path.push(next);
                        let mut new_visited = visited.clone();
                        new_visited.insert(next);
                        stack.push((next, new_path, new_visited));
                    }
                }
            }

            if found_cycle {
                break;
            }
        }
    }

    cycles
}

/// Compute signed area using the shoelace formula.
/// Positive = CCW (outer), Negative = CW (inner/hole).
fn compute_signed_area(points: &[Point2D]) -> f64 {
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

    area / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed_area_rectangle() {
        // CCW rectangle 100x50
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(100.0, 0.0),
            Point2D::new(100.0, 50.0),
            Point2D::new(0.0, 50.0),
        ];
        let area = compute_signed_area(&points);
        assert!((area - 5000.0).abs() < 1e-6);
    }
}
