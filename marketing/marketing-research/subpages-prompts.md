# KCC Automation: Image & Video Prompts for Subpages

All prompts follow the house style defined in `marketing/PROMPTS.md`. Each page gets:
- 1 hero background image (used as `<section>` background)
- 1 optional hero background video (chained to next page)
- 2–6 feature/tile card hover videos (4s loops, 1:1 aspect, play on hover)

---

## PAGE 1: `/features`

### Hero Image: `features-hero.png`

```
[HOUSE STYLE]
Overhead still-life on a matte graphite work surface: a spread of 40+ handwritten index cards, each labeled with a Bulgarian SEK code ("СЕК01", "СЕК05", "СЕК11", etc.), arranged in a loose grid. Cards are cream-colored kraft paper with tack-sharp legible code numbers. A single brass callout pin is pushed through the corner of three cards, creating a visual grouping. In the far corner, a small precision gauge (matte black face, brass needle at rest) sits slightly out of focus. The overall scene reads as "inventory, order, auditability." Negative space dominates the upper-right quadrant. Cold neutral light from above; one amber-bronze highlight catches the brass pin and gauge bezel. Film grain, 35mm feel.
Shot on 50mm, f/5.6, 75-degree overhead, soft diffused skylight, subtle grain, cold neutral with one amber accent.
Negatives: no text legible beyond code numbers, no people, no hands, no logos, no UI, no clutter, no CGI.
```

### Hero Video (Optional Chained): `video-hero-features` (8s, 1080p, 16:9)

```
[HOUSE STYLE condensed]
Open on the static card-and-gauge still-life (continuation from `/` landing page hero end-frame if chaining). Over 8 seconds, execute one very slow rack focus from the foreground brass pin (tack-sharp) to the mid-distance gauge (now coming into focus), ending with the gauge needle and its amber bezel in crisp detail. The cards remain stationary; only the focus plane travels.
[0s] Brass pin tack-sharp, cards soft, gauge blurred.
[4s] Focus plane mid-travel, both pin and gauge in soft bokeh.
[8s] Gauge tack-sharp, pin now in soft bokeh — this frame is handoff.
Style: cinematic macro, 24fps, shallow DOF (f/2.8), subtle grain, cold neutral with one amber accent.
Negatives: no camera movement, no pan, no zoom, no cards shifting, no pin moving, no needle animating, no people, no text appearing, no UI, no audio, no flashing.
```

---

### Feature Card 1 Videos: `video-card-parsing` (4s, 1:1, 1080p)

**Content Focus:** DXF parser, layers, blocks, geometry

```
[HOUSE STYLE condensed]
Macro technical-drawing loop. A portion of a cyanotype-blue DXF drawing (contour lines, dimension callouts, layer markings barely visible) sits on a matte work surface. Over 4 seconds, a thin amber highlight traces along a single dimension line and then jumps to highlight a nearby block boundary, as if the pipeline is "reading" the drawing. The drawing itself and all geometry remain still; only the highlight moves, slowly.
[0s] Highlight at the starting dimension, faint amber.
[2s] Highlight travels along dimension line.
[4s] Highlight reaches the block boundary and holds.
Style: cinematic macro, 24fps, shallow DOF (f/3.5), cold steel-blue grade with moving amber accent.
Negatives: no camera movement, no zoom, no pan, no paper curling, no ink animating, no people, no hands, no text appearing, no UI, no audio, no flashing.
```

---

### Feature Card 2 Videos: `video-card-extraction` (4s, 1:1, 1080p)

**Content Focus:** 6 extraction methods (Shoelace, linear, block, wall, annotation, derived)

```
[HOUSE STYLE condensed]
Overhead static composition on matte black: four small geometric shapes (a closed polygon representing a wall, a line segment representing a dimension, a small rectangle representing a block, a handwritten note) sit in the four corners. Over 4 seconds, thin amber lines draw themselves from each shape toward a central point, as if the pipeline is resolving all methods to a single quantity. The shapes and notes remain still; only the connecting lines animate.
[0s] Four shapes at rest, no lines visible.
[1s] First line begins to draw from the polygon.
[2s] Second line from the dimension, third line from the block.
[3s] Fourth line from the note; all lines reach the center and hold.
[4s] All lines glow faintly and fade.
Style: cinematic overhead, 50mm, f/5.6, soft diffused skylight, subtle grain, cold neutral with moving amber accents.
Negatives: no camera movement, no zoom, no pan, no shapes moving, no text legible, no UI, no people, no hands, no audio, no flashing.
```

---

### Feature Card 3 Videos: `video-card-pricing` (4s, 1:1, 1080p)

