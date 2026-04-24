use super::graph::DimensionGraph;
use super::types::ToleranceChain;
use crate::datum::types::DatumInfo;
use crate::feature::types::Feature;
use crate::geometry::model::Dimension;

/// Analyze tolerance chains for all features relative to datums.
pub fn analyze_chains(
    features: &[Feature],
    dimensions: &[Dimension],
    datums: &[DatumInfo],
) -> Vec<ToleranceChain> {
    let graph = DimensionGraph::build(features, dimensions);
    let mut chains = Vec::new();

    // For each feature with datum references, find paths from datum to feature
    for feature in features {
        if feature.datum_refs.is_empty() {
            continue;
        }

        // Find the datum entity to start from
        for &datum_label in &feature.datum_refs {
            let datum = datums.iter().find(|d| d.label == datum_label);
            if datum.is_none() {
                continue;
            }

            // Try to find a path from any datum-attached entity to this feature's entity
            for &feature_entity in &feature.geometry_refs {
                // Find datum entity in the graph
                for node_id in graph.nodes.keys() {
                    let paths = graph.find_paths(*node_id, feature_entity, 5);
                    for path in paths {
                        let chain = compute_chain_from_path(&graph, &path);
                        if chain.members.len() >= 2 {
                            chains.push(chain);
                        }
                    }
                }
            }
        }
    }

    // Deduplicate chains (same members in same order)
    chains.dedup_by(|a, b| a.members == b.members);

    chains
}

/// Compute tolerance chain metrics from a path of edge indices.
fn compute_chain_from_path(graph: &DimensionGraph, edge_indices: &[usize]) -> ToleranceChain {
    let mut members = Vec::new();
    let mut nominal_stack = 0.0;
    let mut worst_case = 0.0;
    let mut rss_sum = 0.0;
    let mut max_contribution = (0, 0.0_f64); // (index, half_range)

    for (i, &edge_idx) in edge_indices.iter().enumerate() {
        let edge = &graph.edges[edge_idx];
        members.push(edge.dim_id);
        nominal_stack += edge.nominal;

        let half_range = edge.tolerance.half_range();
        worst_case += half_range;
        rss_sum += half_range * half_range;

        if half_range > max_contribution.1 {
            max_contribution = (i, half_range);
        }
    }

    let rss = rss_sum.sqrt();

    // Critical path = the dimension contributing most to tolerance
    let critical_path = if !members.is_empty() {
        vec![members[max_contribution.0]]
    } else {
        Vec::new()
    };

    ToleranceChain {
        members,
        nominal_stack,
        worst_case,
        rss,
        critical_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::model::{EntityId, Tolerance};

    #[test]
    fn test_chain_computation() {
        let graph = DimensionGraph {
            nodes: std::collections::HashMap::new(),
            edges: vec![
                super::super::graph::DimensionEdge {
                    from: EntityId(1),
                    to: EntityId(2),
                    nominal: 50.0,
                    tolerance: Tolerance::symmetric(0.1),
                    dim_id: EntityId(101),
                },
                super::super::graph::DimensionEdge {
                    from: EntityId(2),
                    to: EntityId(3),
                    nominal: 30.0,
                    tolerance: Tolerance::symmetric(0.05),
                    dim_id: EntityId(102),
                },
            ],
            adjacency: std::collections::HashMap::new(),
        };

        let chain = compute_chain_from_path(&graph, &[0, 1]);
        assert!((chain.nominal_stack - 80.0).abs() < 1e-6);
        assert!((chain.worst_case - 0.15).abs() < 1e-6);
        // RSS = sqrt(0.1^2 + 0.05^2) = sqrt(0.01 + 0.0025) = sqrt(0.0125) ≈ 0.1118
        assert!((chain.rss - 0.1118).abs() < 0.001);
    }
}
