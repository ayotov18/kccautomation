//! Drawing-type detection from layer / block / annotation text.
//!
//! Pre-extraction signal so we can:
//!   - Skip detectors that produce noise on the wrong drawing class
//!     (e.g. `steel_detector` mis-flagging every wall pair as a Steel Member
//!     on architectural floor plans).
//!   - Drive the AI pipeline down the right SEK / pricing branch.
//!
//! The detector reads only text — never geometry. That's deliberate: the
//! geometry-only signals were exactly what produced the false-positive
//! cascade we are replacing.

use crate::geometry::model::Drawing;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DrawingType {
    /// Wood frame / KVH / BSH / OSB construction (residential cabins, sheds).
    Timber,
    /// Brick, concrete, plaster, tile, façade — generic architectural BoQ.
    Architectural,
    /// Steel fabrication: I-beams, gussets, bolt groups, weld preps.
    Steel,
    /// Machined parts (the original KCC use case).
    Mechanical,
    /// No strong signal — fall back to broad architectural prompt.
    Unknown,
}

impl DrawingType {
    /// Whether the steel-feature detector should run for this drawing.
    /// Architectural and Timber drawings are pure noise for it: every wall
    /// is a parallel-line pair, every room outline is a closed polygon.
    pub fn allows_steel_detector(&self) -> bool {
        matches!(self, Self::Steel | Self::Mechanical | Self::Unknown)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Timber => "timber",
            Self::Architectural => "architectural",
            Self::Steel => "steel",
            Self::Mechanical => "mechanical",
            Self::Unknown => "unknown",
        }
    }
}

/// Convenience: classify directly from a parsed Drawing using its entity
/// layer set + annotations. (Drawing has no block list — pass blocks via
/// [`classify_from_text`] when querying the DB.)
pub fn classify_drawing(drawing: &Drawing) -> DrawingType {
    let layers: Vec<String> = drawing
        .entities
        .iter()
        .map(|e| e.layer.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let annotations: Vec<String> = drawing.annotations.iter().map(|a| a.text.clone()).collect();
    classify_from_text(&layers, &[], &annotations)
}

/// Score-based classifier. Each matched keyword adds weighted points to
/// its category. Highest score wins; ties or empty signal → Unknown.
pub fn classify_from_text(layers: &[String], blocks: &[String], annotations: &[String]) -> DrawingType {
    let mut timber = 0i32;
    let mut arch = 0i32;
    let mut steel = 0i32;
    let mut mech = 0i32;

    let all_text = layers
        .iter()
        .chain(blocks.iter())
        .chain(annotations.iter())
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");

    // ── Timber signals (weighted heavily — narrow vocabulary) ──
    for kw in [
        "kvh", "bsh", "osb", "wood", "timber", "plywood", "shperplat",
        "шперплат", "дъск", "греда", "constr-wood", "constr_wood",
        "constr wood", "planking", "neopor", "топлоизолац", "минерал",
        "mineral wool", "ламарина", "metal sheet", "kalkan",
    ] {
        if all_text.contains(kw) {
            timber += 3;
        }
    }

    // ── Architectural (residential / building) signals ──
    for kw in [
        "a-wall", "a-door", "a-window", "a-furn", "a-isol", "a-text",
        "a-marks", "a-axes", "a-hatch", "a-planking", "a-constr", "a-view",
        "wall-", "door", "window", "dograma", "врата", "прозорец", "стен",
        "etazh", "floor", "plan", "kota", "kotirov", "наклон",
        "тоалет", "баня", "кухня", "спалня", "хол", "коридор", "стълб",
        "mövel", "movel", "cama", "bed", "furn",
    ] {
        if all_text.contains(kw) {
            arch += 1;
        }
    }

    // ── Steel-fabrication signals (require tokenised matches to avoid
    //    accidentally hitting "ipe" inside "swipe", etc.) ──
    let steel_kws = [
        "steel", "ipe", "heb", "hea", "upn", "rhs", "shs", "gusset",
        "стомана", "профил ipe", "профил heb", "ферма", "болт",
        "заваръч", "заваряван",
    ];
    let token_set: HashSet<&str> = all_text
        .split(|c: char| !c.is_alphanumeric())
        .collect();
    for kw in steel_kws {
        // Substring is fine for multi-word phrases; token match for short ones.
        let hit = if kw.len() <= 4 {
            token_set.contains(kw)
        } else {
            all_text.contains(kw)
        };
        if hit {
            steel += 3;
        }
    }

    // ── Machined-part / mechanical signals ──
    for kw in [
        "datum", "tolerance", "gd&t", "mmc", "lmc", "bolt-circle",
        "thread", "ra ", "surface finish", "нарязва",
    ] {
        if all_text.contains(kw) {
            mech += 2;
        }
    }

    let total: i32 = [timber, arch, steel, mech].iter().sum();
    if total == 0 {
        return DrawingType::Unknown;
    }

    // If timber AND architectural both fire, prefer Timber — wood-frame
    // drawings always look architectural too, but the converse is rarer.
    if timber >= 6 {
        return DrawingType::Timber;
    }
    if steel >= 6 && steel >= timber + arch {
        return DrawingType::Steel;
    }
    if mech >= 6 && mech > arch {
        return DrawingType::Mechanical;
    }
    if arch >= 4 {
        return DrawingType::Architectural;
    }
    DrawingType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_timber_cabin_from_real_layers() {
        // From the actual TASOS prod sample.
        let layers = vec![
            "KOTI".into(), "A-FURN".into(), "K-OS-GR".into(), "A-Text".into(),
            "A-Planking".into(), "A-Marks".into(), "A-CONSTR-Wood".into(),
            "A-DOORS".into(), "A-WALL".into(), "A-ISOL".into(),
        ];
        let blocks = vec!["MÓVEL - cama queen".into(), "vrata_70_20".into(), "WIN DIM TAG".into()];
        let t = classify_from_text(&layers, &blocks, &[]);
        assert_eq!(t, DrawingType::Timber);
        assert!(!t.allows_steel_detector());
    }

    #[test]
    fn detects_steel_when_explicit() {
        let layers = vec!["STEEL-BEAM".into(), "STEEL-COL".into(), "STEEL-PLATE".into()];
        let blocks = vec!["IPE-200".into(), "HEB-300".into(), "GUSSET-A".into()];
        let t = classify_from_text(&layers, &blocks, &[]);
        assert_eq!(t, DrawingType::Steel);
        assert!(t.allows_steel_detector());
    }

    #[test]
    fn unknown_is_permissive() {
        let t = classify_from_text(&["LAYER1".into()], &["BLOCK_X".into()], &[]);
        assert_eq!(t, DrawingType::Unknown);
        // Unknown still allows steel detection (preserves prior behaviour).
        assert!(t.allows_steel_detector());
    }
}
