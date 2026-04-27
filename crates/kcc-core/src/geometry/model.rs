use serde::{Deserialize, Serialize};
use std::fmt;

// === Units & Coordinates ===

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    Millimeters,
    Centimeters,
    Meters,
    Inches,
    Unitless,
}

impl Units {
    /// Multiply drawing coordinates by this factor to get millimeters.
    pub fn to_mm_scale(&self) -> f64 {
        match self {
            Units::Millimeters => 1.0,
            Units::Centimeters => 10.0,
            Units::Meters => 1000.0,
            Units::Inches => 25.4,
            Units::Unitless => 1.0, // assume mm by default; heuristic applied elsewhere
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

// === Entity identification ===

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E-{:04}", self.0)
    }
}

// === Geometry primitives ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeometryPrimitive {
    Line {
        start: Point2D,
        end: Point2D,
    },
    Arc {
        center: Point2D,
        radius: f64,
        start_angle: f64, // radians
        end_angle: f64,   // radians
    },
    Circle {
        center: Point2D,
        radius: f64,
    },
    Polyline {
        points: Vec<Point2D>,
        bulges: Vec<f64>, // bulge factor per segment (0 = line, nonzero = arc)
        closed: bool,
    },
    Spline {
        control_points: Vec<Point2D>,
        knots: Vec<f64>,
        degree: u32,
    },
    Point(Point2D),
}

// === Drawing entity (geometry + metadata) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub geometry: GeometryPrimitive,
    pub layer: String,
    pub color: Option<i32>, // ACI color index
    pub lineweight: Option<f64>,
    pub linetype: Option<String>,
    pub block_ref: Option<String>, // if this entity came from a block INSERT
}

// === Dimensions ===

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DimensionType {
    Linear,
    Aligned,
    Angular,
    Diameter,
    Radius,
    Ordinate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tolerance {
    pub upper: f64,
    pub lower: f64, // negative for minus tolerance
    pub is_symmetric: bool,
}

impl Tolerance {
    pub fn symmetric(value: f64) -> Self {
        Self {
            upper: value,
            lower: -value,
            is_symmetric: true,
        }
    }

    pub fn asymmetric(upper: f64, lower: f64) -> Self {
        Self {
            upper,
            lower,
            is_symmetric: false,
        }
    }

    /// Half-range of the tolerance band.
    pub fn half_range(&self) -> f64 {
        (self.upper - self.lower) / 2.0
    }

    /// Total tolerance band width.
    pub fn band_width(&self) -> f64 {
        self.upper - self.lower
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    pub id: EntityId,
    pub dim_type: DimensionType,
    pub nominal_value: f64,
    pub text_override: Option<String>, // raw text from DXF
    pub tolerance: Option<Tolerance>,
    pub definition_points: Vec<Point2D>, // DXF definition points
    pub text_position: Point2D,
    pub layer: String,
    pub attached_entities: Vec<EntityId>, // resolved geometry refs
}

// === GD&T ===

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GdtSymbol {
    Flatness,
    Straightness,
    Circularity,
    Cylindricity,
    Parallelism,
    Perpendicularity,
    Angularity,
    Position,
    Concentricity,
    Symmetry,
    RunoutCircular,
    RunoutTotal,
    ProfileLine,
    ProfileSurface,
}

impl fmt::Display for GdtSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GdtSymbol::Flatness => "Flatness",
            GdtSymbol::Straightness => "Straightness",
            GdtSymbol::Circularity => "Circularity",
            GdtSymbol::Cylindricity => "Cylindricity",
            GdtSymbol::Parallelism => "Parallelism",
            GdtSymbol::Perpendicularity => "Perpendicularity",
            GdtSymbol::Angularity => "Angularity",
            GdtSymbol::Position => "Position",
            GdtSymbol::Concentricity => "Concentricity",
            GdtSymbol::Symmetry => "Symmetry",
            GdtSymbol::RunoutCircular => "Circular Runout",
            GdtSymbol::RunoutTotal => "Total Runout",
            GdtSymbol::ProfileLine => "Profile of a Line",
            GdtSymbol::ProfileSurface => "Profile of a Surface",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialCondition {
    None,
    MaximumMaterial,     // (M)
    LeastMaterial,       // (L)
    RegardlessOfFeature, // (S)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatumReference {
    pub label: char, // A, B, C, etc.
    pub material_condition: MaterialCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureControlFrame {
    pub id: EntityId,
    pub symbol: GdtSymbol,
    pub tolerance_value: f64,
    pub material_condition: MaterialCondition,
    pub datum_refs: Vec<DatumReference>,
    pub position: Point2D,
    pub attached_entities: Vec<EntityId>,
    pub projected_tolerance: bool,
    pub is_diameter_zone: bool,
}

// === Annotations ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: EntityId,
    pub text: String,
    pub position: Point2D,
    pub height: f64,
    pub rotation: f64,
    pub layer: String,
}

// === Datums ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Datum {
    pub label: char,
    pub attached_entity: Option<EntityId>,
    pub position: Point2D,
}

// === Top-level drawing ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingMetadata {
    pub filename: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub scale: Option<f64>,
    pub sheet_size: Option<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawing {
    pub units: Units,
    pub entities: Vec<Entity>,
    pub dimensions: Vec<Dimension>,
    pub gdt_frames: Vec<FeatureControlFrame>,
    pub annotations: Vec<Annotation>,
    pub datums: Vec<Datum>,
    pub metadata: DrawingMetadata,
    /// Detected spatial modules in this drawing. Always at least one when the
    /// drawing has entities; multi-module sheets (side-by-side floor plans)
    /// produce N entries that downstream takeoff/KSS pipelines iterate over.
    /// Populated by `kcc_core::geometry::structure::detect_structures`.
    #[serde(default)]
    pub structures: Vec<super::structure::Structure>,
}

impl Drawing {
    pub fn new(filename: String) -> Self {
        Self {
            units: Units::Millimeters,
            entities: Vec::new(),
            dimensions: Vec::new(),
            gdt_frames: Vec::new(),
            annotations: Vec::new(),
            datums: Vec::new(),
            metadata: DrawingMetadata {
                filename,
                title: None,
                author: None,
                scale: None,
                sheet_size: None,
            },
            structures: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tolerance_symmetric() {
        let tol = Tolerance::symmetric(0.05);
        assert!((tol.upper - 0.05).abs() < 1e-10);
        assert!((tol.lower + 0.05).abs() < 1e-10);
        assert!(tol.is_symmetric);
        assert!((tol.band_width() - 0.10).abs() < 1e-10);
        assert!((tol.half_range() - 0.05).abs() < 1e-10);
    }

    #[test]
    fn test_tolerance_asymmetric() {
        let tol = Tolerance::asymmetric(0.05, -0.02);
        assert!(!tol.is_symmetric);
        assert!((tol.band_width() - 0.07).abs() < 1e-10);
    }

    #[test]
    fn test_point2d() {
        let p = Point2D::new(3.0, 4.0);
        assert!((p.x - 3.0).abs() < 1e-10);
        assert!((p.y - 4.0).abs() < 1e-10);
    }
}
