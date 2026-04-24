use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Bill of Quantities top-level record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Boq {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub currency: String,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single position (line item) within a BOQ.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Position {
    pub id: Uuid,
    pub boq_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub ordinal: String,
    pub description: String,
    pub unit: Option<String>,
    pub quantity: f64,
    pub unit_rate: f64,
    pub total: f64,
    pub section: Option<String>,
    pub notes: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A markup applied to a BOQ.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BoqMarkup {
    pub id: Uuid,
    pub boq_id: Uuid,
    pub name: String,
    pub markup_type: String,
    pub percentage: f64,
    pub fixed_amount: f64,
    pub apply_to: String,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// A snapshot (versioned copy) of a BOQ at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Snapshot {
    pub id: Uuid,
    pub boq_id: Uuid,
    pub name: String,
    pub data: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Activity log entry for BOQ audit trail.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActivityLog {
    pub id: Uuid,
    pub boq_id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub description: String,
    pub changes: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a new position.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePosition {
    pub parent_id: Option<Uuid>,
    pub ordinal: String,
    pub description: String,
    pub unit: Option<String>,
    pub quantity: f64,
    pub unit_rate: f64,
    pub section: Option<String>,
    pub notes: Option<String>,
    pub sort_order: Option<i32>,
}

/// DTO for updating a position.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePosition {
    pub ordinal: Option<String>,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub quantity: Option<f64>,
    pub unit_rate: Option<f64>,
    pub section: Option<String>,
    pub notes: Option<String>,
    pub sort_order: Option<i32>,
}

/// DTO for creating a markup.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMarkup {
    pub name: String,
    pub markup_type: String,
    pub percentage: f64,
    pub fixed_amount: f64,
    pub apply_to: String,
    pub sort_order: i32,
    pub is_active: Option<bool>,
}

/// DTO for updating a markup.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMarkup {
    pub name: Option<String>,
    pub markup_type: Option<String>,
    pub percentage: Option<f64>,
    pub fixed_amount: Option<f64>,
    pub apply_to: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

/// BOQ with all positions loaded.
#[derive(Debug, Clone, Serialize)]
pub struct BoqWithPositions {
    pub boq: Boq,
    pub positions: Vec<Position>,
}

/// Grand total calculation result.
#[derive(Debug, Clone, Serialize)]
pub struct GrandTotal {
    pub direct_cost: f64,
    pub markups: Vec<erp_core::markup::MarkupResult>,
    pub grand_total: f64,
}
