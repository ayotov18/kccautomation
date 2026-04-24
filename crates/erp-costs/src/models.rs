use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A cost item in the cost database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CostItem {
    pub id: Uuid,
    pub code: Option<String>,
    pub description: String,
    pub unit: Option<String>,
    pub unit_cost: f64,
    pub currency: String,
    pub region: String,
    pub source: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub effective_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating a cost item.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCostItem {
    pub code: Option<String>,
    pub description: String,
    pub unit: Option<String>,
    pub unit_cost: f64,
    pub currency: Option<String>,
    pub region: String,
    pub source: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub tags: Option<serde_json::Value>,
}

/// Row from a CSV import.
#[derive(Debug, Clone, Deserialize)]
pub struct CsvCostRow {
    pub code: Option<String>,
    pub description: String,
    pub unit: Option<String>,
    pub unit_cost: f64,
    pub currency: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
}
