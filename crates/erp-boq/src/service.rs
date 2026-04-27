use crate::models::*;
use erp_core::markup::{self, ApplyTo, Markup, MarkupType};
use erp_core::regional::Region;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BoqError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("BOQ not found: {0}")]
    NotFound(Uuid),
    #[error("Position not found: {0}")]
    PositionNotFound(Uuid),
    #[error("Markup not found: {0}")]
    MarkupNotFound(Uuid),
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(Uuid),
    #[error("Unknown region: {0}")]
    UnknownRegion(String),
}

/// Create a new BOQ.
pub async fn create_boq(
    db: &PgPool,
    project_id: Uuid,
    name: &str,
    created_by: Uuid,
) -> Result<Boq, BoqError> {
    let boq = sqlx::query_as::<_, Boq>(
        r#"INSERT INTO boqs (id, project_id, name, currency, status, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'EUR', 'draft', $4, now(), now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(project_id)
    .bind(name)
    .bind(created_by)
    .fetch_one(db)
    .await?;

    Ok(boq)
}

/// Get a BOQ with all its positions.
pub async fn get_boq(db: &PgPool, boq_id: Uuid) -> Result<BoqWithPositions, BoqError> {
    let boq = sqlx::query_as::<_, Boq>("SELECT * FROM boqs WHERE id = $1")
        .bind(boq_id)
        .fetch_optional(db)
        .await?
        .ok_or(BoqError::NotFound(boq_id))?;

    let positions = sqlx::query_as::<_, Position>(
        "SELECT * FROM boq_positions WHERE boq_id = $1 ORDER BY sort_order, ordinal",
    )
    .bind(boq_id)
    .fetch_all(db)
    .await?;

    Ok(BoqWithPositions { boq, positions })
}

/// Create a position within a BOQ. Auto-computes total = quantity * unit_rate.
pub async fn create_position(
    db: &PgPool,
    boq_id: Uuid,
    dto: CreatePosition,
) -> Result<Position, BoqError> {
    let total = dto.quantity * dto.unit_rate;
    let sort_order = dto.sort_order.unwrap_or(0);

    let position = sqlx::query_as::<_, Position>(
        r#"INSERT INTO boq_positions
           (id, boq_id, parent_id, ordinal, description, unit, quantity, unit_rate, total, section, notes, sort_order, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, now(), now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(boq_id)
    .bind(dto.parent_id)
    .bind(&dto.ordinal)
    .bind(&dto.description)
    .bind(&dto.unit)
    .bind(dto.quantity)
    .bind(dto.unit_rate)
    .bind(total)
    .bind(&dto.section)
    .bind(&dto.notes)
    .bind(sort_order)
    .fetch_one(db)
    .await?;

    // Update BOQ timestamp
    sqlx::query("UPDATE boqs SET updated_at = now() WHERE id = $1")
        .bind(boq_id)
        .execute(db)
        .await?;

    Ok(position)
}

/// Update a position. Re-computes total if quantity or unit_rate changed.
pub async fn update_position(
    db: &PgPool,
    position_id: Uuid,
    dto: UpdatePosition,
) -> Result<Position, BoqError> {
    // Fetch current position
    let current = sqlx::query_as::<_, Position>(
        "SELECT * FROM boq_positions WHERE id = $1",
    )
    .bind(position_id)
    .fetch_optional(db)
    .await?
    .ok_or(BoqError::PositionNotFound(position_id))?;

    let ordinal = dto.ordinal.unwrap_or(current.ordinal);
    let description = dto.description.unwrap_or(current.description);
    let unit = dto.unit.or(current.unit);
    let quantity = dto.quantity.unwrap_or(current.quantity);
    let unit_rate = dto.unit_rate.unwrap_or(current.unit_rate);
    let total = quantity * unit_rate;
    let section = dto.section.or(current.section);
    let notes = dto.notes.or(current.notes);
    let sort_order = dto.sort_order.unwrap_or(current.sort_order);

    let position = sqlx::query_as::<_, Position>(
        r#"UPDATE boq_positions
           SET ordinal = $1, description = $2, unit = $3, quantity = $4, unit_rate = $5,
               total = $6, section = $7, notes = $8, sort_order = $9, updated_at = now()
           WHERE id = $10
           RETURNING *"#,
    )
    .bind(&ordinal)
    .bind(&description)
    .bind(&unit)
    .bind(quantity)
    .bind(unit_rate)
    .bind(total)
    .bind(&section)
    .bind(&notes)
    .bind(sort_order)
    .bind(position_id)
    .fetch_one(db)
    .await?;

    // Update BOQ timestamp
    sqlx::query("UPDATE boqs SET updated_at = now() WHERE id = $1")
        .bind(current.boq_id)
        .execute(db)
        .await?;

    Ok(position)
}

/// Delete a position from a BOQ.
pub async fn delete_position(db: &PgPool, position_id: Uuid) -> Result<(), BoqError> {
    let result = sqlx::query("DELETE FROM boq_positions WHERE id = $1")
        .bind(position_id)
        .execute(db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(BoqError::PositionNotFound(position_id));
    }

    Ok(())
}

/// Compute the grand total for a BOQ using its positions and markups.
pub async fn compute_grand_total(db: &PgPool, boq_id: Uuid) -> Result<GrandTotal, BoqError> {
    let direct_cost: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total), 0) FROM boq_positions WHERE boq_id = $1",
    )
    .bind(boq_id)
    .fetch_one(db)
    .await?;

    let db_markups = sqlx::query_as::<_, BoqMarkup>(
        "SELECT * FROM boq_markups WHERE boq_id = $1 ORDER BY sort_order",
    )
    .bind(boq_id)
    .fetch_all(db)
    .await?;

    let core_markups: Vec<Markup> = db_markups
        .iter()
        .map(|m| Markup {
            name: m.name.clone(),
            markup_type: match m.markup_type.as_str() {
                "fixed" => MarkupType::Fixed,
                "per_unit" => MarkupType::PerUnit,
                _ => MarkupType::Percentage,
            },
            percentage: m.percentage,
            fixed_amount: m.fixed_amount,
            apply_to: match m.apply_to.as_str() {
                "cumulative" => ApplyTo::Cumulative,
                _ => ApplyTo::DirectCost,
            },
            sort_order: m.sort_order,
            is_active: m.is_active,
        })
        .collect();

    let (markup_results, grand_total) = markup::calculate_markups(direct_cost, &core_markups);

    Ok(GrandTotal {
        direct_cost,
        markups: markup_results,
        grand_total,
    })
}

