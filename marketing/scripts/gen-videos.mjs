#!/usr/bin/env node
// Generate landing-page ambient video loops via KIE.ai (bytedance/seedance-2).
// Uses previously-generated images as reference_image_urls.
// Writes MP4 files into marketing/public/assets/gen/.

import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUT = path.resolve(__dirname, '../public/assets/gen');

const KIE_KEY = process.env.KIE_API_KEY || '92d86ec39c86d80da972c68ef8c74a86';
// The KIE seedance input wants HTTP-reachable URLs. We don't have a public host for
// the local PNGs, so we upload each ref image to a free temporary host (0x0.st)
// and feed the hosted URL to Seedance. Files are ephemeral, which is fine for
// one-off marketing asset generation.

// Use aws CLI to upload — correct SigV4 implementation, no node-side crypto.
import { execFileSync } from 'node:child_process';

const AWS = {
  region: process.env.AWS_REGION || 'us-east-1',
  bucket: process.env.S3_BUCKET || 'kcc-files-prod',
  accessKey: process.env.AWS_ACCESS_KEY_ID,
  secretKey: process.env.AWS_SECRET_ACCESS_KEY,
};
if (!AWS.accessKey || !AWS.secretKey) {
  console.error('Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY before running.');
  process.exit(1);
}

async function uploadToHost(filePath) {
  const key = `marketing-refs/${Date.now()}-${path.basename(filePath)}`;
  const s3Uri = `s3://${AWS.bucket}/${key}`;
  const env = {
    ...process.env,
    AWS_ACCESS_KEY_ID: AWS.accessKey,
    AWS_SECRET_ACCESS_KEY: AWS.secretKey,
    AWS_DEFAULT_REGION: AWS.region,
  };
  // Upload without ACL (bucket disallows them).
  execFileSync('aws', ['s3', 'cp', filePath, s3Uri, '--content-type', 'image/png'], {
    env,
    stdio: 'pipe',
  });
  // Generate a 1-hour presigned GET URL — Seedance only needs to fetch once.
  const presigned = execFileSync(
    'aws',
    ['s3', 'presign', s3Uri, '--expires-in', '3600'],
    { env, stdio: 'pipe' },
  )
    .toString()
    .trim();
  console.log(`[upload] ${path.basename(filePath)} → presigned (1h)`);
  return presigned;
}

const SHOTS = {
  'video-hero': {
    ref: 'hero.png',
    prompt: `Premium industrial-premium landing-page ambient loop. Dark workshop interior; the matte black drafting table stays fixed in the lower-third; only atmospheric haze and volumetric dust motes drift slowly left-to-right across the god-ray. Camera: locked-off static for the first 2 seconds, then a very slow dolly-in of roughly 4% magnification over the remaining 4 seconds, ending near-frozen. No lateral movement, no tilt. [0s] Static wide frame, dust particles suspended in the god ray. [2s] Dolly-in begins at 1 unit per second, imperceptible at first. [4s] Faint cyanotype-blue contour lines in the haze glow slightly brighter, then fade. [6s] Hold final frame. Style: cinematic, 24fps, anamorphic 2.39:1, shallow depth of field, Cinestill 800T grade, subtle grain, contemplative slow-cinema pace. Negatives: no camera shake, no handheld, no whip pan, no fast zoom, no cuts, no lens switch, no people entering frame, no text, no UI animation, no glitch, no flashing lights, no color shift, no morphing, no audio, no music, no speech, no subject turning, no glowing orbs, no sci-fi holograms.`,
    aspect_ratio: '16:9',
    duration: 6,
  },
  'video-blueprint': {
    ref: 'feature-blueprint.png',
    prompt: `Macro technical-drawing loop. The cyanotype contour lines and paper fibers stay completely still — nothing on the page moves, ever. Camera: locked-off. Over 6 seconds, execute one extremely slow rack focus from a sharp foreground point in the lower-right to a sharp blueprint intersection in the upper-left, then hold. Bokeh re-shapes softly as focus travels. No translation, no pan. [0s] Foreground engineer's scale tack-sharp, blueprint softly blurred. [3s] Focus plane traveling through the middle of frame. [6s] Blueprint intersection tack-sharp, scale dissolved into bokeh. Style: cinematic macro, 24fps, shallow DOF, f/2.8 bokeh, soft film grain, cold steel-blue grade with one amber highlight unchanging. Negatives: no subject movement, no paper curling, no ink drawing itself in, no animation of lines, no camera shake, no zoom, no pan, no tilt, no people, no hands entering, no text appearing, no UI, no audio, no music, no flash, no color shift.`,
    aspect_ratio: '1:1',
    duration: 6,
  },
  'video-materials': {
    ref: 'feature-materials.png',
    prompt: `Industrial materials hero loop. The materials themselves (aluminum, concrete, carbon fiber, bronze) do not move at all. Only a single slow sweep of warm directional light travels across the composition from camera-left to camera-right over the 6 seconds, revealing surface texture as it goes. Camera: locked-off static. No translation, no zoom, no orbit. [0s] Light at the far left edge, most of frame still in shadow. [3s] Light reaches the center, amber-bronze material catches a highlight. [6s] Light at the far right, shadow returning to the left side. Style: cinematic product still, 24fps, 85mm macro feel, f/4, chiaroscuro, subtle film grain, cold neutral grade with one warm moving highlight, meditative pace. Negatives: no camera movement, no camera shake, no zoom, no pan, no tilt, no dolly, no orbit, no subject movement, no materials sliding, no rotation, no people, no hands, no tools entering frame, no text, no UI, no audio, no flashing lights, no rapid color change, no lens flare pulsing.`,
    aspect_ratio: '16:9',
    duration: 6,
  },
};

