use crate::geometry::model::EntityId;
use serde::{Deserialize, Serialize};

/// A tolerance chain (stack-up) — sequence of dimensions controlling a characteristic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToleranceChain {
    /// Ordered list of dimension EntityIds forming the chain.
    pub members: Vec<EntityId>,
    /// Sum of nominal values along the chain.
    pub nominal_stack: f64,
    /// Worst-case tolerance accumulation.
    pub worst_case: f64,
    /// RSS (statistical) tolerance accumulation.
    pub rss: f64,
    /// The critical path — dimensions contributing most to accumulation.
    pub critical_path: Vec<EntityId>,
}
