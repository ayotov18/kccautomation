use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Relationship type between activities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    /// Finish-to-Start: successor starts after predecessor finishes
    FS,
    /// Finish-to-Finish: successor finishes when predecessor finishes
    FF,
    /// Start-to-Start: successor starts when predecessor starts
    SS,
    /// Start-to-Finish: successor finishes when predecessor starts
    SF,
}

/// An activity in the CPM network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpmActivity {
    pub id: String,
    pub duration: f64,
    /// Predecessors: (activity_id, relationship_type, lag)
    pub predecessors: Vec<(String, RelationType, f64)>,
}

/// CPM calculation result for a single activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpmResult {
    pub early_start: f64,
    pub early_finish: f64,
    pub late_start: f64,
    pub late_finish: f64,
    pub total_float: f64,
    pub free_float: f64,
    pub is_critical: bool,
}

/// Calculate the Critical Path Method for a set of activities.
///
/// Forward pass: ES = max(predecessor constraint + lag), EF = ES + duration
/// Backward pass: LF = min(successor constraint - lag), LS = LF - duration
/// Float: total_float = LS - ES, critical = total_float <= 0.0001
pub fn calculate_cpm(activities: &[CpmActivity]) -> HashMap<String, CpmResult> {
    if activities.is_empty() {
        return HashMap::new();
    }

    let activity_map: HashMap<&str, &CpmActivity> =
        activities.iter().map(|a| (a.id.as_str(), a)).collect();

    // Build successor map for backward pass
    let mut successors: HashMap<&str, Vec<(&str, RelationType, f64)>> = HashMap::new();
    for act in activities {
        for (pred_id, rel, lag) in &act.predecessors {
            successors
                .entry(pred_id.as_str())
                .or_default()
                .push((act.id.as_str(), *rel, *lag));
        }
        // Ensure every activity appears in the successor map even with no successors
        successors.entry(act.id.as_str()).or_default();
    }

    // Topological sort using Kahn's algorithm
    let topo_order = topological_sort(activities);

    // ── Forward pass ──
    let mut early_start: HashMap<&str, f64> = HashMap::new();
    let mut early_finish: HashMap<&str, f64> = HashMap::new();

    for id in &topo_order {
        let act = activity_map[id.as_str()];
        let mut es = 0.0_f64;

        for (pred_id, rel, lag) in &act.predecessors {
            let pred_es = early_start.get(pred_id.as_str()).copied().unwrap_or(0.0);
            let pred_ef = early_finish.get(pred_id.as_str()).copied().unwrap_or(0.0);

            let constraint = match rel {
                RelationType::FS => pred_ef + lag,
                RelationType::FF => pred_ef + lag - act.duration,
                RelationType::SS => pred_es + lag,
                RelationType::SF => pred_es + lag - act.duration,
            };
            es = es.max(constraint);
        }

        // ES must be non-negative
        es = es.max(0.0);
        let ef = es + act.duration;

        early_start.insert(id.as_str(), es);
        early_finish.insert(id.as_str(), ef);
    }

    // Project finish = max of all EF
    let project_finish = early_finish.values().copied().fold(0.0_f64, f64::max);

    // ── Backward pass ──
    let mut late_start: HashMap<&str, f64> = HashMap::new();
    let mut late_finish: HashMap<&str, f64> = HashMap::new();

    for id in topo_order.iter().rev() {
        let act = activity_map[id.as_str()];

        let mut lf = project_finish;

        if let Some(succs) = successors.get(id.as_str()) {
            for &(succ_id, ref rel, lag) in succs {
                let succ_ls = late_start.get(succ_id).copied().unwrap_or(project_finish);
                let succ_lf = late_finish.get(succ_id).copied().unwrap_or(project_finish);

                let derived_lf = match rel {
                    RelationType::FS => succ_ls - lag,
                    RelationType::FF => succ_lf - lag,
                    RelationType::SS => succ_ls - lag + act.duration,
                    RelationType::SF => succ_lf - lag,
                };
                lf = lf.min(derived_lf);
            }
        }

        let ls = lf - act.duration;

        late_start.insert(id.as_str(), ls);
        late_finish.insert(id.as_str(), lf);
    }

    // ── Compute floats and build results ──
    let mut results = HashMap::new();

    for id in &topo_order {
        let es = early_start[id.as_str()];
        let ef = early_finish[id.as_str()];
        let ls = late_start[id.as_str()];
        let lf = late_finish[id.as_str()];
        let total_float = ls - es;

        // Free float = min(successor ES - relationship constraint) for FS
        let free_float = if let Some(succs) = successors.get(id.as_str()) {
            if succs.is_empty() {
                // Terminal activity: free float = total float
                total_float.max(0.0)
            } else {
                let mut min_ff = f64::MAX;
                for &(succ_id, ref rel, lag) in succs {
                    let succ_es = early_start.get(succ_id).copied().unwrap_or(0.0);
                    let ff = match rel {
                        RelationType::FS => succ_es - ef - lag,
                        RelationType::FF => {
                            let succ_ef =
                                early_finish.get(succ_id).copied().unwrap_or(0.0);
                            succ_ef - ef - lag
                        }
                        RelationType::SS => succ_es - es - lag,
                        RelationType::SF => {
                            let succ_ef =
                                early_finish.get(succ_id).copied().unwrap_or(0.0);
                            succ_ef - es - lag
                        }
                    };
                    min_ff = min_ff.min(ff);
                }
                min_ff.max(0.0)
            }
        } else {
            // Terminal activity: free float = total float
            total_float.max(0.0)
        };

        let is_critical = total_float.abs() < 0.0001;

        results.insert(
            id.clone(),
            CpmResult {
                early_start: es,
                early_finish: ef,
                late_start: ls,
                late_finish: lf,
                total_float,
                free_float,
                is_critical,
            },
        );
    }

    results
}

