use dxf::Drawing as DxfDrawing;
use dxf::entities::EntityType as DxfEntityType;
use kcc_core::geometry::model::*;
use kcc_core::geometry::utils::{clean_coordinate, deg_to_rad};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("DXF parse error: {0}")]
    Dxf(String),
}

/// Parse a DXF file from a file path.
pub fn parse_dxf_file(path: &Path) -> Result<Drawing, ParseError> {
    let dxf_drawing = DxfDrawing::load_file(path).map_err(|e| ParseError::Dxf(e.to_string()))?;
    convert_dxf(
        dxf_drawing,
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    )
}

/// Parse a DXF file from bytes.
pub fn parse_dxf_bytes(bytes: &[u8], filename: String) -> Result<Drawing, ParseError> {
    let mut cursor = std::io::Cursor::new(bytes);
    let dxf_drawing = DxfDrawing::load(&mut cursor).map_err(|e| ParseError::Dxf(e.to_string()))?;
    convert_dxf(dxf_drawing, filename)
}

/// Convert a parsed DXF drawing into the canonical model.
fn convert_dxf(dxf: DxfDrawing, filename: String) -> Result<Drawing, ParseError> {
    let mut drawing = Drawing::new(filename);
    let mut next_id: u64 = 1;

    // Detect units from header
    drawing.units = detect_units(&dxf);

    // Build dimension style tolerance map
    let dim_style_tolerances = build_dim_style_map(&dxf);

    // Process entities
    for entity in dxf.entities() {
        let common = &entity.common;
        let layer = common.layer.clone();
        let color: Option<i32> = common.color.index().map(|i| i as i32);
        let linetype = if common.line_type_name.is_empty() {
            None
        } else {
            Some(common.line_type_name.clone())
        };

        match &entity.specific {
            DxfEntityType::Line(line) => {
                let start = point_from_dxf(&line.p1);
                let end = point_from_dxf(&line.p2);
                drawing.entities.push(Entity {
                    id: EntityId(next_id),
                    geometry: GeometryPrimitive::Line { start, end },
                    layer,
                    color,
                    lineweight: None,
                    linetype,
                    block_ref: None,
                });
                next_id += 1;
            }
            DxfEntityType::Circle(circle) => {
                let center = point_from_dxf(&circle.center);
                drawing.entities.push(Entity {
                    id: EntityId(next_id),
                    geometry: GeometryPrimitive::Circle {
                        center,
                        radius: circle.radius,
                    },
                    layer,
                    color,
                    lineweight: None,
                    linetype,
                    block_ref: None,
                });
                next_id += 1;
            }
            DxfEntityType::Arc(arc) => {
                let center = point_from_dxf(&arc.center);
                drawing.entities.push(Entity {
                    id: EntityId(next_id),
                    geometry: GeometryPrimitive::Arc {
                        center,
                        radius: arc.radius,
                        start_angle: deg_to_rad(arc.start_angle),
                        end_angle: deg_to_rad(arc.end_angle),
                    },
                    layer,
                    color,
                    lineweight: None,
                    linetype,
                    block_ref: None,
                });
                next_id += 1;
            }
            DxfEntityType::LwPolyline(poly) => {
                let points: Vec<Point2D> = poly
                    .vertices
                    .iter()
                    .map(|v| Point2D::new(clean_coordinate(v.x), clean_coordinate(v.y)))
                    .collect();
                let bulges: Vec<f64> = poly.vertices.iter().map(|v| v.bulge).collect();
                drawing.entities.push(Entity {
                    id: EntityId(next_id),
                    geometry: GeometryPrimitive::Polyline {
                        points,
                        bulges,
                        closed: poly.is_closed(),
                    },
                    layer,
                    color,
                    lineweight: None,
                    linetype,
                    block_ref: None,
                });
                next_id += 1;
            }
            DxfEntityType::Text(text) => {
                let position = point_from_dxf(&text.location);
                drawing.annotations.push(Annotation {
                    id: EntityId(next_id),
                    text: text.value.clone(),
                    position,
                    height: text.text_height,
                    rotation: deg_to_rad(text.rotation),
                    layer,
                });
                next_id += 1;
            }
            DxfEntityType::MText(mtext) => {
                let position = point_from_dxf(&mtext.insertion_point);
                let text_content = clean_mtext(&mtext.text);

                // Check if this is GD&T content
                if is_gdt_content(&mtext.text) {
                    let frames = kcc_core::gdt::parser::parse_gdt_mtext(
                        &mtext.text,
                        position,
                        EntityId(next_id),
                    );
                    for frame in frames {
                        drawing.gdt_frames.push(frame);
                    }
                }

                drawing.annotations.push(Annotation {
                    id: EntityId(next_id),
                    text: text_content,
                    position,
                    height: mtext.initial_text_height,
                    rotation: deg_to_rad(mtext.rotation_angle),
                    layer,
                });
                next_id += 1;
            }
            // Handle different dimension types
            DxfEntityType::RotatedDimension(dim) => {
                parse_dimension_entity(
                    &dim.dimension_base,
                    DimensionType::Linear,
                    vec![
                        point_from_dxf(&dim.dimension_base.definition_point_1),
                        point_from_dxf(&dim.definition_point_2),
                        point_from_dxf(&dim.definition_point_3),
                    ],
                    &layer,
                    &mut drawing,
                    &mut next_id,
                    &dim_style_tolerances,
                );
            }
            DxfEntityType::RadialDimension(dim) => {
                parse_dimension_entity(
                    &dim.dimension_base,
                    DimensionType::Radius,
                    vec![
                        point_from_dxf(&dim.dimension_base.definition_point_1),
                        point_from_dxf(&dim.definition_point_2),
                    ],
                    &layer,
                    &mut drawing,
                    &mut next_id,
                    &dim_style_tolerances,
                );
            }
            DxfEntityType::DiameterDimension(dim) => {
                parse_dimension_entity(
                    &dim.dimension_base,
                    DimensionType::Diameter,
                    vec![
                        point_from_dxf(&dim.dimension_base.definition_point_1),
                        point_from_dxf(&dim.definition_point_2),
                    ],
                    &layer,
                    &mut drawing,
                    &mut next_id,
                    &dim_style_tolerances,
                );
            }
            DxfEntityType::AngularThreePointDimension(dim) => {
                parse_dimension_entity(
                    &dim.dimension_base,
                    DimensionType::Angular,
                    vec![
                        point_from_dxf(&dim.dimension_base.definition_point_1),
                        point_from_dxf(&dim.definition_point_2),
                        point_from_dxf(&dim.definition_point_3),
                    ],
                    &layer,
                    &mut drawing,
                    &mut next_id,
                    &dim_style_tolerances,
                );
            }
            DxfEntityType::OrdinateDimension(dim) => {
                parse_dimension_entity(
                    &dim.dimension_base,
                    DimensionType::Ordinate,
                    vec![
                        point_from_dxf(&dim.dimension_base.definition_point_1),
                        point_from_dxf(&dim.definition_point_2),
                    ],
                    &layer,
                    &mut drawing,
                    &mut next_id,
                    &dim_style_tolerances,
                );
            }
            DxfEntityType::Spline(spline) => {
                let control_points: Vec<Point2D> = spline
                    .control_points
                    .iter()
                    .map(point_from_dxf)
                    .collect();
                let knots: Vec<f64> = spline.knot_values.clone();
                if control_points.len() >= 2 {
                    drawing.entities.push(Entity {
                        id: EntityId(next_id),
                        geometry: GeometryPrimitive::Spline {
                            control_points,
                            knots,
                            degree: spline.degree_of_curve as u32,
                        },
                        layer,
                        color,
                        lineweight: None,
                        linetype,
                        block_ref: None,
                    });
                    next_id += 1;
                }
            }
            DxfEntityType::Ellipse(ellipse) => {
                let center = point_from_dxf(&ellipse.center);
                let points = approximate_ellipse(
                    &center,
                    ellipse.major_axis.x,
                    ellipse.major_axis.y,
                    ellipse.minor_axis_ratio,
                    ellipse.start_parameter,
                    ellipse.end_parameter,
                );
                if points.len() >= 2 {
                    let is_full =
                        (ellipse.end_parameter - ellipse.start_parameter - std::f64::consts::TAU)
                            .abs()
                            < 1e-6;
                    drawing.entities.push(Entity {
                        id: EntityId(next_id),
                        geometry: GeometryPrimitive::Polyline {
                            points,
                            bulges: Vec::new(),
                            closed: is_full,
                        },
                        layer,
                        color,
                        lineweight: None,
                        linetype,
                        block_ref: None,
                    });
                    next_id += 1;
                }
            }
            DxfEntityType::Solid(solid) => {
                let p1 = point_from_dxf(&solid.first_corner);
                let p2 = point_from_dxf(&solid.second_corner);
                let p3 = point_from_dxf(&solid.third_corner);
                let p4 = point_from_dxf(&solid.fourth_corner);
                let points = if distance_2d(&p3, &p4) < 1e-6 {
                    vec![p1, p2, p3]
                } else {
                    vec![p1, p2, p4, p3] // DXF solid corner order: 1,2,4,3
                };
                drawing.entities.push(Entity {
                    id: EntityId(next_id),
                    geometry: GeometryPrimitive::Polyline {
                        points,
                        bulges: Vec::new(),
                        closed: true,
                    },
                    layer,
                    color,
                    lineweight: None,
                    linetype,
                    block_ref: None,
                });
                next_id += 1;
            }
            DxfEntityType::Face3D(face) => {
                let p1 = point_from_dxf(&face.first_corner);
                let p2 = point_from_dxf(&face.second_corner);
                let p3 = point_from_dxf(&face.third_corner);
                let p4 = point_from_dxf(&face.fourth_corner);
                let points = if distance_2d(&p3, &p4) < 1e-6 {
                    vec![p1, p2, p3]
                } else {
                    vec![p1, p2, p3, p4]
                };
                drawing.entities.push(Entity {
                    id: EntityId(next_id),
                    geometry: GeometryPrimitive::Polyline {
                        points,
                        bulges: Vec::new(),
                        closed: true,
                    },
                    layer,
                    color,
                    lineweight: None,
                    linetype,
                    block_ref: None,
                });
                next_id += 1;
            }
            DxfEntityType::ModelPoint(point) => {
                let p = point_from_dxf(&point.location);
                drawing.entities.push(Entity {
                    id: EntityId(next_id),
                    geometry: GeometryPrimitive::Point(p),
                    layer,
                    color,
                    lineweight: None,
                    linetype,
                    block_ref: None,
                });
                next_id += 1;
            }
            DxfEntityType::Polyline(poly) => {
                // Old-style 2D/3D polyline (not LWPOLYLINE)
                let points: Vec<Point2D> = poly
                    .vertices()
                    .map(|v| point_from_dxf(&v.location))
                    .collect();
                let bulges: Vec<f64> = poly.vertices().map(|v| v.bulge).collect();
                if points.len() >= 2 {
                    drawing.entities.push(Entity {
                        id: EntityId(next_id),
                        geometry: GeometryPrimitive::Polyline {
                            points,
                            bulges,
                            closed: poly.is_closed(),
                        },
                        layer,
                        color,
                        lineweight: None,
                        linetype,
                        block_ref: None,
                    });
                    next_id += 1;
                }
            }
            DxfEntityType::Insert(insert) => {
                let block_name = insert.name.clone();
                let insert_point = point_from_dxf(&insert.location);
                let scale_x = insert.x_scale_factor;
                let scale_y = insert.y_scale_factor;
                let rotation = deg_to_rad(insert.rotation);

                flatten_block(
                    &dxf,
                    &block_name,
                    &mut next_id,
                    &layer,
                    &insert_point,
                    scale_x,
                    scale_y,
                    rotation,
                    0,
                    &mut drawing,
                );
            }
            DxfEntityType::Tolerance(tol) => {
                let position = point_from_dxf(&tol.insertion_point);
                let frames = kcc_core::gdt::parser::parse_tolerance_entity(
                    &tol.display_text,
                    position,
                    EntityId(next_id),
                );
                for frame in frames {
                    drawing.gdt_frames.push(frame);
                }
                next_id += 1;
            }
            _ => {
                tracing::debug!(
                    layer = %layer,
                    "Skipping unsupported top-level entity type"
                );
            }
        }
    }

    // Apply normalization
    super::normalizer::normalize(&mut drawing);

    // Detect spatial modules. Single-module drawings produce one structure;
    // multi-module sheets (side-by-side floor plans) produce N. Downstream
    // takeoff/KSS pipelines iterate over this list.
    drawing.structures = kcc_core::geometry::structure::detect_structures(&drawing);

    Ok(drawing)
}

