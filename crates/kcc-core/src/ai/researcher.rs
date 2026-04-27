//! AI-powered price research agent.
//!
//! Uses OpenRouter's web search (`:online`) to find Bulgarian construction prices
//! across the open web. Identifies gaps in existing pricing data and fills them.

use uuid::Uuid;

use super::{AiError, OpenRouterClient};
use crate::scraper::ScrapedPrice;

/// SEK categories to research with Bulgarian names and example items.
const SEK_CATEGORIES: &[(&str, &str, &str)] = &[
    ("СЕК01", "Земни работи", "изкоп, засипване, хумус"),
    ("СЕК02", "Кофражни работи", "кофраж за плочи, колони, стени"),
    ("СЕК03", "Армировъчни работи", "арматура, заварена мрежа"),
    ("СЕК04", "Бетонови работи", "бетон за основи, фундаменти, стени"),
    ("СЕК05", "Зидарски работи", "тухлена зидария, газобетон, каменна зидария"),
    ("СЕК06", "Покривни работи", "керемиди, битумни листове, метална покривка"),
    ("СЕК09", "Облицовъчни работи", "фаянс, теракот, гранитогрес, мрамор"),
    ("СЕК10", "Мазачески работи", "вътрешна мазилка, външна мазилка, шпакловка"),
    ("СЕК11", "Настилки и замазки", "ламинат, паркет, мозайка, замазка"),
    ("СЕК13", "Бояджийски работи", "латексово боядисване, грундиране, лакиране"),
    ("СЕК14", "Метални конструкции", "стоманени конструкции, метални врати"),
    ("СЕК15", "Хидроизолации", "битумна мембрана, PVC мембрана"),
    ("СЕК16", "Топлоизолации", "EPS, XPS, минерална вата"),
    ("СЕК17", "Столарски работи", "MDF врати, PVC дограма, алуминиева дограма"),
    ("СЕК20", "Сухо строителство", "гипсокартон, окачен таван"),
    ("СЕК22", "Сградни ВиК", "тръби, мивки, тоалетни, вани, душ кабини"),
    ("СЕК34", "Електрически инсталации", "кабели, контакти, ключове, осветление"),
];

const RESEARCH_SYSTEM_PROMPT: &str = r#"You are a Bulgarian construction price researcher.
You search the web for current construction work prices in Bulgaria (in €/EUR).

For each price you find from the web search results, extract:
- description: work item description in Bulgarian
- unit: measurement unit (М2, М3, м, бр., кг, тон)
- price_min_eur: minimum price in € (EUR)
- price_max_eur: maximum price in € (EUR)
- sek_code: СЕК code if identifiable (e.g., "СЕК05.002")
- source_url: the URL where you found this price
- confidence: 0.0-1.0 how certain you are this price is accurate

RULES:
1. Only include prices that are CLEARLY stated in the search results
2. Do NOT guess or interpolate prices
3. Prices should be in € (EUR), not EUR
4. If price is in EUR, convert at 1.956 €/€
5. Include both labor and material prices where available
6. Current 2024-2026 prices only

Output valid JSON:
{
  "prices": [
    {
      "description": "...",
      "unit": "М2",
      "price_min_eur": 12.0,
      "price_max_eur": 18.0,
      "sek_code": "СЕК10.011",
      "source_url": "https://...",
      "confidence": 0.9
    }
  ],
  "sources_found": 3,
  "notes": "..."
}"#;

/// AI-powered price research agent.
pub struct PriceResearchAgent<'a> {
    client: &'a OpenRouterClient,
}

