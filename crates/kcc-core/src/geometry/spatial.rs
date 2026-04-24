use super::model::{Entity, EntityId, GeometryPrimitive, Point2D};
use super::utils;
use rstar::{AABB, PointDistance, RTree, RTreeObject};

/// Wrapper to make entities indexable by rstar.
#[derive(Debug, Clone)]
pub struct SpatialEntity {
    pub id: EntityId,
    pub envelope: AABB<[f64; 2]>,
    pub centroid: [f64; 2],
}

impl RTreeObject for SpatialEntity {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

impl PointDistance for SpatialEntity {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = self.centroid[0] - point[0];
        let dy = self.centroid[1] - point[1];
        dx * dx + dy * dy
    }
}

/// R-tree based spatial index for fast entity queries.
#[derive(Debug)]
pub struct SpatialIndex {
    tree: RTree<SpatialEntity>,
}

impl SpatialIndex {
    /// Build spatial index from a list of entities.
    pub fn build(entities: &[Entity]) -> Self {
        let spatial_entities: Vec<SpatialEntity> = entities
            .iter()
            .map(|e| {
                let (envelope, centroid) = compute_bounds(&e.geometry);
                SpatialEntity {
                    id: e.id,
                    envelope,
                    centroid,
                }
            })
            .collect();

        Self {
            tree: RTree::bulk_load(spatial_entities),
        }
    }

    /// Find all entities within a radius of a point.
    pub fn query_radius(&self, center: &Point2D, radius: f64) -> Vec<EntityId> {
        let min = [center.x - radius, center.y - radius];
        let max = [center.x + radius, center.y + radius];
        let envelope = AABB::from_corners(min, max);

        let r_sq = radius * radius;
        self.tree
            .locate_in_envelope_intersecting(&envelope)
            .filter(|e| {
                let dx = e.centroid[0] - center.x;
                let dy = e.centroid[1] - center.y;
                dx * dx + dy * dy <= r_sq
            })
            .map(|e| e.id)
            .collect()
    }

    /// Find all entities within a rectangle.
    pub fn query_rect(&self, min: &Point2D, max: &Point2D) -> Vec<EntityId> {
        let envelope = AABB::from_corners([min.x, min.y], [max.x, max.y]);
        self.tree
            .locate_in_envelope_intersecting(&envelope)
            .map(|e| e.id)
            .collect()
    }

    /// Find the N nearest entities to a point.
    pub fn nearest(&self, point: &Point2D, n: usize) -> Vec<EntityId> {
        self.tree
            .nearest_neighbor_iter(&[point.x, point.y])
            .take(n)
            .map(|e| e.id)
            .collect()
    }

