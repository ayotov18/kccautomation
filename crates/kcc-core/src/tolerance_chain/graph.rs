use crate::feature::types::{Feature, FeatureId};
use crate::geometry::model::{Dimension, EntityId, Tolerance};
use std::collections::HashMap;

/// A node in the dimension graph representing a feature or datum point.
#[derive(Debug, Clone)]
pub struct DimensionNode {
    pub entity_id: EntityId,
    pub feature_id: Option<FeatureId>,
}

/// An edge connecting two features via a dimension.
#[derive(Debug, Clone)]
pub struct DimensionEdge {
    pub from: EntityId,
    pub to: EntityId,
    pub nominal: f64,
    pub tolerance: Tolerance,
    pub dim_id: EntityId,
}

/// Graph of features connected by dimensions.
#[derive(Debug)]
pub struct DimensionGraph {
    pub nodes: HashMap<EntityId, DimensionNode>,
    pub edges: Vec<DimensionEdge>,
    pub adjacency: HashMap<EntityId, Vec<usize>>, // node -> edge indices
}

impl DimensionGraph {
    /// Build a dimension graph from features and dimensions.
    pub fn build(features: &[Feature], dimensions: &[Dimension]) -> Self {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();
        let mut adjacency: HashMap<EntityId, Vec<usize>> = HashMap::new();

        // Create nodes for each feature's geometry refs
        for feature in features {
            for &entity_id in &feature.geometry_refs {
                nodes.insert(
                    entity_id,
                    DimensionNode {
                        entity_id,
                        feature_id: Some(feature.id),
                    },
                );
            }
        }

        // Create edges from dimensions
        for dim in dimensions {
            if dim.attached_entities.is_empty() {
                continue;
            }

            let tolerance = dim.tolerance.clone().unwrap_or(Tolerance::symmetric(0.0));

            // If dimension connects two entities, create an edge
            if dim.attached_entities.len() >= 2 {
                let from = dim.attached_entities[0];
                let to = dim.attached_entities[1];

                let edge_idx = edges.len();
                edges.push(DimensionEdge {
                    from,
                    to,
                    nominal: dim.nominal_value,
                    tolerance,
                    dim_id: dim.id,
                });

                adjacency.entry(from).or_default().push(edge_idx);
                adjacency.entry(to).or_default().push(edge_idx);
            }
        }

        Self {
            nodes,
            edges,
            adjacency,
        }
    }

    /// Find all paths between two nodes using BFS.
    pub fn find_paths(&self, from: EntityId, to: EntityId, max_depth: usize) -> Vec<Vec<usize>> {
        let mut all_paths = Vec::new();
        let mut queue: Vec<(EntityId, Vec<usize>, std::collections::HashSet<EntityId>)> =
            Vec::new();

        let mut initial_visited = std::collections::HashSet::new();
        initial_visited.insert(from);
        queue.push((from, Vec::new(), initial_visited));

        while let Some((current, path, visited)) = queue.pop() {
            if path.len() > max_depth {
                continue;
            }

            if current == to && !path.is_empty() {
                all_paths.push(path.clone());
                continue;
            }

            if let Some(edge_indices) = self.adjacency.get(&current) {
                for &edge_idx in edge_indices {
                    let edge = &self.edges[edge_idx];
                    let next = if edge.from == current {
                        edge.to
                    } else {
                        edge.from
                    };

                    if !visited.contains(&next) {
                        let mut new_path = path.clone();
                        new_path.push(edge_idx);
                        let mut new_visited = visited.clone();
                        new_visited.insert(next);
                        queue.push((next, new_path, new_visited));
                    }
                }
            }
        }

        all_paths
    }
}
