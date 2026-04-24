'use client';

import { motion, useScroll, useTransform } from 'motion/react';
import { useRef, useState, useEffect } from 'react';
import { Eyebrow } from './eyebrow';
import { Upload, ScanLine, Banknote, FileCheck2, ArrowRight } from 'lucide-react';
import { cn } from '@/lib/cn';

const PHASES = [
  {
    key: 'upload',
    label: 'Upload',
    sub: 'DXF · DWG · PDF',
    body: 'The drawing arrives. SHA-256 dedupe, S3-stored, queued for the worker.',
    image: '/assets/gen/phase-1-upload.png',
    icon: Upload,
    stack: ['kcc-api · Axum', 'Postgres · jobs', 'Redis · kcc:jobs'],
  },
  {
    key: 'parse',
    label: 'Parse',
    sub: 'layers · entities · features',
    body: 'DXF parsed, dimensions linked to geometry, features extracted, layers mapped to SEK groups.',
    image: '/assets/gen/phase-2-parse.png',
    icon: ScanLine,
    stack: ['kcc-worker', 'kcc-dxf · nom', 'spatial index'],
  },
  {
    key: 'price',
    label: 'Price',
    sub: 'scraped + researched',
    body: 'Quantities matched to СЕК codes. Prices pulled from the scrape cache or researched live via Perplexity + Opus.',
    image: '/assets/gen/phase-3-price.png',
    icon: Banknote,
    stack: ['BrightData proxy', 'OpenRouter sonar-pro', 'Opus 4.6 generation'],
  },
  {
    key: 'export',
    label: 'Export',
    sub: 'КСС · Excel · PDF · CSV',
    body: 'Grouped by СЕК, labour/material/overhead split, audit trail persisted. One endpoint per format, streaming.',
    image: '/assets/gen/phase-4-export.png',
    icon: FileCheck2,
    stack: ['kcc-report', 'rust_xlsxwriter', 'ОБРАЗЕЦ 9.1 compatible'],
  },
];

