use crate::geometry::model::Tolerance;
use serde::{Deserialize, Serialize};

/// Result of parsing dimension text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDimension {
    pub nominal: f64,
    pub tolerance: Option<Tolerance>,
    pub prefix: DimensionPrefix,
    pub count: Option<usize>,
    pub is_reference: bool,
    pub is_basic: bool,
    pub raw_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DimensionPrefix {
    None,
    Diameter,
    Radius,
    Thread(String),
    Angular,
}

#[derive(Debug, thiserror::Error)]
pub enum DimensionParseError {
    #[error("empty dimension text")]
    Empty,
    #[error("failed to parse dimension text: {0}")]
    ParseFailed(String),
    #[error("no numeric value found in: {0}")]
    NoValue(String),
}
