'use client';

import { motion, useReducedMotion } from 'motion/react';
import { Eyebrow } from './eyebrow';

const NODES = [
  { id: 'upload', label: 'Upload', sub: 'DXF / DWG / PDF' },
  { id: 'api', label: 'kcc-api', sub: 'Axum · 21 mig.' },
  { id: 'db', label: 'Postgres 16', sub: 'state + KCC rows' },
  { id: 'worker', label: 'kcc-worker', sub: 'DXF + AI KSS' },
  { id: 'store', label: 'S3 · Redis', sub: 'analysis + queues' },
  { id: 'out', label: 'КСС', sub: 'Excel · PDF · CSV' },
];

export function Pipeline() {
  const reduceMotion = useReducedMotion();

  return (
    <section
      id="pipeline"
      className="relative py-24 md:py-36 border-y border-[var(--color-hairline)] bg-[var(--color-bg)]"
    >
      <div aria-hidden className="absolute inset-0 grid-bg opacity-[0.08] pointer-events-none" />
      <div className="relative mx-auto max-w-7xl px-6">
        <div className="max-w-2xl mb-14">
          <Eyebrow className="mb-4 block">The pipeline</Eyebrow>
          <h2 className="text-[clamp(1.75rem,4vw,3rem)] font-semibold leading-[1.08] tracking-tight">
            Four services, one pipeline.
          </h2>
          <p className="mt-5 text-[15px] leading-relaxed text-[var(--color-fg-secondary)]">
            Every stage runs in isolation — the API takes uploads, the worker handles the heavy lifting,
            Postgres keeps state, Redis runs the queue.
          </p>
        </div>

        <div className="relative">
          {/* Nodes */}
          <div className="grid grid-cols-2 md:grid-cols-6 gap-4 relative">
            {NODES.map((node, i) => (
              <motion.div
                key={node.id}
                initial={{ opacity: 0, y: 10 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, margin: '-50px' }}
                transition={{ duration: 0.5, delay: i * 0.08 }}
                className="relative rounded-xl border border-[var(--color-hairline-hi)] bg-[var(--color-surface)]/80 backdrop-blur p-4 h-28 flex flex-col justify-between"
              >
                <span className="font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.18em] text-[var(--color-fg-quaternary)]">
                  step {String(i + 1).padStart(2, '0')}
                </span>
                <div>
                  <div className="text-[14px] font-semibold text-[var(--color-fg)]">{node.label}</div>
                  <div className="mt-0.5 text-[11px] font-[family-name:var(--font-mono)] text-[var(--color-fg-tertiary)]">
                    {node.sub}
                  </div>
                </div>
                {/* Amber dot tracer */}
                {!reduceMotion && (
                  <motion.span
                    className="absolute -top-1 left-1/2 h-2 w-2 rounded-full bg-[var(--color-amber)]"
                    initial={{ opacity: 0 }}
                    whileInView={{ opacity: [0, 1, 0] }}
                    viewport={{ once: false, margin: '-100px' }}
                    transition={{ duration: 3, delay: i * 0.5, repeat: Infinity, repeatDelay: 3 }}
                  />
                )}
              </motion.div>
            ))}
          </div>

          {/* Connecting line (only desktop) */}
          <svg
            aria-hidden
            className="hidden md:block absolute left-0 right-0 top-1/2 -translate-y-1/2 w-full h-px pointer-events-none"
            viewBox="0 0 100 1"
            preserveAspectRatio="none"
          >
            <motion.line
              x1="0"
              y1="0.5"
              x2="100"
              y2="0.5"
              stroke="var(--color-amber)"
              strokeWidth="0.5"
              strokeOpacity="0.25"
              strokeDasharray="0.4 1"
              initial={{ pathLength: 0 }}
              whileInView={{ pathLength: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 1.2, ease: 'easeOut' }}
            />
          </svg>
        </div>

        {/* Code-style footnote */}
        <div className="mt-14 flex flex-wrap gap-x-8 gap-y-2 font-[family-name:var(--font-mono)] text-[11px] text-[var(--color-fg-quaternary)]">
          <span>
            <span className="text-[var(--color-amber)]/70">//</span> median end-to-end: 2m 38s
          </span>
          <span>
            <span className="text-[var(--color-amber)]/70">//</span> p95 time in worker: 1m 52s
          </span>
          <span>
            <span className="text-[var(--color-amber)]/70">//</span> sqlx::migrate! on API boot
          </span>
        </div>
      </div>
    </section>
  );
}
