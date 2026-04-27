//! KSS Generation Audit Trail — captures data from every pipeline phase.
//!
//! Two view modes consume the same data:
//! - DEV: raw technical detail (entity counts, prompts, merge decisions)
//! - USER: natural-language explanation ("We found 4 rooms totaling 85m²...")

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Top-level audit ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KssAuditTrail {
    pub phase1_upload: UploadParseAudit,
    pub phase2_analysis: AnalysisAudit,
    pub phase3_quantities: QuantityAudit,
    pub phase4_prices: Option<PriceResearchAudit>,
    pub phase5_generation: GenerationAudit,
    pub phase6_report: ReportAudit,
    pub timings: Vec<PhaseTimingEntry>,
    pub warnings: Vec<AuditWarning>,
    pub errors: Vec<AuditError>,
    /// Spatial structures (modules) detected in the drawing. One entry for a
    /// single-module drawing; N entries for multi-module sheets that pack
    /// several floor plans into one DWG. Per-module subtotals and KSS line
    /// counts are populated by the worker after generation completes so the
    /// frontend can render one tab per module with its own subtotal.
    #[serde(default)]
    pub structures: Vec<StructureAudit>,
}

impl KssAuditTrail {
    /// Record a phase timing entry.
    pub fn record_timing(&mut self, phase: &str, duration_ms: u64) {
        self.timings.push(PhaseTimingEntry {
            phase: phase.to_string(),
            duration_ms,
        });
    }

    /// Add a warning.
    pub fn warn(&mut self, phase: &str, message: impl Into<String>) {
        self.warnings.push(AuditWarning {
            phase: phase.to_string(),
            message: message.into(),
        });
    }

    /// Add an error.
    pub fn error(&mut self, phase: &str, message: impl Into<String>) {
        self.errors.push(AuditError {
            phase: phase.to_string(),
            message: message.into(),
        });
    }

    /// Total pipeline duration in ms.
    pub fn total_duration_ms(&self) -> u64 {
        self.timings.iter().map(|t| t.duration_ms).sum()
    }

