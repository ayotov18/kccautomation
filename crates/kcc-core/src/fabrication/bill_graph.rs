use serde::{Deserialize, Serialize};

use crate::feature::types::{Feature, FeatureType};
use crate::geometry::model::Drawing;
use super::profile_db;

/// Steel density in kg/m³.
const STEEL_DENSITY: f64 = 7850.0;

/// Project-level fabrication parameters (user-configurable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricationParams {
    /// Default plate thickness in mm (used when thickness is invisible in 2D).
    pub default_plate_thickness_mm: f64,
    /// Whether to include surface treatment estimate.
    pub include_surface_treatment: bool,
    /// Whether to include weld estimate (based on joint count).
    pub include_weld_estimate: bool,
}

impl Default for FabricationParams {
    fn default() -> Self {
        Self {
            default_plate_thickness_mm: 10.0,
            include_surface_treatment: true,
            include_weld_estimate: true,
        }
    }
}

/// A single commercially meaningful quantity item with full derivation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricationItem {
    /// SEK14 nomenclature code.
    pub sek_code: String,
    /// Human-readable description.
    pub description: String,
    /// Unit of measurement (TON, KG, M, M2, PCS).
    pub unit: String,
    /// Computed quantity.
    pub quantity: f64,
    /// How the quantity was derived (audit trail).
    pub derivation: String,
    /// Confidence level (0.0 = pure assumption, 1.0 = measured from geometry).
    pub confidence: f64,
    /// Source feature IDs that contributed to this item.
    pub source_features: Vec<u64>,
    /// Assumptions made in derivation.
    pub assumptions: Vec<String>,
}

/// The fabrication bill graph — commercially meaningful items derived from features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricationBillGraph {
    pub items: Vec<FabricationItem>,
    pub params: FabricationParams,
    /// Total steel weight in kg (sum of all weighted items).
    pub total_weight_kg: f64,
}

