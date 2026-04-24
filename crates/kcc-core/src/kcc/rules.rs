use super::scorer::ScoringContext;
use super::types::KccFactor;
use crate::feature::types::{Feature, FeatureType};
use crate::geometry::model::GdtSymbol;

/// Trait for individual scoring rules.
pub trait ScoringRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor>;
}

// === Rule 1: Tight Tolerance (3 points) ===
pub struct TightToleranceRule;

impl ScoringRule for TightToleranceRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor> {
        let (tol, size) = get_feature_tolerance(feature, context)?;
        let typical = context
            .config
            .typical_tolerance(feature.feature_type.name(), size);

        if tol <= typical * 0.5 && tol > typical * 0.25 {
            Some(KccFactor {
                name: "tight_tolerance".to_string(),
                points: 3,
                reason: format!(
                    "\u{00B1}{tol:.3}mm ({:.0}% of typical {typical:.3}mm)",
                    (tol / typical) * 100.0
                ),
            })
        } else {
            None
        }
    }
}

// === Rule 2: Very Tight Tolerance (5 points) ===
pub struct VeryTightToleranceRule;

impl ScoringRule for VeryTightToleranceRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor> {
        let (tol, size) = get_feature_tolerance(feature, context)?;
        let typical = context
            .config
            .typical_tolerance(feature.feature_type.name(), size);

        if tol <= typical * 0.25 {
            Some(KccFactor {
                name: "very_tight_tolerance".to_string(),
                points: 5,
                reason: format!(
                    "\u{00B1}{tol:.3}mm ({:.0}% of typical {typical:.3}mm)",
                    (tol / typical) * 100.0
                ),
            })
        } else {
            None
        }
    }
}

// === Rule 3: Datum Reference (4 points) ===
pub struct DatumReferenceRule;

impl ScoringRule for DatumReferenceRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor> {
        if !feature.datum_refs.is_empty() {
            let datums: String = feature.datum_refs.iter().collect();
            Some(KccFactor {
                name: "datum_reference".to_string(),
                points: 4,
                reason: format!("Referenced to datum(s) {datums}"),
            })
        } else {
            // Check if any GD&T frame for this feature has datum refs
            let has_gdt_datums = get_feature_gdt_frames(feature, context)
                .iter()
                .any(|f| !f.datum_refs.is_empty());

            if has_gdt_datums {
                Some(KccFactor {
                    name: "datum_reference".to_string(),
                    points: 4,
                    reason: "GD&T frame references datum(s)".to_string(),
                })
            } else {
                None
            }
        }
    }
}

// === Rule 4: GD&T Controlled (3 points) ===
pub struct GdtControlledRule;

impl ScoringRule for GdtControlledRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor> {
        let frames = get_feature_gdt_frames(feature, context);
        if !frames.is_empty() {
            Some(KccFactor {
                name: "gdt_controlled".to_string(),
                points: 3,
                reason: format!("Feature has {} feature control frame(s)", frames.len()),
            })
        } else {
            None
        }
    }
}

// === Rule 5: Position Tolerance (4 points) ===
pub struct PositionToleranceRule;

impl ScoringRule for PositionToleranceRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor> {
        let frames = get_feature_gdt_frames(feature, context);
        for frame in &frames {
            if frame.symbol == GdtSymbol::Position {
                let mc = match frame.material_condition {
                    crate::geometry::model::MaterialCondition::MaximumMaterial => " at MMC",
                    crate::geometry::model::MaterialCondition::LeastMaterial => " at LMC",
                    _ => "",
                };
                return Some(KccFactor {
                    name: "position_tolerance".to_string(),
                    points: 4,
                    reason: format!("True position \u{2300}{:.3}{mc}", frame.tolerance_value),
                });
            }
        }
        None
    }
}

// === Rule 6: Pattern Member (2 points) ===
pub struct PatternMemberRule;

impl ScoringRule for PatternMemberRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor> {
        // Check if this feature is referenced by a bolt circle or linear pattern
        for other in context.features {
            match &other.feature_type {
                FeatureType::BoltCircle { .. } | FeatureType::LinearPattern { .. } => {
                    // Check if our feature's geometry refs overlap
                    if feature
                        .geometry_refs
                        .iter()
                        .any(|r| other.geometry_refs.contains(r))
                    {
                        return Some(KccFactor {
                            name: "pattern_member".to_string(),
                            points: 2,
                            reason: format!("Part of {}", other.description()),
                        });
                    }
                }
                _ => {}
            }
        }
        None
    }
}

// === Rule 7: Assembly Interface (5 points) ===
pub struct AssemblyInterfaceRule;

impl ScoringRule for AssemblyInterfaceRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor> {
        // Mounting holes, alignment pins, and mating surfaces
        match &feature.feature_type {
            FeatureType::Hole { .. } => {
                // Check annotations for assembly keywords
                for ann in &context.drawing.annotations {
                    let text_upper = ann.text.to_uppercase();
                    let dist = crate::geometry::utils::distance(&ann.position, &feature.centroid);
                    if dist < 30.0
                        && (text_upper.contains("MOUNT")
                            || text_upper.contains("ALIGN")
                            || text_upper.contains("PILOT")
                            || text_upper.contains("DOWEL")
                            || text_upper.contains("LOCATING"))
                    {
                        return Some(KccFactor {
                            name: "assembly_interface".to_string(),
                            points: 5,
                            reason: "Assembly interface feature (mounting/alignment)".to_string(),
                        });
                    }
                }
            }
            FeatureType::Thread { .. } => {
                // Threads are fastener interfaces
                return Some(KccFactor {
                    name: "assembly_interface".to_string(),
                    points: 5,
                    reason: "Threaded fastener interface".to_string(),
                });
            }
            _ => {}
        }
        None
    }
}

