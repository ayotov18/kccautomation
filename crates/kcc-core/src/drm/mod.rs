//! Deterministic Retrieval Memory (DRM)
//!
//! A structured retrieval layer that sits between feature extraction and KSS generation.
//! It remembers every mapping decision (layer→SEK, block→fixture, feature→cost item)
//! and retrieves similar historical decisions for new drawings.
//!
//! **Key properties:**
//! - Deterministic: same input + same DRM state = same output
//! - Auditable: every auto-override is logged with full provenance
//! - User-correctable: user corrections feed back with confidence=1.0
//! - Per-user isolation: each user builds their own corpus

pub mod matcher;
pub mod recorder;
pub mod retrieval;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Bundle of historical matches retrieved for a drawing.
#[derive(Debug, Clone, Default)]
pub struct ContextBundle {
    pub layer_mappings: Vec<DrmMatch>,
    pub block_mappings: Vec<DrmMatch>,
    pub annotation_patterns: Vec<DrmMatch>,
    pub feature_sek_mappings: Vec<DrmMatch>,
}

impl ContextBundle {
    /// Find the best DRM match for a given layer name.
    pub fn find_layer_match(&self, layer_name: &str) -> Option<&DrmMatch> {
        let lower = layer_name.to_lowercase();
        self.layer_mappings.iter().find(|m| {
            m.matched_key.to_lowercase() == lower
                || (m.similarity > 0.8 && m.action != DrmAction::NoMatch)
        })
    }

    /// Find the best DRM match for a given block name.
    pub fn find_block_match(&self, block_name: &str) -> Option<&DrmMatch> {
        let lower = block_name.to_lowercase();
        self.block_mappings.iter().find(|m| {
            m.matched_key.to_lowercase() == lower
                || (m.similarity > 0.8 && m.action != DrmAction::NoMatch)
        })
    }

    /// Total number of matches across all categories.
    pub fn total_matches(&self) -> usize {
        self.layer_mappings.len()
            + self.block_mappings.len()
            + self.annotation_patterns.len()
            + self.feature_sek_mappings.len()
    }

    /// Count of auto-overrides across all categories.
    pub fn auto_override_count(&self) -> usize {
        [
            &self.layer_mappings,
            &self.block_mappings,
            &self.annotation_patterns,
            &self.feature_sek_mappings,
        ]
        .iter()
        .flat_map(|v| v.iter())
        .filter(|m| m.action == DrmAction::AutoOverride)
        .count()
    }
}

/// A single historical match from the DRM store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrmMatch {
    pub artifact_id: Uuid,
    pub input_key: String,
    pub matched_key: String,
    pub similarity: f64,
    pub sek_code: Option<String>,
    pub sek_group: Option<String>,
    pub description_bg: Option<String>,
    pub unit: Option<String>,
    pub confidence: f64,
    pub times_confirmed: i32,
    pub source: String,
    pub action: DrmAction,
}

/// What the system should do with a DRM match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrmAction {
    /// High confidence + many confirmations → use this mapping directly
    AutoOverride,
    /// Moderate confidence → boost the rigid rule's confidence
    ConfidenceBoost,
    /// Low confidence → show as suggestion in UI
    Suggest,
    /// No meaningful match found
    NoMatch,
}

/// Artifact types stored in the DRM.
pub const ARTIFACT_LAYER: &str = "layer_mapping";
pub const ARTIFACT_BLOCK: &str = "block_mapping";
pub const ARTIFACT_ANNOTATION: &str = "annotation_pattern";
pub const ARTIFACT_FEATURE_SEK: &str = "feature_sek";
pub const ARTIFACT_PRICE: &str = "price_association";

/// Normalize a key for trigram search: lowercase, collapse whitespace, strip punctuation.
pub fn normalize_key(key: &str) -> String {
    key.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || ('\u{0400}'..='\u{04FF}').contains(&c) {
            c
        } else {
            ' '
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_key() {
        assert_eq!(normalize_key("0-steni-gazobeton"), "0 steni gazobeton");
        assert_eq!(normalize_key("  Metal  "), "metal");
        assert_eq!(normalize_key("ARM1"), "arm1");
        assert_eq!(normalize_key("Зидарски работи"), "зидарски работи");
    }

    #[test]
    fn test_empty_context_bundle() {
        let ctx = ContextBundle::default();
        assert_eq!(ctx.total_matches(), 0);
        assert_eq!(ctx.auto_override_count(), 0);
        assert!(ctx.find_layer_match("anything").is_none());
    }
}
