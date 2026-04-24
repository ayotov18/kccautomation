# KCC Automation — Prompt Engineering Guide

Backgrounds and hero assets for the KCC Automation landing page + short ambient product videos.
Target stack: **image** = `openai/gpt-5.4-image-2` via OpenRouter chat completions, **video** = `bytedance/seedance-2` via KIE.ai `jobs/createTask`.

Brand: dark, industrial-premium, technical, subtle. B2B SaaS for construction (quoting / BOQ / drawing automation).

---

## 0. The House Style Paragraph (prepend to every prompt)

Use this verbatim at the top of every image and video prompt. It locks aesthetic coherence across the whole site.

> **KCC Automation house style —** cinematic industrial-premium aesthetic for a German-engineered B2B construction software. Dark background dominant (#0A0A0C to #14161A), graphite and carbon tones, with restrained accents in desaturated steel-blue (#3E5566) and a single warm highlight in amber-bronze (#B87333) used sparingly. Volumetric atmospheric haze, subtle film grain, 35mm or 50mm lens feel, shallow depth of field, physically-accurate lighting with one motivated key source and soft fill. Materials must read as real: brushed aluminum, anodized titanium, matte concrete, carbon fiber, architectural glass, dark leather, ceramic, blueprint paper, oxidized bronze. Compositions are negative-space heavy, left-weighted or right-weighted to leave room for type. No people's faces, no logos, no text, no rendered UI, no stock-photo cliches, no glossy CGI, no rainbow gradients, no neon cyberpunk, no fantasy. Feels like a Linear / Vercel / Arc landing page crossed with a Leica product photograph shot inside a precision-engineering workshop.

Keep this paragraph identical across assets. Only the per-shot description changes beneath it. This is the #1 thing that makes a landing page feel produced instead of AI-slop.

---

## 1. Prompt Structure Templates

### 1.1 `openai/gpt-5.4-image-2` (gpt-image family)

OpenAI's cookbook is explicit about the ordering. Follow it literally.

**Order of elements (top → bottom):**

1. **Background / scene** — where are we, what surrounds the subject
2. **Subject** — what the image is actually of
3. **Key details** — materials, surface quality, lighting, small props
4. **Camera / photography language** — lens, framing, angle, DOF, film stock
5. **Lighting** — key, fill, ambient, color temperature, time of day
6. **Constraints / negatives** — what must NOT appear

Put literal text inside `"quotes"` or `ALL CAPS` if any word needs to render (usually we want none — constrain with "no text, no typography, no watermark").

**Composition / framing vocabulary that works:**

- `wide establishing shot`, `medium close-up`, `extreme close-up`, `top-down / overhead`, `low-angle`, `eye-level`, `three-quarter view`, `Dutch tilt` (rare, avoid for us)
- `shallow depth of field`, `f/1.4`, `bokeh`, `rack focus`
- `35mm lens` (environmental), `50mm lens` (natural), `85mm lens` (product), `macro lens` (texture hero)
- `left-weighted composition with negative space on the right`, `rule of thirds`, `centered with symmetrical negative space`, `hero subject lower-third`
- `anamorphic 2.39:1 framing`, `cinematic letterbox`, `editorial magazine framing`

**Lighting + color grading vocabulary:**

- Quality: `soft diffused`, `hard directional`, `volumetric god rays`, `practical source`, `motivated lighting`, `rim light`, `back-lit`, `chiaroscuro`
- Mood: `low-key lighting`, `moody`, `golden hour`, `blue hour`, `overcast industrial light`, `skylight through frosted glass`
- Grade: `teal-and-orange grade`, `desaturated steel blue`, `cold neutral with warm accent`, `muted industrial palette`, `natural color balance`, `subtle film grain`, `Kodak Portra 400 look`, `Cinestill 800T look`
- Specificity wins — `one motivated key light from camera-left at 5600K, soft fill at 1/4 ratio, amber rim from behind` will beat `dramatic lighting` every time.

**Material / texture vocabulary (critical for construction-tech feel):**

- `brushed aluminum`, `anodized titanium`, `raw mild steel with mill scale`, `weathered galvanized steel`, `powder-coated matte black`, `oxidized bronze`, `patinated copper`
- `poured concrete`, `polished concrete with aggregate`, `raw formwork concrete`, `microcement`
- `carbon fiber twill weave`, `machined G10`, `sintered ceramic`, `sandblasted glass`, `low-iron architectural glass`
- `vegetable-tanned leather`, `linen book-cloth`, `Forbo linoleum`
- `blueprint paper with cyanotype pigment`, `vellum tracing paper`, `mylar drafting film`
- Always specify **surface state**: `scratched`, `matte`, `satin`, `with fingerprints`, `dust in the grooves`, `subtle wear on the edges`. Imperfection = realism.

**Negative patterns — explicitly exclude these:**

- `no people, no faces, no hands, no body parts` (gpt-image renders faces unevenly; avoid entirely for backgrounds)
- `no text, no typography, no logos, no watermarks, no UI, no buttons`
- `no cartoonish rendering, no stylized illustration, no concept-art feel, no anime, no 3D-render look, no Unreal Engine look, no Octane look`
- `no rainbow gradients, no neon cyberpunk, no sci-fi holograms, no glowing orbs, no fantasy elements`
- `no stock-photo cliches: no men-in-hardhats-pointing, no architect-holding-rolled-blueprints, no handshake shots`
- `no over-saturation, no HDR halos, no lens flare abuse, no heavy retouching`

---

### 1.2 `bytedance/seedance-2` via KIE.ai (image-to-video)

Request shape (confirmed against kie.ai docs):

```json
POST https://api.kie.ai/api/v1/jobs/createTask
Authorization: Bearer <KIE_KEY>

{
  "model": "bytedance/seedance-2",
  "callBackUrl": "https://kccautomation.com/api/kie-callback",
  "input": {
    "prompt": "<house style + shot description + camera language>",
    "reference_image_urls": ["https://.../hero.png"],
    "aspect_ratio": "16:9",
    "resolution": "1080p",
    "duration": 6,
    "generate_audio": false
  }
}
```

Enums confirmed: `resolution` ∈ `480p | 720p | 1080p`, `aspect_ratio` ∈ `1:1 | 4:3 | 3:4 | 16:9 | 9:16 | 21:9 | adaptive`, `duration` ∈ `4..15`. `reference_image_urls` accepts up to 9 entries. For landing-page ambient loops: `16:9`, `1080p`, `duration: 6`, `generate_audio: false`.

**Seedance prompt structure (the formula that works):**

```
[Subject] + [Action / what moves] + [Scene] + [Camera language] + [Style & atmosphere]
```

For subtle landing-page ambient motion, **the "Action" part is 90% of the work** — you must aggressively under-describe motion. Seedance will invent motion if you under-specify it. Name one slow thing and lock everything else.

**Camera / motion vocabulary (verbatim terms Seedance parses well):**

| Intent | Say this |
|---|---|
| Nearly still (best for web bg) | `locked-off static camera`, `tripod-mounted, no camera movement` |
| Slow approach | `very slow dolly-in, 5% over the duration`, `gradual push-in` |
| Slow retreat | `slow dolly-out`, `gentle pull-back` |
| Horizontal reveal | `slow lateral pan left to right`, `gentle tracking shot parallel to subject` |
| Depth / parallax | `slow parallax shift, foreground drifting slower than background` |
| Vertical reveal | `slow tilt-up from floor to ceiling`, `crane up` |
| Orbit (use rarely) | `subtle arc shot, 10-degree orbit around the subject` |
| Focus change | `rack focus from foreground to background over 3 seconds` |
| Environmental motion only | `camera locked, only [steam / dust motes / light ray] drifting` |

**Timeline prompting** (works extremely well for 6–8 second shots — from MindStudio's guide):

```
[0s] Establish — static wide frame of [subject], [lighting].
[2s] Camera begins very slow dolly-in at 5% magnification.
[4s] [Environmental element] drifts across frame — steam, dust, cursor, etc.
[6s] Hold on final frame.
Style: cinematic, 24fps, shallow depth of field, anamorphic, film grain.
```

Use 1 timestamp every ~2 seconds for 6s clips. More than 3 beats in a 6s clip will look frantic.

**Lighting + atmosphere language for Seedance:**

`volumetric lighting`, `motivated practical source`, `god rays through dust`, `soft skylight from frosted industrial window`, `warm amber key, cool teal fill`, `high-key ambient haze`, `low-key chiaroscuro`, `film grain`, `anamorphic lens flare subtle and infrequent`, `shallow depth of field`, `f/1.4 bokeh`

**Seedance negative patterns — must exclude (append to prompt):**

`no camera shake, no handheld movement, no whip pans, no fast zooms, no cuts, no lens switch, no people entering frame, no text overlays, no UI animation, no glitch effects, no rapid color shifts, no flashing lights, no morphing objects, no subjects turning to face camera, no speech, no music, no audio`

Seedance **will** add motion you didn't ask for if you leave negative space. Always include the no-list. Seedance is also trained to do dramatic action — explicitly saying `ambient`, `meditative`, `contemplative pace`, `slow-cinema vibe` biases it toward the subtle end.

---

## 2. Ten Ready-to-Use Image Prompts for `openai/gpt-5.4-image-2`

Each is self-contained. Paste `[HOUSE_STYLE]` = the paragraph from Section 0 at the top of every call. All prompts assume `size: 1792x1024` (or closest wide) for backgrounds, `size: 1024x1024` for feature cards.

### Prompt 1 — Hero Background (primary above-the-fold)

```
[HOUSE_STYLE]

Wide cinematic establishing shot of a dark precision-engineering workshop interior at blue hour. Long exposure of an industrial workspace: a massive drafting table made of matte black powder-coated steel sits in the lower-third of the frame, negative space and atmospheric haze fill the upper two-thirds. Extending outward from the table, faint cyanotype-blue blueprint contour lines appear to float in mid-air as if projected through the haze, fading as they reach the edges of the frame. A single cold skylight from camera-right creates volumetric god rays cutting through dust. Background dissolves into deep graphite black. Composition is heavily right-weighted, leaving a large clean dark area on the left for headline type.

Shot on 35mm, anamorphic 2.39:1 framing, f/1.4 shallow depth of field, Cinestill 800T color science, subtle grain, cold steel-blue palette with one warm amber highlight glinting off the table edge.

Negatives: no people, no faces, no text, no logos, no UI, no rendered screens, no rainbow, no neon, no sci-fi hologram, no cyberpunk, no stock-photo cliches, no over-saturation, no HDR halos, no 3D-render look.
```

### Prompt 2 — Feature Card: Drawing / Blueprint Motion (variant A, macro)

```
[HOUSE_STYLE]

Extreme macro close-up of a single intersection on a technical construction drawing. A vellum tracing paper overlay with crisp ink contour lines rendered in cold cyanotype blue, dimension callouts and tick marks visible along the edge of frame. Paper fibers and subtle creases catch the raking light. A brushed aluminum engineer's scale ruler crosses the lower-right corner of frame at a 20-degree angle, out of focus. Shallow depth of field, only a 1cm strip of the drawing is tack-sharp, everything else falls to soft bokeh.

Shot on 100mm macro lens, f/2.8, single directional soft key from camera-left at 5200K, deep shadow on the right. Background dissolves to deep graphite. Square composition, subject in lower-third-left.

Negatives: no text legible, no real numbers, no logos, no UI, no people, no hands, no colorful pens, no cartoon, no flat illustration.
```

### Prompt 3 — Feature Card: Drawing / Blueprint Motion (variant B, abstract lines)

```
[HOUSE_STYLE]

Abstract architectural detail: a dark matte surface covered with a field of faint intersecting orthographic projection lines — thin, precise, desaturated steel-blue vector lines forming an unreadable technical drawing, as if a BIM model was flattened onto carbon fiber. Some lines are slightly brighter at the nodes where they intersect, implying data points. A single amber-bronze filament of light traces one path through the composition, suggesting automation flow. Heavy negative space top-right.

Shot as editorial flat-lay photograph, top-down 90-degree overhead, 50mm lens, soft overcast skylight, very low contrast, matte finish, subtle grain.

Negatives: no text, no numbers, no legible drawing content, no UI, no cursor, no neon glow, no cyberpunk aesthetic, no 3D-render look, no Unreal Engine, no Octane.
```

### Prompt 4 — Feature Card: Materials / Industrial (variant A, metal texture hero)

```
[HOUSE_STYLE]

Close-up hero shot of a surface composed of overlapping industrial materials arranged like a physical mood board: a slab of brushed aluminum with directional grain, a chunk of raw formwork concrete with aggregate visible, a piece of carbon fiber twill, and a corner of oxidized bronze. The materials sit in shallow stepped layers on a matte black surface, lit from a hard 45-degree window light from camera-left. Dust motes visible in the light beam, subtle fingerprint on the aluminum, a single faint scratch on the bronze — real-world wear. The amber-bronze piece catches the key light; everything else sits in shadow.

Shot on 85mm, f/4, single hard directional key, no fill, deep chiaroscuro, cold ambient bounce. Horizontal framing, subject right-of-center, clean negative space on the left.

Negatives: no people, no hands, no tools, no text, no labels, no price tags, no colorful background, no gradient, no rendered CGI-look, no plastic-y highlights.
```

### Prompt 5 — Feature Card: Materials / Industrial (variant B, concrete + steel architecture)

```
[HOUSE_STYLE]

Architectural fragment: a low-angle close-up of a poured concrete column meeting a raw steel I-beam at a clean weld seam. The concrete shows the texture of the plywood formwork — grain, tie-hole plugs, subtle pour lines. The steel has light mill scale and a satin matte finish. Cold northern daylight from a frosted industrial window wraps the scene. Atmospheric haze in the deep background implies scale.

Shot on 24mm wide lens at f/5.6, low angle looking up, shallow perspective compression. Color grade: cold neutral with faint steel-blue cast, single warm highlight where sunlight catches the steel flange.

Negatives: no people, no hardhats, no safety vests, no tools, no text, no logos, no yellow caution tape, no graffiti, no stock-construction-site cliche, no HDR.
```

### Prompt 6 — Feature Card: Data / Automation (variant A, abstract data surface)

```
[HOUSE_STYLE]

Top-down editorial composition on matte black anodized aluminum. A grid of tiny, precise, embossed square indentations tiles the surface — reading as a physical spreadsheet or data matrix without any legible text. One row of cells is subtly highlighted with a thin amber-bronze fill, as if flagged by an automation. A single faint light sweep moves diagonally across the grid. The surface has micro-scratches and a single piece of dust near the edge. Extremely clean, Braun / Dieter Rams, industrial minimalism.

Shot on 50mm, f/8, perfectly overhead 90-degree angle, soft diffused overcast skylight, low-contrast grade, subtle grain.

Negatives: no readable text, no numbers, no UI chrome, no computer, no laptop, no screen, no cursor, no people, no hands, no glowing orbs, no neon, no futuristic sci-fi.
```

### Prompt 7 — Feature Card: Data / Automation (variant B, process flow implied)

```
[HOUSE_STYLE]

Macro still-life: three physical tokens arranged in a linear flow on a matte graphite surface — on the left, a small stack of cyanotype blueprint paper with one corner curled; in the middle, a precision brass gear the size of a coin; on the right, a single polished black ceramic tile. A single thin line of soft light connects the three objects across the surface, implying a pipeline. Materials read as tactile and real — paper fibers, brushed brass patina, ceramic gloss microreflection. Deep shadow occupies the upper half of frame.

Shot on 85mm, f/2.8, key light low from camera-right creating long horizontal shadows, cold ambient bounce. Letterbox 2.35:1 framing, symmetrical negative space above.

Negatives: no text, no labels, no people, no hands, no computer, no screen, no UI, no arrows drawn on the surface, no cartoon, no diagram, no flowchart graphic.
```

### Prompt 8 — Feature Card: Data / Automation (variant C, BOQ / quoting subtle reference)

```
[HOUSE_STYLE]

Layered still-life from above: a sheet of cream-colored estimator's paper with faint grid lines at the bottom of the stack; on top, a folded technical drawing in cold blue ink; on top of that, a single small dark concrete sample cube (5cm) and a matte black fountain pen at a precise 30-degree angle. Paper edges slightly worn, pen lightly scratched. A sliver of amber-bronze light from camera-upper-left catches only the edge of the concrete cube.

Shot on 50mm, f/4, overhead-angled view at 75 degrees (not fully flat), soft diffuse key from upper-left, long soft shadows falling to lower-right, cold neutral grade with a single warm accent.

Negatives: no legible text on paper, no handwriting, no numbers, no logos, no people, no hands, no calculator, no coffee cup, no laptop, no iPhone, no stock-photo desk cliche.
```

### Prompt 9 — Testimonial Section Background

```
[HOUSE_STYLE]

Quiet architectural interior photograph: the corner of a premium engineering office at dusk. Floor-to-ceiling low-iron glass on the left reveals a softly lit construction site in the background, lights just coming on, cranes in silhouette, deep atmospheric haze. Inside the room, a polished concrete floor and one corner of a dark oak-and-steel desk catch a warm amber interior light. The composition is dominated by the vast dark window and the negative space of the room itself. Very low-key, contemplative, cinematic.

Shot on 35mm, f/2, eye-level, long exposure feel with still subject, teal-and-amber split grade, Cinestill 800T look, subtle anamorphic aura around the highlights.

Negatives: no people, no faces, no text, no logos, no UI, no rendered screens, no monitors, no phones, no plants, no stock-office cliche, no HDR, no rainbow.
```

### Prompt 10 — CTA / Footer Background

```
[HOUSE_STYLE]

Near-abstract close-up: a single vertical edge of a polished dark surface catches a slow sweep of cool daylight, with a faint amber reflection hinting at a light source off-frame. The left 70% of the composition is near-pure deep graphite black with the slightest atmospheric haze and dust particles suspended in light. The right 30% reveals the edge texture — brushed titanium grain, a single precise bevel, a millimeter of warm reflection. Meditative, patient, cinematic.

Shot on 100mm, f/2.8, static locked-off frame, one hard key from right, no fill, subtle film grain, cold neutral grade with one amber accent. Ultra-wide cinematic 21:9 framing, heavy negative space on the left.

Negatives: no people, no text, no logos, no UI, no buttons, no CTA shapes, no arrows, no gradients, no neon, no sci-fi, no fantasy, no over-saturation.
```

---

## 3. Five Video Prompts for `bytedance/seedance-2` (image-to-video, 16:9, 1080p, 6s, no audio)

Each assumes one of the Section 2 images is passed as `reference_image_urls[0]`. The prompt tells Seedance **what minimal motion to add** without changing the composition. Every prompt ends with the same no-list — Seedance leaks motion into anything you don't forbid.

### Video Prompt 1 — Hero Background: Drifting Haze + Slow Push

Reference image: Prompt 1 output.

```
[HOUSE_STYLE — condensed to 2 sentences for Seedance]

Premium industrial-premium landing-page ambient loop. Dark workshop interior; the matte black drafting table stays fixed in the lower-third; only atmospheric haze and volumetric dust motes drift slowly left-to-right across the god-ray.

Camera: locked-off static for the first 2 seconds, then a very slow dolly-in of roughly 4% magnification over the remaining 4 seconds, ending near-frozen. No lateral movement, no tilt.

[0s] Static wide frame, dust particles suspended in the god ray.
[2s] Dolly-in begins at 1 unit per second, imperceptible at first.
[4s] Faint cyanotype-blue contour lines in the haze glow slightly brighter, then fade.
[6s] Hold final frame.

Style: cinematic, 24fps, anamorphic 2.39:1, shallow depth of field, Cinestill 800T grade, subtle grain, contemplative slow-cinema pace.

Negatives: no camera shake, no handheld, no whip pan, no fast zoom, no cuts, no lens switch, no people entering frame, no text, no UI animation, no glitch, no flashing lights, no color shift, no morphing, no audio, no music, no speech, no subject turning, no glowing orbs, no sci-fi holograms.
```

### Video Prompt 2 — Blueprint Macro: Rack Focus Breathing

Reference image: Prompt 2 (macro blueprint close-up).

```
[HOUSE_STYLE — condensed]

Macro technical-drawing loop. The cyanotype contour lines and paper fibers stay completely still — nothing on the page moves, ever.

Camera: locked-off. Over 6 seconds, execute one extremely slow rack focus from a sharp foreground point in the lower-right to a sharp blueprint intersection in the upper-left, then hold. Bokeh re-shapes softly as focus travels. No translation, no pan.

[0s] Foreground engineer's scale tack-sharp, blueprint softly blurred.
[3s] Focus plane traveling through the middle of frame.
[6s] Blueprint intersection tack-sharp, scale dissolved into bokeh.

Style: cinematic macro, 24fps, shallow DOF, f/2.8 bokeh, soft film grain, cold steel-blue grade with one amber highlight unchanging.

Negatives: no subject movement, no paper curling, no ink drawing itself in, no animation of lines, no camera shake, no zoom, no pan, no tilt, no people, no hands entering, no text appearing, no UI, no audio, no music, no flash, no color shift.
```

### Video Prompt 3 — Materials Hero: Parallax Light Sweep

Reference image: Prompt 4 or 5 (materials composition).

```
[HOUSE_STYLE — condensed]

Industrial materials hero loop. The materials themselves (aluminum, concrete, carbon fiber, bronze) do not move at all. Only a single slow sweep of warm directional light travels across the composition from camera-left to camera-right over the 6 seconds, revealing surface texture as it goes.

Camera: locked-off static. No translation, no zoom, no orbit.

[0s] Light at the far left edge, most of frame still in shadow.
[3s] Light reaches the center, amber-bronze material catches a highlight.
[6s] Light at the far right, shadow returning to the left side.

Style: cinematic product still, 24fps, 85mm macro feel, f/4, chiaroscuro, subtle film grain, cold neutral grade with one warm moving highlight, meditative pace.

Negatives: no camera movement, no camera shake, no zoom, no pan, no tilt, no dolly, no orbit, no subject movement, no materials sliding, no rotation, no people, no hands, no tools entering frame, no text, no UI, no audio, no flashing lights, no rapid color change, no lens flare pulsing.
```

### Video Prompt 4 — Data / Automation: Single Amber Trace Travels the Flow

Reference image: Prompt 7 (three tokens in a line).

```
[HOUSE_STYLE — condensed]

Automation pipeline loop. The three objects (blueprint stack, brass gear, ceramic tile) stay perfectly still. Only the thin line of soft warm light that connects them animates, traveling left-to-right along the existing path from the blueprint to the gear to the tile over 6 seconds, then softly fading out.

Camera: locked-off static, tripod, no movement.

[0s] Light just beginning to form at the blueprint stack, very faint.
[2s] Light crosses the midpoint, brass gear catches a gentle micro-reflection.
[4s] Light arrives at the ceramic tile, tile gains a soft glow on its polished edge.
[6s] Light fades out, composition returns to baseline, loop-ready.

Style: cinematic still-life, 24fps, 85mm, f/2.8, soft film grain, cold graphite grade with one amber traveling highlight, contemplative.

Negatives: no objects moving, no gear rotation, no paper flipping, no tile tilting, no camera movement, no zoom, no pan, no dolly, no orbit, no people, no hands, no tools, no text appearing, no arrows drawn, no UI overlay, no cursor, no audio, no music, no glitch, no flashing, no rainbow, no neon pulse.
```

### Video Prompt 5 — Testimonial / CTA: Dusk Window with Distant Site

Reference image: Prompt 9 (architectural interior at dusk).

```
[HOUSE_STYLE — condensed]

Architectural ambient loop. The interior of the office is perfectly still. Outside the window, construction-site lights in the distance shift extremely subtly — one or two additional pinpoint lights fade on over the 6 seconds, and a very faint drift of atmospheric haze moves right-to-left in the far background. The crane silhouettes do not move.

Camera: locked-off static. No movement at all.

[0s] Dusk scene as-is, one construction light visible in the distance.
[3s] A second distant site light fades in very softly, interior unchanged.
[6s] A third distant light just barely appears, haze drift continuing.

Style: cinematic architectural, 24fps, 35mm, f/2, teal-and-amber split grade, Cinestill 800T look, long-exposure stillness, meditative dusk pace.

Negatives: no camera movement, no dolly, no pan, no tilt, no zoom, no people in the room, no reflections of people in the glass, no cars passing, no crane rotating, no text, no UI, no on-screen phone, no monitor turning on, no audio, no music, no ambient sound cues, no flashing, no rapid color shift, no over-saturation.
```

---

## 4. Implementation Notes

### OpenRouter call for gpt-5.4-image-2

```javascript
const resp = await fetch("https://openrouter.ai/api/v1/chat/completions", {
  method: "POST",
  headers: {
    "Authorization": `Bearer ${OPENROUTER_KEY}`,
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    model: "openai/gpt-5.4-image-2",
    modalities: ["image", "text"],
    messages: [{ role: "user", content: PROMPT_FROM_SECTION_2 }],
  }),
});
// Image comes back as base64 data URL in: data.choices[0].message.images[0].image_url.url
```

Note: as of research date, OpenRouter's `gpt-5.4-image-2` model card does not expose aspect ratio or size parameters directly — control composition via the prompt (`wide 16:9 cinematic letterbox`, `square centered`, etc.). If exact pixel dimensions are needed, call OpenAI's direct API for `gpt-image-2` with `size: "1792x1024"`.

### KIE.ai call for seedance-2

```javascript
const task = await fetch("https://api.kie.ai/api/v1/jobs/createTask", {
  method: "POST",
  headers: {
    "Authorization": `Bearer ${KIE_KEY}`,
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    model: "bytedance/seedance-2",
    callBackUrl: "https://kccautomation.com/api/kie-callback",
    input: {
      prompt: PROMPT_FROM_SECTION_3,
      reference_image_urls: ["https://cdn.kccautomation.com/stills/hero.png"],
      aspect_ratio: "16:9",
      resolution: "1080p",
      duration: 6,
      generate_audio: false,
    },
  }),
}).then(r => r.json());
// returns { code: 200, data: { taskId: "..." } } — poll or wait for callback
```

### Iteration workflow

1. Generate the **hero still** first with Prompt 1. Approve the look.
2. Copy the hero's color/material language into the House Style paragraph if anything shifts.
3. Generate feature cards (2–8) using the locked house style. Reject anything with a face, text, or neon.
4. For each approved still, pass its CDN URL as `reference_image_urls[0]` into a Seedance call.
5. First video pass: start from Prompt 1 video template at `duration: 4`. If motion is too strong, add more items to the negatives list. If too static, extend to `duration: 6` and loosen one motion beat.
6. Loop-ready cut: have the video start and end on visually identical frames so the HTML `<video loop>` doesn't jar.

### Common failure modes and the fix

| Symptom | Fix |
|---|---|
| Image looks like a 3D render / plastic | Add `real photograph`, `analog film`, and specific lens + grain language; add `no CGI look, no Octane, no Unreal` to negatives |
| Rainbow neon leaks in | Add `desaturated`, `muted palette`, `no neon, no cyberpunk, no RGB` |
| Faces or hands appear anyway | Be stricter: `empty scene, no people, no human presence, no body parts including hands, arms, legs, torsos` |
| Video has camera shake despite "locked-off" | Add `tripod mounted, cinema robot motion-control, no handheld micro-jitter`; reduce duration to 4s |
| Video animates text / draws lines | Add `all text, all drawings, all graphics remain completely static` + the existing negatives block |
| Subject rotates toward camera | Add `subject facing away from camera throughout, no subject rotation, no character turning` |

---

## 5. Reference Links (verified real URLs)

**OpenAI / gpt-image family:**

- OpenAI Cookbook — *GPT Image Generation Models Prompting Guide* (the canonical one): <https://developers.openai.com/cookbook/examples/multimodal/image-gen-models-prompting-guide>
- OpenAI Cookbook — *gpt-image-1.5 Prompting Guide*: <https://developers.openai.com/cookbook/examples/multimodal/image-gen-1.5-prompting_guide>
- Raw notebook on GitHub: <https://github.com/openai/openai-cookbook/blob/main/examples/multimodal/image-gen-models-prompting-guide.ipynb>
- OpenAI Platform prompt engineering guide: <https://platform.openai.com/docs/guides/prompt-engineering>

**OpenRouter:**

- Image generation guide: <https://openrouter.ai/docs/guides/overview/multimodal/image-generation>
- Multimodal overview: <https://openrouter.ai/docs/guides/overview/multimodal/overview>
- `gpt-5.4-image-2` model card: <https://openrouter.ai/openai/gpt-5.4-image-2>
- Image models collection: <https://openrouter.ai/collections/image-models>
- Server-side image generation tool: <https://openrouter.ai/docs/guides/features/server-tools/image-generation>

**ByteDance Seedance:**

- Official site with tech report link: <https://seed.bytedance.com/en/seedance>
- Tech report release post: <https://seed.bytedance.com/en/blog/tech-report-of-seedance-1-0-is-now-publicly-available>
- ByteDance ModelArk / BytePlus — *Seedance 1.0-lite Prompt Guide*: <https://docs.byteplus.com/en/docs/ModelArk/1587797>

**KIE.ai (your actual video provider):**

- Seedance 2 docs: <https://docs.kie.ai/market/bytedance/seedance-2>
- Seedance 2 Fast variant: <https://docs.kie.ai/market/bytedance/seedance-2-fast>
- Seedance 1.5 Pro docs: <https://docs.kie.ai/market/bytedance/seedance-1.5-pro>
- KIE.ai getting-started / auth: <https://docs.kie.ai/>
- Seedance 2 product page: <https://kie.ai/seedance-2-0>

**Community prompt libraries / practical guides:**

- GitHub — *awesome-seedance-2-prompts* (2000+ curated prompts): <https://github.com/YouMind-OpenLab/awesome-seedance-2-prompts>
- MindStudio — *Timeline Prompting with Seedance 2.0*: <https://www.mindstudio.ai/blog/timeline-prompting-seedance-2-cinematic-ai-video>
- imagine.art — *Seedance 2.0 Prompt Guide (70 ready-to-use prompts)*: <https://www.imagine.art/blogs/seedance-2-0-prompt-guide>
- Akool — *Seedance 1.0 Video Prompt Guide*: <https://akool.com/blog-posts/seedance-1-0-video-prompt-guide>
- Veed — *Seedance 1.0 Prompting Guide in 6 steps*: <https://www.veed.io/learn/seedance-1-0-prompts>
- AI/ML API — *Master Your Video Creations with Seedance 1.0 Lite*: <https://aimlapi.com/blog/master-your-video-creations-with-seedance-1-0-lite-a-comprehensive-prompt-guide>
- SeedancePro — *Prompt Guide*: <https://www.seedancepro.net/seedance/prompt-guide>
