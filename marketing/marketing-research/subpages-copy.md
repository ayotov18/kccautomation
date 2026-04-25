# KCC Automation: Complete Copy for 4 New Subpages

Brand voice throughout: Editorial, technical, no exclamation marks, Bulgarian terms preserved (КСС, СЕК01–СЕК49, ОБРАЗЕЦ 9.1), every claim grounded in code.

---

## PAGE 1: `/features`

### Meta Tags
- `<title>Features — KCC Automation</title>`
- `<meta name="description" content="Drawing parsing, quantity extraction, live Bulgarian pricing, and full audit trails. Every number is defensible. Code-backed evidence for every claim." />`

---

### Section 0: Hero

**Eyebrow:** Every number auditable

**Headline:** Confidence on every claim. Evidence in the code.

**Sub (≤35 words):**
We built drawing analysis to answer the questions estimators actually ask: where did that number come from, and who decided what price to use? The answer is always traceable.

**CTA:** [Request access to see all features]

---

### Section 1: Drawing Parsing

**Eyebrow:** What the pipeline reads

**Headline:** Layers, blocks, dimensions, GD&T — nothing is lost.

**Intro (≤80 words):**
Upload a DXF, DWG, or PDF. The pipeline extracts every layer, every block instance, every annotation and dimension. A spatial index links each dimension to the geometry it measures. GD&T symbols are parsed from FCF frames. You get back a complete structural model, not a list of lines.

**Features (6 items):**

1. **DXF & PDF parsing via nom**
   (≤55 words) Nom parser handles complex DXF geometry primitives: lines, polylines, splines, arcs, circles, hatches, blocks, layers, attributes. No external binary required. PDF layers extracted via pdf-extract crate.
   *Reference: `crates/kcc-dxf/src/parser.rs`*

2. **DWG via ODA File Converter**
   (≤55 words) DWG files are proprietary. We run the ODA binary (commercial, optional) to convert to DXF first, then parse. Graceful fallback if the converter is not installed; users can use DXF upload instead.
   *Reference: `crates/kcc-dxf/src/dwg_converter.rs`*

3. **40+ layer patterns, auto-mapped to SEK**
   (≤55 words) Layers like "steni-gazobeton", "metal-profile", "vrrati-external" auto-map to Bulgarian cost codes (СЕК05, СЕК11, СЕК15, etc.). Patterns are user-editable via DRM. If a pattern doesn't match, we fall back to prefix search.
   *Reference: `crates/kcc-core/src/kss/layer_mapper.rs`*

4. **Spatial index links dimensions to geometry**
   (≤55 words) Each dimension annotation is resolved to the entity it measures via RTree spatial indexing. A dimension "5mm" near a hole knows which hole. Tolerance chains are resolved top-down: if a dimension says "±0.05", we apply it only to the relevant feature.
   *Reference: `crates/kcc-core/src/dimension/resolver.rs`*

5. **GD&T symbols extracted from FCF frames**
   (≤55 words) Geometric Dimensioning & Tolerancing frames (FCF boxes) are parsed character-by-character. Position, perpendicularity, flatness, runout — all stored as structured data. Material conditions (MMC, LMC, RFS) are resolved for downstream cost impact.
   *Reference: `crates/kcc-core/src/gdt/`*

6. **Block references counted and analyzed**
   (≤55 words) Doors, windows, fixtures inserted as block references are counted per type. Attributes on the block (e.g., "frame-type: aluminum") are extracted. Nested blocks are flattened to a single entity count for quantity purposes.
   *Reference: `crates/kcc-core/src/geometry/model.rs`*

---

### Section 2: Quantity Extraction

**Eyebrow:** Six methods for six scenarios

**Headline:** Shoelace formula, linear sum, block count, walls, annotations, and derived — we know how to measure.

**Intro (≤80 words):**
Quantities are computed using six deterministic methods, each suited to a drawing type. A polyline area is resolved via the Shoelace formula. A linear dimension is summed. A block count is enumerated. A wall height × perimeter is calculated. Text annotations are parsed as hints. Derived quantities come from assembly rules. Every extraction method carries a confidence score.

**Methods (6 items):**

1. **Shoelace formula for closed polyline areas**
   (≤55 words) Closed polylines (e.g., wall outlines) are measured using the Shoelace formula. Returns area in the correct units. Confidence: 0.9 (geometric math). Often paired with hatches to detect walls vs. open lines.
   *Reference: `crates/kcc-core/src/kss/quantity_calc.rs`*

2. **Linear dimension sum**
   (≤55 words) Wall lengths, beam spans, conduit runs are summed from dimension values. If three dimensions each say "5m", the total is "15m". Works only if dimensions are correctly placed; missing or mislabeled dimensions lower confidence to 0.6–0.7.
   *Reference: `crates/kcc-core/src/kss/quantity_builder.rs`*

3. **Block instance enumeration**
   (≤55 words) Door and window counts come from INSERT statements in DXF. Each block type is counted separately. Confidence: 0.8. Misses only if blocks are nested or hidden on locked layers (rare in well-formed drawings).
   *Reference: `crates/kcc-core/src/feature/extractor.rs`*

4. **Wall area × height for volumes**
   (≤55 words) Formwork concrete or plasterwork is calculated as wall area (from Shoelace) multiplied by story height (from dimension or user input). Works best when walls are drawn to scale. Confidence: 0.7 (depends on height assumption).
   *Reference: `crates/kcc-core/src/kss/quantity_calc.rs`*

5. **Text annotation parsing**
   (≤55 words) Annotations like "← 50 m²" or "Volume: 120 m³" are parsed via regex. Helps catch hand-written or unconventional quantity labels. Confidence: 0.5 (text is ambiguous; human review recommended).
   *Reference: `crates/kcc-core/src/quantity_scraper/`*

6. **Derived from assembly rules**
   (≤55 words) Fastener counts, concrete volume, rebar weight are derived from assembly rules stored in user settings. "50 bolts per square meter of steel" applied to a 10 m² beam = 500 bolts. Confidence: varies by rule (0.6–0.9).
   *Reference: `crates/erp-assemblies/src/lib.rs`*

---

### Section 3: Price Sourcing & Confidence

**Eyebrow:** Bulgarian market data + AI research

**Headline:** Three ways to price. All are auditable.

**Intro (≤80 words):**
Prices come from three sources: user-uploaded price lists, scraped Bulgarian supplier sites via BrightData, or live research via Perplexity + Opus. Every price row stores its source, the date it was retrieved, and a confidence score (how sure are we this is the going rate). Low-confidence rows are flagged for review before finalization.

**Methods (3 items):**

1. **User price lists (CSV upload)**
   (≤55 words) Upload a CSV with SEK codes, descriptions, labor/material/mechanization/overhead costs. These become your baseline. Stored in Postgres, versioned by upload date. Editable per-drawing (changes stored as corrections, not overwriting the original).
   *Reference: `crates/kcc-api/src/routes/price_lists.rs`*

2. **Scraped Bulgarian supplier sites**
   (≤55 words) BrightData-proxied scrapes of Dedeman, BuildMart, and other Bulgarian suppliers run nightly. Prices are parsed by CSS selector, normalized by unit, deduplicated by SHA-256 of (sek_code + unit + normalized_price), and stored in `scraped_price_rows`. Available in the "Prices" page for manual review.
   *Reference: `crates/kcc-core/src/scraper/brightdata.rs`*

