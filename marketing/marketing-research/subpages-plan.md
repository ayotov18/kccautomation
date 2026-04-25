# KCC Automation: Subpage Strategy

## Rationale

KCC's landing page establishes the core value (DXF → КСС in 3 minutes) but doesn't address:
1. **Teams evaluating deep features** — need per-domain breakdown of capabilities
2. **Technical architects** — must understand why Rust + sqlx + Redis matter
3. **Procurement decision-makers** — rely on product velocity to assess viability
4. **Pipeline engineers** — want to understand the async job model

The 4 new pages fill these gaps without bloating the landing page.

---

## Page 1: `/features`

### Purpose
Deep dive into every real feature, grouped by domain (parsing, pricing, exports, auditing). Assumes reader knows construction and wants proof that the tool handles edge cases, not just happy paths.

### Structure
- Hero: "Confidence on every claim. Evidence in the code."
- 4 feature groups (6-7 items each):
  1. **Drawing parsing** (DXF/DWG/PDF layers, blocks, dimensions, GD&T)
  2. **Quantity extraction** (6 methods: Shoelace, linear, block count, wall/volume, annotation, derived)
  3. **Price sourcing** (scraped Bulgarian market data, Perplexity research, confidence scoring)
  4. **Export & auditing** (Excel ОБРАЗЕЦ 9.1, PDF, CSV, full audit trail per row)
- 3–4 stats cards (21 migrations, 40+ layer patterns, 7 extraction methods)
- FAQ (3–4 technical questions)

### Visitor Intent
Quote managers, estimators, CAD coordinators doing due diligence. They want to know: "Does it handle *my* specific layer naming? Can I override bad extractions? Will it price my local materials?"

### Key Claim
"Every number is defensible. Extraction method, DRM rule applied, price source, confidence score — all auditable."

---

## Page 2: `/pipeline`

### Purpose
Expanded walkthrough of the 4-stage async pipeline from the landing page, but with technical depth and per-stage metaphors. Shows the machinery without UI noise.

### Structure
- Hero: "Async pipeline, linear predictability. Upload once, rest while we work."
- 4 expanded sections (one per stage):
  1. **Upload & Enqueue** (file validation, SHA-256 dedup, Redis queue)
  2. **Parse & Analyze** (DXF nom parser, ODA DWG converter, spatial indexing, feature extraction, GD&T linking)
  3. **Price & Suggest** (layer → SEK mapping, quantity extraction, DRM corrections, live price lookup, confidence flagging)
  4. **Export & Archive** (Excel/PDF/CSV generation, audit trail persistence, S3 snapshots)
- Per-stage: Duration estimates, what happens if it fails (error recovery), what the user sees (progress UI)
- Animated pipeline schematic (reuse landing page SVG, but with expandable stage cards on click)
- "Offline resume" explanation (drawings resume in case of outage)

### Visitor Intent
System architects, integration teams, anyone asking "Can this run on our infrastructure? What happens if Redis dies? Is it idempotent?"

### Key Claim
"Four isolated services. One can fail without stalling the others. Every stage logs to Postgres, every artifact lands in S3."

---

## Page 3: `/stack`

### Purpose
Engineering story. Why Rust? Why sqlx compile-time SQL? Why Redis for the queue? Why Postgres 16? For technical readers who care more about implementation than features.

### Structure
- Hero: "Built to last. Decisions grounded in reality, not hype."
- 9-crate architecture breakdown (visual dependency graph, brief for each):
  1. **kcc-api** (Axum HTTP layer, request validation, auth middleware)
  2. **kcc-worker** (Redis polling, job dispatch, error handling)
  3. **kcc-core** (domain logic, zero web deps, reusable)
  4. **kcc-dxf** (nom parser + ODA integration, geometry primitives)
  5. **kcc-report** (Excel/PDF generation, ОБРАЗЕЦ 9.1 formatting)
  6. **erp-core**, **erp-boq**, **erp-costs**, **erp-assemblies** (future foundation)