**Content Focus:** Live Bulgarian prices, scraping, Perplexity, Opus

```
[HOUSE STYLE condensed]
Close-up of a small stack of three cards (each representing a price source: "Supplier A", "Perplexity", "Opus"), stacked vertically on a matte dark surface, edge-lit from the right. Over 4 seconds, a warm amber light sweeps across the cards from left to right, making the edges and text barely glow in sequence. The cards themselves remain stationary; only the light moves.
[0s] Light at far left, cards mostly in shadow.
[2s] Light at center, top card's edge glowing.
[4s] Light at far right, bottom card's edge glowing, then begins to dim.
Style: cinematic product still, 100mm, f/4, single directional hard key from right, subtle grain, cold neutral with moving warm highlight.
Negatives: no camera movement, no zoom, no pan, no card movement, no text legible, no UI, no people, no hands, no audio, no flashing.
```

---

### Feature Card 4 Videos: `video-card-audit` (4s, 1:1, 1080p)

**Content Focus:** Audit trail per row, defensibility

```
[HOUSE STYLE condensed]
Side-profile macro of a short stack (10–12) of archival ledger cards, bound with a waxed amber linen thread, sitting on a matte graphite surface. Over 4 seconds, a vertical band of cool daylight sweeps across the stack's edge from left to right, making the cut edges of the cards glow briefly in sequence (1–2 cards at a time), as if reviewing the layers of an audit trail.
[0s] Edge fully shadowed, only the amber thread visible.
[2s] Light at midpoint, 3–4 card edges glowing.
[4s] Light at right, stack edge fully lit, then begins to dim.
Style: cinematic product still, 100mm, f/4, single directional key from right, subtle grain, cold neutral with warm accent.
Negatives: no camera movement, no zoom, no pan, no shake, no card movement, no pages flipping, no text appearing, no UI, no people, no hands, no audio, no flashing.
```

---

### Feature Card 5 Videos: `video-card-confidence` (4s, 1:1, 1080p)

**Content Focus:** Confidence scoring (0.0–1.0)

```
[HOUSE STYLE condensed]
Close-up of a physical analog precision gauge with a matte black dial, minimal arc scale, and a thin brass needle at rest on the far left. Over 4 seconds, the needle travels very slowly from left to right, reaching approximately 75% across the arc, then holds steady. The dial and tick marks remain completely still.
[0s] Needle at far-left rest position.
[2s] Needle at 50% position.
[4s] Needle at 75% position and held.
Style: cinematic macro, 100mm, f/2.8, soft directional key, subtle grain, cold neutral.
Negatives: no camera movement, no zoom, no pan, no shake, no numbers or text animating, no UI, no glowing trail, no people, no hands, no audio, no flashing.
```

---

### Feature Card 6 Videos: `video-card-export` (4s, 1:1, 1080p)

**Content Focus:** Excel, PDF, CSV exports, ready to send

```
[HOUSE STYLE condensed]
Overhead static composition on matte black: three small stacks of cream-colored paper (representing Excel, PDF, CSV outputs) sit in a row. Over 4 seconds, a soft vertical amber light moves from left to right across all three stacks, making the top sheet of each stack glow faintly as the light passes. The stacks and papers remain still.
[0s] Light at far left, first stack's top sheet glowing faintly.
[2s] Light at center, middle stack's top sheet glowing.
[4s] Light at far right, last stack's top sheet glowing, then dims.
Style: cinematic overhead, 50mm, f/5.6, soft diffused skylight, subtle grain, cold neutral with moving warm accent.
Negatives: no camera movement, no zoom, no pan, no papers moving, no rotation, no text legible, no UI, no people, no hands, no audio, no flashing.
```

---

## PAGE 2: `/pipeline`

### Hero Image: `pipeline-hero.png`

```
[HOUSE STYLE]
Architectural interior: a precision-engineered workshop at dusk. Dark matte walls. A long work table runs left-to-right in the mid-ground. Four distinct task lamps hang above the table, each illuminating a small zone (representing the 4 pipeline stages). The first zone shows a closed cardboard box (drawing upload). The second zone shows an opened technical drawing with a magnifying loupe resting on it (parsing). The third zone shows three small physical "cost tokens" (aluminum coin, concrete block, bronze wedge) arranged on kraft cards (pricing). The fourth zone shows a rolled drawing tied with an amber cord, sitting on a stack of cream paper (export/КСС). The zones are sequentially lit, with deep negative space between them. The overall composition reads as "linear workflow, step-by-step clarity." One warm key light from upper-right; geometric shadows. Cold neutral grade with one amber highlight on the rolled drawing.
Shot on 35mm, f/4, eye-level, single directional hard key from upper-right, chiaroscuro, subtle grain.
Negatives: no text legible, no logos, no UI, no people, no hands, no computers, no clutter, no stock-photo cliche.
```

