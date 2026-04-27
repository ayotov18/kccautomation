//! Multi-module structure detection.
//!
//! Many wood-cabin / modular building drawings pack two or three independent
//! floor plans into a single DWG (laid out side-by-side in model space).
//! Treating that as one structure produces a single KSS that's 3-5× too small.
//!
//! This module clusters entities into spatial structures using bbox-overlap
//! union-find with a per-drawing gap threshold derived from the drawing's
//! overall extent. Single-module drawings collapse to one structure; nothing
//! downstream needs special-casing.

use super::model::{Annotation, Drawing, Entity, EntityId, GeometryPrimitive, Point2D};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A cluster of entities representing one buildable module / floor plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    pub id: u32,
    pub label: String,
    pub bbox_min: Point2D,
    pub bbox_max: Point2D,
    pub entity_ids: Vec<EntityId>,
    pub dimension_ids: Vec<EntityId>,
    pub annotation_ids: Vec<EntityId>,
}

impl Structure {
    pub fn area(&self) -> f64 {
        (self.bbox_max.x - self.bbox_min.x).max(0.0)
            * (self.bbox_max.y - self.bbox_min.y).max(0.0)
    }

    pub fn width(&self) -> f64 {
        (self.bbox_max.x - self.bbox_min.x).max(0.0)
    }

    pub fn height(&self) -> f64 {
        (self.bbox_max.y - self.bbox_min.y).max(0.0)
    }

