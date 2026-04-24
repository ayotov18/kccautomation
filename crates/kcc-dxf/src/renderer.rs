use kcc_core::AnalysisResult;
use kcc_core::geometry::model::{Drawing, Dimension, DimensionType, GeometryPrimitive, Point2D};
use serde::Serialize;
use std::collections::HashMap;

/// Render packet for the frontend Canvas renderer.
#[derive(Debug, Clone, Serialize)]
pub struct RenderPacket {
    pub bounds: Bounds,
    pub layers: Vec<RenderLayer>,
    pub features: Vec<RenderFeature>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderLayer {
    pub name: String,
    pub color: String,
    pub entities: Vec<StyledEntity>,
}

/// A render entity with optional per-entity styling overrides.
#[derive(Debug, Clone, Serialize)]
pub struct StyledEntity {
    #[serde(flatten)]
    pub entity: RenderEntity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineweight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linetype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<u64>,
}

impl StyledEntity {
    fn plain(entity: RenderEntity) -> Self {
        Self { entity, color: None, lineweight: None, linetype: None, entity_id: None }
    }

    fn styled(entity: RenderEntity, color: Option<String>, lineweight: Option<f64>, linetype: Option<String>, entity_id: u64) -> Self {
        Self { entity, color, lineweight, linetype, entity_id: Some(entity_id) }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum RenderEntity {
    #[serde(rename = "line")]
    Line { x1: f64, y1: f64, x2: f64, y2: f64 },
    #[serde(rename = "circle")]
    Circle { cx: f64, cy: f64, r: f64 },
    #[serde(rename = "arc")]
    Arc {
        cx: f64,
        cy: f64,
        r: f64,
        start: f64,
        end: f64,
    },
    #[serde(rename = "polyline")]
    Polyline { points: Vec<[f64; 2]>, closed: bool },
    #[serde(rename = "text")]
    Text {
        x: f64,
        y: f64,
        text: String,
        height: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotation: Option<f64>,
    },
    #[serde(rename = "ellipse")]
    Ellipse {
        cx: f64,
        cy: f64,
        rx: f64,
        ry: f64,
        rotation: f64,
        start: f64,
        end: f64,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderFeature {
    pub id: String,
    #[serde(rename = "type")]
    pub feature_type: String,
    pub classification: String,
    pub cx: f64,
    pub cy: f64,
    pub highlight_entities: Vec<u64>,
}

/// Generate a render packet from a drawing and analysis result.
pub fn generate_render_packet(drawing: &Drawing, analysis: &AnalysisResult) -> RenderPacket {
    let mut layers_map: HashMap<String, Vec<StyledEntity>> = HashMap::new();

    let mut geo_min_x = f64::MAX;
    let mut geo_min_y = f64::MAX;
    let mut geo_max_x = f64::MIN;
    let mut geo_max_y = f64::MIN;

    // Convert entities to render commands grouped by layer
    for entity in &drawing.entities {
        let ent_color = entity.color.and_then(aci_to_hex);
        let ent_lw = entity.lineweight;
        let ent_lt = entity.linetype.clone();
        let ent_id = entity.id.0;

        let render_entity = match &entity.geometry {
            GeometryPrimitive::Line { start, end } => {
                if !is_valid_coord(start.x, start.y) || !is_valid_coord(end.x, end.y) {
                    continue;
                }
                update_geo_bounds(&mut geo_min_x, &mut geo_min_y, &mut geo_max_x, &mut geo_max_y, start.x, start.y);
                update_geo_bounds(&mut geo_min_x, &mut geo_min_y, &mut geo_max_x, &mut geo_max_y, end.x, end.y);
                RenderEntity::Line {
                    x1: start.x, y1: start.y,
                    x2: end.x, y2: end.y,
                }
            }
            GeometryPrimitive::Circle { center, radius } => {
                if *radius <= 0.0 || !radius.is_finite() || !is_valid_coord(center.x, center.y) {
                    continue;
                }
                update_geo_bounds(&mut geo_min_x, &mut geo_min_y, &mut geo_max_x, &mut geo_max_y, center.x - radius, center.y - radius);
                update_geo_bounds(&mut geo_min_x, &mut geo_min_y, &mut geo_max_x, &mut geo_max_y, center.x + radius, center.y + radius);
                RenderEntity::Circle { cx: center.x, cy: center.y, r: *radius }
            }
            GeometryPrimitive::Arc { center, radius, start_angle, end_angle } => {
                if *radius <= 0.0 || !radius.is_finite() || !start_angle.is_finite() || !end_angle.is_finite() || !is_valid_coord(center.x, center.y) {
                    continue;
                }
                update_geo_bounds(&mut geo_min_x, &mut geo_min_y, &mut geo_max_x, &mut geo_max_y, center.x - radius, center.y - radius);
                update_geo_bounds(&mut geo_min_x, &mut geo_min_y, &mut geo_max_x, &mut geo_max_y, center.x + radius, center.y + radius);
                RenderEntity::Arc { cx: center.x, cy: center.y, r: *radius, start: *start_angle, end: *end_angle }
            }
            GeometryPrimitive::Polyline { points, bulges, closed } => {
                for p in points.iter() {
                    if is_valid_coord(p.x, p.y) {
                        update_geo_bounds(&mut geo_min_x, &mut geo_min_y, &mut geo_max_x, &mut geo_max_y, p.x, p.y);
                    }
                }
                let has_bulges = bulges.iter().any(|b| b.abs() > 1e-10);
                if has_bulges && points.len() >= 2 {
                    let entities = decompose_polyline_with_bulges(points, bulges, *closed);
                    for e in entities {
                        layers_map.entry(entity.layer.clone()).or_default().push(
                            StyledEntity::styled(e, ent_color.clone(), ent_lw, ent_lt.clone(), ent_id),
                        );
                    }
                    continue;
                }
                let render_points: Vec<[f64; 2]> = points.iter()
                    .filter(|p| is_valid_coord(p.x, p.y))
                    .map(|p| [p.x, p.y])
                    .collect();
                if render_points.len() < 2 { continue; }
                RenderEntity::Polyline { points: render_points, closed: *closed }
            }
            GeometryPrimitive::Point(p) => {
                if !is_valid_coord(p.x, p.y) { continue; }
                update_geo_bounds(&mut geo_min_x, &mut geo_min_y, &mut geo_max_x, &mut geo_max_y, p.x, p.y);
                RenderEntity::Circle { cx: p.x, cy: p.y, r: 0.5 }
            }
            GeometryPrimitive::Spline { control_points, knots, degree } => {
                let interp = interpolate_spline(control_points, knots, *degree);
                let render_points: Vec<[f64; 2]> = interp.iter()
                    .filter(|p| is_valid_coord(p.x, p.y))
                    .map(|p| {
                        update_geo_bounds(&mut geo_min_x, &mut geo_min_y, &mut geo_max_x, &mut geo_max_y, p.x, p.y);
                        [p.x, p.y]
                    })
                    .collect();
                if render_points.len() < 2 { continue; }
                RenderEntity::Polyline { points: render_points, closed: false }
            }
        };

        layers_map.entry(entity.layer.clone()).or_default().push(
            StyledEntity::styled(render_entity, ent_color, ent_lw, ent_lt, ent_id),
        );
    }

    // Add annotations as text entities with rotation
    for ann in &drawing.annotations {
        if !is_valid_coord(ann.position.x, ann.position.y) { continue; }
        let rotation = if ann.rotation.abs() > 1e-10 { Some(ann.rotation) } else { None };
        layers_map.entry(ann.layer.clone()).or_default().push(
            StyledEntity::plain(RenderEntity::Text {
                x: ann.position.x,
                y: ann.position.y,
                text: ann.text.clone(),
                height: ann.height,
                rotation,
            }),
        );
    }

    // Decompose dimensions into visual sub-entities
    for dim in &drawing.dimensions {
        let dim_entities = render_dimension(dim);
        for (layer, styled) in dim_entities {
            layers_map.entry(layer).or_default().push(styled);
        }
    }

    // Build layers
    let layers: Vec<RenderLayer> = layers_map.into_iter()
        .map(|(name, entities)| {
            let color = layer_color(&name);
            RenderLayer { name, color, entities }
        })
        .collect();

    // Build feature overlays
    let features: Vec<RenderFeature> = analysis.features.iter()
        .zip(analysis.kcc_results.iter())
        .map(|(feature, (_, score))| RenderFeature {
            id: format!("F-{:03}", feature.id.0),
            feature_type: feature.feature_type.name().to_string(),
            classification: score.classification.as_str().to_string(),
            cx: feature.centroid.x,
            cy: feature.centroid.y,
            highlight_entities: feature.geometry_refs.iter().map(|e| e.0).collect(),
        })
        .collect();

    let (min_x, min_y, max_x, max_y) = if geo_min_x < f64::MAX {
        (geo_min_x, geo_min_y, geo_max_x, geo_max_y)
    } else {
        (0.0, 0.0, 100.0, 100.0)
    };

    RenderPacket {
        bounds: Bounds { min_x, min_y, max_x, max_y },
        layers,
        features,
    }
}

// === Dimension decomposition ===

fn render_dimension(dim: &Dimension) -> Vec<(String, StyledEntity)> {
    let mut result = Vec::new();
    let layer = dim.layer.clone();
    let dim_color = Some("#00FF00".to_string()); // Green for dimensions

    if dim.definition_points.len() < 2 {
        // Fallback: just render text at text_position
        let text = dim.text_override.as_deref()
            .unwrap_or("")
            .to_string();
        let text = if text.is_empty() { format!("{:.2}", dim.nominal_value) } else { text };
        result.push((layer, StyledEntity {
            entity: RenderEntity::Text {
                x: dim.text_position.x,
                y: dim.text_position.y,
                text,
                height: 2.5,
                rotation: None,
            },
            color: dim_color,
            lineweight: None,
            linetype: None,
            entity_id: Some(dim.id.0),
        }));
        return result;
    }

    let p1 = dim.definition_points[0];
    let p2 = dim.definition_points[1];

    // Dimension line between definition points
    result.push((layer.clone(), StyledEntity {
        entity: RenderEntity::Line { x1: p1.x, y1: p1.y, x2: p2.x, y2: p2.y },
        color: dim_color.clone(),
        lineweight: Some(0.15),
        linetype: None,
        entity_id: Some(dim.id.0),
    }));

    // Extension lines from definition points to text position (simplified)
    let tp = dim.text_position;
    if dim.definition_points.len() >= 3 {
        let p3 = dim.definition_points[2];
        // Extension line 1: from p1 toward the dimension line
        result.push((layer.clone(), StyledEntity {
            entity: RenderEntity::Line { x1: p1.x, y1: p1.y, x2: p3.x, y2: p1.y },
            color: dim_color.clone(),
            lineweight: Some(0.1),
            linetype: None,
            entity_id: Some(dim.id.0),
        }));
        // Extension line 2: from p2 toward the dimension line
        result.push((layer.clone(), StyledEntity {
            entity: RenderEntity::Line { x1: p2.x, y1: p2.y, x2: p3.x, y2: p2.y },
            color: dim_color.clone(),
            lineweight: Some(0.1),
            linetype: None,
            entity_id: Some(dim.id.0),
        }));
    }

    // Dimension text
    let text = dim.text_override.as_deref().unwrap_or("").to_string();
    let text = if text.is_empty() {
        match dim.dim_type {
            DimensionType::Diameter => format!("\u{2300}{:.2}", dim.nominal_value),
            DimensionType::Radius => format!("R{:.2}", dim.nominal_value),
            DimensionType::Angular => format!("{:.1}\u{00B0}", dim.nominal_value),
            _ => format!("{:.2}", dim.nominal_value),
        }
    } else {
        text
    };

    result.push((layer, StyledEntity {
        entity: RenderEntity::Text {
            x: tp.x,
            y: tp.y,
            text,
            height: 2.5,
            rotation: None,
        },
        color: dim_color,
        lineweight: None,
        linetype: None,
        entity_id: Some(dim.id.0),
    }));

    result
}

// === Spline interpolation (de Boor) ===

fn interpolate_spline(control_points: &[Point2D], knots: &[f64], degree: u32) -> Vec<Point2D> {
    let n = control_points.len();
    let p = degree as usize;

    // Fallback: if not enough data for de Boor, return control points
    if n < 2 || knots.len() < n + p + 1 || p == 0 {
        return control_points.to_vec();
    }

    let t_start = knots[p];
    let t_end = knots[n]; // knots[n] for clamped splines

    if (t_end - t_start).abs() < 1e-10 {
        return control_points.to_vec();
    }

    let num_samples = 64;
    let mut points = Vec::with_capacity(num_samples + 1);

    for i in 0..=num_samples {
        let t = t_start + (t_end - t_start) * (i as f64 / num_samples as f64);
        let t = t.min(t_end - 1e-10); // clamp to avoid boundary issues

        if let Some(pt) = de_boor(t, p, control_points, knots) {
            points.push(pt);
        }
    }

    if points.is_empty() {
        control_points.to_vec()
    } else {
        points
    }
}

fn de_boor(t: f64, p: usize, control_points: &[Point2D], knots: &[f64]) -> Option<Point2D> {
    let n = control_points.len();

    // Find knot span k such that knots[k] <= t < knots[k+1]
    let mut k = p;
    for i in p..n {
        if i + 1 < knots.len() && t >= knots[i] && t < knots[i + 1] {
            k = i;
            break;
        }
    }

    // Copy relevant control points
    let mut d: Vec<Point2D> = (0..=p)
        .filter_map(|j| {
            let idx = k.checked_sub(p)? + j;
            control_points.get(idx).copied()
        })
        .collect();

    if d.len() != p + 1 {
        return None;
    }

    // De Boor recursion
    for r in 1..=p {
        for j in (r..=p).rev() {
            let i = k.checked_sub(p)? + j;
            let ki = *knots.get(i)?;
            let ki_pr = *knots.get(i + p + 1 - r)?;
            let denom = ki_pr - ki;
            if denom.abs() < 1e-14 {
                continue;
            }
            let alpha = (t - ki) / denom;
            d[j] = Point2D {
                x: (1.0 - alpha) * d[j - 1].x + alpha * d[j].x,
                y: (1.0 - alpha) * d[j - 1].y + alpha * d[j].y,
            };
        }
    }

    Some(d[p])
}

// === ACI Color Table ===

fn aci_to_hex(index: i32) -> Option<String> {
    let hex = match index {
        1 => "#FF0000",   // Red
        2 => "#FFFF00",   // Yellow
        3 => "#00FF00",   // Green
        4 => "#00FFFF",   // Cyan
        5 => "#0000FF",   // Blue
        6 => "#FF00FF",   // Magenta
        7 => "#FFFFFF",   // White/Black (context-dependent)
        8 => "#808080",   // Dark gray
        9 => "#C0C0C0",   // Light gray
        10 => "#FF0000",  11 => "#FF7F7F", 12 => "#CC0000", 13 => "#CC6666",
        14 => "#990000",  15 => "#994C4C",
        20 => "#FF3F00",  21 => "#FF9F7F", 22 => "#CC3300", 23 => "#CC7F66",
        30 => "#FF7F00",  31 => "#FFBF7F", 32 => "#CC6600", 33 => "#CC9966",
        40 => "#FFBF00",  41 => "#FFDF7F", 42 => "#CC9900", 43 => "#CCB266",
        50 => "#FFFF00",  51 => "#FFFF7F", 52 => "#CCCC00", 53 => "#CCCC66",
        60 => "#BFFF00",  70 => "#7FFF00",  80 => "#3FFF00",
        90 => "#00FF00",  100 => "#00FF3F", 110 => "#00FF7F",
        120 => "#00FFBF", 130 => "#00FFFF", 140 => "#00BFFF",
        150 => "#007FFF", 160 => "#003FFF", 170 => "#0000FF",
        180 => "#3F00FF", 190 => "#7F00FF", 200 => "#BF00FF",
        210 => "#FF00FF", 220 => "#FF00BF", 230 => "#FF007F",
        240 => "#FF003F",
        250 => "#333333", 251 => "#505050", 252 => "#696969",
        253 => "#808080", 254 => "#BFBFBF", 255 => "#FFFFFF",
        256 => return None, // ByLayer
        0 => return None,   // ByBlock
        _ => return None,
    };
    Some(hex.to_string())
}

// === Utilities ===

fn is_valid_coord(x: f64, y: f64) -> bool {
    x.is_finite() && y.is_finite()
}

fn update_geo_bounds(min_x: &mut f64, min_y: &mut f64, max_x: &mut f64, max_y: &mut f64, x: f64, y: f64) {
    *min_x = min_x.min(x);
    *min_y = min_y.min(y);
    *max_x = max_x.max(x);
    *max_y = max_y.max(y);
}

fn layer_color(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("dim") || lower.contains("dimension") {
        "#00FF00".to_string()
    } else if lower.contains("center") {
        "#FF00FF".to_string()
    } else if lower.contains("hidden") {
        "#808080".to_string()
    } else {
        "#FFFFFF".to_string()
    }
}

/// Decompose a polyline with bulge factors into a mix of Line and Arc render entities.
fn decompose_polyline_with_bulges(
    points: &[Point2D],
    bulges: &[f64],
    closed: bool,
) -> Vec<RenderEntity> {
    let mut entities = Vec::new();
    let n = points.len();
    if n < 2 { return entities; }

    let segment_count = if closed { n } else { n - 1 };

    for i in 0..segment_count {
        let p1 = &points[i];
        let p2 = &points[(i + 1) % n];

        if !is_valid_coord(p1.x, p1.y) || !is_valid_coord(p2.x, p2.y) { continue; }

        let bulge = if i < bulges.len() { bulges[i] } else { 0.0 };

        if bulge.abs() < 1e-10 {
            entities.push(RenderEntity::Line { x1: p1.x, y1: p1.y, x2: p2.x, y2: p2.y });
        } else {
            let dx = p2.x - p1.x;
            let dy = p2.y - p1.y;
            let chord = (dx * dx + dy * dy).sqrt();
            if chord < 1e-10 { continue; }

            let theta = 4.0 * bulge.atan();
            let half_theta = theta / 2.0;
            if half_theta.sin().abs() < 1e-10 {
                entities.push(RenderEntity::Line { x1: p1.x, y1: p1.y, x2: p2.x, y2: p2.y });
                continue;
            }

            let radius = (chord / 2.0) / half_theta.sin().abs();
            let mx = (p1.x + p2.x) / 2.0;
            let my = (p1.y + p2.y) / 2.0;
            let ux = -dy / chord;
            let uy = dx / chord;
            let d = (chord / 2.0) / half_theta.tan();
            let cx = mx + d * ux;
            let cy = my + d * uy;
            let start_angle = (p1.y - cy).atan2(p1.x - cx);
            let end_angle = (p2.y - cy).atan2(p2.x - cx);

            if radius.is_finite() && radius > 0.0 {
                entities.push(RenderEntity::Arc { cx, cy, r: radius, start: start_angle, end: end_angle });
            }
        }
    }

    entities
}
