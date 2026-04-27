#[cfg(feature = "scraping")]
pub mod ai;
pub mod datum;
pub mod dimension;
pub mod drawing_type;
pub mod drm;
pub mod fabrication;
pub mod feature;
pub mod gdt;
pub mod geometry;
pub mod kcc;
pub mod kss;
#[cfg(feature = "scraping")]
pub mod price_corpus;
#[cfg(feature = "scraping")]
pub mod scraper;
#[cfg(feature = "scraping")]
pub mod quantity_scraper;
pub mod tolerance_chain;

use datum::types::DatumInfo;
use feature::types::Feature;
use geometry::model::Drawing;
use geometry::spatial::SpatialIndex;
use kcc::config::KccConfig;
use kcc::types::KccScore;
use tolerance_chain::types::ToleranceChain;

/// Complete analysis result from processing a drawing.
/// This is the canonical artifact: persist it to S3 and let every downstream
/// consumer (viewer, KSS, reports) read from the same truth.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisResult {
    pub drawing: Drawing,
    pub features: Vec<Feature>,
    pub kcc_results: Vec<(u64, KccScore)>,
    pub tolerance_chains: Vec<ToleranceChain>,
    pub datums: Vec<DatumInfo>,
}

/// Run the complete analysis pipeline on a parsed drawing.
pub fn analyze_drawing(mut drawing: Drawing, config: &KccConfig) -> AnalysisResult {
    // 1. Build spatial index
    let index = SpatialIndex::build(&drawing.entities);

    // 2. Link dimensions to geometry
    dimension::resolver::link_dimensions_to_geometry(&mut drawing, &index);

    // 3. Extract features
    let feature_set = feature::extract_features(&drawing, &index);

    // 4. Parse and link GD&T
    // (GD&T is already parsed during DXF ingestion, linking happens here)
    gdt::linker::link_gdt_to_features(&drawing, &feature_set.features, &index);

    // 5. Extract datums
    let datums = datum::extractor::extract_datums(&drawing, &index);

    // 6. Build tolerance chains
    let chains = tolerance_chain::analyzer::analyze_chains(
        &feature_set.features,
        &drawing.dimensions,
        &datums,
    );

    // 7. Classify all features
    let context = kcc::scorer::ScoringContext {
        drawing: &drawing,
        features: &feature_set.features,
        chains: &chains,
        datums: &datums,
        config,
    };
    let kcc_results = kcc::scorer::classify_all(&feature_set.features, &context);

    AnalysisResult {
        drawing,
        features: feature_set.features,
        kcc_results,
        tolerance_chains: chains,
        datums,
    }
}
