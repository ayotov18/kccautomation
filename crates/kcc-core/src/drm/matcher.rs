//! DRM decision logic: determines what action to take for a retrieval match.
//!
//! Thresholds:
//! - AutoOverride: similarity > 0.95 AND confidence > 0.95 AND times_confirmed >= 5
//! - ConfidenceBoost: similarity > 0.8 AND confidence > 0.7
//! - Suggest: similarity > 0.6
//! - NoMatch: below all thresholds

use super::DrmAction;

/// Auto-override thresholds.
const AUTO_OVERRIDE_SIMILARITY: f64 = 0.95;
const AUTO_OVERRIDE_CONFIDENCE: f64 = 0.95;
const AUTO_OVERRIDE_MIN_CONFIRMATIONS: i32 = 5;

/// Confidence boost thresholds.
const BOOST_SIMILARITY: f64 = 0.8;
const BOOST_CONFIDENCE: f64 = 0.7;

/// Minimum similarity to even consider a match.
const MIN_SIMILARITY: f64 = 0.6;

/// Determine the DRM action based on match quality.
pub fn determine_action(similarity: f64, confidence: f64, times_confirmed: i32) -> DrmAction {
    if similarity >= AUTO_OVERRIDE_SIMILARITY
        && confidence >= AUTO_OVERRIDE_CONFIDENCE
        && times_confirmed >= AUTO_OVERRIDE_MIN_CONFIRMATIONS
    {
        DrmAction::AutoOverride
    } else if similarity >= BOOST_SIMILARITY && confidence >= BOOST_CONFIDENCE {
        DrmAction::ConfidenceBoost
    } else if similarity >= MIN_SIMILARITY {
        DrmAction::Suggest
    } else {
        DrmAction::NoMatch
    }
}

/// Calculate boosted confidence when DRM provides a ConfidenceBoost.
/// Rigid rule confidence is increased but capped at 0.95 (below auto-override threshold).
pub fn apply_confidence_boost(rigid_confidence: f64, drm_confidence: f64, similarity: f64) -> f64 {
    let boost = drm_confidence * similarity * 0.3; // max boost ≈ 0.3
    (rigid_confidence + boost).min(0.95)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_override() {
        assert_eq!(
            determine_action(0.98, 1.0, 10),
            DrmAction::AutoOverride
        );
    }

    #[test]
    fn test_confidence_boost() {
        assert_eq!(
            determine_action(0.85, 0.8, 3),
            DrmAction::ConfidenceBoost
        );
    }

    #[test]
    fn test_suggest() {
        assert_eq!(
            determine_action(0.65, 0.5, 1),
            DrmAction::Suggest
        );
    }

    #[test]
    fn test_no_match() {
        assert_eq!(
            determine_action(0.4, 0.3, 1),
            DrmAction::NoMatch
        );
    }

    #[test]
    fn test_not_enough_confirmations() {
        // High similarity and confidence but only 3 confirmations → boost, not override
        assert_eq!(
            determine_action(0.98, 1.0, 3),
            DrmAction::ConfidenceBoost
        );
    }

    #[test]
    fn test_confidence_boost_value() {
        let boosted = apply_confidence_boost(0.5, 0.9, 0.85);
        assert!(boosted > 0.5);
        assert!(boosted <= 0.95);
    }
}
