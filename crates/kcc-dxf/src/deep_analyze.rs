//! Deep analysis module — extracts EVERYTHING the dxf crate can provide from a DXF file.
//! Produces a comprehensive JSON with data catalogs.

use dxf::Drawing as DxfDrawing;
use dxf::entities::EntityType as DxfEntityType;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Cursor;

/// Perform a deep analysis of a DXF file, extracting all available data.
pub fn deep_analyze(dxf_bytes: &[u8], filename: &str) -> Result<Value, String> {
    let dxf = DxfDrawing::load(&mut Cursor::new(dxf_bytes))
        .map_err(|e| format!("Failed to parse DXF: {e}"))?;

    let file_metadata = extract_file_metadata(&dxf, filename);
    let layers = extract_layers(&dxf);
    let dimension_styles = extract_dim_styles(&dxf);
    let linetypes = extract_linetypes(&dxf);
    let blocks = extract_blocks(&dxf);
    let (entities, entity_type_counts) = extract_all_entities(&dxf);
    let dimensions = extract_dimensions(&dxf);
    let annotations = extract_annotations(&dxf);
    let statistics = build_statistics(&dxf, &entity_type_counts);

    Ok(json!({
        "file_metadata": file_metadata,
        "layers": layers,
        "dimension_styles": dimension_styles,
        "linetypes": linetypes,
        "blocks": blocks,
        "entities": entities,
        "dimensions": dimensions,
        "annotations": annotations,
        "statistics": statistics,
    }))
}

fn extract_file_metadata(dxf: &DxfDrawing, filename: &str) -> Value {
    let h = &dxf.header;
    json!({
        "filename": filename,
        "version": format!("{:?}", h.version),
        "insert_units": h.default_drawing_units as i32,
        "drawing_extents_min": [h.minimum_drawing_extents.x, h.minimum_drawing_extents.y, h.minimum_drawing_extents.z],
        "drawing_extents_max": [h.maximum_drawing_extents.x, h.maximum_drawing_extents.y, h.maximum_drawing_extents.z],
        "drawing_limits_min": [h.minimum_drawing_limits.x, h.minimum_drawing_limits.y],
        "drawing_limits_max": [h.maximum_drawing_limits.x, h.maximum_drawing_limits.y],
        "insertion_base": [h.insertion_base.x, h.insertion_base.y, h.insertion_base.z],
        "dimension_style_name": &h.dimension_style_name,
        "current_layer": &h.current_layer,
        "current_entity_linetype": &h.current_entity_line_type,
        "current_entity_color": h.current_entity_color.index(),
        "angle_zero_direction": h.angle_zero_direction,
    })
}

fn extract_layers(dxf: &DxfDrawing) -> Vec<Value> {
    dxf.layers().map(|layer| {
        json!({
            "name": &layer.name,
            "color": layer.color.index(),
            "linetype": &layer.line_type_name,
            "is_layer_on": layer.is_layer_on,
            "is_layer_plotted": layer.is_layer_plotted,
        })
    }).collect()
}

fn extract_dim_styles(dxf: &DxfDrawing) -> Vec<Value> {
    dxf.dim_styles().map(|ds| {
        json!({
            "name": &ds.name,
            "dimension_text_height": ds.dimensioning_text_height,
            "arrow_size": ds.dimensioning_arrow_size,
            "dimension_line_gap": ds.dimension_line_gap,
            "generate_tolerances": ds.generate_dimension_tolerances,
            "generate_limits": ds.generate_dimension_limits,
            "plus_tolerance": ds.dimension_plus_tolerance,
            "minus_tolerance": ds.dimension_minus_tolerance,
            "text_movement_rule": ds.dimension_text_movement_rule as i32,
            "text_inside_horizontal": ds.dimension_text_inside_horizontal,
            "text_outside_horizontal": ds.dimension_text_outside_horizontal,
            "linear_scale_factor": ds.dimension_linear_measurement_scale_factor,
        })
    }).collect()
}

fn extract_linetypes(dxf: &DxfDrawing) -> Vec<Value> {
    dxf.line_types().map(|lt| {
        json!({
            "name": &lt.name,
            "description": &lt.description,
            "total_pattern_length": lt.total_pattern_length,
            "element_count": lt.element_count,
            "dash_lengths": &lt.dash_dot_space_lengths,
        })
    }).collect()
}

