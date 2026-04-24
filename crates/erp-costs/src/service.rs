use crate::models::*;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum CostError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Cost item not found: {0}")]
    NotFound(Uuid),
    #[error("CSV parse error: {0}")]
    CsvParse(String),
}

/// Search cost items using pg_trgm similarity on description.
pub async fn search_costs(
    db: &PgPool,
    query: &str,
    region: Option<&str>,
    limit: i64,
) -> Result<Vec<CostItem>, CostError> {
    let items = if let Some(region) = region {
        sqlx::query_as::<_, CostItem>(
            r#"SELECT * FROM cost_items
               WHERE region = $1
                 AND (description ILIKE '%' || $2 || '%' OR code ILIKE '%' || $2 || '%')
               ORDER BY similarity(description, $2) DESC
               LIMIT $3"#,
        )
        .bind(region)
        .bind(query)
        .bind(limit)
        .fetch_all(db)
        .await?
    } else {
        sqlx::query_as::<_, CostItem>(
            r#"SELECT * FROM cost_items
               WHERE description ILIKE '%' || $1 || '%' OR code ILIKE '%' || $1 || '%'
               ORDER BY similarity(description, $1) DESC
               LIMIT $2"#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(db)
        .await?
    };

    Ok(items)
}

/// Get a single cost item by ID.
pub async fn get_cost_item(db: &PgPool, id: Uuid) -> Result<CostItem, CostError> {
    let item = sqlx::query_as::<_, CostItem>("SELECT * FROM cost_items WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(CostError::NotFound(id))?;

    Ok(item)
}

/// Create a cost item.
pub async fn create_cost_item(db: &PgPool, dto: CreateCostItem) -> Result<CostItem, CostError> {
    let currency = dto.currency.unwrap_or_else(|| "BGN".to_string());

    let item = sqlx::query_as::<_, CostItem>(
        r#"INSERT INTO cost_items
           (id, code, description, unit, unit_cost, currency, region, source, category, subcategory, tags, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now(), now())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(&dto.code)
    .bind(&dto.description)
    .bind(&dto.unit)
    .bind(dto.unit_cost)
    .bind(&currency)
    .bind(&dto.region)
    .bind(&dto.source)
    .bind(&dto.category)
    .bind(&dto.subcategory)
    .bind(&dto.tags)
    .fetch_one(db)
    .await?;

    Ok(item)
}

/// Import cost items from CSV bytes. Returns the number of rows imported.
pub async fn import_csv(
    db: &PgPool,
    csv_bytes: &[u8],
    region: &str,
    source: &str,
) -> Result<usize, CostError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let mut count = 0;

    for result in reader.deserialize() {
        let row: CsvCostRow = result.map_err(|e| CostError::CsvParse(e.to_string()))?;
        let currency = row.currency.unwrap_or_else(|| "BGN".to_string());

        sqlx::query(
            r#"INSERT INTO cost_items
               (id, code, description, unit, unit_cost, currency, region, source, category, subcategory, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now(), now())"#,
        )
        .bind(Uuid::new_v4())
        .bind(&row.code)
        .bind(&row.description)
        .bind(&row.unit)
        .bind(row.unit_cost)
        .bind(&currency)
        .bind(region)
        .bind(source)
        .bind(&row.category)
        .bind(&row.subcategory)
        .execute(db)
        .await?;

        count += 1;
    }

    tracing::info!("Imported {} cost items from CSV for region {}", count, region);

    Ok(count)
}

/// List all distinct regions in the cost database.
pub async fn list_regions(db: &PgPool) -> Result<Vec<String>, CostError> {
    let regions: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT region FROM cost_items ORDER BY region")
            .fetch_all(db)
            .await?;

    Ok(regions)
}
