use serde::{Deserialize, Serialize};

/// A single item in a user-uploaded price list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceListItem {
    pub sek_code: String,
    pub description: String,
    pub unit: String,
    pub labor_price: f64,
    pub material_price: f64,
    pub mechanization_price: f64,
    pub overhead_price: f64,
}

impl PriceListItem {
    /// Total unit price (all cost components).
    pub fn total_unit_price(&self) -> f64 {
        self.labor_price + self.material_price + self.mechanization_price + self.overhead_price
    }
}

/// User-uploaded price list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceList {
    pub items: Vec<PriceListItem>,
}

impl PriceList {
    pub fn empty() -> Self {
        Self { items: Vec::new() }
    }

    /// Parse a price list from CSV content.
    /// Expected columns: sek_code, description, unit, labor, material, mechanization, overhead
    pub fn from_csv(data: &[u8]) -> Result<Self, String> {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(data);

        let mut items = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| format!("CSV parse error: {e}"))?;
            if record.len() < 4 {
                continue;
            }
            let sek_code = record.get(0).unwrap_or("").to_string();
            if sek_code.is_empty() {
                continue;
            }
            items.push(PriceListItem {
                sek_code,
                description: record.get(1).unwrap_or("").to_string(),
                unit: record.get(2).unwrap_or("pcs").to_string(),
                labor_price: record.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                material_price: record.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                mechanization_price: record.get(5).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                overhead_price: record.get(6).and_then(|s| s.parse().ok()).unwrap_or(0.0),
            });
        }
        Ok(PriceList { items })
    }

    /// Look up a price list item by SEK code.
    /// Tries exact match first, then prefix match (e.g., "СЕК05" finds "СЕК05.002").
    pub fn find_by_code(&self, code: &str) -> Option<&PriceListItem> {
        // Exact match
        if let Some(item) = self.items.iter().find(|i| i.sek_code == code) {
            return Some(item);
        }
        // Prefix match: "СЕК05" matches "СЕК05.002", or "СЕК05.002" matches "СЕК05"
        self.items
            .iter()
            .filter(|i| i.sek_code.starts_with(code) || code.starts_with(&i.sek_code))
            .min_by(|a, b| a.sek_code.cmp(&b.sek_code))
    }
}

/// How a specific QuantityItem's numeric value was arrived at. This drives the
/// `geometry_confidence` field and ultimately the AI prompt's "trust vs flag"
/// decision — see `ai_kss_pipeline::run_generation_phase` for the rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionMethod {
    /// Closed polyline area via the Shoelace formula. Most trustworthy.
    PolylineShoelace,
    /// Length summed over open polylines / lines. Trustworthy for linear takeoffs.
    LinearPolyline,
    /// Count of real INSERT block references (doors, windows, fixtures).
    BlockInstanceCount,
    /// Length × configured wall height (default 2.8 m). Medium trust —
    /// the length is real but the height is assumed.
    WallAreaFromCenterline,
    /// Length × wall height × configured thickness. Lower trust — two
    /// assumptions stacked.
    WallVolumeFromCenterline,
    /// Parsed from a room-area TEXT annotation (e.g. "16 m²").
    TextAnnotation,
    /// Derived from a primary quantity (e.g. plaster = wall_area × 2).
    DerivedFromPrimary,
    /// No measurement — a hardcoded fallback. Always flag for review.
    AssumedDefault,
    /// Produced by the LLM with no geometric source. Must be reviewed.
    AiInferred,
}

impl ExtractionMethod {
    /// Per-variant base confidence 0.0–1.0. The AI prompt uses thresholds at
    /// 0.8 (trust) and 0.6 (flag) — see `ai_kss_pipeline.rs`.
    pub fn base_confidence(self) -> f64 {
        match self {
            ExtractionMethod::PolylineShoelace         => 0.90,
            ExtractionMethod::BlockInstanceCount       => 0.95,
            ExtractionMethod::LinearPolyline           => 0.85,
            ExtractionMethod::TextAnnotation           => 0.80,
            ExtractionMethod::WallAreaFromCenterline   => 0.75,
            ExtractionMethod::WallVolumeFromCenterline => 0.65,
            ExtractionMethod::DerivedFromPrimary       => 0.70,
            ExtractionMethod::AiInferred               => 0.50,
            ExtractionMethod::AssumedDefault           => 0.40,
        }
    }

