# KCC Marketing — All Generative Prompts

Single manifest of every prompt used. `gen-images.mjs` and `gen-videos.mjs` read this logic (copied into the scripts for determinism). Change a prompt → regenerate only that slug → commit the new asset.

## 0. House style paragraph (verbatim, prepended to everything)

> KCC Automation house style — cinematic industrial-premium aesthetic for a German-engineered B2B construction software. Dark background dominant (#0A0A0C to #14161A), graphite and carbon tones, with restrained accents in desaturated steel-blue (#3E5566) and a single warm highlight in amber-bronze (#B87333) used sparingly. Volumetric atmospheric haze, subtle film grain, 35mm or 50mm lens feel, shallow depth of field, physically-accurate lighting with one motivated key source and soft fill. Materials must read as real: brushed aluminum, anodized titanium, matte concrete, carbon fiber, architectural glass, dark leather, ceramic, blueprint paper, oxidized bronze. Compositions are negative-space heavy. No people, no logos, no text, no rendered UI, no stock-photo cliches, no glossy CGI, no rainbow gradients, no neon cyberpunk, no fantasy. Feels like a Linear / Vercel / Arc landing page crossed with a Leica product photograph shot inside a precision-engineering workshop.

---

## 1. Section-background videos — chained continuity

Each video's START frame must match the previous video's END frame. Seedance can't cut — it's image-to-video — so we pass the final frame of the prior video's reference as the reference of the next. Continuity is visual: same space, camera moves through it.

### SEC-1 `video-bg-hero` (intro — empty workshop)
- Ref image: `hero.png`
- Duration: 8s, 16:9, 1080p
- Prompt:
```
[HOUSE STYLE condensed]
Static establishing shot opens on an empty, dark precision-engineering workshop at blue hour. The matte-black drafting table sits lower-third of frame. Atmospheric haze. Over 8 seconds, execute one very slow dolly-in of 5% magnification, ending with the table dominating the lower-half of frame, just reaching a sharper focus on its right edge where a single amber highlight lives.
[0s] Wide frame, dust motes suspended in the god ray, everything still.
[4s] Dolly-in reaches halfway, table beginning to read more clearly.
[8s] Final position: table lower-half, amber edge highlight clean and centered — this frame is the handoff.
Style: cinematic, 24fps, anamorphic 2.39:1, shallow DOF, Cinestill 800T grade, subtle grain, slow-cinema pace.
Negatives: no camera shake, no pan, no tilt, no zoom change, no cuts, no people, no text, no UI, no audio, no flashing, no color shift, no objects moving, no rendered screens, no sci-fi holograms.
```

### SEC-2 `video-bg-features` (features backdrop — blueprint close-up)
- Ref image: `bg-features.png` (NEW — generated specifically as the continuation frame)
- Duration: 8s, 16:9, 1080p
- Prompt:
```
[HOUSE STYLE condensed]
The frame opens on the exact close-up of the drafting-table right edge with amber highlight (continuation). Paper edges of a technical drawing are just visible in the lower-third. Over 8 seconds, the camera slowly pushes forward over the edge and onto the drawing surface, revealing cyanotype-blue contour lines and dimension callouts. The drawing itself does not move.
[0s] Table edge + amber highlight (matching the inbound frame).
[4s] Camera drifting forward over the edge, contour lines entering frame.
[8s] Blueprint macro detail fills the frame, ready as handoff to the bento section.
Style: cinematic, 24fps, shallow DOF, cold steel-blue grade with one warm highlight, subtle grain.
Negatives: no camera shake, no whip, no zoom jumps, no cuts, no drawings animating themselves, no ink appearing, no people, no hands, no text appearing, no UI, no audio, no flashing.
```

### SEC-3 `video-bg-bento` (bento section backdrop — materials mood board)
- Ref image: `bg-bento.png` (NEW)
- Duration: 8s, 16:9, 1080p
- Prompt:
```
[HOUSE STYLE condensed]
Open on the blueprint macro (continuation). Over 8 seconds, the camera slowly pulls back and the plane of focus shifts to reveal the drawing now sitting among industrial materials — a slab of brushed aluminum, raw formwork concrete, carbon fiber, oxidized bronze — arranged in shallow layers on matte black. The light sweeps slowly left-to-right across the composition.
[0s] Blueprint macro matching the inbound frame.
[3s] Pull back begins, materials creep into frame edges.
[6s] Full mood-board composition visible, light reaching center, amber-bronze material picking up a highlight.
[8s] Hold final frame as handoff.
Style: cinematic 85mm feel, f/4, chiaroscuro, subtle film grain, cold neutral with one moving warm highlight.
Negatives: no camera shake, no whip pan, no orbit, no zoom pulse, no cuts, no subject movement, no materials sliding, no rotation, no people, no hands, no tools, no text, no UI, no audio, no flashing, no color cycling.
```

### SEC-4 `video-bg-testimonial` (testimonial backdrop — dusk office)
- Ref image: `bg-testimonial.png` (reuse `testimonial.png`)
- Duration: 8s, 16:9, 1080p
- Prompt:
```
[HOUSE STYLE condensed]
Open on the architectural interior at dusk (continuation — as if the camera has moved out of the workshop and into a premium engineering office overlooking a construction site). Over 8 seconds, the interior is perfectly still; outside the tall window, two additional distant construction-site lights fade on extremely softly, and a gentle drift of atmospheric haze crosses the far background right-to-left. Crane silhouettes do not move.
[0s] Baseline dusk scene, one distant light.
[4s] Second distant light fades in softly, haze drifting slowly.
[8s] Third distant light just barely visible, haze continuing — handoff frame.
Style: cinematic 35mm, f/2, teal-and-amber split grade, Cinestill 800T look, long-exposure stillness.
Negatives: no camera movement, no dolly, no pan, no tilt, no zoom, no people in the room or on the site, no reflections of people, no cars, no crane movement, no text, no UI, no audio, no flashing, no color cycling.
```

### SEC-5 `video-bg-cta` (closer — titanium edge)
- Ref image: `cta.png` (reuse)
- Duration: 8s, 21:9, 1080p
- Prompt:
```
[HOUSE STYLE condensed]
Near-abstract close-up: a vertical edge of polished dark surface catches a slow sweep of cool daylight over the 8 seconds, revealing brushed-titanium grain and a millimeter of warm amber reflection at the very edge. The composition is otherwise still — only the light travels.
[0s] Surface mostly dark, faint haze and dust in the light.
[4s] Light sweep at midpoint, amber reflection strengthening.
[8s] Light at far right, amber glint at peak, shadow returning to the left — handoff for loop or footer.
Style: cinematic 100mm, f/2.8, static locked-off, one hard key from right, subtle grain, cold neutral with one amber accent.
Negatives: no camera shake, no pan, no tilt, no zoom, no cuts, no objects moving, no people, no hands, no text, no UI, no audio, no flashing, no color cycling, no gradients pulsing.
```

---

## 2. Widget hover videos — play only on hover

Short 4s loops, poster image shown by default, `<video>` starts on hover.

### W-1 `video-card-drawing` (feature card 1 — the drawing)
- Ref image: `feature-blueprint.png` (reuse)
- Duration: 4s, 1:1, 1080p
- Prompt:
```
[HOUSE STYLE condensed]
Macro technical-drawing loop. Ink lines and paper fibers stay completely still. Over 4 seconds execute one slow rack focus from the foreground ruler bokeh to a sharp blueprint intersection.
[0s] Foreground ruler edge tack-sharp, blueprint soft.
[2s] Focus plane mid-travel.
[4s] Blueprint intersection tack-sharp.
Style: cinematic macro, 24fps, shallow DOF, f/2.8 bokeh, cold steel-blue grade, one amber micro-highlight.
Negatives: no camera movement, no zoom, no pan, no paper curling, no ink animating, no people, no hands, no text appearing, no UI, no audio, no flashing.
```

### W-2 `video-card-prices` (feature card 2 — live prices)
- Ref image: `feature-materials.png` (reuse)
- Duration: 4s, 1:1, 1080p
- Prompt:
```
[HOUSE STYLE condensed]
Industrial materials hero loop. Materials (aluminum, concrete, carbon fiber, bronze) do not move. A slow sweep of warm directional light travels left-to-right across the composition over 4 seconds.
[0s] Light at far left, most of frame in shadow.
[2s] Light reaches center, amber-bronze catches highlight.
[4s] Light at far right, shadow returning to left.
Style: cinematic product still, 24fps, 85mm, f/4, chiaroscuro, film grain, cold neutral with one moving warm highlight.
Negatives: no camera movement, no camera shake, no zoom, no pan, no orbit, no subjects moving, no rotation, no people, no hands, no tools, no text, no UI, no audio, no flashing, no color cycling.
```

### W-3 `video-card-export` (feature card 3 — КСС export)
- Ref image: `feature-data.png` (reuse)
- Duration: 4s, 1:1, 1080p
- Prompt:
```
[HOUSE STYLE condensed]
Overhead static composition on matte black anodized aluminum. The physical grid of embossed cells and micro-scratches are still. Over 4 seconds, the highlighted amber row slowly brightens and then a single pinpoint of warm light traces left-to-right along the row, fading as it reaches the end.
[0s] Grid baseline, amber row at low glow.
[2s] Row fully lit, pinpoint light at midpoint.
[4s] Pinpoint exits right edge, row dimming back to baseline.
Style: cinematic overhead, 50mm, f/8, soft diffused overcast skylight, subtle grain.
Negatives: no camera movement, no zoom, no pan, no tilt, no grid moving, no cells shifting, no rotation, no text, no UI, no audio, no flashing, no color cycling.
```

### W-4 `video-tile-confidence` (bento tile — confidence scoring)
- Ref image: `tile-confidence.png` (NEW)
- Duration: 4s, 16:9, 1080p
- Prompt:
```
[HOUSE STYLE condensed]
Close-up of a physical analog precision gauge with a matte black dial and a single thin brass needle. Over 4 seconds the needle travels very slowly from its rest position on the left to roughly 80% across the arc, then holds. The dial itself and all tick marks are still. A faint amber rim catches on the bezel.
[0s] Needle at far-left rest position.
[3s] Needle reaches 80% position, slight overshoot micro-settle.
[4s] Needle settled and held.
Style: cinematic macro, 100mm, f/2.8, soft directional key, subtle grain, cold neutral.
Negatives: no camera movement, no zoom, no pan, no shake, no numbers or text animating, no UI, no glowing trail, no people, no hands, no audio, no flashing.
```

### W-5 `video-tile-drm` (bento tile — Drawing Rule Mapping)
- Ref image: `tile-drm.png` (NEW)
- Duration: 4s, 16:9, 1080p
- Prompt:
```
[HOUSE STYLE condensed]
Top-down abstract composition: a grid of faint cyanotype-blue layer labels (unreadable, just marks) on matte dark paper. Over 4 seconds, a single amber highlight travels from one marker in the upper-left to another in the lower-right, tracing a rule mapping. The underlying paper and markers do not move.
[0s] Highlight at origin marker, faint.
[2s] Highlight mid-travel along the diagonal, trailing dim afterglow.
[4s] Highlight arrived at destination marker, trail faded.
Style: cinematic overhead, 50mm, f/5.6, soft skylight, subtle grain, cold neutral with one amber accent.
Negatives: no camera movement, no zoom, no pan, no paper movement, no labels animating into text, no UI, no people, no audio, no flashing, no rainbow.
```

### W-6 `video-tile-audit` (bento tile — audit trail)
- Ref image: `tile-audit.png` (NEW)
- Duration: 4s, 16:9, 1080p
- Prompt:
```
[HOUSE STYLE condensed]
Side profile macro of a short stack of archival ledger cards bound with a waxed linen thread, edge-lit from the right. Over 4 seconds, a soft vertical band of warm key light sweeps across the stack's edge from left to right, making the cut edges of the cards glow in sequence, then settles.
[0s] Edge fully shadowed, only amber thread visible.
[2s] Light at midpoint, half the stack edges glowing.
[4s] Light at right, stack edge fully lit, begins to dim.
Style: cinematic product still, 100mm, f/4, single directional key, subtle grain, cold neutral with warm accent.
Negatives: no camera movement, no zoom, no pan, no shake, no card movement, no pages flipping, no text appearing, no UI, no people, no hands, no audio, no flashing.
```

---

## 3. Hero + section "new" background images (needed as seed frames for chained videos)

### IMG-1 `bg-features.png`
```
[HOUSE STYLE]
Close-up transition frame: the right edge of a matte black drafting table in extreme foreground (bottom third of frame), cutting across into a 2/3 top frame that shows paper edges of a large technical drawing just beginning to be visible beyond the table edge. A warm amber highlight lines the polished metal bevel. Shallow depth of field: table edge tack-sharp, drawing soft and ambient. Still-life quality; this is a narrative handoff frame.
Shot on 50mm, f/2.8, eye-level, soft directional key from camera-right, cold steel-blue grade with one amber highlight.
Negatives: no text, no logos, no UI, no people, no hands, no dramatic lighting, no gradients, no CGI feel.
```

### IMG-2 `bg-bento.png`
```
[HOUSE STYLE]
Near-abstract arrangement: the corner of a cyanotype-blue technical drawing (lower-left of frame, slightly out of focus), transitioning via a shared pool of shadow into a composition of industrial material samples — a slab of brushed aluminum, raw formwork concrete, a piece of carbon fiber, a wedge of oxidized bronze — stacked in shallow layers across the right side of frame. The materials are in the key light; the drawing is in soft shadow. Amber bronze catches the hot highlight. This is a transition frame between blueprint and materials worlds.
Shot on 85mm, f/4, hard directional key from camera-left, chiaroscuro, cold neutral with one moving warm highlight.
Negatives: no text, no labels, no UI, no people, no hands, no tools, no colorful backgrounds, no gradients, no CGI plastic look.
```

### IMG-3 `tile-confidence.png`
```
[HOUSE STYLE]
Macro product still: a single matte-black analog precision gauge with a minimal arc scale, a thin brass needle at rest on the left. Tick marks engraved, not printed. A soft warm key light picks up the bezel edge. Background dissolves to near-black. No legible numbers, no brand, no markings.
Shot on 100mm macro, f/2.8, eye-level, single directional soft key from camera-left, cold neutral with amber accent, subtle grain.
Negatives: no text, no numbers legible, no logos, no UI, no hands, no people, no rainbow.
```

### IMG-4 `tile-drm.png`
```
[HOUSE STYLE]
Overhead top-down composition on dark kraft paper: a grid of roughly a dozen small hand-drawn cyanotype-blue marker dots, lightly connected by faint graphite trace lines. One dot in the upper-left and one in the lower-right are highlighted with amber-bronze ink. No text anywhere. Paper fibers and subtle dust are visible, as if a physical diagram pinned on a matte workshop table.
Shot on 50mm, f/5.6, 90-degree overhead, soft diffused skylight, cold neutral with one amber accent.
Negatives: no text, no labels, no characters, no logos, no UI, no people, no hands, no arrows or words.
```

### IMG-5 `tile-audit.png`
```
[HOUSE STYLE]
Side-profile macro: a short stack (about 12) of archival ledger cards bound with a waxed amber linen thread through punched holes, sitting on a matte graphite surface. Edge-lit from camera-right; the cut edges of the cards show fine paper texture. The thread glows slightly warmer. Deep negative space to the left.
Shot on 100mm, f/4, eye-level, single hard key from right, cold neutral grade with warm thread accent, subtle grain.
Negatives: no text, no writing, no logos, no UI, no people, no hands, no book pages opening, no magic glow, no CGI.
```

---

## 4. Pipeline phased-demo images (NOT videos — used inside the pipeline widget)

The animated pipeline section cycles through 4 still frames to tell the story Upload → Parse → Price → Export. Each is a physical still-life metaphor, not a UI screenshot.

### IMG-P1 `phase-1-upload.png`
```
[HOUSE STYLE]
Overhead still-life: a folded large-format technical drawing (cyanotype blue lines visible on the edge), a dark matte clipboard, and a single matte-black stylus laid in a precise arrangement on a graphite work surface. The drawing is closed — not yet opened. A soft warm key light from upper-right casts long shadows. This is "the drawing arrives."
Shot on 50mm, f/5.6, 75-degree overhead, cold neutral with one amber highlight, subtle grain.
Negatives: no text legible, no people, no hands, no UI, no logos, no stock-photo cliche.
```

### IMG-P2 `phase-2-parse.png`
```
[HOUSE STYLE]
The same technical drawing now opened flat on the graphite surface, a precision engineer's scale ruler laid across it at a slight angle, and a small magnifying loupe resting over one intersection — as if the pipeline is inspecting it. Fine cyanotype contour lines fill the frame. Light is now harder, motivated by an overhead task lamp just out of frame. This is "parse and measure."
Shot on 50mm, f/4, 75-degree overhead, single hard key from above-left, cold steel-blue grade with amber ruler highlight, subtle grain.
Negatives: no legible numbers, no text, no UI, no people, no hands, no arrows drawn, no annotations appearing.
```

### IMG-P3 `phase-3-price.png`
```
[HOUSE STYLE]
The drawing remains on the graphite surface but now alongside it are three small physical "price tokens" — a brushed aluminum coin, a raw concrete sample cube, a cut piece of oxidized bronze — each sitting on a tiny matte card. A thin amber thread visually connects one detail on the drawing to one of the tokens, implying cost-code matching. Deep negative space upper half. This is "the pipeline finds the prices."
Shot on 50mm, f/4, 70-degree overhead, cold neutral grade with warm thread accent, subtle grain.
Negatives: no legible text on cards, no numbers, no UI, no people, no hands, no stock-photo cliche, no price tags in plastic, no cartoons.
```

### IMG-P4 `phase-4-export.png`
```
[HOUSE STYLE]
Closing frame: the drawing now rolled and tied with an amber linen cord, sitting on top of a short stack of cream-cream-colored estimator's paper — the КСС. A matte-black fountain pen lies alongside at a precise angle. Soft warm key light from upper-right, long shadows, deep graphite surface. This is "the КСС is ready to send."
Shot on 50mm, f/5.6, 75-degree overhead, cold neutral with one amber warm key, subtle grain.
Negatives: no text legible, no logos, no writing on the paper, no UI, no people, no hands, no coffee cup, no laptop.
```
