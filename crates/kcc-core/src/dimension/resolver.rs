use crate::geometry::model::{DimensionType, Drawing, GeometryPrimitive};
use crate::geometry::spatial::SpatialIndex;

/// Default search radius for linking dimensions to geometry (mm).
const LINK_SEARCH_RADIUS: f64 = 5.0;

/// Link dimensions to their target geometry entities using spatial proximity.
pub fn link_dimensions_to_geometry(drawing: &mut Drawing, index: &SpatialIndex) {
    for dim in &mut drawing.dimensions {
        if !dim.attached_entities.is_empty() {
            continue; // Already linked
        }

        let mut candidates: Vec<(crate::geometry::model::EntityId, f64)> = Vec::new();

        // Search near each definition point
        for def_point in &dim.definition_points {
            let nearby = index.query_radius_with_distance(def_point, LINK_SEARCH_RADIUS);
            for (entity_id, dist) in nearby {
                candidates.push((entity_id, dist));
            }
        }

        // Also search near the text position
        let text_nearby =
            index.query_radius_with_distance(&dim.text_position, LINK_SEARCH_RADIUS * 2.0);
        for (entity_id, dist) in text_nearby {
            candidates.push((entity_id, dist * 2.0)); // Penalize text-based matches
        }

        // Score and rank candidates
        let mut scored: Vec<(crate::geometry::model::EntityId, f64)> = candidates
            .iter()
            .map(|(id, dist)| {
                let mut score = 1.0 / (dist + 0.001);

                // Type compatibility bonus
                if let Some(entity) = drawing.entities.iter().find(|e| e.id == *id) {
                    match (&dim.dim_type, &entity.geometry) {
                        (DimensionType::Diameter, GeometryPrimitive::Circle { .. }) => {
                            score *= 10.0
                        }
                        (DimensionType::Radius, GeometryPrimitive::Circle { .. }) => score *= 10.0,
                        (DimensionType::Radius, GeometryPrimitive::Arc { .. }) => score *= 10.0,
                        (DimensionType::Linear, GeometryPrimitive::Line { .. }) => score *= 5.0,
                        (DimensionType::Angular, GeometryPrimitive::Line { .. }) => score *= 5.0,
                        _ => {}
                    }

                    // Layer compatibility bonus
                    if entity.layer == dim.layer {
                        score *= 2.0;
                    }
                }

                (*id, score)
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate
        let mut seen = std::collections::HashSet::new();
        scored.retain(|(id, _)| seen.insert(*id));

        // Take the best match(es)
        let best: Vec<_> = scored.iter().take(2).map(|(id, _)| *id).collect();
        dim.attached_entities = best;
    }
}

/// Try to parse dimension text and update the dimension's tolerance field.
pub fn parse_dimension_texts(drawing: &mut Drawing) {
    for dim in &mut drawing.dimensions {
        if let Some(ref text) = dim.text_override {
            if let Ok(parsed) = super::parser::parse_dimension_text(text) {
                if dim.nominal_value == 0.0 {
                    dim.nominal_value = parsed.nominal;
                }
                if dim.tolerance.is_none() {
                    dim.tolerance = parsed.tolerance;
                }
            }
        }
    }
}
