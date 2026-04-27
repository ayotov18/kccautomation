//! Integration test: parse the TASOS DWG (converted to DXF) and verify the
//! parser detects three independent modules.

use kcc_core::geometry::model::GeometryPrimitive;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

#[test]
fn tasos_dxf_yields_three_structures() {
    let path = fixture_path("tasos_three_modules.dxf");
    if !path.exists() {
        eprintln!("Skipping: fixture not present at {}", path.display());
        return;
    }
    let drawing = kcc_dxf::parser::parse_dxf_file(&path).expect("parse should succeed");

    // Top 30 X-centroid gaps overall — quick sanity for module-boundary picking.
    let mut all_x: Vec<f64> = drawing
        .entities
        .iter()
        .filter_map(|e| match &e.geometry {
            GeometryPrimitive::Line { start, end } => Some((start.x + end.x) * 0.5),
            GeometryPrimitive::Polyline { points, .. } if !points.is_empty() => {
                let s: f64 = points.iter().map(|p| p.x).sum();
                Some(s / points.len() as f64)
            }
            GeometryPrimitive::Circle { center, .. } | GeometryPrimitive::Arc { center, .. } => {
                Some(center.x)
            }
            GeometryPrimitive::Point(p) => Some(p.x),
            _ => None,
        })
        .collect();
    all_x.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut gaps: Vec<f64> = (1..all_x.len()).map(|i| all_x[i] - all_x[i - 1]).collect();
    gaps.sort_by(|a, b| b.partial_cmp(a).unwrap());
    eprintln!("Top 20 X-centroid gaps (descending): {:?}", &gaps[..20.min(gaps.len())]);

    // Diagnostic: dump X centroid histogram for the second cluster (the one that
    // wrongly contains Дани67 + Торос35 merged).
    let mut centroids_x: Vec<f64> = drawing
        .entities
        .iter()
        .filter_map(|e| match &e.geometry {
            GeometryPrimitive::Line { start, end } => Some((start.x + end.x) * 0.5),
            GeometryPrimitive::Polyline { points, .. } if !points.is_empty() => {
                let s: f64 = points.iter().map(|p| p.x).sum();
                Some(s / points.len() as f64)
            }
            GeometryPrimitive::Circle { center, .. } | GeometryPrimitive::Arc { center, .. } => {
                Some(center.x)
            }
            GeometryPrimitive::Point(p) => Some(p.x),
            _ => None,
        })
        .filter(|x| *x > 100_000.0 && *x < 125_000.0)
        .collect();
    centroids_x.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if !centroids_x.is_empty() {
        let n = centroids_x.len();
        eprintln!("X centroids in second cluster: n={n}");
        eprintln!("  min={:.0} max={:.0}", centroids_x[0], centroids_x[n - 1]);
        // 50-bin histogram in [105k..125k]
        let lo = 105_000f64;
        let hi = 125_000f64;
        let bins = 50;
        let bw = (hi - lo) / bins as f64;
        let mut h = vec![0usize; bins];
        for c in &centroids_x {
            let i = (((*c - lo) / bw) as usize).min(bins - 1);
            h[i] += 1;
        }
        for (i, c) in h.iter().enumerate() {
            eprintln!(
                "  bin {:2} [{:7.0} {:7.0}] {}",
                i,
                lo + i as f64 * bw,
                lo + (i + 1) as f64 * bw,
                "█".repeat((*c).min(60))
            );
        }
    }

    eprintln!("Detected {} structures:", drawing.structures.len());
    for s in &drawing.structures {
        eprintln!(
            "  #{} {:30} entities={}  bbox=[{:.0}..{:.0}] x [{:.0}..{:.0}]  size={:.0}x{:.0}",
            s.id,
            s.label,
            s.entity_ids.len(),
            s.bbox_min.x,
            s.bbox_max.x,
            s.bbox_min.y,
            s.bbox_max.y,
            s.bbox_max.x - s.bbox_min.x,
            s.bbox_max.y - s.bbox_min.y,
        );
    }
    let n = drawing.structures.len();
    assert_eq!(n, 3, "expected 3 detected modules in TASOS multi-module sheet, got {n}");
}
