#!/usr/bin/env node
// Generate landing-page videos via KIE.ai (bytedance/seedance-2) in parallel.
// Uploads reference images via S3 presigned URLs (bucket disallows ACLs).

import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import { VIDEO_SHOTS } from './prompts.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUT = path.resolve(__dirname, '../public/assets/gen');

const KIE_KEY = process.env.KIE_API_KEY || '92d86ec39c86d80da972c68ef8c74a86';

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
  const key = `marketing-refs/${Date.now()}-${Math.random().toString(36).slice(2, 8)}-${path.basename(filePath)}`;
  const s3Uri = `s3://${AWS.bucket}/${key}`;
  const env = {
    ...process.env,
    AWS_ACCESS_KEY_ID: AWS.accessKey,
    AWS_SECRET_ACCESS_KEY: AWS.secretKey,
    AWS_DEFAULT_REGION: AWS.region,
  };
  execFileSync('aws', ['s3', 'cp', filePath, s3Uri, '--content-type', 'image/png'], { env, stdio: 'pipe' });
  const presigned = execFileSync('aws', ['s3', 'presign', s3Uri, '--expires-in', '3600'], { env, stdio: 'pipe' })
    .toString()
    .trim();
  return presigned;
}

async function createTask(refUrl, spec) {
  const res = await fetch('https://api.kie.ai/api/v1/jobs/createTask', {
    method: 'POST',
    headers: { Authorization: `Bearer ${KIE_KEY}`, 'Content-Type': 'application/json' },
    body: JSON.stringify({
      model: 'bytedance/seedance-2',
      input: {
        prompt: spec.prompt,
        reference_image_urls: [refUrl],
        aspect_ratio: spec.aspect_ratio,
        resolution: '1080p',
        duration: spec.duration,
        generate_audio: false,
        nsfw_checker: true,
      },
    }),
  });
  if (!res.ok) throw new Error(`createTask ${res.status}: ${(await res.text()).slice(0, 400)}`);
  const data = await res.json();
  if (data.code !== 200 || !data.data?.taskId) throw new Error(`createTask ${JSON.stringify(data).slice(0, 400)}`);
  return data.data.taskId;
}

async function pollTask(taskId, slug) {
  const t0 = Date.now();
  while (true) {
    await new Promise((r) => setTimeout(r, 15000));
    try {
      const res = await fetch(`https://api.kie.ai/api/v1/jobs/recordInfo?taskId=${encodeURIComponent(taskId)}`, {
        headers: { Authorization: `Bearer ${KIE_KEY}` },
      });
      if (!res.ok) continue;
      const data = await res.json();
      const state = data.data?.state;
      const elapsed = ((Date.now() - t0) / 1000).toFixed(0);
      console.log(`[poll]  ${slug} state=${state} (${elapsed}s)`);
      if (state === 'success') {
        const r = JSON.parse(data.data.resultJson || '{}');
        if (!r.resultUrls?.[0]) throw new Error('no resultUrls');
        return r.resultUrls[0];
      }
      if (state === 'fail') throw new Error(`fail ${data.data.failCode}: ${data.data.failMsg}`);
      if (Date.now() - t0 > 20 * 60 * 1000) throw new Error('poll timeout 20min');
    } catch (e) {
      console.error(`[poll]  ${slug} error: ${e.message}`);
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
  try {
    await fs.access(refPath);
  } catch {
    throw new Error(`missing reference image ${refPath}`);
  }
  const refUrl = await uploadToHost(refPath);
  console.log(`[upload] ${slug} ref uploaded`);
  const taskId = await createTask(refUrl, spec);
  console.log(`[task]  ${slug} → ${taskId}`);
  const videoUrl = await pollTask(taskId, slug);
  const vRes = await fetch(videoUrl);
  const buf = Buffer.from(await vRes.arrayBuffer());
  await fs.writeFile(outPath, buf);
  console.log(`[done]  ${slug} (${(buf.length / 1024 / 1024).toFixed(1)} MB)`);
  return outPath;
}

async function main() {
  await fs.mkdir(OUT, { recursive: true });
  const want = process.argv.slice(2).length ? process.argv.slice(2) : Object.keys(VIDEO_SHOTS);
  // Run all in parallel. Seedance queues on their side.
  const results = await Promise.allSettled(
    want.map(async (slug) => {
      if (!VIDEO_SHOTS[slug]) throw new Error(`unknown slug ${slug}`);
      return generateOne(slug, VIDEO_SHOTS[slug]);
    }),
  );
  for (const [i, r] of results.entries()) {
    if (r.status === 'rejected') console.error(`[fail] ${want[i]}: ${r.reason.message}`);
  }
  console.log('all done');
}

main();
