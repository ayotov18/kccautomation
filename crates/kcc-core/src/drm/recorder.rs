//! DRM artifact recorder: persists mapping decisions after KSS generation.
//!
//! Upserts into drm_artifacts: if the same (user_id, artifact_type, input_key_normalized, sek_code)
//! already exists, increment times_confirmed. Otherwise insert new.

use uuid::Uuid;

use super::{normalize_key, ARTIFACT_BLOCK, ARTIFACT_FEATURE_SEK, ARTIFACT_LAYER};

/// Record all mappings from a KSS generation run as DRM artifacts.
/// This is the passive learning path — called after every successful KSS generation.
pub async fn record_kss_artifacts(
    db: &sqlx::PgPool,
    user_id: Uuid,
    drawing_id: Uuid,
    layer_mappings: &[(String, String, String)], // (layer_name, sek_group, description)
    block_mappings: &[(String, String, String)],  // (block_name, sek_group, description)
    kss_items: &[(String, String, String, String)], // (description, sek_code, sek_group, unit)
) -> Result<usize, sqlx::Error> {
    let mut recorded = 0;

    // Record layer → SEK mappings
    for (layer, sek_group, desc) in layer_mappings {
        if upsert_artifact(db, user_id, drawing_id, ARTIFACT_LAYER, layer, None, Some(sek_group), Some(desc), None).await? {
            recorded += 1;
        }
    }

    // Record block → SEK mappings
    for (block, sek_group, desc) in block_mappings {
        if upsert_artifact(db, user_id, drawing_id, ARTIFACT_BLOCK, block, None, Some(sek_group), Some(desc), None).await? {
            recorded += 1;
        }
    }

    // Record feature description → SEK code mappings
    for (desc, sek_code, sek_group, unit) in kss_items {
        if upsert_artifact(db, user_id, drawing_id, ARTIFACT_FEATURE_SEK, desc, Some(sek_code), Some(sek_group), Some(desc), Some(unit)).await? {
            recorded += 1;
        }
    }

    tracing::info!(recorded, "DRM artifacts recorded");
    Ok(recorded)
}

/// Record a user correction as a high-confidence DRM artifact.
pub async fn record_user_correction(
    db: &sqlx::PgPool,
    user_id: Uuid,
    drawing_id: Uuid,
    artifact_type: &str,
    input_key: &str,
    sek_code: &str,
    sek_group: &str,
    description: &str,
    unit: &str,
) -> Result<(), sqlx::Error> {
    let normalized = normalize_key(input_key);

    // Check if artifact exists
    let existing: Option<(Uuid, i32)> = sqlx::query_as(
        "SELECT id, times_confirmed FROM drm_artifacts WHERE user_id = $1 AND artifact_type = $2 AND input_key_normalized = $3 AND sek_code = $4",
    )
    .bind(user_id)
    .bind(artifact_type)
    .bind(&normalized)
    .bind(sek_code)
    .fetch_optional(db)
    .await?;

    if let Some((id, _)) = existing {
        // Same mapping confirmed by user → boost confidence to 1.0
        sqlx::query(
            "UPDATE drm_artifacts SET confidence = 1.0, source = 'user_correction', times_confirmed = times_confirmed + 1, updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .execute(db)
        .await?;
    } else {
        // Check if a DIFFERENT mapping existed for this key (user is overriding)
        let overridden: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM drm_artifacts WHERE user_id = $1 AND artifact_type = $2 AND input_key_normalized = $3 AND sek_code != $4 LIMIT 1",
        )
        .bind(user_id)
        .bind(artifact_type)
        .bind(&normalized)
        .bind(sek_code)
        .fetch_optional(db)
        .await?;

        if let Some((old_id,)) = overridden {
            // Mark old artifact as overridden
            sqlx::query("UPDATE drm_artifacts SET times_overridden = times_overridden + 1, confidence = confidence * 0.5, updated_at = now() WHERE id = $1")
                .bind(old_id)
                .execute(db)
                .await?;
        }

        // Insert new user-corrected artifact
        sqlx::query(
            "INSERT INTO drm_artifacts (user_id, drawing_id, artifact_type, input_key, input_key_normalized, sek_code, sek_group, description_bg, unit, source, confidence, times_confirmed)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'user_correction', 1.0, 1)",
        )
        .bind(user_id)
        .bind(drawing_id)
        .bind(artifact_type)
        .bind(input_key)
        .bind(&normalized)
        .bind(sek_code)
        .bind(sek_group)
        .bind(description)
        .bind(unit)
        .execute(db)
        .await?;
    }

    Ok(())
}

/// Upsert an auto-generated artifact. Returns true if a new row was created.
async fn upsert_artifact(
    db: &sqlx::PgPool,
    user_id: Uuid,
    drawing_id: Uuid,
    artifact_type: &str,
    input_key: &str,
    sek_code: Option<&str>,
    sek_group: Option<&str>,
    description: Option<&str>,
    unit: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let normalized = normalize_key(input_key);
    if normalized.is_empty() {
        return Ok(false);
    }

    // Check if exact match exists
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM drm_artifacts WHERE user_id = $1 AND artifact_type = $2 AND input_key_normalized = $3 AND COALESCE(sek_code, '') = COALESCE($4, '')",
    )
    .bind(user_id)
    .bind(artifact_type)
    .bind(&normalized)
    .bind(sek_code)
    .fetch_optional(db)
    .await?;

    if let Some((id,)) = existing {
        // Increment confirmation count, slightly boost confidence
        sqlx::query(
            "UPDATE drm_artifacts SET times_confirmed = times_confirmed + 1, confidence = LEAST(confidence + 0.05, 1.0), updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .execute(db)
        .await?;
        Ok(false)
    } else {
        sqlx::query(
            "INSERT INTO drm_artifacts (user_id, drawing_id, artifact_type, input_key, input_key_normalized, sek_code, sek_group, description_bg, unit, source, confidence)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'auto', 0.5)",
        )
        .bind(user_id)
        .bind(drawing_id)
        .bind(artifact_type)
        .bind(input_key)
        .bind(&normalized)
        .bind(sek_code)
        .bind(sek_group)
        .bind(description)
        .bind(unit)
        .execute(db)
        .await?;
        Ok(true)
    }
}
