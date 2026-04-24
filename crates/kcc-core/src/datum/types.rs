use crate::feature::types::FeatureId;
use crate::geometry::model::Point2D;
use serde::{Deserialize, Serialize};

/// Information about a datum extracted from the drawing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatumInfo {
    pub label: char,
    pub position: Point2D,
    pub attached_feature_id: Option<FeatureId>,
}

/// Datum reference hierarchy extracted from feature control frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatumHierarchy {
    pub primary: Option<char>,
    pub secondary: Option<char>,
    pub tertiary: Option<char>,
}

impl DatumHierarchy {
    pub fn new() -> Self {
        Self {
            primary: None,
            secondary: None,
            tertiary: None,
        }
    }

    pub fn contains(&self, label: char) -> bool {
        self.primary == Some(label) || self.secondary == Some(label) || self.tertiary == Some(label)
    }
}

impl Default for DatumHierarchy {
    fn default() -> Self {
        Self::new()
    }
}
