//! Sanity validators for KSS reports.
//!
//! Industry cross-checks sourced from Bulgarian construction research:
//!   - armature mass / concrete volume should be ~80–200 kg/m³ (alert at 50/300)
//!   - formwork area / concrete volume should be ~6–12 m²/m³
//!   - plaster area / floor footprint should be ~2.5–3.5×
//!
//! Flags are stored on the report so the frontend can surface them as warning
//! banners and the tender-committee verification checklist can run against
//! them.

use crate::kss::types::{KssLineItem, QuantityItem, SectionedKssReport};
use serde::{Deserialize, Serialize};

/// Canonical units the pipeline accepts. Anything else must be normalised
/// before persistence — unknown units break downstream price-lookup joins.
pub const ALLOWED_UNITS: &[&str] = &[
    "m²", "М2", "м2",
    "m³", "М3", "м3",
    "м", "m", "мл",
    "кг", "kg",
    "тон", "t",
    "бр.", "бр", "pcs",
    "компл.", "компл",
    "л", "l",
    "m²·mm", // from the quantity_scraper per-mm TDS case
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub check: String,
    pub severity: Severity,
    pub message: String,
    pub expected_range: Option<String>,
    pub actual: Option<f64>,
}

/// Run all sanity checks on a sectioned report.
pub fn validate(report: &SectionedKssReport) -> Vec<ValidationWarning> {
    let mut out = Vec::new();
    out.extend(check_rebar_to_concrete_ratio(report));
    out.extend(check_formwork_to_concrete_ratio(report));
    out.extend(check_mandatory_sections_for_newbuild(report));
    out.extend(check_nonzero_quantities(report));
    out
}

fn sum_by_sek_prefix(report: &SectionedKssReport, prefix: &str) -> f64 {
    report
        .sections
        .iter()
        .flat_map(|s| s.items.iter())
        .filter(|i| i.sek_code.starts_with(prefix))
        .map(|i| i.quantity)
        .sum()
}

fn check_rebar_to_concrete_ratio(report: &SectionedKssReport) -> Vec<ValidationWarning> {
    let rebar_kg = sum_by_sek_prefix(report, "СЕК03");
    let concrete_m3 = sum_by_sek_prefix(report, "СЕК04");
    if concrete_m3 <= 0.0 || rebar_kg <= 0.0 {
        return vec![];
    }
    let ratio = rebar_kg / concrete_m3;
    let mut out = Vec::new();
    if ratio < 50.0 {
        out.push(ValidationWarning {
            check: "rebar_per_concrete".into(),
            severity: Severity::Error,
            message: format!(
                "Армировка {:.0} кг / бетон {:.1} м³ = {:.1} kg/m³ — под минимума (50). Възможно липсва армировка.",
                rebar_kg, concrete_m3, ratio
            ),
            expected_range: Some("80-200 kg/m³".into()),
            actual: Some(ratio),
        });
    } else if ratio < 80.0 {
        out.push(ValidationWarning {
            check: "rebar_per_concrete".into(),
            severity: Severity::Warning,
            message: format!(
                "Армировка/бетон = {:.0} kg/m³ — под обичайния диапазон.",
                ratio
            ),
            expected_range: Some("80-200 kg/m³".into()),
            actual: Some(ratio),
        });
    } else if ratio > 300.0 {
        out.push(ValidationWarning {
            check: "rebar_per_concrete".into(),
            severity: Severity::Error,
            message: format!(
                "Армировка {:.0} кг / бетон {:.1} м³ = {:.0} kg/m³ — над максимума (300). Вероятна грешка в изчислението.",
                rebar_kg, concrete_m3, ratio
            ),
            expected_range: Some("80-200 kg/m³".into()),
            actual: Some(ratio),
        });
    } else if ratio > 200.0 {
        out.push(ValidationWarning {
            check: "rebar_per_concrete".into(),
            severity: Severity::Warning,
            message: format!("Армировка/бетон = {:.0} kg/m³ — над обичайния диапазон.", ratio),
            expected_range: Some("80-200 kg/m³".into()),
            actual: Some(ratio),
        });
    }
    out
}

fn check_formwork_to_concrete_ratio(report: &SectionedKssReport) -> Vec<ValidationWarning> {
    let formwork_m2 = sum_by_sek_prefix(report, "СЕК02");
    let concrete_m3 = sum_by_sek_prefix(report, "СЕК04");
    if concrete_m3 <= 0.0 || formwork_m2 <= 0.0 {
        return vec![];
    }
    let ratio = formwork_m2 / concrete_m3;
    if (6.0..=12.0).contains(&ratio) {
        return vec![];
    }
    let severity = if !(4.0..=16.0).contains(&ratio) {
        Severity::Error
    } else {
        Severity::Warning
    };
    vec![ValidationWarning {
        check: "formwork_per_concrete".into(),
        severity,
        message: format!(
            "Кофраж/бетон = {:.1} м²/м³ — извън обичайните 6–12.",
            ratio
        ),
        expected_range: Some("6-12 m²/m³".into()),
        actual: Some(ratio),
    }]
}