/// Transform extracted features into a fabrication bill graph.
pub fn build_fabrication_bill(
    features: &[Feature],
    _drawing: &Drawing,
    params: &FabricationParams,
) -> FabricationBillGraph {
    let mut items = Vec::new();
    let mut total_weight_kg = 0.0;

    // === Steel Members ===
    for feature in features {
        if let FeatureType::SteelMember { length, depth, profile_hint } = &feature.feature_type {
            // Filter out title block false positives: reject any "profile" longer than 20 chars
            // or containing non-profile characters like |, @, :, /
            if let Some(hint) = profile_hint {
                let h = hint.trim();
                if h.len() > 20
                    || h.contains('|')
                    || h.contains('@')
                    || h.contains(':')
                    || h.contains('/')
                    || h.contains('\\')
                    || h.contains(',')
                {
                    tracing::debug!(hint = %h, "Skipping non-profile steel member (title block text)");
                    continue;
                }
            }
            let (item, weight) = process_steel_member(feature, *length, *depth, profile_hint.as_deref());
            total_weight_kg += weight;
            items.push(item);
        }
    }

    // === Gusset Plates ===
    let plates: Vec<&Feature> = features.iter()
        .filter(|f| matches!(f.feature_type, FeatureType::GussetPlate { .. }))
        .collect();

    if !plates.is_empty() {
        let (plate_items, plate_weight) = process_gusset_plates(&plates, params);
        total_weight_kg += plate_weight;
        items.extend(plate_items);
    }

    // === Bolt Assemblies ===
    let mut total_bolts = 0usize;
    let mut bolt_feature_ids = Vec::new();

    for feature in features {
        match &feature.feature_type {
            FeatureType::Hole { .. } => {
                total_bolts += 1;
                bolt_feature_ids.push(feature.id.0);
            }
            FeatureType::BoltCircle { hole_count, .. } => {
                total_bolts += hole_count;
                bolt_feature_ids.push(feature.id.0);
            }
            FeatureType::BoltGroup { bolt_count, .. } => {
                total_bolts += bolt_count;
                bolt_feature_ids.push(feature.id.0);
            }
            _ => {}
        }
    }

    if total_bolts > 0 {
        items.push(FabricationItem {
            sek_code: "14.015".into(),
            description: format!("Bolt assemblies (M12-M24 assumed), {} pcs", total_bolts),
            unit: "PCS".into(),
            quantity: total_bolts as f64,
            derivation: format!("{} holes/bolt groups detected in drawing", total_bolts),
            confidence: 0.7,
            source_features: bolt_feature_ids,
            assumptions: vec!["Bolt size assumed M12-M24 range (not measurable from 2D)".into()],
        });
    }

    // === Surface Treatment (if enabled) ===
    if params.include_surface_treatment && total_weight_kg > 0.0 {
        // Estimate surface area from weight using empirical ratio (~25 m²/ton for typical steel)
        let estimated_area_m2 = (total_weight_kg / 1000.0) * 25.0;
        items.push(FabricationItem {
            sek_code: "14.020".into(),
            description: format!("Surface preparation + primer ({:.1}m²)", estimated_area_m2),
            unit: "M2".into(),
            quantity: estimated_area_m2,
            derivation: format!("Estimated from total steel weight ({:.1}kg) × 25 m²/ton empirical ratio", total_weight_kg),
            confidence: 0.4,
            source_features: vec![],
            assumptions: vec![
                "Surface area estimated at 25 m²/ton (empirical average for mixed profiles)".into(),
                "Assumes shotblast SA 2.5 + 1 coat primer".into(),
            ],
        });
    }

    // === Weld Estimate (if enabled) ===
    if params.include_weld_estimate {
        // Count connection points: each gusset plate connects to members via welds
        let weld_connections = plates.len();
        if weld_connections > 0 {
            // Estimate ~1.5m of fillet weld per gusset plate connection (conservative)
            let weld_length_m = weld_connections as f64 * 1.5;
            items.push(FabricationItem {
                sek_code: "14.501".into(),
                description: format!("Fillet welds (a=6mm assumed), {} connections, {:.1}m total", weld_connections, weld_length_m),
                unit: "M".into(),
                quantity: weld_length_m,
                derivation: format!("{} gusset plate connections × 1.5m avg weld per connection", weld_connections),
                confidence: 0.3,
                source_features: plates.iter().map(|p| p.id.0).collect(),
                assumptions: vec![
                    "Weld throat thickness assumed a=6mm".into(),
                    "Average 1.5m weld per plate connection (empirical)".into(),
                    "All welds assumed fillet type".into(),
                ],
            });
        }
    }

    // === Erection/Assembly ===
    let member_count = features.iter().filter(|f| matches!(f.feature_type, FeatureType::SteelMember { .. })).count();
    if member_count > 0 || !plates.is_empty() {
        let total_tons = total_weight_kg / 1000.0;
        items.push(FabricationItem {
            sek_code: "14.001".into(),
            description: format!("Steel structure erection, {:.2} TON", total_tons),
            unit: "TON".into(),
            quantity: total_tons,
            derivation: format!("Total fabricated weight: {:.1}kg = {:.3} TON", total_weight_kg, total_tons),
            confidence: 0.6,
            source_features: vec![],
            assumptions: vec!["Erection weight equals fabrication weight".into()],
        });
    }

    FabricationBillGraph {
        items,
        params: params.clone(),
        total_weight_kg,
    }
}

fn process_steel_member(
    feature: &Feature,
    length: f64,
    depth: f64,
    profile_hint: Option<&str>,
) -> (FabricationItem, f64) {
    // Try to match a profile from the database
    let matched = if let Some(hint) = profile_hint {
        profile_db::find_profile_by_name(hint)
            .or_else(|| profile_db::match_profile_by_depth(depth, None))
    } else if depth > 0.0 {
        profile_db::match_profile_by_depth(depth, None)
    } else {
        None
    };

    if let Some(profile) = matched {
        let length_m = if length > 0.0 { length / 1000.0 } else { 1.0 };
        let weight_kg = profile.kg_per_m * length_m;

        (FabricationItem {
            sek_code: sek_code_for_weight(weight_kg),
            description: format!("{}, L={:.0}mm, {:.1}kg", profile.designation, length, weight_kg),
            unit: if weight_kg >= 50.0 { "TON".into() } else { "KG".into() },
            quantity: if weight_kg >= 50.0 { weight_kg / 1000.0 } else { weight_kg },
            derivation: format!(
                "{} × {:.2}m = {:.1}kg ({:.1} kg/m × {:.2}m)",
                profile.designation, length_m, weight_kg, profile.kg_per_m, length_m
            ),
            confidence: 0.8,
            source_features: vec![feature.id.0],
            assumptions: vec![format!("Profile matched: {} (depth={:.0}mm)", profile.designation, profile.depth_mm)],
        }, weight_kg)
    } else {
        // Unknown profile — report with assumptions
        let desc = profile_hint.unwrap_or("unknown profile");
        (FabricationItem {
            sek_code: "14.001".into(),
            description: format!("Steel member: {}, L={:.0}mm", desc, length),
            unit: "PCS".into(),
            quantity: 1.0,
            derivation: format!("Profile '{}' not matched in database. Length={:.0}mm, depth={:.0}mm", desc, length, depth),
            confidence: 0.3,
            source_features: vec![feature.id.0],
            assumptions: vec![
                format!("Profile '{}' not in standard database", desc),
                "Weight could not be computed — manual entry required".into(),
            ],
        }, 0.0)
    }
}