fn extract_blocks(dxf: &DxfDrawing) -> Vec<Value> {
    dxf.blocks().map(|block| {
        let entity_count = block.entities.len();
        let entity_types: HashMap<String, usize> = {
            let mut counts = HashMap::new();
            for e in &block.entities {
                let name = entity_type_name(&e.specific);
                *counts.entry(name).or_default() += 1;
            }
            counts
        };

        json!({
            "name": &block.name,
            "base_point": [block.base_point.x, block.base_point.y, block.base_point.z],
            "description": &block.description,
            "is_xref": !block.xref_path_name.is_empty(),
            "xref_path": &block.xref_path_name,
            "entity_count": entity_count,
            "entity_types": entity_types,
        })
    }).collect()
}

fn extract_all_entities(dxf: &DxfDrawing) -> (Vec<Value>, HashMap<String, usize>) {
    let mut entities = Vec::new();
    let mut type_counts: HashMap<String, usize> = HashMap::new();

    for entity in dxf.entities() {
        let common = &entity.common;
        let type_name = entity_type_name(&entity.specific);
        *type_counts.entry(type_name.clone()).or_default() += 1;

        let mut ent = json!({
            "type": type_name,
            "layer": &common.layer,
            "color": common.color.index(),
            "linetype": &common.line_type_name,
            "is_visible": common.is_visible,
        });

        let specific = extract_entity_specific(&entity.specific);
        if let Value::Object(map) = specific {
            if let Value::Object(ref mut ent_map) = ent {
                for (k, v) in map {
                    ent_map.insert(k, v);
                }
            }
        }

        entities.push(ent);
    }

    (entities, type_counts)
}

