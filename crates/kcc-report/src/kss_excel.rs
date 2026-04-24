use kcc_core::kss::types::{KssReport, SectionedKssReport};
use rust_xlsxwriter::{Format, Workbook, XlsxError};

/// Generate a KSS Excel report from the legacy flat format.
pub fn generate_kss_excel(report: &KssReport) -> Result<Vec<u8>, KssExcelError> {
    let sectioned = SectionedKssReport::from_items(
        &report.drawing_name,
        &report.generated_at,
        report.items.clone(),
        0.20,
    );
    generate_sectioned_kss_excel(&sectioned)
}

/// Generate a KSS Excel report matching real Bulgarian КСС format:
/// Columns: № | НАИМЕНОВАНИЕ | М-КА | К-ВО | ЕД. ЦЕНА | ЦЕНА МАТЕРИАЛИ | ЦЕНА ТРУД | ОБЩО:
/// Overhead stack: Admin 10%, Contingency 10%, Delivery 8%, Profit 30%, then VAT 20%
pub fn generate_sectioned_kss_excel(report: &SectionedKssReport) -> Result<Vec<u8>, KssExcelError> {
    let mut workbook = Workbook::new();

    // Sheet 1 — Рекапитулация (summary by section, per real Bulgarian
    // tender format — canonical pattern seen in naas_kss_obobshtena_nzzs.xls).
    {
        let recap = workbook.add_worksheet();
        recap.set_name("Рекапитулация")?;
        recap.set_column_width(0, 6)?;
        recap.set_column_width(1, 50)?;
        recap.set_column_width(2, 16)?;

        let title_fmt = Format::new().set_bold().set_font_size(14);
        let header_fmt = Format::new().set_bold().set_font_size(10);
        let money_fmt = Format::new().set_num_format("#,##0.00");
        let total_fmt = Format::new().set_bold().set_num_format("#,##0.00");
        let grand_fmt = Format::new().set_bold().set_font_size(12).set_num_format("#,##0.00");

        recap.write_string_with_format(0, 0, "РЕКАПИТУЛАЦИЯ", &title_fmt)?;
        recap.write_string(1, 0, &format!("ОБЕКТ: {}", report.project_name))?;
        recap.write_string(2, 0, &format!("ДАТА: {}", report.generated_at))?;

        recap.write_string_with_format(4, 0, "№", &header_fmt)?;
        recap.write_string_with_format(4, 1, "Секция", &header_fmt)?;
        recap.write_string_with_format(4, 2, "Стойност (€)", &header_fmt)?;

        let mut r: u32 = 5;
        for section in &report.sections {
            if section.items.is_empty() { continue; }
            recap.write_string(r, 0, &section.number)?;
            recap.write_string(r, 1, &section.title_bg)?;
            recap.write_number_with_format(r, 2, section.section_total_bgn, &money_fmt)?;
            r += 1;
        }
        r += 1;
        recap.write_string_with_format(r, 1, "ОБЩО СМР без ДДС", &total_fmt)?;
        recap.write_number_with_format(r, 2, report.subtotal_bgn, &total_fmt)?;
        r += 1;
        recap.write_string_with_format(r, 1, "ДДС", &total_fmt)?;
        recap.write_number_with_format(r, 2, report.vat_bgn, &total_fmt)?;
        r += 1;
        recap.write_string_with_format(r, 1, "ОБЩО С ДДС", &grand_fmt)?;
        recap.write_number_with_format(r, 2, report.total_with_vat_bgn, &grand_fmt)?;
    }

    let sheet = workbook.add_worksheet();
    sheet.set_name("КСС")?;

    // Column widths matching real Bulgarian КСС
    sheet.set_column_width(0, 8)?;    // №
    sheet.set_column_width(1, 55)?;   // НАИМЕНОВАНИЕ
    sheet.set_column_width(2, 8)?;    // М-КА
    sheet.set_column_width(3, 10)?;   // К-ВО
    sheet.set_column_width(4, 12)?;   // ЕД. ЦЕНА
    sheet.set_column_width(5, 16)?;   // ЦЕНА МАТЕРИАЛИ
    sheet.set_column_width(6, 14)?;   // ЦЕНА ТРУД
    sheet.set_column_width(7, 16)?;   // ОБЩО:

    // Formats
    let title_fmt = Format::new().set_bold().set_font_size(14);
    let subtitle_fmt = Format::new().set_font_size(10);
    let header_fmt = Format::new().set_bold().set_font_size(10).set_text_wrap();
    let section_fmt = Format::new().set_bold().set_font_size(11);
    let money_fmt = Format::new().set_num_format("#,##0.00");
    let qty_fmt = Format::new().set_num_format("#,##0.00");
    let section_total_fmt = Format::new().set_bold().set_italic().set_num_format("#,##0.00");
    let overhead_label_fmt = Format::new().set_italic().set_font_size(10);
    let overhead_value_fmt = Format::new().set_italic().set_num_format("#,##0.00");
    let grand_total_fmt = Format::new().set_bold().set_font_size(11).set_num_format("#,##0.00");
    let grand_total_label_fmt = Format::new().set_bold().set_font_size(11);

    // Title block
    sheet.write_string_with_format(0, 0, "КОЛИЧЕСТВЕНО-СТОЙНОСТНА СМЕТКА", &title_fmt)?;
    sheet.write_string_with_format(1, 0, &format!("ОБЕКТ: {}", report.project_name), &subtitle_fmt)?;
    sheet.write_string_with_format(2, 0, &format!("ДАТА: {}", report.generated_at), &subtitle_fmt)?;

    // Header row — matching real КСС format
    let mut row: u32 = 4;
    let headers = ["№", "НАИМЕНОВАНИЕ", "М-КА", "К-ВО", "ЕД. ЦЕНА", "ЦЕНА МАТЕРИАЛИ", "ЦЕНА ТРУД", "ОБЩО:"];
    for (col, header) in headers.iter().enumerate() {
        sheet.write_string_with_format(row, col as u16, *header, &header_fmt)?;
    }
    row += 1;

    // Sections with items
    for section in &report.sections {
        if section.items.is_empty() {
            continue;
        }

        // Section header with material/labor/total subtotals
        row += 1;
        let mat_total: f64 = section.items.iter().map(|i| i.material_price * i.quantity).sum();
        let lab_total: f64 = section.items.iter().map(|i| i.labor_price * i.quantity).sum();

        sheet.write_string_with_format(row, 0, &section.number, &section_fmt)?;
        sheet.write_string_with_format(row, 1, &section.title_bg, &section_fmt)?;
        sheet.write_number_with_format(row, 5, mat_total, &section_total_fmt)?;
        sheet.write_number_with_format(row, 6, lab_total, &section_total_fmt)?;
        sheet.write_number_with_format(row, 7, section.section_total_bgn, &section_total_fmt)?;
        row += 1;

        // Section items
        for item in &section.items {
            let ordinal = format!("{}.{}.", section.number, item.item_no);
            sheet.write_string(row, 0, &ordinal)?;
            sheet.write_string(row, 1, &item.description)?;
            sheet.write_string(row, 2, &item.unit)?;
            sheet.write_number_with_format(row, 3, item.quantity, &qty_fmt)?;
            // Unit price
            let unit_price = if item.quantity > 0.0 { item.total_price / item.quantity } else { 0.0 };
            sheet.write_number_with_format(row, 4, unit_price, &money_fmt)?;
            // Material total
            sheet.write_number_with_format(row, 5, item.material_price * item.quantity, &money_fmt)?;
            // Labor total
            sheet.write_number_with_format(row, 6, item.labor_price * item.quantity, &money_fmt)?;
            // Total
            sheet.write_number_with_format(row, 7, item.total_price, &money_fmt)?;
            row += 1;
        }
    }

    // Grand total
    row += 1;
    let direct_cost = report.subtotal_bgn;
    sheet.write_string_with_format(row, 1, "ОБЩО:", &grand_total_label_fmt)?;
    sheet.write_number_with_format(row, 7, direct_cost, &grand_total_fmt)?;
    row += 1;

    // Overhead stack (matching real Bulgarian КСС)
    let admin = direct_cost * 0.10;
    let contingency = direct_cost * 0.10;
    let delivery = direct_cost * 0.08;
    let profit = direct_cost * 0.30;
    let total_before_vat = direct_cost + admin + contingency + delivery + profit;
    let vat = total_before_vat * 0.20;
    let grand_total = total_before_vat + vat;

    sheet.write_string_with_format(row, 1, "Административни разходи 10%:", &overhead_label_fmt)?;
    sheet.write_number_with_format(row, 7, admin, &overhead_value_fmt)?;
    row += 1;

    sheet.write_string_with_format(row, 1, "Непредвидени разходи 10%:", &overhead_label_fmt)?;
    sheet.write_number_with_format(row, 7, contingency, &overhead_value_fmt)?;
    row += 1;

    sheet.write_string_with_format(row, 1, "Доставно складови разходи 8%:", &overhead_label_fmt)?;
    sheet.write_number_with_format(row, 7, delivery, &overhead_value_fmt)?;
    row += 1;

    sheet.write_string_with_format(row, 1, "Печалба 30%:", &overhead_label_fmt)?;
    sheet.write_number_with_format(row, 7, profit, &overhead_value_fmt)?;
    row += 1;

    sheet.write_string_with_format(row, 1, "ОБЩО ЗА ОБЕКТА:", &grand_total_label_fmt)?;
    sheet.write_number_with_format(row, 7, total_before_vat, &grand_total_fmt)?;
    row += 1;

    sheet.write_string_with_format(row, 1, "ДДС 20%:", &overhead_label_fmt)?;
    sheet.write_number_with_format(row, 7, vat, &overhead_value_fmt)?;
    row += 1;

    sheet.write_string_with_format(row, 1, "ОБЩО ЗА ОБЕКТА С ДДС 20%:", &grand_total_label_fmt)?;
    sheet.write_number_with_format(row, 7, grand_total, &grand_total_fmt)?;

    let buf = workbook.save_to_buffer()?;
    Ok(buf)
}

#[derive(Debug, thiserror::Error)]
pub enum KssExcelError {
    #[error("Excel generation error: {0}")]
    Xlsx(#[from] XlsxError),
}
