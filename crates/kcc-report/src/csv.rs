use kcc_core::AnalysisResult;

#[derive(Debug, thiserror::Error)]
pub enum CsvError {
    #[error("CSV write error: {0}")]
    Write(#[from] csv::Error),
    #[error("into inner error: {0}")]
    IntoInner(String),
}

impl From<csv::IntoInnerError<csv::Writer<Vec<u8>>>> for CsvError {
    fn from(e: csv::IntoInnerError<csv::Writer<Vec<u8>>>) -> Self {
        CsvError::IntoInner(e.to_string())
    }
}

/// Generate a CSV report from analysis results.
pub fn generate_csv_report(result: &AnalysisResult) -> Result<Vec<u8>, CsvError> {
    let mut wtr = csv::Writer::from_writer(Vec::new());

    wtr.write_record([
        "Feature ID",
        "Type",
        "Description",
        "Classification",
        "Score",
        "Centroid X",
        "Centroid Y",
        "Datums",
        "Factor Count",
    ])?;

    for (feature, (_, score)) in result.features.iter().zip(result.kcc_results.iter()) {
        let datums: String = feature.datum_refs.iter().collect();

        wtr.write_record([
            &format!("F-{:03}", feature.id.0),
            feature.feature_type.name(),
            &feature.description(),
            score.classification.as_str(),
            &score.total.to_string(),
            &format!("{:.2}", feature.centroid.x),
            &format!("{:.2}", feature.centroid.y),
            &datums,
            &score.factors.len().to_string(),
        ])?;
    }

    Ok(wtr.into_inner()?)
}