fn extract_entity_specific(entity: &DxfEntityType) -> Value {
    match entity {
        DxfEntityType::Line(e) => json!({
            "start": [e.p1.x, e.p1.y, e.p1.z],
            "end": [e.p2.x, e.p2.y, e.p2.z],
        }),
        DxfEntityType::Circle(e) => json!({
            "center": [e.center.x, e.center.y, e.center.z],
            "radius": e.radius,
        }),
        DxfEntityType::Arc(e) => json!({
            "center": [e.center.x, e.center.y, e.center.z],
            "radius": e.radius,
            "start_angle": e.start_angle,
            "end_angle": e.end_angle,
        }),
        DxfEntityType::LwPolyline(e) => {
            let vertices: Vec<Value> = e.vertices.iter().map(|v| json!({
                "x": v.x, "y": v.y, "bulge": v.bulge,
                "starting_width": v.starting_width,
                "ending_width": v.ending_width,
            })).collect();
            json!({ "vertices": vertices, "is_closed": e.is_closed() })
        }
        DxfEntityType::Text(e) => json!({
            "value": &e.value,
            "location": [e.location.x, e.location.y, e.location.z],
            "text_height": e.text_height,
            "rotation": e.rotation,
            "horizontal_justification": e.horizontal_text_justification as i32,
            "vertical_justification": e.vertical_text_justification as i32,
            "style_name": &e.text_style_name,
            "width_factor": e.relative_x_scale_factor,
        }),
        DxfEntityType::MText(e) => json!({
            "text": &e.text,
            "insertion_point": [e.insertion_point.x, e.insertion_point.y, e.insertion_point.z],
            "text_height": e.initial_text_height,
            "reference_rectangle_width": e.reference_rectangle_width,
            "rotation": e.rotation_angle,
            "attachment_point": e.attachment_point as i32,
            "drawing_direction": e.drawing_direction as i32,
            "style_name": &e.text_style_name,
            "line_spacing_factor": e.line_spacing_factor,
        }),
        DxfEntityType::Insert(e) => json!({
            "block_name": &e.name,
            "location": [e.location.x, e.location.y, e.location.z],
            "x_scale": e.x_scale_factor,
            "y_scale": e.y_scale_factor,
            "z_scale": e.z_scale_factor,
            "rotation": e.rotation,
            "column_count": e.column_count,
            "row_count": e.row_count,
            "column_spacing": e.column_spacing,
            "row_spacing": e.row_spacing,
        }),
        DxfEntityType::Spline(e) => {
            let ctrl: Vec<[f64; 3]> = e.control_points.iter().map(|p| [p.x, p.y, p.z]).collect();
            json!({
                "degree": e.degree_of_curve,
                "control_points": ctrl,
                "knot_values": &e.knot_values,
            })
        }
        DxfEntityType::Ellipse(e) => json!({
            "center": [e.center.x, e.center.y, e.center.z],
            "major_axis": [e.major_axis.x, e.major_axis.y, e.major_axis.z],
            "minor_axis_ratio": e.minor_axis_ratio,
            "start_parameter": e.start_parameter,
            "end_parameter": e.end_parameter,
        }),
        DxfEntityType::Solid(e) => json!({
            "corners": [
                [e.first_corner.x, e.first_corner.y],
                [e.second_corner.x, e.second_corner.y],
                [e.third_corner.x, e.third_corner.y],
                [e.fourth_corner.x, e.fourth_corner.y],
            ],
        }),
        DxfEntityType::RotatedDimension(e) => json!({
            "dim_type": "rotated",
            "text": &e.dimension_base.text,
            "actual_measurement": e.dimension_base.actual_measurement,
            "style_name": &e.dimension_base.dimension_style_name,
            "definition_point_1": [e.dimension_base.definition_point_1.x, e.dimension_base.definition_point_1.y],
            "definition_point_2": [e.definition_point_2.x, e.definition_point_2.y],
            "definition_point_3": [e.definition_point_3.x, e.definition_point_3.y],
            "text_mid_point": [e.dimension_base.text_mid_point.x, e.dimension_base.text_mid_point.y],
            "rotation_angle": e.rotation_angle,
        }),
        DxfEntityType::RadialDimension(e) => json!({
            "dim_type": "radial",
            "text": &e.dimension_base.text,
            "actual_measurement": e.dimension_base.actual_measurement,
            "style_name": &e.dimension_base.dimension_style_name,
            "definition_point_2": [e.definition_point_2.x, e.definition_point_2.y],
            "leader_length": e.leader_length,
        }),
        DxfEntityType::DiameterDimension(e) => json!({
            "dim_type": "diameter",
            "text": &e.dimension_base.text,
            "actual_measurement": e.dimension_base.actual_measurement,
            "style_name": &e.dimension_base.dimension_style_name,
            "definition_point_2": [e.definition_point_2.x, e.definition_point_2.y],
            "leader_length": e.leader_length,
        }),
        DxfEntityType::AngularThreePointDimension(e) => json!({
            "dim_type": "angular_3point",
            "text": &e.dimension_base.text,
            "actual_measurement": e.dimension_base.actual_measurement,
            "definition_point_2": [e.definition_point_2.x, e.definition_point_2.y],
            "definition_point_3": [e.definition_point_3.x, e.definition_point_3.y],
        }),
        DxfEntityType::OrdinateDimension(e) => json!({
            "dim_type": "ordinate",
            "text": &e.dimension_base.text,
            "actual_measurement": e.dimension_base.actual_measurement,
            "definition_point_2": [e.definition_point_2.x, e.definition_point_2.y],
        }),
        DxfEntityType::Tolerance(e) => json!({
            "display_text": &e.display_text,
            "insertion_point": [e.insertion_point.x, e.insertion_point.y, e.insertion_point.z],
            "direction_vector": [e.direction_vector.x, e.direction_vector.y, e.direction_vector.z],
            "dimension_style_name": &e.dimension_style_name,
        }),
        DxfEntityType::ModelPoint(e) => json!({
            "location": [e.location.x, e.location.y, e.location.z],
            "angle": e.angle,
        }),
        DxfEntityType::Polyline(e) => {
            let verts: Vec<Value> = e.vertices().map(|v| json!({
                "location": [v.location.x, v.location.y, v.location.z],
                "bulge": v.bulge,
                "starting_width": v.starting_width,
                "ending_width": v.ending_width,
            })).collect();
            json!({ "vertices": verts, "is_closed": e.is_closed() })
        }
        DxfEntityType::Face3D(e) => json!({
            "corners": [
                [e.first_corner.x, e.first_corner.y, e.first_corner.z],
                [e.second_corner.x, e.second_corner.y, e.second_corner.z],
                [e.third_corner.x, e.third_corner.y, e.third_corner.z],
                [e.fourth_corner.x, e.fourth_corner.y, e.fourth_corner.z],
            ],
        }),
        _ => json!({ "_note": "Entity type not fully decomposed" }),
    }
}

