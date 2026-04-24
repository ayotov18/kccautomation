use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// KCC scoring configuration with adjustable thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KccConfig {
    /// Minimum score for KCC classification.
    pub kcc_threshold: u32,
    /// Minimum score for Important classification.
    pub important_threshold: u32,
    /// Typical tolerance values by feature type (for tight tolerance evaluation).
    pub tolerance_typical: HashMap<String, f64>,
}

impl Default for KccConfig {
    fn default() -> Self {
        let mut typical = HashMap::new();
        // Typical tolerances for common feature types (mm)
        typical.insert("hole_small".to_string(), 0.1); // ⌀ < 10mm
        typical.insert("hole_medium".to_string(), 0.15); // ⌀ 10-50mm
        typical.insert("hole_large".to_string(), 0.2); // ⌀ > 50mm
        typical.insert("slot".to_string(), 0.15);
        typical.insert("linear".to_string(), 0.2);
        typical.insert("angular".to_string(), 0.5); // degrees

        Self {
            kcc_threshold: 8,
            important_threshold: 5,
            tolerance_typical: typical,
        }
    }
}

impl KccConfig {
    /// Get the typical tolerance for a feature given its type and size.
    pub fn typical_tolerance(&self, feature_type: &str, size: f64) -> f64 {
        if feature_type == "Hole" {
            if size < 10.0 {
                *self.tolerance_typical.get("hole_small").unwrap_or(&0.1)
            } else if size < 50.0 {
                *self.tolerance_typical.get("hole_medium").unwrap_or(&0.15)
            } else {
                *self.tolerance_typical.get("hole_large").unwrap_or(&0.2)
            }
        } else {
            *self
                .tolerance_typical
                .get(feature_type.to_lowercase().as_str())
                .unwrap_or(&0.2)
        }
    }
}