/// Dimension style tolerance configuration extracted from the DIMSTYLE table.
struct DimStyleTolerance {
    generate_tolerances: bool,
    generate_limits: bool,
    plus_tolerance: f64,
    minus_tolerance: f64,
}

/// Build a map of dimension style name → tolerance configuration from the DXF DIMSTYLE table.
fn build_dim_style_map(dxf: &DxfDrawing) -> HashMap<String, DimStyleTolerance> {
    let mut map = HashMap::new();
    for style in dxf.dim_styles() {
        map.insert(
            style.name.clone(),
            DimStyleTolerance {
                generate_tolerances: style.generate_dimension_tolerances,
                generate_limits: style.generate_dimension_limits,
                plus_tolerance: style.dimension_plus_tolerance,
                minus_tolerance: style.dimension_minus_tolerance,
            },
        );
    }
    map
}

fn parse_dimension_entity(
    base: &dxf::entities::DimensionBase,
    dim_type: DimensionType,
    definition_points: Vec<Point2D>,
    layer: &str,
    drawing: &mut Drawing,
    next_id: &mut u64,
    dim_styles: &HashMap<String, DimStyleTolerance>,
) {
    let text_override = if base.text.is_empty() {
        None
    } else {
        Some(base.text.clone())
    };

    let nominal_value = if let Some(ref text) = text_override {
        kcc_core::dimension::parser::parse_dimension_text(text)
            .map(|p| p.nominal)
            .unwrap_or(base.actual_measurement)
    } else {
        base.actual_measurement
    };

    // Try to extract tolerance from dimension text override first
    let mut tolerance = text_override.as_ref().and_then(|text| {
        kcc_core::dimension::parser::parse_dimension_text(text)
            .ok()
            .and_then(|p| p.tolerance)
    });

    // If no tolerance from text, try the DIMSTYLE table
    if tolerance.is_none() {
        if let Some(style) = dim_styles.get(&base.dimension_style_name) {
            if style.generate_tolerances
                && (style.plus_tolerance.abs() > 1e-10 || style.minus_tolerance.abs() > 1e-10)
            {
                let is_symmetric =
                    (style.plus_tolerance - style.minus_tolerance.abs()).abs() < 1e-10;
                tolerance = Some(Tolerance {
                    upper: style.plus_tolerance,
                    lower: -style.minus_tolerance.abs(),
                    is_symmetric,
                });
            } else if style.generate_limits
                && (style.plus_tolerance.abs() > 1e-10 || style.minus_tolerance.abs() > 1e-10)
            {
                // Limit dimensions: DIMSTYLE stores the +/- values even when using limits mode
                tolerance = Some(Tolerance {
                    upper: style.plus_tolerance,
                    lower: -style.minus_tolerance.abs(),
                    is_symmetric: false,
                });
            }
        }
    }

    drawing.dimensions.push(Dimension {
        id: EntityId(*next_id),
        dim_type,
        nominal_value,
        text_override,
        tolerance,
        definition_points,
        text_position: point_from_dxf(&base.text_mid_point),
        layer: layer.to_string(),
        attached_entities: Vec::new(),
    });
    *next_id += 1;
}

