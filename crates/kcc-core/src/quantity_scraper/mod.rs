//! Quantity-norm web scraping for Bulgarian construction sources.
//!
//! Mirrors `crate::scraper` (prices) in shape:
//!   - `ScrapedNorm` is to norms what `ScrapedPrice` is to prices.
//!   - `parsers/` holds one file per source, each implementing [`NormParser`].
//!   - Fetching reuses [`crate::scraper::brightdata::BrightDataClient`] — no duplicate client.
//!   - SEK-group mapping reuses [`crate::scraper::sek_mapper`] — no duplicate rules.
//!
//! A "quantity norm" answers: *"to execute 1 unit of work X, you consume A hours
//! of labor, B kg of material M1, C bags of material M2, and D machine-hours of
//! equipment E."*  The AI KSS pipeline uses these as anchors so it does not
//! hallucinate dosages for Образец 9.1 positions.

pub mod parsers;

use serde::{Deserialize, Serialize};

/// One material line inside a norm, e.g. `{name: "цимент", qty: 8.0, unit: "кг"}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormMaterial {
    pub name: String,
    pub qty: f64,
    pub unit: String,
}

/// One machinery line, e.g. `{name: "миксер", qty: 0.05, unit: "маш.-ч"}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormMachinery {
    pub name: String,
    pub qty: f64,
    pub unit: String,
}

/// A single scraped norm, pre-persistence. All fields are in канонична Bulgarian
/// form — the pipeline does SEK mapping + confidence tagging after parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedNorm {
    pub source_site: String,
    pub source_url: String,
    /// Bulgarian description ("Зидария от газобетонни блокове 25см").
    pub description_bg: String,
    /// Work unit ("m²", "m³", "м", "бр.", "кг", "тон").
    pub work_unit: String,
    /// Hours of qualified (I-степен) labor per unit of work.
    pub labor_qualified_h: f64,
    /// Hours of helper (II-степен) labor per unit of work.
    pub labor_helper_h: f64,
    /// Trade (зидар, армировчик, мазач, бояджия, …) if known.
    pub labor_trade: Option<String>,
    pub materials: Vec<NormMaterial>,
    pub machinery: Vec<NormMachinery>,
    /// Hint from the category URL so `sek_mapper` can fall back when keywords
    /// in the description don't match (e.g. `"СЕК05"`).
    pub sek_group_hint: Option<String>,
    /// Raw source text snippet for audit, max ~500 chars.
    pub raw_snippet: Option<String>,
    /// Parser self-reported confidence 0.0 – 1.0.
    pub extraction_confidence: f64,
}

impl ScrapedNorm {
    pub fn new(site: &str, url: &str, description: &str, unit: &str) -> Self {
        Self {
            source_site: site.to_string(),
            source_url: url.to_string(),
            description_bg: description.to_string(),
            work_unit: unit.to_string(),
            labor_qualified_h: 0.0,
            labor_helper_h: 0.0,
            labor_trade: None,
            materials: Vec::new(),
            machinery: Vec::new(),
            sek_group_hint: None,
            raw_snippet: None,
            extraction_confidence: 0.7,
        }
    }

    pub fn total_labor_h(&self) -> f64 {
        self.labor_qualified_h + self.labor_helper_h
    }
}
