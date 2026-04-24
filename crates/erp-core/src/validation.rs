use serde::{Deserialize, Serialize};

/// Severity level for a validation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Category of a validation rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleCategory {
    Structure,
    Completeness,
    Consistency,
    Compliance,
    Quality,
    Custom,
}

/// Result of evaluating one validation rule against one element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    pub rule_id: String,
    pub passed: bool,
    pub severity: Severity,
    pub message: String,
    pub element_ref: Option<String>,
    pub suggestion: Option<String>,
}

/// Overall status of a validation report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Pass,
    PassWithWarnings,
    Fail,
}

/// Aggregated validation report for a BOQ or other entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub status: ValidationStatus,
    pub score: f64,
    pub results: Vec<RuleResult>,
    pub total_rules: usize,
    pub passed: usize,
    pub warnings: usize,
    pub errors: usize,
}

/// Context passed to validation rules containing the data to validate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationContext {
    /// BOQ positions as JSON array (each position has quantity, unit_rate, description, ordinal, total, section)
    pub positions: Vec<serde_json::Value>,
    /// BOQ-level metadata
    pub metadata: serde_json::Value,
}

/// Trait for implementing a validation rule.
pub trait ValidationRule: Send + Sync {
    fn rule_id(&self) -> &str;
    fn severity(&self) -> Severity;
    fn category(&self) -> RuleCategory;
    fn validate(&self, ctx: &ValidationContext) -> Vec<RuleResult>;
}

/// Engine that runs a set of validation rules and aggregates results.
pub struct ValidationEngine {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ValidationEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    pub fn validate(&self, ctx: &ValidationContext) -> ValidationReport {
        let mut all_results = Vec::new();

        for rule in &self.rules {
            let results = rule.validate(ctx);
            all_results.extend(results);
        }

        let total_rules = all_results.len();
        let passed = all_results.iter().filter(|r| r.passed).count();
        let warnings = all_results
            .iter()
            .filter(|r| !r.passed && r.severity == Severity::Warning)
            .count();
        let errors = all_results
            .iter()
            .filter(|r| !r.passed && r.severity == Severity::Error)
            .count();

        let score = if total_rules > 0 {
            (passed as f64 / total_rules as f64) * 100.0
        } else {
            100.0
        };

        let status = if errors > 0 {
            ValidationStatus::Fail
        } else if warnings > 0 {
            ValidationStatus::PassWithWarnings
        } else {
            ValidationStatus::Pass
        };

        ValidationReport {
            status,
            score,
            results: all_results,
            total_rules,
            passed,
            warnings,
            errors,
        }
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysPass;
    impl ValidationRule for AlwaysPass {
        fn rule_id(&self) -> &str {
            "always_pass"
        }
        fn severity(&self) -> Severity {
            Severity::Error
        }
        fn category(&self) -> RuleCategory {
            RuleCategory::Quality
        }
        fn validate(&self, _ctx: &ValidationContext) -> Vec<RuleResult> {
            vec![RuleResult {
                rule_id: "always_pass".into(),
                passed: true,
                severity: Severity::Info,
                message: "All good".into(),
                element_ref: None,
                suggestion: None,
            }]
        }
    }

    #[test]
    fn test_engine_with_passing_rule() {
        let mut engine = ValidationEngine::new();
        engine.add_rule(Box::new(AlwaysPass));

        let ctx = ValidationContext {
            positions: vec![],
            metadata: serde_json::json!({}),
        };

        let report = engine.validate(&ctx);
        assert_eq!(report.status, ValidationStatus::Pass);
        assert_eq!(report.passed, 1);
        assert_eq!(report.errors, 0);
    }
}
