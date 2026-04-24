use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeDrawingJob {
    pub job_id: Uuid,
    pub drawing_id: Uuid,
    pub s3_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateKssJob {
    pub job_id: Uuid,
    pub drawing_id: Uuid,
    pub price_list_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepAnalyzeJob {
    pub job_id: Uuid,
    pub drawing_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeJob {
    pub job_id: Uuid,
    pub user_id: Uuid,
    pub source_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityScrapeJob {
    pub job_id: Uuid,
    pub user_id: Uuid,
    pub source_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiKssJob {
    pub job_id: Uuid,
    pub drawing_id: Uuid,
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub phase: String,  // "research" | "generate"
}
