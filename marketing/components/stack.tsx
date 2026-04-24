'use client';

import { motion } from 'motion/react';
import { Eyebrow } from './eyebrow';

const ROWS = [
  {
    name: 'Rust workspace, 9 crates',
    desc: 'kcc-api, kcc-worker, kcc-core, kcc-dxf, kcc-report, erp-core, erp-boq, erp-costs, erp-assemblies.',
  },
  {
    name: 'Axum + sqlx + Tokio',
    desc: 'HTTP on the API, background jobs on the worker, compile-time-checked SQL everywhere.',
  },
  {
    name: 'Postgres 16',
    desc: '21 sqlx migrations, applied automatically on API startup. Every КСС, every correction, every audit row.',
  },
  {
    name: 'Redis 7',
    desc: 'BullMQ-style queues for drawing parsing and the AI KSS phases (research → review → generate).',
  },
  {
    name: 'Next.js 15 + Tailwind v4',
    desc: 'Operator UI. Server components where they help, client components where they don\'t.',
  },
  {
    name: 'S3',
    desc: 'User-scoped paths for originals and analysis snapshots. No cross-tenant anything.',
  },
  {
    name: 'BrightData web unlocker',
    desc: 'Supplier price scraping behind a rotating residential proxy.',
  },
  {
    name: 'OpenRouter',
    desc: 'Perplexity sonar-pro for price research, Claude Opus 4.6 for КСС generation, swappable per-job.',
  },
];

export function Stack() {
  return (
    <section id="stack" className="relative py-24 md:py-36">
      <div className="mx-auto max-w-7xl px-6">
        <div className="max-w-2xl mb-14">
          <Eyebrow className="mb-4 block">Stack</Eyebrow>
          <h2 className="text-[clamp(1.75rem,4vw,3rem)] font-semibold leading-[1.08] tracking-tight">
            Rust on the hot path. Everything else where it belongs.
          </h2>
        </div>

        <div className="rounded-2xl border border-[var(--color-hairline)] bg-[var(--color-bg-raised)] overflow-hidden">
          <ul className="divide-y divide-[var(--color-hairline)]">
            {ROWS.map((row, i) => (
              <motion.li
                key={row.name}
                initial={{ opacity: 0, x: -8 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true, margin: '-30px' }}
                transition={{ duration: 0.45, delay: i * 0.04 }}
                className="group grid grid-cols-1 md:grid-cols-[260px_1fr] px-6 py-5 hover:bg-[var(--color-surface)]/50 transition-colors"
              >
                <span className="font-[family-name:var(--font-mono)] text-[12.5px] uppercase tracking-[0.1em] text-[var(--color-fg)] flex items-center gap-2">
                  <span className="inline-block h-1 w-1 rounded-full bg-[var(--color-amber)]/60 group-hover:bg-[var(--color-amber)] transition-colors" />
                  {row.name}
                </span>
                <span className="mt-1 md:mt-0 text-[13.5px] leading-relaxed text-[var(--color-fg-secondary)]">
                  {row.desc}
                </span>
              </motion.li>
            ))}
          </ul>
        </div>
      </div>
    </section>
  );
}
