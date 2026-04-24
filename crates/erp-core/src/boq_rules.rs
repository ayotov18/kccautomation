use crate::validation::{
    RuleCategory, RuleResult, Severity, ValidationContext, ValidationRule,
};

// ──────────────────────────────────────────────
// Rule 1: Every position must have a quantity > 0
// ──────────────────────────────────────────────

pub struct PositionHasQuantity;

impl ValidationRule for PositionHasQuantity {
    fn rule_id(&self) -> &str {
        "position_has_quantity"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Completeness
    }
    fn validate(&self, ctx: &ValidationContext) -> Vec<RuleResult> {
        ctx.positions
            .iter()
            .enumerate()
            .map(|(i, pos)| {
                let qty = pos.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let passed = qty > 0.0;
                RuleResult {
                    rule_id: self.rule_id().into(),
                    passed,
                    severity: self.severity(),
                    message: if passed {
                        format!("Position {} has quantity {}", i + 1, qty)
                    } else {
                        format!("Position {} is missing a valid quantity", i + 1)
                    },
                    element_ref: Some(format!("position[{}]", i)),
                    suggestion: if passed {
                        None
                    } else {
                        Some("Set a quantity greater than 0".into())
                    },
                }
            })
            .collect()
    }
}

// ──────────────────────────────────────────────
// Rule 2: Every position must have a unit rate > 0
// ──────────────────────────────────────────────

pub struct PositionHasUnitRate;

impl ValidationRule for PositionHasUnitRate {
    fn rule_id(&self) -> &str {
        "position_has_unit_rate"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Completeness
    }
    fn validate(&self, ctx: &ValidationContext) -> Vec<RuleResult> {
        ctx.positions
            .iter()
            .enumerate()
            .map(|(i, pos)| {
                let rate = pos
                    .get("unit_rate")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let passed = rate > 0.0;
                RuleResult {
                    rule_id: self.rule_id().into(),
                    passed,
                    severity: self.severity(),
                    message: if passed {
                        format!("Position {} has unit rate {}", i + 1, rate)
                    } else {
                        format!("Position {} is missing a valid unit rate", i + 1)
                    },
                    element_ref: Some(format!("position[{}]", i)),
                    suggestion: if passed {
                        None
                    } else {
                        Some("Set a unit rate greater than 0".into())
                    },
                }
            })
            .collect()
    }
}

// ──────────────────────────────────────────────
// Rule 3: Every position must have a non-empty description
// ──────────────────────────────────────────────

pub struct PositionHasDescription;

impl ValidationRule for PositionHasDescription {
    fn rule_id(&self) -> &str {
        "position_has_description"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Completeness
    }
    fn validate(&self, ctx: &ValidationContext) -> Vec<RuleResult> {
        ctx.positions
            .iter()
            .enumerate()
            .map(|(i, pos)| {
                let desc = pos
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let passed = !desc.trim().is_empty();
                RuleResult {
                    rule_id: self.rule_id().into(),
                    passed,
                    severity: self.severity(),
                    message: if passed {
                        format!("Position {} has description", i + 1)
                    } else {
                        format!("Position {} is missing a description", i + 1)
                    },
                    element_ref: Some(format!("position[{}]", i)),
                    suggestion: if passed {
                        None
                    } else {
                        Some("Add a description for this position".into())
                    },
                }
            })
            .collect()
    }
}

// ──────────────────────────────────────────────
// Rule 4: No duplicate ordinals within the BOQ
// ──────────────────────────────────────────────

pub struct NoDuplicateOrdinals;

impl ValidationRule for NoDuplicateOrdinals {
    fn rule_id(&self) -> &str {
        "no_duplicate_ordinals"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Structure
    }
    fn validate(&self, ctx: &ValidationContext) -> Vec<RuleResult> {
        let mut seen = std::collections::HashMap::<String, Vec<usize>>::new();
        for (i, pos) in ctx.positions.iter().enumerate() {
            if let Some(ordinal) = pos.get("ordinal").and_then(|v| v.as_str()) {
                seen.entry(ordinal.to_string()).or_default().push(i);
            }
        }

        let mut results = Vec::new();
        for (ordinal, indices) in &seen {
            if indices.len() > 1 {
                for &idx in indices {
                    results.push(RuleResult {
                        rule_id: self.rule_id().into(),
                        passed: false,
                        severity: self.severity(),
                        message: format!(
                            "Duplicate ordinal '{}' at position {}",
                            ordinal,
                            idx + 1
                        ),
                        element_ref: Some(format!("position[{}]", idx)),
                        suggestion: Some("Ensure each position has a unique ordinal".into()),
                    });
                }
            }
        }

        // If no duplicates, emit one passing result
        if results.is_empty() {
            results.push(RuleResult {
                rule_id: self.rule_id().into(),
                passed: true,
                severity: Severity::Info,
                message: "No duplicate ordinals found".into(),
                element_ref: None,
                suggestion: None,
            });
        }

        results
    }
}

