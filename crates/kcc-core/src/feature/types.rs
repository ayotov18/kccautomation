use crate::geometry::model::{EntityId, Point2D};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeatureId(pub u64);

impl fmt::Display for FeatureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "F-{:03}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Outer,
    Inner,
    Chamfer,
    Fillet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureType {
    Hole {
        diameter: f64,
        is_through: bool,
    },
    Slot {
        width: f64,
        length: f64,
    },
    Edge {
        edge_type: EdgeType,
    },
    BoltCircle {
        hole_count: usize,
        hole_diameter: f64,
        pattern_diameter: f64,
    },
    LinearPattern {
        feature_count: usize,
        spacing: f64,
        direction: (f64, f64),
    },
    Thread {
        designation: String,
        nominal_diameter: f64,
        pitch: f64,
    },
    Centerline {
        axis: ((f64, f64), (f64, f64)),
    },
    Surface {
        area: f64,
        boundary: Vec<u64>,
    },
    /// Structural steel member (I-beam, channel, angle, etc.)
    SteelMember {
        length: f64,
        depth: f64,
        profile_hint: Option<String>,
    },
    /// Gusset plate or connection plate
    GussetPlate {
        area: f64,
        vertex_count: usize,
    },
    /// Bolt group (cluster of holes at regular spacing)
    BoltGroup {
        bolt_count: usize,
        bolt_diameter: f64,
        group_width: f64,
        group_height: f64,
    },
}

impl FeatureType {
    /// Human-readable name for the feature type.
    pub fn name(&self) -> &str {
        match self {
            FeatureType::Hole { .. } => "Hole",
            FeatureType::Slot { .. } => "Slot",
            FeatureType::Edge { .. } => "Edge",
            FeatureType::BoltCircle { .. } => "Bolt Circle",
            FeatureType::LinearPattern { .. } => "Linear Pattern",
            FeatureType::Thread { .. } => "Thread",
            FeatureType::Centerline { .. } => "Centerline",
            FeatureType::Surface { .. } => "Surface",
            FeatureType::SteelMember { .. } => "Steel Member",
            FeatureType::GussetPlate { .. } => "Gusset Plate",
            FeatureType::BoltGroup { .. } => "Bolt Group",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: FeatureId,
    pub feature_type: FeatureType,
    pub geometry_refs: Vec<EntityId>,
    pub centroid: Point2D,
    pub dimensions: Vec<EntityId>,
    pub gdt_frames: Vec<EntityId>,
    pub datum_refs: Vec<char>,
    pub layer_hint: Option<String>,
}

impl Feature {
    /// Generate a human-readable description of this feature.
    pub fn description(&self) -> String {
        match &self.feature_type {
            FeatureType::Hole {
                diameter,
                is_through,
            } => {
                let through = if *is_through { " THRU" } else { "" };
                format!("\u{2300}{:.2}{through}", diameter)
            }
            FeatureType::Slot { width, length } => {
                format!("{:.2}x{:.2} slot", width, length)
            }
            FeatureType::Edge { edge_type } => {
                format!("{edge_type:?} edge")
            }
            FeatureType::BoltCircle {
                hole_count,
                hole_diameter,
                pattern_diameter,
            } => {
                format!(
                    "{hole_count}X \u{2300}{hole_diameter:.2} on \u{2300}{pattern_diameter:.2} PCD"
                )
            }
            FeatureType::LinearPattern {
                feature_count,
                spacing,
                ..
            } => {
                format!("{feature_count}X @ {spacing:.2} spacing")
            }
            FeatureType::Thread { designation, .. } => designation.clone(),
            FeatureType::Centerline { .. } => "Centerline".to_string(),
            FeatureType::Surface { area, .. } => {
                format!("Surface {area:.2} mm\u{00B2}")
            }
            FeatureType::SteelMember { length, profile_hint, .. } => {
                if let Some(profile) = profile_hint {
                    format!("{profile} L={length:.0}mm")
                } else {
                    format!("Steel member L={length:.0}mm")
                }
            }
            FeatureType::GussetPlate { area, vertex_count } => {
                format!("{vertex_count}-sided plate {area:.0}mm\u{00B2}")
            }
            FeatureType::BoltGroup { bolt_count, bolt_diameter, .. } => {
                format!("{bolt_count}x \u{2300}{bolt_diameter:.1} bolt group")
            }
        }
    }
}

/// Collection of extracted features with auto-incrementing IDs.
pub struct FeatureSet {
    pub features: Vec<Feature>,
    next_id: u64,
}

impl FeatureSet {
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add(&mut self, feature: Feature) -> FeatureId {
        let id = FeatureId(self.next_id);
        self.next_id += 1;
        let mut f = feature;
        f.id = id;
        self.features.push(f);
        id
    }

    pub fn add_new(
        &mut self,
        feature_type: FeatureType,
        geometry_refs: Vec<EntityId>,
        centroid: Point2D,
        layer_hint: Option<String>,
    ) -> FeatureId {
        let id = FeatureId(self.next_id);
        self.next_id += 1;
        self.features.push(Feature {
            id,
            feature_type,
            geometry_refs,
            centroid,
            dimensions: Vec::new(),
            gdt_frames: Vec::new(),
            datum_refs: Vec::new(),
            layer_hint,
        });
        id
    }

    pub fn len(&self) -> usize {
        self.features.len()
    }

    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }
}

impl Default for FeatureSet {
    fn default() -> Self {
        Self::new()
    }
}