fn extract_dimensions(dxf: &DxfDrawing) -> Vec<Value> {
    let mut dims = Vec::new();
    for entity in dxf.entities() {
        let dim_val = match &entity.specific {
            DxfEntityType::RotatedDimension(e) => Some(json!({
                "type": "linear/rotated", "text": &e.dimension_base.text,
                "actual_measurement": e.dimension_base.actual_measurement,
                "style": &e.dimension_base.dimension_style_name, "layer": &entity.common.layer,
            })),
            DxfEntityType::RadialDimension(e) => Some(json!({
                "type": "radial", "text": &e.dimension_base.text,
                "actual_measurement": e.dimension_base.actual_measurement,
                "style": &e.dimension_base.dimension_style_name, "layer": &entity.common.layer,
            })),
            DxfEntityType::DiameterDimension(e) => Some(json!({
                "type": "diameter", "text": &e.dimension_base.text,
                "actual_measurement": e.dimension_base.actual_measurement,
                "style": &e.dimension_base.dimension_style_name, "layer": &entity.common.layer,
            })),
            DxfEntityType::AngularThreePointDimension(e) => Some(json!({
                "type": "angular", "text": &e.dimension_base.text,
                "actual_measurement": e.dimension_base.actual_measurement, "layer": &entity.common.layer,
            })),
            DxfEntityType::OrdinateDimension(e) => Some(json!({
                "type": "ordinate", "text": &e.dimension_base.text,
                "actual_measurement": e.dimension_base.actual_measurement, "layer": &entity.common.layer,
            })),
            _ => None,
        };
        if let Some(v) = dim_val { dims.push(v); }
    }
    dims
}

fn extract_annotations(dxf: &DxfDrawing) -> Vec<Value> {
    let mut annotations = Vec::new();
    for entity in dxf.entities() {
        match &entity.specific {
            DxfEntityType::Text(e) => {
                annotations.push(json!({
                    "type": "TEXT", "value": &e.value,
                    "location": [e.location.x, e.location.y], "height": e.text_height,
                    "rotation": e.rotation, "style": &e.text_style_name, "layer": &entity.common.layer,
                }));
            }
            DxfEntityType::MText(e) => {
                annotations.push(json!({
                    "type": "MTEXT", "text": &e.text,
                    "insertion_point": [e.insertion_point.x, e.insertion_point.y],
                    "height": e.initial_text_height, "rotation": e.rotation_angle,
                    "width": e.reference_rectangle_width, "attachment": e.attachment_point as i32,
                    "style": &e.text_style_name, "layer": &entity.common.layer,
                }));
            }
            _ => {}
        }
    }
    annotations
}

fn build_statistics(dxf: &DxfDrawing, entity_type_counts: &HashMap<String, usize>) -> Value {
    let mut layer_counts: HashMap<String, usize> = HashMap::new();
    for e in dxf.entities() {
        *layer_counts.entry(e.common.layer.clone()).or_default() += 1;
    }
    json!({
        "total_entities": entity_type_counts.values().sum::<usize>(),
        "total_layers": dxf.layers().count(),
        "total_blocks": dxf.blocks().count(),
        "total_dimension_styles": dxf.dim_styles().count(),
        "total_linetypes": dxf.line_types().count(),
        "entity_type_counts": entity_type_counts,
        "entities_per_layer": layer_counts,
    })
}

fn entity_type_name(entity: &DxfEntityType) -> String {
    match entity {
        DxfEntityType::Line(_) => "LINE",
        DxfEntityType::Circle(_) => "CIRCLE",
        DxfEntityType::Arc(_) => "ARC",
        DxfEntityType::LwPolyline(_) => "LWPOLYLINE",
        DxfEntityType::Polyline(_) => "POLYLINE",
        DxfEntityType::Spline(_) => "SPLINE",
        DxfEntityType::Ellipse(_) => "ELLIPSE",
        DxfEntityType::Text(_) => "TEXT",
        DxfEntityType::MText(_) => "MTEXT",
        DxfEntityType::Insert(_) => "INSERT",
        DxfEntityType::Solid(_) => "SOLID",
        DxfEntityType::Face3D(_) => "3DFACE",
        DxfEntityType::ModelPoint(_) => "POINT",
        DxfEntityType::RotatedDimension(_) => "DIMENSION_ROTATED",
        DxfEntityType::RadialDimension(_) => "DIMENSION_RADIAL",
        DxfEntityType::DiameterDimension(_) => "DIMENSION_DIAMETER",
        DxfEntityType::AngularThreePointDimension(_) => "DIMENSION_ANGULAR",
        DxfEntityType::OrdinateDimension(_) => "DIMENSION_ORDINATE",
        DxfEntityType::Tolerance(_) => "TOLERANCE",
        _ => "OTHER",
    }.to_string()
}
