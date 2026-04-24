use super::types::DatumHierarchy;
use crate::geometry::model::FeatureControlFrame;

/// Build datum hierarchy from feature control frames.
/// The order of datum references in a FCF defines the hierarchy:
/// first = primary, second = secondary, third = tertiary.
pub fn build_datum_hierarchy(frames: &[FeatureControlFrame]) -> DatumHierarchy {
    let mut hierarchy = DatumHierarchy::new();

    // Collect all unique datum orderings
    for frame in frames {
        if frame.datum_refs.is_empty() {
            continue;
        }

        if hierarchy.primary.is_none() {
            if let Some(first) = frame.datum_refs.first() {
                hierarchy.primary = Some(first.label);
            }
        }

        if hierarchy.secondary.is_none() && frame.datum_refs.len() >= 2 {
            hierarchy.secondary = Some(frame.datum_refs[1].label);
        }

        if hierarchy.tertiary.is_none() && frame.datum_refs.len() >= 3 {
            hierarchy.tertiary = Some(frame.datum_refs[2].label);
        }
    }

    hierarchy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::model::{DatumReference, EntityId, GdtSymbol, MaterialCondition, Point2D};

    #[test]
    fn test_build_hierarchy() {
        let frames = vec![FeatureControlFrame {
            id: EntityId(1),
            symbol: GdtSymbol::Position,
            tolerance_value: 0.05,
            material_condition: MaterialCondition::None,
            datum_refs: vec![
                DatumReference {
                    label: 'A',
                    material_condition: MaterialCondition::None,
                },
                DatumReference {
                    label: 'B',
                    material_condition: MaterialCondition::None,
                },
                DatumReference {
                    label: 'C',
                    material_condition: MaterialCondition::None,
                },
            ],
            position: Point2D::new(0.0, 0.0),
            attached_entities: Vec::new(),
            projected_tolerance: false,
            is_diameter_zone: false,
        }];

        let hierarchy = build_datum_hierarchy(&frames);
        assert_eq!(hierarchy.primary, Some('A'));
        assert_eq!(hierarchy.secondary, Some('B'));
        assert_eq!(hierarchy.tertiary, Some('C'));
    }
}