// ──────────────────────────────────────────────
// Rule 5: Unit rate outlier detection (>5x median = warning)
// ──────────────────────────────────────────────

pub struct UnitRateInRange;

impl ValidationRule for UnitRateInRange {
    fn rule_id(&self) -> &str {
        "unit_rate_in_range"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Quality
    }
    fn validate(&self, ctx: &ValidationContext) -> Vec<RuleResult> {
        let rates: Vec<f64> = ctx
            .positions
            .iter()
            .filter_map(|pos| pos.get("unit_rate").and_then(|v| v.as_f64()))
            .filter(|r| *r > 0.0)
            .collect();

        if rates.is_empty() {
            return vec![RuleResult {
                rule_id: self.rule_id().into(),
                passed: true,
                severity: Severity::Info,
                message: "No unit rates to check".into(),
                element_ref: None,
                suggestion: None,
            }];
        }

        let median = {
            let mut sorted = rates.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = sorted.len() / 2;
            if sorted.len() % 2 == 0 {
                (sorted[mid - 1] + sorted[mid]) / 2.0
            } else {
                sorted[mid]
            }
        };

        let threshold = median * 5.0;

        ctx.positions
            .iter()
            .enumerate()
            .map(|(i, pos)| {
                let rate = pos
                    .get("unit_rate")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let passed = rate <= threshold;
                RuleResult {
                    rule_id: self.rule_id().into(),
                    passed,
                    severity: self.severity(),
                    message: if passed {
                        format!("Position {} unit rate {:.2} is within range", i + 1, rate)
                    } else {
                        format!(
                            "Position {} unit rate {:.2} exceeds 5x median ({:.2})",
                            i + 1,
                            rate,
                            median
                        )
                    },
                    element_ref: Some(format!("position[{}]", i)),
                    suggestion: if passed {
                        None
                    } else {
                        Some("Verify this unit rate — it is significantly above the median".into())
                    },
                }
            })
            .collect()
    }
}

// ──────────────────────────────────────────────
// Rule 6: No negative values for quantity or unit rate
// ──────────────────────────────────────────────

pub struct NegativeValues;

impl ValidationRule for NegativeValues {
    fn rule_id(&self) -> &str {
        "no_negative_values"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Consistency
    }
    fn validate(&self, ctx: &ValidationContext) -> Vec<RuleResult> {
        ctx.positions
            .iter()
            .enumerate()
            .flat_map(|(i, pos)| {
                let mut results = Vec::new();
                if let Some(qty) = pos.get("quantity").and_then(|v| v.as_f64()) {
                    if qty < 0.0 {
                        results.push(RuleResult {
                            rule_id: self.rule_id().into(),
                            passed: false,
                            severity: self.severity(),
                            message: format!(
                                "Position {} has negative quantity: {}",
                                i + 1,
                                qty
                            ),
                            element_ref: Some(format!("position[{}].quantity", i)),
                            suggestion: Some("Quantity must be non-negative".into()),
                        });
                    }
                }
                if let Some(rate) = pos.get("unit_rate").and_then(|v| v.as_f64()) {
                    if rate < 0.0 {
                        results.push(RuleResult {
                            rule_id: self.rule_id().into(),
                            passed: false,
                            severity: self.severity(),
                            message: format!(
                                "Position {} has negative unit rate: {}",
                                i + 1,
                                rate
                            ),
                            element_ref: Some(format!("position[{}].unit_rate", i)),
                            suggestion: Some("Unit rate must be non-negative".into()),
                        });
                    }
                }
                if results.is_empty() {
                    results.push(RuleResult {
                        rule_id: self.rule_id().into(),
                        passed: true,
                        severity: Severity::Info,
                        message: format!("Position {} values are non-negative", i + 1),
                        element_ref: Some(format!("position[{}]", i)),
                        suggestion: None,
                    });
                }
                results
            })
            .collect()
    }
}

// ──────────────────────────────────────────────
// Rule 7: Section structure (positions should have a section)
// ──────────────────────────────────────────────

pub struct SectionStructure;

impl ValidationRule for SectionStructure {
    fn rule_id(&self) -> &str {
        "section_structure"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Structure
    }
    fn validate(&self, ctx: &ValidationContext) -> Vec<RuleResult> {
        ctx.positions
            .iter()
            .enumerate()
            .map(|(i, pos)| {
                let has_section = pos
                    .get("section")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.trim().is_empty());
                RuleResult {
                    rule_id: self.rule_id().into(),
                    passed: has_section,
                    severity: self.severity(),
                    message: if has_section {
                        format!("Position {} belongs to a section", i + 1)
                    } else {
                        format!("Position {} has no section assignment", i + 1)
                    },
                    element_ref: Some(format!("position[{}]", i)),
                    suggestion: if has_section {
                        None
                    } else {
                        Some("Assign this position to a section for better organization".into())
                    },
                }
            })
            .collect()
    }
}

