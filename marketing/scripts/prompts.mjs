// Authoritative prompts. Mirror of PROMPTS.md — keep in sync.

export const HOUSE =
  `KCC Automation house style — cinematic industrial-premium aesthetic for a German-engineered B2B construction software. Dark background dominant (#0A0A0C to #14161A), graphite and carbon tones, with restrained accents in desaturated steel-blue (#3E5566) and a single warm highlight in amber-bronze (#B87333) used sparingly. Volumetric atmospheric haze, subtle film grain, 35mm or 50mm lens feel, shallow depth of field, physically-accurate lighting with one motivated key source and soft fill. Materials must read as real: brushed aluminum, anodized titanium, matte concrete, carbon fiber, architectural glass, dark leather, ceramic, blueprint paper, oxidized bronze. Compositions are negative-space heavy. No people, no logos, no text, no rendered UI, no stock-photo cliches, no glossy CGI, no rainbow gradients, no neon cyberpunk, no fantasy. Feels like a Linear / Vercel / Arc landing page crossed with a Leica product photograph shot inside a precision-engineering workshop.`;

export const IMAGE_PROMPTS = {
  // ---------- Section seed frames (chained narrative) ----------
  hero: `${HOUSE}\n\nWide cinematic establishing shot of a dark precision-engineering workshop interior at blue hour. A massive drafting table made of matte black powder-coated steel sits in the lower-third of the frame, negative space and atmospheric haze fill the upper two-thirds. Faint cyanotype-blue blueprint contour lines appear to float in mid-air as if projected through the haze, fading as they reach the edges. A single cold skylight from camera-right creates volumetric god rays cutting through dust. Composition heavily right-weighted, leaving a large clean dark area on the left for headline type. Shot on 35mm, anamorphic 2.39:1, f/1.4, Cinestill 800T, subtle grain, cold steel-blue with one warm amber highlight glinting off the table edge. Negatives: no people, no text, no logos, no UI, no rendered screens, no rainbow, no neon, no sci-fi.`,

  'bg-features': `${HOUSE}\n\nClose-up transition frame: the right edge of a matte black drafting table in extreme foreground (bottom third of frame), cutting across into a 2/3 top frame that shows paper edges of a large technical drawing just beginning to be visible beyond the table edge. A warm amber highlight lines the polished metal bevel. Shallow depth of field: table edge tack-sharp, drawing soft and ambient. Still-life quality; this is a narrative handoff frame. Shot on 50mm, f/2.8, eye-level, soft directional key from camera-right, cold steel-blue grade with one amber highlight. Negatives: no text, no logos, no UI, no people, no hands, no dramatic lighting, no gradients, no CGI feel.`,

  'bg-bento': `${HOUSE}\n\nNear-abstract arrangement: the corner of a cyanotype-blue technical drawing (lower-left of frame, slightly out of focus), transitioning via a shared pool of shadow into a composition of industrial material samples — a slab of brushed aluminum, raw formwork concrete, a piece of carbon fiber, a wedge of oxidized bronze — stacked in shallow layers across the right side of frame. The materials are in the key light; the drawing is in soft shadow. Amber bronze catches the hot highlight. Transition frame between blueprint and materials worlds. Shot on 85mm, f/4, hard directional key from camera-left, chiaroscuro, cold neutral with one moving warm highlight. Negatives: no text, no labels, no UI, no people, no hands, no tools, no colorful backgrounds, no gradients, no CGI plastic look.`,

  // ---------- Feature-card poster images ----------
  'feature-blueprint': `${HOUSE}\n\nExtreme macro close-up of a single intersection on a technical construction drawing. A vellum tracing paper overlay with crisp ink contour lines rendered in cold cyanotype blue, dimension callouts and tick marks visible along the edge of frame. Paper fibers and subtle creases catch raking light. A brushed aluminum engineer's scale ruler crosses the lower-right corner at 20 degrees, out of focus. Shallow DOF, only a 1cm strip of the drawing tack-sharp. Shot on 100mm macro, f/2.8, single soft key from camera-left, deep shadow right. Square composition, subject lower-third-left. Negatives: no legible text, no real numbers, no logos, no UI, no people, no hands, no cartoon.`,

  'feature-materials': `${HOUSE}\n\nClose-up hero shot of a surface composed of overlapping industrial materials: a slab of brushed aluminum with directional grain, a chunk of raw formwork concrete with aggregate, a piece of carbon fiber twill, and a corner of oxidized bronze. Materials sit in shallow stepped layers on matte black, lit from a hard 45-degree window light from camera-left. Dust motes in the beam, subtle fingerprint on aluminum, a single faint scratch on bronze. The amber-bronze catches the key; everything else in shadow. Shot on 85mm, f/4, deep chiaroscuro, cold ambient bounce. Horizontal framing, subject right-of-center, clean negative space on the left. Negatives: no people, no hands, no tools, no text, no labels, no colorful background, no plastic CGI.`,

  'feature-data': `${HOUSE}\n\nTop-down editorial composition on matte black anodized aluminum. A grid of tiny, precise, embossed square indentations tiles the surface — reading as a physical spreadsheet/data matrix without any legible text. One row subtly highlighted with a thin amber-bronze fill. A single faint light sweep moves diagonally. Micro-scratches and dust near the edge. Braun / Dieter Rams minimalism. Shot on 50mm, f/8, 90-degree overhead, soft diffused overcast skylight, low-contrast, subtle grain. Negatives: no readable text, no numbers, no UI chrome, no computer, no screen, no cursor, no people, no glowing orbs, no sci-fi.`,

  // ---------- Bento tile poster images ----------
  'tile-confidence': `${HOUSE}\n\nMacro product still: a single matte-black analog precision gauge with a minimal arc scale, a thin brass needle at rest on the left. Tick marks engraved, not printed. A soft warm key light picks up the bezel edge. Background dissolves to near-black. No legible numbers, no brand, no markings. Shot on 100mm macro, f/2.8, eye-level, single directional soft key from camera-left, cold neutral with amber accent, subtle grain. Negatives: no text, no numbers legible, no logos, no UI, no hands, no rainbow.`,

  'tile-drm': `${HOUSE}\n\nOverhead top-down on dark kraft paper: a grid of roughly a dozen small hand-drawn cyanotype-blue marker dots, lightly connected by faint graphite trace lines. One dot upper-left and one lower-right highlighted with amber-bronze ink. No text anywhere. Paper fibers and subtle dust visible, a physical diagram pinned on a matte workshop table. Shot on 50mm, f/5.6, 90-degree overhead, soft diffused skylight, cold neutral with one amber accent. Negatives: no text, no labels, no characters, no logos, no UI, no arrows, no words.`,

  'tile-audit': `${HOUSE}\n\nSide-profile macro: a short stack (about 12) of archival ledger cards bound with a waxed amber linen thread through punched holes, sitting on a matte graphite surface. Edge-lit from camera-right; cut edges show fine paper texture. Thread glows slightly warmer. Deep negative space to the left. Shot on 100mm, f/4, eye-level, single hard key from right, cold neutral with warm thread accent, subtle grain. Negatives: no text, no writing, no logos, no UI, no people, no hands, no pages opening, no magic glow.`,

  // ---------- Backgrounds reused ----------
  testimonial: `${HOUSE}\n\nQuiet architectural interior photograph: the corner of a premium engineering office at dusk. Floor-to-ceiling low-iron glass on the left reveals a softly lit construction site in the background, lights just coming on, cranes in silhouette, deep atmospheric haze. Inside the room, a polished concrete floor and one corner of a dark oak-and-steel desk catch a warm amber interior light. Composition dominated by the vast dark window and negative space. Shot on 35mm, f/2, eye-level, long exposure feel, teal-and-amber split grade, Cinestill 800T look. Negatives: no people, no faces, no text, no logos, no UI, no monitors, no phones, no plants, no HDR.`,

  cta: `${HOUSE}\n\nNear-abstract close-up: a single vertical edge of a polished dark surface catches a slow sweep of cool daylight, with a faint amber reflection hinting at a light source off-frame. The left 70% of the composition is near-pure deep graphite black. The right 30% reveals edge texture — brushed titanium grain, a single precise bevel, a millimeter of warm reflection. Shot on 100mm, f/2.8, static locked-off, one hard key from right, subtle grain, cold neutral with one amber accent. Ultra-wide cinematic 21:9. Negatives: no people, no text, no logos, no UI, no buttons, no arrows, no gradients, no neon.`,

  // ---------- Pipeline phased-demo still frames ----------
  'phase-1-upload': `${HOUSE}\n\nOverhead still-life: a folded large-format technical drawing (cyanotype blue lines visible on the edge), a dark matte clipboard, and a single matte-black stylus laid in a precise arrangement on a graphite work surface. Drawing is closed. Soft warm key from upper-right casts long shadows. This is "the drawing arrives." Shot on 50mm, f/5.6, 75-degree overhead, cold neutral with one amber highlight, subtle grain. Negatives: no text legible, no people, no hands, no UI, no logos, no stock-photo cliche.`,

  'phase-2-parse': `${HOUSE}\n\nSame technical drawing now opened flat on the graphite surface, a precision engineer's scale ruler laid across it at a slight angle, and a small magnifying loupe resting over one intersection. Fine cyanotype contour lines fill the frame. Harder light motivated by an overhead task lamp just out of frame. This is "parse and measure." Shot on 50mm, f/4, 75-degree overhead, single hard key from above-left, cold steel-blue grade with amber ruler highlight, subtle grain. Negatives: no legible numbers, no text, no UI, no people, no hands, no annotations.`,

  'phase-3-price': `${HOUSE}\n\nThe drawing remains on the graphite surface. Alongside: three small physical "price tokens" — a brushed aluminum coin, a raw concrete sample cube, a cut piece of oxidized bronze — each on a tiny matte card. A thin amber thread visually connects one detail on the drawing to one of the tokens, implying cost-code matching. Deep negative space upper half. This is "the pipeline finds the prices." Shot on 50mm, f/4, 70-degree overhead, cold neutral grade with warm thread accent, subtle grain. Negatives: no legible text, no numbers, no UI, no people, no hands, no plastic price tags.`,

  'phase-4-export': `${HOUSE}\n\nClosing frame: the drawing now rolled and tied with an amber linen cord, sitting on top of a short stack of cream-colored estimator's paper — the КСС. A matte-black fountain pen lies alongside at a precise angle. Soft warm key from upper-right, long shadows, deep graphite surface. This is "the КСС is ready to send." Shot on 50mm, f/5.6, 75-degree overhead, cold neutral with one amber warm key, subtle grain. Negatives: no text legible, no logos, no writing on the paper, no UI, no people, no hands, no coffee cup, no laptop.`,

  // ---------- Sub-page hero images ----------
  'page-features-hero': `${HOUSE}\n\nWide cinematic shot — physical workshop wall with multiple precision instruments arranged in a horizontal row: a brushed-aluminum engineer's scale ruler, a brass divider, a small machinist's square, a pair of calipers, a single piece of cyanotype blueprint paper pinned with small magnets. Each tool is precisely positioned, in deep low-key chiaroscuro, single warm light from camera-right. Heavy negative space upper-half for headline. Reads as "many tools, one pipeline." Shot on 35mm anamorphic, f/2, Cinestill 800T, subtle grain. Negatives: no people, no logos, no UI, no text legible, no rainbow, no neon.`,

  'page-pipeline-hero': `${HOUSE}\n\nWide cinematic angle on a precision-engineering workshop with a clear horizontal flow from left to right: in the foreground left, a folded technical drawing; in the mid-ground center, a compact server rack with a single warm power LED visible; in the background right, a stack of bound estimator's paper. A single amber thread of light loosely connects them through the haze. Negative space top, room for headline. Reads as "drawing → compute → KSS." Shot on 28mm wide, f/2.8, anamorphic 2.39:1, subtle grain, deep low-key. Negatives: no people, no logos, no UI on the rack, no rendered screens, no neon, no rainbow.`,

  'page-stack-hero': `${HOUSE}\n\nMacro composition top-down: nine small matte-black metal blocks arranged in a 3x3 grid on a graphite surface, each with a thin amber engraved index line at one corner. Each block is the same precision-machined object — they read as crates in a Rust workspace, but only as physical mass and labels-by-position. Single hard key from camera-upper-left, deep chiaroscuro. Heavy negative space margin around the grid. Shot on 50mm, f/8, 90-degree overhead, cold neutral with one warm accent. Negatives: no text legible, no logos, no UI, no numbers, no hands, no people, no rainbow.`,

  'page-changelog-hero': `${HOUSE}\n\nMacro side-profile: a long bound stack of archival ledger cards — about 21 visible from the cut edges — bound with waxed amber linen thread through punched holes, sitting on a matte graphite surface. Edge-lit from the right; cut edges show fine paper texture, the thread glows slightly warmer where it reaches the binding. Some cards subtly thicker than others, suggesting different weights of work. Heavy negative space upper-left. Shot on 100mm, f/4, single hard key from right, cold neutral with warm thread accent, subtle grain. Negatives: no text, no writing, no logos, no UI, no people, no hands, no pages opening, no magic glow.`,
};

