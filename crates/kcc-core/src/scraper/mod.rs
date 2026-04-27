//! BrightData-powered web scraping for Bulgarian construction prices.
//!
//! Canonical currency is **€ (EUR)**. The platform stores and quotes EUR
//! exclusively; the legacy BGN dual-storage was retired in migration 025.

pub mod brightdata;
pub mod normalizer;
pub mod parsers;
pub mod price_utils;
pub mod sek_mapper;

use serde::{Deserialize, Serialize};

/// A single scraped price entry. Prices are in € (EUR).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedPrice {
    pub source_site: String,
    pub source_url: String,
    pub description_bg: String,
    pub unit: String,
    pub price_min_eur: Option<f64>,
    pub price_max_eur: Option<f64>,
    /// Raw price text as scraped, for audit.
    pub raw_price_text: Option<String>,
    pub category: Option<String>,
    /// Extraction confidence (0.0 – 1.0).
    pub extraction_confidence: f64,
}

impl ScrapedPrice {
    pub fn from_eur(
        site: &str,
        url: &str,
        desc: &str,
        unit: &str,
        min_eur: Option<f64>,
        max_eur: Option<f64>,
        raw_text: Option<&str>,
        category: Option<&str>,
        confidence: f64,
    ) -> Self {
        Self {
            source_site: site.to_string(),
            source_url: url.to_string(),
            description_bg: desc.to_string(),
            unit: unit.to_string(),
            price_min_eur: min_eur,
            price_max_eur: max_eur,
            raw_price_text: raw_text.map(|s| s.to_string()),
            category: category.map(|s| s.to_string()),
            extraction_confidence: confidence,
        }
    }

    pub fn price_avg_eur(&self) -> f64 {
        match (self.price_min_eur, self.price_max_eur) {
            (Some(min), Some(max)) => (min + max) / 2.0,
            (Some(v), None) | (None, Some(v)) => v,
            (None, None) => 0.0,
        }
    }
}