// === Rule 8: Tolerance Chain Critical (4 points) ===
pub struct ToleranceChainCriticalRule;

impl ScoringRule for ToleranceChainCriticalRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor> {
        for chain in context.chains {
            // Check if this feature's dimensions are in the critical path
            for &dim_id in &feature.dimensions {
                if chain.critical_path.contains(&dim_id) {
                    return Some(KccFactor {
                        name: "tolerance_chain_critical".to_string(),
                        points: 4,
                        reason: format!(
                            "On critical path of tolerance chain (WC: {:.3}mm, RSS: {:.3}mm)",
                            chain.worst_case, chain.rss
                        ),
                    });
                }
            }
        }
        None
    }
}

// === Rule 9: Thread Specification (2 points) ===
pub struct ThreadSpecificationRule;

impl ScoringRule for ThreadSpecificationRule {
    fn evaluate(&self, feature: &Feature, _context: &ScoringContext) -> Option<KccFactor> {
        if let FeatureType::Thread { designation, .. } = &feature.feature_type {
            Some(KccFactor {
                name: "thread_specification".to_string(),
                points: 2,
                reason: format!("Thread callout: {designation}"),
            })
        } else {
            None
        }
    }
}

// === Rule 10: Multiple GD&T Controls (2 points) ===
pub struct MultipleGdtControlsRule;

impl ScoringRule for MultipleGdtControlsRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor> {
        let frames = get_feature_gdt_frames(feature, context);
        if frames.len() >= 2 {
            Some(KccFactor {
                name: "multiple_gdt_controls".to_string(),
                points: 2,
                reason: format!("{} feature control frames applied", frames.len()),
            })
        } else {
            None
        }
    }
}

// === Rule 11: Surface Finish Specified (1 point) ===
pub struct SurfaceFinishRule;

impl ScoringRule for SurfaceFinishRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor> {
        // Check annotations for surface finish symbols
        for ann in &context.drawing.annotations {
            let dist = crate::geometry::utils::distance(&ann.position, &feature.centroid);
            if dist < 20.0 {
                let text = &ann.text;
                if text.contains("Ra") || text.contains("Rz") || text.contains('\u{221A}') {
                    return Some(KccFactor {
                        name: "surface_finish".to_string(),
                        points: 1,
                        reason: "Surface finish specified".to_string(),
                    });
                }
            }
        }
        None
    }
}

// === Rule 12: Datum Feature (5 points) ===
pub struct DatumFeatureRule;

impl ScoringRule for DatumFeatureRule {
    fn evaluate(&self, feature: &Feature, context: &ScoringContext) -> Option<KccFactor> {
        // Check if this feature IS a datum
        for datum in context.datums {
            let dist = crate::geometry::utils::distance(&datum.position, &feature.centroid);
            if dist < 5.0 {
                return Some(KccFactor {
                    name: "datum_feature".to_string(),
                    points: 5,
                    reason: format!("Feature is datum {}", datum.label),
                });
            }
        }
        None
    }
}

// === Helper functions ===

/// Get the tolerance half-range and nominal size for a feature.
fn get_feature_tolerance(feature: &Feature, context: &ScoringContext) -> Option<(f64, f64)> {
    // Look through the drawing dimensions for ones attached to this feature
    for dim in &context.drawing.dimensions {
        if feature.dimensions.contains(&dim.id)
            || feature
                .geometry_refs
                .iter()
                .any(|r| dim.attached_entities.contains(r))
        {
            if let Some(ref tol) = dim.tolerance {
                return Some((tol.half_range(), dim.nominal_value));
            }
        }
    }
    None
}

/// Get GD&T frames associated with a feature (by spatial proximity).
fn get_feature_gdt_frames<'a>(
    feature: &Feature,
    context: &'a ScoringContext,
) -> Vec<&'a crate::geometry::model::FeatureControlFrame> {
    context
        .drawing
        .gdt_frames
        .iter()
        .filter(|f| {
            let dist = crate::geometry::utils::distance(&f.position, &feature.centroid);
            dist < 20.0 || feature.gdt_frames.contains(&f.id)
        })
        .collect()
}

/// Get all 12 scoring rules.
pub fn all_rules() -> Vec<Box<dyn ScoringRule>> {
    vec![
        Box::new(TightToleranceRule),
        Box::new(VeryTightToleranceRule),
        Box::new(DatumReferenceRule),
        Box::new(GdtControlledRule),
        Box::new(PositionToleranceRule),
        Box::new(PatternMemberRule),
        Box::new(AssemblyInterfaceRule),
        Box::new(ToleranceChainCriticalRule),
        Box::new(ThreadSpecificationRule),
        Box::new(MultipleGdtControlsRule),
        Box::new(SurfaceFinishRule),
        Box::new(DatumFeatureRule),
    ]
}
