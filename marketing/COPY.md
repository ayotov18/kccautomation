# KCC Automation — Landing Page Copy

Single source of truth for every word on `marketing/`. Copy is editorial, minimal, confident. No exclamation marks. No "revolutionary". No "AI-powered" as a prefix. English headlines, Bulgarian industry terms preserved (КСС, СЕК, ОБРАЗЕЦ 9.1). Sentence case everywhere except product name.

---

## Brand voice

- **Technical, not salesy.** Every claim is grounded in what the product does. No vaporware.
- **Assumes the reader knows construction.** Writes for quote managers and estimators, not CTOs.
- **Bulgarian-first on domain vocabulary.** КСС, СЕК01–СЕК49, ОБРАЗЕЦ 9.1 stay as-is; the wrapping sentence is English.
- **One joke per page, max.** The product is serious, the tone is dry.

---

## Navigation

```
KCC Automation                           [Log in]  [Request access]
```

Single nav row. Product name left, two links right. No mega-menu, no feature dropdowns.

---

## Hero

**Eyebrow** (small, uppercase, monospace, above headline):
`Construction estimating, on rails`

**Headline** (H1, two lines allowed):
**From DXF to КСС in under three minutes.**

**Sub-headline** (one line, grey):
Upload the drawing. The pipeline reads every layer, pulls live Bulgarian market prices, and returns a priced КСС with an audit trail for every row.

**Primary CTA:**
[Request access]

**Secondary CTA (text-only, arrow):**
See how the pipeline works →

**Trust micro-line** under CTAs, 11px monospace, grey-50:
`21 migrations applied · SEK01–SEK49 supported · Excel, PDF, CSV export`

---

## Stack / trust row (below hero, no section heading)

Monospaced marquee of the actual stack, no logos — just names:

```
RUST  ·  AXUM  ·  SQLX  ·  POSTGRES 16  ·  REDIS 7  ·  NEXT.JS 15  ·  S3  ·  BRIGHTDATA  ·  OPENROUTER
```

Scrolls slowly, pauses on hover.

---

## Section 1 — Features (3 cards, short, non-technical)

Section eyebrow: `What it does`
Section headline: **Three steps replace an afternoon of spreadsheet work.**

### Card 1 — Read the drawing

**Heading:** The drawing talks back.
**Body:** Upload a DXF, DWG, or PDF. KCC parses every layer, dimension, block, and annotation — walls, columns, openings, steel members. Spatial index, dimension-to-geometry links, feature extraction, the lot. You get back a structured model of the drawing, not a blob of text.

### Card 2 — Price the scope

**Heading:** Live Bulgarian prices, not a 2019 CSV.
**Body:** KCC maps each quantity to its СЕК cost code, then pulls current market prices — either from supplier pages via the scraping pipeline, or researched on the spot through Perplexity + Claude Opus. Every row shows its price source and confidence.

### Card 3 — Ship the КСС

**Heading:** A КСС you can hand to a client.
**Body:** Grouped by СЕК, labour + material + mechanisation + overhead broken out, audit trail on every line, ready to export to Excel (ОБРАЗЕЦ 9.1 compatible), PDF, or CSV. Changes you make in the UI feed back as corrections the pipeline learns from.

---

## Section 2 — The workflow (hero schematic / animated SVG)

Section eyebrow: `The pipeline`
Section headline: **Four services, one pipeline.**
Sub: Every stage runs in isolation — the API takes uploads, the worker handles the heavy lifting, Postgres keeps state, Redis runs the queue.

Animated SVG schematic (replaces a generic dashboard-in-a-bezel). Nodes:

```
Upload  →  kcc-api  →  Postgres  →  kcc-worker  →  S3 / Redis  →  КСС
         (Axum)       (21 mig.)    (DXF + AI)    (analysis)     (Excel/PDF)
```

Each node is a monospaced label over a 1px outline rectangle. Connecting lines draw in on scroll. A small amber dot travels along the path, looped.

---

## Section 3 — Feature deep-dive (bento, 5 tiles)

Section eyebrow: `Under the hood`
Section headline: **Confidence on every number, not just the ones the demo shows.**

### Tile A (tall, left) — Confidence scoring
Every extracted quantity carries a 0.0–1.0 score based on how it was derived. Shoelace-computed areas score 0.9. Guessed values score 0.4. Anything below 0.6 is surfaced to the review widget instead of silently rolling into the total. You know which rows to trust.

### Tile B (wide, top-right) — DRM (Drawing Rule Mapping)
`layer "steni-beton" → actually brick, use СЕК05 not СЕК04`. Rules you write once, applied automatically on every КСС the pipeline generates. Stored per-user in Postgres. No more retyping the same correction.

### Tile C (square) — Live price research
BrightData-proxied scrapes of Bulgarian supplier sites plus a Perplexity + Opus research loop. Outputs are deduped by normalized key and stored in `scraped_price_rows` for reuse. Prices get cited, not guessed.

