use serde::{Deserialize, Serialize};

/// A standard European steel profile with dimensional and weight data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteelProfile {
    /// Profile designation (e.g., "IPE 200", "HEA 160")
    pub designation: &'static str,
    /// Profile family (e.g., "IPE", "HEA", "UPN")
    pub family: &'static str,
    /// Nominal depth in mm (h)
    pub depth_mm: f64,
    /// Flange width in mm (b)
    pub flange_width_mm: f64,
    /// Web thickness in mm (tw)
    pub web_thickness_mm: f64,
    /// Flange thickness in mm (tf)
    pub flange_thickness_mm: f64,
    /// Weight per meter in kg/m
    pub kg_per_m: f64,
    /// Cross-sectional area in cm²
    pub area_cm2: f64,
}

/// Look up the nearest steel profile by measured depth.
pub fn match_profile_by_depth(depth_mm: f64, family_hint: Option<&str>) -> Option<&'static SteelProfile> {
    let db = profile_database();
    let mut best: Option<&SteelProfile> = None;
    let mut best_diff = f64::MAX;

    for profile in db {
        // If family hint given, filter by it
        if let Some(hint) = family_hint {
            let hint_upper = hint.to_uppercase();
            if !profile.family.starts_with(&hint_upper) {
                continue;
            }
        }

        let diff = (profile.depth_mm - depth_mm).abs();
        if diff < best_diff {
            best_diff = diff;
            best = Some(profile);
        }
    }

    // Only match if within 15% of depth
    if let Some(p) = best {
        if best_diff / p.depth_mm < 0.15 {
            return Some(p);
        }
    }

    None
}

/// Look up a profile by exact designation string.
pub fn find_profile_by_name(name: &str) -> Option<&'static SteelProfile> {
    let normalized = name.to_uppercase().replace(" ", "").replace("-", "");
    let db = profile_database();
    db.iter().find(|p| {
        let pn = p.designation.to_uppercase().replace(" ", "").replace("-", "");
        pn == normalized || normalized.starts_with(&pn) || pn.starts_with(&normalized)
    })
}

/// The embedded European steel profile database.
/// Covers the most common profiles used in Bulgarian/European steel construction.
pub fn profile_database() -> &'static [SteelProfile] {
    &PROFILES
}