    /// Generate user-friendly summaries for each phase.
    pub fn to_user_summary(&self) -> Vec<UserPhaseSummary> {
        let mut summaries = Vec::new();

        // Phase 1
        let p1 = &self.phase1_upload;
        let mut highlights = Vec::new();
        if let Some(ref conv) = p1.oda_conversion {
            if conv.success {
                highlights.push(format!("DWG converted to DXF in {}ms", conv.duration_ms));
            }
        }
        // Show populated/total so an 80-layer drawing with 2 populated ones
        // isn't mistakenly trusted as semantically rich.
        let layers_desc = if p1.layer_count_populated > 0 && p1.layer_count_populated != p1.layer_count {
            format!(
                "{} entities across {} populated layers ({} total)",
                p1.total_entities, p1.layer_count_populated, p1.layer_count,
            )
        } else {
            format!("{} entities across {} layers", p1.total_entities, p1.layer_count)
        };
        highlights.push(layers_desc.clone());
        if !p1.units_detected.is_empty() {
            highlights.push(format!("Units: {} ({})", p1.units_detected, p1.units_detection_method));
        }
        if !p1.dxf_version.is_empty() {
            highlights.push(format!("DXF version: {}", p1.dxf_version));
        }
        summaries.push(UserPhaseSummary {
            phase_number: 1,
            phase_name: "Upload & Parse".into(),
            duration_ms: self.phase_duration("upload_parse"),
            summary: format!(
                "Drawing '{}' parsed as {} — {}.",
                p1.original_filename,
                if p1.original_format.to_lowercase() == "dwg" { "DWG (converted to DXF)" } else { "DXF" },
                layers_desc,
            ),
            highlights,
        });

        // Phase 2
        let p2 = &self.phase2_analysis;
        let mut highlights = Vec::new();
        highlights.push(format!("Drawing type: {}", p2.drawing_type_classification));
        if p2.drm_total_matches > 0 {
            highlights.push(format!("{} DRM matches ({} auto-overrides)", p2.drm_total_matches, p2.drm_auto_overrides));
        }
        if !p2.layer_sek_mappings.is_empty() {
            highlights.push(format!("{} layer→SEK mappings", p2.layer_sek_mappings.len()));
        }
        summaries.push(UserPhaseSummary {
            phase_number: 2,
            phase_name: "Analysis & Features".into(),
            duration_ms: self.phase_duration("analysis"),
            summary: format!(
                "Detected as {} drawing. {} layer mappings, {} DRM historical matches.",
                p2.drawing_type_classification,
                p2.layer_sek_mappings.len(),
                p2.drm_total_matches,
            ),
            highlights,
        });

        // Phase 3
        let p3 = &self.phase3_quantities;
        summaries.push(UserPhaseSummary {
            phase_number: 3,
            phase_name: "Quantity Calculation".into(),
            duration_ms: self.phase_duration("quantities"),
            summary: format!(
                "{} quantity items extracted. Unit scale: {}x ({}).",
                p3.items.len(), p3.unit_scale_factor, p3.unit_scale_source,
            ),
            highlights: p3.items.iter().take(5).map(|i| {
                format!("{}: {:.2} {}", i.description, i.calculated_quantity, i.unit)
            }).collect(),
        });

        // Phase 4 (optional)
        if let Some(ref p4) = self.phase4_prices {
            let mut highlights = Vec::new();
            for (source, count) in &p4.price_source_breakdown {
                highlights.push(format!("{}: {} items", source, count));
            }
            summaries.push(UserPhaseSummary {
                phase_number: 4,
                phase_name: "Price Research".into(),
                duration_ms: self.phase_duration("price_research"),
                summary: format!(
                    "Model {} found {} items, {} approved by user.",
                    p4.model_used, p4.items_parsed, p4.items_approved,
                ),
                highlights,
            });
        }

        // Phase 5
        let p5 = &self.phase5_generation;
        let mut highlights = Vec::new();
        highlights.push(format!("Mode: {}", p5.mode));
        highlights.push(format!("Rule-based: {} items ({:.2} €)", p5.rule_based_items, p5.rule_based_total_eur));
        if p5.ai_enabled {
            highlights.push(format!("AI: {} generated, {} validated", p5.ai_items_generated, p5.ai_items_validated));
            if !p5.ai_rejection_reasons.is_empty() {
                highlights.push(format!("{} items rejected by validation", p5.ai_rejection_reasons.len()));
            }
        }
        summaries.push(UserPhaseSummary {
            phase_number: 5,
            phase_name: "KSS Generation".into(),
            duration_ms: self.phase_duration("generation"),
            summary: format!(
                "{} mode. {} rule-based items, AI {}.",
                p5.mode,
                p5.rule_based_items,
                if p5.ai_enabled {
                    format!("generated {} items ({}ms)", p5.ai_items_generated, p5.ai_latency_ms)
                } else {
                    "disabled".into()
                },
            ),
            highlights,
        });

        // Phase 6
        let p6 = &self.phase6_report;
        summaries.push(UserPhaseSummary {
            phase_number: 6,
            phase_name: "Final Report".into(),
            duration_ms: self.phase_duration("report"),
            summary: format!(
                "{} items across {} sections. Total: {:.2} € (incl. VAT).",
                p6.total_items, p6.total_sections, p6.total_with_vat_eur,
            ),
            highlights: vec![
                format!("Subtotal: {:.2} €", p6.subtotal_eur),
                format!("VAT: {:.2} €", p6.vat_eur),
                format!("DRM artifacts recorded: {}", p6.drm_artifacts_recorded),
            ],
        });

        summaries
    }

    fn phase_duration(&self, phase: &str) -> u64 {
        self.timings.iter()
            .find(|t| t.phase == phase)
            .map(|t| t.duration_ms)
            .unwrap_or(0)
    }
}