/// Parsed price from AI research.
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct AiResearchedPrice {
    description: String,
    unit: String,
    price_min_eur: Option<f64>,
    price_max_eur: Option<f64>,
    sek_code: Option<String>,
    source_url: Option<String>,
    confidence: Option<f64>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct AiResearchResponse {
    prices: Vec<AiResearchedPrice>,
    #[serde(default)]
    sources_found: usize,
    #[serde(default)]
    notes: String,
}

impl<'a> PriceResearchAgent<'a> {
    pub fn new(client: &'a OpenRouterClient) -> Self {
        Self { client }
    }

    /// Research prices for all SEK categories with gaps.
    pub async fn research_gaps(
        &self,
        db: &sqlx::PgPool,
        user_id: Uuid,
    ) -> Result<Vec<ScrapedPrice>, AiError> {
        // Find which SEK groups have few/no prices
        let existing: Vec<(String, i64)> = sqlx::query_as(
            "SELECT COALESCE(sek_group, 'unknown'), COUNT(*) FROM scraped_price_rows WHERE user_id = $1 AND archived_at IS NULL GROUP BY sek_group"
        )
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|e| AiError::ParseError(format!("DB error: {e}")))?;

        let existing_counts: std::collections::HashMap<String, i64> = existing.into_iter().collect();

        let mut all_prices = Vec::new();

        for (sek_group, name_bg, examples) in SEK_CATEGORIES {
            let count = existing_counts.get(*sek_group).copied().unwrap_or(0);
            if count >= 5 {
                tracing::debug!(sek_group, count, "Skipping — sufficient prices");
                continue;
            }

            tracing::info!(sek_group, name_bg, existing = count, "Researching prices for gap");

            match self.research_category(sek_group, name_bg, examples).await {
                Ok(prices) => {
                    tracing::info!(sek_group, found = prices.len(), "AI research results");
                    all_prices.extend(prices);
                }
                Err(e) => {
                    tracing::warn!(sek_group, error = %e, "AI research failed for category");
                }
            }

            // Small delay between categories to avoid rate limiting
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Ok(all_prices)
    }

    /// Research prices for a single SEK category.
    async fn research_category(
        &self,
        sek_group: &str,
        category_name_bg: &str,
        examples: &str,
    ) -> Result<Vec<ScrapedPrice>, AiError> {
        let user_prompt = format!(
            "Search for current Bulgarian construction prices for:\n\
             Category: {} ({})\n\
             Example items: {}\n\n\
             Find 5-15 specific price items with unit prices in € (EUR).\n\
             Focus on labor + material combined rates.\n\
             Search on Bulgarian sites: daibau.bg, stroitelni-remonti.com, homefix.bg",
            category_name_bg, sek_group, examples
        );

        let domains = [
            "daibau.bg", "stroitelni-remonti.com", "homefix.bg",
            "mr-bricolage.bg", "bauhaus.bg", "praktiker.bg",
            "sek-bg.com", "smr.sek-bg.com",
        ];

        let (content, citations) = self.client.search_and_analyze(
            RESEARCH_SYSTEM_PROMPT,
            &user_prompt,
            Some(&domains),
        ).await?;

        tracing::debug!(
            sek_group, content_len = content.len(), citations = citations.len(),
            "AI research response for category"
        );

        // Parse the AI response
        let response: AiResearchResponse = serde_json::from_str(&content)
            .map_err(|e| AiError::ParseError(format!("Research JSON parse error: {e}")))?;

        // Convert to ScrapedPrice
        let prices: Vec<ScrapedPrice> = response.prices.into_iter()
            .filter(|p| {
                !p.description.is_empty()
                    && (p.price_min_eur.is_some() || p.price_max_eur.is_some())
            })
            .map(|p| {
                let min = p.price_min_eur;
                let max = p.price_max_eur.or(min);
                let min = min.or(max);

                ScrapedPrice::from_eur(
                    "ai_research",
                    p.source_url.as_deref().unwrap_or("https://openrouter.ai"),
                    &p.description,
                    &p.unit,
                    min, max,
                    Some(&format!("AI researched: {}", sek_group)),
                    Some(sek_group),
                    p.confidence.unwrap_or(0.7),
                )
            })
            .collect();

        Ok(prices)
    }
}