static PROFILES: [SteelProfile; 62] = [
    // === IPE (European I-Beams) ===
    SteelProfile { designation: "IPE 80",  family: "IPE", depth_mm: 80.0,  flange_width_mm: 46.0,  web_thickness_mm: 3.8, flange_thickness_mm: 5.2,  kg_per_m: 6.0,   area_cm2: 7.64 },
    SteelProfile { designation: "IPE 100", family: "IPE", depth_mm: 100.0, flange_width_mm: 55.0,  web_thickness_mm: 4.1, flange_thickness_mm: 5.7,  kg_per_m: 8.1,   area_cm2: 10.3 },
    SteelProfile { designation: "IPE 120", family: "IPE", depth_mm: 120.0, flange_width_mm: 64.0,  web_thickness_mm: 4.4, flange_thickness_mm: 6.3,  kg_per_m: 10.4,  area_cm2: 13.2 },
    SteelProfile { designation: "IPE 140", family: "IPE", depth_mm: 140.0, flange_width_mm: 73.0,  web_thickness_mm: 4.7, flange_thickness_mm: 6.9,  kg_per_m: 12.9,  area_cm2: 16.4 },
    SteelProfile { designation: "IPE 160", family: "IPE", depth_mm: 160.0, flange_width_mm: 82.0,  web_thickness_mm: 5.0, flange_thickness_mm: 7.4,  kg_per_m: 15.8,  area_cm2: 20.1 },
    SteelProfile { designation: "IPE 180", family: "IPE", depth_mm: 180.0, flange_width_mm: 91.0,  web_thickness_mm: 5.3, flange_thickness_mm: 8.0,  kg_per_m: 18.8,  area_cm2: 23.9 },
    SteelProfile { designation: "IPE 200", family: "IPE", depth_mm: 200.0, flange_width_mm: 100.0, web_thickness_mm: 5.6, flange_thickness_mm: 8.5,  kg_per_m: 22.4,  area_cm2: 28.5 },
    SteelProfile { designation: "IPE 220", family: "IPE", depth_mm: 220.0, flange_width_mm: 110.0, web_thickness_mm: 5.9, flange_thickness_mm: 9.2,  kg_per_m: 26.2,  area_cm2: 33.4 },
    SteelProfile { designation: "IPE 240", family: "IPE", depth_mm: 240.0, flange_width_mm: 120.0, web_thickness_mm: 6.2, flange_thickness_mm: 9.8,  kg_per_m: 30.7,  area_cm2: 39.1 },
    SteelProfile { designation: "IPE 270", family: "IPE", depth_mm: 270.0, flange_width_mm: 135.0, web_thickness_mm: 6.6, flange_thickness_mm: 10.2, kg_per_m: 36.1,  area_cm2: 45.9 },
    SteelProfile { designation: "IPE 300", family: "IPE", depth_mm: 300.0, flange_width_mm: 150.0, web_thickness_mm: 7.1, flange_thickness_mm: 10.7, kg_per_m: 42.2,  area_cm2: 53.8 },
    SteelProfile { designation: "IPE 330", family: "IPE", depth_mm: 330.0, flange_width_mm: 160.0, web_thickness_mm: 7.5, flange_thickness_mm: 11.5, kg_per_m: 49.1,  area_cm2: 62.6 },
    SteelProfile { designation: "IPE 360", family: "IPE", depth_mm: 360.0, flange_width_mm: 170.0, web_thickness_mm: 8.0, flange_thickness_mm: 12.7, kg_per_m: 57.1,  area_cm2: 72.7 },
    SteelProfile { designation: "IPE 400", family: "IPE", depth_mm: 400.0, flange_width_mm: 180.0, web_thickness_mm: 8.6, flange_thickness_mm: 13.5, kg_per_m: 66.3,  area_cm2: 84.5 },
    SteelProfile { designation: "IPE 450", family: "IPE", depth_mm: 450.0, flange_width_mm: 190.0, web_thickness_mm: 9.4, flange_thickness_mm: 14.6, kg_per_m: 77.6,  area_cm2: 98.8 },
    SteelProfile { designation: "IPE 500", family: "IPE", depth_mm: 500.0, flange_width_mm: 200.0, web_thickness_mm: 10.2, flange_thickness_mm: 16.0, kg_per_m: 90.7, area_cm2: 115.5 },
    SteelProfile { designation: "IPE 550", family: "IPE", depth_mm: 550.0, flange_width_mm: 210.0, web_thickness_mm: 11.1, flange_thickness_mm: 17.2, kg_per_m: 106.0, area_cm2: 134.4 },
    SteelProfile { designation: "IPE 600", family: "IPE", depth_mm: 600.0, flange_width_mm: 220.0, web_thickness_mm: 12.0, flange_thickness_mm: 19.0, kg_per_m: 122.4, area_cm2: 156.0 },

    // === HEA (European Wide-Flange, light series) ===
    SteelProfile { designation: "HEA 100", family: "HEA", depth_mm: 96.0,  flange_width_mm: 100.0, web_thickness_mm: 5.0, flange_thickness_mm: 8.0,  kg_per_m: 16.7,  area_cm2: 21.2 },
    SteelProfile { designation: "HEA 120", family: "HEA", depth_mm: 114.0, flange_width_mm: 120.0, web_thickness_mm: 5.0, flange_thickness_mm: 8.0,  kg_per_m: 19.9,  area_cm2: 25.3 },
    SteelProfile { designation: "HEA 140", family: "HEA", depth_mm: 133.0, flange_width_mm: 140.0, web_thickness_mm: 5.5, flange_thickness_mm: 8.5,  kg_per_m: 24.7,  area_cm2: 31.4 },
    SteelProfile { designation: "HEA 160", family: "HEA", depth_mm: 152.0, flange_width_mm: 160.0, web_thickness_mm: 6.0, flange_thickness_mm: 9.0,  kg_per_m: 30.4,  area_cm2: 38.8 },
    SteelProfile { designation: "HEA 180", family: "HEA", depth_mm: 171.0, flange_width_mm: 180.0, web_thickness_mm: 6.0, flange_thickness_mm: 9.5,  kg_per_m: 35.5,  area_cm2: 45.3 },
    SteelProfile { designation: "HEA 200", family: "HEA", depth_mm: 190.0, flange_width_mm: 200.0, web_thickness_mm: 6.5, flange_thickness_mm: 10.0, kg_per_m: 42.3,  area_cm2: 53.8 },
    SteelProfile { designation: "HEA 220", family: "HEA", depth_mm: 210.0, flange_width_mm: 220.0, web_thickness_mm: 7.0, flange_thickness_mm: 11.0, kg_per_m: 50.5,  area_cm2: 64.3 },
    SteelProfile { designation: "HEA 240", family: "HEA", depth_mm: 230.0, flange_width_mm: 240.0, web_thickness_mm: 7.5, flange_thickness_mm: 12.0, kg_per_m: 60.3,  area_cm2: 76.8 },
    SteelProfile { designation: "HEA 260", family: "HEA", depth_mm: 250.0, flange_width_mm: 260.0, web_thickness_mm: 7.5, flange_thickness_mm: 12.5, kg_per_m: 68.2,  area_cm2: 86.8 },
    SteelProfile { designation: "HEA 280", family: "HEA", depth_mm: 270.0, flange_width_mm: 280.0, web_thickness_mm: 8.0, flange_thickness_mm: 13.0, kg_per_m: 76.4,  area_cm2: 97.3 },
    SteelProfile { designation: "HEA 300", family: "HEA", depth_mm: 290.0, flange_width_mm: 300.0, web_thickness_mm: 8.5, flange_thickness_mm: 14.0, kg_per_m: 88.3,  area_cm2: 112.5 },
    SteelProfile { designation: "HEA 360", family: "HEA", depth_mm: 350.0, flange_width_mm: 300.0, web_thickness_mm: 10.0, flange_thickness_mm: 17.5, kg_per_m: 112.2, area_cm2: 142.8 },
    SteelProfile { designation: "HEA 400", family: "HEA", depth_mm: 390.0, flange_width_mm: 300.0, web_thickness_mm: 11.0, flange_thickness_mm: 19.0, kg_per_m: 125.0, area_cm2: 159.0 },

    // === HEB (European Wide-Flange, heavy series) ===
    SteelProfile { designation: "HEB 100", family: "HEB", depth_mm: 100.0, flange_width_mm: 100.0, web_thickness_mm: 6.0, flange_thickness_mm: 10.0, kg_per_m: 20.4,  area_cm2: 26.0 },
    SteelProfile { designation: "HEB 120", family: "HEB", depth_mm: 120.0, flange_width_mm: 120.0, web_thickness_mm: 6.5, flange_thickness_mm: 11.0, kg_per_m: 26.7,  area_cm2: 34.0 },
    SteelProfile { designation: "HEB 140", family: "HEB", depth_mm: 140.0, flange_width_mm: 140.0, web_thickness_mm: 7.0, flange_thickness_mm: 12.0, kg_per_m: 33.7,  area_cm2: 43.0 },
    SteelProfile { designation: "HEB 160", family: "HEB", depth_mm: 160.0, flange_width_mm: 160.0, web_thickness_mm: 8.0, flange_thickness_mm: 13.0, kg_per_m: 42.6,  area_cm2: 54.3 },
    SteelProfile { designation: "HEB 180", family: "HEB", depth_mm: 180.0, flange_width_mm: 180.0, web_thickness_mm: 8.5, flange_thickness_mm: 14.0, kg_per_m: 51.2,  area_cm2: 65.3 },
    SteelProfile { designation: "HEB 200", family: "HEB", depth_mm: 200.0, flange_width_mm: 200.0, web_thickness_mm: 9.0, flange_thickness_mm: 15.0, kg_per_m: 61.3,  area_cm2: 78.1 },
    SteelProfile { designation: "HEB 220", family: "HEB", depth_mm: 220.0, flange_width_mm: 220.0, web_thickness_mm: 9.5, flange_thickness_mm: 16.0, kg_per_m: 71.5,  area_cm2: 91.0 },
    SteelProfile { designation: "HEB 240", family: "HEB", depth_mm: 240.0, flange_width_mm: 240.0, web_thickness_mm: 10.0, flange_thickness_mm: 17.0, kg_per_m: 83.2, area_cm2: 106.0 },
    SteelProfile { designation: "HEB 260", family: "HEB", depth_mm: 260.0, flange_width_mm: 260.0, web_thickness_mm: 10.0, flange_thickness_mm: 17.5, kg_per_m: 93.0, area_cm2: 118.4 },
    SteelProfile { designation: "HEB 280", family: "HEB", depth_mm: 280.0, flange_width_mm: 280.0, web_thickness_mm: 10.5, flange_thickness_mm: 18.0, kg_per_m: 103.1, area_cm2: 131.4 },
    SteelProfile { designation: "HEB 300", family: "HEB", depth_mm: 300.0, flange_width_mm: 300.0, web_thickness_mm: 11.0, flange_thickness_mm: 19.0, kg_per_m: 117.0, area_cm2: 149.1 },

    // === UPN (European U-Channels) ===
    SteelProfile { designation: "UPN 80",  family: "UPN", depth_mm: 80.0,  flange_width_mm: 45.0,  web_thickness_mm: 6.0, flange_thickness_mm: 8.0,  kg_per_m: 8.6,   area_cm2: 11.0 },
    SteelProfile { designation: "UPN 100", family: "UPN", depth_mm: 100.0, flange_width_mm: 50.0,  web_thickness_mm: 6.0, flange_thickness_mm: 8.5,  kg_per_m: 10.6,  area_cm2: 13.5 },
    SteelProfile { designation: "UPN 120", family: "UPN", depth_mm: 120.0, flange_width_mm: 55.0,  web_thickness_mm: 7.0, flange_thickness_mm: 9.0,  kg_per_m: 13.4,  area_cm2: 17.0 },
    SteelProfile { designation: "UPN 140", family: "UPN", depth_mm: 140.0, flange_width_mm: 60.0,  web_thickness_mm: 7.0, flange_thickness_mm: 10.0, kg_per_m: 16.0,  area_cm2: 20.4 },
    SteelProfile { designation: "UPN 160", family: "UPN", depth_mm: 160.0, flange_width_mm: 65.0,  web_thickness_mm: 7.5, flange_thickness_mm: 10.5, kg_per_m: 18.8,  area_cm2: 24.0 },
    SteelProfile { designation: "UPN 180", family: "UPN", depth_mm: 180.0, flange_width_mm: 70.0,  web_thickness_mm: 8.0, flange_thickness_mm: 11.0, kg_per_m: 22.0,  area_cm2: 28.0 },
    SteelProfile { designation: "UPN 200", family: "UPN", depth_mm: 200.0, flange_width_mm: 75.0,  web_thickness_mm: 8.5, flange_thickness_mm: 11.5, kg_per_m: 25.3,  area_cm2: 32.2 },
    SteelProfile { designation: "UPN 220", family: "UPN", depth_mm: 220.0, flange_width_mm: 80.0,  web_thickness_mm: 9.0, flange_thickness_mm: 12.5, kg_per_m: 29.4,  area_cm2: 37.4 },
    SteelProfile { designation: "UPN 240", family: "UPN", depth_mm: 240.0, flange_width_mm: 85.0,  web_thickness_mm: 9.5, flange_thickness_mm: 13.0, kg_per_m: 33.2,  area_cm2: 42.3 },
    SteelProfile { designation: "UPN 260", family: "UPN", depth_mm: 260.0, flange_width_mm: 90.0,  web_thickness_mm: 10.0, flange_thickness_mm: 14.0, kg_per_m: 37.9, area_cm2: 48.3 },
    SteelProfile { designation: "UPN 280", family: "UPN", depth_mm: 280.0, flange_width_mm: 95.0,  web_thickness_mm: 10.0, flange_thickness_mm: 15.0, kg_per_m: 41.8, area_cm2: 53.3 },
    SteelProfile { designation: "UPN 300", family: "UPN", depth_mm: 300.0, flange_width_mm: 100.0, web_thickness_mm: 10.0, flange_thickness_mm: 16.0, kg_per_m: 46.2, area_cm2: 58.8 },

    // === Equal Angles ===
    SteelProfile { designation: "L 40x40x4",  family: "L", depth_mm: 40.0,  flange_width_mm: 40.0,  web_thickness_mm: 4.0, flange_thickness_mm: 4.0, kg_per_m: 2.42,  area_cm2: 3.08 },
    SteelProfile { designation: "L 50x50x5",  family: "L", depth_mm: 50.0,  flange_width_mm: 50.0,  web_thickness_mm: 5.0, flange_thickness_mm: 5.0, kg_per_m: 3.77,  area_cm2: 4.80 },
    SteelProfile { designation: "L 60x60x6",  family: "L", depth_mm: 60.0,  flange_width_mm: 60.0,  web_thickness_mm: 6.0, flange_thickness_mm: 6.0, kg_per_m: 5.42,  area_cm2: 6.91 },
    SteelProfile { designation: "L 70x70x7",  family: "L", depth_mm: 70.0,  flange_width_mm: 70.0,  web_thickness_mm: 7.0, flange_thickness_mm: 7.0, kg_per_m: 7.38,  area_cm2: 9.40 },
    SteelProfile { designation: "L 80x80x8",  family: "L", depth_mm: 80.0,  flange_width_mm: 80.0,  web_thickness_mm: 8.0, flange_thickness_mm: 8.0, kg_per_m: 9.63,  area_cm2: 12.3 },
    SteelProfile { designation: "L 90x90x9",  family: "L", depth_mm: 90.0,  flange_width_mm: 90.0,  web_thickness_mm: 9.0, flange_thickness_mm: 9.0, kg_per_m: 12.2,  area_cm2: 15.5 },
    SteelProfile { designation: "L 100x100x10", family: "L", depth_mm: 100.0, flange_width_mm: 100.0, web_thickness_mm: 10.0, flange_thickness_mm: 10.0, kg_per_m: 15.0, area_cm2: 19.2 },
    SteelProfile { designation: "L 120x120x12", family: "L", depth_mm: 120.0, flange_width_mm: 120.0, web_thickness_mm: 12.0, flange_thickness_mm: 12.0, kg_per_m: 21.6, area_cm2: 27.5 },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_ipe200() {
        let profile = match_profile_by_depth(200.0, Some("IPE")).unwrap();
        assert_eq!(profile.designation, "IPE 200");
        assert!((profile.kg_per_m - 22.4).abs() < 0.1);
    }

    #[test]
    fn test_match_hea160() {
        let profile = match_profile_by_depth(152.0, Some("HEA")).unwrap();
        assert_eq!(profile.designation, "HEA 160");
    }

    #[test]
    fn test_find_by_name() {
        let p = find_profile_by_name("IPE200").unwrap();
        assert_eq!(p.designation, "IPE 200");
        assert!((p.kg_per_m - 22.4).abs() < 0.1);
    }

    #[test]
    fn test_no_match_wild_depth() {
        // 5000mm depth should not match anything
        assert!(match_profile_by_depth(5000.0, None).is_none());
    }

    #[test]
    fn test_database_completeness() {
        let db = profile_database();
        assert_eq!(db.len(), 62);
        // All have positive kg/m
        assert!(db.iter().all(|p| p.kg_per_m > 0.0));
    }
}