    /// True when the row must be routed to the human-review widget, not the
    /// final totals. Matches the < 0.6 threshold in the Opus prompt.
    pub fn needs_review(self) -> bool {
        self.base_confidence() < 0.60
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ExtractionMethod::PolylineShoelace         => "polyline_shoelace",
            ExtractionMethod::BlockInstanceCount       => "block_instance_count",
            ExtractionMethod::LinearPolyline           => "linear_polyline",
            ExtractionMethod::TextAnnotation           => "text_annotation",
            ExtractionMethod::WallAreaFromCenterline   => "wall_area_from_centerline",
            ExtractionMethod::WallVolumeFromCenterline => "wall_volume_from_centerline",
            ExtractionMethod::DerivedFromPrimary       => "derived_from_primary",
            ExtractionMethod::AiInferred               => "ai_inferred",
            ExtractionMethod::AssumedDefault           => "assumed_default",
        }
    }
}

/// A raw extracted quantity before SEK mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityItem {
    pub category: String,
    pub description: String,
    pub unit: String,
    pub quantity: f64,
    pub suggested_sek_code: String,
    /// Which DXF entity / feature backs this row. None for derived rows.
    #[serde(default)]
    pub source_entity_id: Option<String>,
    /// Layer name the row came from (e.g. `"A-WALL"`).
    #[serde(default)]
    pub source_layer: Option<String>,
    /// (x, y) in drawing coordinates — useful for viewer highlight.
    #[serde(default)]
    pub centroid: Option<(f64, f64)>,
    /// How this quantity's number was arrived at.
    #[serde(default = "default_extraction_method")]
    pub extraction_method: ExtractionMethod,
    /// Cached `extraction_method.base_confidence()` so downstream consumers
    /// (AI prompt, DB INSERT) don't have to recompute it.
    #[serde(default = "default_geom_confidence")]
    pub geometry_confidence: f64,
    /// True when `geometry_confidence < 0.6` or a schema-contract validator
    /// flagged the row. The Opus prompt is told NOT to change a row's quantity
    /// when `needs_review = true` — it only adds a `reasoning` string.
    #[serde(default)]
    pub needs_review: bool,
}

fn default_extraction_method() -> ExtractionMethod { ExtractionMethod::AssumedDefault }
fn default_geom_confidence() -> f64 { 0.5 }

impl Default for ExtractionMethod {
    fn default() -> Self { ExtractionMethod::AssumedDefault }
}

impl Default for QuantityItem {
    fn default() -> Self {
        Self {
            category: String::new(),
            description: String::new(),
            unit: String::new(),
            quantity: 0.0,
            suggested_sek_code: String::new(),
            source_entity_id: None,
            source_layer: None,
            centroid: None,
            extraction_method: ExtractionMethod::default(),
            geometry_confidence: default_geom_confidence(),
            needs_review: true,
        }
    }
}

impl QuantityItem {
    /// Convenience constructor that keeps the traceability fields consistent.
    pub fn new(
        category: impl Into<String>,
        description: impl Into<String>,
        unit: impl Into<String>,
        quantity: f64,
        suggested_sek_code: impl Into<String>,
        method: ExtractionMethod,
    ) -> Self {
        let conf = method.base_confidence();
        Self {
            category: category.into(),
            description: description.into(),
            unit: unit.into(),
            quantity,
            suggested_sek_code: suggested_sek_code.into(),
            source_entity_id: None,
            source_layer: None,
            centroid: None,
            extraction_method: method,
            geometry_confidence: conf,
            needs_review: method.needs_review(),
        }
    }

    pub fn with_source(mut self, entity_id: impl Into<String>, layer: impl Into<String>) -> Self {
        self.source_entity_id = Some(entity_id.into());
        self.source_layer = Some(layer.into());
        self
    }

    pub fn with_centroid(mut self, x: f64, y: f64) -> Self {
        self.centroid = Some((x, y));
        self
    }
}

