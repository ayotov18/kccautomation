//! PDF extraction pipeline — extracts text and tables from PDF files
//! using a Python sidecar script (pdfplumber), then stores results
//! in the same normalized tables as DXF analysis for downstream KSS generation.

use anyhow::Result;
use uuid::Uuid;
use std::process::Command;

use crate::pipeline::WorkerContext;

/// Process a PDF file: download from S3, extract text/tables via Python, store in DB.
pub async fn process_pdf_extraction(
    drawing_id: Uuid,
    s3_key: &str,
    ctx: &WorkerContext,
) -> Result<()> {
    tracing::info!(%drawing_id, %s3_key, "Starting PDF extraction");

    // Download PDF from S3 to temp file
    let tmp_dir = std::env::temp_dir();
    let pdf_path = tmp_dir.join(format!("{}.pdf", drawing_id));
    let json_path = tmp_dir.join(format!("{}_pdf.json", drawing_id));

    let pdf_bytes = crate::pipeline::download_from_s3(&ctx.s3, &ctx.bucket, s3_key).await?;
    tokio::fs::write(&pdf_path, &pdf_bytes).await?;

    tracing::info!(%drawing_id, bytes = pdf_bytes.len(), "PDF downloaded, running extraction");

    // Run Python extractor as sidecar
    let output = tokio::task::spawn_blocking({
        let pdf_path = pdf_path.clone();
        let json_path = json_path.clone();
        move || {
            Command::new("python3")
                .arg("scripts/extract_pdf.py")
                .arg(&pdf_path)
                .arg(&json_path)
                .output()
        }
    })
    .await??;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!(%drawing_id, stderr = %stderr, "PDF extraction failed");
        // Cleanup
        let _ = tokio::fs::remove_file(&pdf_path).await;
        return Err(anyhow::anyhow!("PDF extraction failed: {}", stderr));
    }

    // Read extracted JSON
    let json_bytes = tokio::fs::read(&json_path).await?;
    let extracted: serde_json::Value = serde_json::from_slice(&json_bytes)?;

    // Cleanup temp files
    let _ = tokio::fs::remove_file(&pdf_path).await;
    let _ = tokio::fs::remove_file(&json_path).await;

    let total_pages = extracted.get("total_pages").and_then(|v| v.as_u64()).unwrap_or(0);
    tracing::info!(%drawing_id, total_pages, "PDF extracted, storing in DB");

    // Store extracted text as annotations
    if let Some(pages) = extracted.get("pages").and_then(|p| p.as_array()) {
        for page in pages {
            let page_num = page.get("page").and_then(|p| p.as_u64()).unwrap_or(0);
            let text = page.get("text").and_then(|t| t.as_str()).unwrap_or("");

            if !text.is_empty() {
                sqlx::query(
                    "INSERT INTO drawing_annotations (drawing_id, text, layer) VALUES ($1, $2, $3)"
                )
                .bind(drawing_id)
                .bind(text)
                .bind(format!("PDF_Page_{}", page_num))
                .execute(&ctx.db)
                .await?;
            }

            // Store table data as dimensions (parsed quantities)
            if let Some(tables) = page.get("tables").and_then(|t| t.as_array()) {
                for table in tables {
                    if let Some(rows) = table.as_array() {
                        for row in rows.iter().skip(1) {
                            // Try to extract numeric values as dimensions
                            if let Some(cells) = row.as_array() {
                                for cell in cells {
                                    if let Some(s) = cell.as_str() {
                                        if let Ok(val) = s.replace(',', ".").parse::<f64>() {
                                            if val > 0.0 && val < 100000.0 {
                                                sqlx::query(
                                                    "INSERT INTO drawing_dimensions (drawing_id, value, dim_type) VALUES ($1, $2, 'pdf_table')"
                                                )
                                                .bind(drawing_id)
                                                .bind(val)
                                                .execute(&ctx.db)
                                                .await?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Update drawing metadata
    sqlx::query("UPDATE drawings SET entity_count = $1 WHERE id = $2")
        .bind(total_pages as i32)
        .execute(&ctx.db)
        .await?;

    // Store the full extracted JSON in S3 for deep analysis
    let analysis_key = format!("reports/{}/pdf_analysis.json", drawing_id);
    crate::pipeline::upload_to_s3(&ctx.s3, &ctx.bucket, &analysis_key, &json_bytes).await?;

    tracing::info!(%drawing_id, "PDF extraction complete — data stored in DB");
    Ok(())
}