### Hero Video (Optional Chained): `video-hero-pipeline` (8s, 1080p, 16:9)

```
[HOUSE STYLE condensed]
Open on the static workshop four-zone setup (continuation from `/features` hero end-frame if chaining, or static if standalone). Over 8 seconds, execute one extremely slow left-to-right camera pan across all four zones, starting tight on the closed box and ending tight on the rolled drawing+amber cord. The lighting remains constant; only the framing shifts. Each zone briefly becomes the frame's center, allowing the viewer to study each stage.
[0s] Camera framing the closed box (upload stage).
[2s] Pan begins, box leaving frame, parsing stage entering.
[4s] Pan at center, both parsing and pricing stages visible.
[6s] Parsing zone leaving frame, pricing stage centered.
[8s] Pricing leaving, export zone (rolled drawing) centered and held. This frame is handoff.
Style: cinematic 35mm, f/4, slow pan (no zoom), subtle grain, chiaroscuro.
Negatives: no zoom, no tilt, no vertical movement, no objects moving, no people, no text appearing, no UI, no audio, no flashing.
```

---

### Stage Card 1 Videos: `video-stage-upload` (4s, 1:1, 1080p)

**Content Focus:** Upload, validation, deduplication, enqueue

```
[HOUSE STYLE condensed]
Overhead still-life: a closed cardboard box (representing a file) sits on a matte work surface. Next to it, a small SHA-256 hash card lies flat. Over 4 seconds, a faint amber light illuminates the box from above, then the light shifts to the hash card (as if performing the dedup check). Then the light fades. No movement; only light changes.
[0s] Box in dim ambient light.
[1s] Amber light brightens over the box.
[2s] Light shifts to the hash card.
[3s] Light on hash card strengthens, then begins to fade.
[4s] Light returns to ambient; scene at rest.
Style: cinematic overhead, 50mm, f/5.6, soft skylight with moving amber accent, subtle grain.
Negatives: no camera movement, no zoom, no pan, no box opening, no text legible, no UI, no people, no hands, no audio, no flashing.
```

---

### Stage Card 2 Videos: `video-stage-parse` (4s, 1:1, 1080p)

**Content Focus:** DXF parser, spatial index, feature extraction, GD&T

```
[HOUSE STYLE condensed]
Overhead macro: a cyanotype-blue technical drawing lies flat. A precision engineer's scale ruler and a small magnifying loupe are positioned over different sections of the drawing. Over 4 seconds, a subtle green spotlight (representing the parser) traces slowly across the drawing's contour lines and dimensions, highlighting them sequentially. The ruler and loupe remain still.
[0s] Drawing in ambient light, ruler and loupe at rest.
[1s] Green spotlight appears at the top of the drawing.
[2s] Spotlight moves across dimensions and contours.
[3s] Spotlight reaches the bottom and begins to fade.
[4s] Spotlight fully faded; drawing at rest.
Style: cinematic macro, 50mm, f/4, overhead, soft skylight with moving green accent, subtle grain.
Negatives: no camera movement, no zoom, no pan, no ruler moving, no loupe moving, no paper curling, no text legible, no UI, no people, no hands, no audio, no flashing.
```

---

### Stage Card 3 Videos: `video-stage-price` (4s, 1:1, 1080p)

**Content Focus:** Layer mapping, quantity extraction, DRM, price lookup, confidence flagging

```
[HOUSE STYLE condensed]
Overhead static composition on matte black: the drawing from stage 2 is now accompanied by three small "price tokens" (aluminum coin, concrete block, bronze wedge) each sitting on a kraft card. Over 4 seconds, thin amber lines draw themselves from features in the drawing to the corresponding price tokens, as if linking extraction results to prices. Lines appear slowly and then glow.
[0s] Drawing and tokens at rest, no connecting lines.
[1s] First amber line begins to draw from drawing to first token.
[2s] Second and third lines draw from other features.
[3s] All lines fully drawn and glowing.
[4s] Glow fades.
Style: cinematic overhead, 50mm, f/5.6, soft skylight, subtle grain, cold neutral with moving amber accents.
Negatives: no camera movement, no zoom, no pan, no objects moving, no text legible, no UI, no people, no hands, no audio, no flashing.
```

---

### Stage Card 4 Videos: `video-stage-export` (4s, 1:1, 1080p)

**Content Focus:** Excel, PDF, CSV generation, export, archive