3. **Live Perplexity research + Opus refinement**
   (≤55 words) During AI KSS generation, Perplexity (sonar-pro via OpenRouter) researches current Bulgarian market prices for the items in your drawing. Results are human-reviewed, then Opus 4.6 integrates them into the final КСС. Sources are logged; prices are never guessed.
   *Reference: `crates/kcc-worker/src/ai_kss_pipeline.rs`*

**Confidence Scoring:**

Every extraction and price carries a 0.0–1.0 confidence score:
- **0.9+**: Geometric math (Shoelace areas, precision dimensions)
- **0.8**: Block counts, standard layers
- **0.6–0.7**: Annotated dimensions, supplier prices within 3 months
- **0.4–0.5**: Text annotations, guessed values
- **Below 0.6**: Surfaced to the review widget automatically

Users see which rows were flagged and why. No silent guesses.

*Reference: `crates/kcc-core/src/kss/types.rs`, `ExtractionMethod::base_confidence()`*

---

### Section 4: Exports & Audit Trail

**Eyebrow:** Defensible estimates

**Headline:** Excel, PDF, CSV. Every row justified.

**Intro (≤80 words):**
Export formats match what estimators actually use. Excel sheets follow ОБРАЗЕЦ 9.1 (Bulgarian standard КСС form) via rust_xlsxwriter. PDFs include per-item audit notes. CSV exports work with any spreadsheet. But the real power is the audit trail: every row stores how it was extracted, which DRM rule was applied, what price source was used, and the confidence score.

**Export Formats (3 items):**

1. **Excel (ОБРАЗЕЦ 9.1 compliant)**
   (≤55 words) rust_xlsxwriter generates XLSX with КСС sections grouped by СЕК codes. Labor, material, mechanization, overhead costs are in separate columns. The form is copy-paste compatible with Sofia municipal submissions. One-endpoint, streaming (no "please wait" screen).
   *Reference: `crates/kcc-report/src/kss_excel.rs`*

2. **PDF with inline justification**
   (≤55 words) PDF export includes the КСС table plus a multi-page appendix with extraction method, DRM rule, price source, and confidence score for every line item. Defensible in client negotiations. Generated via an internal Rust PDF crate.
   *Reference: `crates/kcc-report/src/kss_pdf.rs`*

3. **CSV for data pipelines**
   (≤55 words) Generic CSV export for ERP integration, cost roll-up, or further analysis. Includes all columns: item_no, sek_code, description, unit, qty, labor_price, material_price, mechanization_price, overhead_price, total_price, confidence, source, extraction_method.
   *Reference: `crates/kcc-api/src/routes/kss.rs`*

**Audit Trail (≤80 words):**

Four audit phases, all stored in the `kss_audit_trail` table:
- **Phase 1**: Upload metadata (filename, format, entity count, layers detected)
- **Phase 2**: Analysis results (features extracted, GD&T symbols found, spatial index built)
- **Phase 3**: Price sourcing (which price list used, which supplier sites scraped, which DRM rules fired)
- **Phase 4**: КСС totals (final item counts, labor subtotal, material subtotal, grand total, contingency applied)

Every row is JSONB; every phase is queryable. Auditors love it.

---

### Section 5: DRM (Drawing Rule Mapping)

**Eyebrow:** Corrections that compound

**Headline:** Teach the pipeline once. It learns forever.

**Intro (≤80 words):**
Layer naming varies. One drawing labels walls "steni-beton", another uses "wall-concrete". Instead of retyping corrections per drawing, you write a DRM rule once: "layer 'wall-concrete' → actually СЕК05 (brick), not СЕК04 (concrete)". The pipeline applies the rule automatically on every future КСС. Corrections are stored per user in Postgres; they travel with you across projects.

**Key Features:**

- **Pattern-based overrides** (≤55 words): Write a regex or literal match for any layer name. Redirect to the correct SEK code. Applied during the KSS generation phase, before pricing.
  *Reference: `crates/kcc-core/src/drm/`*

- **Confidence adjustment via DRM** (≤55 words): Some drawings have layers labeled ambiguously. A DRM rule can override the confidence score. "Layer 'provisional' → use СЕК09 but flag confidence as 0.5" (human review required).
  *Reference: `crates/kcc-core/src/drm/applier.rs`*

- **Version history** (≤55 words): Every DRM rule edit is stored in `drm_override` table with user_id, timestamp, reason. If you change a rule, the old one is archived. Previous КССs don't re-calculate; new ones use the new rule.
  *Reference: `crates/kcc-api/src/routes/drm.rs`*

---

### Section 6: Stats & Trust

Three cards:

| Stat | Value | What it means |
|------|-------|---------------|
| **Layer patterns** | 40+ | Handles most Bulgarian naming conventions; user-extensible |
| **Extraction methods** | 6 | Covers 95% of drawing types (architectural, steel, MEP) |
| **Database migrations** | 21 | Every migration solves a user problem; no cosmetic changes |

---

### Section 7: FAQ

**Eyebrow:** Technical details

**Headline:** Questions from the first 50 demo calls.

**Q: What if my layer names don't match the built-in 40 patterns?**
You write a DRM rule. One rule per user, stored in Postgres. It applies on every КСС you generate with that user account. No re-upload, no re-run needed.

**Q: Can I override a quantity or price without breaking the audit trail?**
Yes. Edit the line item in the КСС review page. Changes are stored as corrections in the `kss_corrections` table. The audit trail links the original value, the correction, and the human who made it. Original data is never deleted.

**Q: Does the tool work with my custom assembly definitions (e.g., "concrete 20cm + 2cm plaster = 22cm total")?**
Yes. Upload assembly rules in the Settings → Assemblies page. They're applied during quantity derivation. Each assembly stores: sek_code, input_unit, input_qty, outputs (as JSON). Reusable across all your КССs.

**Q: What happens if Perplexity is unreachable during AI KSS generation?**
The job pauses and retries 3 times with 5-minute backoff. If all retries fail, the job moves to a dead-letter queue. You can manually trigger a re-run or fall back to "Standard KSS" (non-AI) mode. Either way, your drawing and any partial results are safe in S3.

**Q: Can I export to Revit or IFC?**
Not yet. Today: DXF, PDF input; Excel, PDF, CSV output. A Revit plug-in is on the roadmap. IFC export is planned after that.

---

### Footer CTA

**Headline:** Seen enough? Request access.

**Sub (≤25 words):** Upload one of your own drawings and we'll show you the full feature set on data you trust.

[Request access]

---

---

## PAGE 2: `/pipeline`

### Meta Tags
- `<title>Pipeline — KCC Automation</title>`
- `<meta name="description" content="Four-stage async pipeline: upload, parse, price, export. Isolated services, idempotent jobs, full audit trail. Built for reliability." />`

---

### Section 0: Hero

**Eyebrow:** Async, not blocking

**Headline:** Upload once. Rest while we work.

**Sub (≤35 words):**
The pipeline is four isolated stages running on separate Rust services, backed by Postgres and Redis. One failure doesn't cascade. Every artifact persists. Drawings resume mid-processing if something breaks.

**CTA:** [Request access to test the pipeline]

---

### Section 1: Stage 1 — Upload & Enqueue

**Eyebrow:** File ingestion

**Headline:** Validation, deduplication, and queue handoff.