/// A single line item in the final KSS report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KssLineItem {
    pub item_no: usize,
    pub sek_code: String,
    pub description: String,
    pub unit: String,
    pub quantity: f64,
    pub labor_price: f64,
    pub material_price: f64,
    pub mechanization_price: f64,
    pub overhead_price: f64,
    pub total_price: f64,
    /// Per-item confidence: 0.9=geometry-backed, 0.7=estimated, 0.5=indirect, 0.3=hallucinated
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    /// AI reasoning for this item (how quantity/price was determined)
    #[serde(default)]
    pub reasoning: String,
    /// Provenance: "rule_based", "ai_generated", "user_added", "user_accepted"
    #[serde(default = "default_provenance")]
    pub provenance: String,
    /// DXF handle / internal entity id this row came from. None for AI-only rows.
    #[serde(default)]
    pub source_entity_id: Option<String>,
    /// Layer name this row came from.
    #[serde(default)]
    pub source_layer: Option<String>,
    /// Drawing-coordinate centroid for "highlight-in-viewer" UX.
    #[serde(default)]
    pub centroid_x: Option<f64>,
    #[serde(default)]
    pub centroid_y: Option<f64>,
    /// One of `ExtractionMethod::as_str()` values.
    #[serde(default)]
    pub extraction_method: Option<String>,
    /// Separate from `confidence`: the geometry-side confidence *before* AI touched it.
    /// The AI must respect this — see `ai_kss_pipeline.rs` prompt rules.
    #[serde(default = "default_geom_confidence")]
    pub geometry_confidence: f64,
    /// True when geometry confidence < 0.6 OR schema validators flagged the row.
    /// The frontend routes these into the existing Accept/Deny suggestions widget.
    #[serde(default)]
    pub needs_review: bool,
}

fn default_confidence() -> f64 { 0.8 }
fn default_provenance() -> String { "rule_based".to_string() }

impl Default for KssLineItem {
    fn default() -> Self {
        Self {
            item_no: 0,
            sek_code: String::new(),
            description: String::new(),
            unit: String::new(),
            quantity: 0.0,
            labor_price: 0.0,
            material_price: 0.0,
            mechanization_price: 0.0,
            overhead_price: 0.0,
            total_price: 0.0,
            confidence: default_confidence(),
            reasoning: String::new(),
            provenance: default_provenance(),
            source_entity_id: None,
            source_layer: None,
            centroid_x: None,
            centroid_y: None,
            extraction_method: None,
            geometry_confidence: default_geom_confidence(),
            needs_review: false,
        }
    }
}

/// Summary totals for the KSS report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KssTotals {
    pub labor: f64,
    pub material: f64,
    pub mechanization: f64,
    pub overhead: f64,
    pub grand_total: f64,
}

/// Complete KSS report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KssReport {
    pub drawing_name: String,
    pub generated_at: String,
    pub items: Vec<KssLineItem>,
    pub totals: KssTotals,
}

impl KssReport {
    pub fn compute_totals(items: &[KssLineItem]) -> KssTotals {
        let mut totals = KssTotals {
            labor: 0.0,
            material: 0.0,
            mechanization: 0.0,
            overhead: 0.0,
            grand_total: 0.0,
        };
        for item in items {
            totals.labor += item.labor_price * item.quantity;
            totals.material += item.material_price * item.quantity;
            totals.mechanization += item.mechanization_price * item.quantity;
            totals.overhead += item.overhead_price * item.quantity;
            totals.grand_total += item.total_price;
        }
        totals
    }
}

// === Sectioned KSS types (Образец 9.1 format) ===

/// A section in the Bulgarian KSS report (Roman numeral grouping).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KssSection {
    pub number: String,
    pub title_bg: String,
    pub sek_group: String,
    pub items: Vec<KssLineItem>,
    pub section_total_bgn: f64,
}

/// Overhead / markup percentages that convert "СМР subtotal" into
/// "ОБЩО ЗА ОБЕКТА" (pre-VAT). These MUST be persisted alongside the totals
/// — previously the UI computed them on the fly from the user's
/// `pricing_defaults` while the DB only kept the subtotal + VAT, so the two
/// could drift silently.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct KssOverheads {
    pub contingency_pct: f64,
    pub delivery_storage_pct: f64,
    pub profit_pct: f64,
}

/// Fully-broken-down cost ladder — one canonical struct both the UI and the
/// audit trail read from, byte for byte.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct KssCostLadder {
    pub smr_subtotal: f64,
    pub contingency: f64,
    pub delivery_storage: f64,
    pub profit: f64,
    pub pre_vat_total: f64,
    pub vat: f64,
    pub final_total: f64,
}

