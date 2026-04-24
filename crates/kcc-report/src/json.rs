use chrono::Utc;
use kcc_core::AnalysisResult;
use kcc_core::kcc::types::KccClassification;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Serialize)]
pub struct JsonReport {
    pub drawing_id: Uuid,
    pub filename: String,
    pub generated_at: String,
    pub summary: ReportSummary,
    pub features: Vec<FeatureReport>,
}

#[derive(Serialize)]
pub struct ReportSummary {
    pub total_features: usize,
    pub kcc_count: usize,
    pub important_count: usize,
    pub standard_count: usize,
}

#[derive(Serialize)]
pub struct FeatureReport {
    pub feature_id: String,
    pub feature_type: String,
    pub description: String,
    pub centroid: (f64, f64),
    pub classification: String,
    pub score: u32,
    pub factors: Vec<FactorReport>,
}

#[derive(Serialize)]
pub struct FactorReport {
    pub name: String,
    pub points: u32,
    pub reason: String,
}

/// Generate a JSON report from analysis results.
pub fn generate_json_report(
    result: &AnalysisResult,
    drawing_id: Uuid,
) -> Result<Vec<u8>, ReportError> {
    let mut kcc_count = 0;
    let mut important_count = 0;
    let mut standard_count = 0;

    let features: Vec<FeatureReport> = result
        .features
        .iter()
        .zip(result.kcc_results.iter())
        .map(|(feature, (_, score))| {
            match score.classification {
                KccClassification::Kcc => kcc_count += 1,
                KccClassification::Important => important_count += 1,
                KccClassification::Standard => standard_count += 1,
            }

            FeatureReport {
                feature_id: format!("F-{:03}", feature.id.0),
                feature_type: feature.feature_type.name().to_string(),
                description: feature.description(),
                centroid: (feature.centroid.x, feature.centroid.y),
                classification: score.classification.as_str().to_string(),
                score: score.total,
                factors: score
                    .factors
                    .iter()
                    .map(|f| FactorReport {
                        name: f.name.clone(),
                        points: f.points,
                        reason: f.reason.clone(),
                    })
                    .collect(),
            }
        })
        .collect();

    let report = JsonReport {
        drawing_id,
        filename: result.drawing.metadata.filename.clone(),
        generated_at: Utc::now().to_rfc3339(),
        summary: ReportSummary {
            total_features: features.len(),
            kcc_count,
            important_count,
            standard_count,
        },
        features,
    };

    Ok(serde_json::to_vec_pretty(&report)?)
}
