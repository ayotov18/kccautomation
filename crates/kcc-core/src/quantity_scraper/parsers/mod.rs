//! Site-specific parsers for Bulgarian quantity-norm sources.
//!
//! Mirrors `crate::scraper::parsers::mod` in shape.

pub mod baumit;
pub mod ceresit;
pub mod fibran;
pub mod generic_tds;
pub mod globus;
pub mod knauf;
pub mod mapei;
pub mod procurement_xls;
pub mod sika;
pub mod weber;
pub mod wienerberger;
pub mod ytong;

use crate::quantity_scraper::ScrapedNorm;

/// Result of parsing a single page, with diagnostics.
#[derive(Debug)]
pub struct NormParseResult {
    pub norms: Vec<ScrapedNorm>,
    pub strategy_used: &'static str,
    pub candidates_before_filter: usize,
    pub candidates_after_filter: usize,
    pub diagnostics: Vec<(&'static str, usize)>,
}

impl NormParseResult {
    pub fn empty() -> Self {
        Self {
            norms: Vec::new(),
            strategy_used: "none",
            candidates_before_filter: 0,
            candidates_after_filter: 0,
            diagnostics: Vec::new(),
        }
    }
}

/// Trait implemented by each site-specific norm parser.
pub trait NormParser: Send + Sync {
    fn site_name(&self) -> &str;

    /// Identifier matching `quantity_sources.parser_template`.
    /// The worker uses this to decide whether this parser runs for a given source.
    fn template_key(&self) -> &str;

    /// Parse a fetched page (HTML body or extracted PDF text) + emit norms.
    fn parse_page(&self, content: &str, url: &str) -> NormParseResult;

    /// Seed URLs for this source, with SEK-group hints so the shared
    /// `sek_mapper` has a fallback when keyword matching fails.
    fn category_urls(&self) -> Vec<NormCategoryUrl>;

    /// Hint for BrightData — which selector should be present once the page
    /// is rendered? Returning `None` disables the wait.
    fn expect_selector(&self) -> Option<&str> {
        None
    }
}

/// A URL to scrape with its associated SEK group for mapping.
#[derive(Debug, Clone)]
pub struct NormCategoryUrl {
    pub url: String,
    pub sek_group_hint: String,
    pub category_name: String,
    /// `"html"` (the default), `"pdf"`, or `"xls"`. PDF sources are fetched as
    /// binary and converted to text before handing off to `parse_page`; XLS
    /// sources are fetched as binary and passed as base-64 payload.
    pub fetch_kind: &'static str,
}

impl NormCategoryUrl {
    pub fn html(url: &str, sek_group_hint: &str, category_name: &str) -> Self {
        Self {
            url: url.to_string(),
            sek_group_hint: sek_group_hint.to_string(),
            category_name: category_name.to_string(),
            fetch_kind: "html",
        }
    }

    pub fn pdf(url: &str, sek_group_hint: &str, category_name: &str) -> Self {
        Self {
            url: url.to_string(),
            sek_group_hint: sek_group_hint.to_string(),
            category_name: category_name.to_string(),
            fetch_kind: "pdf",
        }
    }

    pub fn xls(url: &str, sek_group_hint: &str, category_name: &str) -> Self {
        Self {
            url: url.to_string(),
            sek_group_hint: sek_group_hint.to_string(),
            category_name: category_name.to_string(),
            fetch_kind: "xls",
        }
    }
}

/// Registry used by the worker to dispatch URLs to parsers.
pub fn builtin_parsers() -> Vec<Box<dyn NormParser>> {
    vec![
        Box::new(ytong::YtongParser),
        Box::new(wienerberger::WienerbergerParser),
        Box::new(baumit::BaumitParser),
        Box::new(ceresit::CeresitParser),
        Box::new(globus::GlobusParser),
        Box::new(sika::SikaParser),
        Box::new(mapei::MapeiParser),
        Box::new(weber::WeberParser),
        Box::new(fibran::FibranParser),
        Box::new(knauf::KnaufParser),
        Box::new(procurement_xls::ProcurementXlsParser),
    ]
}
