#!/usr/bin/env node
// Generate landing-page background images via OpenRouter (openai/gpt-5.4-image-2).
// Writes PNG files into marketing/public/assets/gen/.
// Usage: node scripts/gen-images.mjs [slug1 slug2 ...]

import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUT = path.resolve(__dirname, '../public/assets/gen');

const KEY = process.env.OPENROUTER_API_KEY || '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';

const HOUSE = `KCC Automation house style — cinematic industrial-premium aesthetic for a German-engineered B2B construction software. Dark background dominant (#0A0A0C to #14161A), graphite and carbon tones, with restrained accents in desaturated steel-blue (#3E5566) and a single warm highlight in amber-bronze (#B87333) used sparingly. Volumetric atmospheric haze, subtle film grain, 35mm or 50mm lens feel, shallow depth of field, physically-accurate lighting with one motivated key source and soft fill. Materials must read as real: brushed aluminum, anodized titanium, matte concrete, carbon fiber, architectural glass, dark leather, ceramic, blueprint paper, oxidized bronze. Compositions are negative-space heavy, left-weighted or right-weighted to leave room for type. No people's faces, no logos, no text, no rendered UI, no stock-photo cliches, no glossy CGI, no rainbow gradients, no neon cyberpunk, no fantasy. Feels like a Linear / Vercel / Arc landing page crossed with a Leica product photograph shot inside a precision-engineering workshop.`;

const PROMPTS = {
  hero: `${HOUSE}

Wide cinematic establishing shot of a dark precision-engineering workshop interior at blue hour. Long exposure of an industrial workspace: a massive drafting table made of matte black powder-coated steel sits in the lower-third of the frame, negative space and atmospheric haze fill the upper two-thirds. Extending outward from the table, faint cyanotype-blue blueprint contour lines appear to float in mid-air as if projected through the haze, fading as they reach the edges of the frame. A single cold skylight from camera-right creates volumetric god rays cutting through dust. Background dissolves into deep graphite black. Composition is heavily right-weighted, leaving a large clean dark area on the left for headline type.
Shot on 35mm, anamorphic 2.39:1 framing, f/1.4 shallow depth of field, Cinestill 800T color science, subtle grain, cold steel-blue palette with one warm amber highlight glinting off the table edge.
Negatives: no people, no faces, no text, no logos, no UI, no rendered screens, no rainbow, no neon, no sci-fi hologram, no cyberpunk, no stock-photo cliches, no over-saturation, no HDR halos, no 3D-render look.`,

  'feature-blueprint': `${HOUSE}

Extreme macro close-up of a single intersection on a technical construction drawing. A vellum tracing paper overlay with crisp ink contour lines rendered in cold cyanotype blue, dimension callouts and tick marks visible along the edge of frame. Paper fibers and subtle creases catch the raking light. A brushed aluminum engineer's scale ruler crosses the lower-right corner of frame at a 20-degree angle, out of focus. Shallow depth of field, only a 1cm strip of the drawing is tack-sharp, everything else falls to soft bokeh.
Shot on 100mm macro lens, f/2.8, single directional soft key from camera-left at 5200K, deep shadow on the right. Background dissolves to deep graphite. Square composition, subject in lower-third-left.
Negatives: no legible text, no real numbers, no logos, no UI, no people, no hands, no colorful pens, no cartoon, no flat illustration.`,

  'feature-materials': `${HOUSE}

Close-up hero shot of a surface composed of overlapping industrial materials arranged like a physical mood board: a slab of brushed aluminum with directional grain, a chunk of raw formwork concrete with aggregate visible, a piece of carbon fiber twill, and a corner of oxidized bronze. The materials sit in shallow stepped layers on a matte black surface, lit from a hard 45-degree window light from camera-left. Dust motes visible in the light beam, subtle fingerprint on the aluminum, a single faint scratch on the bronze — real-world wear. The amber-bronze piece catches the key light; everything else sits in shadow.
Shot on 85mm, f/4, single hard directional key, no fill, deep chiaroscuro, cold ambient bounce. Horizontal framing, subject right-of-center, clean negative space on the left.
Negatives: no people, no hands, no tools, no text, no labels, no price tags, no colorful background, no gradient, no rendered CGI-look, no plastic-y highlights.`,

  'feature-data': `${HOUSE}

Top-down editorial composition on matte black anodized aluminum. A grid of tiny, precise, embossed square indentations tiles the surface — reading as a physical spreadsheet or data matrix without any legible text. One row of cells is subtly highlighted with a thin amber-bronze fill, as if flagged by an automation. A single faint light sweep moves diagonally across the grid. The surface has micro-scratches and a single piece of dust near the edge. Extremely clean, Braun / Dieter Rams, industrial minimalism.
Shot on 50mm, f/8, perfectly overhead 90-degree angle, soft diffused overcast skylight, low-contrast grade, subtle grain.
Negatives: no readable text, no numbers, no UI chrome, no computer, no laptop, no screen, no cursor, no people, no hands, no glowing orbs, no neon, no futuristic sci-fi.`,

  testimonial: `${HOUSE}

Quiet architectural interior photograph: the corner of a premium engineering office at dusk. Floor-to-ceiling low-iron glass on the left reveals a softly lit construction site in the background, lights just coming on, cranes in silhouette, deep atmospheric haze. Inside the room, a polished concrete floor and one corner of a dark oak-and-steel desk catch a warm amber interior light. The composition is dominated by the vast dark window and the negative space of the room itself. Very low-key, contemplative, cinematic.
Shot on 35mm, f/2, eye-level, long exposure feel with still subject, teal-and-amber split grade, Cinestill 800T look, subtle anamorphic aura around the highlights.
Negatives: no people, no faces, no text, no logos, no UI, no rendered screens, no monitors, no phones, no plants, no stock-office cliche, no HDR, no rainbow.`,

  cta: `${HOUSE}

Near-abstract close-up: a single vertical edge of a polished dark surface catches a slow sweep of cool daylight, with a faint amber reflection hinting at a light source off-frame. The left 70% of the composition is near-pure deep graphite black with the slightest atmospheric haze and dust particles suspended in light. The right 30% reveals the edge texture — brushed titanium grain, a single precise bevel, a millimeter of warm reflection. Meditative, patient, cinematic.
Shot on 100mm, f/2.8, static locked-off frame, one hard key from right, no fill, subtle film grain, cold neutral grade with one amber accent. Ultra-wide cinematic 21:9 framing, heavy negative space on the left.
Negatives: no people, no text, no logos, no UI, no buttons, no CTA shapes, no arrows, no gradients, no neon, no sci-fi, no fantasy, no over-saturation.`,
};