/// Get all markups for a BOQ.
pub async fn get_markups(db: &PgPool, boq_id: Uuid) -> Result<Vec<BoqMarkup>, BoqError> {
    let markups = sqlx::query_as::<_, BoqMarkup>(
        "SELECT * FROM boq_markups WHERE boq_id = $1 ORDER BY sort_order",
    )
    .bind(boq_id)
    .fetch_all(db)
    .await?;

    Ok(markups)
}

/// Create a markup for a BOQ.
pub async fn create_markup(
    db: &PgPool,
    boq_id: Uuid,
    dto: CreateMarkup,
) -> Result<BoqMarkup, BoqError> {
    let is_active = dto.is_active.unwrap_or(true);

    let markup = sqlx::query_as::<_, BoqMarkup>(
        r#"INSERT INTO boq_markups
           (id, boq_id, name, markup_type, percentage, fixed_amount, apply_to, sort_order, is_active, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(boq_id)
    .bind(&dto.name)
    .bind(&dto.markup_type)
    .bind(dto.percentage)
    .bind(dto.fixed_amount)
    .bind(&dto.apply_to)
    .bind(dto.sort_order)
    .bind(is_active)
    .fetch_one(db)
    .await?;

    Ok(markup)
}

/// Update a markup.
pub async fn update_markup(
    db: &PgPool,
    markup_id: Uuid,
    dto: UpdateMarkup,
) -> Result<BoqMarkup, BoqError> {
    let current = sqlx::query_as::<_, BoqMarkup>(
        "SELECT * FROM boq_markups WHERE id = $1",
    )
    .bind(markup_id)
    .fetch_optional(db)
    .await?
    .ok_or(BoqError::MarkupNotFound(markup_id))?;

    let name = dto.name.unwrap_or(current.name);
    let markup_type = dto.markup_type.unwrap_or(current.markup_type);
    let percentage = dto.percentage.unwrap_or(current.percentage);
    let fixed_amount = dto.fixed_amount.unwrap_or(current.fixed_amount);
    let apply_to = dto.apply_to.unwrap_or(current.apply_to);
    let sort_order = dto.sort_order.unwrap_or(current.sort_order);
    let is_active = dto.is_active.unwrap_or(current.is_active);

    let markup = sqlx::query_as::<_, BoqMarkup>(
        r#"UPDATE boq_markups
           SET name = $1, markup_type = $2, percentage = $3, fixed_amount = $4,
               apply_to = $5, sort_order = $6, is_active = $7
           WHERE id = $8
           RETURNING *"#,
    )
    .bind(&name)
    .bind(&markup_type)
    .bind(percentage)
    .bind(fixed_amount)
    .bind(&apply_to)
    .bind(sort_order)
    .bind(is_active)
    .bind(markup_id)
    .fetch_one(db)
    .await?;

    Ok(markup)
}

/// Delete a markup.
pub async fn delete_markup(db: &PgPool, markup_id: Uuid) -> Result<(), BoqError> {
    let result = sqlx::query("DELETE FROM boq_markups WHERE id = $1")
        .bind(markup_id)
        .execute(db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(BoqError::MarkupNotFound(markup_id));
    }

    Ok(())
}

/// Apply default regional markups to a BOQ, replacing any existing markups.
pub async fn apply_default_markups(
    db: &PgPool,
    boq_id: Uuid,
    region_code: &str,
) -> Result<Vec<BoqMarkup>, BoqError> {
    let region =
        Region::from_code(region_code).ok_or_else(|| BoqError::UnknownRegion(region_code.into()))?;

    // Delete existing markups
    sqlx::query("DELETE FROM boq_markups WHERE boq_id = $1")
        .bind(boq_id)
        .execute(db)
        .await?;

    let template_markups = region.default_markups();
    let mut created = Vec::new();

    for m in template_markups {
        let markup = sqlx::query_as::<_, BoqMarkup>(
            r#"INSERT INTO boq_markups
               (id, boq_id, name, markup_type, percentage, fixed_amount, apply_to, sort_order, is_active, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
               RETURNING *"#,
        )
        .bind(Uuid::new_v4())
        .bind(boq_id)
        .bind(&m.name)
        .bind(match m.markup_type {
            MarkupType::Percentage => "percentage",
            MarkupType::Fixed => "fixed",
            MarkupType::PerUnit => "per_unit",
        })
        .bind(m.percentage)
        .bind(m.fixed_amount)
        .bind(match m.apply_to {
            ApplyTo::DirectCost => "direct_cost",
            ApplyTo::Cumulative => "cumulative",
        })
        .bind(m.sort_order)
        .bind(m.is_active)
        .fetch_one(db)
        .await?;

        created.push(markup);
    }

    Ok(created)
}

/// Create a snapshot of the current BOQ state.
pub async fn create_snapshot(
    db: &PgPool,
    boq_id: Uuid,
    name: &str,
    user_id: Uuid,
) -> Result<Snapshot, BoqError> {
    // Get current BOQ with positions
    let boq_data = get_boq(db, boq_id).await?;
    let markups = get_markups(db, boq_id).await?;
    let grand_total = compute_grand_total(db, boq_id).await?;

    let snapshot_data = serde_json::json!({
        "boq": boq_data.boq,
        "positions": boq_data.positions,
        "markups": markups,
        "grand_total": grand_total,
    });

    let snapshot = sqlx::query_as::<_, Snapshot>(
        r#"INSERT INTO boq_snapshots (id, boq_id, name, data, created_by, created_at)
           VALUES ($1, $2, $3, $4, $5, now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(boq_id)
    .bind(name)
    .bind(&snapshot_data)
    .bind(user_id)
    .fetch_one(db)
    .await?;

    Ok(snapshot)
}

/// List all snapshots for a BOQ.
pub async fn list_snapshots(db: &PgPool, boq_id: Uuid) -> Result<Vec<Snapshot>, BoqError> {
    let snapshots = sqlx::query_as::<_, Snapshot>(
        "SELECT * FROM boq_snapshots WHERE boq_id = $1 ORDER BY created_at DESC",
    )
    .bind(boq_id)
    .fetch_all(db)
    .await?;

    Ok(snapshots)
}

/// Restore a BOQ from a snapshot.
pub async fn restore_snapshot(
    db: &PgPool,
    boq_id: Uuid,
    snapshot_id: Uuid,
) -> Result<(), BoqError> {
    let snapshot = sqlx::query_as::<_, Snapshot>(
        "SELECT * FROM boq_snapshots WHERE id = $1 AND boq_id = $2",
    )
    .bind(snapshot_id)
    .bind(boq_id)
    .fetch_optional(db)
    .await?
    .ok_or(BoqError::SnapshotNotFound(snapshot_id))?;

    // Delete current positions and markups
    sqlx::query("DELETE FROM boq_positions WHERE boq_id = $1")
        .bind(boq_id)
        .execute(db)
        .await?;
    sqlx::query("DELETE FROM boq_markups WHERE boq_id = $1")
        .bind(boq_id)
        .execute(db)
        .await?;

    // Restore positions from snapshot
    if let Some(positions) = snapshot.data.get("positions").and_then(|v| v.as_array()) {
        for pos in positions {
            let p: Position = serde_json::from_value(pos.clone()).map_err(|_| {
                BoqError::SnapshotNotFound(snapshot_id)
            })?;
            sqlx::query(
                r#"INSERT INTO boq_positions
                   (id, boq_id, parent_id, ordinal, description, unit, quantity, unit_rate, total, section, notes, sort_order, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, now())"#,
            )
            .bind(Uuid::new_v4())
            .bind(boq_id)
            .bind(p.parent_id)
            .bind(&p.ordinal)
            .bind(&p.description)
            .bind(&p.unit)
            .bind(p.quantity)
            .bind(p.unit_rate)
            .bind(p.total)
            .bind(&p.section)
            .bind(&p.notes)
            .bind(p.sort_order)
            .bind(p.created_at)
            .execute(db)
            .await?;
        }
    }

    // Restore markups from snapshot
    if let Some(markups) = snapshot.data.get("markups").and_then(|v| v.as_array()) {
        for mkp in markups {
            let m: BoqMarkup = serde_json::from_value(mkp.clone()).map_err(|_| {
                BoqError::SnapshotNotFound(snapshot_id)
            })?;
            sqlx::query(
                r#"INSERT INTO boq_markups
                   (id, boq_id, name, markup_type, percentage, fixed_amount, apply_to, sort_order, is_active, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())"#,
            )
            .bind(Uuid::new_v4())
            .bind(boq_id)
            .bind(&m.name)
            .bind(&m.markup_type)
            .bind(m.percentage)
            .bind(m.fixed_amount)
            .bind(&m.apply_to)
            .bind(m.sort_order)
            .bind(m.is_active)
            .execute(db)
            .await?;
        }
    }

    // Update BOQ timestamp
    sqlx::query("UPDATE boqs SET updated_at = now() WHERE id = $1")
        .bind(boq_id)
        .execute(db)
        .await?;

    Ok(())
}

/// Log an activity for audit trail.
pub async fn log_activity(
    db: &PgPool,
    boq_id: Uuid,
    user_id: Uuid,
    action: &str,
    target_type: Option<&str>,
    target_id: Option<Uuid>,
    description: &str,
    changes: Option<serde_json::Value>,
) -> Result<(), BoqError> {
    sqlx::query(
        r#"INSERT INTO boq_activity_log
           (id, boq_id, user_id, action, target_type, target_id, description, changes, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())"#,
    )
    .bind(Uuid::new_v4())
    .bind(boq_id)
    .bind(user_id)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(description)
    .bind(changes)
    .execute(db)
    .await?;

    Ok(())
}