// ──────────────────────────────────────────────
// Rule 8: Total cost benchmarks (no single position > 50% of total)
// ──────────────────────────────────────────────

pub struct TotalCostBenchmarks;

impl ValidationRule for TotalCostBenchmarks {
    fn rule_id(&self) -> &str {
        "total_cost_benchmarks"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Quality
    }
    fn validate(&self, ctx: &ValidationContext) -> Vec<RuleResult> {
        let totals: Vec<f64> = ctx
            .positions
            .iter()
            .map(|pos| pos.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .collect();

        let grand_total: f64 = totals.iter().sum();

        if grand_total <= 0.0 {
            return vec![RuleResult {
                rule_id: self.rule_id().into(),
                passed: true,
                severity: Severity::Info,
                message: "No positive totals to benchmark".into(),
                element_ref: None,
                suggestion: None,
            }];
        }

        ctx.positions
            .iter()
            .enumerate()
            .map(|(i, _pos)| {
                let total = totals[i];
                let pct = (total / grand_total) * 100.0;
                let passed = pct <= 50.0;
                RuleResult {
                    rule_id: self.rule_id().into(),
                    passed,
                    severity: self.severity(),
                    message: if passed {
                        format!("Position {} is {:.1}% of total cost", i + 1, pct)
                    } else {
                        format!(
                            "Position {} is {:.1}% of total cost — exceeds 50% threshold",
                            i + 1,
                            pct
                        )
                    },
                    element_ref: Some(format!("position[{}]", i)),
                    suggestion: if passed {
                        None
                    } else {
                        Some(
                            "This position dominates the BOQ — verify it is correct".into(),
                        )
                    },
                }
            })
            .collect()
    }
}

/// Create a default validation engine with all 8 BOQ rules.
pub fn default_boq_engine() -> crate::validation::ValidationEngine {
    let mut engine = crate::validation::ValidationEngine::new();
    engine.add_rule(Box::new(PositionHasQuantity));
    engine.add_rule(Box::new(PositionHasUnitRate));
    engine.add_rule(Box::new(PositionHasDescription));
    engine.add_rule(Box::new(NoDuplicateOrdinals));
    engine.add_rule(Box::new(UnitRateInRange));
    engine.add_rule(Box::new(NegativeValues));
    engine.add_rule(Box::new(SectionStructure));
    engine.add_rule(Box::new(TotalCostBenchmarks));
    engine
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::ValidationContext;

    fn make_position(ordinal: &str, qty: f64, rate: f64, desc: &str) -> serde_json::Value {
        serde_json::json!({
            "ordinal": ordinal,
            "quantity": qty,
            "unit_rate": rate,
            "description": desc,
            "total": qty * rate,
            "section": "A"
        })
    }

    #[test]
    fn test_all_rules_pass() {
        let ctx = ValidationContext {
            positions: vec![
                make_position("1.1", 10.0, 50.0, "Concrete works"),
                make_position("1.2", 5.0, 40.0, "Rebar supply"),
            ],
            metadata: serde_json::json!({}),
        };

        let engine = default_boq_engine();
        let report = engine.validate(&ctx);
        assert_eq!(report.errors, 0);
    }

    #[test]
    fn test_duplicate_ordinals_detected() {
        let ctx = ValidationContext {
            positions: vec![
                make_position("1.1", 10.0, 50.0, "First"),
                make_position("1.1", 5.0, 40.0, "Second"),
            ],
            metadata: serde_json::json!({}),
        };

        let rule = NoDuplicateOrdinals;
        let results = rule.validate(&ctx);
        let failures: Vec<_> = results.iter().filter(|r| !r.passed).collect();
        assert_eq!(failures.len(), 2);
    }

    #[test]
    fn test_outlier_unit_rate() {
        let ctx = ValidationContext {
            positions: vec![
                make_position("1", 1.0, 10.0, "A"),
                make_position("2", 1.0, 12.0, "B"),
                make_position("3", 1.0, 11.0, "C"),
                make_position("4", 1.0, 500.0, "Outlier"),
            ],
            metadata: serde_json::json!({}),
        };

        let rule = UnitRateInRange;
        let results = rule.validate(&ctx);
        let outliers: Vec<_> = results.iter().filter(|r| !r.passed).collect();
        assert_eq!(outliers.len(), 1);
        assert!(outliers[0].message.contains("Outlier") || outliers[0].message.contains("4"));
    }
}
