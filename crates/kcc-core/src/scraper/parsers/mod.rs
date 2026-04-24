//! Site-specific HTML parsers for Bulgarian construction price sources.

pub mod daibau;
pub mod mr_bricolage;

use crate::scraper::ScrapedPrice;

/// Result of parsing a single page, with diagnostics.
#[derive(Debug)]
pub struct ParseResult {
    pub prices: Vec<ScrapedPrice>,
    pub strategy_used: &'static str,
    pub candidates_before_filter: usize,
    pub candidates_after_filter: usize,
    pub diagnostics: Vec<(&'static str, usize)>,
}

impl ParseResult {
    pub fn empty() -> Self {
        Self {
            prices: Vec::new(),
            strategy_used: "none",
            candidates_before_filter: 0,
            candidates_after_filter: 0,
            diagnostics: Vec::new(),
        }
    }
}

/// Trait implemented by each site-specific parser.
pub trait PriceParser: Send + Sync {
    fn site_name(&self) -> &str;

    /// Parse an HTML page and extract price items with diagnostics.
    fn parse_page(&self, html: &str, url: &str) -> ParseResult;

    fn category_urls(&self) -> Vec<CategoryUrl>;

    fn expect_selector(&self) -> Option<&str> {
        None
    }
}

/// A URL to scrape with its associated SEK group for mapping.
#[derive(Debug, Clone)]
pub struct CategoryUrl {
    pub url: String,
    pub sek_group_hint: String,
    pub category_name: String,
}

/// Get all built-in parsers.
pub fn builtin_parsers() -> Vec<Box<dyn PriceParser>> {
    vec![
        Box::new(daibau::DaibauParser),
        Box::new(mr_bricolage::MrBricolageParser),
    ]
}