async function generateOne(slug, prompt) {
  const outPath = path.join(OUT, `${slug}.png`);
  try {
    await fs.access(outPath);
    console.log(`[skip] ${slug} — already exists`);
    return outPath;
  } catch {}

  console.log(`[start] ${slug}`);
  const t0 = Date.now();
  const res = await fetch('https://openrouter.ai/api/v1/chat/completions', {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${KEY}`,
      'Content-Type': 'application/json',
      'HTTP-Referer': 'https://kccgen.xyz',
      'X-Title': 'KCC Marketing Asset Generation',
    },
    body: JSON.stringify({
      model: 'openai/gpt-5.4-image-2',
      modalities: ['image', 'text'],
      messages: [
        {
          role: 'user',
          content: [{ type: 'text', text: prompt }],
        },
      ],
    }),
  });

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`OpenRouter ${res.status} on ${slug}: ${body.slice(0, 500)}`);
  }
  const data = await res.json();
  const imageUrl =
    data.choices?.[0]?.message?.images?.[0]?.image_url?.url ||
    data.choices?.[0]?.message?.content?.find?.((c) => c.type === 'image_url')?.image_url?.url;

  if (!imageUrl) {
    throw new Error(`No image URL in response for ${slug}: ${JSON.stringify(data).slice(0, 400)}`);
  }

  let buf;
  if (imageUrl.startsWith('data:')) {
    const base64 = imageUrl.split(',')[1];
    buf = Buffer.from(base64, 'base64');
  } else {
    const imgRes = await fetch(imageUrl);
    buf = Buffer.from(await imgRes.arrayBuffer());
  }

  await fs.writeFile(outPath, buf);
  console.log(`[done]  ${slug} (${((Date.now() - t0) / 1000).toFixed(1)}s, ${(buf.length / 1024).toFixed(0)} KB)`);
  return outPath;
}

async function main() {
  await fs.mkdir(OUT, { recursive: true });
  const want = process.argv.slice(2).length ? process.argv.slice(2) : Object.keys(PROMPTS);

  const tasks = want.map(async (slug) => {
    if (!PROMPTS[slug]) {
      console.error(`[skip] unknown slug ${slug}`);
      return;
    }
    try {
      await generateOne(slug, PROMPTS[slug]);
    } catch (e) {
      console.error(`[fail] ${slug}: ${e.message}`);
    }
  });

  await Promise.all(tasks);
  console.log('all done');
}

main();
