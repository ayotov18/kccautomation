//! AI-powered KSS generation + price research via OpenRouter.
//!
//! Two capabilities:
//! 1. KSS generation — AI produces a complete bill of quantities from drawing data
//! 2. Price research — AI searches the web for Bulgarian construction prices
//!
//! OpenRouter API: https://openrouter.ai/api/v1/chat/completions
//! Web search: `:online` suffix or `plugins: [{ id: "web" }]`

pub mod merger;
pub mod researcher;
pub mod prompt;
pub mod response;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OpenRouter API client for agentic KSS generation.
pub struct OpenRouterClient {
    api_key: String,
    model: String,
    http: Client,
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("OpenRouter API error (status {status}): {body}")]
    ApiError { status: u16, body: String },
    #[error("Insufficient OpenRouter credits — add funds at openrouter.ai")]
    InsufficientCredits,
    #[error("Rate limited by OpenRouter — retry shortly")]
    RateLimited,
    #[error("AI request timed out")]
    Timeout,
    #[error("Failed to parse AI response: {0}")]
    ParseError(String),
    #[error("AI agent not configured (OPENROUTER_API_KEY not set)")]
    NotConfigured,
}

/// Configuration loaded from environment.
pub struct AiConfig {
    pub api_key: String,
    pub model: String,
    pub timeout_secs: u64,
    pub enabled: bool,
}

impl AiConfig {
    pub fn from_env() -> Self {
        let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();
        Self {
            enabled: !api_key.is_empty(),
            api_key,
            model: std::env::var("OPENROUTER_MODEL")
                .unwrap_or_else(|_| "openrouter/auto".to_string()),
            timeout_secs: std::env::var("AI_AGENT_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(90),
        }
    }
}

impl OpenRouterClient {
    pub fn new(config: &AiConfig) -> Result<Self, AiError> {
        if !config.enabled {
            return Err(AiError::NotConfigured);
        }

        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| AiError::Http(e))?;

        Ok(Self {
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            http,
        })
    }