/// Образец 9.1 requires земни + кофражни + армировъчни + бетонови for any
/// new-build. A renovation-only job won't have all of them — renovation reports
/// are flagged separately so this check is skipped.
fn check_mandatory_sections_for_newbuild(report: &SectionedKssReport) -> Vec<ValidationWarning> {
    let is_renovation = report.sections.iter().any(|s| s.sek_group == "СЕК49")
        && report.sections.iter().filter(|s| !s.items.is_empty()).count() <= 6;
    if is_renovation {
        return vec![];
    }
    let required = ["СЕК01", "СЕК02", "СЕК03", "СЕК04"];
    let mut out = Vec::new();
    for req in required {
        let present = report
            .sections
            .iter()
            .any(|s| s.sek_group == req && !s.items.is_empty());
        if !present {
            out.push(ValidationWarning {
                check: "missing_mandatory_section".into(),
                severity: Severity::Error,
                message: format!(
                    "Липсва задължителна секция {} за ново строителство.",
                    req
                ),
                expected_range: None,
                actual: None,
            });
        }
    }
    out
}

fn check_nonzero_quantities(report: &SectionedKssReport) -> Vec<ValidationWarning> {
    let zero_count = report
        .sections
        .iter()
        .flat_map(|s| s.items.iter())
        .filter(|i| i.quantity <= 0.0)
        .count();
    if zero_count == 0 {
        return vec![];
    }
    vec![ValidationWarning {
        check: "zero_quantity_items".into(),
        severity: Severity::Warning,
        message: format!("{} позиции с нулево количество — проверете входните данни.", zero_count),
        expected_range: None,
        actual: Some(zero_count as f64),
    }]
}

// ── Schema-contract tests (research's "1-10-100 rule" ──
//   Run at the extractor boundary BEFORE prices are computed. Goal: never
//   let a structurally-invalid row reach the AI or the final КСС.

/// Result of a pre-AI schema audit. `violations` is empty when the extractor
/// emitted a clean batch; otherwise each entry names the row and the rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaAuditReport {
    pub total_rows: usize,
    pub passed: usize,
    pub needs_review: usize,
    pub violations: Vec<ValidationWarning>,
}

/// Validate every QuantityItem against the canonical contract. Rows that fail
/// get `needs_review = true` stamped on them; critical rows (e.g. zero quantity,
/// bad SEK code) are still emitted so the human has a chance to fix them — we
/// never silently drop data.
pub fn validate_schema(items: &mut [QuantityItem]) -> SchemaAuditReport {
    let mut violations = Vec::new();
    let mut passed = 0usize;
    let mut flagged = 0usize;

    for (idx, item) in items.iter_mut().enumerate() {
        let mut row_violations: Vec<ValidationWarning> = Vec::new();

        if item.quantity <= 0.0 {
            row_violations.push(ValidationWarning {
                check: "quantity_non_positive".into(),
                severity: Severity::Error,
                message: format!("Ред {idx}: количество {:.3} ≤ 0.", item.quantity),
                expected_range: Some("> 0".into()),
                actual: Some(item.quantity),
            });
        }
        if !ALLOWED_UNITS.iter().any(|u| *u == item.unit) {
            row_violations.push(ValidationWarning {
                check: "unknown_unit".into(),
                severity: Severity::Warning,
                message: format!("Ред {idx}: мярка \"{}\" не е в канонични.", item.unit),
                expected_range: Some(format!("{:?}", ALLOWED_UNITS)),
                actual: None,
            });
        }
        if !is_valid_sek_code(&item.suggested_sek_code) {
            row_violations.push(ValidationWarning {
                check: "invalid_sek_code".into(),
                severity: Severity::Warning,
                message: format!("Ред {idx}: код \"{}\" не отговаря на СЕК формат.", item.suggested_sek_code),
                expected_range: Some("СЕКxx или СЕКxx.yyy".into()),
                actual: None,
            });
        }
        if item.description.trim().chars().count() < 3 {
            row_violations.push(ValidationWarning {
                check: "description_too_short".into(),
                severity: Severity::Warning,
                message: format!("Ред {idx}: описание \"{}\" твърде кратко.", item.description),
                expected_range: Some(">= 3 символа".into()),
                actual: None,
            });
        }
        if !(0.0..=1.0).contains(&item.geometry_confidence) {
            row_violations.push(ValidationWarning {
                check: "confidence_out_of_range".into(),
                severity: Severity::Error,
                message: format!("Ред {idx}: geometry_confidence {:.2} извън [0,1].", item.geometry_confidence),
                expected_range: Some("0.0-1.0".into()),
                actual: Some(item.geometry_confidence),
            });
        }

        if row_violations.is_empty() {
            passed += 1;
        } else {
            // Any violation auto-flags the row for human review before final
            // КСС generation. Does NOT remove the row — the human gets to see
            // and edit it via the existing Accept/Deny widget.
            item.needs_review = true;
            flagged += 1;
            violations.extend(row_violations);
        }
    }

    SchemaAuditReport {
        total_rows: items.len(),
        passed,
        needs_review: flagged,
        violations,
    }
}

