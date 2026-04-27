use kcc_core::kss::types::KssReport;

/// Generate a KSS PDF summary report.
/// Note: PDF Type1 fonts (Helvetica) don't support Cyrillic,
/// so we transliterate Bulgarian text to Latin for PDF output.
pub fn generate_kss_pdf(report: &KssReport) -> Result<Vec<u8>, KssPdfError> {
    let mut lines: Vec<String> = Vec::new();

    lines.push("BT".into());
    lines.push("/F1 16 Tf".into());
    lines.push("56.7 800 Td".into());
    lines.push("(KSS - Bill of Quantities) Tj".into());

    lines.push("/F1 10 Tf".into());
    lines.push("0 -20 Td".into());
    lines.push(format!("(Drawing: {}) Tj", pdf_escape(&report.drawing_name)));
    lines.push("0 -14 Td".into());
    lines.push(format!("(Generated: {}) Tj", pdf_escape(&report.generated_at)));

    lines.push("0 -24 Td".into());
    lines.push("/F1 12 Tf".into());
    lines.push("(Summary) Tj".into());

    lines.push("/F1 9 Tf".into());
    lines.push("0 -16 Td".into());
    lines.push(format!("(Total Items: {}) Tj", report.items.len()));
    lines.push("0 -14 Td".into());
    lines.push(format!(
        "(Grand Total: {:.2} EUR) Tj",
        report.totals.grand_total
    ));
    lines.push("0 -14 Td".into());
    lines.push(format!(
        "(Labor: {:.2} | Material: {:.2} | Mech: {:.2} | Overhead: {:.2}) Tj",
        report.totals.labor,
        report.totals.material,
        report.totals.mechanization,
        report.totals.overhead,
    ));

    lines.push("0 -24 Td".into());
    lines.push("/F1 12 Tf".into());
    lines.push("(Items) Tj".into());

    lines.push("/F1 7 Tf".into());
    lines.push("0 -14 Td".into());
    lines.push(format!(
        "({:4}  {:8}  {:35}  {:6}  {:>10}  {:>10}) Tj",
        "No.", "SEK", "Description", "Unit", "Qty", "Total"
    ));

    for item in report.items.iter().take(40) {
        // Transliterate Cyrillic to Latin for PDF compatibility
        let desc_latin = transliterate_bg(&item.description);
        let desc = {
            let chars: Vec<char> = desc_latin.chars().collect();
            if chars.len() > 32 {
                format!("{}...", chars[..29].iter().collect::<String>())
            } else {
                desc_latin
            }
        };
        let unit_latin = transliterate_bg(&item.unit);
        lines.push("0 -11 Td".into());
        lines.push(format!(
            "({:4}  {:8}  {:35}  {:6}  {:>10.2}  {:>10.2}) Tj",
            item.item_no,
            pdf_escape(&item.sek_code),
            pdf_escape(&desc),
            pdf_escape(&unit_latin),
            item.quantity,
            item.total_price,
        ));
    }

    lines.push("ET".into());

    let stream_content = lines.join("\n");
    let stream_bytes = stream_content.as_bytes();

    let mut pdf = String::new();
    pdf.push_str("%PDF-1.4\n");

    let obj1 = "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n";
    let obj1_offset = pdf.len();
    pdf.push_str(obj1);

    let obj2 = "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n";
    let obj2_offset = pdf.len();
    pdf.push_str(obj2);

    let obj3 = "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595.28 841.89] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n";
    let obj3_offset = pdf.len();
    pdf.push_str(obj3);

    let obj4 = format!(
        "4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        stream_bytes.len(),
        stream_content,
    );
    let obj4_offset = pdf.len();
    pdf.push_str(&obj4);

    let obj5 = "5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n";
    let obj5_offset = pdf.len();
    pdf.push_str(obj5);

    let xref_offset = pdf.len();
    pdf.push_str("xref\n0 6\n");
    pdf.push_str("0000000000 65535 f \n");
    pdf.push_str(&format!("{:010} 00000 n \n", obj1_offset));
    pdf.push_str(&format!("{:010} 00000 n \n", obj2_offset));
    pdf.push_str(&format!("{:010} 00000 n \n", obj3_offset));
    pdf.push_str(&format!("{:010} 00000 n \n", obj4_offset));
    pdf.push_str(&format!("{:010} 00000 n \n", obj5_offset));

    pdf.push_str("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n");
    pdf.push_str(&format!("{}\n%%EOF\n", xref_offset));

    Ok(pdf.into_bytes())
}

fn pdf_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

/// Transliterate Bulgarian Cyrillic to Latin for PDF Type1 font compatibility.
fn transliterate_bg(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            match c {
                'А' => vec!['A'], 'Б' => vec!['B'], 'В' => vec!['V'], 'Г' => vec!['G'],
                'Д' => vec!['D'], 'Е' => vec!['E'], 'Ж' => vec!['Z', 'h'],
                'З' => vec!['Z'], 'И' => vec!['I'], 'Й' => vec!['Y'],
                'К' => vec!['K'], 'Л' => vec!['L'], 'М' => vec!['M'], 'Н' => vec!['N'],
                'О' => vec!['O'], 'П' => vec!['P'], 'Р' => vec!['R'], 'С' => vec!['S'],
                'Т' => vec!['T'], 'У' => vec!['U'], 'Ф' => vec!['F'], 'Х' => vec!['H'],
                'Ц' => vec!['T', 's'], 'Ч' => vec!['C', 'h'],
                'Ш' => vec!['S', 'h'], 'Щ' => vec!['S', 'h', 't'],
                'Ъ' => vec!['A'], 'Ь' => vec!['Y'], 'Ю' => vec!['Y', 'u'],
                'Я' => vec!['Y', 'a'],
                'а' => vec!['a'], 'б' => vec!['b'], 'в' => vec!['v'], 'г' => vec!['g'],
                'д' => vec!['d'], 'е' => vec!['e'], 'ж' => vec!['z', 'h'],
                'з' => vec!['z'], 'и' => vec!['i'], 'й' => vec!['y'],
                'к' => vec!['k'], 'л' => vec!['l'], 'м' => vec!['m'], 'н' => vec!['n'],
                'о' => vec!['o'], 'п' => vec!['p'], 'р' => vec!['r'], 'с' => vec!['s'],
                'т' => vec!['t'], 'у' => vec!['u'], 'ф' => vec!['f'], 'х' => vec!['h'],
                'ц' => vec!['t', 's'], 'ч' => vec!['c', 'h'],
                'ш' => vec!['s', 'h'], 'щ' => vec!['s', 'h', 't'],
                'ъ' => vec!['a'], 'ь' => vec!['y'], 'ю' => vec!['y', 'u'],
                'я' => vec!['y', 'a'],
                // Keep ASCII and common symbols as-is
                c if c.is_ascii() => vec![c],
                // Replace other Unicode with '?'
                '²' => vec!['2'],
                _ => vec!['?'],
            }
        })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum KssPdfError {
    #[error("PDF generation error: {0}")]
    Generation(String),
}