fn process_gusset_plates(
    plates: &[&Feature],
    params: &FabricationParams,
) -> (Vec<FabricationItem>, f64) {
    let mut items = Vec::new();
    let mut total_weight = 0.0;

    // Group plates by similar area (to aggregate into single line items)
    let thickness = params.default_plate_thickness_mm;

    for (i, plate) in plates.iter().enumerate() {
        if let FeatureType::GussetPlate { area, vertex_count } = &plate.feature_type {
            let area_m2 = area / 1_000_000.0;
            let volume_m3 = area_m2 * (thickness / 1000.0);
            let weight_kg = volume_m3 * STEEL_DENSITY;
            total_weight += weight_kg;

            items.push(FabricationItem {
                sek_code: sek_code_for_weight(weight_kg),
                description: format!(
                    "Plate {}: {}-sided, {:.0}mm² ({:.2}m²), t={:.0}mm, {:.1}kg",
                    i + 1, vertex_count, area, area_m2, thickness, weight_kg
                ),
                unit: "KG".into(),
                quantity: weight_kg,
                derivation: format!(
                    "{:.0}mm² × {:.0}mm thickness × 7850 kg/m³ = {:.1}kg",
                    area, thickness, weight_kg
                ),
                confidence: 0.6,
                source_features: vec![plate.id.0],
                assumptions: vec![
                    format!("Plate thickness assumed {:.0}mm (configurable)", thickness),
                    "Material assumed S235/S275 structural steel (7850 kg/m³)".into(),
                ],
            });
        }
    }

    (items, total_weight)
}

/// Assign SEK code based on individual item weight.
fn sek_code_for_weight(weight_kg: f64) -> String {
    if weight_kg >= 1000.0 {
        "14.001".into() // Heavy structural elements (TON)
    } else if weight_kg >= 50.0 {
        "14.011".into() // Medium elements
    } else {
        "14.021".into() // Light elements / minor steel
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::types::FeatureId;
    use crate::geometry::model::{EntityId, Point2D};

    #[test]
    fn test_steel_member_with_known_profile() {
        let features = vec![Feature {
            id: FeatureId(1),
            feature_type: FeatureType::SteelMember {
                length: 6000.0,
                depth: 200.0,
                profile_hint: Some("IPE 200".into()),
            },
            geometry_refs: vec![EntityId(1)],
            centroid: Point2D::new(0.0, 0.0),
            dimensions: vec![],
            gdt_frames: vec![],
            datum_refs: vec![],
            layer_hint: None,
        }];

        let drawing = Drawing::new("test.dxf".into());
        let params = FabricationParams::default();
        let bill = build_fabrication_bill(&features, &drawing, &params);

        // IPE 200 at 6m = 22.4 kg/m × 6m = 134.4 kg → reported as TON (0.1344)
        assert!(!bill.items.is_empty());
        let member_item = &bill.items[0];
        assert!(member_item.description.contains("IPE 200"));
        // 134.4kg >= 50 so unit is TON, quantity = 0.1344
        assert!((bill.total_weight_kg - 134.4).abs() < 1.0);
        assert!(member_item.confidence > 0.5);
    }

    #[test]
    fn test_gusset_plate_weight() {
        let features = vec![Feature {
            id: FeatureId(1),
            feature_type: FeatureType::GussetPlate {
                area: 100_000.0, // 100,000 mm² = 0.1 m²
                vertex_count: 4,
            },
            geometry_refs: vec![EntityId(1)],
            centroid: Point2D::new(0.0, 0.0),
            dimensions: vec![],
            gdt_frames: vec![],
            datum_refs: vec![],
            layer_hint: None,
        }];

        let drawing = Drawing::new("test.dxf".into());
        let params = FabricationParams { default_plate_thickness_mm: 10.0, ..Default::default() };
        let bill = build_fabrication_bill(&features, &drawing, &params);

        // 0.1 m² × 0.01m × 7850 = 7.85 kg
        let plate_item = bill.items.iter().find(|i| i.description.contains("Plate")).unwrap();
        assert!((plate_item.quantity - 7.85).abs() < 0.1);
    }
}
