use crate::models::*;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AssemblyError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Assembly not found: {0}")]
    NotFound(Uuid),
    #[error("Component not found: {0}")]
    ComponentNotFound(Uuid),
}

/// List assemblies for a project (or all template assemblies if project_id is None).
pub async fn list_assemblies(
    db: &PgPool,
    project_id: Option<Uuid>,
) -> Result<Vec<Assembly>, AssemblyError> {
    let assemblies = if let Some(pid) = project_id {
        sqlx::query_as::<_, Assembly>(
            "SELECT * FROM assemblies WHERE project_id = $1 ORDER BY name",
        )
        .bind(pid)
        .fetch_all(db)
        .await?
    } else {
        sqlx::query_as::<_, Assembly>(
            "SELECT * FROM assemblies WHERE is_template = true ORDER BY name",
        )
        .fetch_all(db)
        .await?
    };

    Ok(assemblies)
}

/// Create a new assembly.
pub async fn create_assembly(
    db: &PgPool,
    dto: CreateAssembly,
) -> Result<Assembly, AssemblyError> {
    let is_template = dto.is_template.unwrap_or(false);

    let assembly = sqlx::query_as::<_, Assembly>(
        r#"INSERT INTO assemblies
           (id, project_id, name, description, unit, formula, total_rate, category, is_template, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, 0.0, $7, $8, now(), now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(dto.project_id)
    .bind(&dto.name)
    .bind(&dto.description)
    .bind(&dto.unit)
    .bind(&dto.formula)
    .bind(&dto.category)
    .bind(is_template)
    .fetch_one(db)
    .await?;

    Ok(assembly)
}

/// Get an assembly with all its components.
pub async fn get_assembly(
    db: &PgPool,
    id: Uuid,
) -> Result<AssemblyWithComponents, AssemblyError> {
    let assembly = sqlx::query_as::<_, Assembly>("SELECT * FROM assemblies WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(AssemblyError::NotFound(id))?;

    let components = sqlx::query_as::<_, Component>(
        "SELECT * FROM assembly_components WHERE assembly_id = $1 ORDER BY sort_order",
    )
    .bind(id)
    .fetch_all(db)
    .await?;

    Ok(AssemblyWithComponents {
        assembly,
        components,
    })
}

/// Add a component to an assembly. Auto-computes total = quantity * unit_cost * factor.
pub async fn add_component(
    db: &PgPool,
    assembly_id: Uuid,
    dto: CreateComponent,
) -> Result<Component, AssemblyError> {
    let factor = dto.factor.unwrap_or(1.0);
    let sort_order = dto.sort_order.unwrap_or(0);
    let total = dto.quantity * dto.unit_cost * factor;

    let component = sqlx::query_as::<_, Component>(
        r#"INSERT INTO assembly_components
           (id, assembly_id, cost_item_id, name, description, unit, quantity, unit_cost, factor, total, sort_order, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now(), now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(assembly_id)
    .bind(dto.cost_item_id)
    .bind(&dto.name)
    .bind(&dto.description)
    .bind(&dto.unit)
    .bind(dto.quantity)
    .bind(dto.unit_cost)
    .bind(factor)
    .bind(total)
    .bind(sort_order)
    .fetch_one(db)
    .await?;

    // Recompute assembly total_rate
    recompute_total_rate(db, assembly_id).await?;

    Ok(component)
}

/// Update a component. Re-computes total and assembly total_rate.
pub async fn update_component(
    db: &PgPool,
    component_id: Uuid,
    dto: UpdateComponent,
) -> Result<Component, AssemblyError> {
    let current = sqlx::query_as::<_, Component>(
        "SELECT * FROM assembly_components WHERE id = $1",
    )
    .bind(component_id)
    .fetch_optional(db)
    .await?
    .ok_or(AssemblyError::ComponentNotFound(component_id))?;

    let name = dto.name.unwrap_or(current.name);
    let description = dto.description.or(current.description);
    let unit = dto.unit.or(current.unit);
    let quantity = dto.quantity.unwrap_or(current.quantity);
    let unit_cost = dto.unit_cost.unwrap_or(current.unit_cost);
    let factor = dto.factor.unwrap_or(current.factor);
    let sort_order = dto.sort_order.unwrap_or(current.sort_order);
    let total = quantity * unit_cost * factor;
    let cost_item_id = dto.cost_item_id.or(current.cost_item_id);

    let component = sqlx::query_as::<_, Component>(
        r#"UPDATE assembly_components
           SET cost_item_id = $1, name = $2, description = $3, unit = $4, quantity = $5,
               unit_cost = $6, factor = $7, total = $8, sort_order = $9, updated_at = now()
           WHERE id = $10
           RETURNING *"#,
    )
    .bind(cost_item_id)
    .bind(&name)
    .bind(&description)
    .bind(&unit)
    .bind(quantity)
    .bind(unit_cost)
    .bind(factor)
    .bind(total)
    .bind(sort_order)
    .bind(component_id)
    .fetch_one(db)
    .await?;

    // Recompute assembly total_rate
    recompute_total_rate(db, current.assembly_id).await?;

    Ok(component)
}

/// Delete a component from an assembly.
pub async fn delete_component(db: &PgPool, component_id: Uuid) -> Result<(), AssemblyError> {
    let current = sqlx::query_as::<_, Component>(
        "SELECT * FROM assembly_components WHERE id = $1",
    )
    .bind(component_id)
    .fetch_optional(db)
    .await?
    .ok_or(AssemblyError::ComponentNotFound(component_id))?;

    sqlx::query("DELETE FROM assembly_components WHERE id = $1")
        .bind(component_id)
        .execute(db)
        .await?;

    // Recompute assembly total_rate
    recompute_total_rate(db, current.assembly_id).await?;

    Ok(())
}

/// Compute and return the total rate for an assembly.
/// Total rate = SUM(component.quantity * component.unit_cost * component.factor)
pub async fn compute_total_rate(db: &PgPool, assembly_id: Uuid) -> Result<f64, AssemblyError> {
    let total: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total), 0) FROM assembly_components WHERE assembly_id = $1",
    )
    .bind(assembly_id)
    .fetch_one(db)
    .await?;

    Ok(total)
}

/// Recompute and persist the assembly total_rate from its components.
async fn recompute_total_rate(db: &PgPool, assembly_id: Uuid) -> Result<(), AssemblyError> {
    let total = compute_total_rate(db, assembly_id).await?;

    sqlx::query("UPDATE assemblies SET total_rate = $1, updated_at = now() WHERE id = $2")
        .bind(total)
        .bind(assembly_id)
        .execute(db)
        .await?;

    Ok(())
}