### Tile D (square) — Audit trail per row
Every line item records: how the quantity was extracted, which DRM rule fired, which price source was used, and the Opus reasoning if AI was involved. Goes to `kss_audit_trail`. Defensible when a client asks where a number came from.

### Tile E (wide, bottom) — Exports the estimator actually wants
Excel via rust_xlsxwriter matching ОБРАЗЕЦ 9.1, PDF via the internal report crate, CSV for anything else. One endpoint per format, streaming, no "please wait while we build your file" screen.

---

## Section 4 — Testimonial (placeholder, keep slot)

Section eyebrow: `In production`
Section headline: **Built by people who've priced a thousand drawings by hand.**
Body (short): KCC replaces the 2–4 hour manual КСС pass with 2–3 minutes of pipeline work and 5 minutes of review. The tool exists because the team building it got tired of the alternative.

No avatar photos. A single pull quote, attributed to a real person once the product is public. Placeholder text for v1:

> "The first estimating tool I've used that admits when it's guessing."
> — _placeholder, to be replaced pre-launch_

---

## Section 5 — The stack (for the technical reader)

Section eyebrow: `Stack`
Section headline: **Rust on the hot path. Everything else where it belongs.**

Two-column layout. Left column a vertical list, right column a one-sentence explanation each.

| | |
|---|---|
| **Rust workspace, 9 crates** | `kcc-api`, `kcc-worker`, `kcc-core`, `kcc-dxf`, `kcc-report`, `erp-core`, `erp-boq`, `erp-costs`, `erp-assemblies`. |
| **Axum + sqlx + Tokio** | HTTP on the API, background jobs on the worker, compile-time-checked SQL everywhere. |
| **Postgres 16** | 21 sqlx migrations, applied automatically on API startup. Every KSS, every correction, every audit row. |
| **Redis 7** | BullMQ-style queues for drawing parsing and the AI KSS phases (research → review → generate). |
| **Next.js 15 + Tailwind v4** | Operator UI. Server components where they help, client components where they don't. |
| **S3** | User-scoped paths for originals and analysis snapshots. No cross-tenant anything. |
| **BrightData web unlocker** | Supplier price scraping behind a rotating residential proxy. |
| **OpenRouter** | Perplexity `sonar-pro` for price research, Claude Opus 4.6 for КСС generation, swappable per-job. |

---

## Section 6 — Security / data (keep short)

Section eyebrow: `On data`
Section headline: **Your drawings, your prices, your audit trail.**

Three one-liners in a row:

- **JWT auth + Argon2 hashing** — per-user, every query scoped to the calling user.
- **SHA-256 deduplication** — same file uploaded twice is the same row, not a leak.
- **S3 paths are user-scoped** — nothing sits in a shared bucket prefix.

---

## Section 7 — FAQ (4 items, collapsible)

Section eyebrow: `Questions`
Section headline: **Things people ask in the first demo call.**

**Q: Does it replace the estimator?**
No. It replaces the part of the estimator's day where they type a spreadsheet. Review still lives with the human.

**Q: What if a layer is mis-labelled in the drawing?**
You write a DRM rule once and KCC applies it on every future drawing with that layer name. Corrections compound.

**Q: Which prices does the AI pull?**
Bulgarian market, researched live via Perplexity with sonar-pro, then refined by Claude Opus into line items. Every row stores its source so you can re-check.

**Q: Which drawing formats work today?**
DXF and PDF, fully. DWG works when the host has the ODA File Converter installed. Plans for IFC and RVT sit behind the current roadmap.

---

## Footer CTA

Section headline: **Ready to stop retyping spreadsheets?**
Sub: Request access and we'll show you the full pipeline on one of your own drawings.

[Request access]

Trust line under CTA, 11px monospace, grey-50:
`Built in Sofia · Rust + Next.js · No vendor lock-in`

---

## Footer

Three columns.

**Product** — Features · Pipeline · Stack · Pricing (coming soon)
**Company** — About · Blog (coming soon) · Contact
**Legal** — Terms · Privacy · Status

Bottom row, monospace:
`© 2026 KCC Automation · kccgen.xyz`

---

## Meta tags

- `<title>KCC Automation — DXF to КСС in under three minutes</title>`
- `<meta name="description" content="Upload a construction drawing. Get back a priced КСС with live Bulgarian market data and a full audit trail. Built in Rust." />`
- OG image: the hero background (Prompt 1 output).
- Twitter card: `summary_large_image`.

---

## Word counts (keep honest)

| Section | Word target |
|---|---|
| Hero + sub | ≤ 35 |
| Each feature card body | ≤ 55 |
| Bento tiles | ≤ 50 each |
| Testimonial | ≤ 45 |
| Stack table rows | ≤ 20 each |
| FAQ answer | ≤ 35 each |
| Footer CTA | ≤ 25 |

Anything longer gets cut.
