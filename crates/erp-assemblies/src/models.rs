use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An assembly recipe (composite work item made of components).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Assembly {
    pub id: Uuid,
    pub project_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub formula: Option<String>,
    pub total_rate: f64,
    pub category: Option<String>,
    pub is_template: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A component within an assembly.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Component {
    pub id: Uuid,
    pub assembly_id: Uuid,
    pub cost_item_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub quantity: f64,
    pub unit_cost: f64,
    pub factor: f64,
    pub total: f64,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating an assembly.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAssembly {
    pub project_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub formula: Option<String>,
    pub category: Option<String>,
    pub is_template: Option<bool>,
}

/// DTO for adding a component to an assembly.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateComponent {
    pub cost_item_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub quantity: f64,
    pub unit_cost: f64,
    pub factor: Option<f64>,
    pub sort_order: Option<i32>,
}

/// DTO for updating a component.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateComponent {
    pub cost_item_id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub quantity: Option<f64>,
    pub unit_cost: Option<f64>,
    pub factor: Option<f64>,
    pub sort_order: Option<i32>,
}

/// An assembly with all its components.
#[derive(Debug, Clone, Serialize)]
pub struct AssemblyWithComponents {
    pub assembly: Assembly,
    pub components: Vec<Component>,
}