```
[HOUSE STYLE condensed]
Overhead still-life: the drawing from stages 2–3 is now rolled and tied with an amber linen cord. Next to it, three small stacks of cream-colored paper (representing Excel, PDF, CSV outputs) sit in a row. Over 4 seconds, a warm amber light sweeps across all four objects from left to right, making their surfaces glow softly as the light passes. Objects remain still.
[0s] Rolled drawing and paper stacks in dim light.
[1s] Amber light begins on the rolled drawing.
[2s] Light moves to the first paper stack.
[3s] Light reaches the last paper stack and holds.
[4s] Light fades.
Style: cinematic overhead, 50mm, f/5.6, soft skylight with moving amber key, subtle grain, cold neutral with warm highlight.
Negatives: no camera movement, no zoom, no pan, no objects moving, no papers shuffling, no rotation, no text legible, no UI, no people, no hands, no audio, no flashing.
```

---

## PAGE 3: `/stack`

### Hero Image: `stack-hero.png`

```
[HOUSE STYLE]
Isometric technical illustration rendered as a physical still-life: Nine small matte-black cubic blocks sit on a dark anodized aluminum surface, arranged in a 3×3 grid (representing the 9 crates). Each block is labeled with a monospaced code name ("kcc-core", "kcc-api", "kcc-dxf", etc.) engraved into the top face. The blocks are connected by thin graphite trace lines (representing crate dependencies). The center block (kcc-core) has no dependencies coming out of it (showing it's the pure domain logic center). The outer blocks (kcc-api, kcc-worker) have dependency lines radiating inward. One warm amber highlight catches the edge of the kcc-core block. The composition is perfectly still, geometric, and reads as "modular architecture, orthogonal concerns." Deep negative space surrounds the grid. Cold neutral light from above; one amber key from camera-right.
Shot on 50mm, f/5.6, 75-degree overhead, soft diffused skylight, subtle grain, cold neutral with one amber accent.
Negatives: no text legible beyond code names, no people, no hands, no logos, no UI, no clutter, no CGI, no animated lines.
```

### Hero Video (Optional Chained): `video-hero-stack` (8s, 1080p, 16:9)

```
[HOUSE STYLE condensed]
Open on the static 9-crate grid (continuation from `/pipeline` hero end-frame if chaining, or static if standalone). Over 8 seconds, execute one extremely slow vertical dolly-in from above, starting wide (all 9 blocks visible in frame) and ending tight (centered on kcc-core block, filling 60% of frame). The lighting remains constant. The amber highlight stays on the kcc-core block. The dependency lines remain static.
[0s] Wide framing, all 9 blocks visible, grid lines visible.
[2s] Dolly-in begins, blocks getting larger.
[4s] Dolly-in reaches midpoint, 6 blocks visible, kcc-core entering frame-center.
[6s] Dolly approaches final position, kcc-core dominant.
[8s] Tight framing on kcc-core block and its immediate neighbors, amber highlight glowing. This frame is handoff.
Style: cinematic 50mm, f/5.6, slow dolly-in (no zoom), subtle grain, cold neutral with amber accent.
Negatives: no zoom blur, no pan, no tilt, no rotation, no blocks moving, no dependency lines animating, no people, no text appearing, no UI, no audio, no flashing.
```

---

### Crate Card Videos: `video-card-architecture` (4s, 1:1, 1080p)

**Content Focus:** Hexagonal architecture, layering, separation of concerns

```
[HOUSE STYLE condensed]
Overhead macro: three concentric rectangles drawn on matte dark kraft paper using thin graphite lines. The center rectangle is labeled "domain" (smallest). The middle ring is labeled "services". The outer ring is labeled "frameworks". Over 4 seconds, an amber highlight traces the path from the outer rectangle inward (from "frameworks" → "services" → "domain"), as if showing information flow. The rectangles and text remain still; only the highlight moves.
[0s] Highlight at the outer rectangle, faint.
[1s] Highlight traces along the outer edge.
[2s] Highlight moves inward to the services ring.
[3s] Highlight reaches the domain center and glows.
[4s] Glow fades.
Style: cinematic overhead, 50mm, f/5.6, soft skylight, subtle grain, cold neutral with moving amber accent.
Negatives: no camera movement, no zoom, no pan, no rectangles moving, no text animating, no UI, no people, no hands, no audio, no flashing.
```

---

### Database/Ops Card Videos: `video-card-database` (4s, 1:1, 1080p)

**Content Focus:** Postgres 16, migrations, compile-time SQL