export function Pipeline() {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const { scrollYProgress } = useScroll({
    target: containerRef,
    offset: ['start end', 'end start'],
  });

  const [activeIdx, setActiveIdx] = useState(0);

  useEffect(() => {
    return scrollYProgress.on('change', (p) => {
      // Map the middle 60% of the section to the 4 phases
      const local = Math.min(Math.max((p - 0.2) / 0.6, 0), 0.9999);
      setActiveIdx(Math.floor(local * PHASES.length));
    });
  }, [scrollYProgress]);

  const bgOpacity = useTransform(scrollYProgress, [0, 0.2, 0.8, 1], [0, 1, 1, 0]);

  return (
    <section
      id="pipeline"
      ref={containerRef}
      className="relative border-y border-[var(--color-hairline)] bg-[var(--color-bg)]"
    >
      <motion.div
        aria-hidden
        style={{ opacity: bgOpacity }}
        className="absolute inset-0 grid-bg pointer-events-none"
      />
      <div aria-hidden className="absolute inset-0 pointer-events-none" style={{ background: 'radial-gradient(ellipse at 20% 20%, rgba(184,115,51,0.08), transparent 50%)' }} />

      <div className="relative mx-auto max-w-7xl px-6 pt-24 md:pt-32 pb-24 md:pb-32">
        <div className="max-w-2xl mb-14">
          <Eyebrow className="mb-4 block">The pipeline</Eyebrow>
          <h2 className="text-[clamp(1.75rem,4vw,3rem)] font-semibold leading-[1.08] tracking-tight">
            Four services, one pipeline.
          </h2>
          <p className="mt-5 text-[15px] leading-relaxed text-[var(--color-fg-secondary)]">
            Every stage runs in isolation — the API takes uploads, the worker handles the heavy lifting,
            Postgres keeps state, Redis runs the queue. Scroll to walk through it.
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-[1fr_1.1fr] gap-10 lg:gap-16 items-start">
          {/* Left rail — phase list */}
          <ol className="relative">
            <div aria-hidden className="absolute left-4 top-1 bottom-1 w-px bg-[var(--color-hairline)]" />
            <motion.div
              aria-hidden
              className="absolute left-4 top-1 w-px bg-[var(--color-amber)]"
              style={{ height: useTransform(scrollYProgress, [0.2, 0.8], ['0%', '100%']) }}
            />
            {PHASES.map((p, i) => {
              const active = i === activeIdx;
              return (
                <li key={p.key} className="relative pl-14 py-5">
                  <div
                    className={cn(
                      'absolute left-0 top-5 h-8 w-8 rounded-full border flex items-center justify-center transition-all',
                      active
                        ? 'bg-[var(--color-amber)] border-[var(--color-amber)] text-[var(--color-bg)] scale-110 shadow-[0_0_0_6px_rgba(184,115,51,0.12)]'
                        : 'bg-[var(--color-surface)] border-[var(--color-hairline-hi)] text-[var(--color-fg-tertiary)]',
                    )}
                  >
                    <p.icon className="h-3.5 w-3.5" strokeWidth={2} />
                  </div>
                  <div className="flex items-baseline gap-3">
                    <span className="font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.16em] text-[var(--color-fg-quaternary)]">
                      step {String(i + 1).padStart(2, '0')}
                    </span>
                    <span
                      className={cn(
                        'font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.14em] text-[var(--color-fg-tertiary)]',
                      )}
                    >
                      {p.sub}
                    </span>
                  </div>
                  <h3 className={cn('mt-1 text-[20px] font-semibold tracking-tight transition-colors', active ? 'text-[var(--color-fg)]' : 'text-[var(--color-fg-tertiary)]')}>
                    {p.label}
                  </h3>
                  <p className={cn('mt-2 max-w-md text-[13.5px] leading-relaxed transition-colors', active ? 'text-[var(--color-fg-secondary)]' : 'text-[var(--color-fg-quaternary)]')}>
                    {p.body}
                  </p>
                  <ul className="mt-3 flex flex-wrap gap-x-4 gap-y-1 font-[family-name:var(--font-mono)] text-[11px] text-[var(--color-fg-quaternary)]">
                    {p.stack.map((s) => (
                      <li key={s} className="flex items-center gap-1.5">
                        <span className={cn('inline-block h-1 w-1 rounded-full', active ? 'bg-[var(--color-amber)]' : 'bg-[var(--color-hairline-hi)]')} />
                        {s}
                      </li>
                    ))}
                  </ul>
                </li>
              );
            })}
          </ol>

          {/* Right — sticky phase image, swaps with scroll */}
          <div className="lg:sticky lg:top-24 lg:h-[min(72vh,640px)]">
            <div className="relative h-full w-full rounded-2xl border border-[var(--color-hairline)] bg-[var(--color-bg-raised)] overflow-hidden aspect-square lg:aspect-auto">
              {PHASES.map((p, i) => (
                <motion.div
                  key={p.key}
                  aria-hidden
                  initial={false}
                  animate={{
                    opacity: i === activeIdx ? 1 : 0,
                    scale: i === activeIdx ? 1 : 1.05,
                  }}
                  transition={{ duration: 0.75, ease: [0.22, 0.61, 0.36, 1] }}
                  className="absolute inset-0"
                  style={{
                    backgroundImage: `url('${p.image}')`,
                    backgroundSize: 'cover',
                    backgroundPosition: 'center',
                  }}
                />
              ))}
              <div aria-hidden className="absolute inset-0 bg-gradient-to-t from-[var(--color-bg-raised)]/60 to-transparent" />

              <div className="absolute top-5 left-5 right-5 flex items-center justify-between">
                <span className="inline-flex items-center gap-2 px-2.5 py-1 rounded-full bg-[var(--color-bg)]/60 backdrop-blur border border-[var(--color-hairline-hi)] font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.18em] text-[var(--color-fg-secondary)]">
                  <span className="h-1.5 w-1.5 rounded-full bg-[var(--color-amber)] animate-pulse" />
                  {String(activeIdx + 1).padStart(2, '0')} · {PHASES[activeIdx].label}
                </span>
                <div className="flex gap-1">
                  {PHASES.map((_, i) => (
                    <span
                      key={i}
                      className={cn('h-1 rounded-full transition-all', i === activeIdx ? 'w-6 bg-[var(--color-amber)]' : 'w-3 bg-[var(--color-hairline-hi)]')}
                    />
                  ))}
                </div>
              </div>

              <div className="absolute bottom-5 left-5 right-5 flex items-end justify-between">
                <div>
                  <p className="font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.18em] text-[var(--color-fg-quaternary)]">
                    {PHASES[activeIdx].sub}
                  </p>
                  <p className="mt-1 text-[14px] text-[var(--color-fg-secondary)] max-w-sm">
                    {PHASES[activeIdx].body}
                  </p>
                </div>
                <span className="shrink-0 ml-4 h-8 w-8 rounded-md border border-[var(--color-hairline-hi)] bg-[var(--color-bg)]/60 backdrop-blur flex items-center justify-center">
                  <ArrowRight className="h-3.5 w-3.5 text-[var(--color-amber)]" strokeWidth={1.75} />
                </span>
              </div>
            </div>
          </div>
        </div>

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