export const VIDEO_SHOTS = {
  // ---------- Section background videos (chained continuity, 8s each) ----------
  'video-bg-hero': {
    ref: 'hero.png',
    aspect_ratio: '16:9',
    duration: 8,
    prompt: `${HOUSE}\n\nStatic establishing shot of an empty, dark precision-engineering workshop at blue hour. The matte-black drafting table sits lower-third. Over 8 seconds, execute one very slow dolly-in of 5% magnification, ending with the table dominating the lower-half of frame, a single amber highlight clean on the right edge. [0s] Wide frame, dust motes suspended in the god ray, everything still. [4s] Dolly-in halfway. [8s] Table lower-half, amber edge highlight centered — handoff frame. Style: cinematic, 24fps, anamorphic 2.39:1, shallow DOF, Cinestill 800T, subtle grain. Negatives: no camera shake, no pan, no tilt, no zoom jump, no cuts, no people, no text, no UI, no audio, no flashing, no color shift, no objects moving, no rendered screens, no sci-fi.`,
  },

  'video-bg-features': {
    ref: 'bg-features.png',
    aspect_ratio: '16:9',
    duration: 8,
    prompt: `${HOUSE}\n\nOpen on the close-up of the drafting-table right edge with amber highlight (continuation). Paper edges of a technical drawing are just visible in the lower-third. Over 8 seconds, the camera slowly pushes forward over the edge onto the drawing surface, revealing cyanotype-blue contour lines and dimension callouts. The drawing does not move. [0s] Table edge + amber highlight (matching inbound frame). [4s] Camera drifting forward, contour lines entering frame. [8s] Blueprint macro detail fills frame — handoff to bento section. Style: cinematic 24fps, shallow DOF, cold steel-blue grade with one warm highlight, subtle grain. Negatives: no camera shake, no whip, no zoom jumps, no cuts, no drawings animating, no ink appearing, no people, no hands, no text, no UI, no audio, no flashing.`,
  },

  'video-bg-bento': {
    ref: 'bg-bento.png',
    aspect_ratio: '16:9',
    duration: 8,
    prompt: `${HOUSE}\n\nOpen on the blueprint-plus-materials transition (continuation). Over 8 seconds, the camera slowly pulls back and the plane of focus shifts so the drawing now sits among industrial materials — brushed aluminum, raw formwork concrete, carbon fiber, oxidized bronze — arranged in shallow layers on matte black. Light sweeps slowly left-to-right across the composition. [0s] Blueprint-plus-materials matching the inbound frame. [3s] Pull-back begins, materials creep into frame edges. [6s] Full mood-board composition visible, light reaching center, amber-bronze picks up a highlight. [8s] Hold final — handoff. Style: cinematic 85mm feel, f/4, chiaroscuro, subtle film grain, cold neutral with one moving warm highlight. Negatives: no camera shake, no whip pan, no orbit, no zoom pulse, no cuts, no subject movement, no materials sliding, no rotation, no people, no hands, no tools, no text, no UI, no audio, no flashing.`,
  },

  'video-bg-testimonial': {
    ref: 'testimonial.png',
    aspect_ratio: '16:9',
    duration: 8,
    prompt: `${HOUSE}\n\nArchitectural ambient loop (continuation — camera has moved out of workshop into engineering office at dusk). Over 8 seconds the interior is perfectly still; outside the tall window, two additional distant construction-site lights fade on extremely softly, a gentle drift of haze crosses the far background right-to-left. Crane silhouettes do not move. [0s] Baseline dusk scene, one distant light. [4s] Second distant light fades in softly, haze drifting. [8s] Third distant light just barely visible, haze continuing — handoff. Style: cinematic 35mm, f/2, teal-and-amber split grade, Cinestill 800T look, long-exposure stillness. Negatives: no camera movement, no dolly, no pan, no tilt, no zoom, no people, no reflections of people, no cars, no crane movement, no text, no UI, no audio, no flashing.`,
  },

  'video-bg-cta': {
    ref: 'cta.png',
    aspect_ratio: '21:9',
    duration: 8,
    prompt: `${HOUSE}\n\nNear-abstract close-up: vertical edge of polished dark surface catches a slow sweep of cool daylight over 8 seconds, revealing brushed-titanium grain and a millimeter of warm amber reflection. Composition otherwise still — only light travels. [0s] Surface mostly dark, faint haze and dust in the light. [4s] Light sweep at midpoint, amber reflection strengthening. [8s] Light at far right, amber glint at peak, shadow returning to the left — handoff for loop. Style: cinematic 100mm, f/2.8, static locked-off, one hard key from right, subtle grain, cold neutral with one amber accent. Negatives: no camera shake, no pan, no tilt, no zoom, no cuts, no objects moving, no people, no text, no UI, no audio, no flashing, no gradients pulsing.`,
  },

  // ---------- Widget hover videos (4s, on-hover playback) ----------
  'video-card-drawing': {
    ref: 'feature-blueprint.png',
    aspect_ratio: '1:1',
    duration: 4,
    prompt: `${HOUSE}\n\nMacro technical-drawing loop. Ink lines and paper fibers stay completely still. Over 4 seconds, execute one slow rack focus from foreground ruler bokeh to a sharp blueprint intersection. [0s] Foreground ruler tack-sharp, blueprint soft. [2s] Focus plane mid-travel. [4s] Blueprint intersection tack-sharp. Style: cinematic macro, 24fps, shallow DOF, f/2.8 bokeh, cold steel-blue grade, one amber micro-highlight. Negatives: no camera movement, no zoom, no pan, no paper curling, no ink animating, no people, no hands, no text appearing, no UI, no audio, no flashing.`,
  },

  'video-card-prices': {
    ref: 'feature-materials.png',
    aspect_ratio: '1:1',
    duration: 4,
    prompt: `${HOUSE}\n\nIndustrial materials hero loop. Materials (aluminum, concrete, carbon fiber, bronze) do not move. Slow sweep of warm directional light travels left-to-right over 4 seconds. [0s] Light at far left, most of frame in shadow. [2s] Light reaches center, amber-bronze catches highlight. [4s] Light at far right, shadow returning to left. Style: cinematic 85mm, f/4, chiaroscuro, film grain, cold neutral with one moving warm highlight. Negatives: no camera movement, no camera shake, no zoom, no pan, no orbit, no subjects moving, no rotation, no people, no hands, no tools, no text, no UI, no audio, no flashing.`,
  },

  'video-card-export': {
    ref: 'feature-data.png',
    aspect_ratio: '1:1',
    duration: 4,
    prompt: `${HOUSE}\n\nOverhead static composition on matte black anodized aluminum. The physical grid of embossed cells and micro-scratches are still. Over 4 seconds, the highlighted amber row slowly brightens and a single pinpoint of warm light traces left-to-right along the row, fading as it reaches the end. [0s] Grid baseline, amber row at low glow. [2s] Row fully lit, pinpoint at midpoint. [4s] Pinpoint exits right, row dimming back. Style: cinematic overhead, 50mm, f/8, soft diffused overcast skylight, subtle grain. Negatives: no camera movement, no zoom, no pan, no tilt, no grid moving, no cells shifting, no rotation, no text, no UI, no audio, no flashing.`,
  },

  'video-tile-confidence': {
    ref: 'tile-confidence.png',
    aspect_ratio: '16:9',
    duration: 4,
    prompt: `${HOUSE}\n\nClose-up of a physical analog precision gauge with matte black dial and a thin brass needle. Over 4 seconds the needle travels very slowly from rest on the left to ~80% across the arc, then holds. Dial and tick marks still. A faint amber rim catches on the bezel. [0s] Needle at far-left rest. [3s] Needle reaches 80%, slight overshoot micro-settle. [4s] Needle settled. Style: cinematic macro, 100mm, f/2.8, soft directional key, subtle grain, cold neutral. Negatives: no camera movement, no zoom, no pan, no shake, no numbers or text animating, no UI, no glowing trail, no people, no hands, no audio, no flashing.`,
  },

  'video-tile-drm': {
    ref: 'tile-drm.png',
    aspect_ratio: '16:9',
    duration: 4,
    prompt: `${HOUSE}\n\nTop-down abstract: grid of faint cyanotype-blue marker dots on dark kraft paper. Over 4 seconds, a single amber highlight travels from a marker upper-left to one lower-right, tracing a rule mapping. Paper and markers do not move. [0s] Highlight at origin, faint. [2s] Highlight mid-travel along diagonal, dim afterglow trail. [4s] Highlight at destination, trail faded. Style: cinematic overhead, 50mm, f/5.6, soft skylight, subtle grain, cold neutral with one amber accent. Negatives: no camera movement, no zoom, no pan, no paper movement, no labels animating into text, no UI, no people, no audio, no flashing.`,
  },

  'video-tile-audit': {
    ref: 'tile-audit.png',
    aspect_ratio: '16:9',
    duration: 4,
    prompt: `${HOUSE}\n\nSide profile macro of a short stack of archival ledger cards bound with waxed linen thread, edge-lit from the right. Over 4 seconds, a soft vertical band of warm key light sweeps across the stack's edge from left to right, making the cut edges glow in sequence, then settles. [0s] Edge fully shadowed, only amber thread visible. [2s] Light at midpoint, half the edges glowing. [4s] Light at right, stack edge fully lit, begins to dim. Style: cinematic product still, 100mm, f/4, single directional key, subtle grain, cold neutral with warm accent. Negatives: no camera movement, no zoom, no pan, no shake, no card movement, no pages flipping, no text appearing, no UI, no people, no hands, no audio, no flashing.`,
  },
};