```
[HOUSE STYLE condensed]
Macro still-life: a short stack of archival ledger cards (representing migrations) bound with a waxed thread, sitting on a matte graphite surface. Next to it, a matte-black hardcover book ("Schema Compiler"). Over 4 seconds, a soft vertical amber light sweeps across the stack from left to right, making the cut edges of the cards glow in sequence. The book remains in shadow. The light then briefly touches the book's spine and fades.
[0s] Stack and book in dim ambient light.
[1s] Amber light appears at the left edge of the card stack.
[2s] Light sweeps across the stack, edges glowing.
[3s] Light moves to the book's spine and glows faintly.
[4s] Light fades.
Style: cinematic macro, 100mm, f/4, soft directional key, subtle grain, cold neutral with moving amber accent.
Negatives: no camera movement, no zoom, no pan, no objects moving, no pages flipping, no text legible, no UI, no people, no hands, no audio, no flashing.
```

---

## PAGE 4: `/changelog`

### Hero Image: `changelog-hero.png`

```
[HOUSE STYLE]
Overhead still-life on a dark kraft paper work surface: a timeline of 21 migration "cards" (small cream-colored kraft cards, each numbered 001–021) arranged in a loose chronological path, spiraling from lower-left to upper-right. Each card has a date and a one-word label (e.g., "DRM", "Audit", "AI", "Pricing"). The cards are slightly overlapping, suggesting forward momentum. A single brass pushpin anchors the first card (001) in the lower-left. No other objects. The overall composition reads as "history, velocity, accumulation." Negative space dominates the upper-left and lower-right. Cool neutral light from above; one warm amber highlight catches the brass pin. Film grain, 35mm feel.
Shot on 50mm, f/5.6, 75-degree overhead, soft diffused skylight, subtle grain, cold neutral with one amber accent.
Negatives: no text legible beyond card numbers and one-word labels, no people, no hands, no logos, no UI, no clutter, no CGI, no timeline arrows drawn in digital style.
```

---

### Optional: Timeline Chart Video `video-timeline-velocity` (4s, 1:1, 1080p)

**Content Focus:** 21 migrations over 6 months, velocity visualization

```
[HOUSE STYLE condensed]
Side-profile macro: a short stack (6–7) of cards with increasing thickness (representing months) sits on a matte graphite surface. Each card is labeled with a month abbreviation and a number (e.g., "Jan 5", "Feb 6", "Mar 7", "Apr 3"). Over 4 seconds, an amber light sweeps across the stack from left to right, brightening each card in sequence. The cards themselves remain still.
[0s] Stack in dim light, no card brightening.
[1s] Amber light appears at the left (January card).
[2s] Light sweeps across, brightening February and March cards.
[3s] Light reaches the rightmost card (April) and glows.
[4s] Light fades.
Style: cinematic macro, 100mm, f/4, soft directional key, subtle grain, cold neutral with moving amber accent.
Negatives: no camera movement, no zoom, no pan, no card movement, no text animating, no UI, no people, no hands, no audio, no flashing.
```

---

## Summary: Image & Video Inventory

| Page | Hero Image | Hero Video | Feature/Tile Videos | Total Assets |
|------|-----------|-----------|---------------------|--------------|
| `/features` | `features-hero.png` | `video-hero-features` | 6 card videos | 8 assets |
| `/pipeline` | `pipeline-hero.png` | `video-hero-pipeline` | 4 stage videos | 6 assets |
| `/stack` | `stack-hero.png` | `video-hero-stack` | 2 card videos | 5 assets |
| `/changelog` | `changelog-hero.png` | `video-timeline-velocity` (optional) | — | 2 assets |

**Total:** ~19–21 new images + videos (plus hero chaining to landing page hero for narrative continuity).

---

## Generation Instructions

### Images

Use OpenRouter `openai/gpt-5.4-image-2` (or equivalent) or claude-3.5-sonnet-vision for all hero backgrounds.

```bash
node scripts/gen-images.mjs features pipeline stack changelog
```

Each prompt is ~300 words (house style + specific scene); expect ~2 min per image = 8 min total for 4 hero images.

### Videos

Use KIE.ai `bytedance/seedance-2` (image-to-video) for all hero + chained videos.

```bash
node scripts/gen-videos.mjs features-hero pipeline-hero stack-hero
```

Each prompt is ~200 words (house style + movement cues); expect ~5–10 min per video = 20–40 min total for hero videos.

Feature/tile card videos (4s, 1:1, non-chained) can be generated standalone or batched.

```bash
node scripts/gen-videos.mjs features-cards pipeline-cards stack-cards
```

Expect ~3–5 min per video = 30–50 min total for tile videos.

**All outputs land in `marketing/public/assets/gen/`.**