// ─── Phase 1: Upload & Parse ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UploadParseAudit {
    pub original_filename: String,
    pub original_format: String,
    pub file_size_bytes: u64,
    pub oda_conversion: Option<OdaConversionAudit>,
    pub dxf_version: String,
    pub units_detected: String,
    pub units_detection_method: String,
    pub entity_counts_by_type: HashMap<String, usize>,
    pub total_entities: usize,
    /// Total count of layers declared in the DXF header (many may be empty templates).
    pub layer_count: usize,
    pub layer_list: Vec<String>,
    /// Layers that actually contain at least one entity. Filtering stages only
    /// consider these; reporting surfaces both counts so an 80-layer drawing
    /// with 2 populated ones isn't mistakenly trusted as semantically rich.
    #[serde(default)]
    pub layer_count_populated: usize,
    #[serde(default)]
    pub populated_layers: Vec<PopulatedLayerAudit>,
    pub block_definitions_count: usize,
    pub dimension_count: usize,
    pub annotation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PopulatedLayerAudit {
    pub name: String,
    pub entity_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructureAudit {
    pub structure_id: Option<String>,
    pub structure_index: usize,
    pub label: String,
    pub bbox_min_x: f64,
    pub bbox_min_y: f64,
    pub bbox_max_x: f64,
    pub bbox_max_y: f64,
    pub entity_count: usize,
    pub dimension_count: usize,
    pub annotation_count: usize,
    /// Number of KSS line items generated for this module.
    #[serde(default)]
    pub line_item_count: usize,
    /// Subtotal in € for this module's line items (before VAT/overhead).
    #[serde(default)]
    pub subtotal_eur: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OdaConversionAudit {
    pub success: bool,
    pub duration_ms: u64,
    pub output_size_bytes: u64,
    pub error_message: Option<String>,
}

// ─── Phase 2: Analysis & Features ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisAudit {
    pub features_detected: Vec<FeatureAuditEntry>,
    pub layer_sek_mappings: Vec<LayerSekMappingAudit>,
    pub block_fixture_mappings: Vec<BlockFixtureMappingAudit>,
    pub drm_total_matches: usize,
    pub drm_auto_overrides: usize,
    pub drm_matches: Vec<DrmMatchAudit>,
    pub drawing_type_classification: String,
    pub drawing_type_reasoning: DrawingTypeReasoning,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureAuditEntry {
    pub feature_type: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayerSekMappingAudit {
    pub layer_name: String,
    pub sek_group: String,
    pub description: String,
    pub entity_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockFixtureMappingAudit {
    pub block_name: String,
    pub sek_group: String,
    pub description: String,
    pub insert_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DrmMatchAudit {
    pub input_key: String,
    pub sek_code: String,
    pub similarity: f32,
    pub confidence: f64,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DrawingTypeReasoning {
    pub architectural_layer_count: usize,
    pub has_fixture_blocks: bool,
    pub has_area_annotations: bool,
    pub has_steel_features: bool,
    pub is_steel_fabrication: bool,
}

// ─── Phase 3: Quantity Calculation ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuantityAudit {
    pub items: Vec<QuantityItemAudit>,
    pub unit_scale_factor: f64,
    pub unit_scale_source: String,
    pub multi_view_correction_applied: bool,
    pub multi_view_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuantityItemAudit {
    pub sek_code: String,
    pub description: String,
    pub unit: String,
    pub calculated_quantity: f64,
    pub formula_used: String,
    pub source_layer: String,
}

// ─── Phase 4: Price Research (AI mode only) ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceResearchAudit {
    pub model_used: String,
    pub prompt_preview: String,
    pub response_preview: String,
    pub latency_ms: u64,
    pub sources_cited: Vec<String>,
    pub items_parsed: usize,
    pub items_approved: usize,
    pub items_rejected: usize,
    pub json_repair_attempted: bool,
    pub json_repair_success: bool,
    pub price_source_breakdown: HashMap<String, usize>,
}

// ─── Phase 5: KSS Generation ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerationAudit {
    pub mode: String,
    pub rule_based_items: usize,
    pub rule_based_total_eur: f64,
    pub ai_enabled: bool,
    pub ai_prompt_system_preview: String,
    pub ai_prompt_user_preview: String,
    pub ai_response_preview: String,
    pub ai_model: String,
    pub ai_latency_ms: u64,
    pub ai_token_count: usize,
    pub ai_items_generated: usize,
    pub ai_items_validated: usize,
    pub ai_rejection_reasons: Vec<AiRejectionAudit>,
    pub merge_decisions: Vec<MergeDecisionAudit>,
    pub json_repair_attempted: bool,
    pub json_repair_success: bool,
    /// Per-splitter contribution log — explains the "AI generated N, validated
    /// N+k" delta so the auditor knows which post-AI splitter added rows and
    /// why. Populated by any pass that adds rows to the final items list.
    #[serde(default)]
    pub row_splitter_log: Vec<SplitterContributionAudit>,
    /// Rows that post-AI `validate_schema` flagged. The row is still emitted —
    /// this log just explains WHICH rows will show in the review widget.
    #[serde(default)]
    pub post_ai_schema_violations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SplitterContributionAudit {
    /// Module that emitted the extra rows — e.g. "opening_subtractor",
    /// "derive_secondary_quantities", "steel_profile_extractor".
    pub source: String,
    pub added_rows: usize,
    /// SEK groups of the emitted rows.
    pub sek_groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiRejectionAudit {
    pub item_description: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MergeDecisionAudit {
    pub sek_group: String,
    pub winner: String,
    pub reason: String,
    pub rule_items: usize,
    pub ai_items: usize,
}

// ─── Phase 6: Final Report ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReportAudit {
    pub total_items: usize,
    pub total_sections: usize,
    pub subtotal_eur: f64,
    pub vat_eur: f64,
    pub total_with_vat_eur: f64,
    pub drm_artifacts_recorded: usize,
    pub reports_generated: Vec<String>,
}

// ─── Shared types ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhaseTimingEntry {
    pub phase: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditWarning {
    pub phase: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditError {
    pub phase: String,
    pub message: String,
}

// ─── User-mode summary ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPhaseSummary {
    pub phase_number: u8,
    pub phase_name: String,
    pub duration_ms: u64,
    pub summary: String,
    pub highlights: Vec<String>,
}

/// Truncate a string to max bytes on a char boundary.
pub fn truncate_for_audit(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...[truncated]", &s[..end])
}