- Per-layer reasoning (backend, database, frontend):
  - "Rust on the hot path" (parsing, geometry, extraction)
  - "sqlx::migrate!() on startup" (no separate migration tool, no ORM guessing)
  - "Tokio async" (worker never blocks, queue polling non-blocking)
  - "Postgres 16 + jsonb" (audit trail as structured data, not text blobs)
  - "Redis 7 + BullMQ semantics" (job retry, dead-letter queue, state machine)
  - "Next.js 15 + Server Components" (where they help, client components where they don't)
- Optional: "Swappable backends" diagram (show S3 abstraction, Redis replacement strategy)

### Visitor Intent
SREs, DevOps engineers, backend teams vetting the tech. They want to know: "Is it a ball of mud or layered design? Can I run this myself? What's the operational surface?"

### Key Claim
"Hexagonal architecture. Business logic in the center, frameworks on the edges. You could swap Axum for Actix, or Redis for RabbitMQ, without rewriting the parser."

---

## Page 4: `/changelog` (or `/story`)

### Purpose
Small but meaningful. Shows product velocity and real-world problem-solving. Inferred from migration names and git history.

### Structure
- Hero: "Built by people who price drawings. The tool exists because they got tired of spreadsheets."
- 12–15 migration entries (grouped by version or phase):
  1. **Phase 1: MVP** (initial schema, auth, uploads, basic DXF parsing)
  2. **Phase 2: КСС Generation** (layer mapping, quantity extraction, price lists, exports)
  3. **Phase 3: Intelligence** (confidence scoring, DRM, audit trail, suggestions)
  4. **Phase 4: AI KSS** (Perplexity research, Opus generation, dual-mode pricing)
  5. **Phase 5: Refinement** (explicit totals, extraction traceability, pricing defaults)
- Per entry: Migration number, date inferred, one-sentence summary, impact (user-facing or backend)
- Callout: "21 migrations in 6 months. Not cosmetic. Every one solves a real problem from a real drawing."
- Optional: Small table of "Features added per month" or "Days to MVP" (if data exists)

### Visitor Intent
Early-adopter customers, investors, partners. They want to sense momentum and know the team listens to feedback (migrations are evidence).

### Key Claim
"Velocity with purpose. Every migration answers a user problem, not a trendy architecture decision."

---

## Navigation & Cross-Linking

- Landing page nav stays as-is (no changes to `/`). All 4 new pages are *sibling* routes.
- **Landing → subpages**: Add secondary nav or "Explore" menu:
  - "Features" → `/features`
  - "Pipeline" → `/pipeline`
  - "Stack" → `/stack`
  - "Changelog" → `/changelog`
- **Each subpage footer**: Include quick jump links to siblings.
- **Each subpage CTA**: "Ready to see this in action? Request access" (same as landing).

---

## Estimated Content Volume

| Page | Sections | Words | Images | Complexity |
|------|----------|-------|--------|-----------|
| /features | 4 + FAQ | 1200–1500 | 5–6 hero images + 8–12 feature card videos | Medium |
| /pipeline | 4 stages | 900–1100 | 1 hero + 4 stage diagrams + 4 animated SVGs | Medium-High |
| /stack | 9 crates + 3 layers | 1100–1400 | 1 hero + 1 architecture diagram + 1 dependency graph | High |
| /changelog | 15 entries | 600–800 | 1 hero + optional 1 timeline chart | Low |

---

## Design & Workflow

All pages follow the same house style as the landing page:
- Dark industrial aesthetic (#0A0A0C–#14161A, steel-blue accents, amber highlights)
- Monospace nav, sentence case, no exclamation marks
- Generated hero images + optional hover videos
- Sticky nav, scroll-triggered reveals, 35mm/50mm lens aesthetic
- Same footer columns (Product, Company, Legal)

No new layout components needed — reuse Hero, Eyebrow, Bento, FAQ, Footer patterns from landing page.