/// Detect units from DXF header variable $INSUNITS.
fn detect_units(dxf: &DxfDrawing) -> Units {
    match dxf.header.default_drawing_units {
        dxf::enums::Units::Inches => Units::Inches,
        dxf::enums::Units::Millimeters => Units::Millimeters,
        dxf::enums::Units::Centimeters => Units::Centimeters,
        dxf::enums::Units::Meters => Units::Meters,
        dxf::enums::Units::Unitless => Units::Unitless,
        _ => Units::Millimeters,
    }
}

fn point_from_dxf(p: &dxf::Point) -> Point2D {
    Point2D::new(clean_coordinate(p.x), clean_coordinate(p.y))
}

fn clean_mtext(text: &str) -> String {
    let mut result = String::new();
    let mut in_format = false;
    let mut brace_depth = 0;

    for c in text.chars() {
        match c {
            '{' => {
                brace_depth += 1;
                in_format = true;
            }
            '}' => {
                brace_depth -= 1;
                if brace_depth <= 0 {
                    in_format = false;
                    brace_depth = 0;
                }
            }
            ';' if in_format => {
                in_format = false;
            }
            _ if !in_format => {
                result.push(c);
            }
            _ => {}
        }
    }

    result.trim().to_string()
}

fn is_gdt_content(text: &str) -> bool {
    text.contains("gdt;")
        || text.contains("GDT;")
        || text.contains('\u{2316}')
        || text.contains('\u{23E5}')
        || text.contains('\u{27C2}')
}