fn is_valid_sek_code(code: &str) -> bool {
    // Permissive: СЕКxx, СЕКxx.yyy, or bare xx.yyy legacy codes.
    let trimmed = code.trim();
    if trimmed.is_empty() || trimmed.len() > 24 { return false; }
    // The scrubbed form must at least look like either a SEK prefix or a digit pattern.
    trimmed.starts_with("СЕК")
        || trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
}

/// Pre-AI gate: decides whether the deterministic pipeline has enough trusted
/// rows to warrant running at all. If every row is an `assumed_default` /
/// `ai_inferred` (confidence < 0.5), the caller is expected to route the user
/// to the AI-KSS flow instead.
pub fn has_sufficient_geometry(items: &[QuantityItem]) -> bool {
    if items.is_empty() { return false; }
    let trusted = items.iter().filter(|q| q.geometry_confidence >= 0.60).count();
    trusted * 2 >= items.len() // at least 50% trusted
}

/// Post-AI gate: verifies the AI respected the "don't invent low-confidence
/// numbers" rule. Returns the set of rows where the AI overwrote a quantity
/// on a row flagged `needs_review`.
pub fn detect_ai_overwrites(
    geometry_items: &[QuantityItem],
    ai_items: &[KssLineItem],
) -> Vec<ValidationWarning> {
    let mut out = Vec::new();
    for (g, a) in geometry_items.iter().zip(ai_items.iter()) {
        if g.needs_review && (g.quantity - a.quantity).abs() > 0.001 {
            out.push(ValidationWarning {
                check: "ai_overwrote_flagged_quantity".into(),
                severity: Severity::Warning,
                message: format!(
                    "AI промени количество на ред с нисък confidence ({}) — {:.3} → {:.3}. Върнато към геометричната стойност.",
                    a.sek_code, g.quantity, a.quantity,
                ),
                expected_range: Some(format!("{:.3}", g.quantity)),
                actual: Some(a.quantity),
            });
        }
    }
    out
}

#[cfg(test)]
mod schema_tests {
    use super::*;
    use crate::kss::types::{ExtractionMethod, QuantityItem};

    fn ok_item() -> QuantityItem {
        QuantityItem::new(
            "СЕК05", "Зидария тухлена 25 см",
            "М2", 120.0, "СЕК05.002",
            ExtractionMethod::WallAreaFromCenterline,
        )
    }

    #[test]
    fn clean_batch_has_no_violations() {
        let mut items = vec![ok_item(), ok_item()];
        let rep = validate_schema(&mut items);
        assert_eq!(rep.violations.len(), 0);
        assert_eq!(rep.passed, 2);
        assert!(!items.iter().any(|i| i.needs_review && !i.extraction_method.needs_review()));
    }

    #[test]
    fn zero_quantity_flags_row() {
        let mut items = vec![QuantityItem::new(
            "СЕК05", "Zero", "М2", 0.0, "СЕК05.002",
            ExtractionMethod::WallAreaFromCenterline,
        )];
        let rep = validate_schema(&mut items);
        assert!(!rep.violations.is_empty());
        assert!(items[0].needs_review);
    }

    #[test]
    fn unknown_unit_flags_row() {
        let mut items = vec![QuantityItem::new(
            "СЕК05", "Bad unit", "furlongs", 10.0, "СЕК05.002",
            ExtractionMethod::BlockInstanceCount,
        )];
        let rep = validate_schema(&mut items);
        assert!(rep.violations.iter().any(|v| v.check == "unknown_unit"));
    }

    #[test]
    fn sufficient_geometry_gate_behaves() {
        let mut trusted = vec![ok_item(), ok_item()];
        assert!(has_sufficient_geometry(&trusted));
        trusted[0].geometry_confidence = 0.3;
        trusted[1].geometry_confidence = 0.3;
        assert!(!has_sufficient_geometry(&trusted));
        assert!(!has_sufficient_geometry(&[]));
    }

    #[test]
    fn detects_ai_overwrite_of_flagged_row() {
        use crate::kss::types::KssLineItem;
        let mut g = ok_item();
        g.needs_review = true;
        g.quantity = 10.0;
        let a = KssLineItem {
            item_no: 1,
            sek_code: "СЕК05.002".into(),
            description: "…".into(),
            unit: "М2".into(),
            quantity: 42.0, // different → overwrite
            ..Default::default()
        };
        let warns = detect_ai_overwrites(&[g], &[a]);
        assert_eq!(warns.len(), 1);
        assert_eq!(warns[0].check, "ai_overwrote_flagged_quantity");
    }
}

/// Helper: total a specific field across all items.
#[allow(dead_code)]
pub fn total_materials(report: &SectionedKssReport) -> f64 {
    report
        .sections
        .iter()
        .flat_map(|s| s.items.iter())
        .map(|i: &KssLineItem| i.material_price * i.quantity)
        .sum()
}