    /// Find all entities within a radius, returning with distances.
    pub fn query_radius_with_distance(
        &self,
        center: &Point2D,
        radius: f64,
    ) -> Vec<(EntityId, f64)> {
        let min = [center.x - radius, center.y - radius];
        let max = [center.x + radius, center.y + radius];
        let envelope = AABB::from_corners(min, max);

        let r_sq = radius * radius;
        self.tree
            .locate_in_envelope_intersecting(&envelope)
            .filter_map(|e| {
                let dx = e.centroid[0] - center.x;
                let dy = e.centroid[1] - center.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= r_sq {
                    Some((e.id, dist_sq.sqrt()))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Compute the bounding box and centroid for a geometry primitive.
fn compute_bounds(geom: &GeometryPrimitive) -> (AABB<[f64; 2]>, [f64; 2]) {
    match geom {
        GeometryPrimitive::Line { start, end } => {
            let min_x = start.x.min(end.x);
            let min_y = start.y.min(end.y);
            let max_x = start.x.max(end.x);
            let max_y = start.y.max(end.y);
            let cx = (start.x + end.x) / 2.0;
            let cy = (start.y + end.y) / 2.0;
            (AABB::from_corners([min_x, min_y], [max_x, max_y]), [cx, cy])
        }
        GeometryPrimitive::Circle { center, radius } => (
            AABB::from_corners(
                [center.x - radius, center.y - radius],
                [center.x + radius, center.y + radius],
            ),
            [center.x, center.y],
        ),
        GeometryPrimitive::Arc {
            center,
            radius,
            start_angle,
            end_angle,
        } => {
            // Conservative bounding box for arc
            let bounds = arc_bounding_box(center, *radius, *start_angle, *end_angle);
            let cx = (bounds.0 + bounds.2) / 2.0;
            let cy = (bounds.1 + bounds.3) / 2.0;
            (
                AABB::from_corners([bounds.0, bounds.1], [bounds.2, bounds.3]),
                [cx, cy],
            )
        }
        GeometryPrimitive::Polyline { points, .. } => {
            if points.is_empty() {
                return (AABB::from_point([0.0, 0.0]), [0.0, 0.0]);
            }
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            for p in points {
                min_x = min_x.min(p.x);
                min_y = min_y.min(p.y);
                max_x = max_x.max(p.x);
                max_y = max_y.max(p.y);
                sum_x += p.x;
                sum_y += p.y;
            }
            let n = points.len() as f64;
            (
                AABB::from_corners([min_x, min_y], [max_x, max_y]),
                [sum_x / n, sum_y / n],
            )
        }
        GeometryPrimitive::Spline { control_points, .. } => {
            if control_points.is_empty() {
                return (AABB::from_point([0.0, 0.0]), [0.0, 0.0]);
            }
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            for p in control_points {
                min_x = min_x.min(p.x);
                min_y = min_y.min(p.y);
                max_x = max_x.max(p.x);
                max_y = max_y.max(p.y);
                sum_x += p.x;
                sum_y += p.y;
            }
            let n = control_points.len() as f64;
            (
                AABB::from_corners([min_x, min_y], [max_x, max_y]),
                [sum_x / n, sum_y / n],
            )
        }
        GeometryPrimitive::Point(p) => (AABB::from_point([p.x, p.y]), [p.x, p.y]),
    }
}

/// Compute precise bounding box for an arc considering axis crossings.
fn arc_bounding_box(
    center: &Point2D,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
) -> (f64, f64, f64, f64) {
    use std::f64::consts::PI;

    let sa = utils::normalize_angle(start_angle);
    let ea = utils::normalize_angle(end_angle);

    // Start and end points
    let x_start = center.x + radius * sa.cos();
    let y_start = center.y + radius * sa.sin();
    let x_end = center.x + radius * ea.cos();
    let y_end = center.y + radius * ea.sin();

    let mut min_x = x_start.min(x_end);
    let mut min_y = y_start.min(y_end);
    let mut max_x = x_start.max(x_end);
    let mut max_y = y_start.max(y_end);

    // Check if arc crosses each axis direction
    let check_angles = [0.0, PI / 2.0, PI, 3.0 * PI / 2.0];
    let extremes = [
        (radius, 0.0),  // 0: right
        (0.0, radius),  // PI/2: top
        (-radius, 0.0), // PI: left
        (0.0, -radius), // 3PI/2: bottom
    ];

    for (angle, (dx, dy)) in check_angles.iter().zip(extremes.iter()) {
        if arc_contains_angle(sa, ea, *angle) {
            let x = center.x + dx;
            let y = center.y + dy;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    (min_x, min_y, max_x, max_y)
}

/// Check if an arc from start_angle to end_angle (CCW) contains the given angle.
fn arc_contains_angle(start: f64, end: f64, angle: f64) -> bool {
    if start <= end {
        angle >= start && angle <= end
    } else {
        // Arc wraps around 2*PI
        angle >= start || angle <= end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_circle_entity(id: u64, cx: f64, cy: f64, r: f64) -> Entity {
        Entity {
            id: EntityId(id),
            geometry: GeometryPrimitive::Circle {
                center: Point2D::new(cx, cy),
                radius: r,
            },
            layer: "0".to_string(),
            color: None,
            lineweight: None,
            linetype: None,
            block_ref: None,
        }
    }

    #[test]
    fn test_build_and_query_radius() {
        let entities = vec![
            make_circle_entity(1, 0.0, 0.0, 5.0),
            make_circle_entity(2, 10.0, 0.0, 5.0),
            make_circle_entity(3, 100.0, 100.0, 5.0),
        ];
        let index = SpatialIndex::build(&entities);

        // Query near origin should find entity 1
        let results = index.query_radius(&Point2D::new(0.0, 0.0), 1.0);
        assert!(results.contains(&EntityId(1)));
        assert!(!results.contains(&EntityId(3)));
    }

    #[test]
    fn test_nearest() {
        let entities = vec![
            make_circle_entity(1, 0.0, 0.0, 5.0),
            make_circle_entity(2, 10.0, 0.0, 5.0),
            make_circle_entity(3, 100.0, 100.0, 5.0),
        ];
        let index = SpatialIndex::build(&entities);

        let nearest = index.nearest(&Point2D::new(9.0, 0.0), 1);
        assert_eq!(nearest.len(), 1);
        assert_eq!(nearest[0], EntityId(2));
    }
}