impl KssCostLadder {
    pub fn compute(smr_subtotal: f64, overheads: KssOverheads, vat_rate: f64) -> Self {
        let contingency      = smr_subtotal * overheads.contingency_pct / 100.0;
        let delivery_storage = smr_subtotal * overheads.delivery_storage_pct / 100.0;
        let profit           = smr_subtotal * overheads.profit_pct / 100.0;
        let pre_vat_total    = smr_subtotal + contingency + delivery_storage + profit;
        let vat              = pre_vat_total * vat_rate;
        let final_total      = pre_vat_total + vat;
        Self {
            smr_subtotal, contingency, delivery_storage, profit,
            pre_vat_total, vat, final_total,
        }
    }
}

/// Complete sectioned KSS report per Образец 9.1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionedKssReport {
    pub project_name: String,
    pub generated_at: String,
    pub sections: Vec<KssSection>,
    pub subtotal_bgn: f64,
    pub vat_rate: f64,
    pub vat_bgn: f64,
    pub total_with_vat_bgn: f64,
    /// Full cost ladder: subtotal → markups → pre-VAT → VAT → final. Every UI
    /// screen and audit report renders from this struct so "ОБЩО ЗА ОБЕКТА"
    /// and "total (incl. VAT)" can never contradict each other again.
    #[serde(default)]
    pub cost_ladder: KssCostLadder,
    #[serde(default)]
    pub overheads: KssOverheads,
}

impl SectionedKssReport {
    /// Legacy entry point — no markups, used by tests and a few callers that
    /// don't know the user's overhead defaults. Delegates to `from_items_full`.
    pub fn from_items(
        project_name: &str,
        generated_at: &str,
        items: Vec<KssLineItem>,
        vat_rate: f64,
    ) -> Self {
        Self::from_items_full(
            project_name, generated_at, items, vat_rate,
            KssOverheads::default(),
        )
    }

    /// Build a sectioned report with the full cost ladder baked in.
    /// Fills both the legacy `subtotal_bgn / vat_bgn / total_with_vat_bgn`
    /// fields AND the new `cost_ladder` so both old and new readers stay
    /// consistent.
    pub fn from_items_full(
        project_name: &str,
        generated_at: &str,
        items: Vec<KssLineItem>,
        vat_rate: f64,
        overheads: KssOverheads,
    ) -> Self {
        use crate::kss::sections::{KSS_SECTIONS, section_index};
        use std::collections::HashMap;

        // Group items by SEK group prefix (first 5 chars of sek_code, e.g., "СЕК05")
        let mut groups: HashMap<String, Vec<KssLineItem>> = HashMap::new();
        for item in items {
            let group = extract_sek_group(&item.sek_code);
            groups.entry(group).or_default().push(item);
        }

        // Build sections in canonical order
        let mut sections = Vec::new();
        for def in KSS_SECTIONS {
            if let Some(mut group_items) = groups.remove(def.sek_group) {
                // Re-number items within section
                for (i, item) in group_items.iter_mut().enumerate() {
                    item.item_no = i + 1;
                }
                let section_total: f64 = group_items.iter().map(|i| i.total_price).sum();
                sections.push(KssSection {
                    number: def.number.to_string(),
                    title_bg: def.title_bg.to_string(),
                    sek_group: def.sek_group.to_string(),
                    items: group_items,
                    section_total_bgn: section_total,
                });
            }
        }

        // Any remaining items with unknown SEK groups go in a "misc" section
        for (group, mut items) in groups {
            for (i, item) in items.iter_mut().enumerate() {
                item.item_no = i + 1;
            }
            let section_total: f64 = items.iter().map(|i| i.total_price).sum();
            sections.push(KssSection {
                number: "—".to_string(),
                title_bg: format!("ДРУГИ РАБОТИ ({})", group),
                sek_group: group,
                items,
                section_total_bgn: section_total,
            });
        }

        // Sort sections by canonical order
        sections.sort_by_key(|s| section_index(&s.number));

        let subtotal: f64 = sections.iter().map(|s| s.section_total_bgn).sum();
        let ladder = KssCostLadder::compute(subtotal, overheads, vat_rate);

        SectionedKssReport {
            project_name: project_name.to_string(),
            generated_at: generated_at.to_string(),
            sections,
            // Legacy fields point at the ladder so any consumer still reading
            // the old names (`total_with_vat_bgn`) gets the *same* number the
            // new consumers see via `cost_ladder.final_total`.
            subtotal_bgn: ladder.smr_subtotal,
            vat_rate,
            vat_bgn: ladder.vat,
            total_with_vat_bgn: ladder.final_total,
            cost_ladder: ladder,
            overheads,
        }
    }
}

