//! BrightData-powered web scraping for Bulgarian construction prices.
//!
//! Canonical currency is **лв (BGN)**. EUR is a derived convenience field.

pub mod brightdata;
pub mod normalizer;
pub mod parsers;
pub mod price_utils;
pub mod sek_mapper;

use serde::{Deserialize, Serialize};

/// Fixed EUR → BGN rate (Bulgaria is pegged to the euro via currency board).
pub const EUR_TO_BGN: f64 = 1.95583;
pub const BGN_TO_EUR: f64 = 1.0 / 1.95583;

/// A single scraped price entry. Canonical prices are in лв (BGN).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedPrice {
    pub source_site: String,
    pub source_url: String,
    pub description_bg: String,
    pub unit: String,
    /// Canonical prices in лв (BGN)
    pub price_min_lv: Option<f64>,
    pub price_max_lv: Option<f64>,
    /// Derived prices in EUR
    pub price_min_eur: Option<f64>,
    pub price_max_eur: Option<f64>,
    /// Source currency as detected: "lv" or "EUR"
    pub currency: String,
    /// Raw price text as scraped, for audit
    pub raw_price_text: Option<String>,
    pub category: Option<String>,
    /// How confident we are in this extraction (0.0 - 1.0)
    pub extraction_confidence: f64,
}

impl ScrapedPrice {
    /// Create from EUR source prices, deriving lv.
    pub fn from_eur(
        site: &str, url: &str, desc: &str, unit: &str,
        min_eur: Option<f64>, max_eur: Option<f64>,
        raw_text: Option<&str>, category: Option<&str>,
        confidence: f64,
    ) -> Self {
        Self {
            source_site: site.to_string(),
            source_url: url.to_string(),
            description_bg: desc.to_string(),
            unit: unit.to_string(),
            price_min_lv: min_eur.map(|v| v * EUR_TO_BGN),
            price_max_lv: max_eur.map(|v| v * EUR_TO_BGN),
            price_min_eur: min_eur,
            price_max_eur: max_eur,
            currency: "EUR".to_string(),
            raw_price_text: raw_text.map(|s| s.to_string()),
            category: category.map(|s| s.to_string()),
            extraction_confidence: confidence,
        }
    }

    /// Create from лв source prices, deriving EUR.
    pub fn from_lv(
        site: &str, url: &str, desc: &str, unit: &str,
        min_lv: Option<f64>, max_lv: Option<f64>,
        raw_text: Option<&str>, category: Option<&str>,
        confidence: f64,
    ) -> Self {
        Self {
            source_site: site.to_string(),
            source_url: url.to_string(),
            description_bg: desc.to_string(),
            unit: unit.to_string(),
            price_min_lv: min_lv,
            price_max_lv: max_lv,
            price_min_eur: min_lv.map(|v| v * BGN_TO_EUR),
            price_max_eur: max_lv.map(|v| v * BGN_TO_EUR),
            currency: "lv".to_string(),
            raw_price_text: raw_text.map(|s| s.to_string()),
            category: category.map(|s| s.to_string()),
            extraction_confidence: confidence,
        }
    }

    pub fn price_avg_lv(&self) -> f64 {
        match (self.price_min_lv, self.price_max_lv) {
            (Some(min), Some(max)) => (min + max) / 2.0,
            (Some(v), None) | (None, Some(v)) => v,
            (None, None) => 0.0,
        }
    }
}