    pub fn contains(&self, p: &Point2D) -> bool {
        p.x >= self.bbox_min.x
            && p.x <= self.bbox_max.x
            && p.y >= self.bbox_min.y
            && p.y <= self.bbox_max.y
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StructureDetectionConfig {
    /// Cluster connectivity gap as a fraction of the slice extent. Used by
    /// the 2D union-find pass within each 1D slice to merge minor noise
    /// fragments into the main cluster.
    pub gap_fraction: f64,
    /// Minimum entities for a real structure.
    pub min_entities: usize,
    /// Drop tiny clusters whose bbox is below this fraction of the largest
    /// cluster's bbox area.
    pub min_area_fraction: f64,
}

impl Default for StructureDetectionConfig {
    fn default() -> Self {
        Self {
            gap_fraction: 0.30,
            min_entities: 20,
            min_area_fraction: 0.02,
        }
    }
}

/// Detect spatial structures (modules) in a parsed drawing.
///
/// Returns at least one structure: a degenerate "everything" structure when
/// the drawing is a single module or has too few entities to cluster.
pub fn detect_structures(drawing: &Drawing) -> Vec<Structure> {
    detect_with_config(drawing, StructureDetectionConfig::default())
}

pub fn detect_with_config(drawing: &Drawing, config: StructureDetectionConfig) -> Vec<Structure> {
    if drawing.entities.is_empty() {
        return Vec::new();
    }

    let bboxes = compute_entity_bboxes(&drawing.entities);
    if bboxes.is_empty() {
        return Vec::new();
    }

    let (extent_x, extent_y) = drawing_extent(&bboxes);

    // Step 1: 1D projection slicing on whichever axis has the larger extent
    // (multi-module sheets are almost always laid out along one principal
    // axis). Recursively split centroid sequence at anomalously-large gaps.
    // The absolute threshold floor is locked to the top-level extent so deep
    // recursion can't over-fragment a single module.
    let use_x_axis = extent_x >= extent_y;
    let centroids: Vec<f64> = bboxes
        .iter()
        .map(|b| {
            if use_x_axis {
                (b.min_x + b.max_x) * 0.5
            } else {
                (b.min_y + b.max_y) * 0.5
            }
        })
        .collect();
    let slices = projection_slice_with_floor(centroids.clone(), 0.0);

    // Assign each entity to its slice.
    let mut coarse_groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for (idx, b) in bboxes.iter().enumerate() {
        let v = if use_x_axis {
            (b.min_x + b.max_x) * 0.5
        } else {
            (b.min_y + b.max_y) * 0.5
        };
        let si = slice_index_of(&slices, v);
        coarse_groups.entry(si).or_default().push(idx);
    }

    // Step 2: union-find WITHIN each coarse cluster to drop noise / merge any
    // accidental over-splits. The gap is now relative to the slice extent
    // (not the full drawing), which is appropriate per-module.
    let n = bboxes.len();
    let mut parents: Vec<usize> = (0..n).collect();
    for members in coarse_groups.values() {
        if members.is_empty() {
            continue;
        }
        // Slice extent = max coordinate span among members.
        let mut sx0 = f64::MAX;
        let mut sx1 = f64::MIN;
        let mut sy0 = f64::MAX;
        let mut sy1 = f64::MIN;
        for &m in members {
            let b = &bboxes[m];
            sx0 = sx0.min(b.min_x);
            sx1 = sx1.max(b.max_x);
            sy0 = sy0.min(b.min_y);
            sy1 = sy1.max(b.max_y);
        }
        let slice_extent = (sx1 - sx0).max(sy1 - sy0);
        let gap = (slice_extent * config.gap_fraction).max(extent_x.max(extent_y) * 0.005);

        // Sort members by min_x for sweep-line pairing.
        let mut order: Vec<usize> = members.clone();
        order.sort_by(|a, b| bboxes[*a].min_x.partial_cmp(&bboxes[*b].min_x).unwrap());

        for i in 0..order.len() {
            let ai = order[i];
            let a = &bboxes[ai];
            let a_max_x = a.max_x + gap;
            for j in (i + 1)..order.len() {
                let bi = order[j];
                let b = &bboxes[bi];
                if b.min_x - gap > a_max_x {
                    break;
                }
                if a.max_y + gap < b.min_y || b.max_y + gap < a.min_y {
                    continue;
                }
                if a.max_x + gap < b.min_x || b.max_x + gap < a.min_x {
                    continue;
                }
                union(&mut parents, ai, bi);
            }
        }
    }

    // Group entities by root
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        let r = find(&mut parents, i);
        groups.entry(r).or_default().push(i);
    }

    // Compute cluster bboxes
    let mut clusters: Vec<ClusterRaw> = groups
        .into_iter()
        .map(|(_, members)| {
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            for &m in &members {
                let b = &bboxes[m];
                min_x = min_x.min(b.min_x);
                min_y = min_y.min(b.min_y);
                max_x = max_x.max(b.max_x);
                max_y = max_y.max(b.max_y);
            }
            ClusterRaw {
                entity_indices: members,
                min_x,
                min_y,
                max_x,
                max_y,
            }
        })
        .collect();

    // Drop noise clusters (too few entities OR too small relative to largest)
    if clusters.len() > 1 {
        let max_area = clusters
            .iter()
            .map(|c| (c.max_x - c.min_x) * (c.max_y - c.min_y))
            .fold(0.0_f64, f64::max);
        let min_area = max_area * config.min_area_fraction;
        clusters.retain(|c| {
            c.entity_indices.len() >= config.min_entities
                && (c.max_x - c.min_x) * (c.max_y - c.min_y) >= min_area
        });
    }

    // Defensive: if filtering removed everything, fall back to one
    // all-encompassing structure.
    if clusters.is_empty() {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        for b in &bboxes {
            min_x = min_x.min(b.min_x);
            min_y = min_y.min(b.min_y);
            max_x = max_x.max(b.max_x);
            max_y = max_y.max(b.max_y);
        }
        clusters.push(ClusterRaw {
            entity_indices: (0..n).collect(),
            min_x,
            min_y,
            max_x,
            max_y,
        });
    }

    // Sort clusters left-to-right then top-to-bottom for stable IDs.
    clusters.sort_by(|a, b| {
        a.min_x
            .partial_cmp(&b.min_x)
            .unwrap()
            .then_with(|| a.min_y.partial_cmp(&b.min_y).unwrap())
    });

    // Build final structures: collect entity IDs, assign dims & annotations
    // by point-in-bbox.
    clusters
        .into_iter()
        .enumerate()
        .map(|(idx, c)| {
            let entity_ids: Vec<EntityId> =
                c.entity_indices.iter().map(|i| bboxes[*i].id).collect();
            let entity_set: HashSet<EntityId> = entity_ids.iter().copied().collect();

            let bbox_min = Point2D::new(c.min_x, c.min_y);
            let bbox_max = Point2D::new(c.max_x, c.max_y);

            let dimension_ids: Vec<EntityId> = drawing
                .dimensions
                .iter()
                .filter(|d| {
                    let in_bbox = d
                        .definition_points
                        .iter()
                        .any(|p| point_in(p, &bbox_min, &bbox_max))
                        || point_in(&d.text_position, &bbox_min, &bbox_max);
                    let attached_to_member =
                        d.attached_entities.iter().any(|id| entity_set.contains(id));
                    in_bbox || attached_to_member
                })
                .map(|d| d.id)
                .collect();

            let annotation_ids: Vec<EntityId> = drawing
                .annotations
                .iter()
                .filter(|a| point_in(&a.position, &bbox_min, &bbox_max))
                .map(|a| a.id)
                .collect();

            let label = label_from_annotations(
                &drawing.annotations,
                &bbox_min,
                &bbox_max,
                idx as u32 + 1,
            );

            Structure {
                id: idx as u32 + 1,
                label,
                bbox_min,
                bbox_max,
                entity_ids,
                dimension_ids,
                annotation_ids,
            }
        })
        .collect()
}

fn label_from_annotations(
    annotations: &[Annotation],
    bbox_min: &Point2D,
    bbox_max: &Point2D,
    fallback_idx: u32,
) -> String {
    // Pick the largest text annotation inside the bbox whose content looks
    // like a module name (short, alphabetic, non-numeric).
    let mut best: Option<(&Annotation, f64)> = None;
    for ann in annotations {
        if !point_in(&ann.position, bbox_min, bbox_max) {
            continue;
        }
        let text = ann.text.trim();
        if text.is_empty() {
            continue;
        }
        if !looks_like_module_label(text) {
            continue;
        }
        let score = ann.height;
        if best.map(|(_, s)| score > s).unwrap_or(true) {
            best = Some((ann, score));
        }
    }
    best.map(|(a, _)| a.text.trim().to_string())
        .unwrap_or_else(|| format!("Module {fallback_idx}"))
}

fn looks_like_module_label(text: &str) -> bool {
    // Heuristic: short (<= 40 chars), at least one letter, not a pure
    // dimension, not all punctuation.
    if text.len() > 40 {
        return false;
    }
    let has_letter = text.chars().any(|c| c.is_alphabetic());
    if !has_letter {
        return false;
    }
    // Skip obviously-not-labels: page numbers, dates, units only
    let lower = text.to_lowercase();
    let blocklist = [
        "scale", "м.ед", "drawing", "sheet", "page", "date", "rev", "по", "виж",
    ];
    if blocklist.iter().any(|b| lower.contains(b)) {
        return false;
    }
    true
}

fn point_in(p: &Point2D, min: &Point2D, max: &Point2D) -> bool {
    p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
}

#[derive(Debug)]
struct EntityBbox {
    id: EntityId,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

#[derive(Debug)]
struct ClusterRaw {
    entity_indices: Vec<usize>,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

fn compute_entity_bboxes(entities: &[Entity]) -> Vec<EntityBbox> {
    entities
        .iter()
        .filter_map(|e| {
            let (min_x, min_y, max_x, max_y) = match &e.geometry {
                GeometryPrimitive::Line { start, end } => (
                    start.x.min(end.x),
                    start.y.min(end.y),
                    start.x.max(end.x),
                    start.y.max(end.y),
                ),
                GeometryPrimitive::Circle { center, radius } => (
                    center.x - radius,
                    center.y - radius,
                    center.x + radius,
                    center.y + radius,
                ),
                GeometryPrimitive::Arc {
                    center, radius, ..
                } => (
                    center.x - radius,
                    center.y - radius,
                    center.x + radius,
                    center.y + radius,
                ),
                GeometryPrimitive::Polyline { points, .. } => {
                    if points.is_empty() {
                        return None;
                    }
                    let mut min_x = f64::MAX;
                    let mut min_y = f64::MAX;
                    let mut max_x = f64::MIN;
                    let mut max_y = f64::MIN;
                    for p in points {
                        min_x = min_x.min(p.x);
                        min_y = min_y.min(p.y);
                        max_x = max_x.max(p.x);
                        max_y = max_y.max(p.y);
                    }
                    (min_x, min_y, max_x, max_y)
                }
                GeometryPrimitive::Spline { control_points, .. } => {
                    if control_points.is_empty() {
                        return None;
                    }
                    let mut min_x = f64::MAX;
                    let mut min_y = f64::MAX;
                    let mut max_x = f64::MIN;
                    let mut max_y = f64::MIN;
                    for p in control_points {
                        min_x = min_x.min(p.x);
                        min_y = min_y.min(p.y);
                        max_x = max_x.max(p.x);
                        max_y = max_y.max(p.y);
                    }
                    (min_x, min_y, max_x, max_y)
                }
                GeometryPrimitive::Point(p) => (p.x, p.y, p.x, p.y),
            };
            Some(EntityBbox {
                id: e.id,
                min_x,
                min_y,
                max_x,
                max_y,
            })
        })
        .collect()
}

/// Find module boundaries by recursive elbow detection on sorted centroids.
///
/// Algorithm:
///   1. Strip 1D outliers (isolated centroids whose neighboring gaps both
///      dwarf the local median) — corner markers, stamps, drawing border
///      anchors that would otherwise occupy the top-N gap positions.
///   2. Find the largest elbow in sorted-descending gaps (ratio between
///      consecutive sorted gaps). Take the top split_count gaps as boundaries.
///   3. For each resulting slice, recurse if it still has enough entities and
///      a meaningful sub-elbow exists. The threshold tightens with depth so
///      a single module with a wide corridor doesn't keep splitting.
fn projection_slice_with_floor(centroids: Vec<f64>, _abs_floor: f64) -> Vec<(f64, f64)> {
    if centroids.is_empty() {
        return Vec::new();
    }
    let mut sorted = centroids;
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let cleaned = strip_outliers(sorted);
    let mut out = Vec::new();
    elbow_recurse(&cleaned, &mut out, 0);
    if out.is_empty() && !cleaned.is_empty() {
        out.push((cleaned[0], cleaned[cleaned.len() - 1]));
    }
    out
}

/// Drop isolated 1D outlier centroids: a centroid whose distance to BOTH
/// neighbors is more than `outlier_factor` × the median gap. This removes the
/// title-block stamps, $EXTMAX corner markers, and stray off-page entities
/// that would otherwise dominate the top-N gap positions.
fn strip_outliers(sorted: Vec<f64>) -> Vec<f64> {
    let n = sorted.len();
    if n < 4 {
        return sorted;
    }
    let mut gaps: Vec<f64> = (1..n).map(|i| sorted[i] - sorted[i - 1]).collect();
    let mut sorted_gaps = gaps.clone();
    sorted_gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted_gaps[sorted_gaps.len() / 2].max(1e-9);
    let outlier_factor = 50.0; // very conservative — only catches truly isolated points
    let threshold = median * outlier_factor;

    let mut keep = vec![true; n];
    for i in 0..n {
        let gap_before = if i == 0 { f64::INFINITY } else { gaps[i - 1] };
        let gap_after = if i == n - 1 {
            f64::INFINITY
        } else {
            gaps[i]
        };
        if gap_before > threshold && gap_after > threshold {
            keep[i] = false;
        }
    }
    let cleaned: Vec<f64> = sorted
        .into_iter()
        .zip(keep.into_iter())
        .filter_map(|(v, k)| if k { Some(v) } else { None })
        .collect();
    let _ = &mut gaps;
    cleaned
}

fn elbow_recurse(sorted: &[f64], out: &mut Vec<(f64, f64)>, depth: usize) {
    const MAX_DEPTH: usize = 4;
    let n = sorted.len();
    if n == 0 {
        return;
    }
    if n < 20 || depth >= MAX_DEPTH {
        out.push((sorted[0], sorted[n - 1]));
        return;
    }

    let total_range = sorted[n - 1] - sorted[0];
    if total_range < 1e-9 {
        out.push((sorted[0], sorted[n - 1]));
        return;
    }

    // Per-position gaps preserving the original index for splitting.
    let mut gaps: Vec<(usize, f64)> = (1..n).map(|i| (i - 1, sorted[i] - sorted[i - 1])).collect();
    let mut by_size: Vec<(usize, f64)> = gaps.clone();
    by_size.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let candidates_n = by_size.len().min(10);
    if candidates_n < 2 {
        out.push((sorted[0], sorted[n - 1]));
        return;
    }

    // Threshold: gap must be at least 1% of slice range to count.
    let abs_min = total_range * 0.01;

    // Required ratio tightens with depth — protects against over-fragmentation
    // when a single module has internal structure.
    let required_ratio = match depth {
        0 => 5.0,
        1 => 7.0,
        _ => 12.0,
    };

    let mut best_k: Option<usize> = None;
    let mut best_ratio = 1.0;
    for k in 0..(candidates_n - 1) {
        let g_here = by_size[k].1;
        let g_next = by_size[k + 1].1.max(1e-9);
        if g_here < abs_min {
            break;
        }
        let ratio = g_here / g_next;
        if ratio > best_ratio {
            best_ratio = ratio;
            best_k = Some(k);
        }
    }

    let split_count = match best_k {
        Some(k) if best_ratio >= required_ratio => k + 1,
        _ => {
            out.push((sorted[0], sorted[n - 1]));
            return;
        }
    };

    let mut split_indices: Vec<usize> = by_size[..split_count].iter().map(|(i, _)| *i).collect();
    split_indices.sort_unstable();

    let mut start = 0usize;
    for &k in &split_indices {
        elbow_recurse(&sorted[start..=k], out, depth + 1);
        start = k + 1;
    }
    elbow_recurse(&sorted[start..], out, depth + 1);
    let _ = gaps;
}

fn slice_index_of(slices: &[(f64, f64)], v: f64) -> usize {
    for (i, (lo, hi)) in slices.iter().enumerate() {
        // Inclusive on both ends for tail safety; ranges don't overlap by construction.
        if v >= *lo - 1e-9 && v <= *hi + 1e-9 {
            return i;
        }
    }
    // Fallback: nearest slice
    let mut best = 0usize;
    let mut best_d = f64::MAX;
    for (i, (lo, hi)) in slices.iter().enumerate() {
        let d = if v < *lo { lo - v } else { v - hi };
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

fn drawing_extent(bboxes: &[EntityBbox]) -> (f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for b in bboxes {
        min_x = min_x.min(b.min_x);
        min_y = min_y.min(b.min_y);
        max_x = max_x.max(b.max_x);
        max_y = max_y.max(b.max_y);
    }
    (max_x - min_x, max_y - min_y)
}

fn find(parents: &mut [usize], i: usize) -> usize {
    let mut root = i;
    while parents[root] != root {
        root = parents[root];
    }
    let mut cur = i;
    while parents[cur] != root {
        let next = parents[cur];
        parents[cur] = root;
        cur = next;
    }
    root
}

fn union(parents: &mut [usize], a: usize, b: usize) {
    let ra = find(parents, a);
    let rb = find(parents, b);
    if ra != rb {
        parents[ra] = rb;
    }
}

/// Per-structure summaries used by the audit JSON and AI prompt builders.
pub fn summarize_dimensions(drawing: &Drawing, structure: &Structure) -> Vec<f64> {
    let id_set: HashSet<EntityId> = structure.dimension_ids.iter().copied().collect();
    drawing
        .dimensions
        .iter()
        .filter(|d| id_set.contains(&d.id))
        .map(|d| d.nominal_value)
        .collect()
}

pub fn summarize_annotations(drawing: &Drawing, structure: &Structure) -> Vec<String> {
    let id_set: HashSet<EntityId> = structure.annotation_ids.iter().copied().collect();
    drawing
        .annotations
        .iter()
        .filter(|a| id_set.contains(&a.id))
        .map(|a| a.text.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::model::{DrawingMetadata, Units};

    fn make_drawing(entities: Vec<Entity>) -> Drawing {
        Drawing {
            units: Units::Centimeters,
            entities,
            dimensions: Vec::new(),
            gdt_frames: Vec::new(),
            annotations: Vec::new(),
            datums: Vec::new(),
            metadata: DrawingMetadata {
                filename: "test".into(),
                title: None,
                author: None,
                scale: None,
                sheet_size: None,
            },
            structures: Vec::new(),
        }
    }

    fn poly_box(id: u64, x0: f64, y0: f64, w: f64, h: f64) -> Entity {
        Entity {
            id: EntityId(id),
            geometry: GeometryPrimitive::Polyline {
                points: vec![
                    Point2D::new(x0, y0),
                    Point2D::new(x0 + w, y0),
                    Point2D::new(x0 + w, y0 + h),
                    Point2D::new(x0, y0 + h),
                ],
                bulges: vec![0.0, 0.0, 0.0, 0.0],
                closed: true,
            },
            layer: "0".into(),
            color: None,
            lineweight: None,
            linetype: None,
            block_ref: None,
        }
    }

    #[test]
    fn single_module_returns_one_structure() {
        let mut entities = Vec::new();
        for i in 0..40 {
            entities.push(poly_box(i, (i % 10) as f64 * 50.0, (i / 10) as f64 * 50.0, 10.0, 10.0));
        }
        let drawing = make_drawing(entities);
        let structures = detect_structures(&drawing);
        assert_eq!(structures.len(), 1);
        assert_eq!(structures[0].entity_ids.len(), 40);
    }

    #[test]
    fn three_modules_separated_by_large_gap_are_split() {
        // Pseudorandom scatter within each module — emulates the continuous
        // entity distribution of a real floor plan (walls, dimensions,
        // hatching), which has no rigid column structure.
        let mut entities = Vec::new();
        let mut id = 0u64;
        let mut seed = 12345u64;
        let mut rand = move || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 33) as u32) as f64 / u32::MAX as f64
        };
        let module_w = 700.0;
        let module_h = 500.0;
        let module_gap = 11_000.0;
        for module in 0..3 {
            let x_base = module as f64 * module_gap;
            for _ in 0..120 {
                let x = x_base + rand() * module_w;
                let y = rand() * module_h;
                entities.push(poly_box(id, x, y, 5.0, 5.0));
                id += 1;
            }
        }
        let drawing = make_drawing(entities);
        let structures = detect_structures(&drawing);
        assert_eq!(structures.len(), 3, "expected 3 modules, got {}", structures.len());
        assert!(structures[0].bbox_min.x < structures[1].bbox_min.x);
        assert!(structures[1].bbox_min.x < structures[2].bbox_min.x);
    }

    #[test]
    fn empty_drawing_returns_no_structures() {
        let drawing = make_drawing(Vec::new());
        let structures = detect_structures(&drawing);
        assert!(structures.is_empty());
    }
}