/// Extract SEK group from a full SEK code (e.g., "СЕК05.002" → "СЕК05").
fn extract_sek_group(code: &str) -> String {
    if let Some(dot_pos) = code.find('.') {
        code[..dot_pos].to_string()
    } else {
        // Try to extract "СЕКxx" prefix
        let trimmed = code.trim();
        if trimmed.starts_with("СЕК") && trimmed.len() >= 7 {
            // "СЕК" is 6 bytes in UTF-8, plus 2 digit chars
            let prefix_end = "СЕК".len() + 2;
            if prefix_end <= trimmed.len() {
                return trimmed[..prefix_end].to_string();
            }
        }
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_sum_invariant_holds() {
        // For any overhead + VAT combination, the ladder components must
        // reconstruct final_total exactly. Regression guard for the
        // "65,685 UI vs 49,761 audit" class of bug.
        let cases = [
            (0.0, 0.0, 0.0, 0.0),
            (10.0, 12.0, 10.0, 0.20),
            (5.5, 8.25, 22.0, 0.20),
            (0.0, 0.0, 0.0, 0.00),
        ];
        for (c, ds, p, v) in cases {
            let oh = KssOverheads { contingency_pct: c, delivery_storage_pct: ds, profit_pct: p };
            let ladder = KssCostLadder::compute(100_000.0, oh, v);
            let reconstructed = ladder.smr_subtotal + ladder.contingency
                + ladder.delivery_storage + ladder.profit + ladder.vat;
            let diff = (ladder.final_total - reconstructed).abs();
            assert!(diff < 0.001, "ladder mismatch {c}/{ds}/{p}/{v}: diff={diff}");
        }
    }

    #[test]
    fn sectioned_report_ladder_matches_legacy_fields() {
        let items = vec![
            KssLineItem { item_no: 1, sek_code: "СЕК05.002".into(), total_price: 1000.0, ..Default::default() },
            KssLineItem { item_no: 2, sek_code: "СЕК10.011".into(), total_price: 500.0,  ..Default::default() },
        ];
        let oh = KssOverheads { contingency_pct: 10.0, delivery_storage_pct: 12.0, profit_pct: 10.0 };
        let r = SectionedKssReport::from_items_full("test", "now", items, 0.20, oh);
        // Legacy subtotal_bgn aligns with the new ladder.
        assert!((r.subtotal_bgn - r.cost_ladder.smr_subtotal).abs() < 0.001);
        // Legacy total_with_vat_bgn also matches the new ladder's final.
        assert!((r.total_with_vat_bgn - r.cost_ladder.final_total).abs() < 0.001);
        // VAT is computed on the pre-VAT amount (incl. markups), not the raw SMR.
        assert!(r.cost_ladder.pre_vat_total > r.cost_ladder.smr_subtotal);
    }

    #[test]
    fn test_price_list_from_csv() {
        let csv = b"sek_code,description,unit,labor,material,mechanization,overhead\n\
                     14.001,Steel beams IPE,kg,0.50,2.80,0.30,0.15\n\
                     14.015,Bolt assembly M16,pcs,1.20,3.50,0.00,0.10";
        let pl = PriceList::from_csv(csv).unwrap();
        assert_eq!(pl.items.len(), 2);
        assert_eq!(pl.items[0].sek_code, "14.001");
        assert!((pl.items[0].total_unit_price() - 3.75).abs() < 0.01);
    }

    #[test]
    fn test_find_by_code() {
        let pl = PriceList {
            items: vec![PriceListItem {
                sek_code: "14.001".into(),
                description: "test".into(),
                unit: "kg".into(),
                labor_price: 1.0,
                material_price: 2.0,
                mechanization_price: 0.0,
                overhead_price: 0.0,
            }],
        };
        assert!(pl.find_by_code("14.001").is_some());
        assert!(pl.find_by_code("99.999").is_none());
    }
}
