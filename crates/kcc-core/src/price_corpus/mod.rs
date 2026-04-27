//! User price corpus — self-hosted RAG for pricing.
//!
//! Users upload their actual project offers (Excel) and the contents become
//! a per-user price corpus that the KSS generator can scatter through to
//! find prices instead of re-discovering them via Perplexity each time.
//!
//! Retrieval is pg_trgm + tsvector ranking — no embeddings, no model server.
//! Bulgarian KSS descriptions are highly stylized templates; trigram
//! similarity catches near-matches like "Шперплат 18мм" vs "Шперплат
//! мебелен 18 мм" without an embedding model.

pub mod parser;
pub mod search;

pub use parser::{parse_offer_xlsx, OfferRow, ParsedOffer};
pub use search::{CorpusMatch, SearchOptions};
