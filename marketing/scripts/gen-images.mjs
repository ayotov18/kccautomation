#!/usr/bin/env node
// Generate all landing-page still images via OpenRouter (openai/gpt-5.4-image-2).
// Reads prompt definitions from scripts/prompts.mjs and writes PNGs to public/assets/gen/.
// Usage: node scripts/gen-images.mjs [slug1 slug2 ...]

import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { IMAGE_PROMPTS } from './prompts.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUT = path.resolve(__dirname, '../public/assets/gen');

const KEY =
  process.env.OPENROUTER_API_KEY ||
  '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';

async function generateOne(slug, prompt) {
  const outPath = path.join(OUT, `${slug}.png`);
  try {
    await fs.access(outPath);
    console.log(`[skip]  ${slug} — already exists`);
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
      messages: [{ role: 'user', content: [{ type: 'text', text: prompt }] }],
    }),
  });

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`OpenRouter ${res.status}: ${body.slice(0, 500)}`);
  }
  const data = await res.json();
  const imageUrl =
    data.choices?.[0]?.message?.images?.[0]?.image_url?.url ||
    data.choices?.[0]?.message?.content?.find?.((c) => c.type === 'image_url')?.image_url?.url;
  if (!imageUrl) {
    throw new Error(`No image URL in response: ${JSON.stringify(data).slice(0, 400)}`);
  }

  let buf;
  if (imageUrl.startsWith('data:')) {
    buf = Buffer.from(imageUrl.split(',')[1], 'base64');
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
  const want = process.argv.slice(2).length ? process.argv.slice(2) : Object.keys(IMAGE_PROMPTS);
  await Promise.all(
    want.map(async (slug) => {
      if (!IMAGE_PROMPTS[slug]) {
        console.error(`[skip] unknown slug ${slug}`);
        return;
      }
      try {
        await generateOne(slug, IMAGE_PROMPTS[slug]);
      } catch (e) {
        console.error(`[fail] ${slug}: ${e.message}`);
      }
    }),
  );
  console.log('all done');
}

main();