async function createTask(refUrl, prompt, aspect, duration) {
  const res = await fetch('https://api.kie.ai/api/v1/jobs/createTask', {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${KIE_KEY}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      model: 'bytedance/seedance-2',
      input: {
        prompt,
        reference_image_urls: [refUrl],
        aspect_ratio: aspect,
        resolution: '1080p',
        duration,
        generate_audio: false,
        nsfw_checker: true,
      },
    }),
  });
  if (!res.ok) {
    const body = await res.text();
    throw new Error(`createTask ${res.status}: ${body.slice(0, 400)}`);
  }
  const data = await res.json();
  if (data.code !== 200 || !data.data?.taskId) {
    throw new Error(`createTask returned ${JSON.stringify(data).slice(0, 400)}`);
  }
  return data.data.taskId;
}

async function pollTask(taskId, slug) {
  const t0 = Date.now();
  while (true) {
    await new Promise((r) => setTimeout(r, 15000));
    const res = await fetch(
      `https://api.kie.ai/api/v1/jobs/recordInfo?taskId=${encodeURIComponent(taskId)}`,
      {
        headers: { Authorization: `Bearer ${KIE_KEY}` },
      },
    );
    if (!res.ok) {
      const body = await res.text();
      console.error(`[poll] ${slug}: HTTP ${res.status} — ${body.slice(0, 200)}`);
      continue;
    }
    const data = await res.json();
    const state = data.data?.state;
    const elapsed = ((Date.now() - t0) / 1000).toFixed(0);
    console.log(`[poll]  ${slug} state=${state} (${elapsed}s)`);
    if (state === 'success') {
      const result = JSON.parse(data.data.resultJson || '{}');
      const url = result.resultUrls?.[0];
      if (!url) throw new Error(`No resultUrl in ${data.data.resultJson}`);
      return url;
    }
    if (state === 'fail') {
      throw new Error(`task failed: ${data.data.failCode} ${data.data.failMsg}`);
    }
    if (Date.now() - t0 > 15 * 60 * 1000) {
      throw new Error(`timeout after 15 min for ${slug}`);
    }
  }
}

async function generateOne(slug, spec) {
  const outPath = path.join(OUT, `${slug}.mp4`);
  try {
    await fs.access(outPath);
    console.log(`[skip]  ${slug} — already exists`);
    return outPath;
  } catch {}

  const refPath = path.join(OUT, spec.ref);
  const refUrl = await uploadToHost(refPath);

  console.log(`[start] ${slug} — seedance`);
  const taskId = await createTask(refUrl, spec.prompt, spec.aspect_ratio, spec.duration);
  console.log(`[task]  ${slug} → ${taskId}`);
  const videoUrl = await pollTask(taskId, slug);
  console.log(`[fetch] ${slug} ← ${videoUrl}`);
  const vRes = await fetch(videoUrl);
  const buf = Buffer.from(await vRes.arrayBuffer());
  await fs.writeFile(outPath, buf);
  console.log(`[done]  ${slug} (${(buf.length / 1024 / 1024).toFixed(1)} MB)`);
  return outPath;
}

async function main() {
  await fs.mkdir(OUT, { recursive: true });
  const want = process.argv.slice(2).length ? process.argv.slice(2) : Object.keys(SHOTS);

  for (const slug of want) {
    if (!SHOTS[slug]) {
      console.error(`[skip] unknown slug ${slug}`);
      continue;
    }
    try {
      await generateOne(slug, SHOTS[slug]);
    } catch (e) {
      console.error(`[fail] ${slug}: ${e.message}`);
    }
  }
  console.log('all done');
}

main();
