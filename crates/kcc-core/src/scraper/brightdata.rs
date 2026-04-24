//! BrightData Web Unlocker API client.
//!
//! Fetches raw HTML from any URL using BrightData's proxy/unlocker infrastructure.
//! Supports `x-unblock-expect` to wait for specific CSS selectors before returning.
//!
//! Correct `x-unblock-expect` format per BrightData docs:
//!   "headers": { "x-unblock-expect": "{\"element\": \".css-selector\"}" }
//! The value is a JSON *string* containing a JSON object with "element" or "text" key.

use reqwest::Client;
use std::time::Duration;

pub struct BrightDataClient {
    api_key: String,
    zone: String,
    http: Client,
}

#[derive(Debug, thiserror::Error)]
pub enum BrightDataError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("BrightData API returned status {status}: {body}")]
    ApiError { status: u16, body: String },
}

/// Debug info returned alongside the HTML for diagnostics.
#[derive(Debug)]
pub struct FetchDebugInfo {
    pub status: u16,
    pub elapsed_ms: u128,
    pub html_len: usize,
    pub html_preview: String,
}

impl BrightDataClient {
    pub fn new(api_key: String, zone: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to build HTTP client");
        Self { api_key, zone, http }
    }

    /// Fetch raw HTML from a URL via BrightData Web Unlocker.
    pub async fn fetch_html(&self, url: &str) -> Result<String, BrightDataError> {
        let (html, _debug) = self.fetch_html_debug(url, None).await?;
        Ok(html)
    }

    /// Fetch raw HTML, waiting for a CSS selector to appear before returning.
    pub async fn fetch_html_with_expect(
        &self,
        url: &str,
        expect_selector: Option<&str>,
    ) -> Result<String, BrightDataError> {
        let (html, _debug) = self.fetch_html_debug(url, expect_selector).await?;
        Ok(html)
    }

    /// Fetch with full debug info returned for diagnostics.
    pub async fn fetch_html_debug(
        &self,
        url: &str,
        expect_selector: Option<&str>,
    ) -> Result<(String, FetchDebugInfo), BrightDataError> {
        let mut body = serde_json::json!({
            "zone": self.zone,
            "url": url,
            "format": "raw"
        });

        // x-unblock-expect requires "manual expect" enabled on the BrightData zone.
        // Only attach if BRIGHTDATA_MANUAL_EXPECT_ENABLED=true in env.
        if let Some(selector) = expect_selector {
            let manual_expect = std::env::var("BRIGHTDATA_MANUAL_EXPECT_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);
            if manual_expect {
                let expect_value = serde_json::json!({"element": selector}).to_string();
                body["headers"] = serde_json::json!({
                    "x-unblock-expect": expect_value
                });
                tracing::debug!(url, selector, "Using manual expect (zone-enabled)");
            }
        }

        tracing::debug!(url, ?expect_selector, body = %body, "BrightData request");

        let start = std::time::Instant::now();

        let resp = self
            .http
            .post("https://api.brightdata.com/request")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let elapsed_ms = start.elapsed().as_millis();
        let status = resp.status().as_u16();

        if status != 200 {
            let resp_body = resp.text().await.unwrap_or_default();
            tracing::warn!(url, status, elapsed_ms, preview = &resp_body[..resp_body.len().min(500)], "BrightData non-200");
            return Err(BrightDataError::ApiError {
                status,
                body: resp_body,
            });
        }

        let html = resp.text().await?;
        let html_len = html.len();
        // Safe UTF-8 truncation — find a char boundary near 1500 bytes
        let preview_len = {
            let target = html_len.min(1500);
            let mut end = target;
            while end > 0 && !html.is_char_boundary(end) {
                end -= 1;
            }
            end
        };
        let html_preview = html[..preview_len].to_string();

        tracing::info!(url, status, elapsed_ms, html_len, "BrightData response");
        tracing::debug!(url, preview = %html_preview, "BrightData HTML preview");

        let debug = FetchDebugInfo {
            status,
            elapsed_ms,
            html_len,
            html_preview,
        };

        Ok((html, debug))
    }
}
