#!/usr/bin/env python3
"""Extract text and tables from a PDF file using pdfplumber.
Usage: python3 extract_pdf.py <input.pdf> [output.json]
Outputs JSON: { "pages": [{ "page": 1, "text": "...", "tables": [[["cell", ...], ...], ...] }], "total_pages": N }
"""
import sys, json, os

try:
    import pdfplumber
except ImportError:
    print(json.dumps({"error": "pdfplumber not installed. Run: pip3 install pdfplumber"}), file=sys.stderr)
    sys.exit(1)

def extract(pdf_path: str) -> dict:
    pages = []
    with pdfplumber.open(pdf_path) as pdf:
        for p in pdf.pages:
            text = p.extract_text() or ""
            tables = p.extract_tables() or []
            # Clean table cells
            clean_tables = []
            for table in tables:
                clean_table = [[str(cell or "").strip() for cell in row] for row in table]
                clean_tables.append(clean_table)
            pages.append({
                "page": p.page_number,
                "text": text,
                "tables": clean_tables,
            })
    return {"pages": pages, "total_pages": len(pages)}

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 extract_pdf.py <input.pdf> [output.json]", file=sys.stderr)
        sys.exit(1)

    result = extract(sys.argv[1])

    if len(sys.argv) >= 3:
        with open(sys.argv[2], "w", encoding="utf-8") as f:
            json.dump(result, f, ensure_ascii=False, indent=2)
    else:
        print(json.dumps(result, ensure_ascii=False))
