use super::types::{Feature, FeatureId, FeatureType};
use crate::geometry::model::Drawing;
use crate::geometry::utils;

/// Detect thread callouts from annotations and link to holes.
pub fn detect_threads(drawing: &Drawing, holes: &[Feature]) -> Vec<Feature> {
    let mut threads = Vec::new();

    for annotation in &drawing.annotations {
        if let Some(thread) = parse_thread_callout(&annotation.text) {
            // Find the nearest hole to link this thread to
            let nearest_hole = holes.iter().min_by(|a, b| {
                let dist_a = utils::distance(&annotation.position, &a.centroid);
                let dist_b = utils::distance(&annotation.position, &b.centroid);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let (centroid, geometry_refs) = match nearest_hole {
                Some(hole) if utils::distance(&annotation.position, &hole.centroid) < 50.0 => {
                    (hole.centroid, hole.geometry_refs.clone())
                }
                _ => (annotation.position, vec![annotation.id]),
            };

            threads.push(Feature {
                id: FeatureId(0),
                feature_type: FeatureType::Thread {
                    designation: thread.designation,
                    nominal_diameter: thread.nominal_diameter,
                    pitch: thread.pitch,
                },
                geometry_refs,
                centroid,
                dimensions: Vec::new(),
                gdt_frames: Vec::new(),
                datum_refs: Vec::new(),
                layer_hint: Some(annotation.layer.clone()),
            });
        }
    }

    threads
}

struct ParsedThread {
    designation: String,
    nominal_diameter: f64,
    pitch: f64,
}

/// Parse metric thread callout: M8x1.25, M10, M6x1
fn parse_thread_callout(text: &str) -> Option<ParsedThread> {
    let text = text.trim();

    // Metric thread: M followed by diameter, optional x pitch
    if text.starts_with('M') || text.starts_with('m') {
        let rest = &text[1..];
        let parts: Vec<&str> = rest.splitn(2, ['x', 'X']).collect();

        let diameter_str = parts[0].trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.');
        let diameter: f64 = diameter_str.parse().ok()?;

        let pitch = if parts.len() > 1 {
            let pitch_str = parts[1]
                .trim()
                .trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.');
            pitch_str.parse().unwrap_or(standard_pitch(diameter))
        } else {
            standard_pitch(diameter)
        };

        return Some(ParsedThread {
            designation: text.to_string(),
            nominal_diameter: diameter,
            pitch,
        });
    }

    None
}

/// Standard coarse pitch for common metric thread sizes.
fn standard_pitch(diameter: f64) -> f64 {
    match diameter as u32 {
        3 => 0.5,
        4 => 0.7,
        5 => 0.8,
        6 => 1.0,
        8 => 1.25,
        10 => 1.5,
        12 => 1.75,
        14 | 16 => 2.0,
        18 | 20 => 2.5,
        22 | 24 => 3.0,
        _ => 1.0, // default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metric_thread() {
        let t = parse_thread_callout("M8x1.25").unwrap();
        assert_eq!(t.designation, "M8x1.25");
        assert!((t.nominal_diameter - 8.0).abs() < 1e-6);
        assert!((t.pitch - 1.25).abs() < 1e-6);
    }

    #[test]
    fn test_parse_metric_thread_no_pitch() {
        let t = parse_thread_callout("M10").unwrap();
        assert!((t.nominal_diameter - 10.0).abs() < 1e-6);
        assert!((t.pitch - 1.5).abs() < 1e-6); // standard coarse pitch
    }

    #[test]
    fn test_not_a_thread() {
        assert!(parse_thread_callout("25.00").is_none());
    }
}
