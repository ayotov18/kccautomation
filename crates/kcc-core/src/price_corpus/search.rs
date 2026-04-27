//! Trigram + tsvector retrieval over `user_price_corpus`.
//!
//! Combines pg_trgm `similarity()` (fuzzy character-level match) with
//! `ts_rank` over the description tsvector (token-level relevance) and
//! returns top-K matches above a similarity floor.
//!
//! No embeddings, no model server. Postgres extensions only. Search runs
//! in tens of milliseconds on tens of thousands of corpus rows thanks to
//! the GIN trigram index.

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusMatch {
    pub id: Uuid,
    pub sek_code: Option<String>,
    pub description: String,
    pub unit: String,
    pub material_price_lv: f64,
    pub labor_price_lv: f64,
    pub total_unit_price_lv: f64,
    pub similarity: f64,
    pub source_sheet: Option<String>,
    pub source_row: Option<i32>,
}

#[derive(Debug, Clone, Copy)]
pub struct SearchOptions {
    /// Minimum trigram similarity for a row to be returned. 0.30 catches
    /// "Шперплат 18мм" vs "Шперплат мебелен 18 мм"; 0.45 is roughly the
    /// "I'm pretty sure these mean the same thing" threshold we use for
    /// hybrid mode's auto-acceptance.
    pub min_similarity: f64,
    pub top_k: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            min_similarity: 0.30,
            top_k: 5,
        }
    }
}

/// Search the user's corpus for rows similar to `query`. Returns top-K
/// matches above the similarity floor, ordered by combined score
/// (similarity weighted 0.7 + ts_rank weighted 0.3).
pub async fn search_corpus(
    db: &PgPool,
    user_id: Uuid,
    query: &str,
    opts: SearchOptions,
) -> Result<Vec<CorpusMatch>, sqlx::Error> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    // We use plainto_tsquery on the 'simple' config to match the trigger.
    // The combined score lets ts_rank break ties when several rows have
    // the same trigram similarity.
    let rows = sqlx::query(
        r#"
        SELECT id, sek_code, description, unit,
               COALESCE(material_price_lv, 0.0)   AS material_price_lv,
               COALESCE(labor_price_lv, 0.0)      AS labor_price_lv,
               COALESCE(total_unit_price_lv, 0.0) AS total_unit_price_lv,
               source_sheet, source_row,
               similarity(description, $2)        AS sim,
               ts_rank(description_tsv, plainto_tsquery('simple', $2)) AS rank
          FROM user_price_corpus
         WHERE user_id = $1
           AND similarity(description, $2) >= $3
         ORDER BY (similarity(description, $2) * 0.7
                  + ts_rank(description_tsv, plainto_tsquery('simple', $2)) * 0.3) DESC
         LIMIT $4
        "#,
    )
    .bind(user_id)
    .bind(query)
    .bind(opts.min_similarity)
    .bind(opts.top_k as i64)
    .fetch_all(db)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(CorpusMatch {
            id: r.try_get("id")?,
            sek_code: r.try_get("sek_code").ok(),
            description: r.try_get("description")?,
            unit: r.try_get("unit")?,
            material_price_lv: r.try_get("material_price_lv")?,
            labor_price_lv: r.try_get("labor_price_lv")?,
            total_unit_price_lv: r.try_get("total_unit_price_lv")?,
            similarity: r.try_get::<f32, _>("sim")? as f64,
            source_sheet: r.try_get("source_sheet").ok(),
            source_row: r.try_get("source_row").ok(),
        });
    }
    Ok(out)
}

/// Returns the count of corpus rows for a user. Used by the frontend mode
/// chooser to grey-out "RAG" when the user has no library yet.
pub async fn corpus_size(db: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM user_price_corpus WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(db)
            .await?;
    Ok(row.0)
}
