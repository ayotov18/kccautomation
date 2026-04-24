use super::config::KccConfig;
use super::rules;
use super::types::{KccClassification, KccScore};
use crate::datum::types::DatumInfo;
use crate::feature::types::Feature;
use crate::geometry::model::Drawing;
use crate::tolerance_chain::types::ToleranceChain;

/// Context provided to scoring rules.
pub struct ScoringContext<'a> {
    pub drawing: &'a Drawing,
    pub features: &'a [Feature],
    pub chains: &'a [ToleranceChain],
    pub datums: &'a [DatumInfo],
    pub config: &'a KccConfig,
}

/// Classify a single feature using all scoring rules.
pub fn classify_feature(feature: &Feature, context: &ScoringContext) -> KccScore {
    let all_rules = rules::all_rules();
    let mut factors = Vec::new();

    for rule in &all_rules {
        if let Some(factor) = rule.evaluate(feature, context) {
            factors.push(factor);
        }
    }

    let total: u32 = factors.iter().map(|f| f.points).sum();

    let classification = if total >= context.config.kcc_threshold {
        KccClassification::Kcc
    } else if total >= context.config.important_threshold {
        KccClassification::Important
    } else {
        KccClassification::Standard
    };

    KccScore {
        total,
        factors,
        classification,
    }
}

/// Classify all features, returning (FeatureId.0, KccScore) pairs.
pub fn classify_all(features: &[Feature], context: &ScoringContext) -> Vec<(u64, KccScore)> {
    features
        .iter()
        .map(|feature| (feature.id.0, classify_feature(feature, context)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::types::{FeatureId, FeatureType};
    use crate::geometry::model::{
        DatumReference, Dimension, DimensionType, EntityId, FeatureControlFrame, GdtSymbol,
        MaterialCondition, Point2D, Tolerance,
    };

    static DEFAULT_CONFIG: std::sync::LazyLock<KccConfig> =
        std::sync::LazyLock::new(KccConfig::default);

    fn make_test_context<'a>(drawing: &'a Drawing, features: &'a [Feature]) -> ScoringContext<'a> {
        ScoringContext {
            drawing,
            features,
            chains: &[],
            datums: &[],
            config: &DEFAULT_CONFIG,
        }
    }

    #[test]
    fn test_kcc_classification_tight_tolerance() {
        let mut drawing = Drawing::new("test.dxf".to_string());
        drawing.dimensions.push(Dimension {
            id: EntityId(100),
            dim_type: DimensionType::Diameter,
            nominal_value: 10.0,
            text_override: None,
            tolerance: Some(Tolerance::symmetric(0.02)), // Very tight: 0.02 vs typical 0.1
            definition_points: vec![],
            text_position: Point2D::new(50.0, 50.0),
            layer: "0".to_string(),
            attached_entities: vec![EntityId(1)],
        });

        drawing.gdt_frames.push(FeatureControlFrame {
            id: EntityId(200),
            symbol: GdtSymbol::Position,
            tolerance_value: 0.05,
            material_condition: MaterialCondition::MaximumMaterial,
            datum_refs: vec![DatumReference {
                label: 'A',
                material_condition: MaterialCondition::None,
            }],
            position: Point2D::new(50.0, 50.0),
            attached_entities: vec![EntityId(1)],
            projected_tolerance: false,
            is_diameter_zone: false,
        });

        let features = vec![Feature {
            id: FeatureId(1),
            feature_type: FeatureType::Hole {
                diameter: 10.0,
                is_through: false,
            },
            geometry_refs: vec![EntityId(1)],
            centroid: Point2D::new(50.0, 50.0),
            dimensions: vec![EntityId(100)],
            gdt_frames: vec![EntityId(200)],
            datum_refs: vec!['A'],
            layer_hint: None,
        }];

        let datums = vec![DatumInfo {
            label: 'A',
            position: Point2D::new(0.0, 0.0),
            attached_feature_id: None,
        }];

        let context = ScoringContext {
            drawing: &drawing,
            features: &features,
            chains: &[],
            datums: &datums,
            config: &KccConfig::default(),
        };

        let score = classify_feature(&features[0], &context);

        // Should score high:
        // - Very tight tolerance: 5 pts (0.02 is 20% of typical 0.1)
        // - Datum reference: 4 pts
        // - GD&T controlled: 3 pts
        // - Position tolerance: 4 pts
        // Total >= 8, so KCC
        assert!(
            score.total >= 8,
            "Score was {}: {:?}",
            score.total,
            score.factors
        );
        assert_eq!(score.classification, KccClassification::Kcc);
    }

    #[test]
    fn test_standard_classification() {
        let drawing = Drawing::new("test.dxf".to_string());
        let features = vec![Feature {
            id: FeatureId(1),
            feature_type: FeatureType::Hole {
                diameter: 10.0,
                is_through: false,
            },
            geometry_refs: vec![EntityId(1)],
            centroid: Point2D::new(50.0, 50.0),
            dimensions: Vec::new(),
            gdt_frames: Vec::new(),
            datum_refs: Vec::new(),
            layer_hint: None,
        }];

        let context = make_test_context(&drawing, &features);
        let score = classify_feature(&features[0], &context);

        // No tolerance, no GD&T, no datums = Standard
        assert!(score.total < 5);
        assert_eq!(score.classification, KccClassification::Standard);
    }
}