/// Topological sort via Kahn's algorithm.
fn topological_sort(activities: &[CpmActivity]) -> Vec<String> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for act in activities {
        in_degree.entry(act.id.as_str()).or_insert(0);
        for (pred_id, _, _) in &act.predecessors {
            adj.entry(pred_id.as_str())
                .or_default()
                .push(act.id.as_str());
            *in_degree.entry(act.id.as_str()).or_insert(0) += 1;
        }
    }

    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(id, _)| *id)
        .collect();
    queue.sort(); // deterministic ordering

    let mut result = Vec::new();

    while let Some(node) = queue.pop() {
        result.push(node.to_string());
        if let Some(neighbors) = adj.get(node) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(neighbor);
                        queue.sort();
                    }
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_chain() {
        // A(5) → B(3) → C(2) = total 10 days, all critical
        let activities = vec![
            CpmActivity {
                id: "A".into(),
                duration: 5.0,
                predecessors: vec![],
            },
            CpmActivity {
                id: "B".into(),
                duration: 3.0,
                predecessors: vec![("A".into(), RelationType::FS, 0.0)],
            },
            CpmActivity {
                id: "C".into(),
                duration: 2.0,
                predecessors: vec![("B".into(), RelationType::FS, 0.0)],
            },
        ];

        let results = calculate_cpm(&activities);
        assert_eq!(results.len(), 3);

        let a = &results["A"];
        assert!((a.early_start - 0.0).abs() < 0.001);
        assert!((a.early_finish - 5.0).abs() < 0.001);
        assert!(a.is_critical);

        let b = &results["B"];
        assert!((b.early_start - 5.0).abs() < 0.001);
        assert!((b.early_finish - 8.0).abs() < 0.001);
        assert!(b.is_critical);

        let c = &results["C"];
        assert!((c.early_start - 8.0).abs() < 0.001);
        assert!((c.early_finish - 10.0).abs() < 0.001);
        assert!(c.is_critical);
    }

    #[test]
    fn test_parallel_paths_with_float() {
        // A(5) → C(2), B(3) → C(2)
        // Critical path: A → C = 7
        // B has float = 2
        let activities = vec![
            CpmActivity {
                id: "A".into(),
                duration: 5.0,
                predecessors: vec![],
            },
            CpmActivity {
                id: "B".into(),
                duration: 3.0,
                predecessors: vec![],
            },
            CpmActivity {
                id: "C".into(),
                duration: 2.0,
                predecessors: vec![
                    ("A".into(), RelationType::FS, 0.0),
                    ("B".into(), RelationType::FS, 0.0),
                ],
            },
        ];

        let results = calculate_cpm(&activities);

        assert!(results["A"].is_critical);
        assert!(!results["B"].is_critical);
        assert!((results["B"].total_float - 2.0).abs() < 0.001);
        assert!(results["C"].is_critical);
    }

    #[test]
    fn test_empty_activities() {
        let results = calculate_cpm(&[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_single_activity() {
        let activities = vec![CpmActivity {
            id: "X".into(),
            duration: 10.0,
            predecessors: vec![],
        }];

        let results = calculate_cpm(&activities);
        let x = &results["X"];
        assert!((x.early_start - 0.0).abs() < 0.001);
        assert!((x.early_finish - 10.0).abs() < 0.001);
        assert!(x.is_critical);
    }

    #[test]
    fn test_fs_with_lag() {
        // A(5) --FS+2--> B(3): B starts at day 7
        let activities = vec![
            CpmActivity {
                id: "A".into(),
                duration: 5.0,
                predecessors: vec![],
            },
            CpmActivity {
                id: "B".into(),
                duration: 3.0,
                predecessors: vec![("A".into(), RelationType::FS, 2.0)],
            },
        ];

        let results = calculate_cpm(&activities);
        let b = &results["B"];
        assert!((b.early_start - 7.0).abs() < 0.001);
        assert!((b.early_finish - 10.0).abs() < 0.001);
    }
}