    /// Call OpenRouter to generate a full KSS draft from structured drawing data.
    pub async fn generate_kss(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<AiKssResponse, AiError> {
        let body = serde_json::json!({
            "model": self.model,
            "models": ["openrouter/auto", "anthropic/claude-sonnet-4", "openai/gpt-4o"],
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ],
            "response_format": { "type": "json_object" },
            "temperature": 0.1,
            "max_tokens": 16384,
            "seed": 42
        });

        tracing::info!(model = %self.model, "Calling OpenRouter AI agent");

        let resp = self
            .http
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://kcc-automation.com")
            .header("X-OpenRouter-Title", "KCC Automation")
            .json(&body)
            .send()
            .await?;

        let status = resp.status().as_u16();

        match status {
            200 => {}
            402 => return Err(AiError::InsufficientCredits),
            429 => return Err(AiError::RateLimited),
            408 => return Err(AiError::Timeout),
            _ => {
                let body = resp.text().await.unwrap_or_default();
                return Err(AiError::ApiError { status, body });
            }
        }

        // Read as text first — resp.json() can fail on large/unusual responses
        let raw_body = resp.text().await
            .map_err(|e| AiError::ParseError(format!("Failed to read response body: {e}")))?;

        tracing::debug!(body_len = raw_body.len(), body_preview = &raw_body[..safe_truncate(&raw_body, 500)], "OpenRouter raw response");

        let response_json: serde_json::Value = serde_json::from_str(&raw_body)
            .map_err(|e| {
                tracing::error!(error = %e, body_preview = &raw_body[..raw_body.len().min(1000)], "Failed to parse OpenRouter response JSON");
                AiError::ParseError(format!("Response JSON parse error: {e}"))
            })?;

        // Extract the content from the chat completion response
        let content = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                tracing::error!(response = %response_json, "No content in OpenRouter response");
                AiError::ParseError("No content in response — check model output".into())
            })?;

        // Extract which model was actually used
        let model_used = response_json
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown")
            .to_string();

        tracing::info!(model_used = %model_used, content_len = content.len(), "AI response received");

        // Strip reasoning tags (<think>…</think>) and markdown fences before parsing
        let stripped = strip_tag_block(content, "think");
        let stripped = strip_tag_block(&stripped, "reasoning");
        let stripped = strip_tag_block(&stripped, "reflection");
        let json_str = strip_markdown_fences(&stripped).to_string();

        // Parse the JSON content into our KSS response struct.
        // First try direct parse, then attempt repair if truncated.
        let kss_response: AiKssResponse = match serde_json::from_str(&json_str) {
            Ok(parsed) => parsed,
            Err(first_err) => {
                tracing::warn!(
                    error = %first_err,
                    content_len = json_str.len(),
                    "Direct JSON parse failed — attempting repair for truncated response"
                );

                // For repair, use everything from first '{' onward of the tag-stripped content.
                let full_from_brace = stripped.trim().find('{')
                    .map(|i| &stripped.trim()[i..])
                    .unwrap_or(json_str.as_str());

                let repaired = repair_truncated_json(full_from_brace);
                serde_json::from_str(&repaired).map_err(|repair_err| {
                    let raw_preview: String = content.chars().take(500).collect();
                    tracing::error!(
                        original_error = %first_err,
                        repair_error = %repair_err,
                        raw_preview = %raw_preview,
                        "Failed to parse AI KSS JSON even after repair"
                    );
                    AiError::ParseError(format!(
                        "KSS JSON parse error (repair also failed): original={first_err}, repair={repair_err}; raw preview: {raw_preview}"
                    ))
                })?
            }
        };

        Ok(kss_response)
    }

    /// Generic JSON-completion call. Returns the raw assistant content (after
    /// stripping reasoning tags and markdown fences). Caller is responsible
    /// for parsing and any schema enforcement. Used by callers that want a
    /// custom response shape — the dedicated `generate_kss` should still be
    /// used for the canonical KSS path.
    pub async fn complete_json(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, AiError> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ],
            "response_format": { "type": "json_object" },
            "temperature": 0.1,
            "max_tokens": 4096
        });

        let resp = self
            .http
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://kcc-automation.com")
            .header("X-OpenRouter-Title", "KCC Automation")
            .json(&body)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if status != 200 {
            let body = resp.text().await.unwrap_or_default();
            return Err(match status {
                402 => AiError::InsufficientCredits,
                429 => AiError::RateLimited,
                408 => AiError::Timeout,
                _ => AiError::ApiError { status, body },
            });
        }

        let raw_body = resp
            .text()
            .await
            .map_err(|e| AiError::ParseError(format!("Failed to read response body: {e}")))?;
        let response_json: serde_json::Value = serde_json::from_str(&raw_body)
            .map_err(|e| AiError::ParseError(format!("Response JSON parse error: {e}")))?;
        let content = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| AiError::ParseError("No content in response".into()))?;

        let stripped = strip_tag_block(content, "think");
        let stripped = strip_tag_block(&stripped, "reasoning");
        let stripped = strip_tag_block(&stripped, "reflection");
        let json_str = strip_markdown_fences(&stripped).to_string();
        Ok(json_str)
    }

    /// Call OpenRouter with web search enabled for price research.
    /// Uses `:online` model suffix for real-time web grounding.
    pub async fn search_and_analyze(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        include_domains: Option<&[&str]>,
    ) -> Result<(String, Vec<WebCitation>), AiError> {
        let model_online = format!("{}:online", self.model);

        let mut plugin = serde_json::json!({
            "id": "web",
            "max_results": 10,
            "search_prompt": "Web search for Bulgarian construction pricing data. Cite sources with URLs."
        });

        if let Some(domains) = include_domains {
            plugin["include_domains"] = serde_json::json!(domains);
        }

        let body = serde_json::json!({
            "model": model_online,
            "plugins": [plugin],
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ],
            "response_format": { "type": "json_object" },
            "temperature": 0.1,
            "max_tokens": 4096
        });

        tracing::info!(model = %model_online, "Calling OpenRouter with web search");

        let resp = self
            .http
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://kcc-automation.com")
            .header("X-OpenRouter-Title", "KCC Price Research")
            .json(&body)
            .send()
            .await?;

        let status = resp.status().as_u16();
        match status {
            200 => {}
            402 => return Err(AiError::InsufficientCredits),
            429 => return Err(AiError::RateLimited),
            408 => return Err(AiError::Timeout),
            _ => {
                let body = resp.text().await.unwrap_or_default();
                return Err(AiError::ApiError { status, body });
            }
        }

        let raw_body = resp.text().await
            .map_err(|e| AiError::ParseError(format!("Failed to read response: {e}")))?;

        let response_json: serde_json::Value = serde_json::from_str(&raw_body)
            .map_err(|e| AiError::ParseError(format!("JSON parse error: {e}")))?;

        // Extract content
        let content = response_json
            .get("choices").and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        // Extract web citations from annotations
        let citations: Vec<WebCitation> = response_json
            .get("choices").and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("annotations"))
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter().filter_map(|a| {
                    let cite = a.get("url_citation")?;
                    Some(WebCitation {
                        url: cite.get("url")?.as_str()?.to_string(),
                        title: cite.get("title").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                        content: cite.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string(),
                    })
                }).collect()
            })
            .unwrap_or_default();

        let model_used = response_json.get("model").and_then(|m| m.as_str()).unwrap_or("unknown");
        tracing::info!(
            model_used, content_len = content.len(), citations = citations.len(),
            "Web search response received"
        );

        Ok((content, citations))
    }
}

