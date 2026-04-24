'use client';

import { motion } from 'motion/react';
import { Eyebrow } from './eyebrow';
import { Gauge, Layers, Search, FileBadge, FileSpreadsheet } from 'lucide-react';

const TILES = [
  {
    span: 'md:col-span-2 md:row-span-2',
    icon: Gauge,
    eyebrow: 'Confidence scoring',
    title: 'Every number knows how sure it is.',
    body: 'Each extracted quantity carries a 0.0–1.0 score based on how it was derived. Shoelace-computed areas score 0.9. Guessed values score 0.4. Anything below 0.6 surfaces to the review widget instead of silently rolling into the total.',
    accent: true,
  },
  {
    span: 'md:col-span-2',
    icon: Layers,
    eyebrow: 'DRM',
    title: 'Drawing Rule Mapping',
    body: `layer "steni-beton" → actually brick, use СЕК05 not СЕК04. Rules you write once, applied automatically on every КСС the pipeline generates. Corrections compound.`,
  },
  {
    span: 'md:col-span-1',
    icon: Search,
    eyebrow: 'Price research',
    title: 'Cited, not guessed.',
    body: 'BrightData-proxied scrapes and a Perplexity + Opus loop. Outputs deduped by normalized key, cached in `scraped_price_rows` for reuse.',
  },
  {
    span: 'md:col-span-1',
    icon: FileBadge,
    eyebrow: 'Audit trail',
    title: 'Defensible on every row.',
    body: 'Which rule fired, which price source was used, and the Opus reasoning. Persisted to `kss_audit_trail`.',
  },
  {
    span: 'md:col-span-4',
    icon: FileSpreadsheet,
    eyebrow: 'Exports',
    title: 'The files the estimator actually hands over.',
    body: 'Excel via rust_xlsxwriter matching ОБРАЗЕЦ 9.1, PDF via the internal report crate, CSV for anything else. One endpoint per format, streaming, no "please wait while we build your file" screen.',
    wide: true,
  },
];

export function Bento() {
  return (
    <section className="relative py-24 md:py-36">
      <div className="mx-auto max-w-7xl px-6">
        <div className="max-w-2xl mb-14">
          <Eyebrow className="mb-4 block">Under the hood</Eyebrow>
          <h2 className="text-[clamp(1.75rem,4vw,3rem)] font-semibold leading-[1.08] tracking-tight">
            Confidence on every number, not just the ones the demo shows.
          </h2>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 md:auto-rows-[220px]">
          {TILES.map((tile, i) => (
            <motion.div
              key={tile.title}
              initial={{ opacity: 0, y: 14 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-60px' }}
              transition={{ duration: 0.55, delay: i * 0.06 }}
              className={`${tile.span} relative overflow-hidden rounded-2xl border border-[var(--color-hairline)] bg-[var(--color-bg-raised)] p-6 flex flex-col justify-between ${
                tile.accent
                  ? 'bg-gradient-to-br from-[var(--color-surface-hi)] to-[var(--color-bg-raised)]'
                  : ''
              }`}
            >
              {tile.accent && (
                <div
                  aria-hidden
                  className="absolute inset-0 opacity-[0.25] pointer-events-none"
                  style={{
                    backgroundImage: `url('/assets/gen/feature-data.png')`,
                    backgroundSize: 'cover',
                    backgroundPosition: 'center',
                  }}
                />
              )}
              <div className="relative">
                <div className="mb-4 inline-flex h-8 w-8 items-center justify-center rounded-md border border-[var(--color-hairline-hi)] bg-[var(--color-surface)]/80 backdrop-blur">
                  <tile.icon className="h-4 w-4 text-[var(--color-amber)]" strokeWidth={1.75} />
                </div>
                <div className="font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.18em] text-[var(--color-fg-quaternary)] mb-2">
                  {tile.eyebrow}
                </div>
                <h3 className="text-[17px] font-semibold leading-snug tracking-tight">{tile.title}</h3>
              </div>
              <p className="relative mt-4 text-[13px] leading-relaxed text-[var(--color-fg-secondary)]">
                {tile.body}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