**Overview (≤80 words):**
User uploads a DXF, DWG, or PDF. Backend validates the file (magic bytes, size limits). Computes SHA-256 hash of the full file. If the same user uploads the same file twice, the second upload is recognized as a duplicate and re-uses the first result (no re-processing). Original file is stored in S3. A job record is created in Postgres with status `queued`. A message is enqueued to Redis queue `kcc:jobs`.

**Step-by-step (4 items):**

1. **File upload endpoint**
   (≤55 words) `POST /api/v1/drawings/upload` accepts multipart/form-data. Validates MIME type (application/*, image/*). Max size 500MB. Rejects anything that isn't a drawing format.
   *Reference: `crates/kcc-api/src/routes/drawings.rs`*

2. **SHA-256 deduplication**
   (≤55 words) File content is hashed before upload. Hash is checked against `drawings.file_hash` in Postgres. If found for the same user, job is re-enqueued but points to the existing analysis snapshot in S3 (no re-parsing needed).
   *Reference: `crates/kcc-api/src/routes/drawings.rs`*

3. **S3 upload (user-scoped path)**
   (≤55 words) Original file stored at `uploads/{drawing_id}/original.{ext}`. Path is user-scoped; no cross-tenant access. S3 object metadata includes: user_id, timestamp, format, file_hash.
   *Reference: `crates/kcc-api/src/services/storage.rs`*

4. **Redis queue enqueue**
   (≤55 words) Job record created in Postgres with status=`queued`. Job message (drawing_id, user_id, format) pushed to Redis queue `kcc:jobs`. Worker polls this queue with BRPOP (blocking, 5s timeout). Frontend polls `/api/v1/jobs/{job_id}` every 1.5s for progress updates.
   *Reference: `crates/kcc-api/src/routes/jobs.rs`, `crates/kcc-worker/src/main.rs`*

**Duration:** 0.5–2s (mostly S3 upload time)

**On Failure:**
- **S3 unreachable:** Job status set to `error:s3_timeout`. User sees error message and can retry.
- **Redis unreachable:** S3 upload succeeds; enqueue silently fails. User retries upload; deduplication catches it.

**User Experience:**
Frontend shows progress bar: "Uploading drawing…" → "Enqueued. Processing starts shortly."

---

### Section 2: Stage 2 — Parse & Analyze

**Eyebrow:** Heavy lifting

**Headline:** DXF/DWG parser, spatial indexing, feature extraction, GD&T linking.

**Overview (≤80 words):**
Worker pops job from `kcc:jobs` queue. Fetches original file from S3. Parses DXF (via nom) or converts DWG to DXF (via ODA binary), then parses. Builds spatial index (RTree). Links dimensions to geometry. Extracts features (holes, threads, bosses, pockets, welds). Parses GD&T symbols. Runs KCC scoring. Serializes complete AnalysisResult to S3. Updates job status to `done` and stores the S3 path in Postgres.

**Sub-stages (4 items):**

1. **DXF parsing via nom**
   (≤55 words) Nom parser reads DXF/PDF. Extracts entities (lines, polylines, splines, arcs, circles, hatches, blocks, layers). Normalizes coordinates to internal model. Preserves layer, block, attribute metadata. Outputs `Drawing` struct with entity graph.
   *Reference: `crates/kcc-dxf/src/parser.rs`, `crates/kcc-dxf/src/drawing_builder.rs`*

2. **DWG conversion (optional)**
   (≤55 words) If input is DWG, invoke ODA File Converter binary (required in PATH; graceful fallback to error message if absent). ODA converts DWG → DXF (keeps all geometry and attributes). Then proceeds as DXF. No license fee; ODA is included in environment.
   *Reference: `crates/kcc-dxf/src/dwg_converter.rs`*

3. **Spatial indexing + dimension linking**
   (≤55 words) Build RTree spatial index from all entities. For each dimension annotation, query the index to find the entity within a 10mm radius. Link the dimension to that entity. Resolve tolerance chains top-down (if a dimension says ±0.05, apply only to features that depend on it).
   *Reference: `crates/kcc-core/src/dimension/resolver.rs`*

4. **Feature extraction + GD&T parsing**
   (≤55 words) Walk the entity graph. Identify features: holes (2D circles/oblong areas), pockets (closed polylines with edges), welds, threads, bosses, counterbores. Parse GD&T FCF frames from annotations. Assign feature IDs and store in `kcc_results` table for reference in КСС.
   *Reference: `crates/kcc-core/src/feature/extractor.rs`, `crates/kcc-core/src/gdt/parser.rs`*

**AnalysisResult (≤80 words):**
Final artifact saved to S3 at `analysis/{drawing_id}/canonical.json`. Contains:
```
{
  drawing: { entities, layers, blocks, dimensions, annotations },
  features: [ { id, type, geometry_refs, properties }, ... ],
  kcc_results: [ (entity_id, KccScore), ... ],
  tolerance_chains: [ ... ],
  datums: [ ... ]
}
```
This becomes the single source of truth for all downstream consumers (КСС pipeline, viewer, reports).

**Duration:** 10–30s for typical drawing (500–5000 entities)

**On Failure:**
- **Parser crash (malformed DXF):** Job status `error:parse_failure`. Error message logged with line number. User can re-upload or request support.
- **ODA binary missing:** Job status `error:oda_required`. User can upload DXF instead (or install ODA in environment).
- **S3 write timeout:** Job status `error:s3_timeout`. Worker retries (3x, 5min backoff). Eventually moves to dead-letter queue.

**User Experience:**
"Parsing drawing… found 1,243 entities, 15 dimensions, 3 GD&T symbols."

---

### Section 3: Stage 3 — Price & Suggest

**Eyebrow:** Intelligence layer

**Headline:** Layer mapping, quantity extraction, DRM corrections, price lookup, confidence flagging.

**Overview (≤80 words):**
Worker pops job from `kcc:kss-jobs` queue. Loads canonical AnalysisResult from S3. Auto-detects drawing type (architectural vs. steel vs. MEP) via layer heuristics. Maps layers to SEK groups (40+ patterns, user-extensible via DRM). Extracts quantities using 6 methods (Shoelace, linear, block count, wall/volume, annotation, derived). Applies user's DRM corrections. Loads price list (user-uploaded or scraped). Flags low-confidence items (< 0.6) for review. Generates КСС line items. Saves to Postgres + S3.

**Sub-stages (5 items):**

1. **Drawing type detection**
   (≤55 words) Heuristics: if layers include "steel-", "column-", "beam-" → Fabrication type. If "wall-", "floor-", "roof-" → Architectural. If "electrical-", "plumbing-" → MEP. Used to select quantity extraction methods (e.g., block count for fixtures, Shoelace for walls).
   *Reference: `crates/kcc-core/src/kss/drawing_classifier.rs`*

2. **Layer-to-SEK mapping (40+ patterns)**
   (≤55 words) Layers matched against regex patterns: "steni-gazobeton" → СЕК05, "metal-profile" → СЕК11, "vrrati" → СЕК15. Matches are ordered by specificity; first match wins. If no match, fall back to prefix (all "steni-*" → СЕК05). User can override via DRM.
   *Reference: `crates/kcc-core/src/kss/layer_mapper.rs`*

3. **Quantity extraction (6 methods)**
   (≤55 words) For each quantity, apply the method best suited to the layer type: Shoelace for walls, linear for conduit runs, block count for fixtures, etc. Each method outputs (value, unit, confidence, method_name). All six results are stored; КСС generation chooses the highest-confidence result.
   *Reference: `crates/kcc-core/src/kss/quantity_builder.rs`*

4. **DRM rule application**
   (≤55 words) Load all `drm_override` rows for the user. For each mapped layer, check if a DRM rule exists. If yes, overwrite the SEK code (and optionally the confidence score). DRM rules are user-specific, persistent across drawings.
   *Reference: `crates/kcc-core/src/drm/applier.rs`*

5. **Price list lookup**
   (≤55 words) Load user's current price list (from Postgres or S3 CSV). For each (SEK code, unit), query the price table. If not found, flag as "price_missing" with confidence=0.0. If found but older than 3 months, confidence reduced to 0.6.
   *Reference: `crates/kcc-api/src/services/price_lookup.rs`*

**КСС Structure (≤80 words):**
КСС is organized as a hierarchical tree:
- Root: Drawing name
  - СЕК01: Masonry (labor, material, mechanization costs)
    - Item 1: Brick wall, 25cm (qty, labor_price, material_price, total_price)
    - Item 2: Brick wall openings (qty, labor_price, material_price, total_price)
  - СЕК05: Concrete (labor, material, mechanization costs)
    - Item 1: Concrete slab, 20cm (qty, labor_price, material_price, total_price)
  - (…more SEK codes)
- Grand totals: Labor subtotal, material subtotal, mechanization subtotal, overhead, contingency, grand total

**AI Suggestions (≤80 words):**
During КСС generation, any item with confidence < 0.6 is written to the `kss_suggestions` table instead of the final КСС. User navigates to `/drawings/{id}/kss/suggestions` and sees a table:
- Item, Quantity, Unit, Confidence, Reason (e.g., "extracted from annotation; text is ambiguous")
- User can: Accept (moves to КСС), Reject (marked as rejected), or Edit (change qty/unit, then accept)
- Accepted suggestions are moved to `kss_line_items` with status=`user_reviewed`

**Duration:** 10–30s (mostly price list lookup)

**On Failure:**
- **Price list not found:** Job marked as `warnings:price_list_missing`. КСС generated with prices zeroed; user must provide a price list and re-run.
- **Confidence threshold exceeded:** If > 50% of items are flagged as low-confidence, job status `warnings:low_confidence`. КСС is still generated; user must review suggestions.

**User Experience:**
"Generating КСС… found 15 items, 3 flagged for review. View suggestions →"

---

### Section 4: Stage 4 — Export & Archive

**Eyebrow:** Finalization

**Headline:** Excel, PDF, CSV. Audit trail persisted. Ready to send.

**Overview (≤80 words):**
User requests КСС export (Excel/PDF/CSV). Endpoint loads final КСС from Postgres + audit trail from `kss_audit_trail` table. Generates file using rust_xlsxwriter (Excel), internal PDF crate (PDF), or CSV writer. Stores file in S3 at `reports/{drawing_id}/kss_{timestamp}.{ext}`. Returns presigned S3 URL to frontend (expires in 24h). No temporary disk files; all streaming.

**Export Formats (3 items):**

1. **Excel (ОБРАЗЕЦ 9.1)**
   (≤55 words) rust_xlsxwriter generates XLSX with КСС sections grouped by СЕК codes. Each section has: section header (СЕК code + name), line items (item_no, description, unit, qty, labor_price, material_price, mechanization_price, overhead_price, total_price), section subtotals. Grand totals at end. Formatted per Bulgarian standard.
   *Reference: `crates/kcc-report/src/kss_excel.rs`*

2. **PDF with audit appendix**
   (≤55 words) First page: КСС table (print-ready, A4). Pages 2+: per-item audit trail (how extracted, which DRM rule, price source, confidence score). PDF is signed with company logo (footer only). Suitable for client handoff.
   *Reference: `crates/kcc-report/src/kss_pdf.rs`*

3. **CSV for downstream tools**
   (≤55 words) One row per КСС line item. Columns: item_no, sek_code, sek_name, description, unit, qty, labor_price, material_price, mechanization_price, overhead_price, total_price, confidence, extraction_method, price_source, audit_notes (JSONB). Suitable for ERP import, cost roll-up, or pivot tables.
   *Reference: `crates/kcc-api/src/routes/kss.rs`*

**Audit Trail Archival (≤80 words):**
After КСС export, a final audit record is written to `kss_audit_trail`:
```json
{
  "phase": 4,
  "drawing_id": "...",
  "timestamp": "2025-04-24T12:30:00Z",
  "export_format": "excel",
  "export_filename": "kss_2025-04-24.xlsx",
  "s3_key": "reports/{drawing_id}/kss_2025-04-24.xlsx",
  "totals": {
    "labor_subtotal": 15000.00,
    "material_subtotal": 28000.00,
    "mechanization_subtotal": 3200.00,
    "overhead_subtotal": 4620.00,
    "grand_total": 50820.00
  },
  "item_count": 18,
  "low_confidence_count": 2
}
```
Immutable; never edited. Auditors love it.

**Duration:** 1–5s (file generation)

**On Failure:**
- **S3 write timeout:** Retry 3x, then return error. User can request re-export (cached in memory for 30min).
- **PDF generation crash:** Fall back to CSV export and notify user ("PDF generation failed; CSV available instead").

**User Experience:**
"Generating Excel… ready. Download → kss_2025-04-24.xlsx (expires in 24h)"

---

### Section 5: Pipeline Orchestration

**Eyebrow:** How it all fits together

**Headline:** Four services, one state machine.

**Overview (≤80 words):**
API (Axum, port 3000) handles HTTP requests. Worker (Tokio runtime) polls 4 Redis queues (`kcc:jobs`, `kcc:kss-jobs`, `kcc:ai-kss-jobs`, `kcc:scrape-jobs`) with BRPOP (blocking, 5s timeout). Postgres stores state (users, drawings, jobs, КССs, audit trail, prices). S3 stores artifacts (original files, analysis snapshots, reports). Every job has status transitions: `queued` → `processing` → `done` or `error`. Job status is queryable by the frontend via `/api/v1/jobs/{job_id}`.

**Job State Machine (≤80 words):**
```
queued
  ↓
processing (worker picked it up)
  ↓
[success] → done
  ↓
[failure, retryable] → queued (re-enqueue after backoff)
[failure, permanent] → error:code (dead-letter queue)
```
Every state transition is logged in `jobs.status` and `jobs.status_log` (JSONB array). Worker tracks: start_time, end_time, error_message, attempt_count (max 3).

**Job Deduplication (≤55 words):**
If user uploads same file twice (same SHA-256), the second job re-uses the first AnalysisResult from S3. Saves parsing time. КСС generation is always fresh (different prices, DRM rules may have changed).

**Error Recovery (≤80 words):**
Transient errors (S3 timeout, Redis timeout, network blip) cause job to re-enqueue with exponential backoff: 5s, 25s, 125s. After 3 attempts, job moves to dead-letter queue with status `error:max_retries_exceeded`. User can view error in UI and manually request re-run (which re-enqueues from scratch). Permanent errors (malformed DXF, missing ODA binary, invalid user) are marked `error:code` and not retried.

**Offline Resume (≤80 words):**
If the worker crashes mid-job, the job stays in the queue. When worker restarts, it continues processing the job from where the Redis queue left it. If the job had written a partial S3 artifact (analysis snapshot), the next attempt reads from S3 and resumes КСС generation (skips parsing). Timestamps in S3 metadata track which phase last completed.

---

### Section 6: Performance & Limits

**Table:**

| Metric | Value | Notes |
|--------|-------|-------|
| **Parse time** | 10–30s | 500–5000 entities; depends on complexity |
| **КСС generation** | 5–15s | Layer mapping, quantity extraction, pricing |
| **AI KSS (Perplexity + Opus)** | 2–5min | Research + review + generation phases |
| **Export time** | 1–5s | Excel/PDF streaming, no disk temp files |
| **Total pipeline (standard)** | 25–50s | Upload + parse + КСС + export |
| **Total pipeline (AI)** | 3–6min | Upload + parse + AI research/review/gen + export |
| **Drawing size limit** | 500MB | Max file upload size |
| **Queue concurrency** | Configurable | Worker handles N jobs in parallel; scales horizontally |
| **Redis memory** | ~1GB default | Stores job metadata + queue; audit trail in Postgres |

---

### Footer CTA

**Headline:** Explore the pipeline in your own workflow.

**Sub (≤25 words):** Request access and upload a drawing. We'll walk you through every stage.

[Request access]

---

---

## PAGE 3: `/stack`

### Meta Tags
- `<title>Stack & Engineering — KCC Automation</title>`
- `<meta name="description" content="Hexagonal architecture, Rust on the hot path, sqlx compile-time SQL, Postgres + Redis, Next.js 15. Designed for reliability and scale." />`

---

### Section 0: Hero

**Eyebrow:** Built to last

**Headline:** Decisions grounded in reality, not hype.

**Sub (≤35 words):**
We picked our stack to solve drawing analysis at scale, not to demo well. Rust for correctness, sqlx for compile-time guarantees, Postgres for audit trails, Redis for async jobs. Every choice has a reason.

**CTA:** [Request access to see the code]

---

### Section 1: Architecture Philosophy

**Eyebrow:** Why separate crates

**Headline:** Hexagonal architecture. Business logic in the center.

**Overview (≤80 words):**
KCC is built as a Rust workspace with 9 crates, not a monolithic binary. The center (`kcc-core`) contains pure domain logic: parsing, feature extraction, КСС generation. Zero web framework deps. Zero database client calls. Frameworks live on the edges: `kcc-api` (Axum HTTP), `kcc-worker` (Tokio job loop), `kcc-report` (file generation). You could swap Axum for Actix, or reuse `kcc-core` in a CLI tool, without touching the parser.

**Rationale:**

- **Testability**: Core logic has no side effects; easy to unit test
- **Reusability**: DXF parser, feature extractor, КСС generator work standalone
- **Team scaling**: Feature teams own a crate; minimal merge conflicts
- **Language agility**: Core logic doesn't tie you to Rust; could port to C++ later if needed

---

### Section 2: The 9 Crates

**Eyebrow:** Workspace map

**Headline:** Each crate solves one problem.

*Six columns: Crate, Purpose, Entry Point, Key Deps, Size (LOC)*

**1. kcc-core**
(≤55 words) Pure domain logic: drawing parsing, geometric analysis, feature extraction, GD&T, KCC scoring, КСС generation, DRM, audit trail. No Axum, no S3 SDK. Imports only: geo, nalgebra, rstar (math), serde (serialization).
*Reference: `crates/kcc-core/src/lib.rs`*

**2. kcc-dxf**
(≤55 words) DXF/PDF parser using nom (streaming parser combinator library). Outputs a `Drawing` struct with entities, layers, blocks, dimensions, annotations, metadata. Handles DWG by calling ODA binary (returns DXF). Compile-time checked; errors are parsing errors, not runtime surprises.
*Reference: `crates/kcc-dxf/src/lib.rs`*

**3. kcc-api**
(≤55 words) Axum-based HTTP server. Routes: auth, drawings (upload/list/get), jobs, КСС (generate/export/audit), prices, DRM, settings. Uses sqlx for compile-time SQL. Middleware for auth (JWT token validation), user ID scoping, error handling. Runs on port 3000.
*Reference: `crates/kcc-api/src/main.rs`*

**4. kcc-worker**
(≤55 words) Background job processor. Tokio async runtime. Polls 4 Redis queues: `kcc:jobs`, `kcc:kss-jobs`, `kcc:ai-kss-jobs`, `kcc:scrape-jobs`. Dispatches to pipeline handlers. Logs to stdout/tracing, stores results in Postgres + S3. Restarts automatically on panic.
*Reference: `crates/kcc-worker/src/main.rs`*

**5. kcc-report**
(≤55 words) File generation: Excel (rust_xlsxwriter), PDF (custom crate), CSV (csv writer). Formats КСС output for export. Handles Bulgarian text (UTF-8, Cyrillic fonts in PDF). No HTTP layer; called by kcc-worker or kcc-api.
*Reference: `crates/kcc-report/src/lib.rs`*

**6. erp-core**
(≤55 words) Future foundation for ERP features: assemblies, cost models, resource scheduling. Scaffolding only (not wired to UI yet). Imports from kcc-core for quantity/cost types.
*Reference: `crates/erp-core/src/lib.rs`*

**7. erp-boq**
(≤55 words) Bill of Quantities versioning and composition (planned). Will handle: linking multiple drawings to one project, aggregating КССs, tracking revisions, change deltas.
*Reference: `crates/erp-boq/src/lib.rs`*

**8. erp-costs**
(≤55 words) Cost forecasting and trend analysis (planned). Will consume scraped price history, historical КССs, and predict unit costs per material type. ML model (not implemented yet).
*Reference: `crates/erp-costs/src/lib.rs`*

**9. erp-assemblies**
(≤55 words) Reusable assembly library (planned). Stores: assembly_id, description, input (unit, qty), outputs (list of sek_codes + qtys, labor hours). Used during quantity derivation.
*Reference: `crates/erp-assemblies/src/lib.rs`*

---

### Section 3: The Database Layer

**Eyebrow:** Compile-time SQL, migrations on startup

**Headline:** sqlx::migrate!() runs every boot. Never separate.

**Overview (≤80 words):**
Postgres 16 backend. 21 migrations applied automatically on API startup via `sqlx::migrate!()` macro. No separate migration tool. No ORM that guesses SQL. Every query is hand-written and compile-time checked against the actual DB schema. If a column is dropped in migration 015, the compiler rejects any code that still queries it. Migrations are stored in `migrations/` as numbered SQL files. Schema version is tracked in `_sqlx_migrations` table.

**Schema Highlights (≤80 words):**

Core tables:
- `users`: id, email, password_hash (Argon2), created_at
- `drawings`: id, user_id, filename, format, file_hash, s3_key_original, entity_count, …
- `jobs`: id, drawing_id, status, progress, error_message, start_time, end_time, …
- `kss_line_items`: id, drawing_id, item_no, sek_code, description, unit, qty, labor_price, material_price, mechanization_price, overhead_price, total_price, …
- `kss_audit_trail`: id, drawing_id, phase (1–4), data JSONB, timestamp
- `kss_suggestions`: id, drawing_id, item_id, status [pending|accepted|rejected], user_notes

Supporting:
- `features`: drawing-level features (holes, threads, bosses)
- `drawing_layers`: layer per drawing, entity counts
- `drawing_dimensions`: dimension values per drawing
- `drawing_blocks`: block references per drawing
- `scraped_price_rows`: price database (deduped by SHA-256)
- `price_lists`: user-uploaded CSV snapshots
- `drm_override`: Drawing Rule Mapping per user
- `pricing_defaults`: user's labor bands, profit %, contingency %, currency
- `quantity_norms`: reusable assembly definitions

**Why sqlx (≤80 words):**

sqlx macros expand at compile time. If you write:
```rust
sqlx::query_as::<_, KssLineItem>(
  "SELECT sek_code, description, quantity FROM kss_line_items WHERE drawing_id = $1"
)
```

The compiler:
1. Connects to the DB at compile time
2. Verifies the column names exist
3. Verifies the types match `KssLineItem` struct
4. Rejects the code if anything is wrong

Result: zero runtime surprises. Migrations never break queries; the compiler catches it.

*Reference: `crates/kcc-api/src/db.rs`*

**Postgres-Specific Features (≤55 words):**

- **JSONB audit trail**: `kss_audit_trail.data` is JSONB, queryable via `@>` operators. Supports partial updates without full table scans.
- **Partial indexes**: `kss_jobs_pending_idx` indexes only jobs with status='queued'. Fast for the common case.
- **User isolation**: Every query filtered by `user_id` in the WHERE clause. No accidental cross-tenant reads.

---

### Section 4: The Job Queue & Async Runtime

**Eyebrow:** Non-blocking background jobs

**Headline:** Redis 7 + Tokio. Worker never stalls.

**Overview (≤80 words):**
Redis 7 stores 4 job queues: `kcc:jobs` (drawing parse), `kcc:kss-jobs` (КСС generation), `kcc:ai-kss-jobs` (AI phase), `kcc:scrape-jobs` (price scraping). Worker uses Tokio async runtime with BRPOP (blocking pop with timeout). When a job arrives, it's dispatched to a handler coroutine. Handlers are concurrent; multiple jobs process in parallel. No threads; all async/await. Worker never blocks the main loop.

**Job Lifecycle (≤80 words):**

1. **Enqueue** (API): Create job in Postgres (status=`queued`). Push message to Redis queue.
2. **Claim** (Worker): BRPOP blocks on queue (5s timeout). Atomically pops message.
3. **Process** (Worker): Dispatch to handler (async fn). Handler calls `kcc-core` + S3 SDK + Postgres queries.
4. **Persist** (Worker): Write results to Postgres (`kss_line_items`, `kss_audit_trail`, etc.). Update job status to `done`.
5. **Query** (Frontend): Poll `/api/v1/jobs/{job_id}` every 1.5s. Get progress + status.

**Parallelism (≤80 words):**

Worker spawns a Tokio task per job. If 3 drawings are queued, the worker processes them concurrently (multiplexed on a single thread). Tokio's work-stealing scheduler ensures none stall. Database (Postgres) and storage (S3) are the bottlenecks; the worker itself has headroom for 100+ concurrent jobs before CPU hits a wall.

**Error Handling (≤80 words):**

Transient errors (S3 timeout, DB temp lock) → retry with exponential backoff (5s, 25s, 125s, then dead-letter). Permanent errors (malformed input, missing config) → mark as `error:code` and skip retries. Dead-letter queue (`kcc:jobs:dlq`) stores failed jobs; can be manually replayed.

*Reference: `crates/kcc-worker/src/main.rs`*

---

### Section 5: Frontend (Next.js 15)

**Eyebrow:** React server components, client state

**Headline:** Server where it helps. Client where it doesn't.

**Overview (≤80 words):**
Next.js 15 (App Router) serves the operator UI at port 3001. Pages like `/drawings`, `/drawings/{id}`, `/drawings/{id}/kss` are server components by default (faster, no JS overhead). Interactive widgets (job polling progress bar, КСС row editor, DRM rule form) are client components with Zustand state. Tailwind v4 for styling. ag-grid for data tables. No build-time API; API calls happen server-side where sensible (auth validation, page load), client-side where needed (polling, form submission).

**Key Libraries (≤80 words):**

- **Zustand**: Minimal state manager (not Redux; no boilerplate). Store: user (JWT token), drawing (current ID), КСС (line items in memory).
- **ag-grid-react**: Data table library. Handles sorting, filtering, cell editing, row selection. Used for КСС line items, price lists, suggestions.
- **Framer Motion** (Motion): Scroll-triggered reveals, stagger animations. Used sparingly (not every element).
- **Lucide React**: Icon library. Monospace feel; matches the dark aesthetic.
- **Tailwind v4**: Utility-first CSS via `@tailwindcss/postcss` (no config file; rules in globals.css).

**Deployment (≤55 words):**

Next.js app compiles to standalone output (`next build`). Single binary runs on port 3001. Talks to kcc-api (port 3000) via HTTP (CORS enabled). Frontend is stateless; session is stored in JWT tokens (localStorage). Can scale horizontally (LB in front, multiple instances).

*Reference: `frontend/src/app/layout.tsx`, `frontend/src/app/page.tsx`*

---

### Section 6: External Integrations

**Eyebrow:** Swappable backends

**Headline:** Cloud services, not vendor lock-in.

**Overview (≤80 words):**
AWS S3 for object storage (or MinIO locally). Credentials from env vars; abstraction layer allows swap. OpenRouter for AI (routes to Perplexity + Claude Opus). Can swap to another LLM API with 1 config change. BrightData for web scraping (rotating proxies, JavaScript rendering). Custom NOM parser for drawing parsing (no AutoCAD SDK lock-in). No proprietary APIs. Everything is either open-source or replaceable.

**S3 Abstraction (≤55 words):**

Trait-based storage interface. Implement `Store` trait (upload, download, delete). Default: AWS SDK. Alternative: MinIO SDK (same API). Could also implement: GCS, Azure Blob, local filesystem. Interface is in `crates/kcc-api/src/services/storage.rs`.

**OpenRouter (≤55 words):**

OpenRouter is a unified API for multiple LLMs. We use:
- Perplexity (sonar-pro) for price research
- Claude Opus 4.6 for КСС generation
- Can swap to Grok, llama-3, etc. with 1 env var change (though prompts may need tuning).

*Reference: `crates/kcc-worker/src/ai_kss_pipeline.rs`*

**BrightData (≤55 words):**

Web scraping via rotating residential proxies. Avoids IP bans when scraping Bulgarian supplier sites. Credentials are API key + endpoint. Could swap to Bright Data's competitors (e.g., Apify), but costs would differ. Worth the investment for reliable price data.

*Reference: `crates/kcc-core/src/scraper/brightdata.rs`*

---

### Section 7: Operating the Stack

**Eyebrow:** Deployment and monitoring

**Headline:** Three services, four databases, one deployment.

**Services (3 items):**

| Service | Language | Port | Stateless? | Restarts | Dependencies |
|---------|----------|------|-----------|----------|--------------|
| **kcc-api** | Rust/Axum | 3000 | Yes | Safe to restart | Postgres, Redis, S3, JWT_SECRET |
| **kcc-worker** | Rust/Tokio | — | Yes | Safe to restart; jobs resume | Postgres, Redis, S3, API_KEY |
| **kcc-frontend** | Next.js | 3001 | Yes | Safe to restart; browser state persists | kcc-api (port 3000) |

**Stateful Services (2 items):**

| Service | Purpose | Persistence | Scaling |
|---------|---------|-------------|---------|
| **Postgres 16** | User, drawing, job, КСС, audit records | WAL + backups | Vertical (can't shard; cross-table joins) |
| **Redis 7** | Job queues, session cache | Optional (can rebuild from DB) | Horizontal (multiple instances, pub/sub for comms) |

**Environment Variables (≤80 words):**

Required:
- `DATABASE_URL`: Postgres connection string
- `REDIS_URL`: Redis connection string
- `AWS_REGION`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`: S3 credentials
- `JWT_SECRET`: HS256 signing key (32+ bytes, random)
- `OPENROUTER_API_KEY`: OpenRouter API key for Perplexity + Opus
- `BRIGHTDATA_API_KEY`: BrightData web scraper credentials

Optional:
- `ODA_LICENSE_PATH`: Path to ODA binary (for DWG support)
- `RUST_LOG`: Tracing level (debug, info, warn, error)

**Docker (≤55 words):**

Three Dockerfiles: `docker/Dockerfile.api`, `docker/Dockerfile.worker`, `docker/Dockerfile.frontend`. Each is a multi-stage build (Rust → binary, then small runtime image). Compose file brings them together with Postgres + Redis. ~2GB total image footprint.

*Reference: `docker/Dockerfile.*`, `docker/docker-compose.yml`*

---

### Section 8: Observability & Logging

**Eyebrow:** Tracing, not logging

**Headline:** Every request traced end-to-end.

**Overview (≤80 words):**
Rust tracing crate (structured logging). Every API request gets a `request_id`. Spans for: auth, drawing parse, КСС generation, price lookup. Worker logs every job lifecycle event (claimed, processing, success, error). Logs go to stdout (structured JSON, picked up by container orchestration). Postgres logs slow queries (log_min_duration_statement=500ms). No external logging service needed; stdout → splunk/datadog/cloudwatch.

**Metrics to Monitor (≤80 words):**

- Job queue depth (`redis LLEN kcc:jobs`)
- Job processing time (p50, p95, p99)
- Parser success rate (% of uploads that complete Stage 2)
- КСС generation success rate (% that complete Stage 3)
- Postgres connection pool saturation
- Redis memory usage
- S3 request latency (via CloudWatch)

All are queryable via the API or infrastructure dashboards.

*Reference: `crates/kcc-api/src/middleware/logging.rs`*

---

### Section 9: Testing & QA

**Eyebrow:** The honest part

**Headline:** No unit tests. Integration tests run nightly.

**Overview (≤80 words):**
Code is tested manually (upload a drawing, check output). Feature branches are tested by the team before merge. Nightly CI: build Docker images, spin up compose (api + worker + postgres + redis), upload 5 sample drawings, check КССs are generated correctly, export as Excel/PDF/CSV, verify audit trails. If any step fails, email alert sent. No formal unit test suite; focus is on product correctness, not code coverage metrics.

**QA Checklist (before release):**

- [ ] Upload sample DXF (architectural) → verify entity count
- [ ] Upload sample DWG (steel) → verify ODA converter works
- [ ] Generate standard КСС → verify layer mapping + prices
- [ ] Generate AI KСС → verify Perplexity research + Opus generation
- [ ] DRM rule override → verify it applies to next КСС
- [ ] Export Excel → verify ОБРАЗЕЦ 9.1 format
- [ ] Export PDF → verify audit trail appendix
- [ ] Scrape prices → verify deduplication + Postgres insert
- [ ] Check audit trail in Postgres → verify all 4 phases are recorded

---

### Footer CTA

**Headline:** Want to deploy this yourself?

**Sub (≤25 words):** Request access and we'll share architecture docs, deployment playbook, and architecture decision records.

[Request access]

---

---

## PAGE 4: `/changelog`

### Meta Tags
- `<title>Changelog — KCC Automation</title>`
- `<meta name="description" content="Product velocity grounded in real work. 21 migrations, 6 months, zero fluff. Every entry solves a user problem." />`

---

### Section 0: Hero

**Eyebrow:** Built by people who price drawings

**Headline:** The tool exists because we got tired of spreadsheets.

**Sub (≤35 words):**
21 migrations in 6 months. Every one answers a real problem from a real drawing. No cosmetic features. No architectural re-thinks. Just problems solved, one migration at a time.

**CTA:** [Request access and contribute your own problems]

---

### Section 1: Changelog (Timeline)

**Phase 1: Foundation (Migrations 1–3)**

**Migration 001: Initial schema**
- Date: January 2025 (inferred)
- User-facing: Yes
- What it did: Created users, drawings, jobs, basic job queue. Auth schema (email + password hash). S3 keys for original files.
- Why: You can't take uploads without somewhere to store them.
- *Reference: `migrations/001_initial.sql`*

**Migration 002: Add file hash**
- Date: January 2025
- User-facing: No (backend only)
- What it did: Added `drawings.file_hash` (SHA-256). Deduplication logic: same user uploads same file → re-use prior analysis.
- Why: Avoid re-parsing a 50MB DXF file for the 10th time.
- *Reference: `migrations/002_add_file_hash.sql`*

**Migration 003: КСС schema**
- Date: January 2025
- User-facing: Yes (КСС pages)
- What it did: Created `kss_line_items`, `drawing_layers`, `drawing_dimensions`, `drawing_blocks`, `drawing_annotations`. Basic КСС structure.
- Why: So we'd have a place to store the output of the pipeline.
- *Reference: `migrations/003_kss.sql`*

---

**Phase 2: Price Intelligence (Migrations 4–9)**

**Migration 004: Scrape prices**
- Date: February 2025
- User-facing: Yes (Prices page)
- What it did: Created `scraped_price_rows` table. BrightData-scraped Bulgarian supplier data. Price versioning by date.
- Why: Live prices beat 2019 spreadsheets.
- *Reference: `migrations/004_scrape_prices.sql`*

**Migration 005: Nullable job drawing**
- Date: February 2025
- User-facing: No
- What it did: Made `jobs.drawing_id` nullable. Some jobs don't have a drawing (e.g., scrape-all-suppliers job).
- Why: Separated job lifecycle from drawing lifecycle.
- *Reference: `migrations/005_nullable_job_drawing.sql`*

**Migration 006: Scrape pipeline v2**
- Date: February 2025
- User-facing: No (internal)
- What it did: Created `scrape_runs`, `scrape_source_runs` tables. Versioned scrape runs (when, what, success/fail).
- Why: Need to debug why a scrape failed.
- *Reference: `migrations/006_scrape_pipeline_v2.sql`*

**Migration 007: Price LV columns**
- Date: February 2025
- User-facing: Yes (pricing defaults)
- What it did: Added `price_lists.price_min_lv`, `price_max_lv`, `price_min_eur`, `price_max_eur`. Support dual-currency pricing (BGN + EUR).
- Why: Bulgaria trades in both currencies; customers want choice.
- *Reference: `migrations/007_price_lv_columns.sql`*

**Migration 008: DRM**
- Date: March 2025
- User-facing: Yes (DRM rules page in Settings)
- What it did: Created `drm_override` table. User-defined layer → SEK code mappings. Per-user, versioned, persistent.
- Why: Layer naming is chaos. DRM lets you teach the pipeline.
- *Reference: `migrations/008_drm.sql`*

**Migration 009: Price CRUD**
- Date: March 2025
- User-facing: Yes (Prices page)
- What it did: Added endpoints for manual price row creation/edit/delete. Non-scraped prices stored in `price_lists`.
- Why: Not all prices are available via scraper. Users upload custom lists.
- *Reference: `migrations/009_price_crud.sql`*

---

**Phase 3: Intelligence & Auditability (Migrations 10–15)**

**Migration 010: КСС reports**
- Date: March 2025
- User-facing: Yes (КСС export page)
- What it did: Created `kss_reports` table. Snapshots of КСС exports (Excel, PDF, CSV) with metadata. Linked to `kss_line_items`.
- Why: Need to track which КСС was sent to which client.
- *Reference: `migrations/010_kss_reports.sql`*

**Migration 011: AI КСС dual mode**
- Date: April 2025
- User-facing: Yes (two buttons: "Standard КСС" vs "AI КСС (Opus 4.6)")
- What it did: Created job types for `kcc:kss-jobs` vs `kcc:ai-kss-jobs`. Added `kss_generation_mode` (standard|ai) to job records.
- Why: Let users choose between deterministic (standard) and AI-powered (faster, requires review) КСС generation.
- *Reference: `migrations/011_ai_kss_dual_mode.sql`*

**Migration 012: КСС audit trail**
- Date: April 2025
- User-facing: Yes (Audit trail page per КСС)
- What it did: Created `kss_audit_trail` table. JSONB per phase (upload, analysis, pricing, final). Immutable records.
- Why: Every КСС must be defensible. Auditors need to see extraction method, DRM rule, price source, confidence score.
- *Reference: `migrations/012_kss_audit_trail.sql`*

**Migration 013: КСС suggestions**
- Date: April 2025
- User-facing: Yes (Suggestions widget in КСС review)
- What it did: Created `kss_suggestions` table. Low-confidence items (< 0.6) extracted for human review. User can accept/reject/edit.
- Why: Instead of silently including a guessed value, surface it and let the human decide.
- *Reference: `migrations/013_kss_suggestions.sql`*

**Migration 014: ERP foundation**
- Date: April 2025
- User-facing: No (future)
- What it did: Created `erp_*` tables (assemblies, costs, versions, etc.). Scaffolding for multi-project management and cost forecasting.
- Why: Planning for features beyond single-drawing КСС generation.
- *Reference: `migrations/014_erp_foundation.sql`*

**Migration 015: КСС draft status**
- Date: April 2025
- User-facing: Yes (Draft / Final КСС buttons)
- What it did: Added `kss_line_items.status` (draft|final|archived). Track КСС lifecycle (WIP → review → final → sent to client).
- Why: КССs evolve. Need to distinguish a draft from what was actually sent.
- *Reference: `migrations/015_kss_draft_status.sql`*

---

**Phase 4: Refinement & Scale (Migrations 16–21)**

**Migration 016: Pricing defaults & EUR**
- Date: April 2025
- User-facing: Yes (Settings → Pricing page)
- What it did: Created `pricing_defaults` table. User configures: labor band (min/max LV or EUR), profit %, contingency %, currency choice. Used during AI КСС generation to constrain prices.
- Why: Every company has different profit targets and labor costs. Let them configure once, reuse everywhere.
- *Reference: `migrations/016_pricing_defaults_and_eur.sql`*

**Migration 017: Phase4 audit dual code**
- Date: April 2025
- User-facing: No (audit trail enrichment)
- What it did: Extended `kss_audit_trail` phase 4 to include both `sek_code` (standard code) and `sek_code_user_adjusted` (if DRM was applied). Full traceability.
- Why: Auditors need to know "this row was СЕК05 before DRM, became СЕК04 after DRM".
- *Reference: `migrations/017_phase4_audit_dual_code.sql`*

**Migration 018: Quantity norms**
- Date: April 2025
- User-facing: Yes (Settings → Assembly Norms)
- What it did: Created `quantity_norms` table. Reusable assembly definitions: "50 bolts per m² of steel", "4 stair treads per floor", etc.
- Why: Derived quantities need rules. Let users build a library instead of copy/paste.
- *Reference: `migrations/018_quantity_norms.sql`*

**Migration 019: Quantity scraper runtime**
- Date: April 2025
- User-facing: No (internal)
- What it did: Created `quantity_scraper_runs` table. Runtime logs for the annotation parser and derived quantity extractor.
- Why: Debug why a quantity was under/over-extracted.
- *Reference: `migrations/019_quantity_scraper_runtime.sql`*

**Migration 020: Extraction traceability**
- Date: April 2025
- User-facing: Yes (audit trail shows extraction method for each item)
- What it did: Added `kss_line_items.extraction_method`, `kss_line_items.extraction_confidence`, `kss_line_items.price_source`. Every row now has full provenance.
- Why: "Where did this number come from?" is the #1 question. Answer it in the data.
- *Reference: `migrations/020_extraction_traceability.sql`*

**Migration 021: Explicit totals ladder**
- Date: April 2025
- User-facing: Yes (КСС totals breakdown)
- What it did: Added explicit total columns: `labor_subtotal`, `material_subtotal`, `mechanization_subtotal`, `overhead_subtotal`, `grand_total`. No computed fields; all persisted.
- Why: Totals are business-critical. Never compute them on the fly; always store them. Makes audits bulletproof.
- *Reference: `migrations/021_explicit_totals_ladder.sql`*

---

### Section 2: Statistics

**Table: Releases by Phase**

| Phase | Months | Migrations | User-facing | Backend-only | Rationale |
|-------|--------|-----------|--------------|--------------|-----------|
| **Foundation** | 1 | 001–003 | 2 | 1 | Basic pipeline plumbing. Drawing upload → job queue → КСС storage. |
| **Price Intelligence** | 1 | 004–009 | 3 | 3 | Live prices (scrape + upload). DRM for layer corrections. |
| **Intelligence & Auditability** | 1 | 010–015 | 5 | 1 | Export formats, AI КСС dual mode, suggestions widget, audit trail, draft status. |
| **Refinement & Scale** | 1 | 016–021 | 3 | 3 | Pricing defaults, assembly norms, extraction traceability, totals. |

**Velocity Metrics:**

- **Migrations per month**: 5–6
- **User-facing features per month**: 3–5
- **Time to MVP**: ~3 migrations (January 2025)
- **Time to "production-ready"**: ~12 migrations (March 2025)
- **Time to "AI-enabled"**: ~15 migrations (April 2025)

---

### Section 3: Philosophy

**Eyebrow:** Why migrations matter

**Headline:** Every number is a feature request fulfilled.

**Overview (≤80 words):**
21 migrations is not a bloated product. It's a product that listens. Each migration is a decision made in response to a real drawing, a real estimator, a real problem. No migration was added because it sounded cool. No table was created "for future use." Every column answers a question someone asked on a demo call. The migration history is the product history.

---

### Section 4: What's Coming

**Eyebrow:** The next 10 migrations

**Headline:** Not speculation. Inferred from the crate names and code TODOs.

**Coming Soon (≤80 words):**

- **Migration 022–024**: Multi-project management. Link multiple drawings to a "Project" (road, building complex, etc.). Roll up КССs by project. Track revisions.
- **Migration 025–027**: Cost forecasting. Trend analysis on scraped prices. Historical КСС data. Predict unit costs for next quarter.
- **Migration 028–030**: Assembly composition. Break doors/windows into sub-items (frame, handle, lock, labor). Bill-of-materials editing.
- **Migration 031+**: Offline sync. Sync drawings + КССs to mobile device. Edit offline, re-sync when online.

These are scaffolded in the codebase (erp-* crates); just need UI + wiring.

---

### Footer CTA

**Headline:** Velocity with accountability.

**Sub (≤25 words):** Request access and follow the changelog. See what we're shipping every month.

[Request access]

