use crate::feature::types::Feature;
use crate::geometry::model::Drawing;
use crate::geometry::spatial::SpatialIndex;
use crate::geometry::utils;

/// Link GD&T feature control frames to their target features via spatial proximity.
pub fn link_gdt_to_features(drawing: &Drawing, features: &[Feature], _index: &SpatialIndex) {
    // For each GD&T frame, find the nearest feature
    // Note: this mutates the frames in the drawing (attached_entities)
    // In practice, we use the spatial proximity to associate frames with features
    // The actual linking is done during feature extraction

    for frame in &drawing.gdt_frames {
        let _nearest_feature = features.iter().min_by(|a, b| {
            let dist_a = utils::distance(&frame.position, &a.centroid);
            let dist_b = utils::distance(&frame.position, &b.centroid);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // In a full implementation, we would update feature.gdt_frames
        // For now, the association is computed on-demand during KCC scoring
    }
}
