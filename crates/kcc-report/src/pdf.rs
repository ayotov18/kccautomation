use chrono::Utc;
use kcc_core::AnalysisResult;
use kcc_core::kcc::types::KccClassification;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("PDF generation error: {0}")]
    Generation(String),
}

/// Generate a minimal PDF report from analysis results.
///
/// For the MVP, this generates a simple text-based PDF.
/// A full implementation would use printpdf with embedded fonts.
pub fn generate_pdf_report(result: &AnalysisResult, drawing_id: Uuid) -> Result<Vec<u8>, PdfError> {
    // Count classifications
    let mut kcc_count = 0usize;
    let mut important_count = 0usize;
    let mut standard_count = 0usize;
    for (_, score) in &result.kcc_results {
        match score.classification {
            KccClassification::Kcc => kcc_count += 1,
            KccClassification::Important => important_count += 1,
            KccClassification::Standard => standard_count += 1,
        }
    }

    // Build feature lines
    let mut feature_lines = Vec::new();
    for (feature, (_, score)) in result.features.iter().zip(result.kcc_results.iter()) {
        let desc = feature.description();
        let desc_short = if desc.len() > 40 {
            format!("{}...", &desc[..37])
        } else {
            desc
        };
        feature_lines.push(format!(
            "F-{:03}  {:12}  {:40}  {:3}  {}",
            feature.id.0,
            feature.feature_type.name(),
            desc_short,
            score.total,
            score.classification.as_str(),
        ));
    }

    // Generate a simple PDF with raw PDF operators
    let params = PdfParams {
        filename: &result.drawing.metadata.filename,
        report_id: &drawing_id.to_string(),
        generated_at: &Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        total_features: result.features.len(),
        kcc_count,
        important_count,
        standard_count,
        feature_lines: &feature_lines,
    };
    let content = build_pdf_content(&params);

    Ok(content)
}

struct PdfParams<'a> {
    filename: &'a str,
    report_id: &'a str,
    generated_at: &'a str,
    total_features: usize,
    kcc_count: usize,
    important_count: usize,
    standard_count: usize,
    feature_lines: &'a [String],
}

/// Build a raw PDF file with Helvetica (built-in PDF font, no embedding needed).
fn build_pdf_content(p: &PdfParams<'_>) -> Vec<u8> {
    let mut lines: Vec<String> = Vec::new();

    // Text stream content
    lines.push("BT".to_string());
    lines.push("/F1 16 Tf".to_string());
    lines.push("56.7 800 Td".to_string());
    lines.push("(KCC Analysis Report) Tj".to_string());

    lines.push("/F1 10 Tf".to_string());
    lines.push("0 -20 Td".to_string());
    lines.push(format!("(Drawing: {}) Tj", pdf_escape(p.filename)));

    lines.push("0 -14 Td".to_string());
    lines.push(format!("(Report ID: {}) Tj", pdf_escape(p.report_id)));

    lines.push("0 -14 Td".to_string());
    lines.push(format!("(Generated: {}) Tj", pdf_escape(p.generated_at)));

    lines.push("0 -24 Td".to_string());
    lines.push("/F1 12 Tf".to_string());
    lines.push("(Summary) Tj".to_string());

    lines.push("/F1 9 Tf".to_string());
    lines.push("0 -16 Td".to_string());
    lines.push(format!("(Total Features: {}) Tj", p.total_features));

    lines.push("0 -14 Td".to_string());
    lines.push(format!(
        "(KCC: {}  |  Important: {}  |  Standard: {}) Tj",
        p.kcc_count, p.important_count, p.standard_count
    ));

    lines.push("0 -24 Td".to_string());
    lines.push("/F1 12 Tf".to_string());
    lines.push("(Features) Tj".to_string());

    lines.push("/F1 7 Tf".to_string());
    lines.push("0 -14 Td".to_string());
    lines.push(format!(
        "({:6}  {:12}  {:40}  {:5}  {}) Tj",
        "ID", "Type", "Description", "Score", "Class"
    ));

    for line in p.feature_lines.iter().take(50) {
        // Limit to 50 features per page for MVP
        lines.push("0 -11 Td".to_string());
        lines.push(format!("({}) Tj", pdf_escape(line)));
    }

    lines.push("ET".to_string());

    let stream_content = lines.join("\n");
    let stream_bytes = stream_content.as_bytes();

    // Build minimal PDF structure
    let mut pdf = String::new();
    pdf.push_str("%PDF-1.4\n");

    // Object 1: Catalog
    let obj1 = "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n";
    let obj1_offset = pdf.len();
    pdf.push_str(obj1);

    // Object 2: Pages
    let obj2 = "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n";
    let obj2_offset = pdf.len();
    pdf.push_str(obj2);

    // Object 3: Page
    let obj3 = "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595.28 841.89] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n";
    let obj3_offset = pdf.len();
    pdf.push_str(obj3);

    // Object 4: Content stream
    let obj4 = format!(
        "4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        stream_bytes.len(),
        stream_content,
    );
    let obj4_offset = pdf.len();
    pdf.push_str(&obj4);

    // Object 5: Font (Helvetica - built-in)
    let obj5 = "5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n";
    let obj5_offset = pdf.len();
    pdf.push_str(obj5);

    // Cross-reference table
    let xref_offset = pdf.len();
    pdf.push_str("xref\n");
    pdf.push_str("0 6\n");
    pdf.push_str("0000000000 65535 f \n");
    pdf.push_str(&format!("{:010} 00000 n \n", obj1_offset));
    pdf.push_str(&format!("{:010} 00000 n \n", obj2_offset));
    pdf.push_str(&format!("{:010} 00000 n \n", obj3_offset));
    pdf.push_str(&format!("{:010} 00000 n \n", obj4_offset));
    pdf.push_str(&format!("{:010} 00000 n \n", obj5_offset));

    // Trailer
    pdf.push_str("trailer\n");
    pdf.push_str("<< /Size 6 /Root 1 0 R >>\n");
    pdf.push_str("startxref\n");
    pdf.push_str(&format!("{}\n", xref_offset));
    pdf.push_str("%%EOF\n");

    pdf.into_bytes()
}

/// Escape special characters for PDF string.
fn pdf_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}