const MAX_BLOCK_DEPTH: usize = 10;

/// Recursively flatten a block INSERT into the drawing, handling nested blocks.
#[allow(clippy::too_many_arguments)]
fn flatten_block(
    dxf: &DxfDrawing,
    block_name: &str,
    next_id: &mut u64,
    parent_layer: &str,
    insert_point: &Point2D,
    scale_x: f64,
    scale_y: f64,
    rotation: f64,
    depth: usize,
    drawing: &mut Drawing,
) {
    if depth >= MAX_BLOCK_DEPTH {
        tracing::warn!(
            block = %block_name,
            depth,
            "Maximum block nesting depth reached, skipping"
        );
        return;
    }

    let block = match dxf.blocks().find(|b| b.name == block_name) {
        Some(b) => b,
        None => {
            tracing::debug!(block = %block_name, "Block definition not found");
            return;
        }
    };

    // Collect entities from the block to avoid borrow conflicts
    let block_entities: Vec<_> = block.entities.clone();

    let cos_r = rotation.cos();
    let sin_r = rotation.sin();

    let transform_point = |p: &Point2D| -> Point2D {
        let x = p.x * scale_x;
        let y = p.y * scale_y;
        Point2D::new(
            x * cos_r - y * sin_r + insert_point.x,
            x * sin_r + y * cos_r + insert_point.y,
        )
    };

    for block_entity in &block_entities {
        let layer = if block_entity.common.layer.is_empty() || block_entity.common.layer == "0" {
            parent_layer.to_string()
        } else {
            block_entity.common.layer.clone()
        };

        match &block_entity.specific {
            DxfEntityType::Line(line) => {
                let mut start = point_from_dxf(&line.p1);
                let mut end = point_from_dxf(&line.p2);
                start = transform_point(&start);
                end = transform_point(&end);
                drawing.entities.push(Entity {
                    id: EntityId(*next_id),
                    geometry: GeometryPrimitive::Line { start, end },
                    layer,
                    color: None,
                    lineweight: None,
                    linetype: None,
                    block_ref: Some(block_name.to_string()),
                });
                *next_id += 1;
            }
            DxfEntityType::Circle(circle) => {
                let mut center = point_from_dxf(&circle.center);
                center = transform_point(&center);
                drawing.entities.push(Entity {
                    id: EntityId(*next_id),
                    geometry: GeometryPrimitive::Circle {
                        center,
                        radius: circle.radius * scale_x,
                    },
                    layer,
                    color: None,
                    lineweight: None,
                    linetype: None,
                    block_ref: Some(block_name.to_string()),
                });
                *next_id += 1;
            }
            DxfEntityType::Arc(arc) => {
                let mut center = point_from_dxf(&arc.center);
                center = transform_point(&center);
                drawing.entities.push(Entity {
                    id: EntityId(*next_id),
                    geometry: GeometryPrimitive::Arc {
                        center,
                        radius: arc.radius * scale_x,
                        start_angle: deg_to_rad(arc.start_angle) + rotation,
                        end_angle: deg_to_rad(arc.end_angle) + rotation,
                    },
                    layer,
                    color: None,
                    lineweight: None,
                    linetype: None,
                    block_ref: Some(block_name.to_string()),
                });
                *next_id += 1;
            }
            DxfEntityType::LwPolyline(poly) => {
                let points: Vec<Point2D> = poly
                    .vertices
                    .iter()
                    .map(|v| {
                        let p = Point2D::new(clean_coordinate(v.x), clean_coordinate(v.y));
                        transform_point(&p)
                    })
                    .collect();
                let bulges: Vec<f64> = poly.vertices.iter().map(|v| v.bulge).collect();
                if points.len() >= 2 {
                    drawing.entities.push(Entity {
                        id: EntityId(*next_id),
                        geometry: GeometryPrimitive::Polyline {
                            points,
                            bulges,
                            closed: poly.is_closed(),
                        },
                        layer,
                        color: None,
                        lineweight: None,
                        linetype: None,
                        block_ref: Some(block_name.to_string()),
                    });
                    *next_id += 1;
                }
            }
            DxfEntityType::Polyline(poly) => {
                let points: Vec<Point2D> = poly
                    .vertices()
                    .map(|v| {
                        let p = point_from_dxf(&v.location);
                        transform_point(&p)
                    })
                    .collect();
                let bulges: Vec<f64> = poly.vertices().map(|v| v.bulge).collect();
                if points.len() >= 2 {
                    drawing.entities.push(Entity {
                        id: EntityId(*next_id),
                        geometry: GeometryPrimitive::Polyline {
                            points,
                            bulges,
                            closed: poly.is_closed(),
                        },
                        layer,
                        color: None,
                        lineweight: None,
                        linetype: None,
                        block_ref: Some(block_name.to_string()),
                    });
                    *next_id += 1;
                }
            }
            DxfEntityType::Spline(spline) => {
                let control_points: Vec<Point2D> = spline
                    .control_points
                    .iter()
                    .map(|p| transform_point(&point_from_dxf(p)))
                    .collect();
                if control_points.len() >= 2 {
                    drawing.entities.push(Entity {
                        id: EntityId(*next_id),
                        geometry: GeometryPrimitive::Spline {
                            control_points,
                            knots: spline.knot_values.clone(),
                            degree: spline.degree_of_curve as u32,
                        },
                        layer,
                        color: None,
                        lineweight: None,
                        linetype: None,
                        block_ref: Some(block_name.to_string()),
                    });
                    *next_id += 1;
                }
            }
            DxfEntityType::Ellipse(ellipse) => {
                let center = point_from_dxf(&ellipse.center);
                let points: Vec<Point2D> = approximate_ellipse(
                    &center,
                    ellipse.major_axis.x,
                    ellipse.major_axis.y,
                    ellipse.minor_axis_ratio,
                    ellipse.start_parameter,
                    ellipse.end_parameter,
                )
                .iter()
                .map(&transform_point)
                .collect();
                if points.len() >= 2 {
                    let is_full = (ellipse.end_parameter - ellipse.start_parameter
                        - std::f64::consts::TAU)
                        .abs()
                        < 1e-6;
                    drawing.entities.push(Entity {
                        id: EntityId(*next_id),
                        geometry: GeometryPrimitive::Polyline {
                            points,
                            bulges: Vec::new(),
                            closed: is_full,
                        },
                        layer,
                        color: None,
                        lineweight: None,
                        linetype: None,
                        block_ref: Some(block_name.to_string()),
                    });
                    *next_id += 1;
                }
            }
            DxfEntityType::Solid(solid) => {
                let p1 = transform_point(&point_from_dxf(&solid.first_corner));
                let p2 = transform_point(&point_from_dxf(&solid.second_corner));
                let p3 = transform_point(&point_from_dxf(&solid.third_corner));
                let p4 = transform_point(&point_from_dxf(&solid.fourth_corner));
                let points = if distance_2d(&p3, &p4) < 1e-6 {
                    vec![p1, p2, p3]
                } else {
                    vec![p1, p2, p4, p3]
                };
                drawing.entities.push(Entity {
                    id: EntityId(*next_id),
                    geometry: GeometryPrimitive::Polyline {
                        points,
                        bulges: Vec::new(),
                        closed: true,
                    },
                    layer,
                    color: None,
                    lineweight: None,
                    linetype: None,
                    block_ref: Some(block_name.to_string()),
                });
                *next_id += 1;
            }
            DxfEntityType::Face3D(face) => {
                let p1 = transform_point(&point_from_dxf(&face.first_corner));
                let p2 = transform_point(&point_from_dxf(&face.second_corner));
                let p3 = transform_point(&point_from_dxf(&face.third_corner));
                let p4 = transform_point(&point_from_dxf(&face.fourth_corner));
                let points = if distance_2d(&p3, &p4) < 1e-6 {
                    vec![p1, p2, p3]
                } else {
                    vec![p1, p2, p3, p4]
                };
                drawing.entities.push(Entity {
                    id: EntityId(*next_id),
                    geometry: GeometryPrimitive::Polyline {
                        points,
                        bulges: Vec::new(),
                        closed: true,
                    },
                    layer,
                    color: None,
                    lineweight: None,
                    linetype: None,
                    block_ref: Some(block_name.to_string()),
                });
                *next_id += 1;
            }
            DxfEntityType::ModelPoint(point) => {
                let p = transform_point(&point_from_dxf(&point.location));
                drawing.entities.push(Entity {
                    id: EntityId(*next_id),
                    geometry: GeometryPrimitive::Point(p),
                    layer,
                    color: None,
                    lineweight: None,
                    linetype: None,
                    block_ref: Some(block_name.to_string()),
                });
                *next_id += 1;
            }
            DxfEntityType::Text(text) => {
                let mut position = point_from_dxf(&text.location);
                position = transform_point(&position);
                drawing.annotations.push(Annotation {
                    id: EntityId(*next_id),
                    text: text.value.clone(),
                    position,
                    height: text.text_height * scale_x.abs(),
                    rotation: deg_to_rad(text.rotation) + rotation,
                    layer,
                });
                *next_id += 1;
            }
            DxfEntityType::MText(mtext) => {
                let mut position = point_from_dxf(&mtext.insertion_point);
                position = transform_point(&position);
                let text_content = clean_mtext(&mtext.text);

                if is_gdt_content(&mtext.text) {
                    let frames = kcc_core::gdt::parser::parse_gdt_mtext(
                        &mtext.text,
                        position,
                        EntityId(*next_id),
                    );
                    for frame in frames {
                        drawing.gdt_frames.push(frame);
                    }
                }

                drawing.annotations.push(Annotation {
                    id: EntityId(*next_id),
                    text: text_content,
                    position,
                    height: mtext.initial_text_height * scale_x.abs(),
                    rotation: deg_to_rad(mtext.rotation_angle) + rotation,
                    layer,
                });
                *next_id += 1;
            }
            DxfEntityType::Insert(nested_insert) => {
                // Recursive block flattening — compose transforms
                let nested_name = nested_insert.name.clone();
                let nested_point = transform_point(&point_from_dxf(&nested_insert.location));
                let nested_scale_x = scale_x * nested_insert.x_scale_factor;
                let nested_scale_y = scale_y * nested_insert.y_scale_factor;
                let nested_rotation = rotation + deg_to_rad(nested_insert.rotation);

                flatten_block(
                    dxf,
                    &nested_name,
                    next_id,
                    &layer,
                    &nested_point,
                    nested_scale_x,
                    nested_scale_y,
                    nested_rotation,
                    depth + 1,
                    drawing,
                );
            }
            DxfEntityType::Tolerance(tol) => {
                let mut position = point_from_dxf(&tol.insertion_point);
                position = transform_point(&position);
                let frames = kcc_core::gdt::parser::parse_tolerance_entity(
                    &tol.display_text,
                    position,
                    EntityId(*next_id),
                );
                for frame in frames {
                    drawing.gdt_frames.push(frame);
                }
                *next_id += 1;
            }
            // Dimension entities inside blocks
            DxfEntityType::RotatedDimension(dim) => {
                let def_pts = vec![
                    transform_point(&point_from_dxf(&dim.dimension_base.definition_point_1)),
                    transform_point(&point_from_dxf(&dim.definition_point_2)),
                    transform_point(&point_from_dxf(&dim.definition_point_3)),
                ];
                parse_dimension_entity(
                    &dim.dimension_base, DimensionType::Linear, def_pts,
                    &layer, drawing, next_id, &HashMap::new(),
                );
            }
            DxfEntityType::RadialDimension(dim) => {
                let def_pts = vec![
                    transform_point(&point_from_dxf(&dim.dimension_base.definition_point_1)),
                    transform_point(&point_from_dxf(&dim.definition_point_2)),
                ];
                parse_dimension_entity(
                    &dim.dimension_base, DimensionType::Radius, def_pts,
                    &layer, drawing, next_id, &HashMap::new(),
                );
            }
            DxfEntityType::DiameterDimension(dim) => {
                let def_pts = vec![
                    transform_point(&point_from_dxf(&dim.dimension_base.definition_point_1)),
                    transform_point(&point_from_dxf(&dim.definition_point_2)),
                ];
                parse_dimension_entity(
                    &dim.dimension_base, DimensionType::Diameter, def_pts,
                    &layer, drawing, next_id, &HashMap::new(),
                );
            }
            DxfEntityType::AngularThreePointDimension(dim) => {
                let def_pts = vec![
                    transform_point(&point_from_dxf(&dim.dimension_base.definition_point_1)),
                    transform_point(&point_from_dxf(&dim.definition_point_2)),
                    transform_point(&point_from_dxf(&dim.definition_point_3)),
                ];
                parse_dimension_entity(
                    &dim.dimension_base, DimensionType::Angular, def_pts,
                    &layer, drawing, next_id, &HashMap::new(),
                );
            }
            DxfEntityType::OrdinateDimension(dim) => {
                let def_pts = vec![
                    transform_point(&point_from_dxf(&dim.dimension_base.definition_point_1)),
                    transform_point(&point_from_dxf(&dim.definition_point_2)),
                ];
                parse_dimension_entity(
                    &dim.dimension_base, DimensionType::Ordinate, def_pts,
                    &layer, drawing, next_id, &HashMap::new(),
                );
            }
            _ => {
                // Intentionally skip unsupported entity types in blocks
            }
        }
    }
}

/// Approximate an ellipse as a polyline by sampling points along the parametric curve.
fn approximate_ellipse(
    center: &Point2D,
    major_axis_x: f64,
    major_axis_y: f64,
    minor_axis_ratio: f64,
    start_param: f64,
    end_param: f64,
) -> Vec<Point2D> {
    let segments = 64;
    let major_len = (major_axis_x * major_axis_x + major_axis_y * major_axis_y).sqrt();
    if major_len < 1e-10 {
        return Vec::new();
    }
    let angle = major_axis_y.atan2(major_axis_x);
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let minor_len = major_len * minor_axis_ratio;

    let mut end = end_param;
    if end <= start_param {
        end += std::f64::consts::TAU;
    }

    (0..=segments)
        .map(|i| {
            let t = start_param + (end - start_param) * (i as f64) / (segments as f64);
            let ex = major_len * t.cos();
            let ey = minor_len * t.sin();
            Point2D::new(
                clean_coordinate(center.x + ex * cos_a - ey * sin_a),
                clean_coordinate(center.y + ex * sin_a + ey * cos_a),
            )
        })
        .collect()
}

fn distance_2d(a: &Point2D, b: &Point2D) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