/// A web citation returned by OpenRouter's web search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebCitation {
    pub url: String,
    pub title: String,
    pub content: String,
}

/// Remove `<tag>...</tag>` blocks (case-insensitive on the ASCII tag name).
/// Used to strip reasoning-model scaffolding like `<think>…</think>`, which appears
/// before the JSON payload in models such as `perplexity/sonar-reasoning-*`.
/// Unclosed blocks have their trailing content discarded — this is intentional,
/// since an unclosed `<think>` means the response was truncated before the real payload.
pub fn strip_tag_block(s: &str, tag: &str) -> String {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    loop {
        let Some(open_start) = find_ascii_ci(rest, &open) else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..open_start]);
        let after_open = &rest[open_start..];
        let Some(gt) = after_open.find('>') else {
            return out;
        };
        let after_tag = &after_open[gt + 1..];
        match find_ascii_ci(after_tag, &close) {
            Some(close_start) => {
                rest = &after_tag[close_start + close.len()..];
            }
            None => return out,
        }
    }
}

fn find_ascii_ci(haystack: &str, needle: &str) -> Option<usize> {
    let hb = haystack.as_bytes();
    let nb = needle.as_bytes();
    if nb.is_empty() || hb.len() < nb.len() {
        return None;
    }
    'outer: for i in 0..=(hb.len() - nb.len()) {
        for j in 0..nb.len() {
            if !hb[i + j].eq_ignore_ascii_case(&nb[j]) {
                continue 'outer;
            }
        }
        return Some(i);
    }
    None
}

/// Full pipeline for turning a chat-completion `content` string into a JSON payload:
/// strip reasoning tags (`<think>`, `<reasoning>`, `<reflection>`) then markdown fences.
pub fn extract_json_payload(s: &str) -> String {
    let a = strip_tag_block(s, "think");
    let b = strip_tag_block(&a, "reasoning");
    let c = strip_tag_block(&b, "reflection");
    strip_markdown_fences(&c).to_string()
}

/// Strip markdown code fences from AI response content.
/// Models sometimes wrap JSON in ```json ... ``` despite response_format: json_object.
pub fn strip_markdown_fences(s: &str) -> &str {
    let trimmed = s.trim();
    // Find the first '{' — we always expect a JSON object, not array.
    // Perplexity uses [1], [2] as citation markers which look like JSON arrays.
    let start = trimmed.find('{').unwrap_or(0);
    let end = trimmed.rfind('}').map(|i| i + 1).unwrap_or(trimmed.len());
    if start < end {
        &trimmed[start..end]
    } else {
        trimmed
    }
}

