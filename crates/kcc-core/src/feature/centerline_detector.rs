use super::types::{Feature, FeatureId, FeatureType};
use crate::geometry::model::{Drawing, GeometryPrimitive};

/// Detect centerlines from entities on CENTER layers or with center linetypes.
pub fn detect_centerlines(drawing: &Drawing) -> Vec<Feature> {
    let mut centerlines = Vec::new();

    for entity in &drawing.entities {
        let layer_lower = entity.layer.to_lowercase();
        let linetype_lower = entity.linetype.as_deref().unwrap_or("").to_lowercase();

        let is_centerline = layer_lower.contains("center")
            || layer_lower == "cl"
            || linetype_lower.contains("center")
            || linetype_lower.contains("dashdot");

        if !is_centerline {
            continue;
        }

        if let GeometryPrimitive::Line { start, end } = &entity.geometry {
            centerlines.push(Feature {
                id: FeatureId(0),
                feature_type: FeatureType::Centerline {
                    axis: ((start.x, start.y), (end.x, end.y)),
                },
                geometry_refs: vec![entity.id],
                centroid: crate::geometry::utils::midpoint(start, end),
                dimensions: Vec::new(),
                gdt_frames: Vec::new(),
                datum_refs: Vec::new(),
                layer_hint: Some(entity.layer.clone()),
            });
        }
    }

    centerlines
}
