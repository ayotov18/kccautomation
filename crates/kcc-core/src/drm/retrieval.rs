//! DRM retrieval: queries PostgreSQL pg_trgm for similar historical artifacts.
//!
//! Called between feature extraction and KSS generation to provide
//! historical context for mapping decisions.

use uuid::Uuid;

use super::{
    ContextBundle, DrmMatch,
    ARTIFACT_ANNOTATION, ARTIFACT_BLOCK, ARTIFACT_FEATURE_SEK, ARTIFACT_LAYER, ARTIFACT_PRICE,
    matcher, normalize_key,
};

/// Row returned from the pg_trgm similarity query.
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct ArtifactRow {
    id: Uuid,
    input_key: String,
    input_key_normalized: String,
    sek_code: Option<String>,
    sek_group: Option<String>,
    description_bg: Option<String>,
    unit: Option<String>,
    confidence: f64,
    times_confirmed: i32,
    source: String,
    similarity: Option<f32>,  // pg_trgm similarity() returns FLOAT4/real
}

/// Retrieve DRM context for a drawing's features.
///
/// Extracts unique layer names, block names, and annotation texts from the drawing,
/// then queries drm_artifacts for similar historical mappings.
pub async fn retrieve_context(
    db: &sqlx::PgPool,
    user_id: Uuid,
    drawing: &crate::geometry::model::Drawing,
) -> Result<ContextBundle, sqlx::Error> {
    let mut bundle = ContextBundle::default();

    // Collect unique layer names
    let mut layers: Vec<String> = drawing
        .entities
        .iter()
        .map(|e| e.layer.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    layers.sort();

    // Collect unique block references
    let mut blocks: Vec<String> = drawing
        .entities
        .iter()
        .filter_map(|e| e.block_ref.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    blocks.sort();

    // Collect annotation texts
    let annotations: Vec<String> = drawing
        .annotations
        .iter()
        .map(|a| a.text.clone())
        .collect();

    // Query DRM for each input type
    for layer in &layers {
        let matches = query_similar(db, user_id, ARTIFACT_LAYER, layer).await?;
        bundle.layer_mappings.extend(matches);
    }

    for block in &blocks {
        let matches = query_similar(db, user_id, ARTIFACT_BLOCK, block).await?;
        bundle.block_mappings.extend(matches);
    }

    for annotation in &annotations {
        let matches = query_similar(db, user_id, ARTIFACT_ANNOTATION, annotation).await?;
        bundle.annotation_patterns.extend(matches);
    }

    // Feature-level SEK mappings (populated after KSS generation)
    let feature_matches = query_similar(db, user_id, ARTIFACT_FEATURE_SEK, "").await?;
    bundle.feature_sek_mappings = feature_matches;

    // Price association artifacts (from manual pricing + user corrections)
    let price_matches = query_similar(db, user_id, ARTIFACT_PRICE, "").await?;
    bundle.feature_sek_mappings.extend(price_matches);

    tracing::info!(
        layers = bundle.layer_mappings.len(),
        blocks = bundle.block_mappings.len(),
        annotations = bundle.annotation_patterns.len(),
        features = bundle.feature_sek_mappings.len(),
        auto_overrides = bundle.auto_override_count(),
        "DRM context retrieved"
    );

    Ok(bundle)
}

/// Query drm_artifacts for similar entries using pg_trgm.
async fn query_similar(
    db: &sqlx::PgPool,
    user_id: Uuid,
    artifact_type: &str,
    input_key: &str,
) -> Result<Vec<DrmMatch>, sqlx::Error> {
    let normalized = normalize_key(input_key);
    if normalized.is_empty() {
        return Ok(Vec::new());
    }

    let rows: Vec<ArtifactRow> = sqlx::query_as(
        r#"SELECT id, input_key, input_key_normalized, sek_code, sek_group, description_bg, unit, confidence, times_confirmed, source,
                  similarity(input_key_normalized, $1) as similarity
           FROM drm_artifacts
           WHERE user_id = $2
             AND artifact_type = $3
             AND similarity(input_key_normalized, $1) > 0.3
           ORDER BY similarity DESC, times_confirmed DESC
           LIMIT 5"#,
    )
    .bind(&normalized)
    .bind(user_id)
    .bind(artifact_type)
    .fetch_all(db)
    .await?;

    let matches = rows
        .into_iter()
        .map(|row| {
            let sim = row.similarity.unwrap_or(0.0) as f64; // f32 → f64
            let action = matcher::determine_action(sim, row.confidence, row.times_confirmed);

            DrmMatch {
                artifact_id: row.id,
                input_key: input_key.to_string(),
                matched_key: row.input_key,
                similarity: sim,
                sek_code: row.sek_code,
                sek_group: row.sek_group,
                description_bg: row.description_bg,
                unit: row.unit,
                confidence: row.confidence,
                times_confirmed: row.times_confirmed,
                source: row.source,
                action,
            }
        })
        .collect();

    Ok(matches)
}