/// Attempt to repair truncated JSON from AI responses.
/// Closes unclosed strings, arrays, and objects so serde can parse partial output.
pub fn repair_truncated_json(input: &str) -> String {
    let mut s = input.trim().to_string();

    // Remove trailing commas before } or ]
    loop {
        let before = s.len();
        s = s.replace(",}", "}").replace(",]", "]");
        while let Some(pos) = s.find(",\n}") { s.replace_range(pos..pos+3, "\n}"); }
        while let Some(pos) = s.find(",\n]") { s.replace_range(pos..pos+3, "\n]"); }
        while let Some(pos) = s.find(", }") { s.replace_range(pos..pos+3, " }"); }
        while let Some(pos) = s.find(", ]") { s.replace_range(pos..pos+3, " ]"); }
        if s.len() == before { break; }
    }

    // Count open/close brackets
    let mut open_braces = 0i32;
    let mut open_brackets = 0i32;
    let mut in_string = false;
    let mut escape_next = false;

    for c in s.chars() {
        if escape_next { escape_next = false; continue; }
        if c == '\\' && in_string { escape_next = true; continue; }
        if c == '"' { in_string = !in_string; continue; }
        if in_string { continue; }
        match c {
            '{' => open_braces += 1,
            '}' => open_braces -= 1,
            '[' => open_brackets += 1,
            ']' => open_brackets -= 1,
            _ => {}
        }
    }

    // Close unclosed string
    if in_string {
        s.push('"');
    }

    // Close unclosed arrays then objects
    for _ in 0..open_brackets { s.push(']'); }
    for _ in 0..open_braces { s.push('}'); }

    // Final cleanup: remove trailing commas that appeared after closing
    s = s.replace(",}", "}").replace(",]", "]");

    tracing::debug!(original_len = input.len(), repaired_len = s.len(), "JSON repair attempted");
    s
}

/// Safe UTF-8 string truncation — finds the nearest char boundary at or before `max_bytes`.
fn safe_truncate(s: &str, max_bytes: usize) -> usize {
    let target = s.len().min(max_bytes);
    let mut end = target;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    end
}

// === Response types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiKssResponse {
    #[serde(default)]
    pub kss_sections: Vec<AiKssSection>,
    #[serde(default)]
    pub overhead: AiKssOverhead,
    #[serde(default)]
    pub total_items: usize,
    #[serde(default)]
    pub construction_subtotal_eur: f64,
    #[serde(default)]
    pub total_eur: f64,
    #[serde(default)]
    pub drawing_type: String,
    #[serde(default)]
    pub language_detected: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiKssOverhead {
    #[serde(default)]
    pub admin_rate_pct: f64,
    #[serde(default)]
    pub contingency_rate_pct: f64,
    #[serde(default)]
    pub delivery_storage_rate_pct: f64,
    #[serde(default)]
    pub profit_rate_pct: f64,
    #[serde(default)]
    pub vat_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiKssSection {
    pub number: String,
    pub title: String,
    pub items: Vec<AiKssItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiKssItem {
    pub sek_code: String,
    pub description: String,
    pub unit: String,
    pub quantity: f64,
    #[serde(default)]
    pub material_price_eur: f64,
    #[serde(default)]
    pub labor_price_eur: f64,
    /// Total = (material + labor) × quantity. Fallback: price_eur × quantity
    #[serde(default)]
    pub price_eur: f64,
    pub confidence: f64,
    pub reasoning: String,

    // Required traceability fields (Phase 4 of the audit-driven plan).
    // The Opus prompt asks for these explicitly; absence → they default to
    // "none" / "assumed_typical" and confidence is capped at 0.5 downstream.
    #[serde(default)]
    pub source_layer: Option<String>,
    #[serde(default)]
    pub source_annotation: Option<String>,
    #[serde(default)]
    pub extraction_basis: Option<String>,
}
